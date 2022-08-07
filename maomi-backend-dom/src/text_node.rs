use maomi::backend::*;

#[doc(hidden)]
pub struct DomTextNode {
    dom_elem: web_sys::Text,
}

impl DomTextNode {
    pub(crate) fn dom(&self) -> &web_sys::Node {
        &self.dom_elem
    }

    pub(crate) fn new(content: &str) -> Self {
        Self {
            dom_elem: crate::DOCUMENT.with(|document| document.create_text_node(content)),
        }
    }

    pub fn inner_html(&self) -> String {
        self.dom_elem.text_content().unwrap_or_default()
    }
}

impl BackendTextNode for DomTextNode {
    type BaseBackend = crate::DomBackend;

    fn set_text(&mut self, content: &str) {
        self.dom_elem.set_text_content(Some(content));
    }
}
