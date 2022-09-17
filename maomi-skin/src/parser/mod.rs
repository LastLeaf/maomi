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
pub mod write_css;
use write_css::*;

mod kw {
    syn::custom_keyword!(only);
    syn::custom_keyword!(not);
    syn::custom_keyword!(and);
}

// TODO consider a proper way to handle global styling (font, css-reset, etc.)

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
        tokens: &mut CssTokenStream,
    ) -> syn::Result<Self> where Self: Sized;
}

pub trait ParseWithVars: Sized {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self>;
    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent));
}

/// A CSS property (name-value pair)
pub struct Property<V> {
    pub name: CssIdent,
    pub colon_token: CssColon,
    pub value: V,
    pub semi_token: CssSemi,
    pub refs: Vec<CssIdent>,
}

impl<V: ParseStyleSheetValue> Property<V> {
    fn parse_property_with_name(name: CssIdent, input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let colon_token = input.parse()?;
        let tokens = ParseTokenUntilSemi::parse_with_vars(input, vars)?;
        let (tokens, refs) = tokens.get();
        let mut stream = CssTokenStream::new(input.span(), tokens);
        let value = V::parse_value(&name, &mut stream)?;
        stream.expect_ended()?;
        Ok(Self {
            name,
            colon_token,
            value,
            semi_token: input.parse()?,
            refs,
        })
    }
}

impl<V: ParseStyleSheetValue> ParseWithVars for Property<V> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let name = input.parse()?;
        Self::parse_property_with_name(name, input, vars)
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for r in &self.refs {
            f(r);
        }
    }
}

impl<V: WriteCss> WriteCss for Property<V> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::fmt::Result {
        self.name.write_css(cssw)?;
        self.colon_token.write_css(cssw)?;
        self.value.write_css(cssw)?;
        self.semi_token.write_css(cssw)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MediaQuery<V> {
    pub only: Option<CssIdent>,
    pub media_type: MediaType,
    pub cond_list: Vec<MediaCond<V>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    All,
    Screen,
    Print,
}

#[derive(Debug, Clone)]
pub struct MediaCond<V> {
    pub not: Option<CssIdent>,
    pub name: CssIdent,
    pub colon_token: CssColon,
    pub cond: V,
    pub refs: Vec<CssIdent>,
}

impl<V: ParseStyleSheetValue> ParseWithVars for MediaQuery<V> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let only = if input.peek(kw::only) {
            Some(input.parse()?)
        } else {
            None
        };
        let (media_type, has_media_feature) = if only.is_some() || input.peek(Ident) {
            let ident: CssIdent = input.parse()?;
            let media_type = match ident.formal_name.as_str() {
                "all" => MediaType::All,
                "screen" => MediaType::Screen,
                "print" => MediaType::Print,
                _ => {
                    return Err(Error::new(ident.span(), "Unknown media type"));
                }
            };
            let has_media_feature = input.peek(kw::and);
            if has_media_feature {
                input.parse::<CssIdent>()?;
            }
            (media_type, has_media_feature)
        } else {
            (MediaType::All, true)
        };
        let mut cond_list = vec![];
        if has_media_feature {
            loop {
                let not = if input.peek(kw::not) {
                    Some(input.parse()?)
                } else {
                    None
                };
                let content;
                let _paren = parenthesized!(content in input);
                let input = content;
                let name: CssIdent = input.parse()?;
                let colon_token = input.parse()?;
                let (tokens, refs) = Repeat::parse_with_vars(&input, vars)?.get();
                let mut stream = CssTokenStream::new(input.span(), tokens);
                let cond = V::parse_value(&name, &mut stream)?;
                stream.expect_ended()?;
                cond_list.push(MediaCond { not, name, colon_token, cond, refs });
                if !input.peek(kw::and) {
                    break;
                }
                input.parse::<kw::and>()?;
            }
        }
        Ok(MediaQuery { only, media_type, cond_list })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for cond in &self.cond_list {
            for r in &cond.refs {
                f(r);
            }
        }
    }
}

