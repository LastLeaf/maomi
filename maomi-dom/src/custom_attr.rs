//! Some utilities to define custom DOM attributes.
//!
//! Sometimes it is needed to add some custom attribute to DOM elements.
//! In these cases, `maomi_dom::dom_define_attribute!` can be used to define a custom attribute.
//! Then the attribute can be used with `attr:xxx` template syntax.
//!
//! ```no_run
//! // define a new attribute `role`
//! maomi_dom::dom_define_attribute!(aria_hidden);
//! // use in template like this
//! // <div attr:aria_hidden="true" />
//!

use maomi::prop::{ListPropertyInit, ListPropertyItem, ListPropertyUpdate};

use crate::{base_element::DomElement, DomState};

/// The custom DOM attributes.
///
/// Can be used in template with `attr:name="value"` syntax, e.g. `attr:aria-hidden="true"`.
/// Caution! This bypass type checks and directly write to DOM.
/// Do not use this if there are other proper attributes.
pub struct DomCustomAttrs {
    pub(crate) inner: Vec<Option<String>>,
}

impl DomCustomAttrs {
    pub(crate) fn new() -> Self {
        Self {
            inner: Vec::with_capacity(0),
        }
    }
}

impl ListPropertyInit for DomCustomAttrs {
    type UpdateContext = DomElement;

    #[inline]
    fn init_list(dest: &mut Self, count: usize, _ctx: &mut Self::UpdateContext) {
        dest.inner = Vec::with_capacity(count);
        dest.inner.resize(count, None);
    }
}

impl ListPropertyUpdate<str> for DomCustomAttrs {
    type ItemValue = &'static str;

    #[inline]
    fn compare_and_set_item_ref<U: ListPropertyItem<Self, str, Value = Self::ItemValue>>(
        dest: &mut Self,
        index: usize,
        src: &str,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        let attr_name = U::item_value(dest, index, src, ctx);
        if dest.inner[index].as_ref().map(|x| x.as_str()) != Some(src) {
            dest.inner[index] = Some(src.to_string());
            match &mut ctx.elem {
                DomState::Normal(x) => {
                    let _ = x.set_attribute(attr_name, src);
                }
                #[cfg(feature = "prerendering")]
                DomState::Prerendering(x) => {
                    x.set_attribute(attr_name, src.to_string());
                }
                #[cfg(feature = "prerendering-apply")]
                DomState::PrerenderingApply(_) => {}
            }
        }
    }
}

impl ListPropertyUpdate<bool> for DomCustomAttrs {
    type ItemValue = &'static str;

    #[inline]
    fn compare_and_set_item_ref<U: ListPropertyItem<Self, bool, Value = Self::ItemValue>>(
        dest: &mut Self,
        index: usize,
        src: &bool,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        let attr_name = U::item_value(dest, index, src, ctx);
        let v = if *src { Some("") } else { None };
        if dest.inner[index].as_ref().map(|x| x.as_str()) != v {
            dest.inner[index] = v.map(|x| x.to_string());
            match &mut ctx.elem {
                DomState::Normal(x) => {
                    if v.is_some() {
                        let _ = x.set_attribute(attr_name, "");
                    } else {
                        let _ = x.remove_attribute(attr_name);
                    }
                }
                #[cfg(feature = "prerendering")]
                DomState::Prerendering(x) => {
                    if v.is_some() {
                        x.set_attribute(attr_name, "".to_string());
                    } else {
                        x.remove_attribute(attr_name);
                    }
                }
                #[cfg(feature = "prerendering-apply")]
                DomState::PrerenderingApply(_) => {}
            }
        }
    }
}
