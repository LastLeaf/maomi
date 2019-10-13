use std::ops::Range;

mod empty;
pub use empty::Empty;
mod dom;
pub use dom::Dom;

pub trait BackendNode: Sized {
    type BackendElement: BackendElement<BackendNode = Self>;
    type BackendTextNode: BackendTextNode<BackendNode = Self>;
    type BackendComment: BackendComment<BackendNode = Self>;
    fn is_element(&self) -> bool;
    fn is_text_node(&self) -> bool;
    fn is_comment(&self) -> bool;
    fn element_ref(&self) -> &Self::BackendElement;
    fn text_node_ref(&self) -> &Self::BackendTextNode;
    fn comment_ref(&self) -> &Self::BackendComment;
    fn ref_clone(&self) -> Self;
}

pub trait BackendTextNode {
    type BackendNode: BackendNode;
    fn into_node(self) -> Self::BackendNode;
    fn ref_clone(&self) -> Self;
    fn set_text_content(&self, text_content: &str);
}

pub trait BackendElement {
    type BackendNode: BackendNode;
    fn into_node(self) -> Self::BackendNode;
    fn ref_clone(&self) -> Self;
    fn append_list(&self, child: Vec<Self::BackendNode>);
    fn insert_list(&self, pos: usize, child: Vec<Self::BackendNode>);
    fn remove_range(&self, range: Range<usize>);
}

pub trait BackendComment {
    type BackendNode: BackendNode;
    fn into_node(self) -> Self::BackendNode;
    fn ref_clone(&self) -> Self;
}

pub trait Backend: 'static {
    type BackendNode: BackendNode;
    fn set_root_node(&self, root_node: &<<Self as Backend>::BackendNode as BackendNode>::BackendElement);
    fn create_element(&self, tag_name: &'static str) -> <<Self as Backend>::BackendNode as BackendNode>::BackendElement;
    fn create_text_node(&self, text_content: &str) -> <<Self as Backend>::BackendNode as BackendNode>::BackendTextNode;
    fn create_comment(&self) -> <<Self as Backend>::BackendNode as BackendNode>::BackendComment;
}
