use wasm_bindgen::{prelude::*, JsCast};

use super::{DomEventRegister, BubbleEvent};

pub(super) fn init_dom_listeners(element: &web_sys::Element) -> Result<Closure::<dyn Fn(web_sys::TouchEvent)>, JsValue> {
    let cb = Closure::new(move |ev: web_sys::TouchEvent| {
        let changed_touches = ev.changed_touches();
        let len = changed_touches.length();
        for index in 0..len {
            let touch = changed_touches.get(index).unwrap();
            let mut single_touch = SingleTouchEvent {
                propagation_stopped: false,
                default_prevented: false,
                identifier: TouchIdentifier(touch.identifier()),
                client_x: touch.client_x(),
                client_y: touch.client_y(),
                dom_touch: touch,
            };
            // let target = touch.target();
            // TODO
        }
    });
    element.add_event_listener_with_callback_and_bool(
        "touchstart",
        cb.as_ref().unchecked_ref(),
        true,
    )?;
    Ok(cb)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchIdentifier(i32);

#[derive(Debug, Clone, PartialEq)]
pub struct SingleTouchEvent {
    propagation_stopped: bool,
    default_prevented: bool,
    identifier: TouchIdentifier,
    client_x: i32,
    client_y: i32,
    dom_touch: web_sys::Touch,
}

impl BubbleEvent for SingleTouchEvent {
    fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
        // TODO stop in backend
    }

    fn prevent_default(&mut self) {
        self.default_prevented = true;
        // TODO stop in backend
    }
}

impl SingleTouchEvent {
    /// Get the identifier for a series of touch events
    pub fn identifier(&self) -> TouchIdentifier {
        self.identifier
    }

    /// Get the x-position reletive to the viewport
    pub fn client_x(&self) -> i32 {
        self.client_x
    }

    /// Get the y-position reletive to the viewport
    pub fn client_y(&self) -> i32 {
        self.client_y
    }
}

macro_rules! hot_event {
    ($t:ident, $field:ident, $detail:ty) => {
        pub struct $t {}

        impl DomEventRegister for $t {
            type Detail = $detail;

            fn bind(target: &mut crate::base_element::DomElement, f: Box<dyn 'static + Fn(&mut Self::Detail)>) {
                let list = target.hot_event_list_mut();
                list.$field = Some(f);
            }

            fn trigger(target: &mut crate::base_element::DomElement, detail: &mut Self::Detail) {
                let list = target.hot_event_list_mut();
                if let Some(f) = &list.$field {
                    f(detail);
                }
            }
        }
    };
}

hot_event!(TouchStart, touch_start, SingleTouchEvent);
hot_event!(TouchMove, touch_move, SingleTouchEvent);
hot_event!(TouchEnd, touch_end, SingleTouchEvent);
hot_event!(TouchCancel, touch_cancel, SingleTouchEvent);
