use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::fmt;
use std::cell::UnsafeCell;

pub struct MeBorrowError {
    msg: &'static str
}
impl fmt::Debug for MeBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl fmt::Display for MeBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for MeBorrowError {}

struct Ctx {
    is_mut: UnsafeCell<bool>,
    count: UnsafeCell<usize>,
}

pub struct MeCell<T> {
    ctx: Rc<Ctx>,
    content: UnsafeCell<T>,
}

impl<T> MeCell<T> {
    pub fn new_group(content: T) -> Self {
        Self {
            ctx: Rc::new(Ctx {
                is_mut: UnsafeCell::new(false),
                count: UnsafeCell::new(0),
            }),
            content: UnsafeCell::new(content),
        }
    }
    pub fn borrow(&self) -> MeRef<T> {
        unsafe {
            if *self.ctx.is_mut.get() { panic!("MeCell has been mutably borrowed") };
            *self.ctx.count.get() += 1;
        }
        MeRef { handle: MeRefHandle { is_source: true, ctx: &self.ctx }, content: unsafe { &*self.content.get() } }
    }
    pub fn try_borrow(&self) -> Result<MeRef<T>, MeBorrowError> {
        unsafe {
            if *self.ctx.is_mut.get() { return Err(MeBorrowError { msg: "MeCell has been mutably borrowed" }) };
            *self.ctx.count.get() += 1;
        }
        Ok(MeRef { handle: MeRefHandle { is_source: true, ctx: &self.ctx }, content: unsafe { &*self.content.get() } })
    }
    pub fn borrow_mut(&self) -> MeRefMut<T> {
        unsafe {
            if *self.ctx.is_mut.get() || *self.ctx.count.get() > 0 { panic!("MeCell has been borrowed") };
            *self.ctx.is_mut.get() = true;
        }
        MeRefMut { handle: MeRefMutHandle { is_source: true, ctx: &self.ctx }, content: unsafe { &mut *self.content.get() } }
    }
    pub fn try_borrow_mut(&self) -> Result<MeRefMut<T>, MeBorrowError> {
        unsafe {
            if *self.ctx.is_mut.get() || *self.ctx.count.get() > 0 { return Err(MeBorrowError { msg: "MeCell has been borrowed" }) };
            *self.ctx.is_mut.get() = true;
        }
        Ok(MeRefMut { handle: MeRefMutHandle { is_source: true, ctx: &self.ctx }, content: unsafe { &mut *self.content.get() } })
    }
    pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'b MeRef<'a, U>) -> MeRef<'b, T> {
        source.another(self)
    }
    pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'b mut MeRefMut<'a, U>) -> MeRefMut<'b, T> {
        source.another_mut(self)
    }
    pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, 'c, U>(&'c self, source: &'b mut MeRefMut<'a, U>) -> MeRefMut<'c, T> {
        source.another_mut_unsafe(self)
    }
    pub fn borrow_with_handle<'a: 'b, 'b>(&'b self, source: &'b MeRefHandle<'a>) -> MeRef<'b, T> {
        source.another(self)
    }
    pub fn borrow_mut_with_handle<'a: 'b, 'b>(&'b self, source: &'b mut MeRefMutHandle<'a>) -> MeRefMut<'b, T> {
        source.another_mut(self)
    }
    pub unsafe fn borrow_mut_unsafe_with_handle<'a: 'b, 'b, 'c>(&'c self, source: &'b mut MeRefMutHandle<'a>) -> MeRefMut<'c, T> {
        source.another_mut_unsafe(self)
    }
}

pub struct MeRefHandle<'a> {
    is_source: bool,
    ctx: &'a Rc<Ctx>,
}
impl<'a> MeRefHandle<'a> {
    pub fn another<'b, U>(&'b self, another: &'b MeCell<U>) -> MeRef<'b, U> where 'a: 'b {
        if !Rc::ptr_eq(&another.ctx, self.ctx) {
            panic!("A MeCell can only be borrowed with another MeCell in the same group");
        }
        MeRef { handle: MeRefHandle { is_source: false, ctx: self.ctx }, content: unsafe { &*another.content.get() } }
    }
}
impl<'a> Drop for MeRefHandle<'a> {
    fn drop(&mut self) {
        if !self.is_source { return };
        let c = self.ctx.count.get();
        unsafe {
            *c -= 1;
        }
    }
}

