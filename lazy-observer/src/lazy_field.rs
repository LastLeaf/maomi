use std::ops::*;
use std::pin::Pin;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

use super::{notify_updater, exec_field_updater};
use super::dirty_marker::DirtyMarker;

pub struct LazyField<S, T> {
    data: RefCell<Option<T>>,
    updater: Rc<dyn for<'r> Fn(&'r S) -> T>,
    dirty: Pin<Rc<DirtyMarker>>,
}

impl<S, T> LazyField<S, T> {
    #[inline]
    pub fn new<F: 'static + for<'r> Fn(&'r S) -> T>(updater: F) -> Self {
        let updater = Rc::new(updater);
        let dirty = Rc::pin(DirtyMarker::new(true));
        Self { data: RefCell::new(None), updater, dirty }
    }
    #[inline]
    pub fn check_update(&self, s: &S) {
        if self.dirty.clear_dirty() {
            match self.data.try_borrow_mut() {
                Ok(mut x) => *x = Some(exec_field_updater(&self.dirty, s, &self.updater)),
                Err(_) => { },
            }
        }
    }
    #[inline]
    pub fn get_ref(&self, s: &S) -> Ref<T> {
        self.check_update(s);
        notify_updater(&self.dirty);
        Ref::map(self.data.borrow(), |x| x.as_ref().unwrap())
    }
}

impl<S, T: Clone> LazyField<S, T> {
    #[inline]
    pub fn get(&self, s: &S) -> T {
        self.check_update(s);
        notify_updater(&self.dirty);
        self.data.borrow().as_ref().unwrap().clone()
    }
}

impl<S, T> Drop for LazyField<S, T> {
    fn drop(&mut self) {
        self.dirty.destroy();
    }
}

impl<S, T: Clone> Clone for LazyField<S, T> {
    #[inline]
    fn clone(&self) -> LazyField<S, T> {
        let updater = self.updater.clone();
        let dirty = Rc::pin(DirtyMarker::new(false));
        Self { data: RefCell::new(None), updater, dirty }
    }
}
