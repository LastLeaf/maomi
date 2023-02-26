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
                    let file = std::fs::File::create(p).ok()?;
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
    inner: Option<&'static FxHashMap<String, String>>,
}

impl LocaleGroup {
    pub(crate) fn get_default() -> Result<LocaleGroup, String> {
        Self::get(DEFAULT_GROUP_NAME)
    }

    pub(crate) fn get(group: &str) -> Result<LocaleGroup, String> {
        CUR_LOCALE.as_ref().map_err(|x| x.clone()).and_then(|locale| {
            if let Some(locale) = locale {
                match locale.get(group) {
                    None => Err(format!("no translation group {:?} found", group)),
                    Some(x) => Ok(Self {
                        namespace: FORMAT_METADATA_OUTPUT.as_ref().map(|_| group.to_string()),
                        inner: Some(x),
                    }),
                }
            } else {
                Ok(Self {
                    namespace: FORMAT_METADATA_OUTPUT.as_ref().map(|_| group.to_string()),
                    inner: None,
                })
            }
        })
    }

    pub(crate) fn need_trans(&self) -> bool {
        self.inner.is_some()
    }

    pub(crate) fn trans<'a>(&'a self, s: &'a str) -> TransRes<'a> {
        if let Some(x) = &self.inner {
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
        } else {
            TransRes::NotNeeded
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TransRes<'a> {
    NotNeeded,
    Done(&'a str),
    LackTrans,
}

pub(crate) mod mac {
    use quote::*;
    use syn::*;
    use syn::parse::*;

    pub(crate) struct I18nArgs {
        group: Option<Ident>,
        s: LitStr,
    }
    
    impl Parse for I18nArgs {
        fn parse(input: ParseStream) -> Result<Self> {
            let la = input.lookahead1();
            let ret = if la.peek(Ident) {
                let group = input.parse()?;
                let _: token::Comma = input.parse()?;
                let s = input.parse()?;
                Self { group, s }
            } else {
                let s = input.parse()?;
                Self { group: None, s }
            };
            Ok(ret)
        }
    }
    
    impl ToTokens for I18nArgs {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let locale_group_res = match &self.group {
                None => super::LocaleGroup::get_default(),
                Some(group) => super::LocaleGroup::get(&group.to_string()),
            };
            let r = match locale_group_res {
                Err(e) => {
                    let hint = format!("{}", e);
                    quote! { compile_error!(#hint) }
                }
                Ok(locale_group) => {
                    let s = &self.s;
                    let span = s.span();
                    match locale_group.trans(&s.value()) {
                        super::TransRes::LackTrans => quote_spanned! {span=> compile_error!("lacks translation") },
                        super::TransRes::Done(x) => {
                            let s = LitStr::new(x, span);
                            quote! { maomi::locale_string::LocaleStaticStr::translated(#s) }
                        }
                        super::TransRes::NotNeeded => quote_spanned! {span=> maomi::locale_string::LocaleStaticStr::translated(#s) },
                    }
                }
            };
            tokens.append_all(r);
        }
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

    pub(crate) fn parse_str(s: &str) -> String {
        let ss: mac::I18nArgs = syn::parse_str(s).unwrap();
        quote::quote!(#ss).to_string()
    }

    #[test]
    #[serial]
    fn simple_translation() {
        let (a, b) = setup_env("test", |env| {
            env.write_locale_file("test.toml", r#"
                [translation]
                "abc" = "def"
                [tt]
                "abc" = "ghi"
            "#);
            (parse_str(r#""abc""#), parse_str(r#"tt, "abc""#))
        });
        assert_eq!(a, r#"maomi :: locale_string :: LocaleStaticStr :: translated ("def")"#);
        assert_eq!(b, r#"maomi :: locale_string :: LocaleStaticStr :: translated ("ghi")"#);
    }
}
