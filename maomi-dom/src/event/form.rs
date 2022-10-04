use wasm_bindgen::{JsCast, closure::Closure};

use super::{ColdEventItem, DomEventRegister};
use crate::DomGeneralElement;

fn trigger_ev_submit<T: DomEventRegister<Detail = SubmitEvent>>(dom_event: web_sys::SubmitEvent) {
    let target = dom_event
        .target()
        .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref(), false));
    if let Some(n) = target {
        if let DomGeneralElement::Element(x) = &mut *n.borrow_mut() {
            T::trigger(
                x,
                &mut SubmitEvent {
                    dom_event,
                },
            );
        }
    }
}

/// The mouse-related event detail
#[derive(Debug, Clone, PartialEq)]
pub struct SubmitEvent {
    dom_event: web_sys::SubmitEvent,
}

cold_event!(
    Submit,
    SubmitEvent,
    Closure::new(move |dom_event: web_sys::SubmitEvent| {
        trigger_ev_submit::<Submit>(dom_event);
    })
);

fn trigger_ev_change<T: DomEventRegister<Detail = ChangeEvent>>(dom_event: web_sys::Event) {
    let target = dom_event
        .target()
        .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref(), false));
    if let Some(n) = target {
        if let DomGeneralElement::Element(x) = &mut *n.borrow_mut() {
            T::trigger(
                x,
                &mut ChangeEvent {
                    dom_event,
                },
            );
        }
    }
}

/// The mouse-related event detail
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeEvent {
    dom_event: web_sys::Event,
}

cold_event!(
    Change,
    ChangeEvent,
    Closure::new(move |dom_event: web_sys::Event| {
        trigger_ev_change::<Change>(dom_event);
    })
);
