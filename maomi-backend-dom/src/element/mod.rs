use maomi::backend::SupportBackend;
use maomi::error::Error;

use crate::{tree::*, DomBackend, DomGeneralElement};

pub struct DomElement(pub(crate) web_sys::Element);

impl std::fmt::Debug for DomElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.0.tag_name())
    }
}

impl DomElement {
    pub fn dom(&self) -> &web_sys::Node {
        &self.0
    }

    pub fn inner_html(&self) -> String {
        self.0.inner_html()
    }

    pub fn outer_html(&self) -> String {
        self.0.outer_html()
    }
}

// TODO generate via macro

#[allow(non_camel_case_types)]
pub struct div {
    backend_element_token: ForestToken,
    dom_elem: web_sys::Element,
    hidden: bool,
}

impl div {
    pub fn set_property_hidden(&mut self, v: bool) {
        if self.hidden == v {
            return;
        }
        self.hidden = v;
        if v {
            self.dom_elem.set_attribute("hidden", "").unwrap();
        } else {
            self.dom_elem.remove_attribute("hidden").unwrap();
        }
    }
}

impl SupportBackend<DomBackend> for div {
    fn create<'b>(
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
        init: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(Self, ForestNodeRc<DomGeneralElement>), Error>
    where
        Self: Sized,
    {
        let elem = crate::DOCUMENT.with(|document| document.create_element("div").unwrap());
        let backend_element = crate::DomGeneralElement::create_dom_element(owner, DomElement(elem.clone()));
        let mut this = Self {
            backend_element_token: backend_element.token(),
            dom_elem: elem,
            hidden: false,
        };
        init(&mut this)?;
        Ok((
            this,
            backend_element,
        ))
    }

    fn apply_updates<'b>(
        &'b mut self,
        _owner: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<(), maomi::error::Error> {
        Ok(())
    }

    fn backend_element_mut<'b>(
        &'b mut self,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<ForestNodeMut<DomGeneralElement>, Error> {
        Ok(owner.borrow_mut_token(&self.backend_element_token))
    }

    fn backend_element_rc<'b>(
        &'b mut self,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<ForestNodeRc<DomGeneralElement>, Error> {
        Ok(owner.resolve_token(&self.backend_element_token))
    }
}
