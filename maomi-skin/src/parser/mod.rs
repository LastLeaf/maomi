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
pub mod pseudo;

mod kw {
    syn::custom_keyword!(only);
    syn::custom_keyword!(not);
    syn::custom_keyword!(and);
    syn::custom_keyword!(or);
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
        ));
    }
    let mut target = CSS_IMPORT_DIR.with(|import_dir| match import_dir {
        None => Err(Error::new(
            src.span(),
            "no MAOMI_CSS_IMPORT_DIR or CARGO_MANIFEST_DIR environment variables provided",
        )),
        Some(s) => Ok(s.clone()),
    })?;
    for slice in p[1..].split('/') {
        match slice {
            "." => {}
            ".." => {
                target.pop();
            }
            x => {
                target.push(x);
            }
        }
    }
    std::fs::read_to_string(&target)
        .map_err(|_| Error::new(src.span(), &format!("cannot open file {:?}", target)))
}

/// Handlers for CSS details (varies between backends)
pub trait StyleSheetConstructor {
    type PropertyValue: ParseStyleSheetValue;
    type FontFacePropertyValue: ParseStyleSheetValue;
    type MediaCondValue: ParseStyleSheetValue;

    fn new() -> Self
    where
        Self: Sized;

    fn set_config(&mut self, name: &CssIdent, tokens: &mut CssTokenStream) -> syn::Result<()>;

    fn define_key_frames(
        &mut self,
        name: &CssIdent,
        content: &Vec<(
            CssPercentage,
            CssBrace<Repeat<Property<Self::PropertyValue>>>,
        )>,
    ) -> CssIdent;

    fn to_tokens(&self, ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized;
}

/// Parse value positions
pub trait ParseStyleSheetValue {
    fn parse_value(name: &CssIdent, tokens: &mut CssTokenStream) -> syn::Result<Self>
    where
        Self: Sized;
}

pub trait ParseWithVars: Sized {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self>;
    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent));
}

pub struct ScopeVars<'a> {
    macro_pat_vars: Option<&'a mut mac::MacroPatVars>,
}

