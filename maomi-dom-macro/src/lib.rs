#![recursion_limit = "128"]

use nanoid::nanoid;
use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::spanned::Spanned;
use std::fs::File;
use std::io::Write;
use std::num::NonZeroU32;
use std::path::PathBuf;
use syn::Error;

use maomi_skin::parser::{CssToken, Repeat, StyleSheetConstructor, ParseStyleSheetValue, CssIdent, CssDimension, CssTokenStream, ParseWithVars};
use maomi_skin::parser::{
    PropertyOrSubRule, StyleSheet, StyleSheetItem, WriteCss, WriteCssSepCond,
};

const CLASS_CHARS: [char; 64] = [
    '_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const CLASS_START_CHARS: [char; 52] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
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
            let file_name = std::env::var("CARGO_PKG_NAME")
                .unwrap_or("index".into())
                .replace('-', "_")
                + ".css";
            dir.push(file_name);
            let mut file = std::fs::File::create(&dir).unwrap();
            if CSS_OUT_MODE.with(|x| x.clone()) == CssOutMode::Debug {
                file.write(b"/* auto-generated by maomi-dom (debug mode) */\n")
                    .unwrap();
            }
            std::sync::Mutex::new(file)
        })
    })
});

#[derive(Debug, Clone, Copy, PartialEq)]
enum CssOutMode {
    Debug,
    Release,
}

struct DomCssProperty {
    // TODO really parse the value
    inner: Repeat<CssToken>,
}

impl ParseStyleSheetValue for DomCssProperty {
    fn parse_value(_: &CssIdent, tokens: &mut CssTokenStream) -> syn::Result<Self> {
        let mut v = vec![];
        while tokens.peek().is_ok() {
            v.push(tokens.next().unwrap())
        }
        Ok(Self {
            inner: Repeat::from_vec(v),
        })
    }
}

impl WriteCss for DomCssProperty {
    fn write_css(
        &self,
        sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error> {
        self.inner.write_css(sc, debug_mode, w)
    }
}

enum DomStyleSheetConfig {
    NameMangling(bool),
}

impl ParseStyleSheetValue for DomStyleSheetConfig {
    fn parse_value(
        name: &maomi_skin::parser::CssIdent,
        tokens: &mut CssTokenStream,
    ) -> syn::Result<Self> where Self: Sized {
        let ret = match name.formal_name.as_str() {
            "name_mangling" => {
                let v = tokens.expect_ident()?;
                match v.formal_name.as_str() {
                    "off" => Self::NameMangling(false),
                    "on" => Self::NameMangling(true),
                    _ => {
                        return Err(Error::new(name.span, "Unsupported config value"));
                    }
                }
            }
            _ => {
                return Err(Error::new(name.span, "Unknown config item"));
            }
        };
        Ok(ret)
    }
}

// TODO really parse the value
type DomFontFaceProperty = DomCssProperty;

enum DomMediaCondValue {
    AspectRatio(NonZeroU32, NonZeroU32),
    MinAspectRatio(NonZeroU32, NonZeroU32),
    MaxAspectRatio(NonZeroU32, NonZeroU32),
    Orientation(DomMediaOrientation),
    PrefersColorScheme(DomMediaColorScheme),
    Resolution(CssDimension),
    MinResolution(CssDimension),
    MaxResolution(CssDimension),
    Width(CssDimension),
    MinWidth(CssDimension),
    MaxWidth(CssDimension),
    Height(CssDimension),
    MinHeight(CssDimension),
    MaxHeight(CssDimension),
}

enum DomMediaOrientation {
    Landscape,
    Portrait,
}

enum DomMediaColorScheme {
    Light,
    Dark,
}

impl ParseStyleSheetValue for DomMediaCondValue {
    fn parse_value(
        name: &CssIdent,
        tokens: &mut CssTokenStream,
    ) -> syn::Result<Self> where Self: Sized {
        fn parse_aspect_ratio(tokens: &mut CssTokenStream) -> syn::Result<(NonZeroU32, NonZeroU32)> {
            let span = tokens.span();
            let a = tokens.expect_integer()?;
            if a < 0 || a > u32::MAX as i64 {
                return Err(Error::new(span, "Expected positive integer"));
            }
            let _ = tokens.expect_delim("/")?;
            let span = tokens.span();
            let b = tokens.expect_integer()?;
            if b < 0 || b > u32::MAX as i64 {
                return Err(Error::new(span, "Expected positive integer"));
            }
            Ok((NonZeroU32::new(a as u32).unwrap(), NonZeroU32::new(b as u32).unwrap()))
        }
        let ret = match name.formal_name.as_str() {
            "aspect-ratio" => {
                let (a, b) = parse_aspect_ratio(tokens)?;
                Self::AspectRatio(a, b)
            }
            "min-aspect-ratio" => {
                let (a, b) = parse_aspect_ratio(tokens)?;
                Self::MinAspectRatio(a, b)
            }
            // TODO
            _ => {
                return Err(Error::new(name.span(), "Unknown media feature"));
            }
        };
        Ok(ret)
    }
}

struct DomStyleSheet {}

impl StyleSheetConstructor for DomStyleSheet {
    type ConfigValue = DomStyleSheetConfig;
    type PropertyValue = DomCssProperty;
    type FontFacePropertyValue = DomFontFaceProperty;
    type MediaCondValue = DomMediaCondValue;