pub struct MeRefMutHandle<'a> {
    is_source: bool,
    ctx: &'a Rc<Ctx>
}
impl<'a> MeRefMutHandle<'a> {
    pub fn entrance<U>(&mut self, content: U) -> MeCell<U> {
        MeCell {
            ctx: self.ctx.clone(),
            content: UnsafeCell::new(content),
        }
    }
    pub fn another<'b, U>(&'b self, another: &'b MeCell<U>) -> MeRef<'b, U> where 'a: 'b {
        if !Rc::ptr_eq(&another.ctx, self.ctx) {
            panic!("A MeCell can only be borrowed with another MeCell in the same group");
        }
        MeRef { handle: MeRefHandle { is_source: false, ctx: self.ctx }, content: unsafe { &*another.content.get() } }
    }
    pub fn another_mut<'b, U>(&'b mut self, another: &'b MeCell<U>) -> MeRefMut<'b, U> where 'a: 'b {
        if !Rc::ptr_eq(&another.ctx, self.ctx) {
            panic!("A MeCell can only be borrowed with another MeCell in the same group");
        }
        MeRefMut { handle: MeRefMutHandle { is_source: false, ctx: self.ctx }, content: unsafe { &mut *another.content.get() } }
    }
    pub unsafe fn another_mut_unsafe<'b, 'c, U>(&'b mut self, another: &'c MeCell<U>) -> MeRefMut<'c, U> where 'a: 'b {
        if !Rc::ptr_eq(&another.ctx, self.ctx) {
            panic!("A MeCell can only be borrowed with another MeCell in the same group");
        }
        MeRefMut { handle: MeRefMutHandle { is_source: false, ctx: &another.ctx }, content: &mut *another.content.get() }
    }
}
impl<'a> Drop for MeRefMutHandle<'a> {
    fn drop(&mut self) {
        if !self.is_source { return };
        let is_mut = self.ctx.is_mut.get();
        unsafe {
            *is_mut = false;
        }
    }
}

pub struct MeRef<'a, T> {
    handle: MeRefHandle<'a>,
    content: &'a T,
}
impl<'a, T> MeRef<'a, T> {
    pub fn handle(&self) -> &MeRefHandle<'a> {
        &self.handle
    }
    pub fn another<'b, U>(&'b self, another: &'b MeCell<U>) -> MeRef<'b, U> where 'a: 'b {
        self.handle.another(another)
    }
    pub fn map<'b, U, F>(self, f: F) -> MeRef<'b, U> where 'a: 'b, F: FnOnce(&T) -> &U {
        MeRef { handle: self.handle, content: f(self.content) }
    }
}
impl<'a, T> Deref for MeRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.content
    }
}
impl<'a, T: fmt::Debug> fmt::Debug for MeRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.content)
    }
}
impl<'a, T: fmt::Display> fmt::Display for MeRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

pub struct MeRefMut<'a, T> {
    handle: MeRefMutHandle<'a>,
    content: &'a mut T,
}
impl<'a, T> MeRefMut<'a, T> {
    pub fn handle_mut(&mut self) -> &mut MeRefMutHandle<'a> {
        &mut self.handle
    }
    pub fn entrance<U>(&mut self, content: U) -> MeCell<U> {
        self.handle.entrance(content)
    }
    pub fn another<'b, U>(&'b self, another: &'b MeCell<U>) -> MeRef<'b, U> where 'a: 'b {
        self.handle.another(another)
    }
    pub fn another_mut<'b, U>(&'b mut self, another: &'b MeCell<U>) -> MeRefMut<'b, U> where 'a: 'b {
        self.handle.another_mut(another)
    }
    pub unsafe fn another_mut_unsafe<'b, 'c, U>(&'b mut self, another: &'c MeCell<U>) -> MeRefMut<'c, U> where 'a: 'b {
        self.handle.another_mut_unsafe(another)
    }
    pub fn to_ref<'b>(&'b self) -> MeRef<'b, T> where 'a: 'b {
        MeRef { handle: MeRefHandle { is_source: false, ctx: self.handle.ctx }, content: &*self.content }
    }
    pub fn map<'b, U, F>(self, f: F) -> MeRefMut<'b, U> where 'a: 'b, F: FnOnce(&mut T) -> &mut U {
        MeRefMut { handle: self.handle, content: f(self.content) }
    }
}
impl<'a, T> Deref for MeRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.content
    }
}
impl<'a, T> DerefMut for MeRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.content
    }
}
impl<'a, T: fmt::Debug> fmt::Debug for MeRefMut<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.content)
    }
}
impl<'a, T: fmt::Display> fmt::Display for MeRefMut<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
