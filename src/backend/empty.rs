use crate::node::NodeWeak;
use super::*;

#[derive(Clone)]
pub struct EmptyBackendElement { }
impl BackendElement for EmptyBackendElement {
    type Backend = Empty;
    fn bind_node_weak(&mut self, _node_weak: NodeWeak<Empty>) {
        // empty
    }
    fn append_list(&self, _children: Vec<BackendNodeRef<Empty>>) {
        // empty
    }
    fn insert_list_before<'a>(&'a self, _children: Vec<BackendNodeRef<Empty>>, _before: Option<BackendNodeRef<'a, Empty>>) {
        // empty
    }
    fn remove_list(&self, _children: Vec<BackendNodeRef<Empty>>) {
        // empty
    }
    fn remove_self(&self) {
        // empty
    }
    fn set_attribute(&self, _name: &'static str, _value: &str) {
        // empty
    }
}

#[derive(Clone)]
pub struct EmptyBackendTextNode { }
impl BackendTextNode for EmptyBackendTextNode {
    type Backend = Empty;
    fn set_text_content(&self, _text_content: &str) {
        // empty
    }
    fn remove_self(&self) {
        // empty
    }
}

pub struct Empty { }
impl Empty {
    pub fn new() -> Self {
        Self { }
    }
}

impl super::Backend for Empty {
    type BackendElement = EmptyBackendElement;
    type BackendTextNode = EmptyBackendTextNode;
    fn set_root_node(&self, _root_node: &EmptyBackendElement) {
        // empty
    }
    fn create_element(&self, _tag_name: &'static str) -> EmptyBackendElement {
        EmptyBackendElement { }
    }
    fn create_text_node(&self, _text_content: &str) -> EmptyBackendTextNode {
        EmptyBackendTextNode { }
    }
}
