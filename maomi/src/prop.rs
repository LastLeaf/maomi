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

/// A property that can be used in templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
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
    fn property_to_owned(s: &S) -> Self
    where
        Self: Sized;
}

impl<S: ?Sized + PartialEq + ToOwned<Owned = T>, T: Borrow<S>> PropAsRef<S> for T {
    fn property_as_ref(&self) -> &S {
        self.borrow()
    }

    fn property_to_owned(s: &S) -> Self
    where
        Self: Sized,
    {
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
///
/// List properties can be updated in `prop:item_name={}` form,
/// while the `item_name` is a type that implements `ListPropertyItem` .
pub trait ListPropertyUpdate<S: ?Sized> {
    /// Must be `bool` if used in components
    type UpdateContext;

    /// The updater return value
    type ItemValue;

    /// The initiator with item count provided
    fn init_list(dest: &mut Self, count: usize, ctx: &mut Self::UpdateContext);

    /// The updater
    ///
    /// If used in components, `ctx` must be set to true if updated
    fn compare_and_set_item_ref<U: ListPropertyItem<Self, S, Value = Self::ItemValue>>(
        dest: &mut Self,
        index: usize,
        src: &S,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized;
}

/// The item updater for a specified list property `L`
pub trait ListPropertyItem<L: ListPropertyUpdate<S>, S: ?Sized> {
    type Value: ?Sized;

    /// Generate the item value
    fn item_value(dest: &mut L, index: usize, s: &S, ctx: &mut L::UpdateContext) -> Self::Value;
}

/// A list property that can be used in templates
pub struct ListProp<T: Default> {
    inner: Vec<T>,
}

impl<T: Default> ListProp<T> {
    pub fn new() -> Self {
        Self {
            inner: Vec::with_capacity(0),
        }
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
    type ItemValue = ();

    #[inline]
    fn init_list(dest: &mut Self, count: usize, _ctx: &mut bool) {
        dest.inner.resize_with(count, T::default);
    }

    #[inline]
    fn compare_and_set_item_ref<U: ListPropertyItem<Self, S, Value = ()>>(
        dest: &mut Self,
        index: usize,
        src: &S,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        U::item_value(dest, index, src, ctx);
    }
}

impl<S: ?Sized + PartialEq, T: Default + PropAsRef<S>> ListPropertyItem<ListProp<T>, S>
    for ListProp<T>
{
    type Value = ();

    #[inline]
    fn item_value(dest: &mut Self, index: usize, src: &S, ctx: &mut bool) -> Self::Value {
        if dest.inner[index].property_as_ref() == src {
            return;
        }
        dest.inner[index] = PropAsRef::property_to_owned(src);
        *ctx = true;
    }
}
