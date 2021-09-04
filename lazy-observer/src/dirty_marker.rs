use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::hash::*;
use std::pin::Pin;
use std::rc::Rc;

pub(crate) struct DirtyMarker {
    dirty: Cell<bool>,
    send: RefCell<HashSet<Pin<Rc<Self>>>>,
    listen: RefCell<HashSet<Pin<Rc<Self>>>>,
}

impl DirtyMarker {
    pub(crate) fn new(dirty: bool) -> Self {
        Self {
            dirty: Cell::new(dirty),
            send: RefCell::new(HashSet::new()),
            listen: RefCell::new(HashSet::new()),
        }
    }
    pub(crate) fn destroy(&self) {
        self.send.borrow_mut().clear();
    }
    pub(crate) fn mark_dirty(&self) {
        if !self.dirty.replace(true) {
            for c in self.send.borrow().iter() {
                c.mark_dirty();
            }
        }
    }
    pub(crate) fn mark_connected_dirty(&self) {
        for c in self.send.borrow().iter() {
            c.mark_dirty();
        }
    }
    pub(crate) fn clear_dirty(&self) -> bool {
        self.dirty.replace(false)
    }
    pub(crate) fn update_listen_targets(s: &Pin<Rc<Self>>, targets: HashSet<Pin<Rc<Self>>>) {
        let mut old = s.listen.borrow_mut();
        let remove = old.difference(&targets);
        for target in remove {
            target.send.borrow_mut().remove(s);
        }
        let add = targets.difference(&old);
        for target in add {
            target.send.borrow_mut().insert(s.clone());
        }
        *old = targets;
    }
}

impl PartialEq for DirtyMarker {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl Eq for DirtyMarker {}

impl Hash for DirtyMarker {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const _ as usize).hash(state);
    }
}
