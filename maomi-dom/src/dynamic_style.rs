//! The utilities for DOM `ClassList` handling.

use maomi::prop::{ListPropertyInit, ListPropertyItem, ListPropertyUpdate};

use crate::{base_element::DomElement, DomState};

#[doc(hidden)]
pub fn set_style(name: &'static str, value: &str, ctx: &mut DomElement) {
    match &mut ctx.elem {
        DomState::Normal(x) => {
            use wasm_bindgen::JsCast;
            if let Some(x) = x.dyn_ref::<web_sys::HtmlElement>() {
                x.style().set_property(name, value).unwrap();
            }
        }
        #[cfg(feature = "prerendering")]
        DomState::Prerendering(x) => {
            x.set_style(name, value);
        }
        #[cfg(feature = "prerendering-apply")]
        DomState::PrerenderingApply(_) => {}
    }
}

/// The manager for DOM `ClassList` .
pub struct DomStyleList {
    values: Box<[DomStyleItemValue]>,
}

#[derive(Debug, Clone, PartialEq)]
enum DomStyleItemValue {
    None,
    Str(String),
    I32(i32),
    F32(f32),
}

impl DomStyleList {
    pub(crate) fn new() -> Self {
        Self {
            values: Box::new([]),
        }
    }
}

impl ListPropertyInit for DomStyleList {
    type UpdateContext = DomElement;

    #[inline]
    fn init_list(dest: &mut Self, count: usize, _ctx: &mut Self::UpdateContext) {
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, || DomStyleItemValue::None);
        dest.values = v.into_boxed_slice();
    }
}

impl ListPropertyUpdate<i32> for DomStyleList {
    type ItemValue = ();

    #[inline]
    fn compare_and_set_item_ref<
        U: ListPropertyItem<Self, i32, Value = Self::ItemValue>,
    >(
        dest: &mut Self,
        index: usize,
        src: &i32,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        if dest.values[index] == DomStyleItemValue::I32(*src) {
            return;
        }
        dest.values[index] = DomStyleItemValue::I32(*src);
        U::item_value(dest, index, src, ctx);
    }
}

impl ListPropertyUpdate<f32> for DomStyleList {
    type ItemValue = ();

    #[inline]
    fn compare_and_set_item_ref<
        U: ListPropertyItem<Self, f32, Value = Self::ItemValue>,
    >(
        dest: &mut Self,
        index: usize,
        src: &f32,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        if dest.values[index] == DomStyleItemValue::F32(*src) {
            return;
        }
        dest.values[index] = DomStyleItemValue::F32(*src);
        U::item_value(dest, index, src, ctx);
    }
}

impl ListPropertyUpdate<str> for DomStyleList {
    type ItemValue = ();

    #[inline]
    fn compare_and_set_item_ref<
        U: ListPropertyItem<Self, str, Value = Self::ItemValue>,
    >(
        dest: &mut Self,
        index: usize,
        src: &str,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        if let DomStyleItemValue::Str(x) = &dest.values[index] {
            if x.as_str() == src {
                return;
            }
        }
        dest.values[index] = DomStyleItemValue::Str(src.to_string());
        U::item_value(dest, index, src, ctx);
    }
}
