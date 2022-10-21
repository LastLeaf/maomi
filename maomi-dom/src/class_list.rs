//! The utilities for DOM `ClassList` handling.

use maomi::prop::{ListPropertyInit, ListPropertyItem, ListPropertyUpdate};
use std::rc::Rc;
use web_sys::DomTokenList;

use crate::{base_element::DomElement, DomState};

type DomClassListTy = dom_state_ty!(DomTokenList, (), ());

fn toggle_class_name(class_list: &mut DomClassListTy, class_name: &'static str, v: bool, _ctx: &mut DomElement) {
    match class_list {
        DomState::Normal(x) => {
            // TODO if a class is used multiple times in a single element (may through external), this breaks
            x.toggle_with_force(class_name, v).unwrap();
        }
        #[cfg(feature = "prerendering")]
        DomState::Prerendering(_) => {
            if let DomState::Prerendering(x) = &mut _ctx.elem {
                if v {
                    x.add_class(class_name);
                } else {
                    x.remove_class(class_name);
                }
            }
        }
        #[cfg(feature = "prerendering-apply")]
        class_list => match &_ctx.elem {
            DomState::Normal(x) => {
                let cl = x.class_list();
                cl.toggle_with_force(class_name, v).unwrap();
                *class_list = DomState::Normal(cl);
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => {}
            DomState::PrerenderingApply(_) => {}
        },
    }
}

/// The manager for DOM `ClassList` .
pub struct DomClassList {
    class_list: DomClassListTy,
    enabled: Box<[DomClassItem]>,
}

enum DomClassItem {
    Enabled(bool),
    External(DomExternalClasses),
}

impl DomClassList {
    pub(crate) fn new(class_list: dom_state_ty!(DomTokenList, (), ())) -> Self {
        Self {
            class_list,
            enabled: Box::new([]),
        }
    }
}

impl ListPropertyInit for DomClassList {
    type UpdateContext = DomElement;

    #[inline]
    fn init_list(dest: &mut Self, count: usize, _ctx: &mut Self::UpdateContext) {
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, || DomClassItem::Enabled(false));
        dest.enabled = v.into_boxed_slice();
    }
}

impl ListPropertyUpdate<bool> for DomClassList {
    type ItemValue = &'static str;

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
        if let DomClassItem::Enabled(x) = old_v {
            if *x == v {
                return;
            }
        }
        toggle_class_name(&mut dest.class_list, class_name, v, ctx);
        *old_v = DomClassItem::Enabled(v);
    }
}

impl ListPropertyUpdate<DomExternalClasses> for DomClassList {
    type ItemValue = ();

    #[inline]
    fn compare_and_set_item_ref<
        U: maomi::prop::ListPropertyItem<Self, DomExternalClasses, Value = Self::ItemValue>,
    >(
        dest: &mut Self,
        index: usize,
        src: &DomExternalClasses,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        U::item_value(dest, index, src, ctx);
        let old_v = dest.enabled.get_mut(index).unwrap();
        let class_list = &mut dest.class_list;
        if let DomClassItem::External(x) = old_v {
            src.diff_list(x, &mut |c, enabled| {
                toggle_class_name(class_list, c, enabled, ctx)
            });
        } else {
            let x = src.init_list(&mut |c, enabled| toggle_class_name(class_list, c, enabled, ctx));
            *old_v = DomClassItem::External(x);
        }
    }
}

impl ListPropertyItem<DomClassList, DomExternalClasses> for DomExternalClasses {
    type Value = ();
    #[inline(always)]
    fn item_value<'a>(
        _dest: &mut DomClassList,
        _index: usize,
        _s: &'a DomExternalClasses,
        _ctx: &mut <DomClassList as ListPropertyInit>::UpdateContext,
    ) -> &'a Self::Value {
        &()
    }
}

/// The external classes type used to pass class list between components.
///
/// This type has similar iterface to the `DomClassList` .
/// It can be used as a property that accepts classes,
/// and then pass the classes to other components and elements.
#[derive(Debug, Clone, PartialEq)]
pub struct DomExternalClasses {
    id: Rc<()>,
    items: Box<[DomExternalClassItem]>,
}

