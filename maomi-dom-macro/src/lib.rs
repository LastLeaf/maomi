#![recursion_limit = "128"]

use nanoid::nanoid;
use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned, TokenStreamExt};
use std::cell::Cell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use syn::Error;

use maomi_skin::parser::*;
use maomi_skin::parser::write_css::{CssWriter, WriteCss};

mod media_cond;
use media_cond::*;
mod property;
use property::*;

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
    static CSS_OUT_MODE: Cell<CssOutMode>  = {
        match std::env::var("MAOMI_CSS_OUT_MODE").ok().as_ref().map(|x| x.as_str()) {
            Some("debug") => Cell::new(CssOutMode::Debug),
            _ => Cell::new(CssOutMode::Release),
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
            if CSS_OUT_MODE.with(|x| x.get()) == CssOutMode::Debug {
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
        let debug_mode = CSS_OUT_MODE.with(|x| x.get() == CssOutMode::Debug);
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
                // generate macro def
                StyleSheetItem::MacroDefinition { name, .. } => {
                    write_proc_macro_def(
                        tokens,
                        name.span,
                        &syn::Ident::new(&name.formal_name, name.span),
                    );
                }

                // generate const def and ref
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

                // handling config
                StyleSheetItem::Config { value, .. } => {
                    match value {
                        DomStyleSheetConfig::NameMangling(enabled) => {
                            name_mangling = *enabled;
                        }
                    }
                }

                // generate @key-frames block
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

                // generate @font-face block
                StyleSheetItem::FontFaceRule { at_keyword, items } => {
                    if let Some(css_out_dir) = CSS_OUT_FILE.as_ref() {
                        // TODO
                    }
                }

                // generate common rule
                StyleSheetItem::Rule { ident, content, .. } => {
                    // a helper for CSS class name generation
                    fn generate_css_name(full_ident: &CssIdent, name_mangling: bool, debug_mode: bool) -> String {
                        let class_id_start = nanoid!(1, &CLASS_START_CHARS);
                        let class_id = nanoid!(10, &CLASS_CHARS);
                        if !name_mangling {
                            full_ident.css_name()
                        } else if debug_mode {
                            full_ident.css_name() + "_" + &class_id_start + &class_id
                        } else {
                            class_id_start + &class_id
                        }
                    }

                    // rec to generate all CSS rules
                    fn handle_rule_content(
                        tokens: &mut proc_macro2::TokenStream,
                        inner_tokens: &mut proc_macro2::TokenStream,
                        name_mangling: bool,
                        debug_mode: bool,
                        full_ident: &CssIdent,
                        content: &RuleContent<DomStyleSheet>,
                        cssw: &mut CssWriter<String>,
                    ) -> Result<(), std::fmt::Error> {
                        let class_name = generate_css_name(full_ident, name_mangling, debug_mode);

                        // generate proc macro output (for IDE hinting)
                        write_proc_macro_class(
                            tokens,
                            full_ident.span,
                            &syn::Ident::new(&full_ident.formal_name, full_ident.span),
                            &class_name,
                        );
                        content.for_each_ref(&mut |r| {
                            write_proc_macro_ref(
                                inner_tokens,
                                r.span,
                                &syn::Ident::new(&r.formal_name, r.span),
                            );
                        });

                        // a helper for write css name
                        let write_selector = |cssw: &mut CssWriter<String>| {
                            cssw.write_delim(".", true)?;
                            cssw.write_ident(&class_name, false)?;
                            Ok(())
                        };

                        // a helper for write prop list
                        let write_prop_list = |cssw: &mut CssWriter<String>, props: &[Property<DomCssProperty>]| {
                            for (index, prop) in props.iter().enumerate() {
                                prop.name.write_css(cssw)?;
                                prop.colon_token.write_css(cssw)?;
                                prop.value.write_css(cssw)?;
                                if debug_mode || index + 1 < content.props.len() {
                                    prop.semi_token.write_css(cssw)?;
                                    if debug_mode {
                                        cssw.line_wrap()?;
                                    }
                                }
                            }
                            Ok(())
                        };

                        // a helper for write at-blocks
                        let write_main_rule_and_at_blocks = |
                            cssw: &mut CssWriter<String>,
                            pseudo_class: Option<&CssIdent>,
                            props: &[Property<DomCssProperty>],
                            at_blocks: &[AtBlock<DomStyleSheet>],
                        | {
                            if props.len() > 0 {
                                write_selector(cssw)?;
                                if let Some(ident) = pseudo_class {
                                    cssw.write_delim(":", false)?;
                                    cssw.write_ident(&ident.css_name(), false)?;
                                }
                                cssw.write_brace_block(|cssw| {
                                    write_prop_list(cssw, &props)
                                })?;
                            }
                            for block in at_blocks {
                                let items = match block {
                                    AtBlock::Media { at_keyword, expr, items } => {
                                        if items.block.as_slice().len() > 0 {
                                            at_keyword.write_css(cssw)?;
                                            for (index, q) in expr.iter().enumerate() {
                                                if index > 0 { cssw.write_delim(",", false)?; }
                                                q.write_css(cssw)?;
                                            }
                                            Some(items)
                                        } else {
                                            None
                                        }
                                    }
                                    AtBlock::Supports { at_keyword, expr, items } => {
                                        if items.block.as_slice().len() > 0 {
                                            at_keyword.write_css(cssw)?;
                                            expr.write_css(cssw)?;
                                            Some(items)
                                        } else {
                                            None
                                        }
                                    }
                                };
                                if let Some(items) = items {
                                    cssw.write_brace_block(|cssw| {
                                        write_selector(cssw)?;
                                        if let Some(ident) = pseudo_class {
                                            cssw.write_delim(":", false)?;
                                            cssw.write_ident(&ident.css_name(), false)?;
                                        }
                                        cssw.write_brace_block(|cssw| {
                                            write_prop_list(cssw, items.block.as_slice())
                                        })?;
                                        Ok(())
                                    })?;
                                }
                            }
                            Ok(())
                        };

                        // write CSS for the class itself
                        write_main_rule_and_at_blocks(cssw, None, &content.props, &content.at_blocks)?;
                        for c in content.pseudo_classes.iter() {
                            write_main_rule_and_at_blocks(
                                cssw,
                                Some(&c.ident),
                                c.content.props.as_slice(),
                                c.content.at_blocks.as_slice(),
                            )?;
                        }

                        // write CSS for sub classes
                        for c in content.sub_classes.iter() {
                            let full_ident = CssIdent {
                                formal_name: format!("{}{}", full_ident.formal_name, c.partial_ident.formal_name),
                                span: c.partial_ident.span,
                            };
                            handle_rule_content(
                                tokens,
                                inner_tokens,
                                name_mangling,
                                debug_mode,
                                &full_ident,
                                content,
                                cssw,
                            )?;
                        }

                        Ok(())
                    }
                    let mut s = String::new();
                    let mut cssw = CssWriter::new(&mut s, debug_mode);
                    handle_rule_content(
                        tokens,
                        &mut inner_tokens,
                        name_mangling,
                        debug_mode,
                        ident,
                        &content.block,
                        &mut cssw,
                    ).unwrap();

                    // write generated string to file
                    if let Some(css_out_file) = CSS_OUT_FILE.as_ref() {
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
    use std::io::Seek;
    use std::path::Path;

    use serial_test::serial;

    use super::*;

    pub(crate) struct Env<'a> {
        out_dir: &'a Path,
        import_dir: &'a Path,
    }

    impl<'a> Env<'a> {
        pub(crate) fn write_import_file(&self, name: &str, content: &str) {
            std::fs::write(&self.import_dir.join(name), content).unwrap();
        }

        pub(crate) fn read_output(&self) -> String {
            std::fs::read_to_string(&self.out_dir.join("maomi_dom_macro.css")).unwrap()
        }
    }

    static TEST_DIRS: Lazy<(PathBuf, PathBuf)> = Lazy::new(|| {
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
        (out_dir, import_dir)
    });
    
    pub(crate) fn setup_env(debug_mode: bool, f: impl FnOnce(Env)) {
        CSS_OUT_MODE.with(|css_out_mode| {
            css_out_mode.set(match debug_mode {
                false => CssOutMode::Release,
                true => CssOutMode::Debug,
            });
        });
        let (out_dir, import_dir) = &*TEST_DIRS;
        if let Some(css_out_file) = CSS_OUT_FILE.as_ref() {
            let mut file = css_out_file.lock().unwrap();
            file.rewind().unwrap();
            file.set_len(0).unwrap();
        }
        f(Env {
            out_dir,
            import_dir,
        });
    }

    pub(crate) fn parse_str(s: &str) -> String {
        let ss: StyleSheet<DomStyleSheet> = syn::parse_str(s).unwrap();
        quote!(#ss).to_string()
    }

    #[test]
    #[serial]
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

    #[test]
    #[serial]
    fn media() {
        setup_env(false, |env| {
            parse_str(r#"
                @config name_mangling: off;
                .c {
                    padding: 1px;
                    @media (aspect_ratio: 16/9) {
                        margin: 2px;
                    }
                }
            "#);
            assert_eq!(
                env.read_output(),
                r#".c{padding:1px}@media(aspect-ratio:16/9){.c{margin:2px}}"#,
            );
        });
        setup_env(true, |env| {
            parse_str(r#"
                @config name_mangling: off;
                .c {
                    padding: 1px;
                    @media (aspect_ratio: 16/9) {
                        margin: 2px;
                    }
                }
            "#);
            assert_eq!(
                env.read_output(),
                r#"
.c {
    padding: 1px;
}

@media (aspect-ratio: 16 / 9) {
    .c {
        margin: 2px;
    }
}
"#,
            );
        });
    }
}
