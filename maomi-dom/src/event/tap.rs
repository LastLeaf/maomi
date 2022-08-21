use wasm_bindgen::{prelude::*, JsCast};
use maomi::backend::tree::{ForestNodeRc, ForestToken};

use crate::DomGeneralElement;
use super::{DomEventRegister, BubbleEvent, utils, touch::TouchIdentifier};

const CANCEL_TAP_DIST: i32 = 5;
const LONG_TAP_TIME_MS: i32 = 500;

thread_local! {
    pub(super) static TOUCH_TRACKER: std::cell::RefCell<TouchTracker>  = Default::default();
}

pub(crate) fn remove_element_touch_state(target: &ForestToken) {
    TOUCH_TRACKER.with(|this| {
        this.borrow_mut().interrupt_by_elem(target)
    })
}

#[derive(Default)]
pub(super) struct TouchTracker {
    cur: Vec<CurrentTouch>,
    touch_mode: bool,
}

struct CurrentTouch {
    identifier: TouchIdentifier,
    target: ForestToken,
    client_x: i32,
    client_y: i32,
    #[allow(dead_code)]
    long_tap_cb: Option<Closure<dyn FnMut()>>,
    long_tap_cb_id: i32,
}

impl TouchTracker {
    pub(super) fn touch_mode(&self) -> bool {
        self.touch_mode
    }

    pub(super) fn add(
        &mut self,
        identifier: TouchIdentifier,
        target: ForestNodeRc<DomGeneralElement>,
        client_x: i32,
        client_y: i32,
        touch_mode: bool,
    ) {
        if touch_mode {
            self.touch_mode = true;
        }
        let (long_tap_cb, long_tap_cb_id) = crate::WINDOW.with(move |window| {
            let cb = Closure::new(move || {
                TOUCH_TRACKER.with(|this| {
                    this.borrow_mut().trigger_long_tap(identifier);
                })
            });
            let cb_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                LONG_TAP_TIME_MS,
            );
            match cb_id {
                Err(err) => {
                    crate::log_js_error(&err);
                    log::error!("Setup long tap handler failed.");
                    (None, 0)
                }
                Ok(cb_id) => {
                    (Some(cb), cb_id)
                }
            }
        });
        self.cur.push(CurrentTouch {
            identifier,
            target: target.token(),
            client_x,
            client_y,
            long_tap_cb,
            long_tap_cb_id,
        });
    }

    fn trigger_long_tap(&mut self, identifier: TouchIdentifier) {
        if let Some((i, t)) = self.cur.iter_mut().enumerate().find(|(_, x)| x.identifier == identifier) {
            t.long_tap_cb = None;
            // generate long_tap event
            let mut ev = TapEvent {
                propagation_stopped: false,
                default_prevented: false,
                client_x: t.client_x,
                client_y: t.client_y,
            };
            if let Some(target) = unsafe { t.target.unsafe_resolve_token() } {
                utils::bubble_event::<LongTap>(target, &mut ev);
            }
            if ev.default_prevented {
                self.cur.swap_remove(i);
            }
        }
    }

    pub(super) fn update(
        &mut self,
        identifier: TouchIdentifier,
        client_x: i32,
        client_y: i32,
    ) {
        if let Some((i, t)) = self.cur.iter_mut().enumerate().find(|(_, x)| x.identifier == identifier) {
            if (t.client_x - client_x).abs() > CANCEL_TAP_DIST
                || (t.client_y - client_y).abs() > CANCEL_TAP_DIST {
                if t.long_tap_cb.is_some() {
                    crate::WINDOW.with(|window| {
                        window.clear_timeout_with_handle(t.long_tap_cb_id);
                    });
                }
                // generate cancel_tap event
                let mut ev = TapEvent {
                    propagation_stopped: false,
                    default_prevented: false,
                    client_x: t.client_x,
                    client_y: t.client_y,
                };
                if let Some(target) = unsafe { t.target.unsafe_resolve_token() } {
                    utils::bubble_event::<CancelTap>(target, &mut ev);
                }
                self.cur.swap_remove(i);
            }
        }
    }

    pub(super) fn remove(
        &mut self,
        identifier: TouchIdentifier,
    ) {
        if let Some((i, t)) = self.cur.iter_mut().enumerate().find(|(_, x)| x.identifier == identifier) {
            if t.long_tap_cb.is_some() {
                crate::WINDOW.with(|window| {
                    window.clear_timeout_with_handle(t.long_tap_cb_id);
                });
            }
            // generate tap event
            let mut ev = TapEvent {
                propagation_stopped: false,
                default_prevented: false,
                client_x: t.client_x,
                client_y: t.client_y,
            };
            if let Some(target) = unsafe { t.target.unsafe_resolve_token() } {
                utils::bubble_event::<Tap>(target, &mut ev);
            }
            self.cur.swap_remove(i);
        }
        if self.cur.len() == 0 {
            self.touch_mode = false;
        }
    }

    pub(super) fn interrupt(
        &mut self,
        identifier: TouchIdentifier,
    ) {
        if let Some((i, t)) = self.cur.iter_mut().enumerate().find(|(_, x)| x.identifier == identifier) {
            if t.long_tap_cb.is_some() {
                crate::WINDOW.with(|window| {
                    window.clear_timeout_with_handle(t.long_tap_cb_id);
                });
            }
            self.cur.swap_remove(i);
        }
        if self.cur.len() == 0 {
            self.touch_mode = false;
        }
    }

    pub fn interrupt_by_elem(
        &mut self,
        forest_token: &ForestToken,
    ) {
        if let Some((i, t)) = self.cur.iter_mut().enumerate().find(|(_, x)| x.target.stable_addr() == forest_token.stable_addr()) {
            if t.long_tap_cb.is_some() {
                crate::WINDOW.with(|window| {
                    window.clear_timeout_with_handle(t.long_tap_cb_id);
                });
            }
            self.cur.swap_remove(i);
        }
        if self.cur.len() == 0 {
            self.touch_mode = false;
        }
    }
}

/// A tap event
///
/// Tap events are generated from DOM `touch*` or `mouse*` events automatically.
pub struct TapEvent {
    propagation_stopped: bool,
    default_prevented: bool,
    client_x: i32,
    client_y: i32,  
}

impl TapEvent {
    pub fn client_x(&self) -> i32 {
        self.client_x
    }

    pub fn client_y(&self) -> i32 {
        self.client_y
    }
}

impl BubbleEvent for TapEvent {
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

hot_event!(Tap, tap, TapEvent);
hot_event!(LongTap, long_tap, TapEvent);
hot_event!(CancelTap, cancel_tap, TapEvent);