impl<V: WriteCss> WriteCss for MediaQuery<V> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::fmt::Result {
        self.only.write_css(cssw)?;
        let mut need_and = match self.media_type {
            MediaType::All => {
                if self.only.is_some() {
                    cssw.write_ident("all", true)?;
                    true
                } else {
                    false
                }
            }
            MediaType::Print => {
                cssw.write_ident("print", true)?;
                true
            }
            MediaType::Screen => {
                cssw.write_ident("screen", true)?;
                true
            }
        };
        for item in self.cond_list.iter() {
            if need_and {
                cssw.write_ident("and", true)?;
            } else {
                need_and = true;
            }
            item.not.write_css(cssw)?;
            cssw.write_paren_block(|cssw| {
                item.name.write_css(cssw)?;
                item.colon_token.write_css(cssw)?;
                item.cond.write_css(cssw)?;
                Ok(())
            })?;
        }
        Ok(())
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

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        todo!() // TODO
    }
}

impl<V: WriteCss> WriteCss for SupportsCond<V> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::fmt::Result {
        todo!() // TODO
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
    content: ParseTokenUntilSemi,
    #[allow(dead_code)]
    semi_token: token::Semi,
}

impl ParseWithVars for StyleSheetConstItem {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        Ok(Self {
            dollar_token: input.parse()?,
            name: input.parse()?,
            colon_token: input.parse()?,
            content: ParseTokenUntilSemi::parse_with_vars(input, vars)?,
            semi_token: input.parse()?,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.content.for_each_ref(f)
    }
}

pub struct RuleContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    pub at_blocks: Vec<AtBlock<T>>,
    pub pseudo_classes: Vec<PseudoClass<T>>,
    pub sub_classes: Vec<SubClass<T>>,
}

pub struct PseudoClassContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    pub at_blocks: Vec<AtBlock<T>>,
}

pub enum AtBlock<T: StyleSheetConstructor> {
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

pub struct PseudoClass<T: StyleSheetConstructor> {
    pub colon_token: CssColon,
    pub ident: CssIdent,
    pub content: PseudoClassContent<T>,
}

pub struct SubClass<T: StyleSheetConstructor> {
    pub partial_ident: CssIdent,
    pub content: RuleContent<T>,
}

impl<T: StyleSheetConstructor> ParseWithVars for PseudoClassContent<T> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let content: RuleContent<T> = ParseWithVars::parse_with_vars(input, vars)?;
        let RuleContent { props, at_blocks, pseudo_classes, sub_classes } = content;
        if let Some(x) = pseudo_classes.get(0) {
            return Err(Error::new(x.ident.span, "Pseudo classes are not allowed inside pseudo classes"));
        }
        if let Some(x) = sub_classes.get(0) {
            return Err(Error::new(x.partial_ident.span, "Sub classes are not allowed inside pseudo classes"));
        }
        Ok(Self { props, at_blocks })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for item in &self.props {
            item.for_each_ref(f);
        }
        for item in &self.at_blocks {
            match item {
                AtBlock::Media { expr, items, .. } => {
                    for item in expr {
                        item.for_each_ref(f);
                    }
                    items.for_each_ref(f);
                }
                AtBlock::Supports { expr, items, .. } => {
                    expr.for_each_ref(f);
                    items.for_each_ref(f);
                }
            }
        }
    }
}

