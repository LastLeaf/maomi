use once_cell::sync::Lazy;
use quote::{quote, TokenStreamExt, quote_spanned};
use std::cell::Cell;
use std::fs::File;
use std::hash::Hasher;
use std::io::Write;
use std::path::PathBuf;

use maomi_skin::write_css::{CssWriter, WriteCss};
use maomi_skin::{css_token::*, VarDynValue, MaybeDyn};
use maomi_skin::style_sheet::*;
use maomi_skin::{ParseError, pseudo};

mod media_cond;
use media_cond::*;
mod property;
use property::*;

const CLASS_CHARS: [char; 63] = [
    '_', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
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
    static CARGO_PKG_NAME: String = {
        std::env::var("CARGO_PKG_NAME").unwrap_or_default()
    };
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

fn generate_span_hash(span: proc_macro2::Span) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    CARGO_PKG_NAME.with(|x| {
        hasher.write(x.as_bytes());
    });
    hasher.write(format!("{:?}", span).as_bytes());
    let mut h = hasher.finish();
    let mut ret = String::with_capacity(16);
    ret.push(CLASS_START_CHARS[(h % CLASS_START_CHARS.len() as u64) as usize]);
    h /= CLASS_START_CHARS.len() as u64;
    while h > 0 {
        ret.push(CLASS_CHARS[(h % CLASS_CHARS.len() as u64) as usize]);
        h /= CLASS_CHARS.len() as u64;
    }
    ret
}

fn generate_css_name(full_ident: &VarName, debug_mode: bool) -> String {
    let class_id = generate_span_hash(full_ident.span());
    if debug_mode {
        full_ident.css_name() + "_" + &class_id
    } else {
        class_id
    }
}

pub(crate) struct DomStyleSheet {
    key_frames_def: Vec<(CssIdent, Vec<KeyFrame<DomCssProperty>>)>,
}

