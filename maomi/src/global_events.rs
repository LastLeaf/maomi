use super::{backend::*, event::SystemEv};

/// An event without any content
#[derive(Default, Clone, PartialEq, Debug)]
pub struct CommonEvent { }

/// Position relative to viewport left-top
#[derive(Default, Clone, PartialEq, Debug)]
pub struct ViewportPosition<T: Clone> {
    pub x: T,
    pub y: T,
}

/// Mouse button information
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

/// Decorative keys information
#[derive(Default, Clone, PartialEq, Debug)]
pub struct DecorationKeys {
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}

/// An event with mouse information
#[derive(Default, Clone, PartialEq, Debug)]
pub struct MouseEvent {
    pub pos: ViewportPosition<i32>,
    pub button: MouseButton,
    pub decoration_keys: DecorationKeys,
}

/// Touch information
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Touch {
    pub id: i32,
    pub pos: ViewportPosition<i32>,
}

/// An event with touch information
#[derive(Default, Clone, PartialEq, Debug)]
pub struct TouchEvent {
    pub touches: Vec<Touch>,
    pub changed_touches: Vec<Touch>,
    pub decoration_keys: DecorationKeys,
}

/// Tap information
#[derive(Default, Clone, PartialEq, Debug)]
pub struct TapEvent {
    pub pos: ViewportPosition<i32>,
}

/// An event with keyboard information
#[derive(Default, Clone, PartialEq, Debug)]
pub struct KeyboardEvent {
    pub key_code: u32,
    pub char_code: u32,
    pub key: String,
}

/// Global events list.
/// Global events can be triggered by backend, and can bubble in shadow tree or composed tree.
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

/// Trigger a global event.
/// In most cases, global events should be triggered by backend.
#[macro_export]
macro_rules! trigger_global_event {
    ($node_mut: expr, $event_name: ident, $data: expr) => {
        let node_mut: $crate::node::NodeMut<_> = $node_mut;
        let data = $data;
        let e = match &node_mut {
            $crate::node::NodeMut::NativeNode(n) => {
                Some(n.global_events.$event_name.new_event())
            },
            $crate::node::NodeMut::VirtualNode(_) => {
                None
            },
            $crate::node::NodeMut::ComponentNode(n) => {
                Some(n.global_events.$event_name.new_event())
            },
            $crate::node::NodeMut::TextNode(_) => {
                None
            },
        };
        if let Some(e) = e {
            e.trigger(node_mut, data);
        }
    }
}

/// Trigger a global event, and bubble it in shadow tree.
/// In most cases, global events should be triggered by backend.
#[macro_export]
macro_rules! bubble_global_event {
    ($node_mut: expr, $event_name: ident, $data: expr) => { {
        use $crate::node::MutIterator;
        let mut node_mut: $crate::node::NodeMut<_> = $node_mut;
        let data = $data;
        {
            trigger_global_event!(node_mut.as_mut(), $event_name, data);
        }
        let mut parent = node_mut.ancestors_mut(crate::node::TraversalOrder::ParentLast);
        while let Some(p) = parent.next() {
            trigger_global_event!(p, $event_name, data);
        }
    } }
}

/// Trigger a global event, and bubble it in composed tree.
/// In most cases, global events should be triggered by backend.
#[macro_export]
macro_rules! bubble_composed_global_event {
    ($node_mut: expr, $event_name: ident, $data: expr) => { {
        use $crate::node::MutIterator;
        let mut node_mut: $crate::node::NodeMut<_> = $node_mut;
        let data = $data;
        {
            trigger_global_event!(node_mut.as_mut(), $event_name, data);
        }
        let mut parent = node_mut.composed_ancestors_mut(crate::node::TraversalOrder::ParentLast);
        while let Some(p) = parent.next() {
            trigger_global_event!(p, $event_name, data);
        }
    } }
}
