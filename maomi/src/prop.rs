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

impl<S: ?Sized + PartialEq + ToOwned<Owned = T>, T: Borrow<S>> PropAsRef<S> for T {
    fn property_as_ref(&self) -> &S {
        self.borrow()
    }

    fn property_to_owned(s: &S) -> Self where Self: Sized {
        s.to_owned()
    }
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

/// The list property updater
pub trait ListPropertyUpdate<S: ?Sized> {
    /// Must be `bool` if used in components
    type UpdateContext;

    /// The initiator with item count provided
    fn init_list(dest: &mut Self, count: usize);

    /// The updater
    ///
    /// If used in components, `ctx` must be set to true if updated
    fn compare_and_set_item_ref<U: ListPropertyUpdateItem<Self, S, UpdateContext = Self::UpdateContext>>(
        dest: &mut Self,
        index: usize,
        src: &S,
        ctx: &mut Self::UpdateContext,
    ) where Self: Sized {
        U::compare_and_set_ref(dest, index, src, ctx)
    }
}

pub trait ListPropertyUpdateItem<L: ListPropertyUpdate<S>, S: ?Sized> {
    /// Must be `bool` if used in components
    type UpdateContext;

    /// The updater
    ///
    /// If used in components, `ctx` must be set to true if updated
    fn compare_and_set_ref(list: &mut L, index: usize, s: &S, ctx: &mut Self::UpdateContext);
}

pub struct ListProp<T: Default> {
    inner: Vec<T>,
}

impl<T: Default> ListProp<T> {
    pub fn new() -> Self {
        Self { inner: Vec::with_capacity(0) }
    }
}

impl<T: Default> Deref for ListProp<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Default> AsRef<[T]> for ListProp<T> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

impl<T: Default> Borrow<[T]> for ListProp<T> {
    fn borrow(&self) -> &[T] {
        &self.inner
    }
}

impl<S: ?Sized + PartialEq, T: Default + PropAsRef<S>> ListPropertyUpdate<S> for ListProp<T> {
    type UpdateContext = bool;

    fn init_list(dest: &mut Self, count: usize) {
        dest.inner.resize_with(count, T::default);
    }
}

impl<S: ?Sized + PartialEq, T: Default + PropAsRef<S>> ListPropertyUpdateItem<ListProp<T>, S> for ListProp<T> {
    type UpdateContext = bool;

    fn compare_and_set_ref(list: &mut ListProp<T>, index: usize, src: &S, ctx: &mut Self::UpdateContext) {
        if list.inner[index].property_as_ref() == src {
            return;
        }
        list.inner[index] = PropAsRef::property_to_owned(src);
        *ctx = true;
    }
}
