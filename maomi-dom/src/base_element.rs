use maomi::prop::PropertyUpdate;
use std::{borrow::Borrow, ops::Deref};
use wasm_bindgen::JsCast;

#[doc(hidden)]
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

pub struct DomStrAttr {
    pub(crate) inner: String,
    pub(crate) f: fn(&web_sys::HtmlElement, &str),
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
    type UpdateContext = web_sys::Element;

    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut web_sys::Element) {
        if dest.inner.borrow() == src {
            return;
        }
        dest.inner = src.to_owned();
        (dest.f)(ctx.unchecked_ref(), &dest.inner);
    }
}

pub struct DomBoolAttr {
    pub(crate) inner: bool,
    pub(crate) f: fn(&web_sys::HtmlElement, bool),
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
    type UpdateContext = web_sys::Element;

    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut web_sys::Element) {
        if dest.inner.borrow() == src {
            return;
        }
        dest.inner = src.to_owned();
        (dest.f)(ctx.unchecked_ref(), dest.inner);
    }
}
