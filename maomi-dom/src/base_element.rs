use maomi::{
    backend::tree::{ForestNode, ForestNodeRc, ForestToken, ForestTokenAddr},
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
    DomGeneralElement, DomState, WriteHtmlState,
};

#[wasm_bindgen]
extern "C" {
    type MaomiDomElement;
    #[wasm_bindgen(method, getter)]
    fn maomi(this: &MaomiDomElement) -> Option<u32>;
    #[wasm_bindgen(method, setter)]
    fn set_maomi(this: &MaomiDomElement, ptr: Option<u32>);
}

#[cfg(feature = "prerendering")]
#[derive(Debug, Clone)]
pub(crate) struct PrerenderingElement {
    tag_name: &'static str,
    classes: Vec<&'static str>,
    attrs: Vec<(&'static str, String)>,
}

#[cfg(feature = "prerendering")]
impl PrerenderingElement {
    pub(crate) fn new(tag_name: &'static str) -> Self {
        Self {
            tag_name,
            classes: vec![],
            attrs: vec![],
        }
    }

    pub(crate) fn set_attribute(&mut self, name: &'static str, value: String) {
        if let Some((_, v)) = self.attrs.iter_mut().find(|(n, _)| *n == name) {
            *v = value;
        } else {
            self.attrs.push((name, value));
        }
    }

    pub(crate) fn remove_attribute(&mut self, name: &'static str) {
        if let Some(index) = self.attrs.iter_mut().position(|(n, _)| *n == name) {
            self.attrs.swap_remove(index);
        }
    }

    pub(crate) fn set_class_count(&mut self, count: usize) {
        self.classes.resize(count, "");
    }

    pub(crate) fn set_class(&mut self, index: usize, class_name: &'static str) {
        self.classes[index] = class_name;
    }

    #[cfg(feature = "prerendering")]
    pub(crate) fn write_children_html(
        &self,
        w: &mut impl std::io::Write,
        this: &ForestNode<DomGeneralElement>,
        state: &mut WriteHtmlState,
    ) -> std::io::Result<()> {
        let mut cur = this.first_child();
        while let Some(c) = &cur {
            DomGeneralElement::write_outer_html(&c, w, state)?;
            cur = c.next_sibling();
        }
        Ok(())
    }