impl<'a> ScopeVars<'a> {
    fn new() -> Self {
        Self {
            macro_pat_vars: None,
        }
    }
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
    fn parse_property_with_name(
        name: CssIdent,
        input: ParseStream,
        vars: &StyleSheetVars,
    ) -> Result<Self> {
        let colon_token = input.parse()?;
        let tokens = ParseTokenUntilSemi::parse_with_vars(input, vars, &mut ScopeVars::new())?;
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
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        _scope: &mut ScopeVars,
    ) -> Result<Self> {
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
    fn write_css<W: std::fmt::Write>(&self, cssw: &mut CssWriter<W>) -> std::fmt::Result {
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
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let only = if input.peek(kw::only) {
            Some(input.parse()?)
        } else {
            None
        };
        let (media_type, has_media_feature) =
            if only.is_some() || (!input.peek(kw::not) && input.peek(Ident)) {
                let ident: CssIdent = input.parse()?;
                let media_type = match ident.formal_name.as_str() {
                    "all" => MediaType::All,
                    "screen" => MediaType::Screen,
                    "print" => MediaType::Print,
                    _ => {
                        return Err(Error::new(ident.span(), "unknown media type"));
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
                {
                    let input = content;
                    let name: CssIdent = input.parse()?;
                    let colon_token = input.parse()?;
                    let (tokens, refs) = Repeat::parse_with_vars(&input, vars, scope)?.get();
                    let mut stream = CssTokenStream::new(input.span(), tokens);
                    let cond = V::parse_value(&name, &mut stream)?;
                    stream.expect_ended()?;
                    cond_list.push(MediaCond {
                        not,
                        name,
                        colon_token,
                        cond,
                        refs,
                    });
                }
                if !input.peek(kw::and) {
                    break;
                }
                input.parse::<kw::and>()?;
            }
        }
        Ok(MediaQuery {
            only,
            media_type,
            cond_list,
        })
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
    fn write_css<W: std::fmt::Write>(&self, cssw: &mut CssWriter<W>) -> std::fmt::Result {
        self.only.write_css(cssw)?;
        let mut need_and = match self.media_type {
            MediaType::All => {
                if self.only.is_some() || self.cond_list.is_empty() {
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

pub enum SupportsQuery<V> {
    Cond(SupportsCond<V>),
    And(Vec<CssParen<SupportsQuery<V>>>),
    Or(Vec<CssParen<SupportsQuery<V>>>),
    Not(Box<CssParen<SupportsQuery<V>>>),
    Sub(Box<CssParen<SupportsQuery<V>>>),
}

pub struct SupportsCond<V> {
    pub name: CssIdent,
    pub colon_token: CssColon,
    pub value: V,
    pub refs: Vec<CssIdent>,
}

impl<V: ParseStyleSheetValue> ParseWithVars for SupportsQuery<V> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let la = input.lookahead1();
        let ret = if la.peek(kw::not) {
            let _: kw::not = input.parse()?;
            let item: CssParen<SupportsQuery<V>> =
                ParseWithVars::parse_with_vars(input, vars, scope)?;
            if let Self::Sub(item) = item.block {
                Self::Not(item)
            } else {
                Self::Not(Box::new(item))
            }
        } else if la.peek(token::Paren) {
            let first: CssParen<SupportsQuery<V>> =
                ParseWithVars::parse_with_vars(input, vars, scope)?;
            let la = input.lookahead1();
            let is_and = la.peek(kw::and);
            let is_or = la.peek(kw::or);
            if is_and || is_or {
                let mut list = vec![if let Self::Sub(item) = first.block {
                    *item
                } else {
                    first
                }];
                loop {
                    let _: Ident = input.parse()?;
                    let item: CssParen<SupportsQuery<V>> =
                        ParseWithVars::parse_with_vars(input, vars, scope)?;
                    if let Self::Sub(item) = item.block {
                        list.push(*item);
                    } else {
                        list.push(item);
                    }
                    let next_is_and = input.peek(kw::and);
                    let next_is_or = input.peek(kw::and);
                    if next_is_and || next_is_or {
                        if is_and && next_is_or || is_or && next_is_and {
                            return Err(input.error("cannot mix `and` and `or`"));
                        }
                    } else {
                        break;
                    }
                }
                if is_and {
                    Self::And(list)
                } else {
                    Self::Or(list)
                }
            } else {
                if let Self::Sub(item) = first.block {
                    Self::Sub(item)
                } else {
                    Self::Sub(Box::new(first))
                }
            }
        } else if la.peek(Ident) || la.peek(token::Sub) {
            let name = input.parse()?;
            let colon_token = input.parse()?;
            let (tokens, refs) = Repeat::parse_with_vars(&input, vars, scope)?.get();
            let mut stream = CssTokenStream::new(input.span(), tokens);
            let value = V::parse_value(&name, &mut stream)?;
            stream.expect_ended()?;
            Self::Cond(SupportsCond {
                name,
                colon_token,
                value,
                refs,
            })
        } else {
            return Err(la.error());
        };
        Ok(ret)
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        match self {
            Self::And(list) | Self::Or(list) => {
                for item in list {
                    item.block.for_each_ref(f);
                }
            }
            Self::Not(item) | Self::Sub(item) => {
                item.block.for_each_ref(f);
            }
            Self::Cond(cond) => {
                for r in cond.refs.iter() {
                    f(r);
                }
            }
        }
    }
}

impl<V: WriteCss> WriteCss for SupportsQuery<V> {
    fn write_css<W: std::fmt::Write>(&self, cssw: &mut CssWriter<W>) -> std::fmt::Result {
        match self {
            Self::Cond(cond) => {
                cond.name.write_css(cssw)?;
                cond.colon_token.write_css(cssw)?;
                cond.value.write_css(cssw)?;
            }
            Self::And(list) => {
                for (index, item) in list.iter().enumerate() {
                    if index > 0 {
                        cssw.write_ident("and", true)?;
                    }
                    item.write_css(cssw)?;
                }
            }
            Self::Or(list) => {
                for (index, item) in list.iter().enumerate() {
                    if index > 0 {
                        cssw.write_ident("or", true)?;
                    }
                    item.write_css(cssw)?;
                }
            }
            Self::Not(item) => {
                cssw.write_ident("not", true)?;
                item.write_css(cssw)?;
            }
            Self::Sub(item) => {
                item.write_css(cssw)?;
            }
        }
        Ok(())
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
    mac: CssBrace<MacroDefinition>,
}

impl ParseWithVars for StyleSheetMacroItem {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            mac: ParseWithVars::parse_with_vars(input, vars, scope)?,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.mac.for_each_ref(f);
    }
}

struct StyleSheetConstItem {
    #[allow(dead_code)]
    dollar_token: token::Dollar,
    name: CssIdent,
    #[allow(dead_code)]
    colon_token: CssColon,
    content: ParseTokenUntilSemi,
    #[allow(dead_code)]
    semi_token: token::Semi,
}

impl ParseWithVars for StyleSheetConstItem {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        Ok(Self {
            dollar_token: input.parse()?,
            name: input.parse()?,
            colon_token: input.parse()?,
            content: ParseTokenUntilSemi::parse_with_vars(input, vars, scope)?,
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
    pub refs: Vec<CssIdent>,
}

pub struct PseudoClassContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    pub at_blocks: Vec<AtBlock<T>>,
    pub refs: Vec<CssIdent>,
}

pub enum AtBlock<T: StyleSheetConstructor> {
    Media {
        at_keyword: CssAtKeyword,
        expr: Vec<MediaQuery<T::MediaCondValue>>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
        refs: Vec<CssIdent>,
    },
    Supports {
        at_keyword: CssAtKeyword,
        expr: SupportsQuery<T::PropertyValue>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
        refs: Vec<CssIdent>,
    },
}

pub struct PseudoClass<T: StyleSheetConstructor> {
    pub colon_token: CssColon,
    pub pseudo: pseudo::Pseudo,
    pub content: CssBrace<PseudoClassContent<T>>,
}

pub struct SubClass<T: StyleSheetConstructor> {
    pub partial_ident: CssIdent,
    pub content: CssBrace<RuleContent<T>>,
}

impl<T: StyleSheetConstructor> ParseWithVars for PseudoClassContent<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let content: RuleContent<T> = ParseWithVars::parse_with_vars(input, vars, scope)?;
        let RuleContent {
            props,
            at_blocks,
            pseudo_classes,
            sub_classes,
            refs,
        } = content;
        if let Some(x) = pseudo_classes.get(0) {
            return Err(Error::new(
                x.colon_token.span,
                "pseudo classes are not allowed inside pseudo classes",
            ));
        }
        if let Some(x) = sub_classes.get(0) {
            return Err(Error::new(
                x.partial_ident.span,
                "sub classes are not allowed inside pseudo classes",
            ));
        }
        Ok(Self { props, at_blocks, refs })
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
        self.refs.iter().for_each(f);
    }
}

impl<T: StyleSheetConstructor> ParseWithVars for RuleContent<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let mut props = vec![];
        let mut at_blocks = vec![];
        let mut pseudo_classes = vec![];
        let mut sub_classes = vec![];
        let mut refs = vec![];
        while !input.is_empty() {
            let la = input.lookahead1();
            if la.peek(Ident) || la.peek(token::Sub) {
                let ident: CssIdent = input.parse()?;
                let la = input.lookahead1();
                if la.peek(Token![!]) {
                    let call = ParseWithVars::parse_with_vars(input, vars, scope)?;
                    let mut tokens = vec![];
                    mac::MacroArgsToken::write_macro_ref(
                        &mut tokens,
                        &mut refs,
                        &ident,
                        &call,
                        vars,
                    )?;
                    let mut stream = CssTokenStream::new(ident.span, tokens);
                    while stream.peek().is_ok() {
                        // TODO support mixin blocks (need overall CssTokenStream parsing?)
                        let name = stream.expect_ident()?;
                        let colon_token = stream.expect_colon()?;
                        let mut value_tokens: Vec<CssToken> = vec![];
                        loop {
                            match stream.peek() {
                                Err(_) => {
                                    let span = value_tokens
                                        .last()
                                        .map(|x| x.span())
                                        .unwrap_or(colon_token.span);
                                    return Err(Error::new(span, "expected `;`"));
                                }
                                Ok(CssToken::Semi(_)) => break,
                                Ok(_) => {}
                            }
                            value_tokens.push(stream.next()?);
                        }
                        let first = value_tokens.first().ok_or_else(|| {
                            Error::new(colon_token.span, "expected property value")
                        })?;
                        let mut sub_stream = CssTokenStream::new(first.span(), value_tokens);
                        let value = T::PropertyValue::parse_value(&name, &mut sub_stream)?;
                        let semi_token = stream.expect_semi()?;
                        props.push(Property {
                            name,
                            colon_token,
                            value,
                            semi_token,
                            refs: Vec::with_capacity(0),
                        });
                    }
                    if !call.is_braced() {
                        input.parse::<token::Semi>()?;
                    }
                } else if la.peek(token::Colon) {
                    props.push(Property::parse_property_with_name(ident, input, vars)?);
                } else if la.peek(token::Brace) {
                    if ident.formal_name.chars().nth(0) != Some('_') {
                        return Err(Error::new(
                            ident.span,
                            "sub class names must be started with `_` or `-`",
                        ));
                    }
                    sub_classes.push(SubClass {
                        partial_ident: ident,
                        content: ParseWithVars::parse_with_vars(input, vars, scope)?,
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
                            expr.push(ParseWithVars::parse_with_vars(input, vars, scope)?);
                            if !input.peek(Token![,]) {
                                break;
                            }
                            input.parse::<Token![,]>()?;
                        }
                        // TODO support macro
                        at_blocks.push(AtBlock::Media {
                            at_keyword,
                            expr,
                            items: ParseWithVars::parse_with_vars(input, vars, scope)?,
                            refs: Vec::with_capacity(0),
                        });
                    }
                    "supports" => {
                        let la = input.lookahead1();
                        if la.peek(kw::not) || la.peek(token::Paren) {
                            // empty
                        } else {
                            return Err(la.error());
                        }
                        // TODO support macro
                        at_blocks.push(AtBlock::Supports {
                            at_keyword,
                            expr: ParseWithVars::parse_with_vars(input, vars, scope)?,
                            items: ParseWithVars::parse_with_vars(input, vars, scope)?,
                            refs: Vec::with_capacity(0),
                        });
                    }
                    _ => {
                        return Err(Error::new(at_keyword.span(), "unknown at-keyword"));
                    }
                }
            } else if la.peek(token::Colon) {
                let colon_token = input.parse()?;
                let pseudo = input.parse()?;
                pseudo_classes.push(PseudoClass {
                    colon_token,
                    pseudo,
                    content: ParseWithVars::parse_with_vars(input, vars, scope)?,
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
            refs,
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
    MacroDefinition {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        refs: Vec<CssIdent>,
    },
    ConstDefinition {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        refs: Vec<CssIdent>,
    },
    KeyFramesDefinition {
        at_keyword: CssAtKeyword,
        dollar_token: token::Dollar,
        name: CssIdent,
        brace_token: token::Brace,
        content: Vec<(CssPercentage, CssBrace<Repeat<Property<T::PropertyValue>>>)>,
        def: CssIdent,
    },
    Rule {
        dot_token: token::Dot,
        ident: CssIdent,
        content: CssBrace<RuleContent<T>>,
    },
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    ssc: T,
    pub items: Vec<StyleSheetItem<T>>,
    vars: StyleSheetVars,
}

pub struct StyleSheetVars {
    macros: FxHashMap<String, MacroDefinition>,
    consts: FxHashMap<String, Vec<CssToken>>,
    keyframes: FxHashMap<String, CssIdent>,
}

impl<T: StyleSheetConstructor> Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ssc = T::new();
        let mut items = vec![];
        let mut vars = StyleSheetVars {
            macros: FxHashMap::default(),
            consts: FxHashMap::default(),
            keyframes: FxHashMap::default(),
        };

        // parse items
        while !input.is_empty() {
            let vars = &mut vars;
            let la = input.lookahead1();
            if la.peek(token::At) {
                let at_keyword: CssAtKeyword = input.parse()?;
                match at_keyword.formal_name.as_str() {
                    "import" => {
                        // IDEA considering a proper cache to avoid parsing during every import
                        let item: StyleSheetImportItem = input.parse()?;
                        let content = get_import_content(&item.src)?;
                        let token_stream = proc_macro2::TokenStream::from_str(&content)?;
                        let mut ss = parse2::<StyleSheet<T>>(token_stream).map_err(|err| {
                            let original_span = err.span();
                            let start = original_span.start();
                            Error::new(
                                at_keyword.span(),
                                format_args!(
                                    "when parsing {}:{}:{}: {}",
                                    item.src.value(),
                                    start.line,
                                    start.column,
                                    err
                                ),
                            )
                        })?;
                        vars.macros.extend(ss.vars.macros);
                        vars.consts.extend(ss.vars.consts);
                        items.append(&mut ss.items);
                    }
                    "macro" => {
                        let item = StyleSheetMacroItem::parse_with_vars(
                            input,
                            vars,
                            &mut ScopeVars::new(),
                        )?;
                        let mut refs = vec![];
                        item.for_each_ref(&mut |x| refs.push(x.clone()));
                        if vars
                            .macros
                            .insert(item.name.formal_name.clone(), item.mac.block)
                            .is_some()
                        {
                            return Err(Error::new(
                                item.name.span,
                                format!(
                                    "macro named `{}` has already defined",
                                    item.name.formal_name
                                ),
                            ));
                        }
                        items.push(StyleSheetItem::MacroDefinition {
                            at_keyword,
                            name: item.name,
                            refs,
                        })
                    }
                    "const" => {
                        let item = StyleSheetConstItem::parse_with_vars(
                            &input,
                            vars,
                            &mut ScopeVars::new(),
                        )?;
                        let (tokens, refs) = item.content.get();
                        if vars
                            .consts
                            .insert(item.name.formal_name.clone(), tokens)
                            .is_some()
                        {
                            return Err(Error::new(
                                item.name.span,
                                format!(
                                    "const named `{}` has already defined",
                                    item.name.formal_name
                                ),
                            ));
                        }
                        items.push(StyleSheetItem::ConstDefinition {
                            at_keyword,
                            name: item.name,
                            refs,
                        })
                    }
                    "config" => {
                        let name: CssIdent = input.parse()?;
                        let _: CssColon = input.parse()?;
                        let (tokens, _refs) = ParseTokenUntilSemi::parse_with_vars(
                            input,
                            vars,
                            &mut ScopeVars::new(),
                        )?
                        .get();
                        let mut stream = CssTokenStream::new(input.span(), tokens);
                        ssc.set_config(&name, &mut stream)?;
                        stream.expect_ended()?;
                        let _: CssSemi = input.parse()?;
                    }
                    "keyframes" => {
                        let dollar_token = input.parse()?;
                        let name: CssIdent = input.parse()?;
                        let content;
                        let brace_token = braced!(content in input);
                        let input = content;
                        let mut content = vec![];
                        while !input.is_empty() {
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
                                    _ => return Err(Error::new(s.span(), "illegal ident")),
                                }
                            } else if la.peek(Lit) {
                                input.parse()?
                            } else {
                                return Err(la.error());
                            };
                            let props = ParseWithVars::parse_with_vars(
                                &input,
                                vars,
                                &mut ScopeVars::new(),
                            )?;
                            content.push((percentage, props));
                        }
                        let def = ssc.define_key_frames(&name, &content);
                        vars.keyframes.insert(name.formal_name.clone(), def.clone());
                        items.push(StyleSheetItem::KeyFramesDefinition {
                            at_keyword,
                            dollar_token,
                            name,
                            brace_token,
                            content,
                            def,
                        })
                    }
                    _ => {
                        return Err(Error::new(at_keyword.span(), "unknown at-keyword"));
                    }
                }
            } else if la.peek(token::Dot) {
                let dot_token = input.parse()?;
                let ident = input.parse()?;
                items.push(StyleSheetItem::Rule {
                    dot_token,
                    ident,
                    content: ParseWithVars::parse_with_vars(input, vars, &mut ScopeVars::new())?,
                })
            } else {
                return Err(la.error());
            };
        }

        Ok(Self { ssc, items, vars })
    }
}

impl<T: StyleSheetConstructor> ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ssc.to_tokens(self, tokens)
    }
}
