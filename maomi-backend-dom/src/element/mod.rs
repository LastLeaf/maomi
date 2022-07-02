use maomi::backend::SupportBackend;

use crate::{DomBackend, DomGeneralElement, DomShadowRoot};

pub struct DomElement(web_sys::Element);

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
    fn create(
        _parent: &mut DomGeneralElement,
    ) -> Result<
        (
            Self,
            <DomBackend as maomi::backend::Backend>::GeneralElement,
        ),
        maomi::error::Error,
    >
    where
        Self: Sized,
    {
        let elem = crate::DOCUMENT.with(|document| document.create_element("div").unwrap());
        let this = Self {
            dom_elem: elem.clone(),
            hidden: false,
        };
        Ok((this, crate::DomGeneralElement::DomElement(DomElement(elem))))
    }

    fn apply_updates(
        &mut self,
        _backend_element: &mut <DomBackend as maomi::backend::Backend>::GeneralElement,
    ) -> Result<(), maomi::error::Error> {
        Ok(())
    }
}
