use wasm_bindgen::{prelude::*, JsCast};

pub(super) fn init_dom_listeners(element: &web_sys::Element) -> Result<Closure::<dyn Fn(web_sys::TouchEvent)>, JsValue> {
    let cb = Closure::new(move |ev: web_sys::TouchEvent| {
        let changed_touches = ev.changed_touches();
        let len = changed_touches.length();
        for index in 0..len {
            let touch = changed_touches.get(index).unwrap();
            let mut single_touch = SingleTouch {
                identifier: TouchIdentifier(touch.identifier()),
                client_x: touch.client_x(),
                client_y: touch.client_y(),
            };
            let target = touch.target();
            // TODO
        }
    }).into_js_value();
    element.add_event_listener_with_callback_and_bool(
        "touchstart",
        cb.as_ref().unchecked_ref(),
        true,
    )?;
    Ok(cb)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TouchIdentifier(i32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SingleTouch {
    identifier: TouchIdentifier,
    client_x: i32,
    client_y: i32,
}
