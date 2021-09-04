pub mod empty;
pub use empty::Empty;
pub mod dom;
pub use dom::Dom;

use crate::node::NodeWeak;

pub enum BackendNodeRef<'a, B: Backend> {
    Element(&'a B::BackendElement),
    TextNode(&'a B::BackendTextNode),
}
impl<'a, B: Backend> BackendNodeRef<'a, B> {
    pub(crate) fn remove_self(&self) {
        match self {
            Self::Element(x) => x.remove_self(),
            Self::TextNode(x) => x.remove_self(),
        }
    }
}

pub enum BackendNodeRefMut<'a, B: Backend> {
    Element(&'a mut B::BackendElement),
    TextNode(&'a mut B::BackendTextNode),
}

pub enum BackendNode<B: Backend> {
    Element(B::BackendElement),
    TextNode(B::BackendTextNode),
}

pub trait BackendTextNode {
    type Backend: Backend;
    fn set_text_content(&self, text_content: &str);
    fn remove_self(&self);
    fn match_prerendered_next_sibling(&mut self, _node: BackendNodeRefMut<Self::Backend>) {
        unreachable!()
    }
}

pub trait BackendElement {
    type Backend: Backend;
    fn bind_node_weak(&mut self, node_weak: NodeWeak<Self::Backend>);
    fn append_list(&self, children: Vec<BackendNodeRef<Self::Backend>>);
    fn insert_list_before<'a>(
        &'a self,
        children: Vec<BackendNodeRef<Self::Backend>>,
        before: Option<BackendNodeRef<'a, Self::Backend>>,
    );
    fn remove_list(&self, children: Vec<BackendNodeRef<Self::Backend>>);
    fn remove_self(&self);
    fn set_attribute(&self, name: &'static str, value: &str);
    fn get_attribute(&self, name: &'static str) -> Option<String>;
    fn get_field(&self, _name: &'static str) -> Option<wasm_bindgen::JsValue> {
        None
    }
    fn match_prerendered_first_child(&mut self, _node: BackendNodeRefMut<Self::Backend>) {
        unreachable!()
    }
    fn match_prerendered_next_sibling(&mut self, _node: BackendNodeRefMut<Self::Backend>) {
        unreachable!()
    }
}

pub trait Backend: 'static + Sized {
    type BackendElement: BackendElement<Backend = Self>;
    type BackendTextNode: BackendTextNode<Backend = Self>;
    fn set_root_node(&self, root_node: &Self::BackendElement);
    fn create_element(&self, tag_name: &'static str) -> Self::BackendElement;
    fn create_text_node(&self, text_content: &str) -> Self::BackendTextNode;
    fn is_prerendering(&self) -> bool {
        false
    }
    fn match_prerendered_root_element(&self, _root_node: &mut Self::BackendElement) {
        unreachable!()
    }
    fn end_prerendering(&self) {
        unreachable!()
    }
}
