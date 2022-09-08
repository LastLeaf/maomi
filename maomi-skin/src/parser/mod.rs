//! Parsing details of the stylesheets

use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::str::FromStr;

use quote::ToTokens;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

mod css_token;
pub use css_token::*;
mod mac;
use mac::MacroDefinition;

thread_local! {
    static CSS_IMPORT_DIR: Option<PathBuf> = {
        std::env::var("MAOMI_CSS_IMPORT_DIR")
            .map(|s| PathBuf::from(&s))
            .or_else(|_| {
                std::env::var("CARGO_MANIFEST_DIR")
                    .map(|s| PathBuf::from(&s).join("src"))
            })
            .ok()
    };
}

fn get_import_content(src: &CssString) -> Result<String> {
    let p = src.value();
    if !p.starts_with("/") {
        return Err(Error::new(
            src.span(),
            "Currently only paths started with `/` are supported (which means the path relative to crate `src` or MAOMI_CSS_IMPORT_DIR)",
        ))
    }
    let mut target = CSS_IMPORT_DIR.with(|import_dir| {
        match import_dir {
            None => Err(Error::new(
                src.span(),
                "No MAOMI_CSS_IMPORT_DIR or CARGO_MANIFEST_DIR environment variables provided",
            )),
            Some(s) => Ok(s.clone()),
        }
    })?;
    for slice in p[1..].split('/') {
        match slice {
            "." => {}
            ".." => { target.pop(); }
            x => { target.push(x); }
        }
    }
    std::fs::read_to_string(&target).map_err(|_| {
        Error::new(
            src.span(),
            &format!("Cannot open file {:?}", target),
        )
    })
}

/// Handlers for CSS details (varies between backends)
pub trait StyleSheetConstructor {
    type ConfigValue: ParseStyleSheetValue;
    type PropertyValue: ParseStyleSheetValue;
    type FontFacePropertyValue: ParseStyleSheetValue;

    fn to_tokens(ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized;
}

/// Parse value positions
pub trait ParseStyleSheetValue {
    fn parse_value(
        name: &CssIdent,
        input: syn::parse::ParseStream,
    ) -> syn::Result<Self> where Self: Sized;
}

/// Display as CSS text
pub trait WriteCss {
    /// Write CSS text
    fn write_css(
        &self,
        sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error>;
}

/// Separator indicator for `WriteCss`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WriteCssSepCond {
    /// The CSS string ends with `CssIdent`
    ///
    /// It should not be followed by alphabets, digits, `-`, and `(`
    Ident,
    /// The CSS string ends with alphabets or digits (but not an ident nor number), `-` or `#`
    ///
    /// It should not be followed by alphabets, digits, and `-`
    NonIdentAlpha,
    /// The CSS string ends with `CssNumber`
    ///
    /// It should not be followed by alphabets, digits, `.`, `-`, and `%`
    Digit,
    /// The CSS string ends with `@`
    ///
    /// It should not be followed by alphabets and `-`
    At,
    /// The CSS string ends with `.` `+`
    ///
    /// It should not be followed by digits
    DotOrPlus,
    /// The CSS string ends with `$` `^` `~` `*`
    ///
    /// It should not be followed by `=`
    Equalable,
    /// The CSS string ends with `|`
    ///
    /// It should not be followed by `=` `|` `|=`
    Bar,
    /// The CSS string ends with `/`
    ///
    /// It should not be followed by `*` `*=`
    Slash,
    /// Always no separators needed
    Other,
}

/// A CSS property (name-value pair)
pub struct Property<V> {
    pub name: CssIdent,
    pub colon_token: token::Colon,
    pub value: V,
    pub semi_token: token::Semi,
}

impl<V: ParseStyleSheetValue> Parse for Property<V> {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let value = V::parse_value(&name, input)?;
        Ok(Self {
            name,
            colon_token,
            value,
            semi_token: input.parse()?,
        })
    }
}

