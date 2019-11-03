#[derive(Clone)]
pub enum EmptyBackendNode {
    Element(EmptyBackendElement),
    TextNode(EmptyBackendTextNode),
    Comment(EmptyBackendComment),
}
impl super::BackendNode for EmptyBackendNode {
    type BackendElement = EmptyBackendElement;
    type BackendTextNode = EmptyBackendTextNode;
    type BackendComment = EmptyBackendComment;
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
        // empty
    }
}

#[derive(Clone)]
pub struct EmptyBackendElement { }
impl super::BackendElement for EmptyBackendElement {
    type BackendNode = EmptyBackendNode;
    fn into_node(self) -> Self::BackendNode {
        EmptyBackendNode::Element(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
    fn append_list(&self, _children: Vec<Self::BackendNode>) {
        // empty
    }
    fn insert_list_before(&self, _children: Vec<Self::BackendNode>, _before: Option<Self::BackendNode>) {
        // empty
    }
    fn remove_list(&self, _children: Vec<Self::BackendNode>) {
        // empty
    }
    fn set_attribute(&self, _name: &'static str, _value: &str) {
        // empty
    }
}

#[derive(Clone)]
pub struct EmptyBackendTextNode { }
impl super::BackendTextNode for EmptyBackendTextNode {
    type BackendNode = EmptyBackendNode;
    fn into_node(self) -> Self::BackendNode {
        EmptyBackendNode::TextNode(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
    fn set_text_content(&self, _text_content: &str) {
        // empty
    }
}

#[derive(Clone)]
pub struct EmptyBackendComment { }
impl super::BackendComment for EmptyBackendComment {
    type BackendNode = EmptyBackendNode;
    fn into_node(self) -> Self::BackendNode {
        EmptyBackendNode::Comment(self)
    }
    fn ref_clone(&self) -> Self {
        self.clone()
    }
}

pub struct Empty { }
impl Empty {
    pub fn new() -> Self {
        Self { }
    }
}

impl super::Backend for Empty {
    type BackendNode = EmptyBackendNode;
    fn set_root_node(&self, _root_node: &EmptyBackendElement) {
        // empty
    }
    fn create_element(&self, _tag_name: &'static str) -> EmptyBackendElement {
        EmptyBackendElement { }
    }
    fn create_text_node(&self, _text_content: &str) -> EmptyBackendTextNode {
        EmptyBackendTextNode { }
    }
    fn create_comment(&self) -> EmptyBackendComment {
        EmptyBackendComment { }
    }
}
