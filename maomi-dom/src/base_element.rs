use maomi::{
    backend::tree::{ForestNodeRc, ForestToken, ForestTokenAddr},
    prop::PropertyUpdate,
};
use std::{
    borrow::Borrow,
    mem::{ManuallyDrop, MaybeUninit},
    ops::Deref,
};
use wasm_bindgen::{prelude::*, JsCast};

use crate::{
    event::{ColdEventList, HotEventList},
    DomGeneralElement,
};

#[wasm_bindgen]
extern "C" {
    type MaomiDomElement;
    #[wasm_bindgen(method, getter)]
    fn maomi(this: &MaomiDomElement) -> Option<u32>;
    #[wasm_bindgen(method, setter)]
    fn set_maomi(this: &MaomiDomElement, ptr: Option<u32>);
}

#[doc(hidden)]
pub struct DomElement {
    pub(crate) elem: web_sys::Element,
    pub(crate) forest_token: ManuallyDrop<ForestToken>,
    hot_event_list: Option<Box<HotEventList>>,
    cold_event_list: Option<Box<ColdEventList>>,
}

impl Drop for DomElement {
    fn drop(&mut self) {
        if self.hot_event_list.is_some() || self.cold_event_list.is_some() {
            self.elem.unchecked_ref::<MaomiDomElement>().set_maomi(None);
            crate::event::tap::remove_element_touch_state(&self.forest_token);
        }
        unsafe {
            ManuallyDrop::drop(&mut self.forest_token);
        }
    }
}

impl std::fmt::Debug for DomElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.elem.tag_name())
    }
}

impl DomElement {
    // Safety: must call `init` later (before dropped)
    pub(crate) unsafe fn new(elem: web_sys::Element) -> Self {
        Self {
            elem,
            forest_token: ManuallyDrop::new(MaybeUninit::uninit().assume_init()),
            hot_event_list: None,
            cold_event_list: None,
        }
    }

    pub(crate) fn init(&mut self, forest_token: ForestToken) {
        self.forest_token = ManuallyDrop::new(forest_token);
    }

    pub(crate) fn dom(&self) -> &web_sys::Node {
        &self.elem
    }

    pub(crate) fn inner_html(&self) -> String {
        self.elem.inner_html()
    }

    pub(crate) fn outer_html(&self) -> String {
        self.elem.outer_html()
    }

    pub(crate) fn from_event_dom_elem(
        dom_elem: &web_sys::Element,
    ) -> Option<ForestNodeRc<DomGeneralElement>> {
        let ptr = dom_elem.unchecked_ref::<MaomiDomElement>().maomi();
        if let Some(ptr) = ptr {
            unsafe {
                ForestTokenAddr::from_ptr(ptr as *const ())
                    .token()
                    .unsafe_resolve_token()
            }
        } else {
            None
        }
    }

    fn init_event_token(&mut self) {
        let ptr = self.forest_token.stable_addr().ptr() as usize;
        self.elem
            .unchecked_ref::<MaomiDomElement>()
            .set_maomi(Some(ptr as u32));
    }

    pub(crate) fn hot_event_list_mut(&mut self) -> &mut HotEventList {
        if self.hot_event_list.is_none() {
            self.hot_event_list = Some(Default::default());
            if self.cold_event_list.is_none() {
                self.init_event_token();
            }
        }
        self.hot_event_list.as_mut().unwrap()
    }

    pub(crate) fn hot_event_list(&self) -> Option<&HotEventList> {
        self.hot_event_list.as_ref().map(|x| &**x)
    }

    pub(crate) fn cold_event_list_mut(&mut self) -> &mut ColdEventList {
        if self.cold_event_list.is_none() {
            self.cold_event_list = Some(Default::default());
            if self.hot_event_list.is_none() {
                self.init_event_token();
            }
        }
        self.cold_event_list.as_mut().unwrap()
    }

    pub(crate) fn cold_event_list(&self) -> Option<&ColdEventList> {
        self.cold_event_list.as_ref().map(|x| &**x)
    }
}

pub struct DomStrAttr {
    pub(crate) inner: String,
    pub(crate) f: fn(&web_sys::HtmlElement, &str),
}

impl Deref for DomStrAttr {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: ?Sized + PartialEq + ToOwned<Owned = String>> PropertyUpdate<S> for DomStrAttr
where
    String: Borrow<S>,
{
    type UpdateContext = DomElement;

    #[inline]
    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut DomElement) {
        if dest.inner.borrow() == src {
            return;
        }
        dest.inner = src.to_owned();
        (dest.f)(ctx.elem.unchecked_ref(), &dest.inner);
    }
}

pub struct DomBoolAttr {
    pub(crate) inner: bool,
    pub(crate) f: fn(&web_sys::HtmlElement, bool),
}

impl Deref for DomBoolAttr {
    type Target = bool;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: ?Sized + PartialEq + ToOwned<Owned = bool>> PropertyUpdate<S> for DomBoolAttr
where
    bool: Borrow<S>,
{
    type UpdateContext = DomElement;

    #[inline]
    fn compare_and_set_ref(dest: &mut Self, src: &S, ctx: &mut DomElement) {
        if dest.inner.borrow() == src {
            return;
        }
        dest.inner = src.to_owned();
        (dest.f)(ctx.elem.unchecked_ref(), dest.inner);
    }
}
