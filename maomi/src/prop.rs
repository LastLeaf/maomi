use std::{borrow::Borrow, ops::Deref};

pub trait PropertyUpdate<S: ?Sized> {
    fn compare_and_set_ref(dest: &mut Self, src: &S) -> bool;
}

pub struct Prop<T> {
    inner: T,
}

impl<T> Prop<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Deref for Prop<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Borrow<S>, S: ?Sized + PartialEq + ToOwned<Owned = T>> PropertyUpdate<S> for Prop<T> {
    fn compare_and_set_ref(dest: &mut Self, src: &S) -> bool {
        if dest.inner.borrow() == src {
            return false;
        }
        dest.inner = src.to_owned();
        true
    }
}

fn test() {
    let mut t: Prop<String> = Prop::new(String::new());
    PropertyUpdate::compare_and_set_ref(&mut t, "abc");
    PropertyUpdate::compare_and_set_ref(&mut t, &String::from("abc"));
    let mut u: Prop<usize> = Prop::new(123);
    PropertyUpdate::compare_and_set_ref(&mut u, &&789usize);
    PropertyUpdate::compare_and_set_ref(&mut u, &456usize);
}
