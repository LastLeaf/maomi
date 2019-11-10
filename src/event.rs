use std::rc::Rc;

use super::{backend::Backend, node::{NodeRefMut, ComponentNodeRefMut}, Component, ComponentRefMut, Property};

#[derive(Default)]
pub struct Ev<B: Backend, T> {
    handler: Option<Rc<Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>>>,
}

impl<B: Backend, T> Ev<B, T> {
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }
    pub fn set_handler(&mut self, v: Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>) {
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

impl<B: Backend, T> Property<Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>> for Ev<B, T> {
    fn update(&mut self, v: Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

pub struct Event<B: Backend, T> {
    handler: Option<Rc<Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>>>,
}

impl<B: Backend, T> Event<B, T> {
    pub fn trigger<C: Component<B>>(self, mut target: ComponentRefMut<B, C>, data: T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner() {
                let parent = parent.borrow_mut_with(target.as_node());
                handler(parent, data);
            }
        }
    }
}

#[derive(Default)]
pub struct SystemEv<B: Backend, T> {
    handler: Option<Rc<Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>>>,
}

impl<B: Backend, T> SystemEv<B, T> {
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }
    pub fn set_handler(&mut self, v: Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>) {
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

impl<B: Backend, T> Property<Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>> for SystemEv<B, T> {
    fn update(&mut self, v: Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

pub struct SystemEvent<B: Backend, T> {
    handler: Option<Rc<Box<dyn 'static + Fn(ComponentNodeRefMut<B>, T)>>>,
}

impl<B: Backend, T> SystemEvent<B, T> {
    pub fn trigger(self, mut target: NodeRefMut<B>, data: T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner() {
                let parent = parent.borrow_mut_with(&mut target);
                handler(parent, data);
            }
        }
    }
}
