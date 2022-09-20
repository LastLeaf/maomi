use wasm_bindgen::{prelude::*, JsCast};

use super::{ColdEventItem, DomEventRegister};
use crate::DomGeneralElement;

/// The animation event detail
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationEvent {
    dom_event: web_sys::AnimationEvent,
}

impl AnimationEvent {
    /// Get the elapsed time of the animation
    #[inline]
    pub fn elapsed_time(&self) -> f32 {
        self.dom_event.elapsed_time()
    }
}

fn trigger_ev<T: DomEventRegister<Detail = AnimationEvent>>(dom_event: web_sys::AnimationEvent) {
    let target = dom_event
        .target()
        .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref(), false));
    if let Some(n) = target {
        if let DomGeneralElement::Element(x) = &mut *n.borrow_mut() {
            T::trigger(x, &mut AnimationEvent { dom_event });
        }
    }
}

cold_event!(
    AnimationStart,
    AnimationEvent,
    Closure::new(move |dom_event: web_sys::AnimationEvent| {
        trigger_ev::<AnimationStart>(dom_event);
    })
);

cold_event!(
    AnimationIteration,
    AnimationEvent,
    Closure::new(move |dom_event: web_sys::AnimationEvent| {
        trigger_ev::<AnimationIteration>(dom_event);
    })
);

cold_event!(
    AnimationEnd,
    AnimationEvent,
    Closure::new(move |dom_event: web_sys::AnimationEvent| {
        trigger_ev::<AnimationEnd>(dom_event);
    })
);

cold_event!(
    AnimationCancel,
    AnimationEvent,
    Closure::new(move |dom_event: web_sys::AnimationEvent| {
        trigger_ev::<AnimationCancel>(dom_event);
    })
);
