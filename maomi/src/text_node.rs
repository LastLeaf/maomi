use crate::{
    backend::{Backend, BackendGeneralElement, BackendTextNode},
    error::Error,
};

pub struct TextNode {
    content: String,
    changed: bool,
}

impl TextNode {
    pub fn create<B: Backend>(
        parent: &mut <B as Backend>::GeneralElement,
        content: &str,
    ) -> Result<(Self, <B as Backend>::GeneralElement), Error>
    where
        Self: Sized,
    {
        let elem = parent.create_text_node(content);
        let this = Self {
            content: String::new(),
            changed: false,
        };
        Ok((this, elem.into_general_element()))
    }

    pub fn apply_updates<B: Backend>(
        &mut self,
        backend_element: &mut <B as Backend>::GeneralElement,
    ) -> Result<(), Error> {
        if self.changed {
            self.changed = false;
            let text_node = backend_element
                .as_text_node_mut()
                .ok_or(Error::TreeNotMatchedError)?;
            text_node.set_text(&self.content);
        }
        Ok(())
    }

    pub fn set_text(&mut self, text: &str) {
        if self.content.as_str() != text {
            self.content = text.to_string();
            self.changed = true;
        }
    }
}
