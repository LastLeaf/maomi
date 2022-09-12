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

mod kw {
    syn::custom_keyword!(only);
    syn::custom_keyword!(not);
    syn::custom_keyword!(and);
}

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
    type MediaCondValue: ParseStyleSheetValue;

    fn to_tokens(ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized;
}

/// Parse value positions
pub trait ParseStyleSheetValue {
    fn parse_value(
        name: &CssIdent,
        tokens: &[CssToken],
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

trait ParseWithVars: Sized {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self>;
}

/// A CSS property (name-value pair)
pub struct Property<V> {
    pub name: CssIdent,
    pub colon_token: token::Colon,
    pub value: V,
    pub semi_token: token::Semi,
}

impl<V: ParseStyleSheetValue> ParseWithVars for Property<V> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let tokens = ParseTokenUntilSemi::parse_with_vars(input, vars)?.get();
        let value = V::parse_value(&name, tokens.as_slice())?;
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
        expr: Vec<MediaQuery<T::MediaCondValue>>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
    Supports {
        at_keyword: CssAtKeyword,
        expr: SupportsCond<T::PropertyValue>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
}

pub struct MediaQuery<V> {
    pub has_only: bool,
    pub cond_list: Vec<MediaCond<V>>,
}

pub struct MediaCond<V> {
    pub has_not: bool,
    pub cond: V,
    // All,
    // Screen,
    // Print,
    // Width(CssDimension),
    // Height(CssDimension),
    // MinWidth(CssDimension),
    // MinHeight(CssDimension),
    // MaxWidth(CssDimension),
    // MaxHeight(CssDimension),
    // TODO add features in https://developer.mozilla.org/en-US/docs/Web/CSS/@media
}

impl<V: ParseStyleSheetValue> ParseWithVars for Vec<MediaQuery<V>> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let mut ret = vec![];
        loop {
            let has_only = if input.peek(kw::only) {
                input.parse::<kw::only>()?;
                true
            } else {
                false
            };
            let mut cond_list = vec![];
            loop {
                let has_not = if input.peek(kw::not) {
                    input.parse::<kw::not>()?;
                    true
                } else {
                    false
                };
                let la = input.lookahead1();
                let cond = if la.peek(Ident) {
                    let name: CssIdent = input.parse()?;
                    V::parse_value(&name, &[])?
                } else if la.peek(token::Paren) {
                    let content;
                    let _paren = parenthesized!(content in input);
                    let input = content;
                    let name: CssIdent = input.parse()?;
                    input.parse::<token::Colon>()?;
                    let tokens = Repeat::parse_with_vars(&input, vars)?;
                    V::parse_value(&name, tokens.as_slice())?
                } else {
                    return Err(la.error());
                };
                cond_list.push(MediaCond { has_not, cond });
                if !input.peek(kw::and) {
                    break;
                }
                input.parse::<kw::and>()?;
            }
            ret.push(MediaQuery { has_only, cond_list });
            if !input.peek(Token![,]) {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(ret)
    }
}

pub enum SupportsCond<V> {
    And(Vec<MediaCond<V>>),
    Or(V),
    Not(V),
    V,
}

impl<V: ParseStyleSheetValue> ParseWithVars for SupportsCond<V> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        todo!() // TODO
    }
}

impl<T: StyleSheetConstructor> ParseWithVars for PropertyOrSubRule<T> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let la = input.lookahead1();
        let item = if la.peek(Ident) || la.peek(token::Sub) {
            Self::Property(ParseWithVars::parse_with_vars(&input, vars)?)
        } else if la.peek(token::Sub) {
            Self::SubClass {
                ident: input.parse()?,
                items: ParseWithVars::parse_with_vars(input, vars)?,
            }
        } else if la.peek(token::Colon) {
            Self::PseudoClass {
                colon_token: input.parse()?,
                ident: input.parse()?,
                items: ParseWithVars::parse_with_vars(input, vars)?,
            }
        } else if la.peek(token::At) {
            let at_keyword: CssAtKeyword = input.parse()?;
            match at_keyword.formal_name.as_str() {
                "media" => Self::Media {
                    at_keyword,
                    expr: ParseWithVars::parse_with_vars(input, vars)?,
                    items: ParseWithVars::parse_with_vars(input, vars)?,
                },
                "supports" => Self::Supports {
                    at_keyword,
                    expr: ParseWithVars::parse_with_vars(input, vars)?,
                    items: ParseWithVars::parse_with_vars(input, vars)?,
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

struct StyleSheetImportItem {
    src: CssString,
    #[allow(dead_code)]
    semi_token: token::Semi,
}

impl Parse for StyleSheetImportItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            src: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

struct StyleSheetMacroItem {
    name: CssIdent,
    mac: MacroDefinition,
}

impl Parse for StyleSheetMacroItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            mac: input.parse()?,
        })
    }
}