impl StyleSheetConstructor for DomStyleSheet {
    type PropertyValue = DomCssProperty;
    type MediaCondValue = DomMediaCondValue;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            key_frames_def: vec![],
        }
    }

    fn define_key_frames(
        &mut self,
        name: &VarName,
        css_name: &Option<String>,
        content: Vec<KeyFrame<Self::PropertyValue>>,
    ) -> Result<CssToken, ParseError> {
        let debug_mode = CSS_OUT_MODE.with(|x| x.get() == CssOutMode::Debug);
        let generated_ident = CssIdent::new(
            name.span(),
            css_name.as_ref().unwrap_or(&generate_css_name(&name, debug_mode)),
        );
        self.key_frames_def.push((
            generated_ident.clone(),
            content,
        ));
        Ok(CssToken::Ident(generated_ident).into())
    }

    fn to_tokens(&self, ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized,
    {
        let debug_mode = CSS_OUT_MODE.with(|x| x.get() == CssOutMode::Debug);
        let inner_tokens = &mut proc_macro2::TokenStream::new();

        // a helper for write prop list
        fn write_prop_list(
            tokens: &mut proc_macro2::TokenStream,
            debug_mode: bool,
            cssw: &mut CssWriter<String>,
            items: &[StyleContentItem<DomCssProperty>],
            var_context: &VarContext<DomStyleSheet>,
            var_values: &[VarDynValue],
            is_last_item: bool,
        ) -> Result<(), std::fmt::Error> {
            for (index, item) in items.iter().enumerate() {
                let is_last_item = is_last_item && index + 1 == items.len();
                match item {
                    StyleContentItem::CompilationError(err) => {
                        tokens.append_all(err.to_compile_error());
                    }
                    StyleContentItem::Property(prop) => {
                        prop.name.write_css_with_args(cssw, var_values)?;
                        cssw.write_colon()?;
                        prop.value.write_css_with_args(cssw, var_values)?;
                        if debug_mode || !is_last_item {
                            cssw.write_semi()?;
                            if debug_mode {
                                cssw.line_wrap()?;
                            }
                        }
                    }
                    StyleContentItem::StyleRef(name, args) => {
                        let style_fn = var_context.get(&name);
                        let style_fn = style_fn.as_ref()
                            .and_then(|x| {
                                if let StyleSheetItem::StyleFn(x) = &**x {
                                    Some(x)
                                } else {
                                    None
                                }
                            })
                            .expect("style section not found");
                        let args: Vec<_> = args.iter().map(|arg| {
                            match arg {
                                MaybeDyn::Dyn(x) => var_values.get(x.index).expect("argument value not enough"),
                                MaybeDyn::Static(v) => v,
                            }.clone()
                        }).collect();
                        write_prop_list(
                            tokens,
                            debug_mode,
                            cssw,
                            &style_fn.content,
                            &style_fn.var_context,
                            &args,
                            is_last_item,
                        )?;
                    }
                }
            }
            Ok(())
        }

        // generate @keyframes output
        if let Some(css_out_file) = CSS_OUT_FILE.as_ref() {
            for (generated_ident, content) in self.key_frames_def.iter() {
                let mut s = String::new();
                let cssw = &mut CssWriter::new(&mut s, debug_mode);
                cssw.write_at_keyword("keyframes").unwrap();
                generated_ident.write_css(cssw).unwrap();
                cssw.write_brace_block(|cssw| {
                    for kf in content.iter() {
                        kf.progress.write_css(cssw)?;
                        cssw.write_brace_block(|cssw| {
                            write_prop_list(
                                tokens,
                                debug_mode,
                                cssw,
                                &kf.items,
                                &ss.var_context,
                                &[],
                                true,
                            )
                        })?;
                    }
                    Ok(())
                })
                .unwrap();
                css_out_file.lock().unwrap().write(s.as_bytes()).unwrap();
            }
        }

        // generate css output
        for item in ss.items.iter() {
            match &**item {
                // generate compilation error
                StyleSheetItem::CompilationError(err) => {
                    tokens.append_all(err.to_compile_error());
                }

                // generate const def
                StyleSheetItem::ConstValue(ConstValueDefinition { name, .. }) => {
                    tokens.append_all(quote! {
                        const #name: &'static str = "(value)";
                    });
                }
                StyleSheetItem::KeyFrames(KeyFramesDefinition { name, .. }) => {
                    tokens.append_all(quote! {
                        const #name: &'static str = "(keyframes)";
                    });
                }
                StyleSheetItem::StyleFn(StyleFnDefinition { name, args, sub_var_refs, .. }) => {
                    let args_var_name = args.iter().map(|x| &x.0);
                    let args_ty = args.iter().map(|x| x.1.type_tokens());
                    tokens.append_all(quote! {
                        #[allow(non_camel_case_types)]
                        struct #name();
                        impl #name {
                            #[allow(dead_code)]
                            fn __stylesheet(#(#args_var_name: #args_ty),*) {
                                #(#sub_var_refs;)*
                            }
                        }
                    });
                }

                // style as const def
                StyleSheetItem::Style(StyleDefinition {
                    extern_vis,
                    name,
                    args,
                    content,
                    sub_var_refs,
                    ..
                }) => {
                    let args_var_name = args.iter().map(|x| &x.0);
                    let args_ty = args.iter().map(|x| x.1.type_tokens());
                    tokens.append_all(quote! {
                        #[allow(non_camel_case_types)]
                        #extern_vis struct #name();
                        impl #name {
                            #[allow(dead_code)]
                            fn __stylesheet(#(#args_var_name: #args_ty),*) {
                                #(#sub_var_refs;)*
                            }
                        }
                        // TODO style
                    });
                }

                // generate common rule
                StyleSheetItem::Class(ClassDefinition {
                    extern_vis,
                    error_css_output,
                    css_name,
                    name,
                    content,
                    sub_var_refs,
                    ..
                }) => {
                    let var_context = &ss.var_context;
                    let class_name = css_name.clone().unwrap_or_else(|| generate_css_name(name, debug_mode));

                    // generate proc macro output
                    tokens.append_all(quote! {
                        #[allow(non_camel_case_types)]
                        #extern_vis struct #name {}
                        impl #name {
                            #[allow(dead_code)]
                            fn __stylesheet() {
                                #(#sub_var_refs;)*
                            }
                        }
                        impl maomi::prop::ListPropertyItem<maomi_dom::class_list::DomClassList, bool> for #name {
                            type Value = &'static str;
                            #[inline(always)]
                            fn item_value<'a>(
                                _dest: &mut maomi_dom::class_list::DomClassList,
                                _index: usize,
                                _s: &'a bool,
                                _ctx: &mut <maomi_dom::class_list::DomClassList as maomi::prop::ListPropertyInit>::UpdateContext,
                            ) -> &'a Self::Value {
                                &#class_name
                            }
                        }
                        impl maomi::prop::ListPropertyItem<maomi_dom::class_list::DomExternalClasses, bool> for #name {
                            type Value = &'static str;
                            #[inline(always)]
                            fn item_value<'a>(
                                _dest: &mut maomi_dom::class_list::DomExternalClasses,
                                _index: usize,
                                _s: &'a bool,
                                _ctx: &mut <maomi_dom::class_list::DomExternalClasses as maomi::prop::ListPropertyInit>::UpdateContext,
                            ) -> &'a Self::Value {
                                &#class_name
                            }
                        }
                    });

                    // generate all CSS rules
                    fn handle_rule_content(
                        tokens: &mut proc_macro2::TokenStream,
                        debug_mode: bool,
                        class_name: &str,
                        pseudo: Option<&pseudo::Pseudo>,
                        content: &RuleContent<DomStyleSheet>,
                        cssw: &mut CssWriter<String>,
                        var_context: &VarContext<DomStyleSheet>,
                    ) -> Result<(), std::fmt::Error> {
                        // a helper for write css name
                        let write_selector = |cssw: &mut CssWriter<String>| {
                            cssw.write_delim(".", true)?;
                            cssw.write_ident(&class_name, false)?;
                            Ok(())
                        };

                        // a helper for write at-blocks
                        let mut write_main_rule_and_at_blocks =
                            |
                                cssw: &mut CssWriter<String>,
                                pseudo: Option<&pseudo::Pseudo>,
                                items: &[StyleContentItem<DomCssProperty>],
                                at_blocks: &[AtBlock<DomStyleSheet>],
                            | {
                                if items.len() > 0 {
                                    write_selector(cssw)?;
                                    if let Some(pseudo) = pseudo {
                                        cssw.write_delim(":", false)?;
                                        pseudo.write_css(cssw)?;
                                    }
                                    cssw.write_brace_block(|cssw| write_prop_list(
                                        tokens,
                                        debug_mode,
                                        cssw,
                                        &items,
                                        var_context,
                                        &[],
                                        true,
                                    ))?;
                                }
                                for block in at_blocks {
                                    let content = match block {
                                        AtBlock::Media {
                                            expr,
                                            content,
                                        } => {
                                            if content.items.len() > 0 {
                                                cssw.write_at_keyword("media")?;
                                                for (index, q) in expr.iter().enumerate() {
                                                    if index > 0 {
                                                        cssw.write_delim(",", false)?;
                                                    }
                                                    q.write_css(cssw)?;
                                                }
                                                Some(content)
                                            } else {
                                                None
                                            }
                                        }
                                        AtBlock::Supports {
                                            expr,
                                            content,
                                        } => {
                                            if content.items.len() > 0 {
                                                cssw.write_at_keyword("supports")?;
                                                expr.write_css(cssw)?;
                                                Some(content)
                                            } else {
                                                None
                                            }
                                        }
                                    };
                                    if let Some(content) = content {
                                        cssw.write_brace_block(|cssw| {
                                            handle_rule_content(
                                                tokens,
                                                debug_mode,
                                                class_name,
                                                pseudo,
                                                &content,
                                                cssw,
                                                var_context,
                                            )
                                        })?;
                                    }
                                }
                                Ok(())
                            };

                        // write CSS for the class itself
                        write_main_rule_and_at_blocks(
                            cssw,
                            pseudo,
                            &content.items,
                            &content.at_blocks,
                        )?;
                        for c in content.pseudo_classes.iter() {
                            write_main_rule_and_at_blocks(
                                cssw,
                                Some(&c.pseudo),
                                c.content.items.as_slice(),
                                c.content.at_blocks.as_slice(),
                            )?;
                        }

                        Ok(())
                    }

                    // write generated string to file
                    if let Some(span) = error_css_output {
                        let span = *span;
                        let mut s = String::new();
                        if debug_mode {
                            s += "/* error_css_output */\n";
                        }
                        let mut cssw = CssWriter::new(&mut s, debug_mode);
                        handle_rule_content(
                            tokens,
                            debug_mode,
                            &class_name,
                            None,
                            &content,
                            &mut cssw,
                            &var_context,
                        )
                        .unwrap();
                        tokens.append_all(quote_spanned! {span=>
                            compile_error!(#s);
                        });
                    } else if let Some(css_out_file) = CSS_OUT_FILE.as_ref() {
                        let mut s = String::new();
                        let mut cssw = CssWriter::new(&mut s, debug_mode);
                        handle_rule_content(
                            tokens,
                            debug_mode,
                            &class_name,
                            None,
                            &content,
                            &mut cssw,
                            var_context,
                        )
                        .unwrap();
                        css_out_file.lock().unwrap().write(s.as_bytes()).unwrap();
                    } else {
                        // empty
                    }
                }
            }
        }

        // write refs
        for r in &ss.var_refs {
            inner_tokens.append_all(quote! {
                #r;
            });
        }

        // write extra tokens
        let fn_name = syn::Ident::new(
            &generate_span_hash(proc_macro2::Span::call_site()),
            proc_macro2::Span::call_site(),
        );
        tokens.append_all(quote! {
            fn #fn_name() {
                #inner_tokens
            }
        });
    }
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
        std::env::set_var("MAOMI_CSS_OUT_DIR", out_dir.to_str().unwrap());
        let import_dir = tmp_path.join("maomi-dom-macro").join("test-import");
        std::fs::create_dir_all(&import_dir).unwrap();
        std::env::set_var("MAOMI_STYLESHEET_MOD_ROOT", import_dir.join("lib.mcss").to_str().unwrap());
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
            env.write_import_file(
                "lib.mcss",
                r#"
                    const A: value = Px(1);
                    const COLOR: value = rgb(1, 2, 3);
                "#,
            );
            parse_str(
                r#"
                    use crate::*;
                    const B: value = A;
                    #[css_name("self")]
                    .self {
                        padding: B A;
                        color: COLOR;
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#".self{padding:1px 1px;color:rgb(1,2,3)}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn style_fn() {
        setup_env(false, |env| {
            parse_str(
                r#"
                    fn a(padding: f32, color: &str) {
                        padding = Px(padding);
                        color = Color(color);
                    }
                    #[css_name("c")]
                    class cc {
                        a(1.2, "aaa");
                        if hover {
                            a(34, "bbb");
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#".c{padding:1.2px;color:#aaa}.c:hover{padding:34px;color:#bbb}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn cascaded_style_fn() {
        setup_env(false, |env| {
            parse_str(
                r#"
                    fn a(v: f32) {
                        padding = Em(v);
                    }
                    fn b(v: f32) {
                        a(v);
                        color = Color("123456");
                    }
                    #[css_name("c")]
                    class c {
                        b(1);
                        a(2);
                    }
                "#,
            );
            assert_eq!(env.read_output(), r#".c{padding:1em;color:#123456;padding:2em}"#,);
        });
    }

    #[test]
    #[serial]
    fn media() {
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        padding = Px(1);
                        if media (aspect_ratio = 16/9) {
                            margin = Px(2);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#".c{padding:1px}@media(aspect-ratio:16/9){.c{margin:2px}}"#,
            );
        });
        setup_env(true, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        padding = Px(1);
                        if media (aspect_ratio = 16/9) {
                            margin = Px(2);
                        }
                    }
                "#,
            );
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

    #[test]
    #[serial]
    fn supports() {
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        padding = Px(1);
                        if supports (margin = Percent(2)) {
                            margin = Percent(2);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#".c{padding:1px}@supports(margin:2%){.c{margin:2%}}"#,
            );
        });
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if supports not (margin = Px(2)) {
                            margin = Px(2);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@supports not (margin:2px){.c{margin:2px}}"#,
            );
        });
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if supports (margin = Px(2)) and (margin = Px(3)) {
                            margin = Px(2);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@supports(margin:2px)and (margin:3px){.c{margin:2px}}"#,
            );
        });
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if supports (margin = Px(2)) or (margin = Px(3)) {
                            margin = Px(2);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@supports(margin:2px)or (margin:3px){.c{margin:2px}}"#,
            );
        });
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if supports (not ((margin = Px(2)))) and ((((margin = Px(3))) or (margin = Px(4)))) {
                            margin = Px(2);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@supports(not (margin:2px))and ((margin:3px)or (margin:4px)){.c{margin:2px}}"#,
            );
        });
        setup_env(false, |_| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    class c {
                        if supports margin = Px(2) {}
                    }
                "#
            )
            .is_err());
        });
        setup_env(false, |_| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    class c {
                        if supports (margin = Px(2)) and (margin = Px(3)) or (margin = Px(4)) {}
                    }
                "#
            )
            .is_err());
        });
    }

    #[test]
    #[serial]
    fn key_frames() {
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("kf")]
                    const KF: keyframes = {
                        from {
                            transform = translateX(0);
                        }
                        50% {
                            transform = translateX(10%);
                        }
                        to {
                            transform = translateX(100%);
                        }
                    };
                    #[css_name("c")]
                    class c {
                        animation_name = KF;
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@keyframes kf{0%{transform:translateX(0)}50%{transform:translateX(10%)}100%{transform:translateX(100%)}}.c{animation-name:kf}"#,
            );
        });
        setup_env(true, |env| {
            parse_str(
                r#"
                    #[css_name("kf")]
                    const KF: keyframes = {
                        from {
                            transform = translateX(0);
                        }
                        50% {
                            transform = translateX(10%);
                        }
                        to {
                            transform = translateX(100%);
                        }
                    };
                    #[css_name("c")]
                    class c {
                        animation_name = KF;
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"
@keyframes kf {
    0% {
        transform: translateX(0);
    }

    50% {
        transform: translateX(10%);
    }

    100% {
        transform: translateX(100%);
    }
}

.c {
    animation-name: kf;
}
"#,
            );
        });
    }

    #[test]
    #[serial]
    fn pseudo_classes() {
        setup_env(false, |env| {
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        padding = Px(1);
                        if hover {
                            if media (aspect_ratio = 16/9) {
                                margin = Px(2);
                            }
                        }
                        if active {
                            margin = Px(3);
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#".c{padding:1px}@media(aspect-ratio:16/9){.c:hover{margin:2px}}.c:active{margin:3px}"#,
            );
        });
        setup_env(false, |_| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    class c {
                        if hover {
                            if active {}
                        }
                    }
                "#
            )
            .is_err());
        });
    }
}
