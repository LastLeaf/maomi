use std::ops::Deref;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::pin::Pin;
use web_sys::*;
use futures::prelude::*;
use futures::task::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::global_events;
use crate::global_events::*;
use crate::node::{NodeWeak};
use super::*;

const ELEMENT_INNER_ID_MAGIC: u32 = 1234567890;

thread_local! {
    static DOCUMENT: Document = window().unwrap().document().unwrap();
    static ELEMENT_MAP: RefCell<HashMap<usize, NodeWeak<Dom>>> = RefCell::new(HashMap::new());
    static ELEMENT_INNER_ID_INC: Cell<usize> = Cell::new(0);
}

pub struct TimeoutHandler {
    _cb: Closure<dyn FnMut()>,
    id: i32,
}
impl Drop for TimeoutHandler {
    fn drop(&mut self) {
        window().unwrap().clear_timeout_with_handle(self.id);
    }
}
pub fn set_timeout<F: 'static + FnOnce()>(cb: F, timeout: i32) -> TimeoutHandler {
    let cb_wrapper = Closure::once(Box::new(cb) as Box<dyn FnOnce()>);
    let id = window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(cb_wrapper.as_ref().unchecked_ref(), timeout).unwrap();
    TimeoutHandler {
        _cb: cb_wrapper,
        id,
    }
}

pub struct Timeout {
    _timeout_handler: Option<TimeoutHandler>,
    done: Rc<Cell<bool>>,
    waker: Rc<Cell<Option<Waker>>>,
}
impl Future for Timeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.done.get() {
            Poll::Ready(())
        } else {
            self.waker.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}
impl Timeout {
    pub fn new(timeout: i32) -> Self {
        let done = Rc::new(Cell::new(false));
        let done2 = done.clone();
        let waker = Rc::new(Cell::new(None));
        let waker2 = waker.clone();
        let timeout_handler = set_timeout(move || {
            done2.set(true);
            match waker2.take() {
                Some(waker) => {
                    let waker: Waker = waker;
                    waker.wake();
                },
                None => { }
            }
        }, timeout);
        Self {
            _timeout_handler: Some(timeout_handler),
            done,
            waker,
        }
    }
}

