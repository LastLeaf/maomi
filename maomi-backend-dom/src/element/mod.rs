use maomi::{
    backend::SupportBackend, diff::ListItemChange, error::Error, node::SlotChildren,
    prop::PropertyUpdate, BackendContext,
};
use std::{borrow::Borrow, ops::Deref, rc::Rc};

use crate::{tree::*, DomBackend, DomGeneralElement};

pub struct DomElement(pub(crate) Rc<web_sys::Element>);

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

pub struct DomStrAttr {
    dom_elem: Rc<web_sys::Element>,
    attr_name: &'static str,
    inner: String,
}

impl Deref for DomStrAttr {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: ?Sized + PartialEq + ToOwned<Owned = String>> PropertyUpdate<S> for DomStrAttr
where
    String: Borrow<S>,
{
    fn compare_and_set_ref(dest: &mut Self, src: &S) -> bool {
        if dest.inner.borrow() == src {
            return false;
        }
        dest.inner = src.to_owned();
        dest.dom_elem
            .set_attribute(dest.attr_name, &dest.inner)
            .unwrap();
        true
    }
}

pub struct DomBoolAttr {
    dom_elem: Rc<web_sys::Element>,
    attr_name: &'static str,
    inner: bool,
}

impl Deref for DomBoolAttr {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: ?Sized + PartialEq + ToOwned<Owned = bool>> PropertyUpdate<S> for DomBoolAttr
where
    bool: Borrow<S>,
{
    fn compare_and_set_ref(dest: &mut Self, src: &S) -> bool {
        if dest.inner.borrow() == src {
            return false;
        }
        dest.inner = src.to_owned();
        if dest.inner {
            dest.dom_elem.set_attribute(dest.attr_name, "").unwrap();
        } else {
            dest.dom_elem.remove_attribute(dest.attr_name).unwrap();
        }
        true
    }
}

// TODO generate via macro

#[allow(non_camel_case_types)]
pub struct div {
    backend_element_token: ForestToken,
    pub title: DomStrAttr,
    pub hidden: DomBoolAttr,
}

impl SupportBackend<DomBackend> for div {
    type SlotData = ();

    fn init<'b>(
        _backend_context: &'b BackendContext<DomBackend>,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
    ) -> Result<(Self, ForestNodeRc<DomGeneralElement>), Error>
    where
        Self: Sized,
    {
        let elem =
            Rc::new(crate::DOCUMENT.with(|document| document.create_element("div").unwrap()));
        let backend_element =
            crate::DomGeneralElement::create_dom_element(owner, DomElement(elem.clone()));
        let this = Self {
            backend_element_token: backend_element.token(),
            title: DomStrAttr {
                dom_elem: elem.clone(),
                attr_name: "title",
                inner: String::new(),
            },
            hidden: DomBoolAttr {
                dom_elem: elem.clone(),
                attr_name: "hidden",
                inner: false,
            },
        };
        Ok((this, backend_element))
    }

    fn create<'b, R>(
        &'b mut self,
        _backend_context: &'b BackendContext<DomBackend>,
        owner: &'b mut ForestNodeMut<DomGeneralElement>,
        mut slot_fn: impl FnMut(
            &mut ForestNodeMut<DomGeneralElement>,
            &Self::SlotData,
        ) -> Result<R, Error>,
    ) -> Result<SlotChildren<R>, Error> {
        let mut node = owner.borrow_mut_token(&self.backend_element_token);
        let r = slot_fn(&mut node, &())?;
        Ok(SlotChildren::Single(r))
    }

    fn apply_updates<'b>(
        &'b mut self,
        _backend_context: &'b BackendContext<DomBackend>,
        owner: &'b mut ForestNodeMut<<DomBackend as maomi::backend::Backend>::GeneralElement>,
        _force_dirty: bool,
        mut slot_fn: impl FnMut(
            ListItemChange<&mut ForestNodeMut<DomGeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut node = owner.borrow_mut_token(&self.backend_element_token);
        slot_fn(ListItemChange::Unchanged(&mut node, &()))?;
        Ok(())
    }
}
