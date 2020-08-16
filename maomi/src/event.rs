use std::rc::Rc;

use super::{backend::Backend, node::{NodeMut, ComponentNode}, Property};

/// A custom event binding position.
/// `T` is the argument type provided when triggering.
#[derive(Default)]
pub struct Ev<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> Ev<B, T> {
    /// Create a new event binding position
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }

    /// Set the handler
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn set_handler(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>) {
        self.handler = Some(Rc::new(v));
    }

    /// Remove the handler
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn unset_handler(&mut self) {
        self.handler = None;
    }

    /// Generate a new event
    pub fn new_event(&self) -> Event<B, T> {
        Event {
            handler: self.handler.clone()
        }
    }
}

impl<B: Backend, T: ?Sized> Property<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>> for Ev<B, T> {
    fn update(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

/// A custom event
pub struct Event<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> Event<B, T> {
    /// Trigger the event
    pub fn trigger(self, target: &mut ComponentNode<B>, data: &T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner_mut().next() {
                handler(parent, data);
            }
        }
    }
}

/// A system event binding position.
/// System event is auto triggered by backend.
/// **Should be done through template engine!**
#[doc(hidden)]
#[derive(Default)]
pub struct SystemEv<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> SystemEv<B, T> {
    /// Create a new event binding position
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }

    /// Set the handler
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn set_handler(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>) {
        self.handler = Some(Rc::new(v));
    }

    /// Remove the handler
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn unset_handler(&mut self) {
        self.handler = None;
    }

    /// Generate a new event
    pub fn new_event(&self) -> SystemEvent<B, T> {
        SystemEvent {
            handler: self.handler.clone()
        }
    }
}

impl<B: Backend, T: ?Sized> Property<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>> for SystemEv<B, T> {
    fn update(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

/// A system event.
/// System event is auto triggered by backend.
/// **Should be done through template engine!**
#[doc(hidden)]
pub struct SystemEvent<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNode<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> SystemEvent<B, T> {
    /// Trigger the event.
    /// System event is auto triggered by backend.
    /// **Should be done through template engine!**
    pub fn trigger(self, mut target: NodeMut<B>, data: &T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner_mut().next() {
                handler(parent, data);
            }
        }
    }
}
