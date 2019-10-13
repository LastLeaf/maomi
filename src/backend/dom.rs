use std::ops::{Deref, Range};
use std::cell::RefCell;
use web_sys::*;

pub enum DomNode {
    Element(DomElement),
    TextNode(DomTextNode),
}

impl DomNode {
    pub fn dom_node(&self) -> &Node {
        match self {
            Self::Element(x) => &x.node,
            Self::TextNode(x) => &x.node,
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
}

pub struct DomElement {
    node: Element,
}

impl DomElement {
    pub fn dom_node(&self) -> &Element {
        &self.node
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
    fn append_list(&self, children: Vec<&Self::BackendNode>) {
        for child in children {
            self.node.append_child(child.dom_node()).unwrap();
        }
    }
    fn insert_list(&self, pos: usize, children: Vec<&Self::BackendNode>) {
        let before = self.node.child_nodes().get(pos as u32);
        for child in children {
            self.node.insert_before(child.dom_node(), before.as_ref()).unwrap();
        }
    }
    fn remove_range(&self, range: Range<usize>) {
        let child_nodes = self.node.child_nodes();
        let children: Vec<Option<Node>> = range.into_iter().map(|x| child_nodes.get(x as u32)).collect();
        for child in children {
            match child {
                None => { },
                Some(node) => {
                    self.node.remove_child(&node).unwrap();
                }
            }
        }
    }
}

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
    fn set_text_content(&self, text_content: &str) {
        self.node.set_text_content(Some(text_content));
    }
}

pub struct Dom {
    root: RefCell<Element>,
    document: Document,
}

impl Dom {
    pub fn new(placeholder_id: &str) -> Self {
        let window = window().unwrap();
        let document = window.document().unwrap();
        Self {
            root: RefCell::new(document.get_element_by_id(placeholder_id).unwrap().into()),
            document,
        }
    }
}

impl super::Backend for Dom {
    type BackendElement = DomElement;
    type BackendTextNode = DomTextNode;
    fn set_root_node(&self, root_node: &Self::BackendElement) {
        let mut root = self.root.borrow_mut();
        root.parent_node().unwrap().replace_child(&root_node.node, &root).unwrap();
        *root = root_node.node.clone();
    }
    fn create_element(&self, tag_name: &'static str) -> Self::BackendElement {
        DomElement {
            node: self.document.create_element(tag_name).unwrap().into()
        }
    }
    fn create_text_node(&self, text_content: &str) -> Self::BackendTextNode {
        DomTextNode {
            node: self.document.create_text_node(text_content).into()
        }
    }
}
