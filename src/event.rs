use std::rc::Rc;

use super::{backend::Backend, node::ComponentNodeRefMut, Component, ComponentRefMut, Property};

pub struct Ev<B: Backend, T> {
    handler: Option<Rc<Box<dyn 'static + Fn(&mut ComponentNodeRefMut<B>, T)>>>,
}

impl<B: Backend, T> Ev<B, T> {
    pub fn new() -> Self {
        Self {
            handler: None,
        }
    }
    pub fn new_event(&self) -> Event<B, T> {
        Event {
            handler: self.handler.clone()
        }
    }
}

impl<B: Backend, T> Property<Box<dyn 'static + Fn(&mut ComponentNodeRefMut<B>, T)>> for Ev<B, T> {
    fn update(&mut self, v: Box<dyn 'static + Fn(&mut ComponentNodeRefMut<B>, T)>) -> bool {
        self.handler = Some(Rc::new(v));
        false
    }
}

pub struct Event<B: Backend, T> {
    handler: Option<Rc<Box<dyn 'static + Fn(&mut ComponentNodeRefMut<B>, T)>>>,
}

impl<B: Backend, T> Event<B, T> {
    pub fn trigger<C: Component<B>>(self, target: &mut ComponentRefMut<B, C>, data: T) {
        if let Some(handler) = self.handler {
            if let Some(parent) = target.owner() {
                let mut parent = parent.borrow_mut_with(target.as_node());
                handler(&mut parent, data);
            }
        }
    }
    pub fn bubble<C: Component<B>>(self, target: &mut ComponentRefMut<B, C>, data: T) {
        // TODO
        unimplemented!()
    }
    pub fn bubble_composed<C: Component<B>>(self, target: &mut ComponentRefMut<B, C>, data: T) {
        // TODO
        unimplemented!()
    }
}
