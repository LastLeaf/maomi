use maomi::prop::ListPropertyUpdate;
use web_sys::DomTokenList;

use crate::{base_element::DomElement, DomState};

/// The manager for DOM `ClassList`
pub struct DomClassList {
    class_list: dom_state_ty!(DomTokenList, ()),
    enabled: Vec<bool>,
}

impl DomClassList {
    pub(crate) fn new(class_list: dom_state_ty!(DomTokenList, ())) -> Self {
        Self {
            class_list,
            enabled: Vec::with_capacity(0),
        }
    }

    #[cfg(feature = "prerendering-apply")]
    pub(crate) fn apply_prerendered_class_list(&mut self, class_list: dom_state_ty!(DomTokenList, ())) {
        // TODO
    }
}

impl ListPropertyUpdate<bool> for DomClassList {
    type UpdateContext = DomElement;
    type ItemValue = &'static str;

    #[inline]
    fn init_list(dest: &mut Self, count: usize, ctx: &mut Self::UpdateContext) {
        dest.enabled.resize(count, false);
        dest.enabled.reserve_exact(0);
        #[cfg(feature = "prerendering")]
        if let DomState::Prerendering(x) = &mut ctx.elem {
            x.set_class_count(count);
        }
    }

    #[inline]
    fn compare_and_set_item_ref<
        U: maomi::prop::ListPropertyItem<Self, bool, Value = Self::ItemValue>,
    >(
        dest: &mut Self,
        index: usize,
        src: &bool,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        let class_name = U::item_value(dest, index, src, ctx);
        let v = *src;
        let old_v = dest.enabled.get_mut(index).unwrap();
        if *old_v != v {
            *old_v = v;
            match &dest.class_list {
                DomState::Normal(x) => {
                    x.toggle_with_force(class_name, v).unwrap();
                }
                #[cfg(feature = "prerendering")]
                DomState::Prerendering(_) => {
                    if let DomState::Prerendering(x) = &mut ctx.elem {
                        if v {
                            x.set_class(index, class_name);
                        } else {
                            x.set_class(index, "");
                        }
                    }
                }
                #[cfg(feature = "prerendering-apply")]
                DomState::PrerenderingApply => {}
            }
        }
    }
}