struct StyleSheetConstItem {
    #[allow(dead_code)]
    dollar_token: Token![$],
    name: CssIdent,
    #[allow(dead_code)]
    colon_token: CssColon,
    content: Repeat<CssToken>,
    #[allow(dead_code)]
    semi_token: token::Semi,
}

impl ParseWithVars for StyleSheetConstItem {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        Ok(Self {
            dollar_token: input.parse()?,
            name: input.parse()?,
            colon_token: input.parse()?,
            content: ParseWithVars::parse_with_vars(input, vars)?,
            semi_token: input.parse()?,
        })
    }
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
        // IDEA support use with @media and @supports
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

pub struct StyleSheet<T: StyleSheetConstructor> {
    pub items: Vec<StyleSheetItem<T>>,
    vars: StyleSheetVars,
}

struct StyleSheetVars {
    macros: FxHashMap<String, MacroDefinition>,
    consts: FxHashMap<String, Repeat<CssToken>>,
}

impl<T: StyleSheetConstructor> Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = vec![];
        let mut vars = StyleSheetVars {
            macros: FxHashMap::default(),
            consts: FxHashMap::default(),
        };

        // parse items
        while !input.is_empty() {
            let la = input.lookahead1();
            if la.peek(token::At) {
                let at_keyword: CssAtKeyword = input.parse()?;
                match at_keyword.formal_name.as_str() {
                    "import" => {
                        // IDEA considering a proper cache to avoid parsing during every import
                        let item: StyleSheetImportItem = input.parse()?;
                        let content = get_import_content(&item.src)?;
                        let token_stream = proc_macro2::TokenStream::from_str(&content)?;
                        let mut ss = parse2::<StyleSheet<T>>(token_stream)
                            .map_err(|err| {
                                let original_span = err.span();
                                let start = original_span.start();
                                Error::new(
                                    at_keyword.span(),
                                    format_args!("When parsing {}:{}:{}: {}", item.src.value(), start.line, start.column, err),
                                )
                            })?;
                        vars.macros.extend(ss.vars.macros);
                        vars.consts.extend(ss.vars.consts);
                        items.append(&mut ss.items);
                    },
                    "macro" => {
                        let item: StyleSheetMacroItem = input.parse()?;
                        vars.macros.insert(item.name.formal_name, item.mac);
                    },
                    "const" => {
                        let item = StyleSheetConstItem::parse_with_vars(&input, &vars)?;
                        vars.consts.insert(item.name.formal_name, item.content);
                    },
                    "config" => {
                        let name = input.parse()?;
                        let colon_token = input.parse()?;
                        let tokens = ParseTokenUntilSemi::parse_with_vars(input, &vars)?.get();
                        let value = ParseStyleSheetValue::parse_value(&name, tokens.as_slice())?;
                        items.push(StyleSheetItem::Config {
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
                            let props = ParseWithVars::parse_with_vars(&input, &vars)?;
                            content.push((percentage, props));
                        }
                        items.push(StyleSheetItem::KeyFrames {
                            at_keyword,
                            name,
                            brace_token,
                            content: content.into(),
                        })
                    }
                    "font_face" => {
                        items.push(StyleSheetItem::FontFaceRule {
                            at_keyword,
                            items: ParseWithVars::parse_with_vars(&input, &vars)?,
                        })
                    },
                    _ => {
                        return Err(Error::new(at_keyword.span(), "Unknown at-keyword"));
                    }
                }
            } else if la.peek(token::Dot) {
                let dot_token = input.parse()?;
                let ident = input.parse()?;
                items.push(StyleSheetItem::Rule {
                    dot_token,
                    ident,
                    items: ParseWithVars::parse_with_vars(&input, &vars)?,
                })
            } else {
                return Err(la.error());
            };
        }

        Ok(Self {
            items,
            vars,
        })
    }
}

impl<T: StyleSheetConstructor> ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        T::to_tokens(self, tokens)
    }
}
