use std::ops::Deref;
use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::JsCast;

use crate::global_events::*;

thread_local! {
    static DOCUMENT: Document = window().unwrap().document().unwrap();
    static ELEMENT_MAP: RefCell<HashMap<u32, DomElement>> = RefCell::new(HashMap::new());
}

#[derive(Clone)]
pub enum DomNode {
    Element(DomElement),
    TextNode(DomTextNode),
    Comment(DomComment),
}
impl DomNode {
    pub fn dom_node(&self) -> &Node {
        match self {
            Self::Element(x) => &x.node,
            Self::TextNode(x) => &x.node,
            Self::Comment(x) => &x.node,
        }
    }
}
impl Deref for DomNode {
    type Target = Node;
    fn deref(&self) -> &Node {
        self.dom_node()
    }
}
impl super::BackendNode for DomNode {
    type BackendElement = DomElement;
    type BackendTextNode = DomTextNode;
    type BackendComment = DomComment;
    fn is_element(&self) -> bool {
        if let Self::Element(_) = self {
            true
        } else {
            false
        }
    }
    fn is_text_node(&self) -> bool {
        if let Self::TextNode(_) = self {
            true
        } else {
            false
        }
    }
    fn is_comment(&self) -> bool {
        if let Self::Comment(_) = self {
            true
        } else {
            false
        }
    }
    fn element_ref(&self) -> &Self::BackendElement {
        if let Self::Element(x) = self {
            x
        } else {
            panic!()
        }
    }
    fn text_node_ref(&self) -> &Self::BackendTextNode {
        if let Self::TextNode(x) = self {
            x
        } else {
            panic!()
        }
    }
    fn comment_ref(&self) -> &Self::BackendComment {
        if let Self::Comment(x) = self {
            x
        } else {
            panic!()
        }
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
    fn remove_self(&self) {
        match self.dom_node().parent_node() {
            Some(p) => {
                p.remove_child(self).unwrap();
            },
            None => { },
        }
    }
}

#[derive(Clone)]
pub struct DomElement {
    node: Element,

}
impl DomElement {
    fn new(tag_name: &'static str) -> Self {
        let ret = DomElement {
            node: DOCUMENT.with(|document| {
                document.create_element(tag_name).unwrap().into()
            })
        };
        ELEMENT_MAP.with(|element_map| {
            element_map.borrow_mut().insert((&ret.node).into_abi(), ret.clone());
        });
        ret
    }
    pub fn dom_node(&self) -> &Element {
        &self.node
    }
}
impl Drop for DomElement {
    fn drop(&mut self) {
        ELEMENT_MAP.with(|element_map| {
            // TODO this may remove early if cloned
            element_map.borrow_mut().remove(&(&self.node).into_abi());
        });
    }
}
impl Deref for DomElement {
    type Target = Element;
    fn deref(&self) -> &Element {
        self.dom_node()
    }
}
impl super::BackendElement for DomElement {
    type BackendNode = DomNode;
    pub fn bind_node_weak(&mut self, n: NodeWeak<Dom>) {

    }
    fn into_node(self) -> Self::BackendNode {
        DomNode::Element(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
    fn append_list(&self, children: Vec<Self::BackendNode>) {
        DOCUMENT.with(|document| {
            let frag = document.create_document_fragment();
            for child in children {
                frag.append_child(child.dom_node()).unwrap();
            }
            self.node.append_child(&frag).unwrap();
        })
    }
    fn insert_list_before(&self, children: Vec<Self::BackendNode>, before: Option<Self::BackendNode>) {
        DOCUMENT.with(|document| {
            let frag = document.create_document_fragment();
            for child in children {
                frag.append_child(child.dom_node()).unwrap();
            }
            self.node.insert_before(&frag, before.as_ref().map(|x| {x.dom_node()})).unwrap();
        })
    }
    fn remove_list(&self, children: Vec<Self::BackendNode>) {
        for child in children {
            self.node.remove_child(&child).unwrap();
        }
    }
    fn set_attribute(&self, name: &'static str, value: &str) {
        self.node.set_attribute(name, value).unwrap();
    }
}

#[derive(Clone)]
pub struct DomTextNode {
    node: Text,
}
impl DomTextNode {
    pub fn dom_node(&self) -> &Text {
        &self.node
    }
}
impl Deref for DomTextNode {
    type Target = Text;
    fn deref(&self) -> &Text {
        self.dom_node()
    }
}
impl super::BackendTextNode for DomTextNode {
    type BackendNode = DomNode;
    fn into_node(self) -> Self::BackendNode {
        DomNode::TextNode(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
    fn set_text_content(&self, text_content: &str) {
        self.node.set_text_content(Some(text_content));
    }
}

#[derive(Clone)]
pub struct DomComment {
    node: Comment,
}
impl DomComment {
    pub fn dom_node(&self) -> &Comment {
        &self.node
    }
}
impl Deref for DomComment {
    type Target = Comment;
    fn deref(&self) -> &Comment {
        self.dom_node()
    }
}
impl super::BackendComment for DomComment {
    type BackendNode = DomNode;
    fn into_node(self) -> Self::BackendNode {
        DomNode::Comment(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
}

struct DomEvent {
    event: Event,
}
impl DomEvent {
    fn to_mouse_event(self) -> MouseEvent {
        // TODO
        unimplemented!()
    }
    fn to_touch_event(self) -> TouchEvent {
        // TODO
        unimplemented!()
    }
    fn to_keyboard_event(self) -> KeyboardEvent {
        // TODO
        unimplemented!()
    }
}

pub struct Dom {
    root: RefCell<Element>,
}
impl Dom {
    pub fn new(placeholder_id: &str) -> Self {
        Self {
            root: RefCell::new(DOCUMENT.with(|document| {
                document.get_element_by_id(placeholder_id).unwrap().into()
            })),
        }
    }
    fn init_event_listeners_on_root_node(&self) {
        let init_single_event = |name, f: fn(&DomElement, DomEvent)| {
            let cb = Closure::wrap(Box::new(move |element: Element, event: Event| {
                let dom_event = DomEvent { event };
                ELEMENT_MAP.with(|element_map| {
                    match element_map.borrow_mut().get(&element.into_abi()) {
                        None => { },
                        Some(dom_element) => {
                            f(dom_element, dom_event);
                        }
                    }
                });
            }) as Box<dyn FnMut(Element, Event)>);
            let root = self.root.borrow();
            root.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
        };
        init_single_event("click", |elem, ev| { trigger_global_events!(elem, click, ev.to_mouse_event()); });
    }
}
impl super::Backend for Dom {
    type BackendNode = DomNode;
    fn set_root_node(&self, root_node: &DomElement) {
        let mut root = self.root.borrow_mut();
        root.parent_node().unwrap().replace_child(&root_node.node, &root).unwrap();
        *root = root_node.node.clone();
        self.init_event_listeners_on_root_node();
    }
    fn create_element(&self, tag_name: &'static str) -> DomElement {
        DomElement::new(tag_name)
    }
    fn create_text_node(&self, text_content: &str) -> DomTextNode {
        DomTextNode {
            node: DOCUMENT.with(|document| {
                document.create_text_node(text_content).into()
            })
        }
    }
    fn create_comment(&self) -> DomComment {
        DomComment {
            node: DOCUMENT.with(|document| {
                document.create_comment("").into()
            })
        }
    }
}


