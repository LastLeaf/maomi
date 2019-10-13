use std::ops::Range;

mod empty;
pub use empty::Empty;
mod dom;
pub use dom::Dom;

pub trait BackendNode {
    type BackendElement: BackendElement;
    type BackendTextNode: BackendTextNode;
    fn is_element(&self) -> bool;
    fn is_text_node(&self) -> bool;
    fn element_ref(&self) -> &Self::BackendElement;
    fn text_node_ref(&self) -> &Self::BackendTextNode;
}

pub trait BackendTextNode {
    fn set_text_content(&self, text_content: &str);
}

pub trait BackendElement {
    type BackendNode: BackendNode;
    fn append_list(&self, _child: Vec<&Self::BackendNode>);
    fn insert_list(&self, _pos: usize, _child: Vec<&Self::BackendNode>);
    fn remove_range(&self, _range: Range<usize>);
}

pub trait Backend: 'static {
    type BackendElement: BackendElement;
    type BackendTextNode: BackendTextNode;
    fn set_root_node(&self, root_node: &Self::BackendElement);
    fn create_element(&self, tag_name: &'static str) -> Self::BackendElement;
    fn create_text_node(&self, text_content: &str) -> Self::BackendTextNode;
}
