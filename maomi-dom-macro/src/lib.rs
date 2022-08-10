#![recursion_limit = "128"]

use proc_macro::TokenStream;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use quote::{quote, TokenStreamExt, quote_spanned};
use syn::parse::Parse;
use once_cell::sync::Lazy;
use nanoid::nanoid;

use maomi_skin::parser::{StyleSheet, StyleSheetItem, PropertyOrSubRule};
use maomi_skin::parser::{StyleSheetConstructor, CssToken, Repeat};

const CLASS_CHARS: [char; 64] = [
    '_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const CLASS_START_CHARS: [char; 52] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

thread_local! {
    static CSS_OUT_DIR: Option<PathBuf> = {
        std::env::var("MAOMI_CSS_OUT_DIR").ok().map(|x| {
            let p = PathBuf::from(x);
            std::fs::create_dir_all(&p).unwrap();
            p
        })
    };
    static CSS_OUT_MODE: CssOutMode = {
        match std::env::var("MAOMI_CSS_OUT_MODE").ok().as_ref().map(|x| x.as_str()) {
            Some("debug") => CssOutMode::Debug,
            _ => CssOutMode::Release,
        }
    };
}

static CSS_OUT_FILE: Lazy<Option<std::sync::Mutex<File>>> = Lazy::new(|| {
    CSS_OUT_DIR.with(|dir| {
        dir.clone().map(|mut dir| {
            let file_name = std::env::var("CARGO_PKG_NAME").unwrap_or("index".into()).replace('-', "_") + ".css";
            dir.push(file_name);
            std::sync::Mutex::new(std::fs::File::create(&dir).unwrap())
        })
    })
});

#[derive(Debug, Clone, Copy, PartialEq)]
enum CssOutMode {
    Debug,
    Release,
}

struct DomCssProperty {
    inner: Repeat<CssToken>,
}

impl Parse for DomCssProperty {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: Repeat::parse_while(input, |x| !x.peek(syn::token::Semi))?,
        })
    }
}

struct DomStyleSheet {}

impl StyleSheetConstructor for DomStyleSheet {
    type PropertyValue = DomCssProperty;
    type FontFacePropertyValue = DomCssProperty;

    fn to_tokens(
        ss: &StyleSheet<Self>,
        tokens: &mut proc_macro2::TokenStream,
    ) where Self: Sized {
        let debug_out = CSS_OUT_MODE.with(|x| x.clone());

        // a helper for proc macro output
        fn write_proc_macro_class(
            tokens: &mut proc_macro2::TokenStream,
            span: proc_macro2::Span,
            struct_name: &syn::Ident,
            class_name: &str,
        ) {
            tokens.append_all(quote_spanned! {span=>
                #[allow(non_camel_case_types)]
                struct #struct_name {}
                impl maomi::prop::ListPropertyItem<maomi_dom::class_list::DomClassList, bool> for #struct_name {
                    type Value = &'static str;
                    #[inline(always)]
                    fn item_value(
                        _dest: &mut maomi_dom::class_list::DomClassList,
                        _index: usize,
                        _s: &bool,
                        _ctx: &mut <maomi_dom::class_list::DomClassList as maomi::prop::ListPropertyUpdate<bool>>::UpdateContext,
                    ) -> Self::Value {
                        #class_name
                    }
                }
            });
        }

        // generate css output
        for item in ss.items.iter() {
            match item {
                StyleSheetItem::Macro { .. } => unreachable!(),
                StyleSheetItem::Import { .. } => unreachable!(),
                StyleSheetItem::KeyFrames { at_keyword, name, brace_token, content } => {
                    if let Some(css_out_dir) = CSS_OUT_FILE.as_ref() {
                        // TODO
                    }
                }
                StyleSheetItem::FontFaceRule { at_keyword, items } => {
                    if let Some(css_out_dir) = CSS_OUT_FILE.as_ref() {
                        // TODO
                    }
                }
                StyleSheetItem::Rule { ident, items, .. } => {
                    let class_id_start = nanoid!(1, &CLASS_START_CHARS);
                    let class_id = nanoid!(10, &CLASS_CHARS);
                    let class_name = ident.name.clone() + "__" + &class_id_start + &class_id;
                    write_proc_macro_class(
                        tokens,
                        ident.span,
                        &syn::Ident::new(&ident.name.replace('-', "_"), ident.span),
                        &class_name,
                    );
                    if let Some(css_out_file) = CSS_OUT_FILE.as_ref() {
                        let mut s = format!(".{} {{\n", class_name);
                        for item in items.block.iter() {
                            match item {
                                PropertyOrSubRule::Property(prop) => {
                                    s += "\t";
                                    s += &prop.name.name;
                                    s += ": ";
                                    // s += &prop.value; // TODO
                                    s += ";\n";
                                }
                                _ => todo!("PropertyOrSubRule"),
                            }
                        }
                        s += "}\n";
                        css_out_file.lock().unwrap().write(s.as_bytes()).unwrap();
                    }
                }
            }
        }
    }
}

#[proc_macro]
pub fn dom_css(item: TokenStream) -> TokenStream {
    let ss = syn::parse_macro_input!(item as StyleSheet<DomStyleSheet>);
    quote! {
        #ss
    }.into()
}
