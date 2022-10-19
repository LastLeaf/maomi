//! The event handling utilities.
//! 
//! The event fields in components can be binded by component users,
//! and triggered by the component itself.
//! 
//! ```rust
//! use maomi::prelude::*;
//! 
//! #[component]
//! struct MyComponent {
//!     template: template! {
//!         /* ... */
//!     },
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
//! }
//! 
//! #[component]
//! struct MyComponentUser {
//!     template: template! {
//!         <MyComponent my_event=@my_ev() />
//!     },
//! }
//! 
//! impl MyComponentUser {
//!     fn my_ev(this: ComponentRc<Self>, detail: &mut usize) { /* ... */ }
//! }
//! ```
//! 
//! 

/// The event handler setter.
/// 
/// This trait is implemented by `Event` .
/// It can be triggered by 
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

/// An event that can be binded and triggered
pub struct Event<D: ?Sized> {
    handler: Option<Box<dyn 'static + Fn(&mut D)>>,
}

impl<D> Default for Event<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> Event<D> {
    pub fn new() -> Self {
        Self { handler: None }
    }

    /// Trigger the event
    ///
    /// Binded handler will be called immediately.
    pub fn trigger(&self, detail: &mut D) {
        if let Some(f) = &self.handler {
            f(detail);
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
