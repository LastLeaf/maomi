use wasm_bindgen::{prelude::*, JsCast};

use crate::DomGeneralElement;
use super::{ColdEventItem, DomEventRegister};

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationEvent {
    dom_event: web_sys::AnimationEvent,
}

impl AnimationEvent {
    pub fn animation_name(&self) -> String {
        self.dom_event.animation_name()
    }

    pub fn elapsed_time(&self) -> f32 {
        self.dom_event.elapsed_time()
    }
}

fn trigger_ev<T: DomEventRegister<Detail = AnimationEvent>>(dom_event: web_sys::AnimationEvent) {
    let target = dom_event.target()
        .and_then(|x| {
            crate::DomElement::from_event_dom_elem(x.unchecked_ref())
        });
    if let Some(n) = target {
        if let DomGeneralElement::DomElement(x) = &mut *n.borrow_mut() {
            T::trigger(x, &mut AnimationEvent {
                dom_event,
            });
        }
    }
}

cold_event!(AnimationStart, AnimationEvent, "animationstart", Closure::new(move |dom_event: web_sys::AnimationEvent| {
    trigger_ev::<AnimationStart>(dom_event);
}));

cold_event!(AnimationIteration, AnimationEvent, "animationiteration", Closure::new(move |dom_event: web_sys::AnimationEvent| {
    trigger_ev::<AnimationIteration>(dom_event);
}));

cold_event!(AnimationEnd, AnimationEvent, "animationend", Closure::new(move |dom_event: web_sys::AnimationEvent| {
    trigger_ev::<AnimationEnd>(dom_event);
}));

cold_event!(AnimationCancel, AnimationEvent, "animationcancel", Closure::new(move |dom_event: web_sys::AnimationEvent| {
    trigger_ev::<AnimationCancel>(dom_event);
}));
