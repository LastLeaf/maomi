use web_sys::DomTokenList;
use maomi::prop::ListPropertyUpdate;

use crate::base_element::DomElement;

/// The manager for DOM `ClassList`
pub struct DomClassList {
    class_list: DomTokenList,
    enabled: Vec<bool>,
}

impl DomClassList {
    pub fn new(class_list: DomTokenList) -> Self {
        Self {
            class_list,
            enabled: Vec::with_capacity(0),
        }
    }
}

impl ListPropertyUpdate<bool> for DomClassList {
    type UpdateContext = DomElement;
    type ItemValue = &'static str;

    fn init_list(dest: &mut Self, count: usize, _ctx: &mut Self::UpdateContext) {
        dest.enabled.resize(count, false);
        dest.enabled.reserve_exact(0);
    }

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
            dest.class_list.toggle_with_force(class_name, v).unwrap();
        }
    }
}
