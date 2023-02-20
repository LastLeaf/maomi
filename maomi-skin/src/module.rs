use std::{path::Path, cell::RefCell, any::{Any, TypeId}, rc::Rc};
use rustc_hash::FxHashMap;

use crate::{style_sheet::*, css_token::VarName, ModPath};

thread_local! {
    static ROOT_MODULE_MAP: RefCell<FxHashMap<TypeId, Option<Rc<dyn Any>>>> = RefCell::new(FxHashMap::default());
}

fn parse_mod_file<T: StyleSheetConstructor>(mod_path: ModPath, p: &Path) -> Option<Rc<StyleSheet<T>>> {
    use syn::parse::Parser;
    let s = std::fs::read_to_string(p).ok()?;
    let style_sheet = StyleSheet::<T>::parse_mod_fn(mod_path)
        .parse_str(&s)
        .unwrap_or_else(|err| {
            StyleSheet::new_err(err)
        });
    Some(Rc::new(style_sheet))
}

pub(crate) fn parse_mod_path<T: StyleSheetConstructor>(cur_mod_path: &crate::ModPath, mod_name: &VarName) -> Option<Rc<StyleSheet<T>>> {
    crate::config::crate_config(|crate_config| {
        let mod_root: &Path = crate_config.stylesheet_mod_root.as_ref()?;
        let mut cur_dir = mod_root.parent()?.to_path_buf();
        for seg in cur_mod_path.segs.iter() {
            cur_dir.push(seg.to_string());
        }
        let mut p1 = cur_dir.clone();
        p1.push(&format!("{}.mcss", mod_name.ident.to_string()));
        let mut p2 = cur_dir;
        p2.push(mod_name.ident.to_string());
        p2.push("mod.mcss");
        let mut full_mod_path = cur_mod_path.clone();
        full_mod_path.segs.push(mod_name.ident.clone());
        parse_mod_file(full_mod_path.clone(), &p2).or_else(|| parse_mod_file(full_mod_path, &p1))
    })
}

fn init_root_module<T: StyleSheetConstructor>() -> Option<Rc<StyleSheet<T>>> {
    crate::config::crate_config(|crate_config| {
        let mod_root: &Path = crate_config.stylesheet_mod_root.as_ref()?;
        parse_mod_file(Default::default(), mod_root)
    })
}

pub fn root_module<T: StyleSheetConstructor>() -> Option<Rc<StyleSheet<T>>> {
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