impl<V: WriteCss> WriteCss for Property<V> {
    fn write_css(
        &self,
        sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error> {
        self.name.write_css(sc, false, w)?;
        write!(w, ":")?;
        let sc = self
            .value
            .write_css(WriteCssSepCond::Other, debug_mode, w)?;
        Ok(sc)
    }
}

pub enum PropertyOrSubRule<T: StyleSheetConstructor> {
    Property(Property<T::PropertyValue>),
    SubClass {
        and_token: token::Sub,
        ident: CssIdent,
        items: CssBrace<Repeat<PropertyOrSubRule<T>>>,
    },
    PseudoClass {
        colon_token: token::Colon,
        ident: CssIdent,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
    Media {
        at_keyword: CssAtKeyword,
        expr: Repeat<CssToken>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
    Supports {
        at_keyword: CssAtKeyword,
        expr: Repeat<CssToken>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
}

impl<T: StyleSheetConstructor> Parse for PropertyOrSubRule<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let item = if la.peek(Ident) || la.peek(token::Sub) {
            Self::Property(input.parse()?)
        } else if la.peek(token::And) {
            Self::SubClass {
                and_token: input.parse()?,
                ident: input.parse()?,
                items: input.parse()?,
            }
        } else if la.peek(token::Colon) {
            Self::PseudoClass {
                colon_token: input.parse()?,
                ident: input.parse()?,
                items: input.parse()?,
            }
        } else if la.peek(token::At) {
            let at_keyword: CssAtKeyword = input.parse()?;
            match at_keyword.formal_name.as_str() {
                "media" => Self::Media {
                    at_keyword,
                    expr: Repeat::parse_while(input, |input| {
                        !input.peek(token::Brace) && !input.peek(token::Semi)
                    })?,
                    items: input.parse()?,
                },
                "supports" => Self::Supports {
                    at_keyword,
                    expr: Repeat::parse_while(input, |input| {
                        !input.peek(token::Brace) && !input.peek(token::Semi)
                    })?,
                    items: input.parse()?,
                },
                _ => {
                    return Err(Error::new(at_keyword.span(), "Unknown at-keyword"));
                }
            }
        } else {
            return Err(la.error());
        };
        Ok(item)
    }
}

pub enum StyleSheetProcItem<T: StyleSheetConstructor> {
    Import {
        at_keyword: CssAtKeyword,
        src: CssString,
        semi_token: token::Semi,
    },
    Macro {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        mac: MacroDefinition,
    },
    Const {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        colon_token: CssColon,
        content: Repeat<CssToken>,
        semi_token: token::Semi,
    },
    Content(StyleSheetItem<T>),
}

pub enum StyleSheetItem<T: StyleSheetConstructor> {
    Config {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        colon_token: CssColon,
        value: T::ConfigValue,
        semi_token: token::Semi,
    },
    KeyFrames {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        brace_token: token::Brace,
        content: Repeat<(CssPercentage, CssBrace<Repeat<Property<T::PropertyValue>>>)>,
    },
    FontFaceRule {
        at_keyword: CssAtKeyword,
        items: CssBrace<Repeat<Property<T::FontFacePropertyValue>>>,
    },
    Rule {
        dot_token: token::Dot,
        ident: CssIdent,
        items: CssBrace<Repeat<PropertyOrSubRule<T>>>,
    },
}

impl<T: StyleSheetConstructor> Parse for StyleSheetProcItem<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let item = if la.peek(token::At) {
            let at_keyword: CssAtKeyword = input.parse()?;
            match at_keyword.formal_name.as_str() {
                "import" => {
                    let item = Self::Import {
                        at_keyword,
                        src: input.parse()?,
                        semi_token: input.parse()?,
                    };
                    item
                },
                "macro" => {
                    let item = Self::Macro {
                        at_keyword,
                        name: input.parse()?,
                        mac: input.parse()?,
                    };
                    item
                },
                "const" => {
                    let item = Self::Const {
                        at_keyword,
                        name: input.parse()?,
                        colon_token: input.parse()?,
                        content: input.parse()?,
                        semi_token: input.parse()?,
                    };
                    item
                },
                "config" => {
                    let name = input.parse()?;
                    let colon_token = input.parse()?;
                    let value = ParseStyleSheetValue::parse_value(&name, input)?;
                    Self::Content(StyleSheetItem::Config {
                        at_keyword,
                        name,
                        colon_token,
                        value,
                        semi_token: input.parse()?,
                    })
                },
                "key_frames" => {
                    let name = input.parse()?;
                    let content;
                    let brace_token = braced!(content in input);
                    let input = content;
                    let mut content = vec![];
                    while !content.is_empty() {
                        let la = input.lookahead1();
                        let percentage = if la.peek(Ident) {
                            let s: CssIdent = input.parse()?;
                            match s.formal_name.as_str() {
                                "from" => CssPercentage {
                                    span: s.span(),
                                    num: Number::Int(0),
                                },
                                "to" => CssPercentage {
                                    span: s.span(),
                                    num: Number::Int(100),
                                },
                                _ => return Err(Error::new(s.span(), "Illegal ident")),
                            }
                        } else if la.peek(Lit) {
                            input.parse()?
                        } else {
                            return Err(la.error());
                        };
                        let props = input.parse()?;
                        content.push((percentage, props));
                    }
                    Self::Content(StyleSheetItem::KeyFrames {
                        at_keyword,
                        name,
                        brace_token,
                        content: content.into(),
                    })
                }
                "font_face" => {
                    Self::Content(StyleSheetItem::FontFaceRule {
                        at_keyword,
                        items: input.parse()?,
                    })
                },
                _ => {
                    return Err(Error::new(at_keyword.span(), "Unknown at-keyword"));
                }
            }
        } else if la.peek(token::Dot) {
            let dot_token = input.parse()?;
            let ident = input.parse()?;
            let items = input.parse()?;
            Self::Content(StyleSheetItem::Rule {
                dot_token,
                ident,
                items,
            })
        } else {
            return Err(la.error());
        };
        Ok(item)
    }
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    pub items: Vec<StyleSheetItem<T>>,
    macros: FxHashMap<String, MacroDefinition>,
    consts: FxHashMap<String, Repeat<CssToken>>,
}

