use maomi::{
    backend::SupportBackend, BackendContext, error::Error, diff::ListItemChange,
};

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
    type SlotData = ();

    fn init<'b>(
        _backend_context: &'b BackendContext<DomBackend>,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let elem = crate::DOCUMENT.with(|document| document.create_element("div").unwrap());
        let backend_element = crate::DomGeneralElement::create_dom_element(owner, DomElement(elem.clone()));
        let this = Self {
            backend_element_token: backend_element.token(),
            dom_elem: elem,
            hidden: false,
        };
        Ok(this)
    }

    fn create<'b>(
        &'b mut self,
        _backend_context: &'b BackendContext<DomBackend>,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
        slot_fn: impl Fn(ListItemChange<&mut ForestNodeMut<DomGeneralElement>, &Self::SlotData>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut node = owner.borrow_mut_token(&self.backend_element_token);
        slot_fn(ListItemChange::Added(&mut node, &()))?;
        Ok(())
    }

    fn apply_updates<'b>(
        &'b mut self,
        _backend_context: &'b BackendContext<DomBackend>,
        owner: &'b mut ForestNodeMut<<DomBackend as maomi::backend::Backend>::GeneralElement>,
        slot_fn: impl Fn(ListItemChange<&mut ForestNodeMut<DomGeneralElement>, &Self::SlotData>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut node = owner.borrow_mut_token(&self.backend_element_token);
        slot_fn(ListItemChange::Unchanged(&mut node, &()))?;
        Ok(())
    }

    fn backend_element_rc<'b>(
        &'b mut self,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> ForestNodeRc<DomGeneralElement> {
        owner.resolve_token(&self.backend_element_token)
    }
}