pub struct DomElement {
    node: Option<Element>,
    inner_id: usize,
    pending_node_weak: Option<NodeWeak<Dom>>,
}
impl DomElement {
    fn new(tag_name: &'static str, prerendering: bool) -> Self {
        let inner_id = ELEMENT_INNER_ID_INC.with(|inner_id_inc| {
            let inner_id = inner_id_inc.take();
            inner_id_inc.set(inner_id + 1);
            inner_id
        });
        DomElement {
            node: if prerendering {
                None
            } else {
                DOCUMENT.with(|document| {
                    Some(document.create_element(tag_name).unwrap())
                })
            },
            inner_id,
            pending_node_weak: None,
        }
    }
    pub fn dom_node(&self) -> &Element {
        self.node.as_ref().expect("Dom node cannot be accessed while prerendering")
    }
    fn add_node_to_hash_map(&self, node_weak: NodeWeak<Dom>) {
        js_sys::Reflect::set_u32(self.node.as_ref().unwrap(), ELEMENT_INNER_ID_MAGIC, &JsValue::from_f64(self.inner_id as f64)).unwrap();
        ELEMENT_MAP.with(|element_map| {
            element_map.borrow_mut().insert(self.inner_id, node_weak);
        });
    }
    fn apply_pending_node_weak(&mut self) {
        if let Some(node_weak) = self.pending_node_weak.take() {
            self.add_node_to_hash_map(node_weak);
        }
    }
}
impl Drop for DomElement {
    fn drop(&mut self) {
        ELEMENT_MAP.with(|element_map| {
            let mut element_map = element_map.borrow_mut();
            element_map.remove(&self.inner_id);
        });
    }
}
impl Deref for DomElement {
    type Target = Element;
    fn deref(&self) -> &Element {
        self.dom_node()
    }
}
impl BackendElement for DomElement {
    type Backend = Dom;
    fn bind_node_weak(&mut self, node_weak: NodeWeak<Dom>) {
        if self.node.is_some() {
            self.add_node_to_hash_map(node_weak);
        } else {
            self.pending_node_weak = Some(node_weak);
        }
    }
    fn append_list(&self, children: Vec<BackendNodeRef<Dom>>) {
        if let Some(node) = self.node.as_ref() {
            DOCUMENT.with(|document| {
                let frag = document.create_document_fragment();
                for child in children {
                    let dom_node: &Node = match child {
                        BackendNodeRef::Element(x) => x.dom_node(),
                        BackendNodeRef::TextNode(x) => x.dom_node(),
                    };
                    frag.append_child(dom_node).unwrap();
                }
                node.append_child(&frag).unwrap();
            })
        }
    }
    fn insert_list_before<'a>(&'a self, children: Vec<BackendNodeRef<Dom>>, before: Option<BackendNodeRef<'a, Dom>>) {
        if let Some(node) = self.node.as_ref() {
            DOCUMENT.with(|document| {
                let frag = document.create_document_fragment();
                for child in children {
                    let dom_node: &Node = match child {
                        BackendNodeRef::Element(x) => x.dom_node(),
                        BackendNodeRef::TextNode(x) => x.dom_node(),
                    };
                    frag.append_child(dom_node).unwrap();
                }
                node.insert_before(&frag, before.as_ref().map(|x| {
                    let n: &Node = match x {
                        BackendNodeRef::Element(x) => x.dom_node(),
                        BackendNodeRef::TextNode(x) => x.dom_node(),
                    };
                    n
                })).unwrap();
            })
        }
    }
    fn remove_list(&self, children: Vec<BackendNodeRef<Dom>>) {
        if let Some(node) = self.node.as_ref() {
            for child in children {
                let dom_node: &Node = match child {
                    BackendNodeRef::Element(x) => x.dom_node(),
                    BackendNodeRef::TextNode(x) => x.dom_node(),
                };
                node.remove_child(&dom_node).unwrap();
            }
        }
    }
    fn remove_self(&self) {
        if let Some(node) = self.node.as_ref() {
            match node.parent_node() {
                Some(p) => {
                    p.remove_child(self).unwrap();
                },
                None => { },
            }
        }
    }
    fn set_attribute(&self, name: &'static str, value: &str) {
        if let Some(node) = self.node.as_ref() {
            node.set_attribute(name, value).unwrap();
        }
    }
    fn match_prerendered_first_child(&mut self, node: BackendNodeRefMut<Dom>) {
        let p = self.node.as_ref().unwrap();
        match p.first_child() {
            Some(next) => {
                match node {
                    BackendNodeRefMut::Element(n) => {
                        n.node = Some(next.dyn_into().expect("no matching prerendered element found"));
                        n.apply_pending_node_weak();
                    },
                    BackendNodeRefMut::TextNode(n) => {
                        match next.dyn_into::<Text>() {
                            Ok(next) => {
                                n.node = Some(next);
                            },
                            Err(next) => {
                                match next.dyn_into::<Comment>() {
                                    Ok(comment) => {
                                        let next = DOCUMENT.with(|document| {
                                            document.create_text_node("")
                                        });
                                        p.replace_child(&next, &comment).unwrap();
                                        n.node = Some(next);
                                    },
                                    Err(_) => {
                                        panic!("no matching prerendered text node found");
                                    }
                                }
                            },
                        }
                    },
                }
            },
            None => {
                panic!("no matching prerendered node found");
            }
        }
    }
    fn match_prerendered_next_sibling(&mut self, node: BackendNodeRefMut<Dom>) {
        let p = self.node.as_ref().unwrap();
        match p.next_sibling() {
            Some(next) => {
                match node {
                    BackendNodeRefMut::Element(n) => {
                        n.node = Some(next.dyn_into().expect("no matching prerendered element found"));
                        n.apply_pending_node_weak();
                    },
                    BackendNodeRefMut::TextNode(n) => {
                        match next.dyn_into::<Text>() {
                            Ok(next) => {
                                n.node = Some(next);
                            },
                            Err(next) => {
                                match next.dyn_into::<Comment>() {
                                    Ok(comment) => {
                                        let next = DOCUMENT.with(|document| {
                                            document.create_text_node("")
                                        });
                                        p.replace_child(&next, &comment).unwrap();
                                        n.node = Some(next);
                                    },
                                    Err(_) => {
                                        panic!("no matching prerendered text node found");
                                    }
                                }
                            },
                        }
                    },
                }
            },
            None => {
                panic!("no matching prerendered node found")
            }
        }
    }
}

