use super::{backend::Backend, event::SystemEv};

#[derive(Default)]
pub struct ViewportPosition {
    pub x: f32,
    pub y: f32,
}

pub enum MouseButton {
    Primary,
    Secondary,
    Auxiliary,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Primary
    }
}

#[derive(Default)]
pub struct DecorationKeys {
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}

#[derive(Default)]
pub struct MouseEvent {
    pub pos: ViewportPosition,
    pub button: MouseButton,
    pub decoration_keys: DecorationKeys,
}

#[derive(Default)]
pub struct Touch {
    pub id: usize,
    pub pos: ViewportPosition,
}

#[derive(Default)]
pub struct TouchEvent {
    pub touches: Vec<Touch>,
    pub changed_touches: Vec<Touch>,
    pub decoration_keys: DecorationKeys,
}

#[derive(Default)]
pub struct TapEvent {
    pub pos: ViewportPosition,
}

#[derive(Default)]
pub struct KeyboardEvent {
    pub key_code: usize,
    pub char_code: char,
    pub key: String,
}

#[derive(Default)]
pub struct GlobalEvents<B: Backend> {
    pub click: SystemEv<B, MouseEvent>,
    pub mouse_down: SystemEv<B, MouseEvent>,
    pub mouse_move: SystemEv<B, MouseEvent>,
    pub mouse_up: SystemEv<B, MouseEvent>,
    pub touch_start: SystemEv<B, TouchEvent>,
    pub touch_move: SystemEv<B, TouchEvent>,
    pub touch_end: SystemEv<B, TouchEvent>,
    pub touch_cancel: SystemEv<B, TouchEvent>,
    pub tap: SystemEv<B, TapEvent>,
    pub long_tap: SystemEv<B, TapEvent>,
    pub cancel_tap: SystemEv<B, TapEvent>,
    pub key_down: SystemEv<B, KeyboardEvent>,
    pub key_press: SystemEv<B, KeyboardEvent>,
    pub key_up: SystemEv<B, KeyboardEvent>,
    pub change: SystemEv<B, ()>,
    pub submit: SystemEv<B, ()>,
    pub animation_start: SystemEv<B, ()>,
    pub animation_iteration: SystemEv<B, ()>,
    pub animation_end: SystemEv<B, ()>,
    pub transition_end: SystemEv<B, ()>,
}

impl<B: Backend> GlobalEvents<B> {
    pub(crate) fn new() -> Self {
        Self {
            click: SystemEv::new(),
            mouse_down: SystemEv::new(),
            mouse_move: SystemEv::new(),
            mouse_up: SystemEv::new(),
            touch_start: SystemEv::new(),
            touch_move: SystemEv::new(),
            touch_end: SystemEv::new(),
            touch_cancel: SystemEv::new(),
            tap: SystemEv::new(),
            long_tap: SystemEv::new(),
            cancel_tap: SystemEv::new(),
            key_down: SystemEv::new(),
            key_press: SystemEv::new(),
            key_up: SystemEv::new(),
            change: SystemEv::new(),
            submit: SystemEv::new(),
            animation_start: SystemEv::new(),
            animation_iteration: SystemEv::new(),
            animation_end: SystemEv::new(),
            transition_end: SystemEv::new(),
        }
    }
}

#[macro_export]
macro_rules! trigger_global_events {
    ($node_ref_mut: expr, $event_name: ident, $data: expr) => {
        let node_ref_mut: $crate::node::NodeRefMut<_> = $node_ref_mut;
        let data = $data;
        match node_ref_mut {
            $crate::node::NodeRefMut::NativeNode(n) => {
                n.global_events.$event_name.new_event().trigger(node_ref_mut, data);
            },
            $crate::node::NodeRefMut::VirtualNode(n) => {
                // empty
            },
            $crate::node::NodeRefMut::ComponentNode(n) => {
                n.global_events.$event_name.new_event().trigger(node_ref_mut, data);
            },
            $crate::node::NodeRefMut::TextNode(n) => {
                // empty
            },
        }
    }
}

#[macro_export]
macro_rules! bubble_global_events {
    ($node_ref_mut: expr, $event_name: ident, $data: expr) => {
        // TODO
    }
}

#[macro_export]
macro_rules! bubble_composed_global_events {
    ($node_ref_mut: expr, $event_name: ident, $data: expr) => {
        // TODO
    }
}