    fn to_tokens(ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized,
    {
        let debug_mode = CSS_OUT_MODE.with(|x| *x == CssOutMode::Debug);
        let mut name_mangling = true;
        let mut inner_tokens = proc_macro2::TokenStream::new();

        // generate proc macro output for a class
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

        // generate proc macro output for a definition
        fn write_proc_macro_def(
            tokens: &mut proc_macro2::TokenStream,
            span: proc_macro2::Span,
            name: &syn::Ident,
        ) {
            tokens.append_all(quote_spanned! {span=>
                #[allow(dead_code, non_camel_case_types)]
                struct #name();
            });
        }

        // generate proc macro output for a reference
        fn write_proc_macro_ref(
            tokens: &mut proc_macro2::TokenStream,
            span: proc_macro2::Span,
            name: &syn::Ident,
        ) {
            tokens.append_all(quote_spanned! {span=>
                #[allow(dead_code)]
                #name();
            });
        }

        // generate css output
        for item in ss.items.iter() {
            match item {
                StyleSheetItem::MacroDefinition { name, .. } => {
                    write_proc_macro_def(
                        tokens,
                        name.span,
                        &syn::Ident::new(&name.formal_name, name.span),
                    );
                }
                StyleSheetItem::ConstDefinition { name, refs, .. } => {
                    write_proc_macro_def(
                        tokens,
                        name.span,
                        &syn::Ident::new(&name.formal_name, name.span),
                    );
                    for r in refs {
                        write_proc_macro_ref(
                            &mut inner_tokens,
                            r.span,
                            &syn::Ident::new(&r.formal_name, r.span),
                        );
                    }
                }
                StyleSheetItem::Config { value, .. } => {
                    match value {
                        DomStyleSheetConfig::NameMangling(enabled) => {
                            name_mangling = *enabled;
                        }
                    }
                },
                StyleSheetItem::KeyFrames {
                    at_keyword,
                    name,
                    brace_token,
                    content,
                } => {
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
                    let class_name = if !name_mangling {
                        ident.css_name()
                    } else if debug_mode {
                        ident.css_name() + "_" + &class_id_start + &class_id
                    } else {
                        class_id_start + &class_id
                    };
                    write_proc_macro_class(
                        tokens,
                        ident.span,
                        &syn::Ident::new(&ident.formal_name, ident.span),
                        &class_name,
                    );
                    items.for_each_ref(&mut |r| {
                        write_proc_macro_ref(
                            &mut inner_tokens,
                            r.span,
                            &syn::Ident::new(&r.formal_name, r.span),
                        );
                    });
                    if let Some(css_out_file) = CSS_OUT_FILE.as_ref() {
                        let s = if debug_mode {
                            let mut s = format!("\n.{} {{\n", class_name);
                            for item in items.block.iter() {
                                match item {
                                    PropertyOrSubRule::Property(prop) => {
                                        s += "\t";
                                        prop.write_css(WriteCssSepCond::Other, true, &mut s)
                                            .unwrap();
                                        s += ";\n";
                                    }
                                    _ => todo!("PropertyOrSubRule"),
                                }
                            }
                            s + "}\n"
                        } else {
                            let mut s = format!(".{}{{", class_name);
                            for (index, item) in items.block.iter().enumerate() {
                                if index > 0 {
                                    s += ";";
                                }
                                match item {
                                    PropertyOrSubRule::Property(prop) => {
                                        prop.write_css(WriteCssSepCond::Other, false, &mut s)
                                            .unwrap();
                                    }
                                    _ => todo!("PropertyOrSubRule"),
                                }
                            }
                            s + "}"
                        };
                        css_out_file.lock().unwrap().write(s.as_bytes()).unwrap();
                    }
                }
            }
        }

        // write extra tokens
        let fn_name = syn::Ident::new(&nanoid!(16, &CLASS_START_CHARS), proc_macro2::Span::call_site());
        tokens.append_all(quote! {
            fn #fn_name() {
                #inner_tokens
            }
        });
    }
}

#[proc_macro]
pub fn dom_css(item: TokenStream) -> TokenStream {
    let ss = syn::parse_macro_input!(item as StyleSheet<DomStyleSheet>);
    quote! {
        #ss
    }
    .into()
}

#[cfg(test)]
mod test {
    use super::*;

