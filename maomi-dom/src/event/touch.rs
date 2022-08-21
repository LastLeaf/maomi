use maomi::backend::tree::ForestNodeRc;
use wasm_bindgen::{prelude::*, JsCast};

use super::{tap::TOUCH_TRACKER, utils, BubbleEvent, DomEventRegister};
use crate::{DomGeneralElement, DOCUMENT};

pub(super) struct TouchEventCbs {
    root: web_sys::Element,
    touchstart: Closure<dyn Fn(web_sys::TouchEvent)>,
    touchmove: Closure<dyn Fn(web_sys::TouchEvent)>,
    touchend: Closure<dyn Fn(web_sys::TouchEvent)>,
    touchcancel: Closure<dyn Fn(web_sys::TouchEvent)>,
    mousedown: Closure<dyn Fn(web_sys::MouseEvent)>,
    mouseup: Closure<dyn Fn(web_sys::MouseEvent)>,
    mousemove: Closure<dyn Fn(web_sys::MouseEvent)>,
}

impl Drop for TouchEventCbs {
    fn drop(&mut self) {
        self.root
            .remove_event_listener_with_callback(
                "touchstart",
                self.touchstart.as_ref().unchecked_ref(),
            )
            .ok();
        self.root
            .remove_event_listener_with_callback(
                "touchmove",
                self.touchmove.as_ref().unchecked_ref(),
            )
            .ok();
        self.root
            .remove_event_listener_with_callback("touchend", self.touchend.as_ref().unchecked_ref())
            .ok();
        self.root
            .remove_event_listener_with_callback(
                "touchcancel",
                self.touchcancel.as_ref().unchecked_ref(),
            )
            .ok();
        self.root
            .remove_event_listener_with_callback(
                "mousedown",
                self.mousedown.as_ref().unchecked_ref(),
            )
            .ok();
        DOCUMENT.with(|document| {
            document
                .remove_event_listener_with_callback(
                    "mouseup",
                    self.mouseup.as_ref().unchecked_ref(),
                )
                .ok();
            document
                .remove_event_listener_with_callback(
                    "mousemove",
                    self.mousemove.as_ref().unchecked_ref(),
                )
                .ok();
        });
    }
}

fn add_touch_event_listener<T: DomEventRegister<Detail = TouchEvent>>(
    root: &web_sys::Element,
    ev_name: &'static str,
    final_fn: impl 'static + Fn(ForestNodeRc<DomGeneralElement>, TouchEvent),
) -> Closure<dyn Fn(web_sys::TouchEvent)> {
    let cb = Closure::new(move |ev: web_sys::TouchEvent| {
        let changed_touches = ev.changed_touches();
        let len = changed_touches.length();
        let mut propagation_stopped = false;
        let mut default_prevented = false;
        for index in 0..len {
            let touch = changed_touches.get(index).unwrap();
            let target = touch
                .target()
                .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref()));
            if let Some(target) = target {
                let mut single_touch = TouchEvent {
                    propagation_stopped: false,
                    default_prevented: false,
                    identifier: TouchIdentifier(touch.identifier()),
                    client_x: touch.client_x(),
                    client_y: touch.client_y(),
                    dom_touch: touch,
                };
                utils::bubble_event::<T>(target.clone(), &mut single_touch);
                if single_touch.propagation_stopped {
                    propagation_stopped = true;
                }
                if single_touch.default_prevented {
                    default_prevented = true;
                }
                final_fn(target, single_touch);
            }
        }
        if propagation_stopped {
            ev.stop_propagation()
        };
        if default_prevented {
            ev.prevent_default()
        };
    });
    if let Err(err) = root.add_event_listener_with_callback(ev_name, cb.as_ref().unchecked_ref()) {
        crate::log_js_error(&err);
        log::error!(
            "Failed adding listener for event {:?}. This event will not be triggered.",
            ev_name
        );
    }
    cb
}

fn add_mouse_event_listener(
    root: &web_sys::Element,
    ev_name: &'static str,
    final_fn: impl 'static + Fn(ForestNodeRc<DomGeneralElement>, web_sys::MouseEvent),
) -> Closure<dyn Fn(web_sys::MouseEvent)> {
    let cb = Closure::new(move |ev: web_sys::MouseEvent| {
        let target = ev
            .target()
            .and_then(|x| crate::DomElement::from_event_dom_elem(x.unchecked_ref()));
        if let Some(target) = target {
            final_fn(target, ev);
        }
    });
    if let Err(err) = root.add_event_listener_with_callback(ev_name, cb.as_ref().unchecked_ref()) {
        crate::log_js_error(&err);
        log::error!(
            "Failed adding listener for event {:?}. This event will not be triggered.",
            ev_name
        );
    }
    cb
}