#[derive(Debug, Clone, PartialEq)]
enum DomExternalClassItem {
    Enabled(bool, &'static str),
    External(DomExternalClasses),
}

impl DomExternalClasses {
    /// Initialize an external class.
    pub fn new() -> Self {
        Self {
            id: Rc::new(()),
            items: Box::new([]),
        }
    }

    fn init_list(&self, update_fn: &mut impl FnMut(&'static str, bool)) -> Self {
        let items = self
            .items
            .iter()
            .map(|item| match item {
                DomExternalClassItem::Enabled(enabled, class_name) => {
                    if *enabled {
                        update_fn(class_name, true);
                    }
                    DomExternalClassItem::Enabled(*enabled, class_name)
                }
                DomExternalClassItem::External(x) => {
                    DomExternalClassItem::External(x.init_list(update_fn))
                }
            })
            .collect();
        Self {
            id: self.id.clone(),
            items,
        }
    }

    fn deinit_list(&self, update_fn: &mut impl FnMut(&'static str, bool)) {
        for item in self.items.iter() {
            match item {
                DomExternalClassItem::Enabled(enabled, class_name) => {
                    if *enabled {
                        update_fn(class_name, false);
                    }
                }
                DomExternalClassItem::External(x) => {
                    x.deinit_list(update_fn);
                }
            }
        }
    }

    fn diff_list(&self, old: &mut Self, update_fn: &mut impl FnMut(&'static str, bool)) {
        if Rc::ptr_eq(&self.id, &old.id) {
            for (new, old) in self.items.iter().zip(old.items.iter_mut()) {
                match new {
                    DomExternalClassItem::Enabled(enabled, class_name) => {
                        if *old != *new {
                            update_fn(class_name, *enabled);
                        }
                        *old = DomExternalClassItem::Enabled(*enabled, class_name);
                    }
                    DomExternalClassItem::External(newc) => {
                        if let DomExternalClassItem::External(oldc) = old {
                            newc.diff_list(oldc, update_fn);
                        } else {
                            *old = DomExternalClassItem::External(newc.init_list(update_fn));
                        }
                    }
                }
            }
        } else {
            old.deinit_list(update_fn);
            self.init_list(update_fn);
        }
    }
}

impl ListPropertyInit for DomExternalClasses {
    type UpdateContext = bool;

    #[inline]
    fn init_list(dest: &mut Self, count: usize, _ctx: &mut Self::UpdateContext) {
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, || DomExternalClassItem::Enabled(false, ""));
        dest.items = v.into_boxed_slice();
    }
}

impl ListPropertyUpdate<bool> for DomExternalClasses {
    type ItemValue = &'static str;

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
        let old_v = dest.items.get_mut(index).unwrap();
        if let DomExternalClassItem::Enabled(x, _) = old_v {
            if *x == v {
                return;
            }
        }
        *ctx = true;
        *old_v = DomExternalClassItem::Enabled(v, class_name);
    }
}

impl ListPropertyUpdate<DomExternalClasses> for DomExternalClasses {
    type ItemValue = ();

    #[inline]
    fn compare_and_set_item_ref<
        U: maomi::prop::ListPropertyItem<Self, DomExternalClasses, Value = Self::ItemValue>,
    >(
        dest: &mut Self,
        index: usize,
        src: &DomExternalClasses,
        ctx: &mut Self::UpdateContext,
    ) where
        Self: Sized,
    {
        U::item_value(dest, index, src, ctx);
        let old_v = dest.items.get_mut(index).unwrap();
        if let DomExternalClassItem::External(x) = old_v {
            src.diff_list(x, &mut |_, _| {
                *ctx = true;
            });
        } else {
            let x = src.init_list(&mut |_, _| {
                *ctx = true;
            });
            *old_v = DomExternalClassItem::External(x);
        }
    }
}

impl ListPropertyItem<DomExternalClasses, DomExternalClasses> for DomExternalClasses {
    type Value = ();
    #[inline(always)]
    fn item_value<'a>(
        _dest: &mut DomExternalClasses,
        _index: usize,
        _s: &'a DomExternalClasses,
        _ctx: &mut <DomExternalClasses as ListPropertyInit>::UpdateContext,
    ) -> &'a Self::Value {
        &()
    }
}