impl<T: StyleSheetConstructor> Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = vec![];
        let mut macros = FxHashMap::default();
        let mut consts = FxHashMap::default();
        let mut imports_ended = false;
        while !input.is_empty() {
            let proc_item: StyleSheetProcItem<T> = input.parse()?;
            if let StyleSheetProcItem::Import { at_keyword, .. } = &proc_item {
                if imports_ended {
                    return Err(Error::new(at_keyword.span(), "`@import` should be in the start of the segment"));
                }
            } else {
                imports_ended = true;
            };
            match proc_item {
                StyleSheetProcItem::Import { at_keyword, src, .. } => {
                    let content = get_import_content(&src)?;
                    let token_stream = proc_macro2::TokenStream::from_str(&content)?;
                    let mut ss = parse2::<StyleSheet<T>>(token_stream)
                        .map_err(|err| {
                            let original_span = err.span();
                            let start = original_span.start();
                            Error::new(
                                at_keyword.span(),
                                format_args!("When parsing {}:{}:{}: {}", src.value(), start.line, start.column, err),
                            )
                        })?;
                    macros.extend(ss.macros);
                    consts.extend(ss.consts);
                    items.append(&mut ss.items);
                }
                StyleSheetProcItem::Macro { name, mac, .. } => {
                    macros.insert(name.formal_name, mac);
                }
                StyleSheetProcItem::Const { name, content, .. } => {
                    consts.insert(name.formal_name, content);
                }
                StyleSheetProcItem::Content(x) => {
                    items.push(x);
                }
            }
        }
        Ok(Self {
            items,
            macros,
            consts,
        })
    }
}

impl<T: StyleSheetConstructor> ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        T::to_tokens(self, tokens)
    }
}