pub struct DomTextNode {
    node: Option<Text>,
}
impl DomTextNode {
    pub fn dom_node(&self) -> &Text {
        self.node.as_ref().expect("Dom node cannot be accessed while prerendering")
    }
}
impl Deref for DomTextNode {
    type Target = Text;
    fn deref(&self) -> &Text {
        self.dom_node()
    }
}
impl BackendTextNode for DomTextNode {
    type Backend = Dom;
    fn set_text_content(&self, text_content: &str) {
        if let Some(node) = self.node.as_ref() {
            node.set_text_content(Some(text_content));
        }
    }
    fn remove_self(&self) {
        if let Some(node) = self.node.as_ref() {
            match node.parent_node() {
                Some(p) => {
                    p.remove_child(self).unwrap();
                },
                None => { },
            }
        }
    }
    fn match_prerendered_next_sibling(&mut self, node: BackendNodeRefMut<Dom>) {
        let p = self.node.as_ref().unwrap();
        match p.next_sibling() {
            Some(next) => {
                match node {
                    BackendNodeRefMut::Element(n) => {
                        n.node = Some(next.dyn_into().expect("no matching prerendered element found"));
                        n.apply_pending_node_weak();
                    },
                    BackendNodeRefMut::TextNode(n) => {
                        match next.dyn_into::<Text>() {
                            Ok(next) => {
                                n.node = Some(next);
                            },
                            Err(next) => {
                                match next.dyn_into::<Comment>() {
                                    Ok(comment) => {
                                        let next = DOCUMENT.with(|document| {
                                            document.create_text_node("")
                                        });
                                        p.replace_child(&next, &comment).unwrap();
                                        n.node = Some(next);
                                    },
                                    Err(_) => {
                                        panic!("no matching prerendered text node found");
                                    }
                                }
                            },
                        }
                    },
                }
            },
            None => {
                panic!("no matching prerendered node found")
            }
        }
    }
}

struct DomEvent {
    event: Event,
}
impl DomEvent {
    fn to_common_event(self) -> global_events::CommonEvent {
        global_events::CommonEvent { }
    }
    fn to_mouse_event(self) -> global_events::MouseEvent {
        let ev = self.event.dyn_into::<web_sys::MouseEvent>().unwrap();
        global_events::MouseEvent {
            pos: ViewportPosition {
                x: ev.client_x(),
                y: ev.client_y(),
            },
            button: match ev.button() {
                0 => MouseButton::Primary,
                1 => MouseButton::Secondary,
                2 => MouseButton::Auxiliary,
                _ => MouseButton::Other
            },
            decoration_keys: DecorationKeys {
                alt: ev.alt_key(),
                ctrl: ev.ctrl_key(),
                shift: ev.shift_key(),
                meta: ev.meta_key(),
            },
            ..Default::default()
        }
    }
    fn to_touch_event(self) -> global_events::TouchEvent {
        let ev = self.event.dyn_into::<web_sys::TouchEvent>().unwrap();
        let convert_touch_list = |list: TouchList| {
            let mut ret = vec![];
            for i in 0..list.length() {
                let touch = list.get(i).unwrap();
                ret.push(global_events::Touch {
                    id: touch.identifier(),
                    pos: ViewportPosition {
                        x: touch.client_x(),
                        y: touch.client_y(),
                    }
                });
            }
            ret
        };
        global_events::TouchEvent {
            touches: convert_touch_list(ev.touches()),
            changed_touches: convert_touch_list(ev.changed_touches()),
            decoration_keys: DecorationKeys {
                alt: ev.alt_key(),
                ctrl: ev.ctrl_key(),
                shift: ev.shift_key(),
                meta: ev.meta_key(),
            },
            ..Default::default()
        }
    }
    fn to_keyboard_event(self) -> global_events::KeyboardEvent {
        let ev = self.event.dyn_into::<web_sys::KeyboardEvent>().unwrap();
        global_events::KeyboardEvent {
            key_code: ev.key_code(),
            char_code: ev.char_code(),
            key: ev.key(),
        }
    }
}

