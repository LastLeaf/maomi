use std::cell::{Ref, RefCell};
use std::cmp::*;
use std::fmt;
use std::hash::*;
use std::ops::*;
use std::pin::Pin;
use std::rc::Rc;

use super::dirty_marker::DirtyMarker;
use super::{exec_updater, notify_updater};

pub struct Lazy<T> {
    data: RefCell<Option<T>>,
    updater: Rc<dyn Fn() -> T>,
    dirty: Pin<Rc<DirtyMarker>>,
}

impl<T> Lazy<T> {
    #[inline]
    pub fn new<F: 'static + Fn() -> T>(updater: F) -> Self {
        let updater = Rc::new(updater);
        let dirty = Rc::pin(DirtyMarker::new(true));
        Self {
            data: RefCell::new(None),
            updater,
            dirty,
        }
    }
    #[inline]
    pub fn check_update(&self) {
        if self.dirty.clear_dirty() {
            match self.data.try_borrow_mut() {
                Ok(mut x) => *x = Some(exec_updater(&self.dirty, &self.updater)),
                Err(_) => {}
            }
        }
    }
    #[inline]
    pub fn get_ref(&self) -> Ref<T> {
        self.check_update();
        notify_updater(&self.dirty);
        Ref::map(self.data.borrow(), |x| x.as_ref().unwrap())
    }
}

impl<T: Clone> Lazy<T> {
    #[inline]
    pub fn get(&self) -> T {
        self.check_update();
        notify_updater(&self.dirty);
        self.data.borrow().as_ref().unwrap().clone()
    }
}

impl<T> Drop for Lazy<T> {
    fn drop(&mut self) {
        self.dirty.destroy();
    }
}

impl<T: PartialEq> PartialEq for Lazy<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.get_ref() == *other.get_ref()
    }
}

impl<T: Eq> Eq for Lazy<T> {}

impl<T: PartialOrd> PartialOrd for Lazy<T> {
    #[inline]
    fn partial_cmp(&self, other: &Lazy<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&*self.get_ref(), &*other.get_ref())
    }
    #[inline]
    fn lt(&self, other: &Lazy<T>) -> bool {
        PartialOrd::lt(&*self.get_ref(), &*other.get_ref())
    }
    #[inline]
    fn le(&self, other: &Lazy<T>) -> bool {
        PartialOrd::le(&*self.get_ref(), &*other.get_ref())
    }
    #[inline]
    fn ge(&self, other: &Lazy<T>) -> bool {
        PartialOrd::ge(&*self.get_ref(), &*other.get_ref())
    }
    #[inline]
    fn gt(&self, other: &Lazy<T>) -> bool {
        PartialOrd::gt(&*self.get_ref(), &*other.get_ref())
    }
}

impl<T: Ord> Ord for Lazy<T> {
    #[inline]
    fn cmp(&self, other: &Lazy<T>) -> Ordering {
        Ord::cmp(&*self.get_ref(), &*other.get_ref())
    }
}

impl<T: Hash> Hash for Lazy<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self.get_ref()).hash(state);
    }
}

impl<T: Clone> Clone for Lazy<T> {
    #[inline]
    fn clone(&self) -> Lazy<T> {
        let updater = self.updater.clone();
        let dirty = Rc::pin(DirtyMarker::new(false));
        Self {
            data: RefCell::new(None),
            updater,
            dirty,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Lazy<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.get_ref(), f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Lazy<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.get_ref(), f)
    }
}