    #[cfg(feature = "prerendering")]
    pub(crate) fn write_html(
        &self,
        w: &mut impl std::io::Write,
        this: &ForestNode<DomGeneralElement>,
        state: &mut WriteHtmlState,
    ) -> std::io::Result<()> {
        write!(w, "<{}", self.tag_name)?;
        let mut has_class = false;
        for c in &self.classes {
            if c.len() == 0 {
                continue;
            };
            if !has_class {
                has_class = true;
                write!(w, r#" class=""#)?;
            } else {
                write!(w, " ")?;
            }
            write!(w, "{}", c)?;
        }
        if has_class {
            write!(w, r#"""#)?;
        }
        for (name, value) in &self.attrs {
            write!(w, r#" {}=""#, name)?;
            html_escape::encode_double_quoted_attribute_to_writer(&value, w)?;
            write!(w, r#"""#)?;
        }
        write!(w, ">")?;
        state.prev_is_text_node = false;
        self.write_children_html(w, this, state)?;
        write!(w, "</{}>", self.tag_name)?;
        state.prev_is_text_node = false;
        Ok(())
    }
}

#[cfg(feature = "prerendering-apply")]
#[derive(Clone)]
pub(crate) struct RematchedDomElem {
    inner: std::rc::Rc<std::cell::Cell<Option<web_sys::Element>>>,
}

#[cfg(feature = "prerendering-apply")]
impl RematchedDomElem {
    pub(crate) fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub(crate) fn set(&mut self, e: web_sys::Element) {
        self.inner.set(Some(e));
    }

    pub(crate) fn take(&self) -> Option<web_sys::Element> {
        self.inner.take()
    }
}

#[doc(hidden)]
pub struct DomElement {
    pub(crate) elem: dom_state_ty!(web_sys::Element, PrerenderingElement, RematchedDomElem),
    pub(crate) forest_token: ManuallyDrop<ForestToken>,
    hot_event_list: Option<Box<HotEventList>>,
    cold_event_list: Option<Box<ColdEventList>>,
}

impl Drop for DomElement {
    fn drop(&mut self) {
        if self.hot_event_list.is_some() || self.cold_event_list.is_some() {
            match &self.elem {
                DomState::Normal(x) => {
                    x.unchecked_ref::<MaomiDomElement>().set_maomi(None);
                }
                #[cfg(feature = "prerendering")]
                DomState::Prerendering(_) => {}
                #[cfg(feature = "prerendering-apply")]
                DomState::PrerenderingApply(_) => {}
            }
            crate::event::tap::remove_element_touch_state(&self.forest_token);
        }
        unsafe {
            ManuallyDrop::drop(&mut self.forest_token);
        }
    }
}

impl std::fmt::Debug for DomElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.elem {
            DomState::Normal(x) => write!(f, "<{}>", x.tag_name()),
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => write!(f, "<(prerendering)>"),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => write!(f, "<(prerendering)>"),
        }
    }
}

impl DomElement {
    // Safety: must call `init` later (before dropped)
    pub(crate) unsafe fn new(
        elem: dom_state_ty!(web_sys::Element, PrerenderingElement, RematchedDomElem),
    ) -> Self {
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

    pub(crate) fn is_prerendering(&self) -> dom_state_ty!((), (), ()) {
        match &self.elem {
            DomState::Normal(_) => DomState::Normal(()),
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => DomState::Prerendering(()),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => DomState::PrerenderingApply(()),
        }
    }

    pub(crate) fn composing_dom(&self) -> &web_sys::Node {
        match &self.elem {
            DomState::Normal(x) => &x,
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => unreachable!(),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => unreachable!(),
        }
    }

    #[cfg(feature = "prerendering-apply")]
    pub(crate) fn rematch_dom(&mut self, e: web_sys::Node) {
        if let DomState::PrerenderingApply(x) = &mut self.elem {
            x.set(e.clone().unchecked_into());
        }
        for item in self.cold_event_list_mut() {
            item.apply(e.unchecked_ref());
        }
        self.elem = DomState::Normal(e.unchecked_into());
        if self.hot_event_list.is_some() || self.cold_event_list.is_some() {
            self.init_event_token();
        }
    }

    pub(crate) fn write_inner_html(
        &self,
        _this: &ForestNode<DomGeneralElement>,
        w: &mut impl std::io::Write,
        _state: &mut WriteHtmlState,
    ) -> std::io::Result<()> {
        match &self.elem {
            DomState::Normal(x) => {
                write!(w, "{}", x.inner_html())?;
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(x) => {
                x.write_children_html(w, _this, _state).unwrap();
            }
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => {}
        }
        Ok(())
    }

    pub(crate) fn write_outer_html(
        &self,
        _this: &ForestNode<DomGeneralElement>,
        w: &mut impl std::io::Write,
        _state: &mut WriteHtmlState,
    ) -> std::io::Result<()> {
        match &self.elem {
            DomState::Normal(x) => {
                write!(w, "{}", x.outer_html())?;
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(x) => {
                x.write_html(w, _this, _state).unwrap();
            }
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => {}
        }
        Ok(())
    }

    pub(crate) fn from_event_dom_elem(
        dom_elem: &web_sys::Element,
        bubbles: bool,
    ) -> Option<ForestNodeRc<DomGeneralElement>> {
        let ptr = dom_elem.unchecked_ref::<MaomiDomElement>().maomi();
        if let Some(ptr) = ptr {
            return unsafe {
                ForestTokenAddr::from_ptr(ptr as *const ())
                    .token()
                    .unsafe_resolve_token()
            };
        }
        if !bubbles {
            return None;
        }
        let mut next = dom_elem.parent_element();
        while let Some(cur) = next.as_ref() {
            let ptr = cur.unchecked_ref::<MaomiDomElement>().maomi();
            if let Some(ptr) = ptr {
                return unsafe {
                    ForestTokenAddr::from_ptr(ptr as *const ())
                        .token()
                        .unsafe_resolve_token()
                };
            }
            next = cur.parent_element();
        }
        None
    }

    fn init_event_token(&mut self) {
        let ptr = self.forest_token.stable_addr().ptr() as usize;
        match &self.elem {
            DomState::Normal(x) => {
                x.unchecked_ref::<MaomiDomElement>()
                    .set_maomi(Some(ptr as u32));
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => {}
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => {}
        }
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
    #[cfg(feature = "prerendering")]
    pub(crate) attr_name: &'static str,
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
        match &mut ctx.elem {
            DomState::Normal(x) => {
                (dest.f)(x.unchecked_ref(), &dest.inner);
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(x) => {
                x.set_attribute(dest.attr_name, dest.inner.clone());
            }
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => {}
        }
    }
}

pub struct DomBoolAttr {
    pub(crate) inner: bool,
    pub(crate) f: fn(&web_sys::HtmlElement, bool),
    #[cfg(feature = "prerendering")]
    pub(crate) attr_name: &'static str,
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
        match &mut ctx.elem {
            DomState::Normal(x) => {
                (dest.f)(x.unchecked_ref(), dest.inner);
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(x) => {
                if dest.inner {
                    x.set_attribute(dest.attr_name, String::with_capacity(0));
                } else {
                    x.remove_attribute(dest.attr_name);
                }
            }
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => {}
        }
    }
}
