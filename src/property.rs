use std::hash::*;
use std::ops::*;
use std::cmp::*;
use std::fmt;
use std::borrow;

pub trait Property<T> {
    fn update(&mut self, v: T) -> bool;
    #[inline]
    fn update_from<I: Into<T>>(&mut self, v: I) -> bool where I: ?Sized {
        self.update(v.into())
    }
}

impl<T: PartialEq> Property<T> for T {
    #[inline]
    fn update(&mut self, v: T) -> bool {
        if *self == v {
            return false
        }
        *self = v;
        true
    }
}

#[derive(PartialEq)]
pub struct Prop<T: PartialEq> {
    data: T
}

impl<T: PartialEq> Prop<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: PartialEq> Property<T> for Prop<T> {
    #[inline]
    fn update(&mut self, v: T) -> bool {
        if self.data == v {
            return false
        }
        self.data = v;
        true
    }
}

impl<T: PartialEq> From<T> for Prop<T> {
    fn from(data: T) -> Self {
        Self { data }
    }
}

impl<T: PartialEq> Deref for Prop<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: PartialEq> DerefMut for Prop<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Eq> Eq for Prop<T> {}

impl<T: PartialOrd> PartialOrd for Prop<T> {
    #[inline]
    fn partial_cmp(&self, other: &Prop<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &Prop<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &Prop<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &Prop<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &Prop<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: Ord> Ord for Prop<T> {
    #[inline]
    fn cmp(&self, other: &Prop<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Hasher + PartialEq> Hasher for Prop<T> {
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

impl<T: Hash + PartialEq> Hash for Prop<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: PartialEq> borrow::Borrow<T> for Prop<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: PartialEq> borrow::BorrowMut<T> for Prop<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: PartialEq> AsRef<T> for Prop<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T: PartialEq> AsMut<T> for Prop<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: Default + PartialEq> Default for Prop<T> {
    fn default() -> Prop<T> {
        Self { data: Default::default() }
    }
}

impl<T: Copy + PartialEq> Copy for Prop<T> { }

impl<T: Clone + PartialEq> Clone for Prop<T> {
    #[inline]
    fn clone(&self) -> Prop<T> {
        Self { data: (**self).clone() }
    }
}

impl<T: fmt::Display + PartialEq> fmt::Display for Prop<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug + PartialEq> fmt::Debug for Prop<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