impl<T: StyleSheetConstructor> ParseWithVars for RuleContent<T> {
    fn parse_with_vars(input: ParseStream, vars: &StyleSheetVars) -> Result<Self> {
        let mut props = vec![];
        let mut at_blocks = vec![];
        let mut pseudo_classes = vec![];
        let mut sub_classes = vec![];
        while !input.is_empty() {
            let la = input.lookahead1();
            if la.peek(Ident) || la.peek(token::Sub) {
                let ident: CssIdent = input.parse()?;
                let la = input.lookahead1();
                if la.peek(Token![!]) {
                    todo!() // TODO apply macro
                } else if la.peek(token::Colon) {
                    props.push(Property::parse_property_with_name(ident, input, vars)?);
                } else if la.peek(token::Brace) {
                    if ident.formal_name.chars().nth(0) != Some('_') {
                        return Err(Error::new(ident.span, "Sub class names must be started with `_` or `-`"));
                    }
                    sub_classes.push(SubClass {
                        partial_ident: ident,
                        content: ParseWithVars::parse_with_vars(input, vars)?,
                    });
                } else {
                    return Err(la.error());
                }
            } else if la.peek(token::At) {
                let at_keyword: CssAtKeyword = input.parse()?;
                match at_keyword.formal_name.as_str() {
                    "media" => {
                        let mut expr = vec![];
                        loop {
                            expr.push(ParseWithVars::parse_with_vars(input, vars)?);
                            if !input.peek(Token![,]) {
                                break;
                            }
                            input.parse::<Token![,]>()?;
                        }
                        at_blocks.push(AtBlock::Media {
                            at_keyword,
                            expr,
                            items: ParseWithVars::parse_with_vars(input, vars)?,
                        });
                    },
                    "supports" => {
                        at_blocks.push(AtBlock::Supports {
                            at_keyword,
                            expr: ParseWithVars::parse_with_vars(input, vars)?,
                            items: ParseWithVars::parse_with_vars(input, vars)?,
                        })
                    },
                    _ => {
                        return Err(Error::new(at_keyword.span(), "Unknown at-keyword"));
                    }
                }
            } else if la.peek(token::Colon) {
                let colon_token = input.parse()?;
                let ident = input.parse()?;
                pseudo_classes.push(PseudoClass {
                    colon_token,
                    ident,
                    content: ParseWithVars::parse_with_vars(input, vars)?,
                });
            } else {
                return Err(la.error());
            };
        }
        Ok(Self {
            props,
            at_blocks,
            pseudo_classes,
            sub_classes,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for item in &self.props {
            item.for_each_ref(f);
        }
        for item in &self.at_blocks {
            match item {
                AtBlock::Media { expr, items, .. } => {
                    for item in expr {
                        item.for_each_ref(f);
                    }
                    items.for_each_ref(f);
                }
                AtBlock::Supports { expr, items, .. } => {
                    expr.for_each_ref(f);
                    items.for_each_ref(f);
                }
            }
        }
        for item in &self.pseudo_classes {
            item.content.for_each_ref(f);
        }
        for item in &self.sub_classes {
            item.content.for_each_ref(f);
        }
    }
}

pub enum StyleSheetItem<T: StyleSheetConstructor> {
    Config {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        colon_token: CssColon,
        value: T::ConfigValue,
        semi_token: token::Semi,
        refs: Vec<CssIdent>,
    },
    MacroDefinition {
        at_keyword: CssAtKeyword,
        name: CssIdent,
    },
    ConstDefinition {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        refs: Vec<CssIdent>,
    },
    KeyFrames {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        brace_token: token::Brace,
        content: Vec<(CssPercentage, CssBrace<Repeat<Property<T::PropertyValue>>>)>,
    },
    FontFaceRule {
        at_keyword: CssAtKeyword,
        items: CssBrace<Repeat<Property<T::FontFacePropertyValue>>>,
    },
    Rule {
        dot_token: token::Dot,
        ident: CssIdent,
        content: CssBrace<RuleContent<T>>,
    },
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    pub items: Vec<StyleSheetItem<T>>,
    vars: StyleSheetVars,
}

pub struct StyleSheetVars {
    macros: FxHashMap<String, MacroDefinition>,
    consts: FxHashMap<String, Vec<CssToken>>,
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
                        // TODO add proper refs
                        vars.macros.insert(item.name.formal_name.clone(), item.mac);
                        items.push(StyleSheetItem::MacroDefinition { at_keyword, name: item.name })
                    },
                    "const" => {
                        let item = StyleSheetConstItem::parse_with_vars(&input, &vars)?;
                        let (tokens, refs) = item.content.get();
                        vars.consts.insert(item.name.formal_name.clone(), tokens);
                        items.push(StyleSheetItem::ConstDefinition { at_keyword, name: item.name, refs })
                    },
                    "config" => {
                        let name = input.parse()?;
                        let colon_token = input.parse()?;
                        let (tokens, refs) = ParseTokenUntilSemi::parse_with_vars(input, &vars)?.get();
                        let mut stream = CssTokenStream::new(input.span(), tokens);
                        let value = ParseStyleSheetValue::parse_value(&name, &mut stream)?;
                        stream.expect_ended()?;
                        items.push(StyleSheetItem::Config {
                            at_keyword,
                            name,
                            colon_token,
                            value,
                            semi_token: input.parse()?,
                            refs,
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
                            content,
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
                    content: ParseWithVars::parse_with_vars(input, &vars)?,
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
