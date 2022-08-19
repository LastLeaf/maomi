use wasm_bindgen::{prelude::*, JsCast};

use crate::DomGeneralElement;
use super::{ColdEventItem, DomEventRegister};

#[derive(Debug, Clone, PartialEq)]
pub struct TransitionEvent {
    dom_event: web_sys::TransitionEvent,
}

impl TransitionEvent {
    pub fn property_name(&self) -> String {
        self.dom_event.property_name()
    }

    pub fn elapsed_time(&self) -> f32 {
        self.dom_event.elapsed_time()
    }
}

fn trigger_ev<T: DomEventRegister<Detail = TransitionEvent>>(dom_event: web_sys::TransitionEvent) {
    let target = dom_event.target()
        .and_then(|x| {
            crate::DomElement::from_event_dom_elem(x.unchecked_ref())
        });
    if let Some(n) = target {
        if let DomGeneralElement::DomElement(x) = &mut *n.borrow_mut() {
            T::trigger(x, &mut TransitionEvent {
                dom_event,
            });
        }
    }
}

cold_event!(TransitionRun, TransitionEvent, "transitionrun", Closure::new(move |dom_event: web_sys::TransitionEvent| {
    trigger_ev::<TransitionRun>(dom_event);
}));

cold_event!(TransitionStart, TransitionEvent, "transitionstart", Closure::new(move |dom_event: web_sys::TransitionEvent| {
    trigger_ev::<TransitionStart>(dom_event);
}));

cold_event!(TransitionEnd, TransitionEvent, "transitionend", Closure::new(move |dom_event: web_sys::TransitionEvent| {
    trigger_ev::<TransitionEnd>(dom_event);
}));

cold_event!(TransitionCancel, TransitionEvent, "transitioncancel", Closure::new(move |dom_event: web_sys::TransitionEvent| {
    trigger_ev::<TransitionCancel>(dom_event);
}));
