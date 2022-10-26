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

/// The mouse-related event detail.
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

/// The mouse-related event detail.
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

fn trigger_ev_input<T: DomEventRegister<Detail = InputEvent>>(dom_event: web_sys::InputEvent) {
    let target = dom_event
        .target()
        .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref(), false));
    if let Some(n) = target {
        if let DomGeneralElement::Element(x) = &mut *n.borrow_mut() {
            T::trigger(
                x,
                &mut InputEvent {
                    dom_event,
                },
            );
        }
    }
}

/// The input event detail.
#[derive(Debug, Clone, PartialEq)]
pub struct InputEvent {
    dom_event: web_sys::InputEvent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEventType {
    
}

impl InputEvent {
    /// Get the inserted characters.
    #[inline]
    pub fn data(&self) -> Option<String> {
        self.dom_event.data()
    }

    /// Get whether action is during the composition progress, a.k.a. input with IME.
    #[inline]
    pub fn is_composing(&self) -> bool {
        self.dom_event.is_composing()
    }
}

cold_event!(
    Input,
    InputEvent,
    Closure::new(move |dom_event: web_sys::InputEvent| {
        trigger_ev_input::<Input>(dom_event);
    })
);
