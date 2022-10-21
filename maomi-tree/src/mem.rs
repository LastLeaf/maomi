use std::{
    cell::{Cell, RefCell, UnsafeCell},
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr,
    rc::Rc,
};

pub(crate) struct SliceWeak<T, const N: usize> {
    mem: *const SliceInner<T, N>,
}

impl<T, const N: usize> Drop for SliceWeak<T, N> {
    #[inline]
    fn drop(&mut self) {
        let mem = unsafe { &*self.mem };
        mem.weak.set(mem.weak.get() - 1);
        if mem.strong.get() == 0 && mem.weak.get() == 0 {
            let mem = unsafe { &mut *(self.mem as *mut SliceInner<T, N>) };
            unsafe {
                ManuallyDrop::drop(&mut mem.strong);
                ManuallyDrop::drop(&mut mem.weak);
            }
            SliceAlloc::add_to_free_list(&mut mem.owner.borrow_mut(), self.mem);
        }
    }
}

impl<T, const N: usize> SliceWeak<T, N> {
    pub(crate) fn mem(&self) -> *const () {
        self.mem as *const ()
    }

    pub(crate) fn rc(&self) -> Option<SliceRc<T, N>> {
        let mem = unsafe { &*self.mem };
        let strong = mem.strong.get();
        if strong == 0 {
            return None;
        }
        mem.strong.set(strong + 1);
        Some(SliceRc { mem: self.mem })
    }

    pub(crate) fn leak(self) -> *const () {
        let ret = self.mem();
        std::mem::forget(self);
        ret
    }

    pub(crate) unsafe fn from_leaked(p: *const ()) -> Self {
        let ret = SliceWeak {
            mem: p as *const SliceInner<T, N>,
        };
        ret
    }
}

impl SliceWeak<(), 1> {
    pub(crate) unsafe fn revoke_leaked(p: *const ()) {
        let _ret: SliceWeak<(), 1> = SliceWeak {
            mem: p as *const SliceInner<(), 1>,
        };
    }

    pub(crate) unsafe fn clone_weak(p: *const ()) -> *const () {
        let ret: SliceWeak<(), 1> = SliceWeak {
            mem: p as *const SliceInner<(), 1>,
        };
        (*ret.mem).weak.set((*ret.mem).weak.get() + 1);
        ret.leak()
    }
}

impl<T, const N: usize> Clone for SliceWeak<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        let mem = unsafe { &*self.mem };
        mem.weak.set(mem.weak.get() + 1);
        Self { mem: self.mem }
    }
}

pub(crate) struct SliceRc<T, const N: usize> {
    mem: *const SliceInner<T, N>,
}

impl<T, const N: usize> Drop for SliceRc<T, N> {
    #[inline]
    fn drop(&mut self) {
        let mem = unsafe { &*self.mem };
        mem.strong.set(mem.strong.get() - 1);
        if mem.strong.get() == 0 {
            let need_drop_mem = mem.weak.get() == 0;
            let mem = unsafe { &mut *(self.mem as *mut SliceInner<T, N>) };
            unsafe {
                ManuallyDrop::drop(&mut mem.data);
            }
            if need_drop_mem {
                unsafe {
                    ManuallyDrop::drop(&mut mem.strong);
                    ManuallyDrop::drop(&mut mem.weak);
                }
                SliceAlloc::add_to_free_list(&mut mem.owner.borrow_mut(), self.mem);
            }
        }
    }
}

impl<T, const N: usize> SliceRc<T, N> {
    pub(crate) fn mem(&self) -> *const () {
        self.mem as *const ()
    }

    pub(crate) unsafe fn data_ref(&self) -> &T {
        let ptr = (&*self.mem).data.get();
        &*ptr
    }

    pub(crate) unsafe fn data_mut(&self) -> &mut T {
        let ptr = (&*self.mem).data.get();
        &mut *ptr
    }

    pub(crate) fn weak(&self) -> SliceWeak<T, N> {
        let mem = unsafe { &*self.mem };
        mem.weak.set(mem.weak.get() + 1);
        SliceWeak { mem: self.mem }
    }
}

impl<T, const N: usize> Clone for SliceRc<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        let mem = unsafe { &*self.mem };
        mem.strong.set(mem.strong.get() + 1);
        Self { mem: self.mem }
    }
}

pub(crate) struct SliceAlloc<T, const N: usize> {
    inner: Rc<RefCell<SliceAllocInner<T, N>>>,
}

pub(crate) struct SliceAllocInner<T, const N: usize> {
    slices: Vec<SliceBuf<T, N>>,
    last_freed: *const SliceInner<T, N>,
    last_used_count: usize,
}

struct SliceBuf<T, const N: usize> {
    slices: Pin<Box<[SliceInner<T, N>; N]>>,
}

