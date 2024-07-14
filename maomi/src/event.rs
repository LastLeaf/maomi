//! The event handling utilities.
//!
//! The event fields in components can be binded by component users,
//! and triggered by the component itself.
//!
//! The following example shows the basic usage of events.
//!
//! ```rust
//! use maomi::prelude::*;
//!
//! #[component]
//! struct MyComponent {
//!     template: template! {
//!         /* ... */
//!     },
//!     // define an event with the detailed type
//!     my_event: Event<usize>,
//! }
//!
//! impl Component for MyComponent {
//!     fn new() -> Self {
//!         Self {
//!             template: Default::default(),
//!             my_event: Event::new(),
//!         }
//!     }
//!
//!     fn created(&self) {
//!         // trigger the event
//!         self.my_event.trigger(&mut 123);
//!     }
//! }
//!
//! #[component]
//! struct MyComponentUser {
//!     template: template! {
//!         // set the event listener
//!         <MyComponent my_event=@my_ev() />
//!         // extra arguments can be added in the listener
//!         // (arguments should implement `Clone` or `ToOwned`)
//!         <MyComponent my_event=@my_ev_with_data("abc") />
//!     },
//! }
//!
//! impl MyComponentUser {
//!     // the event listener has two preset arguments: `this` and the event detailed type
//!     fn my_ev(this: ComponentEvent<Self, usize>) {
//!         assert_eq!(this.detail().clone(), 123);
//!     }
//!
//!     // with extra arguments
//!     fn my_ev_with_data(this: ComponentEvent<Self, usize>, data: &str) {
//!         assert_eq!(this.detail().clone(), 123);
//!         assert_eq!(data, "abc");
//!     }
//! }
//! ```

use crate::component::ComponentRc;

/// The event handler setter.
///
/// This trait is implemented by `Event` .
/// Custom event types that implements this trait can also be used in templates with `=@` syntax.
pub trait EventHandler<D: ?Sized> {
    /// Must be `bool` if used in components
    type UpdateContext;

    /// Set the handler fn
    fn set_handler_fn(
        dest: &mut Self,
        handler_fn: Box<dyn 'static + Fn(&mut D)>,
        ctx: &mut Self::UpdateContext,
    );
}

/// An event that can be binded and triggered.
pub struct Event<D: ?Sized> {
    handler: Option<Box<dyn 'static + Fn(&mut D)>>,
}

impl<D> Default for Event<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> Event<D> {
    /// Initialize the event.
    pub fn new() -> Self {
        Self { handler: None }
    }

    /// Trigger the event.
    ///
    /// Binded handler will be called immediately.
    pub fn trigger(&self, detail: &mut D) {
        if let Some(f) = &self.handler {
            f(detail)
        }
    }
}

impl<D: ?Sized> EventHandler<D> for Event<D> {
    type UpdateContext = bool;

    fn set_handler_fn(
        dest: &mut Self,
        handler_fn: Box<dyn 'static + Fn(&mut D)>,
        _ctx: &mut Self::UpdateContext,
    ) {
        dest.handler = Some(handler_fn);
    }
}

/// A helper type that contains the event target and the event detail.
/// 
/// It implements `Deref<ComponentRc<C>>` so that any associated function in `ComponentRc<C>` can be visited.
pub struct ComponentEvent<'d, C: 'static, D> {
    rc: ComponentRc<C>,
    detail: &'d mut D,
}

impl<'d, C: 'static, D> std::ops::Deref for ComponentEvent<'d, C, D> {
    type Target = ComponentRc<C>;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}

impl<'d, C: 'static, D> ComponentEvent<'d, C, D> {
    /// Create with event target and detail.
    pub fn new(rc: ComponentRc<C>, detail: &'d mut D) -> Self {
        Self { rc, detail }
    }

    /// Get the target component.
    pub fn rc(&self) -> ComponentRc<C> {
        self.rc.clone()
    }

    /// Clone the event detail.
    pub fn clone_detail(&self) -> D where D: Clone {
        self.detail.clone()
    }

    /// Get the event detail.
    pub fn detail(&self) -> &D {
        &self.detail
    }

    /// Get the mutable reference of the event detail.
    pub fn detail_mut(&mut self) -> &mut D {
        &mut self.detail
    }
}
