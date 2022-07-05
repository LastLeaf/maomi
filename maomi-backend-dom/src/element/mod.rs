use maomi::backend::SupportBackend;

use crate::{tree::*, DomBackend, DomGeneralElement};

pub struct DomElement(web_sys::Element);

impl DomElement {
    pub fn inner_html(&self) -> String {
        self.0.inner_html()
    }

    pub fn outer_html(&self) -> String {
        self.0.outer_html()
    }
}

#[allow(non_camel_case_types)]
pub struct div {
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
        parent: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<(div, ForestTree<DomGeneralElement>), maomi::error::Error>
    where
        Self: Sized,
    {
        let elem = crate::DOCUMENT.with(|document| document.create_element("div").unwrap());
        let this = Self {
            dom_elem: elem.clone(),
            hidden: false,
        };
        Ok((
            this,
            crate::DomGeneralElement::create_dom_element(parent, DomElement(elem)),
        ))
    }

    fn apply_updates<'b>(
        &'b mut self,
        _backend_element: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<(), maomi::error::Error> {
        Ok(())
    }
}