    struct Env {
        out_dir: PathBuf,
        import_dir: PathBuf,
    }

    impl Env {
        fn write_import_file(&self, name: &str, content: &str) {
            std::fs::write(&self.import_dir.join(name), content).unwrap();
        }

        fn read_output(&self) -> String {
            std::fs::read_to_string(&self.out_dir.join("maomi_dom_macro.css")).unwrap()
        }
    }

    fn setup_env(debug_mode: bool, f: impl FnOnce(Env)) {
        if debug_mode {
            std::env::set_var("MAOMI_CSS_OUT_MODE", "debug");
        }
        let tmp_path = std::env::temp_dir();
        let out_dir = tmp_path.join("maomi-dom-macro").join("test-out");
        std::fs::create_dir_all(&out_dir).unwrap();
        std::env::set_var(
            "MAOMI_CSS_OUT_DIR",
            out_dir.to_str().unwrap(),
        );
        let import_dir = tmp_path.join("maomi-dom-macro").join("test-import");
        std::fs::create_dir_all(&import_dir).unwrap();
        std::env::set_var(
            "MAOMI_CSS_IMPORT_DIR",
            import_dir.to_str().unwrap(),
        );
        f(Env {
            out_dir,
            import_dir,
        });
    }

    fn parse_str(s: &str) -> String {
        let ss: StyleSheet<DomStyleSheet> = syn::parse_str(s).unwrap();
        quote!(#ss).to_string()
    }

    #[test]
    fn import() {
        setup_env(false, |env| {
            env.write_import_file("a.css", r#"
                @const $a: 1px;                
                .imported {
                    padding: $a;
                }
            "#);
            parse_str(r#"
                @config name_mangling: off;
                @import "/a.css";
                @const $b: $a 2px;
                .self {
                    padding: $b $a;
                    margin: $b;
                }
            "#);
            assert_eq!(
                env.read_output(),
                r#".imported{padding:1px}.self{padding:1px 2px 1px;margin:1px 2px}"#,
            );
        });
    }
}
