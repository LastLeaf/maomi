use crate::{
    backend::{tree, Backend, BackendGeneralElement, BackendTextNode},
    error::Error,
};

pub struct TextNode {
    content: String,
    changed: bool,
}

impl TextNode {
    pub fn create<B: Backend>(
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
        content: &str,
    ) -> Result<(Self, tree::ForestTree<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let elem = B::GeneralElement::create_text_node(parent, content)?;
        let this = Self {
            content: String::new(),
            changed: false,
        };
        Ok((this, elem))
    }

    pub fn apply_updates<B: Backend>(
        &mut self,
        mut backend_element: tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error> {
        if self.changed {
            self.changed = false;
            let mut text_node = B::GeneralElement::as_text_node_mut(&mut backend_element)
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
