use std::rc::Rc;

use super::{backend::Backend, node::{NodeRefMut, ComponentNodeRefMut}, Component, ComponentRefMut, Property};

#[derive(Default)]
pub struct Ev<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> Ev<B, T> {
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }
    pub fn set_handler(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>) {
        self.handler = Some(Rc::new(v));
    }
    pub fn unset_handler(&mut self) {
        self.handler = None;
    }
    pub fn new_event(&self) -> Event<B, T> {
        Event {
            handler: self.handler.clone()
        }
    }
}

impl<B: Backend, T: ?Sized> Property<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>> for Ev<B, T> {
    fn update(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

pub struct Event<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> Event<B, T> {
    pub fn trigger<C: Component<B>>(self, mut target: ComponentRefMut<B, C>, data: &T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner() {
                let mut parent = parent.borrow_mut_with(target.as_node());
                handler(&mut parent, data);
            }
        }
    }
}

#[derive(Default)]
pub struct SystemEv<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> SystemEv<B, T> {
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }
    pub fn set_handler(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>) {
        self.handler = Some(Rc::new(v));
    }
    pub fn unset_handler(&mut self) {
        self.handler = None;
    }
    pub fn new_event(&self) -> SystemEvent<B, T> {
        SystemEvent {
            handler: self.handler.clone()
        }
    }
}

impl<B: Backend, T: ?Sized> Property<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>> for SystemEv<B, T> {
    fn update(&mut self, v: Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

pub struct SystemEvent<B: Backend, T: ?Sized> {
    handler: Option<Rc<Box<dyn 'static + for<'r> Fn(&'r mut ComponentNodeRefMut<B>, &'r T)>>>,
}

impl<B: Backend, T: ?Sized> SystemEvent<B, T> {
    pub fn trigger(self, mut target: NodeRefMut<B>, data: &T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner() {
                let mut parent = parent.borrow_mut_with(&mut target);
                handler(&mut parent, data);
            }
        }
    }
}
