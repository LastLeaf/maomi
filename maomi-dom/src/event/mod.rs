//! The event definition.
//! 
//! Besides DOM events, this module also provides `tap` events,
//! which are generated from mouse or touch events.
//! * `tap` event refers to a finger tap (or mouse click);
//! * `cancal_tap` event refers to a finger (or mouse) that moved;
//! * `long_tap` event refers to a long finger tap (or mouse hold).
//! Calling `detail.preventDefault()` in the `long_tap` handler will prevent the `tap` event.

use maomi::event::EventHandler;
use std::marker::PhantomData;
use wasm_bindgen::{prelude::*, JsCast};

use crate::base_element::DomElement;

#[macro_use]
mod utils;
pub(crate) mod tap;
pub use tap::TapEvent;
pub(crate) mod touch;
pub use touch::TouchEvent;
pub(crate) mod mouse;
pub use mouse::{MouseButton, MouseEvent};
pub(crate) mod scroll;
pub use scroll::ScrollEvent;
pub(crate) mod animation;
pub use animation::AnimationEvent;
pub(crate) mod transition;
pub use transition::TransitionEvent;
pub(crate) mod form;
pub use form::{SubmitEvent, ChangeEvent, InputEvent};

pub(crate) struct DomListeners {
    #[allow(dead_code)]
    touch: touch::TouchEventCbs,
}

impl DomListeners {
    pub(crate) fn new(element: &web_sys::Element) -> Self {
        Self {
            touch: touch::init_dom_listeners(element),
        }
    }
}

// hot event list is usually used to store popular events and bubble events
#[derive(Default)]
pub(crate) struct HotEventList {
    touch_start: Option<Box<dyn 'static + Fn(&mut TouchEvent)>>,
    touch_move: Option<Box<dyn 'static + Fn(&mut TouchEvent)>>,
    touch_end: Option<Box<dyn 'static + Fn(&mut TouchEvent)>>,
    touch_cancel: Option<Box<dyn 'static + Fn(&mut TouchEvent)>>,
    tap: Option<Box<dyn 'static + Fn(&mut TapEvent)>>,
    long_tap: Option<Box<dyn 'static + Fn(&mut TapEvent)>>,
    cancel_tap: Option<Box<dyn 'static + Fn(&mut TapEvent)>>,
}

// code event list is slow to visit but memory-efficient
pub(crate) type ColdEventList = Vec<ColdEventItem>;

pub(crate) enum ColdEventItem {
    BindingEventListener(
        &'static str,
        Closure<dyn Fn(web_sys::Event)>,
    ),
    MouseDown(
        Box<dyn 'static + Fn(&mut MouseEvent)>,
        Closure<dyn Fn(web_sys::MouseEvent)>,
    ),
    MouseUp(
        Box<dyn 'static + Fn(&mut MouseEvent)>,
        Closure<dyn Fn(web_sys::MouseEvent)>,
    ),
    MouseMove(
        Box<dyn 'static + Fn(&mut MouseEvent)>,
        Closure<dyn Fn(web_sys::MouseEvent)>,
    ),
    MouseEnter(
        Box<dyn 'static + Fn(&mut MouseEvent)>,
        Closure<dyn Fn(web_sys::MouseEvent)>,
    ),
    MouseLeave(
        Box<dyn 'static + Fn(&mut MouseEvent)>,
        Closure<dyn Fn(web_sys::MouseEvent)>,
    ),
    Click(
        Box<dyn 'static + Fn(&mut MouseEvent)>,
        Closure<dyn Fn(web_sys::MouseEvent)>,
    ),
    Scroll(
        Box<dyn 'static + Fn(&mut ScrollEvent)>,
        Closure<dyn Fn(web_sys::Event)>,
    ),
    AnimationStart(
        Box<dyn 'static + Fn(&mut AnimationEvent)>,
        Closure<dyn Fn(web_sys::AnimationEvent)>,
    ),
    AnimationIteration(
        Box<dyn 'static + Fn(&mut AnimationEvent)>,
        Closure<dyn Fn(web_sys::AnimationEvent)>,
    ),
    AnimationEnd(
        Box<dyn 'static + Fn(&mut AnimationEvent)>,
        Closure<dyn Fn(web_sys::AnimationEvent)>,
    ),
    AnimationCancel(
        Box<dyn 'static + Fn(&mut AnimationEvent)>,
        Closure<dyn Fn(web_sys::AnimationEvent)>,
    ),
    TransitionRun(
        Box<dyn 'static + Fn(&mut TransitionEvent)>,
        Closure<dyn Fn(web_sys::TransitionEvent)>,
    ),
    TransitionStart(
        Box<dyn 'static + Fn(&mut TransitionEvent)>,
        Closure<dyn Fn(web_sys::TransitionEvent)>,
    ),
    TransitionEnd(
        Box<dyn 'static + Fn(&mut TransitionEvent)>,
        Closure<dyn Fn(web_sys::TransitionEvent)>,
    ),
    TransitionCancel(
        Box<dyn 'static + Fn(&mut TransitionEvent)>,
        Closure<dyn Fn(web_sys::TransitionEvent)>,
    ),
    Submit(
        Box<dyn 'static + Fn(&mut SubmitEvent)>,
        Closure<dyn Fn(web_sys::SubmitEvent)>,
    ),
    Change(
        Box<dyn 'static + Fn(&mut ChangeEvent)>,
        Closure<dyn Fn(web_sys::Event)>,
    ),
    Input(
        Box<dyn 'static + Fn(&mut InputEvent)>,
        Closure<dyn Fn(web_sys::InputEvent)>,
    ),
}

