use wasm_bindgen::{prelude::*, JsCast};

use crate::DomGeneralElement;
use super::{ColdEventItem, DomEventRegister, BubbleEvent};

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollEvent {
    propagation_stopped: bool,
    default_prevented: bool,
    dom_event: web_sys::Event,
}

impl BubbleEvent for ScrollEvent {
    fn stop_propagation(&mut self) {
        if self.propagation_stopped { return };
        self.propagation_stopped = true;
        self.dom_event.stop_propagation()
    }

    fn propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }

    fn prevent_default(&mut self) {
        if self.default_prevented { return };
        self.default_prevented = true;
        self.dom_event.prevent_default()
    }

    fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}

fn trigger_ev<T: DomEventRegister<Detail = ScrollEvent>>(dom_event: web_sys::Event) {
    let target = dom_event.target()
        .and_then(|x| {
            crate::DomElement::from_event_dom_elem(x.unchecked_ref())
        });
    if let Some(n) = target {
        if let DomGeneralElement::DomElement(x) = &mut *n.borrow_mut() {
            T::trigger(x, &mut ScrollEvent {
                propagation_stopped: false,
                default_prevented: false,
                dom_event,
            });
        }
    }
}

cold_event!(Scroll, ScrollEvent, "scroll", Closure::new(move |dom_event: web_sys::Event| {
    trigger_ev::<Scroll>(dom_event);
}));
