use crate::{
    backend::{tree, Backend, BackendGeneralElement, BackendTextNode},
    error::Error,
};

pub struct TextNode {
    backend_element_token: tree::ForestToken,
    content: String,
    changed: bool,
}

impl TextNode {
    #[inline]
    pub fn create<B: Backend>(
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        content: &str,
    ) -> Result<(Self, tree::ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let content: &str = &content;
        let elem = B::GeneralElement::create_text_node(owner, content)?;
        let this = Self {
            backend_element_token: elem.token(),
            content: String::new(),
            changed: false,
        };
        Ok((this, elem))
    }

    #[inline]
    pub fn apply_updates<B: Backend>(
        &mut self,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error> {
        if self.changed {
            self.changed = false;
            let mut text_node = owner.borrow_mut_token(&self.backend_element_token);
            let mut text_node = B::GeneralElement::as_text_node_mut(&mut text_node)
                .ok_or(Error::TreeNodeTypeWrong)?;
            text_node.set_text(&self.content);
        }
        Ok(())
    }

    /// Get the backend element
    #[inline]
    pub fn backend_element_rc<'b, B: Backend>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeRc<B::GeneralElement>, Error> {
        Ok(owner.resolve_token(&self.backend_element_token))
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) {
        if self.content.as_str() != text {
            self.content = text.to_string();
            self.changed = true;
        }
    }
}
