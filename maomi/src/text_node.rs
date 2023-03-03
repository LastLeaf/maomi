//! Helper types for text nodes.

use crate::{
    backend::{tree, Backend, BackendGeneralElement, BackendTextNode},
    error::Error, locale_string::ToLocaleStr,
};

/// A text node
pub struct TextNode {
    backend_element_token: tree::ForestToken,
}

impl TextNode {
    /// Create a text node.
    #[inline(never)]
    pub fn create<B: Backend>(
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        content: impl ToLocaleStr,
    ) -> Result<(Self, tree::ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let elem = B::GeneralElement::create_text_node(owner, content.to_locale_str())?;
        let this = Self {
            backend_element_token: elem.token(),
        };
        Ok((this, elem))
    }

    /// Get the backend element.
    #[inline]
    pub fn backend_element_rc<'b, B: Backend>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeRc<B::GeneralElement>, Error> {
        owner
            .resolve_token(&self.backend_element_token)
            .ok_or(Error::TreeNodeReleased)
    }

    /// Set the text content.
    #[inline(never)]
    pub fn set_text<B: Backend>(
        &mut self,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        content: impl ToLocaleStr,
    ) -> Result<(), Error> {
        if let Some(mut text_node) = owner.borrow_mut_token(&self.backend_element_token) {
            let mut text_node = B::GeneralElement::as_text_node_mut(&mut text_node)
                .ok_or(Error::TreeNodeTypeWrong)?;
            text_node.set_text(content.to_locale_str());
        }
        Ok(())
    }
}
