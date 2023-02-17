use std::{path::PathBuf, cell::RefCell, any::{Any, TypeId}, rc::Rc};
use rustc_hash::FxHashMap;

use crate::style_sheet::*;

thread_local! {
    static MOD_ROOT: Option<PathBuf> = {
        std::env::var("MAOMI_STYLESHEET_MOD_ROOT")
            .map(|s| PathBuf::from(&s))
            .or_else(|_| {
                std::env::var("CARGO_MANIFEST_DIR")
                    .map(|s| PathBuf::from(&s).join("src").join("styles.mcss"))
            })
            .ok()
    };

    static ROOT_MODULE_MAP: RefCell<FxHashMap<TypeId, Option<Rc<dyn Any>>>> = RefCell::new(FxHashMap::default());
}

fn init_root_module<T: StyleSheetConstructor>() -> Option<Rc<StyleSheet<T>>> {
    MOD_ROOT.with(|mod_root| {
        let mod_root = mod_root.as_ref()?;
        let s = std::fs::read_to_string(&mod_root).ok()?;
        let style_sheet = syn::parse_str(&s).unwrap_or_else(|err| {
            StyleSheet::new_err(err)
        });
        Some(Rc::new(style_sheet))
    })
}

pub(crate) fn root_module<T: StyleSheetConstructor>() -> Option<Rc<StyleSheet<T>>> {
    let ret = ROOT_MODULE_MAP.with(|map| {
        let map = &mut *map.borrow_mut();
        map.entry(TypeId::of::<T>()).or_insert_with(|| {
            init_root_module::<T>().map(|x| {
                let x: Rc<dyn Any> = x;
                x
            })
        }).clone()
    })?;
    Some(ret.downcast::<StyleSheet<T>>().unwrap())
}
