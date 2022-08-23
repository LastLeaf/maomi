use wasm_bindgen::{prelude::*, JsCast};

use super::{BubbleEvent, ColdEventItem, DomEventRegister};
use crate::DomGeneralElement;

/// The mouse-related event detail
#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    propagation_stopped: bool,
    default_prevented: bool,
    dom_event: web_sys::MouseEvent,
}

/// A mouse button
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    /// The main button, i.e. left button
    Main,
    /// The auxiliary button, i.e. middle button or wheel button
    Auxiliary,
    /// The secondary button, i.e. right button
    Secondary,
    /// The fourth button, i.e. history-back button
    Fourth,
    /// The fifth button, i.e. history-forward button
    Fifth,
    /// Other unknwon button
    Unknown(i16),
}

impl MouseEvent {
    /// Get the button that triggers `mouse_down` or `mouse_up`
    #[inline]
    pub fn button(&self) -> MouseButton {
        match self.dom_event.button() {
            0 => MouseButton::Main,
            1 => MouseButton::Auxiliary,
            2 => MouseButton::Secondary,
            3 => MouseButton::Fourth,
            4 => MouseButton::Fifth,
            x => MouseButton::Unknown(x),
        }
    }

    /// Check whether keyboard alt key is pressed
    #[inline]
    pub fn alt_key(&self) -> bool {
        self.dom_event.alt_key()
    }

    /// Check whether keyboard ctrl key is pressed
    #[inline]
    pub fn ctrl_key(&self) -> bool {
        self.dom_event.ctrl_key()
    }

    /// Check whether keyboard meta key is pressed
    #[inline]
    pub fn meta_key(&self) -> bool {
        self.dom_event.meta_key()
    }

    /// Check whether keyboard shift key is pressed
    #[inline]
    pub fn shift_key(&self) -> bool {
        self.dom_event.shift_key()
    }

    /// Get the x-position reletive to the viewport
    #[inline]
    pub fn client_x(&self) -> i32 {
        self.dom_event.client_x()
    }

    /// Get the y-position reletive to the viewport
    #[inline]
    pub fn client_y(&self) -> i32 {
        self.dom_event.client_y()
    }
}

impl BubbleEvent for MouseEvent {
    #[inline]
    fn stop_propagation(&mut self) {
        if self.propagation_stopped {
            return;
        };
        self.propagation_stopped = true;
        self.dom_event.stop_propagation()
    }

    #[inline]
    fn propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }

    #[inline]
    fn prevent_default(&mut self) {
        if self.default_prevented {
            return;
        };
        self.default_prevented = true;
        self.dom_event.prevent_default()
    }

    #[inline]
    fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}

fn trigger_ev<T: DomEventRegister<Detail = MouseEvent>>(dom_event: web_sys::MouseEvent) {
    let target = dom_event
        .target()
        .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref()));
    if let Some(n) = target {
        if let DomGeneralElement::Element(x) = &mut *n.borrow_mut() {
            T::trigger(
                x,
                &mut MouseEvent {
                    propagation_stopped: false,
                    default_prevented: false,
                    dom_event,
                },
            );
        }
    }
}

cold_event!(
    MouseDown,
    MouseEvent,
    "mousedown",
    Closure::new(move |dom_event: web_sys::MouseEvent| {
        trigger_ev::<MouseDown>(dom_event);
    })
);

cold_event!(
    MouseUp,
    MouseEvent,
    "mouseup",
    Closure::new(move |dom_event: web_sys::MouseEvent| {
        trigger_ev::<MouseUp>(dom_event);
    })
);

cold_event!(
    MouseMove,
    MouseEvent,
    "mousemove",
    Closure::new(move |dom_event: web_sys::MouseEvent| {
        trigger_ev::<MouseMove>(dom_event);
    })
);

cold_event!(
    MouseEnter,
    MouseEvent,
    "mouseenter",
    Closure::new(move |dom_event: web_sys::MouseEvent| {
        trigger_ev::<MouseEnter>(dom_event);
    })
);

cold_event!(
    MouseLeave,
    MouseEvent,
    "mouseleave",
    Closure::new(move |dom_event: web_sys::MouseEvent| {
        trigger_ev::<MouseLeave>(dom_event);
    })
);
