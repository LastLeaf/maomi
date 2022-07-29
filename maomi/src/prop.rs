use std::{borrow::Borrow, ops::Deref};

/// The property updater
pub trait PropertyUpdate<S: ?Sized> {
    /// Must be `bool` if used in components
    type UpdateContext;

    /// The updater
    ///
    /// If used in components, `ctx` must be set to true if updated
    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut Self::UpdateContext);
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

impl<T> AsRef<T> for Prop<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> Borrow<T> for Prop<T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

/// Indicate that `&S` is assignable to `Prop<Self>`
pub trait PropAsRef<S: ?Sized + PartialEq> {
    fn property_as_ref(&self) -> &S;
    fn property_to_owned(s: &S) -> Self where Self: Sized;
}

impl<S: ?Sized + PartialEq, T: PropAsRef<S>> PropertyUpdate<S> for Prop<T> {
    type UpdateContext = bool;

    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut bool) {
        if dest.inner.property_as_ref() == src {
            return;
        }
        dest.inner = PropAsRef::property_to_owned(src);
        *ctx = true;
    }
}

impl<S: ?Sized + PartialEq + ToOwned<Owned = T>, T: Borrow<S>> PropAsRef<S> for T {
    fn property_as_ref(&self) -> &S {
        self.borrow()
    }

    fn property_to_owned(s: &S) -> Self where Self: Sized {
        s.to_owned()
    }
}