#[repr(C)]
struct SliceInner<T, const N: usize> {
    strong: ManuallyDrop<Cell<usize>>,
    weak: ManuallyDrop<Cell<usize>>,
    owner: Rc<RefCell<SliceAllocInner<T, N>>>,
    data: ManuallyDrop<UnsafeCell<T>>,
}

impl<T, const N: usize> SliceAlloc<T, N> {
    pub(crate) fn new() -> Self {
        let inner = SliceAllocInner {
            slices: vec![],
            last_freed: ptr::null(),
            last_used_count: N,
        };
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub(crate) fn alloc(&mut self, data: T) -> SliceRc<T, N> {
        let owner = self.inner.clone();
        let mut inner = self.inner.borrow_mut();
        let mut mem = if !inner.last_freed.is_null() {
            let p = inner.last_freed;
            inner.last_freed = unsafe { *(p as *mut *const SliceInner<T, N>) };
            unsafe { &mut *(p as *mut SliceInner<T, N>) }
        } else {
            if inner.last_used_count == N {
                let new_buf = SliceBuf {
                    slices: Box::pin(unsafe { MaybeUninit::uninit().assume_init() }),
                };
                inner.slices.push(new_buf);
                inner.last_used_count = 0;
            }
            let i = inner.last_used_count;
            inner.last_used_count += 1;
            let r = unsafe {
                &mut inner
                    .slices
                    .last_mut()
                    .unwrap()
                    .slices
                    .as_mut()
                    .get_unchecked_mut()[i]
            };
            r
        };
        mem.strong = ManuallyDrop::new(Cell::new(1));
        mem.weak = ManuallyDrop::new(Cell::new(0));
        mem.data = ManuallyDrop::new(UnsafeCell::new(data));
        std::mem::forget(std::mem::replace(&mut mem.owner, owner));
        SliceRc { mem }
    }

    fn add_to_free_list(inner: &mut SliceAllocInner<T, N>, p: *const SliceInner<T, N>) {
        let prev_p = inner.last_freed;
        inner.last_freed = p;
        unsafe { *(p as *mut *const SliceInner<T, N>) = prev_p };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reuse() {
        let mut sa: SliceAlloc<usize, 3> = SliceAlloc::new();
        let d10 = sa.alloc(10);
        let d11 = sa.alloc(11);
        assert_eq!(
            d11.mem() as usize - d10.mem() as usize,
            std::mem::size_of::<SliceInner<usize, 3>>()
        );
        let (d12mem, d13mem) = {
            let d12 = sa.alloc(12);
            let d13 = {
                let mut sa2: SliceAlloc<(), 3> = SliceAlloc::new();
                sa2.alloc(());
                let d13 = sa.alloc(13);
                d13
            };
            assert_eq!(unsafe { *d10.data_ref() }, 10);
            assert_eq!(unsafe { *d11.data_ref() }, 11);
            assert_eq!(unsafe { *d12.data_ref() }, 12);
            assert_eq!(unsafe { *d13.data_ref() }, 13);
            (d13.mem(), d12.mem())
        };
        assert_ne!(
            std::num::Wrapping(d13mem as usize) - std::num::Wrapping(d12mem as usize),
            std::num::Wrapping(std::mem::size_of::<SliceInner<usize, 3>>()),
        );
        let d23 = sa.alloc(23);
        let d22 = sa.alloc(22);
        assert_eq!(unsafe { *d22.data_ref() }, 22);
        assert_eq!(unsafe { *d23.data_ref() }, 23);
        assert_eq!(d22.mem(), d12mem);
        assert_eq!(d23.mem(), d13mem);
    }

    #[test]
    fn leak() {
        let mut sa: SliceAlloc<usize, 16> = SliceAlloc::new();
        let _d10 = sa.alloc(10);
        let (d11mem, _d12) = {
            let (d11mem, w11) = {
                let d11 = sa.alloc(11);
                let w11 = d11.weak().leak();
                let w11rc = unsafe { SliceWeak::<usize, 16>::from_leaked(w11) }
                    .rc()
                    .unwrap();
                assert_eq!(unsafe { *w11rc.data_ref() }, 11);
                (d11.mem(), w11rc.weak().leak())
            };
            let d12 = sa.alloc(12);
            assert_ne!(d12.mem(), d11mem);
            {
                let w = unsafe { SliceWeak::<usize, 16>::from_leaked(w11) };
                assert!(w.rc().is_none());
                w.leak();
            }
            unsafe { SliceWeak::revoke_leaked(w11) };
            (d11mem, d12)
        };
        let d13 = sa.alloc(13);
        assert_eq!(d13.mem(), d11mem);
    }
}
