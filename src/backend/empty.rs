use std::ops::Range;

pub enum EmptyBackendNode {
    Element(EmptyBackendElement),
    TextNode(EmptyBackendTextNode),
}

impl super::BackendNode for EmptyBackendNode {
    type BackendElement = EmptyBackendElement;
    type BackendTextNode = EmptyBackendTextNode;
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

pub struct EmptyBackendElement {

}

impl super::BackendElement for EmptyBackendElement {
    type BackendNode = EmptyBackendNode;
    fn append_list(&self, _children: Vec<&Self::BackendNode>) {
        // empty
    }
    fn insert_list(&self, _pos: usize, _children: Vec<&Self::BackendNode>) {
        // empty
    }
    fn remove_range(&self, _range: Range<usize>) {
        // empty
    }
}

pub struct EmptyBackendTextNode {

}

impl super::BackendTextNode for EmptyBackendTextNode {
    fn set_text_content(&self, _text_content: &str) {
        // empty
    }
}

pub struct Empty {

}

impl Empty {
    pub fn new() -> Self {
        Self { }
    }
}

impl super::Backend for Empty {
    type BackendElement = EmptyBackendElement;
    type BackendTextNode = EmptyBackendTextNode;
    fn set_root_node(&self, _root_node: &Self::BackendElement) {
        // empty
    }
    fn create_element(&self, _tag_name: &'static str) -> Self::BackendElement {
        EmptyBackendElement { }
    }
    fn create_text_node(&self, _text_content: &str) -> Self::BackendTextNode {
        EmptyBackendTextNode { }
    }
}