struct DomEventListener {
    element: Element,
    name: &'static str,
    _cb: Closure<dyn 'static + FnMut(Event)>,
    el: EventListener,
}
impl Drop for DomEventListener {
    fn drop(&mut self) {
        self.element.remove_event_listener_with_event_listener_and_bool(self.name, &self.el, true).unwrap();
    }
}

pub struct Dom {
    root: RefCell<Element>,
    event_listeners: RefCell<Vec<DomEventListener>>,
    dom_prerendering: Cell<bool>,
}
impl Dom {
    pub fn new(placeholder_id: &str) -> Self {
        Self {
            root: RefCell::new(DOCUMENT.with(|document| {
                document.get_element_by_id(placeholder_id).unwrap().into()
            })),
            event_listeners: RefCell::new(vec![]),
            dom_prerendering: Cell::new(false),
        }
    }
    pub fn new_prerendering(placeholder_id: &str) -> Self {
        Self {
            root: RefCell::new(DOCUMENT.with(|document| {
                document.get_element_by_id(placeholder_id).unwrap().into()
            })),
            event_listeners: RefCell::new(vec![]),
            dom_prerendering: Cell::new(true),
        }
    }
    fn set_event_listener_on_root_node<F: 'static + Fn(&NodeWeak<Dom>, DomEvent)>(&self, name: &'static str, f: F) {
        let cb = Closure::wrap(Box::new(move |event: Event| {
            let element: &Element = &event.target().unwrap().dyn_into().unwrap();
            let dom_event = DomEvent { event };
            let node_weak: Option<NodeWeak<Dom>> = ELEMENT_MAP.with(|element_map| {
                let inner_id = js_sys::Reflect::get_u32(&element, ELEMENT_INNER_ID_MAGIC).unwrap().as_f64();
                if let Some(inner_id) = inner_id {
                    let inner_id = inner_id as usize;
                    let e = element_map.borrow();
                    e.get(&inner_id).cloned()
                } else {
                    None
                }
            });
            match node_weak {
                None => { },
                Some(node_weak) => {
                    f(&node_weak, dom_event);
                }
            }
        }) as Box<dyn FnMut(Event)>);
        let mut el = EventListener::new();
        el.handle_event(cb.as_ref().unchecked_ref());
        let root = self.root.borrow();
        root.add_event_listener_with_event_listener_and_bool(name, &el, true).unwrap();
        self.event_listeners.borrow_mut().push(DomEventListener {
            element: root.clone(),
            name,
            _cb: cb,
            el,
        });
    }
}
impl Backend for Dom {
    type BackendElement = DomElement;
    type BackendTextNode = DomTextNode;
    fn set_root_node(&self, root_node: &DomElement) {
        if self.dom_prerendering.get() {
            return;
        }
        {
            let mut root = self.root.borrow_mut();
            root.parent_node().unwrap().replace_child(root_node.node.as_ref().unwrap(), &root).unwrap();
            *root = root_node.node.as_ref().unwrap().clone();
        }
        self.event_listeners.borrow_mut().clear();
        event::init_backend_event(self);
    }
    fn create_element(&self, tag_name: &'static str) -> DomElement {
        DomElement::new(tag_name, self.dom_prerendering.get())
    }
    fn create_text_node(&self, text_content: &str) -> DomTextNode {
        DomTextNode {
            node: if self.dom_prerendering.get() {
                None
            } else {
                DOCUMENT.with(|document| {
                    Some(document.create_text_node(text_content))
                })
            }
        }
    }
    fn is_prerendering(&self) -> bool {
        self.dom_prerendering.get()
    }
    fn match_prerendered_root_element(&self, root_node: &mut DomElement) {
        let n: &Element = &self.root.borrow();
        root_node.node = Some(n.clone());
        root_node.apply_pending_node_weak();
    }
    fn end_prerendering(&self) {
        self.dom_prerendering.set(false);
        event::init_backend_event(self);
    }
}

mod event {
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::node::NodeRc;
    use super::*;

