use std::hash::*;
use std::ops::*;
use std::cmp::*;
use std::fmt;
use std::borrow;
use std::pin::Pin;
use std::rc::Rc;

use super::notify_updater;
use super::dirty_marker::DirtyMarker;

pub struct Observable<T> {
    data: T,
    dirty: Pin<Rc<DirtyMarker>>,
}

impl<T> Observable<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self { data, dirty: Rc::pin(DirtyMarker::new(false)) }
    }
}

impl<T> Drop for Observable<T> {
    fn drop(&mut self) {
        self.dirty.destroy();
    }
}

impl<T> Deref for Observable<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        notify_updater(&self.dirty);
        &self.data
    }
}

impl<T> DerefMut for Observable<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.dirty.mark_connected_dirty();
        notify_updater(&self.dirty);
        &mut self.data
    }
}

impl<T: PartialEq> PartialEq for Observable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T: Eq> Eq for Observable<T> {}

impl<T: PartialOrd> PartialOrd for Observable<T> {
    #[inline]
    fn partial_cmp(&self, other: &Observable<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &Observable<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &Observable<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &Observable<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &Observable<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: Ord> Ord for Observable<T> {
    #[inline]
    fn cmp(&self, other: &Observable<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Hasher> Hasher for Observable<T> {
    fn finish(&self) -> u64 {
        (**self).finish()
    }
    fn write(&mut self, bytes: &[u8]) {
        (**self).write(bytes)
    }
    fn write_u8(&mut self, i: u8) {
        (**self).write_u8(i)
    }
    fn write_u16(&mut self, i: u16) {
        (**self).write_u16(i)
    }
    fn write_u32(&mut self, i: u32) {
        (**self).write_u32(i)
    }
    fn write_u64(&mut self, i: u64) {
        (**self).write_u64(i)
    }
    fn write_u128(&mut self, i: u128) {
        (**self).write_u128(i)
    }
    fn write_usize(&mut self, i: usize) {
        (**self).write_usize(i)
    }
    fn write_i8(&mut self, i: i8) {
        (**self).write_i8(i)
    }
    fn write_i16(&mut self, i: i16) {
        (**self).write_i16(i)
    }
    fn write_i32(&mut self, i: i32) {
        (**self).write_i32(i)
    }
    fn write_i64(&mut self, i: i64) {
        (**self).write_i64(i)
    }
    fn write_i128(&mut self, i: i128) {
        (**self).write_i128(i)
    }
    fn write_isize(&mut self, i: isize) {
        (**self).write_isize(i)
    }
}

impl<T: Hash> Hash for Observable<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T> borrow::Borrow<T> for Observable<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T> borrow::BorrowMut<T> for Observable<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T> AsRef<T> for Observable<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> AsMut<T> for Observable<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: Default> Default for Observable<T> {
    fn default() -> Observable<T> {
        Self { data: Default::default(), dirty: Rc::pin(DirtyMarker::new(false)) }
    }
}

impl<T: Clone> Clone for Observable<T> {
    #[inline]
    fn clone(&self) -> Observable<T> {
        Self { data: (**self).clone(), dirty: Rc::pin(DirtyMarker::new(false)) }
    }
}

impl<T: fmt::Display> fmt::Display for Observable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Observable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
