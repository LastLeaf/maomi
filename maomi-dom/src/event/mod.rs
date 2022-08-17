use wasm_bindgen::prelude::*;
use maomi::event::EventHandler;

use crate::base_element::DomElement;

mod touch;
pub use touch::SingleTouch;

pub(crate) struct DomListeners {
    touch: Closure::<dyn Fn(web_sys::TouchEvent)>,
}

impl DomListeners {
    pub(crate) fn new(element: &web_sys::Element) -> Result<(), JsValue> {
        Ok(Self {
            touch: touch::init_dom_listeners(element)?,
        })
    }
}

pub(crate) struct BubbleEventList {
    touch_start: Option<Box<dyn 'static + Fn(&mut SingleTouch)>>,
    touch_move: Option<Box<dyn 'static + Fn(&mut SingleTouch)>>,
    touch_end: Option<Box<dyn 'static + Fn(&mut SingleTouch)>>,
    touch_cancel: Option<Box<dyn 'static + Fn(&mut SingleTouch)>>,
}

pub(crate) trait EventHandlerMap {
    // TODO
}

// A non-bubble DOM event
pub struct DomEvent<D, M: EventHandlerMap> {
    // handler_fn: Option<Box<dyn 'static + Fn(&mut D)>>,
}

impl<D, M: EventHandlerMap> DomEvent<D, M> {
    pub(crate) fn new() -> Self {
        // Self { handler_fn: None }
    }

    pub(crate) fn trigger(&self, data: &mut D) {
        // if let Some(f) = &self.handler_fn {
        //     f(data);
        // }
    }
}

impl<D, M: EventHandlerMap> EventHandler<D> for DomEvent<D, M> {
    type UpdateContext = DomElement;

    fn set_handler_fn(dest: &mut Self, handler_fn: Box<dyn 'static + Fn(&mut D)>, ctx: &mut DomElement) {
        // dest.handler_fn = Some(handler_fn);
    }
}
