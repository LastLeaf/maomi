//! The properties utilities.
//! 
//! The properties of components can be set through templates by component users.
//! 
//! The following example show the basic usage of properties.
//! 
//! ```rust
//! use maomi::prelude::*;
//! 
//! #[component]
//! struct MyComponent {
//!     template: template! {
//!         /* ... */
//!     },
//!     // define a property with the detailed type
//!     my_prop: Prop<usize>,
//! }
//! 
//! impl Component for MyComponent {
//!     fn new() -> Self {
//!         Self {
//!             template: Default::default(),
//!             // init the property with a default value
//!             my_prop: Prop::new(123),
//!         }
//!     }
//! }
//! 
//! #[component]
//! struct MyComponentUser {
//!     template: template! {
//!         // set the property value
//!         <MyComponent my_prop=&{ 456 } />
//!     },
//! }
//! ```
//! 
//! `ListProp` is one special kind of properties.
//! It can accepts one attribute more than once.
//! This helps some special cases like `class:xxx` syntax in `maomi_dom` crate.
//! 
//! ```rust
//! use maomi::prelude::*;
//! use maomi::prop::ListProp;
//! 
//! #[component]
//! struct MyComponent {
//!     template: template! {
//!         /* ... */
//!     },
//!     // define a list property with the detailed item type
//!     my_prop: ListProp<String>,
//! }
//! 
//! impl Component for MyComponent {
//!     fn new() -> Self {
//!         Self {
//!             template: Default::default(),
//!             // init the list property
//!             my_prop: ListProp::new(),
//!         }
//!     }
//! }
//! 
//! #[component]
//! struct MyComponentUser {
//!     template: template! {
//!         // set the list property value
//!         <MyComponent my_prop:String="abc" my_prop:String="def" />
//!         // this is the same as following
//!         <MyComponent my_prop={ &["abc".to_string(), "def".to_string()] } />
//!     },
//! }
//! ```

use std::{borrow::Borrow, ops::Deref, fmt::Display};

/// The property updater.
/// 
/// This trait is implemented by `Prop` .
/// Custom property types that implements this trait can also be set through templates.
pub trait PropertyUpdate<S: ?Sized> {
    /// Must be `bool` if used in components and updated through templates.
    type UpdateContext;

    /// The updater.
    ///
    /// If used in components and updated through templates,
    /// `ctx` must be set to true if updated.
    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut Self::UpdateContext);
}

/// A property of components that can be set through templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Prop<T> {
    inner: T,
}

impl<T> Prop<T> {
    /// Create the property with initial value.
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

impl<T: Display> Display for Prop<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// Indicate that `&S` is assignable to `Prop<Self>` .
/// 
/// Every type that implements `PartialEq` and can be borrowed as `&S` automatically implements this trait.
/// For example:
/// * `usize` implements `PropAsRef<usize>` ;
/// * `String` implements `PropAsRef<String>` and `PropAsRef<str>` .
pub trait PropAsRef<S: ?Sized + PartialEq> {
    /// Borrow `&Self` as `&S` .
    fn property_as_ref(&self) -> &S;
    /// Clone `&S` as a new `Self` .
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

/// The list property initializer.
pub trait ListPropertyInit {
    /// Must be `bool` if used in components and updated through templates.
    type UpdateContext;

    /// Initialize with item count provided.
    /// 
    /// Will be called once before any list value set.
    fn init_list(dest: &mut Self, count: usize, ctx: &mut Self::UpdateContext)
    where
        Self: Sized;
}

/// The list property updater.
/// 
/// This trait is implemented by `ListProp` .
/// Custom event types that implements this trait can also be used in templates with `:xxx=` syntax.
pub trait ListPropertyUpdate<S: ?Sized>: ListPropertyInit {
    /// The item value type.
    /// 
    /// Must match the corresponding `ListPropertyItem::Value` .
    type ItemValue: ?Sized;

    /// The updater.
    ///
    /// If used in components and updated through templates,
    /// `ctx` must be set to true if updated.
    fn compare_and_set_item_ref<U: ListPropertyItem<Self, S, Value = Self::ItemValue>>(
        dest: &mut Self,
        index: usize,
        src: &S,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized;
}

/// The item updater for a specified list property `L` .
pub trait ListPropertyItem<L: ListPropertyUpdate<S>, S: ?Sized> {
    /// The item value type.
    /// 
    /// Must match the corresponding `ListPropertyUpdate::ItemValue` .
    type Value: ?Sized;

    /// Get the item value.
    ///
    /// If used in components and updated through templates,
    /// `ctx` must be set to true if updated.
    fn item_value<'a>(
        dest: &mut L,
        index: usize,
        s: &'a S,
        ctx: &mut L::UpdateContext,
    ) -> &'a Self::Value;
}

/// A list property that can be used in templates.
///
/// List properties can be updated in `:xxx=` syntax.
/// while the `item_name` is a type that implements `ListPropertyItem` .
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ListProp<T: Default> {
    inner: Box<[T]>,
}

impl<T: Default> ListProp<T> {
    /// Create the property with no item.
    pub fn new() -> Self {
        Self {
            inner: Box::new([]),
        }
    }
}

impl<'a, T: Default> IntoIterator for &'a ListProp<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
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

impl<T: Default + PartialEq + Clone> PropertyUpdate<[T]> for ListProp<T> {
    type UpdateContext = bool;

    #[inline]
    fn compare_and_set_ref(dest: &mut Self, src: &[T], ctx: &mut Self::UpdateContext) {
        if &*dest.inner == src {
            return;
        }
        dest.inner = src.iter().cloned().collect();
        *ctx = true;
    }
}

impl<T: Default> ListPropertyInit for ListProp<T> {
    type UpdateContext = bool;

    #[inline]
    fn init_list(dest: &mut Self, count: usize, _ctx: &mut bool) {
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, T::default);
        dest.inner = v.into_boxed_slice();
    }
}

impl<S: ?Sized + PartialEq, T: Default + PropAsRef<S>> ListPropertyUpdate<S> for ListProp<T> {
    type ItemValue = ();

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
    for T
{
    type Value = ();

    #[inline]
    fn item_value<'a>(
        dest: &mut ListProp<T>,
        index: usize,
        src: &'a S,
        ctx: &mut bool,
    ) -> &'a Self::Value {
        if dest.inner[index].property_as_ref() == src {
            return &();
        }
        dest.inner[index] = PropAsRef::property_to_owned(src);
        *ctx = true;
        &()
    }
}