impl ColdEventItem {
    pub(crate) fn apply(&self, elem: &web_sys::Element) {
        let (ev_name, cb): (&str, &JsValue) = match self {
            Self::BindingEventListener(name, cb) => (name, cb.as_ref()),
            Self::MouseDown(_, cb) => ("mousedown", cb.as_ref()),
            Self::MouseUp(_, cb) => ("mouseup", cb.as_ref()),
            Self::MouseMove(_, cb) => ("mousemove", cb.as_ref()),
            Self::MouseEnter(_, cb) => ("mouseenter", cb.as_ref()),
            Self::MouseLeave(_, cb) => ("mouseleave", cb.as_ref()),
            Self::Click(_, cb) => ("click", cb.as_ref()),
            Self::Scroll(_, cb) => ("scroll", cb.as_ref()),
            Self::AnimationStart(_, cb) => ("animationstart", cb.as_ref()),
            Self::AnimationIteration(_, cb) => ("animationiteration", cb.as_ref()),
            Self::AnimationEnd(_, cb) => ("animationend", cb.as_ref()),
            Self::AnimationCancel(_, cb) => ("animationcancel", cb.as_ref()),
            Self::TransitionRun(_, cb) => ("transitionrun", cb.as_ref()),
            Self::TransitionStart(_, cb) => ("transitionstart", cb.as_ref()),
            Self::TransitionEnd(_, cb) => ("transitionend", cb.as_ref()),
            Self::TransitionCancel(_, cb) => ("transitioncancel", cb.as_ref()),
            Self::Submit(_, cb) => ("submit", cb.as_ref()),
            Self::Change(_, cb) => ("change", cb.as_ref()),
            Self::Input(_, cb) => ("input", cb.as_ref()),
        };
        // Seriously, there should be a removal on the element dropped,
        // otherwise the closure is lost and a js error is displayed in console.
        // However, most events do not trigger after element removal,
        // so here just do no removal.
        if let Err(err) = elem.add_event_listener_with_callback(ev_name, cb.unchecked_ref()) {
            crate::log_js_error(&err);
            log::error!(
                "Failed adding listener for event {:?}. This event will not be triggered.",
                ev_name
            );
        }
    }
}

/// A DOM event that can be binded.
pub trait DomEventRegister {
    /// The event detailed type.
    type Detail;

    /// Bind the event.
    /// 
    /// It is auto-managed by the `#[component]` .
    /// Do not touch unless you know how it works exactly.
    fn bind(target: &mut DomElement, f: Box<dyn 'static + Fn(&mut Self::Detail)>);

    /// Trigger the event.
    fn trigger(target: &mut DomElement, detail: &mut Self::Detail);
}

/// A DOM event
pub struct DomEvent<M: DomEventRegister> {
    _phantom: PhantomData<M>,
}

impl<M: DomEventRegister> Default for DomEvent<M> {
    #[inline]
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<M: DomEventRegister> EventHandler<M::Detail> for DomEvent<M> {
    type UpdateContext = DomElement;

    #[inline]
    fn set_handler_fn(
        _dest: &mut Self,
        handler_fn: Box<dyn 'static + Fn(&mut M::Detail)>,
        ctx: &mut DomElement,
    ) {
        M::bind(ctx, handler_fn);
    }
}

/// A DOM event that bubbles.
pub trait BubbleEvent {
    /// Stop bubbling.
    fn stop_propagation(&mut self);
    /// Get whether the bubbling is stopped.
    fn propagation_stopped(&self) -> bool;
    /// Prevent the default DOM operation.
    fn prevent_default(&mut self);
    /// Get whether the default DOM operation is prevented.
    fn default_prevented(&self) -> bool;
}
