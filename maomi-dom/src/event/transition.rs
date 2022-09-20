use wasm_bindgen::{prelude::*, JsCast};

use super::{ColdEventItem, DomEventRegister};
use crate::DomGeneralElement;

/// The transition event detail
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionEvent {
    dom_event: web_sys::TransitionEvent,
}

impl TransitionEvent {
    /// Get the property name that the transition runs on
    #[inline]
    pub fn property_name(&self) -> String {
        self.dom_event.property_name()
    }

    /// Get the elapsed time of the transition
    #[inline]
    pub fn elapsed_time(&self) -> f32 {
        self.dom_event.elapsed_time()
    }
}

fn trigger_ev<T: DomEventRegister<Detail = TransitionEvent>>(dom_event: web_sys::TransitionEvent) {
    let target = dom_event
        .target()
        .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref(), false));
    if let Some(n) = target {
        if let DomGeneralElement::Element(x) = &mut *n.borrow_mut() {
            T::trigger(x, &mut TransitionEvent { dom_event });
        }
    }
}

cold_event!(
    TransitionRun,
    TransitionEvent,
    Closure::new(move |dom_event: web_sys::TransitionEvent| {
        trigger_ev::<TransitionRun>(dom_event);
    })
);

cold_event!(
    TransitionStart,
    TransitionEvent,
    Closure::new(move |dom_event: web_sys::TransitionEvent| {
        trigger_ev::<TransitionStart>(dom_event);
    })
);

cold_event!(
    TransitionEnd,
    TransitionEvent,
    Closure::new(move |dom_event: web_sys::TransitionEvent| {
        trigger_ev::<TransitionEnd>(dom_event);
    })
);

cold_event!(
    TransitionCancel,
    TransitionEvent,
    Closure::new(move |dom_event: web_sys::TransitionEvent| {
        trigger_ev::<TransitionCancel>(dom_event);
    })
);