    const LONG_TAP_TIME_MS: i32 = 2000;
    const CANCEL_TAP_DISTANCE: f32 = 5.;

    struct TapStatus {
        tapping: Option<(NodeRc<Dom>, Option<TimeoutHandler>)>,
        pos: ViewportPosition<i32>,
    }

    impl TapStatus {
        fn new() -> Self {
            Self {
                tapping: None,
                pos: Default::default(),
            }
        }
    }

    pub(super) fn init_backend_event(backend: &Dom) {
        let current_tap_status: Rc<RefCell<TapStatus>> = Rc::new(RefCell::new(TapStatus::new()));

        macro_rules! reg_event {
            (trigger, $str_name: expr, $name: ident, $convert: ident) => {
                backend.set_event_listener_on_root_node($str_name, |node_weak, ev| {
                    if let Some(node_rc) = node_weak.upgrade() {
                        let ev = ev.$convert();
                        trigger_global_event!(node_rc.borrow_mut(), $name, &ev);
                    }
                });
            };
            (bubble, $str_name: expr, $name: ident, $convert: ident) => {
                backend.set_event_listener_on_root_node($str_name, |node_weak, ev| {
                    if let Some(node_rc) = node_weak.upgrade() {
                        let ev = ev.$convert();
                        bubble_global_event!(node_rc.borrow_mut(), $name, &ev);
                    }
                });
            };
            (composed, $str_name: expr, $name: ident, $convert: ident) => {
                backend.set_event_listener_on_root_node($str_name, |node_weak, ev| {
                    if let Some(node_rc) = node_weak.upgrade() {
                        let ev = ev.$convert();
                        bubble_composed_global_event!(node_rc.borrow_mut(), $name, &ev);
                    }
                });
            };
        }
        reg_event!(composed, "click", click, to_mouse_event);
        reg_event!(composed, "keydown", key_down, to_keyboard_event);
        reg_event!(composed, "keypress", key_press, to_keyboard_event);
        reg_event!(composed, "keyup", key_up, to_keyboard_event);
        reg_event!(trigger, "change", change, to_common_event);
        reg_event!(trigger, "submit", submit, to_common_event);
        reg_event!(trigger, "animationstart", animation_start, to_common_event);
        reg_event!(trigger, "animationiteration", animation_iteration, to_common_event);
        reg_event!(trigger, "animationend", animation_end, to_common_event);
        reg_event!(trigger, "transitionend", transition_end, to_common_event);

        // convert mouse to tap
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("mousedown", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_mouse_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), mouse_down, &ev);
                    }
                    if ev.button == MouseButton::Primary {
                        let long_tap_timeout = {
                            let current_tap_status = current_tap_status.clone();
                            set_timeout(move || {
                                let mut current_tap_status = current_tap_status.borrow_mut();
                                if current_tap_status.tapping.is_some() {
                                    let n = current_tap_status.tapping.take().unwrap().0;
                                    let tap_ev = TapEvent {
                                        pos: current_tap_status.pos.clone(),
                                    };
                                    bubble_composed_global_event!(n.borrow_mut(), long_tap, &tap_ev);
                                }
                            }, LONG_TAP_TIME_MS)
                        };
                        let mut current_tap_status = current_tap_status.borrow_mut();
                        current_tap_status.tapping = Some((node_rc, Some(long_tap_timeout)));
                        current_tap_status.pos = ev.pos.clone();
                    }
                }
            });
        }
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("mousemove", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_mouse_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), mouse_move, &ev);
                    }
                    let mut current_tap_status = current_tap_status.borrow_mut();
                    if current_tap_status.tapping.is_some() {
                        let dx = (current_tap_status.pos.x - ev.pos.x) as f32;
                        let dy = (current_tap_status.pos.y - ev.pos.y) as f32;
                        if dx * dx + dy * dy >= CANCEL_TAP_DISTANCE * CANCEL_TAP_DISTANCE {
                            let tap_ev = TapEvent {
                                pos: current_tap_status.pos.clone(),
                            };
                            let n = current_tap_status.tapping.take().unwrap().0;
                            bubble_composed_global_event!(n.borrow_mut(), cancel_tap, &tap_ev);
                        }
                    }
                }
            });
        }
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("mouseup", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_mouse_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), mouse_up, &ev);
                    }
                    let mut current_tap_status = current_tap_status.borrow_mut();
                    if current_tap_status.tapping.is_some() {
                        let tap_ev = TapEvent {
                            pos: current_tap_status.pos.clone(),
                        };
                        let n = current_tap_status.tapping.take().unwrap().0;
                        bubble_composed_global_event!(n.borrow_mut(), tap, &tap_ev);
                    }
                }
            });
        }

        // convert touch to tap
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("touchstart", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_touch_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), touch_start, &ev);
                    }
                    if ev.touches.len() > 1 {
                        let mut current_tap_status = current_tap_status.borrow_mut();
                        if current_tap_status.tapping.is_some() {
                            let n = current_tap_status.tapping.take().unwrap().0;
                            let tap_ev = TapEvent {
                                pos: current_tap_status.pos.clone(),
                            };
                            bubble_composed_global_event!(n.borrow_mut(), cancel_tap, &tap_ev);
                        }
                    } else if ev.touches.len() == 1 {
                        let long_tap_timeout = {
                            let current_tap_status = current_tap_status.clone();
                            set_timeout(move || {
                                let mut current_tap_status = current_tap_status.borrow_mut();
                                if current_tap_status.tapping.is_some() {
                                    let n = current_tap_status.tapping.take().unwrap().0;
                                    let tap_ev = TapEvent {
                                        pos: current_tap_status.pos.clone(),
                                    };
                                    bubble_composed_global_event!(n.borrow_mut(), long_tap, &tap_ev);
                                }
                            }, LONG_TAP_TIME_MS)
                        };
                        let mut current_tap_status = current_tap_status.borrow_mut();
                        current_tap_status.tapping = Some((node_rc, Some(long_tap_timeout)));
                        current_tap_status.pos = ev.touches[0].pos.clone();
                    }
                }
            });
        }
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("touchmove", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_touch_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), touch_move, &ev);
                    }
                    let mut current_tap_status = current_tap_status.borrow_mut();
                    if current_tap_status.tapping.is_some() && ev.touches.len() == 1 {
                        let dx = (current_tap_status.pos.x - ev.touches[0].pos.x) as f32;
                        let dy = (current_tap_status.pos.y - ev.touches[0].pos.y) as f32;
                        if dx * dx + dy * dy >= CANCEL_TAP_DISTANCE * CANCEL_TAP_DISTANCE {
                            let tap_ev = TapEvent {
                                pos: current_tap_status.pos.clone(),
                            };
                            let n = current_tap_status.tapping.take().unwrap().0;
                            bubble_composed_global_event!(n.borrow_mut(), cancel_tap, &tap_ev);
                        }
                    }
                }
            });
        }
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("touchend", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_touch_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), touch_end, &ev);
                    }
                    let mut current_tap_status = current_tap_status.borrow_mut();
                    if current_tap_status.tapping.is_some() {
                        let n = current_tap_status.tapping.take().unwrap().0;
                        let tap_ev = TapEvent {
                            pos: current_tap_status.pos.clone(),
                        };
                        if ev.touches.len() > 0 {
                            bubble_composed_global_event!(n.borrow_mut(), cancel_tap, &tap_ev);
                        } else {
                            bubble_composed_global_event!(n.borrow_mut(), tap, &tap_ev);
                        }
                    }
                }
            });
        }
        {
            let current_tap_status = current_tap_status.clone();
            backend.set_event_listener_on_root_node("touchcancel", move |node_weak, ev| {
                if let Some(node_rc) = node_weak.upgrade() {
                    let ev = ev.to_touch_event();
                    {
                        bubble_composed_global_event!(node_rc.borrow_mut(), touch_cancel, &ev);
                    }
                    let mut current_tap_status = current_tap_status.borrow_mut();
                    if current_tap_status.tapping.is_some() {
                        let n = current_tap_status.tapping.take().unwrap().0;
                        let tap_ev = TapEvent {
                            pos: current_tap_status.pos.clone(),
                        };
                        bubble_composed_global_event!(n.borrow_mut(), cancel_tap, &tap_ev);
                    }
                }
            });
        }
    }
}
