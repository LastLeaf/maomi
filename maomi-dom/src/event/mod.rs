use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use maomi::event::EventHandler;

use crate::base_element::DomElement;

pub(crate) mod touch;
pub use touch::SingleTouchEvent;
pub(crate) mod animation;
pub use animation::AnimationEvent;

pub(crate) struct DomListeners {
    #[allow(dead_code)]
    touch: Closure::<dyn Fn(web_sys::TouchEvent)>,
}

impl DomListeners {
    pub(crate) fn new(element: &web_sys::Element) -> Result<Self, JsValue> {
        Ok(Self {
            touch: touch::init_dom_listeners(element)?,
        })
    }
}

// hot event list is usually used to store popular events and bubble events
#[derive(Default)]
pub(crate) struct HotEventList {
    touch_start: Option<Box<dyn 'static + Fn(&mut SingleTouchEvent)>>,
    touch_move: Option<Box<dyn 'static + Fn(&mut SingleTouchEvent)>>,
    touch_end: Option<Box<dyn 'static + Fn(&mut SingleTouchEvent)>>,
    touch_cancel: Option<Box<dyn 'static + Fn(&mut SingleTouchEvent)>>,
}

// code event list is slow to visit but memory-efficient
pub(crate) type ColdEventList = Vec<ColdEventItem>;

pub(crate) enum ColdEventItem {
    AnimationStart(Box<dyn 'static + Fn(&mut AnimationEvent)>),
    AnimationIteration(Box<dyn 'static + Fn(&mut AnimationEvent)>),
    AnimationEnd(Box<dyn 'static + Fn(&mut AnimationEvent)>),
    AnimationCancel(Box<dyn 'static + Fn(&mut AnimationEvent)>),
    // TransitionRun(Box<dyn 'static + Fn(&mut TransitionEvent)>),
    // TransitionStart(Box<dyn 'static + Fn(&mut TransitionEvent)>),
    // TransitionEnd(Box<dyn 'static + Fn(&mut TransitionEvent)>),
    // TransitionCancel(Box<dyn 'static + Fn(&mut TransitionEvent)>),
}

/// A DOM event that can be binded
pub trait DomEventRegister {
    type Detail;

    fn bind(target: &mut DomElement, f: Box<dyn 'static + Fn(&mut Self::Detail)>);
    fn trigger(target: &mut DomElement, detail: &mut Self::Detail);
}

// A non-bubble DOM event
pub struct DomEvent<M: DomEventRegister> {
    _phantom: PhantomData<M>,
}

impl<M: DomEventRegister> Default for DomEvent<M> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<M: DomEventRegister> EventHandler<M::Detail> for DomEvent<M> {
    type UpdateContext = DomElement;

    fn set_handler_fn(_dest: &mut Self, handler_fn: Box<dyn 'static + Fn(&mut M::Detail)>, ctx: &mut DomElement) {
        M::bind(ctx, handler_fn);
    }
}

pub trait BubbleEvent {
    fn stop_propagation(&mut self);
    fn prevent_default(&mut self);
}
