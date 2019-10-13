use std::ops::{Deref, Range};
use std::cell::RefCell;
use web_sys::*;

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
}

#[derive(Clone)]
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
    fn into_node(self) -> Self::BackendNode {
        DomNode::Element(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
    fn append_list(&self, children: Vec<Self::BackendNode>) {
        for child in children {
            self.node.append_child(child.dom_node()).unwrap();
        }
    }
    fn insert_list(&self, pos: usize, children: Vec<Self::BackendNode>) {
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
    type BackendNode = DomNode;
    fn set_root_node(&self, root_node: &DomElement) {
        let mut root = self.root.borrow_mut();
        root.parent_node().unwrap().replace_child(&root_node.node, &root).unwrap();
        *root = root_node.node.clone();
    }
    fn create_element(&self, tag_name: &'static str) -> DomElement {
        DomElement {
            node: self.document.create_element(tag_name).unwrap().into()
        }
    }
    fn create_text_node(&self, text_content: &str) -> DomTextNode {
        DomTextNode {
            node: self.document.create_text_node(text_content).into()
        }
    }
    fn create_comment(&self) -> DomComment {
        DomComment {
            node: self.document.create_comment("").into()
        }
    }
}
