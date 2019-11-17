use super::{backend::*, event::SystemEv};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct CommonEvent { }

#[derive(Default, Clone, PartialEq, Debug)]
pub struct ViewportPosition<T: Clone> {
    pub x: T,
    pub y: T,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MouseButton {
    Primary,
    Secondary,
    Auxiliary,
    Other,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Other
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct DecorationKeys {
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct MouseEvent {
    pub pos: ViewportPosition<i32>,
    pub button: MouseButton,
    pub decoration_keys: DecorationKeys,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Touch {
    pub id: i32,
    pub pos: ViewportPosition<i32>,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct TouchEvent {
    pub touches: Vec<Touch>,
    pub changed_touches: Vec<Touch>,
    pub decoration_keys: DecorationKeys,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct TapEvent {
    pub pos: ViewportPosition<i32>,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct KeyboardEvent {
    pub key_code: u32,
    pub char_code: u32,
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
    pub change: SystemEv<B, CommonEvent>,
    pub submit: SystemEv<B, CommonEvent>,
    pub animation_start: SystemEv<B, CommonEvent>,
    pub animation_iteration: SystemEv<B, CommonEvent>,
    pub animation_end: SystemEv<B, CommonEvent>,
    pub transition_end: SystemEv<B, CommonEvent>,
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
macro_rules! trigger_global_event {
    ($node_ref_mut: expr, $event_name: ident, $data: expr) => {
        let node_ref_mut: $crate::node::NodeRefMut<_> = $node_ref_mut;
        let data = $data;
        let e = match &node_ref_mut {
            $crate::node::NodeRefMut::NativeNode(n) => {
                Some(n.global_events.$event_name.new_event())
            },
            $crate::node::NodeRefMut::VirtualNode(_) => {
                None
            },
            $crate::node::NodeRefMut::ComponentNode(n) => {
                Some(n.global_events.$event_name.new_event())
            },
            $crate::node::NodeRefMut::TextNode(_) => {
                None
            },
        };
        if let Some(e) = e {
            e.trigger(node_ref_mut, data);
        }
    }
}

#[macro_export]
macro_rules! bubble_global_event {
    ($node_ref_mut: expr, $event_name: ident, $data: expr) => {
        let mut node_ref_mut: $crate::node::NodeRefMut<_> = $node_ref_mut;
        let data = $data;
        {
            trigger_global_events!(node_ref_mut.duplicate(), $event_name, data);
        }
        let mut parent = node_ref_mut.to_ref().parent();
        loop {
            match parent.clone() {
                Some(p) => {
                    let node_ref_mut = p.borrow_mut_with(&mut node_ref_mut);
                    parent = node_ref_mut.to_ref().parent();
                    trigger_global_event!(node_ref_mut, $event_name, data);
                },
                None => break
            }
        }
    }
}

#[macro_export]
macro_rules! bubble_composed_global_event {
    ($node_ref_mut: expr, $event_name: ident, $data: expr) => {
        let mut node_ref_mut: $crate::node::NodeRefMut<_> = $node_ref_mut;
        let data = $data;
        {
            trigger_global_event!(node_ref_mut.duplicate(), $event_name, data);
        }
        let mut parent = node_ref_mut.to_ref().composed_parent();
        loop {
            match parent.clone() {
                Some(p) => {
                    let node_ref_mut = p.borrow_mut_with(&mut node_ref_mut);
                    parent = node_ref_mut.to_ref().composed_parent();
                    trigger_global_event!(node_ref_mut, $event_name, data);
                },
                None => break
            }
        }
    }
}
