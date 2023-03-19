use std::{sync::Mutex, path::PathBuf, io::Write};
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;

use maomi_tools::i18n::*;

const DEFAULT_GROUP_NAME: &'static str = "translation";

thread_local! {
    static DIR_LOCALE_NAME: Option<(PathBuf, String)> = {
        maomi_tools::config::crate_config(|crate_config| {
            let locale_name = match crate_config.i18n_locale.as_ref() {
                None => {
                    return None;
                }
                Some(x) => x.clone(),
            };
            let dir_name = match crate_config.i18n_dir.as_ref() {
                None => {
                    return None;
                }
                Some(x) => x.clone(),
            };
            Some((dir_name, locale_name))
        })
    };
}

static CUR_LOCALE: Lazy<Result<Option<Locale>, String>> = Lazy::new(|| read_locale());
static FORMAT_METADATA_OUTPUT: Lazy<Option<Mutex<std::fs::File>>> = Lazy::new(|| {
    maomi_tools::config::crate_config(|crate_config| {
        if crate_config.i18n_format_metadata {
            DIR_LOCALE_NAME.with(|x| {
                if let Some((dir_name, locale_name)) = x.as_ref() {
                    let tmp_dir = dir_name.join("format-metadata");
                    std::fs::create_dir_all(&tmp_dir).ok()?;
                    let gitignore = tmp_dir.join(".gitignore");
                    if !std::path::Path::exists(&gitignore) {
                        let _ = std::fs::File::create(&gitignore).map(|mut file| {
                            write!(file, "*")
                        });
                    }
                    let p = tmp_dir.join(&(locale_name.clone() + ".toml"));
                    let mut file = std::fs::File::create(p).ok()?;
                    writeln!(file, "version = {}", METADATA_VERSION).ok()?;
                    Some(Mutex::new(file))
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
});

fn read_locale() -> Result<Option<Locale>, String> {
    DIR_LOCALE_NAME.with(|x| {
        if let Some((dir_name, locale_name)) = x.as_ref() {
            let file_name = dir_name.join(&(locale_name.clone() + ".toml"));
            let content = std::fs::read(&file_name)
                .map_err(|_| format!("cannot read i18n file {:?}", file_name))?;
            let locale = toml::from_str(&String::from_utf8_lossy(&content))
                .map_err(|x| format!("parsing i18n TOML failed: {}", x))?;
            Ok(Some(locale))
        } else {
            Ok(None)
        }
    })
}

pub(crate) struct LocaleGroup {
    namespace: Option<String>,
    inner: LocaleGroupStatus,
}

enum LocaleGroupStatus {
    NotNeeded,
    Normal(&'static FxHashMap<String, String>),
    Missing(String),
}

impl LocaleGroup {
    pub(crate) fn get_default() -> LocaleGroup {
        Self::get(DEFAULT_GROUP_NAME)
    }

    pub(crate) fn get(group: &str) -> LocaleGroup {
        let locale = CUR_LOCALE.as_ref().ok().and_then(|locale| {
            locale.as_ref()
        });
        if let Some(locale) = locale {
            let inner = match locale.get(group) {
                Some(x) => LocaleGroupStatus::Normal(x),
                None => LocaleGroupStatus::Missing(group.to_string()),
            };
            Self {
                namespace: FORMAT_METADATA_OUTPUT.as_ref().map(|_| group.to_string()),
                inner,
            }
        } else {
            Self {
                namespace: FORMAT_METADATA_OUTPUT.as_ref().map(|_| group.to_string()),
                inner: LocaleGroupStatus::NotNeeded,
            }
        }
    }

    pub(crate) fn need_trans(&self) -> bool {
        if let LocaleGroupStatus::NotNeeded = self.inner {
            false
        } else {
            true
        }
    }

    pub(crate) fn trans<'a>(&'a self, s: &'a str) -> TransRes<'a> {
        match &self.inner {
            LocaleGroupStatus::Normal(x) => {
                let ret = match x.get(s) {
                    None => TransRes::LackTrans,
                    Some(s) => TransRes::Done(s),
                };
                if let Some(file) = &*FORMAT_METADATA_OUTPUT {
                    let file = &mut *file.lock().unwrap();
                    let items = FormatMetadata {
                        item: vec![FormatMetadataItem {
                            namespace: self.namespace.as_ref().unwrap(),
                            src: s,
                            translated: if let TransRes::Done(s) = &ret {
                                Some(*s)
                            } else {
                                None
                            },
                        }],
                    };
                    write!(file, "{}", toml::to_string(&items).unwrap()).unwrap();
                }
                ret
            }
            LocaleGroupStatus::Missing(x) => {
                if let Some(file) = &*FORMAT_METADATA_OUTPUT {
                    let file = &mut *file.lock().unwrap();
                    let items = FormatMetadata {
                        item: vec![FormatMetadataItem {
                            namespace: self.namespace.as_ref().unwrap(),
                            src: s,
                            translated: None,
                        }],
                    };
                    write!(file, "{}", toml::to_string(&items).unwrap()).unwrap();
                }
                TransRes::LackTransGroup(x.as_str())
            }
            LocaleGroupStatus::NotNeeded => {
                TransRes::NotNeeded
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TransRes<'a> {
    NotNeeded,
    Done(&'a str),
    LackTrans,
    LackTransGroup(&'a str),
}

pub(crate) mod mac {
    use quote::*;
    use syn::*;
    use syn::parse::*;

    pub(crate) struct I18nGroupArgs {
        group: Ident,
        macro_name: Ident,
    }

    impl Parse for I18nGroupArgs {
        fn parse(input: ParseStream) -> Result<Self> {
            let group = input.parse()?;
            input.parse::<Token![as]>()?;
            let macro_name = input.parse()?;
            Ok(Self { group, macro_name })
        }
    }

    impl ToTokens for I18nGroupArgs {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let Self { group, macro_name } = self;
            tokens.append_all(quote! {
                macro_rules! #macro_name {
                    ($($t: tt)*) => {
                        maomi::prelude::i18n_group_format!(#group, $($t)*)
                    }
                }
            });
        }
    }

    struct I18nVar {
        name: Option<Ident>,
        expr: Expr,
    }

    impl Parse for I18nVar {
        fn parse(input: ParseStream) -> Result<Self> {
            if input.peek(Ident) && input.peek2(Token![=]) {
                let name = input.parse()?;
                input.parse::<Token![=]>()?;
                let expr = input.parse()?;
                Ok(Self { name: Some(name), expr })
            } else {
                let expr = input.parse()?;
                Ok(Self { name: None, expr })
            }
        }
    }

    impl ToTokens for I18nVar {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let expr = &self.expr;
            let r = if let Some(name) = self.name.as_ref() {
                quote! {
                    #name = maomi::locale_string::ToLocaleStr::to_locale_str(&(#expr))
                }
            } else {
                quote! {
                    maomi::locale_string::ToLocaleStr::to_locale_str(&(#expr))
                }
            };
            tokens.append_all(r);
        }
    }

    pub(crate) struct I18nArgs {
        s: LitStr,
        vars: Vec<I18nVar>,
    }
    
    impl Parse for I18nArgs {
        fn parse(input: ParseStream) -> Result<Self> {
            let s = input.parse()?;
            let mut vars = vec![];
            while !input.is_empty() {
                input.parse::<Token![,]>()?;
                vars.push(input.parse()?);
            }
            Ok(Self { s, vars })
        }
    }

    impl ToTokens for I18nArgs {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            trans_to_tokens(&self, None, tokens);
        }
    }

    pub(crate) struct I18nGroupFormatArgs {
        group: Ident,
        args: I18nArgs,
    }

    impl Parse for I18nGroupFormatArgs {
        fn parse(input: ParseStream) -> Result<Self> {
            let group = input.parse()?;
            input.parse::<Token![,]>()?;
            let args = input.parse()?;
            Ok(Self { group, args })
        }
    }

    impl ToTokens for I18nGroupFormatArgs {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            trans_to_tokens(&self.args, Some(&self.group), tokens);
        }
    }

    fn trans_to_tokens(args: &I18nArgs, group: Option<&Ident>, tokens: &mut proc_macro2::TokenStream) {
        let locale_group = match group {
            None => super::LocaleGroup::get_default(),
            Some(group) => super::LocaleGroup::get(&group.to_string()),
        };
        let s = &args.s;
        let vars = &args.vars;
        let span = s.span();
        let r = match locale_group.trans(&s.value()) {
            super::TransRes::LackTrans => quote_spanned! {span=> compile_error!("lacks translation") },
            super::TransRes::LackTransGroup(x) => {
                let msg = format!("translation group {:?} not found", x);
                quote_spanned! {span=> compile_error!(#msg) }
            },
            super::TransRes::Done(x) => {
                let s = LitStr::new(x, span);
                if args.vars.len() == 0 {
                    quote! { maomi::locale_string::LocaleStaticStr::translated(#s) }
                } else {
                    quote! { maomi::locale_string::LocaleString::translated(format!(#s, #(#vars),*)) }
                }
            }
            super::TransRes::NotNeeded => {
                if args.vars.len() == 0 {
                    quote_spanned! {span=> maomi::locale_string::LocaleStaticStr::translated(#s) }
                } else {
                    quote_spanned! {span=> maomi::locale_string::LocaleString::translated(format!(#s, #(#vars),*)) }
                }
            }
        };
        tokens.append_all(r);
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};
    use serial_test::serial;

    use super::*;

    pub(crate) struct Env<'a> {
        locale_dir: &'a Path,
    }

    impl<'a> Env<'a> {
        pub(crate) fn write_locale_file(&self, name: &str, content: &str) {
            std::fs::write(&self.locale_dir.join(name), content).unwrap();
        }
    }

    static TEST_DIRS: Lazy<PathBuf> = Lazy::new(|| {
        let tmp_path = std::env::temp_dir();
        let locale_dir = tmp_path.join("maomi-macro").join("test-i18n");
        std::fs::create_dir_all(&locale_dir).unwrap();
        std::env::set_var("MAOMI_I18N_DIR", locale_dir.to_str().unwrap());
        locale_dir
    });

    pub(crate) fn setup_env<R>(locale_name: &str, f: impl FnOnce(Env) -> R) -> R {
        let locale_dir = &*TEST_DIRS;
        std::env::set_var("MAOMI_I18N_LOCALE", locale_name);
        let r = f(Env {
            locale_dir,
        });
        std::env::remove_var("MAOMI_I18N_LOCALE");
        r
    }

    #[test]
    #[serial]
    fn simple_translation() {
        fn parse_str(s: &str) -> String {
            let ss: mac::I18nArgs = syn::parse_str(s).unwrap();
            quote::quote!(#ss).to_string()
        }
        let a = setup_env("test", |env| {
            env.write_locale_file("test.toml", r#"
                [translation]
                "abc" = "def"
            "#);
            parse_str(r#""abc""#)
        });
        assert_eq!(a, r#"maomi :: locale_string :: LocaleStaticStr :: translated ("def")"#);
    }

    #[test]
    #[serial]
    fn scoped_translation() {
        fn parse_str(s: &str) -> String {
            let ss: mac::I18nGroupFormatArgs = syn::parse_str(s).unwrap();
            quote::quote!(#ss).to_string()
        }
        let a = setup_env("test", |env| {
            env.write_locale_file("test.toml", r#"
                [translation]
                "abc" = "def"
                [tt]
                "abc" = "ghi"
            "#);
            parse_str(r#"tt, "abc""#)
        });
        assert_eq!(a, r#"maomi :: locale_string :: LocaleStaticStr :: translated ("ghi")"#);
    }
}