fn add_mouse_event_listener_without_target(
    root: &web_sys::Element,
    ev_name: &'static str,
    final_fn: impl 'static + Fn(web_sys::MouseEvent),
) -> Closure<dyn Fn(web_sys::MouseEvent)> {
    let cb = Closure::new(final_fn);
    if let Err(err) = root.add_event_listener_with_callback(ev_name, cb.as_ref().unchecked_ref()) {
        crate::log_js_error(&err);
        log::error!(
            "Failed adding listener for event {:?}. This event will not be triggered.",
            ev_name
        );
    }
    cb
}

pub(super) fn init_dom_listeners(root: &web_sys::Element) -> TouchEventCbs {
    let touchstart = add_touch_event_listener::<TouchStart>(root, "touchstart", |target, ev| {
        TOUCH_TRACKER.with(|tracker| {
            let tracker = &mut tracker.borrow_mut();
            tracker.add(ev.identifier, target, ev.client_x, ev.client_y, true);
        })
    });
    let touchmove = add_touch_event_listener::<TouchMove>(root, "touchmove", |_, ev| {
        TOUCH_TRACKER.with(|tracker| {
            let tracker = &mut tracker.borrow_mut();
            tracker.update(ev.identifier, ev.client_x, ev.client_y);
        })
    });
    let touchend = add_touch_event_listener::<TouchEnd>(root, "touchend", |_, ev| {
        TOUCH_TRACKER.with(|tracker| {
            let tracker = &mut tracker.borrow_mut();
            tracker.remove(ev.identifier);
        })
    });
    let touchcancel = add_touch_event_listener::<TouchCancel>(root, "touchcancel", |_, ev| {
        TOUCH_TRACKER.with(|tracker| {
            let tracker = &mut tracker.borrow_mut();
            tracker.interrupt(ev.identifier);
        })
    });
    let mousedown = add_mouse_event_listener(root, "mousedown", |target, ev| {
        if ev.button() != 0 {
            return;
        };
        TOUCH_TRACKER.with(|tracker| {
            let tracker = &mut tracker.borrow_mut();
            if !tracker.touch_mode() {
                tracker.add(
                    TouchIdentifier(0),
                    target,
                    ev.client_x(),
                    ev.client_y(),
                    false,
                );
            }
        })
    });
    let mouseup = DOCUMENT.with(|document| {
        add_mouse_event_listener_without_target(document.unchecked_ref(), "mouseup", |ev| {
            if ev.button() != 0 {
                return;
            };
            TOUCH_TRACKER.with(|tracker| {
                let tracker = &mut tracker.borrow_mut();
                if !tracker.touch_mode() {
                    tracker.update(TouchIdentifier(0), ev.client_x(), ev.client_y());
                    tracker.remove(TouchIdentifier(0));
                }
            })
        })
    });
    let mousemove = DOCUMENT.with(|document| {
        add_mouse_event_listener_without_target(document.unchecked_ref(), "mousemove", |ev| {
            TOUCH_TRACKER.with(|tracker| {
                let tracker = &mut tracker.borrow_mut();
                if !tracker.touch_mode() {
                    tracker.update(TouchIdentifier(0), ev.client_x(), ev.client_y());
                }
            })
        })
    });
    TouchEventCbs {
        root: root.clone(),
        touchstart,
        touchmove,
        touchend,
        touchcancel,
        mousedown,
        mouseup,
        mousemove,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchIdentifier(i32);

/// A touch event that contains only one changed touch information
#[derive(Debug, Clone, PartialEq)]
pub struct TouchEvent {
    propagation_stopped: bool,
    default_prevented: bool,
    identifier: TouchIdentifier,
    client_x: i32,
    client_y: i32,
    dom_touch: web_sys::Touch,
}

impl BubbleEvent for TouchEvent {
    fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    fn propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }

    fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}

impl TouchEvent {
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

hot_event!(TouchStart, touch_start, TouchEvent);
hot_event!(TouchMove, touch_move, TouchEvent);
hot_event!(TouchEnd, touch_end, TouchEvent);
hot_event!(TouchCancel, touch_cancel, TouchEvent);
