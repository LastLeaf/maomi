use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut, UnsafeCell};
use std::fmt;

pub struct NodeBorrowError {
    msg: &'static str
}
impl fmt::Debug for NodeBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl fmt::Display for NodeBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for NodeBorrowError {}

pub(crate) struct MeCell<T> {
    ctx: Rc<RefCell<()>>,
    content: UnsafeCell<T>,
}

impl<T> MeCell<T> {
    pub(crate) fn new_group(content: T) -> Self {
        Self {
            ctx: Rc::new(RefCell::new(())),
            content: UnsafeCell::new(content),
        }
    }

    pub(crate) fn another<U>(&self, content: U) -> MeCell<U> {
        MeCell {
            ctx: self.ctx.clone(),
            content: UnsafeCell::new(content),
        }
    }

    pub(crate) fn borrow(&self) -> Ref<T> {
        match self.try_borrow() {
            Ok(r) => r,
            Err(e) => panic!(e.msg),
        }
    }

    pub(crate) fn try_borrow<'a>(&'a self) -> Result<Ref<'a, T>, NodeBorrowError> {
        let ptr = self.content.get();
        match self.ctx.try_borrow() {
            Ok(r) => Ok(Ref::map(r, |_| unsafe { &*ptr })),
            Err(_) => Err(NodeBorrowError { msg: "Node has been mutably borrowed" }),
        }
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<T> {
        match self.try_borrow_mut() {
            Ok(r) => r,
            Err(e) => panic!(e.msg),
        }
    }

    pub(crate) fn try_borrow_mut<'a>(&'a self) -> Result<RefMut<'a, T>, NodeBorrowError> {
        let ptr = self.content.get();
        match self.ctx.try_borrow_mut() {
            Ok(r) => Ok(RefMut::map(r, |_| unsafe { &mut *ptr })),
            Err(_) => Err(NodeBorrowError { msg: "Node has been borrowed" }),
        }
    }

    pub(crate) unsafe fn deref_unsafe(&self) -> &T {
        &*self.content.get()
    }

    pub(crate) unsafe fn deref_mut_unsafe(&self) -> &mut T {
        &mut *self.content.get()
    }

    pub(crate) unsafe fn deref_unsafe_with_lifetime<'a, 'b>(&'a self) -> &'b T {
        &*self.content.get()
    }

    pub(crate) unsafe fn deref_mut_unsafe_with_lifetime<'a, 'b>(&'a self) -> &'b mut T {
        &mut *self.content.get()
    }
}
