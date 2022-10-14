use std::path::PathBuf;

use crate::{ParseError, css_token::*, StyleSheetVars, ScopeVars, ParseWithVars, write_css::*};

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

fn get_import_content(src: &CssString) -> Result<String, ParseError> {
    let p = src.value();
    if !p.starts_with("/") {
        return Err(ParseError::new(
            src.span,
            "Currently only paths started with `/` are supported (which means the path relative to crate `src` or MAOMI_CSS_IMPORT_DIR)",
        ));
    }
    let mut target = CSS_IMPORT_DIR.with(|import_dir| match import_dir {
        None => Err(ParseError::new(
            src.span,
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
        .map_err(|_| ParseError::new(src.span, &format!("cannot open file {:?}", target)))
}

/// Handlers for CSS details (varies between backends)
pub trait StyleSheetConstructor {
    type PropertyValue: ParseStyleSheetValue;
    type MediaCondValue: ParseStyleSheetValue;

    fn new() -> Self
    where
        Self: Sized;

    fn set_config(&mut self, name: &CssIdent, tokens: &mut CssTokenStream) -> Result<(), ParseError>;

    fn define_key_frames(
        &mut self,
        name: &CssVarRef,
        content: Vec<(
            CssPercentage,
            CssBrace<Repeat<Property<Self::PropertyValue>>>,
        )>,
    ) -> Result<Vec<CssToken>, ParseError>;

    fn to_tokens(&self, ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized;
}

/// Parse value positions
pub trait ParseStyleSheetValue {
    fn parse_value(name: &CssIdent, tokens: &mut CssTokenStream) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    ssc: T,
    pub items: Vec<StyleSheetItem<T>>,
    vars: StyleSheetVars,
}

pub enum StyleSheetItem<T: StyleSheetConstructor> {
    ConfigDefinition {
        name: CssIdent,
        refs: Vec<CssRef>,
    },
    // MacroDefinition {
    //     at_keyword: CssAtKeyword,
    //     name: CssIdent,
    //     refs: Vec<CssRef>,
    // },
    ConstDefinition {
        name: CssVarRef,
        refs: Vec<CssRef>,
    },
    KeyFramesDefinition {
        name: CssVarRef,
    },
    Rule {
        dot_token: CssDelim,
        ident: CssIdent,
        content: CssBrace<RuleContent<T>>,
    },
}

impl<T: StyleSheetConstructor> syn::parse::Parse for StyleSheet<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tokens = &mut CssTokenStream::parse(input)?;
        let vars = &mut StyleSheetVars::default();
        let scope = &mut ScopeVars::default();
        let ss = StyleSheet::parse_with_vars(tokens, vars, scope)
            .map_err(|x| x.into_syn_error())?;
        tokens.expect_ended().map_err(|x| x.into_syn_error())?;
        Ok(ss)
    }
}

impl<T: StyleSheetConstructor> quote::ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ssc.to_tokens(self, tokens)
    }
}

impl<T: StyleSheetConstructor> StyleSheet<T> {
    fn do_parsing(
        input: &mut CssTokenStream,
        ssc: &mut T,
        vars: &mut StyleSheetVars,
        items: &mut Vec<StyleSheetItem<T>>,
        in_imports: bool,
    ) -> Result<(), ParseError> {
        let scope = &mut ScopeVars::default();
        while !input.is_ended() {
            if let Ok(dot_token) = input.expect_delim(".") {
                if in_imports {
                    return Err(ParseError::new(input.span(), "common rule is not allowed in imports"));
                }
                let item = StyleSheetItem::Rule {
                    dot_token,
                    ident: input.expect_ident()?,
                    content: ParseWithVars::parse_with_vars(input, vars, scope)?,
                };
                items.push(item);
            } else if let Ok(at_keyword) = input.expect_at_keyword() {
                match at_keyword.formal_name.as_str() {
                    "import" => {
                        // IDEA considering a proper cache to avoid parsing during every import
                        let name = input.expect_string()?;
                        let content = get_import_content(&name)?;
                        let mut tokens = syn::parse_str::<CssTokenStream>(&content).map_err(|err| {
                            let original_span = err.span();
                            let start = original_span.start();
                            ParseError::new(
                                at_keyword.span,
                                format_args!(
                                    "when parsing {}:{}:{}: {}",
                                    name.value(),
                                    start.line,
                                    start.column,
                                    err
                                ),
                            )
                        })?;
                        StyleSheet::do_parsing(&mut tokens, ssc, vars, items, true)?;
                        tokens.expect_ended()?;
                    }
                    "config" => {
                        let name = input.expect_ident()?;
                        let _: CssColon = input.expect_colon()?;
                        let (tokens, refs) = input.resolve_until_semi(vars, scope)?;
                        let ts = &mut CssTokenStream::new(input.span(), tokens);
                        ssc.set_config(&name, ts)?;
                        ts.expect_ended()?;
                        input.expect_semi()?;
                        items.push(StyleSheetItem::ConfigDefinition { name, refs });
                    }
                    "macro" => {
                        todo!() // TODO
                    }
                    "const" => {
                        let name = input.expect_var_ref()?;
                        let _: CssColon = input.expect_colon()?;
                        let (tokens, refs) = input.resolve_until_semi(vars, scope)?;
                        if vars.consts.insert(name.clone(), crate::ConstOrKeyframe { tokens }).is_some() {
                            return Err(ParseError::new(name.ident.span, "redefined const or keyframes"));
                        }
                        input.expect_semi()?;
                        items.push(StyleSheetItem::ConstDefinition { name, refs });
                    }
                    "keyframes" => {
                        if in_imports {
                            return Err(ParseError::new(at_keyword.span, "`@keyframes` is not allowed in imports"));
                        }
                        let name = input.expect_var_ref()?;
                        let content = input.parse_brace(|input| {
                            let mut content = vec![];
                            while !input.is_ended() {
                                let percentage = if let Ok(ident) = input.expect_ident() {
                                    match ident.formal_name.as_str() {
                                        "from" => CssPercentage::new_int(ident.span, 0),
                                        "to" => CssPercentage::new_int(ident.span, 100),
                                        _ => return Err(ParseError::new(ident.span, "illegal ident")),
                                    }
                                } else if let Ok(n) = input.expect_percentage() {
                                    n
                                } else {
                                    return Err(ParseError::new(input.span(), "unknown at-keyword"));
                                };
                                content.push((percentage, ParseWithVars::parse_with_vars(input, vars, scope)?));
                            }
                            Ok(content)
                        })?;
                        let tokens = ssc.define_key_frames(&name, content.block)?;
                        if vars.consts.insert(name.clone(), crate::ConstOrKeyframe { tokens }).is_some() {
                            return Err(ParseError::new(name.ident.span, "redefined const or keyframes"));
                        }
                        items.push(StyleSheetItem::KeyFramesDefinition { name })
                    }
                    _ => {
                        return Err(ParseError::new(at_keyword.span, "unknown at-keyword"));
                    }
                }
            } else {
                return Err(ParseError::new(input.span(), "unexpected CSS token"));
            };
        }
        Ok(())
    }
}

impl<T: StyleSheetConstructor> ParseWithVars for StyleSheet<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        _vars: &mut StyleSheetVars,
        _scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let mut ssc = T::new();
        let mut vars = StyleSheetVars::default();
        let mut items = vec![];
        Self::do_parsing(input, &mut ssc, &mut vars, &mut items, false)?;
        Ok(Self { ssc, items, vars })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        // TODO
        for item in self.items.iter() {
            match item {
                StyleSheetItem::ConfigDefinition { refs, .. } => {
                    for r in refs {
                        f(r);
                    }
                }
                StyleSheetItem::ConstDefinition { refs, .. } => {
                    for r in refs {
                        f(r);
                    }
                }
                StyleSheetItem::KeyFramesDefinition { .. } => {}
                StyleSheetItem::Rule { content, .. } => {
                    content.block.for_each_ref(f);
                }
            }
        }
    }
}

pub struct RuleContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    pub at_blocks: Vec<AtBlock<T>>,
    pub pseudo_classes: Vec<PseudoClass<T>>,
    pub refs: Vec<CssRef>,
}

impl<T: StyleSheetConstructor> ParseWithVars for RuleContent<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let mut props = vec![];
        let mut at_blocks = vec![];
        let mut pseudo_classes = vec![];
        let mut refs = vec![];
        while !input.is_ended() {
            let next = input.peek()?.clone();
            match next {
                // TODO expend macro
                CssToken::Ident(_) => {
                    props.push(ParseWithVars::parse_with_vars(input, vars, scope)?);
                }
                CssToken::AtKeyword(_) => {
                    at_blocks.push(ParseWithVars::parse_with_vars(input, vars, scope)?);
                }
                CssToken::Colon(_) => {
                    pseudo_classes.push(ParseWithVars::parse_with_vars(input, vars, scope)?);
                }
                x => {
                    return Err(ParseError::new(x.span(), "unexpected token"));
                }
            }
        }
        Ok(Self {
            props,
            at_blocks,
            pseudo_classes,
            refs,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for r in self.refs.iter() {
            f(r);
        }
        for i in &self.at_blocks {
            i.for_each_ref(f);
        }
        for i in &self.pseudo_classes {
            i.for_each_ref(f);
        }
        for p in &self.props {
            p.for_each_ref(f);
        }
    }
}

/// A CSS property (name-value pair)
pub struct Property<V> {
    pub name: CssIdent,
    pub colon_token: CssColon,
    pub value: V,
    pub semi_token: CssSemi,
    pub refs: Vec<CssRef>,
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

impl<V: ParseStyleSheetValue> ParseWithVars for Property<V> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let name = input.expect_ident()?;
        let colon_token = input.expect_colon()?;
        let (value_tokens, refs) = input.resolve_until_semi(vars, scope)?;
        let ts = &mut CssTokenStream::new(input.span(), value_tokens);
        let value = V::parse_value(&name, ts)?;
        ts.expect_ended()?;
        let semi_token = input.expect_semi()?;
        Ok(Property { name, colon_token, value, semi_token, refs })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for r in &self.refs {
            f(r);
        }
    }
}

/// A CSS at-rule inside a class
pub enum AtBlock<T: StyleSheetConstructor> {
    Media {
        at_keyword: CssAtKeyword,
        expr: Vec<MediaQuery<T::MediaCondValue>>,
        content: CssBrace<AtBlockContent<T>>,
    },
    Supports {
        at_keyword: CssAtKeyword,
        expr: SupportsQuery<T::PropertyValue>,
        content: CssBrace<AtBlockContent<T>>,
    },
}

impl<T: StyleSheetConstructor> ParseWithVars for AtBlock<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let at_keyword = input.expect_at_keyword()?;
        let ret = if at_keyword.is("media") {
            let mut expr = vec![];
            loop {
                let q = ParseWithVars::parse_with_vars(input, vars, scope)?;
                expr.push(q);
                if input.expect_comma().is_err() {
                    break;
                }
            }
            Self::Media {
                at_keyword,
                expr,
                content: ParseWithVars::parse_with_vars(input, vars, scope)?,
            }
        } else if at_keyword.is("supports") {
            Self::Supports {
                at_keyword,
                expr: ParseWithVars::parse_with_vars(input, vars, scope)?,
                content: ParseWithVars::parse_with_vars(input, vars, scope)?,
            }
        } else {
            return Err(ParseError::new(at_keyword.span, "unknown at-keyword"));
        };
        Ok(ret)
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        match self {
            Self::Media { expr, content, .. } => {
                for e in expr {
                    e.for_each_ref(f);
                }
                content.for_each_ref(f);
            }
            Self::Supports { expr, content, .. } => {
                expr.for_each_ref(f);
                content.for_each_ref(f);
            }
        }
    }
}

pub struct AtBlockContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    pub refs: Vec<CssRef>,
}

impl<T: StyleSheetConstructor> ParseWithVars for AtBlockContent<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let content: RuleContent<T> = ParseWithVars::parse_with_vars(input, vars, scope)?;
        let RuleContent {
            props,
            at_blocks,
            pseudo_classes,
            refs,
        } = content;
        if let Some(x) = at_blocks.get(0) {
            return Err(ParseError::new(
                match x {
                    AtBlock::Media { at_keyword, .. } => at_keyword.span,
                    AtBlock::Supports { at_keyword, .. } => at_keyword.span,
                },
                "pseudo classes are not allowed inside pseudo classes",
            ));
        }
        if let Some(x) = pseudo_classes.get(0) {
            return Err(ParseError::new(
                x.colon_token.span,
                "pseudo classes are not allowed inside pseudo classes",
            ));
        }
        Ok(Self { props, refs })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for r in self.refs.iter() {
            f(r);
        }
        for item in &self.props {
            item.for_each_ref(f);
        }
    }
}

pub struct MediaQuery<V> {
    pub only: Option<CssIdent>,
    pub media_type: MediaType,
    pub cond_list: Vec<CssParen<MediaCond<V>>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    All,
    Screen,
    Print,
}

pub struct MediaCond<V> {
    pub not: Option<CssIdent>,
    pub name: CssIdent,
    pub colon_token: CssColon,
    pub cond: V,
    pub refs: Vec<CssRef>,
}

impl<V: ParseStyleSheetValue> ParseWithVars for MediaQuery<V> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let only = input.expect_keyword("only").ok();
        let (media_type, has_media_feature) = {
            let need_media_type = if only.is_some() {
                true
            } else if let CssToken::Ident(x) = input.peek()? {
                !x.is("not")
            } else {
                false
            };
            if need_media_type {
                let ident = input.expect_ident()?;
                let media_type = match ident.formal_name.as_str() {
                    "all" => MediaType::All,
                    "screen" => MediaType::Screen,
                    "print" => MediaType::Print,
                    _ => {
                        return Err(ParseError::new(ident.span, "unknown media type"));
                    }
                };
                let has_media_feature = input.expect_keyword("and").is_ok();
                (media_type, has_media_feature)
            } else {
                (MediaType::All, true)
            }
        };
        let mut cond_list = vec![];
        if has_media_feature {
            loop {
                let not = input.expect_keyword("not").ok();
                let cond = input.parse_paren(|input| {
                    let name = input.expect_ident()?;
                    let colon_token = input.expect_colon()?;
                    let (value_tokens, refs) = input.resolve_until_semi(vars, scope)?;
                    let ts = &mut CssTokenStream::new(input.span(), value_tokens);
                    let cond = V::parse_value(&name, ts)?;
                    ts.expect_ended()?;
                    Ok(MediaCond {
                        not,
                        name,
                        colon_token,
                        cond,
                        refs,
                    })
                })?;
                cond_list.push(cond);
                if let Ok(_) = input.expect_keyword("and") {
                    // empty
                } else {
                    break;
                }
            }
        }
        Ok(MediaQuery {
            only,
            media_type,
            cond_list,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for cond in &self.cond_list {
            for r in &cond.block.refs {
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
            item.block.not.write_css(cssw)?;
            cssw.write_paren_block(|cssw| {
                item.block.name.write_css(cssw)?;
                item.block.colon_token.write_css(cssw)?;
                item.block.cond.write_css(cssw)?;
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
    pub refs: Vec<CssRef>,
}

impl<V: ParseStyleSheetValue> ParseWithVars for SupportsQuery<V> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let ret = if let Ok(_) = input.expect_keyword("not") {
            let item: CssParen<SupportsQuery<V>> =
                ParseWithVars::parse_with_vars(input, vars, scope)?;
            if let Self::Sub(item) = item.block {
                Self::Not(item)
            } else {
                Self::Not(Box::new(item))
            }
        } else if let Ok(CssToken::Paren(_)) = input.peek() {
            let first: CssParen<SupportsQuery<V>> =
                ParseWithVars::parse_with_vars(input, vars, scope)?;
            let is_and = input.expect_keyword("and").is_ok();
            let is_or = input.expect_keyword("or").is_ok();
            if is_and || is_or {
                let mut list = vec![if let Self::Sub(item) = first.block {
                    *item
                } else {
                    first
                }];
                loop {
                    let item: CssParen<SupportsQuery<V>> =
                        ParseWithVars::parse_with_vars(input, vars, scope)?;
                    if let Self::Sub(item) = item.block {
                        list.push(*item);
                    } else {
                        list.push(item);
                    }
                    let next_is_and = input.expect_keyword("and").ok();
                    let next_is_or = input.expect_keyword("or").ok();
                    if next_is_and.is_some() || next_is_or.is_some() {
                        if is_and && next_is_or.is_some() {
                            return Err(ParseError::new(next_is_or.as_ref().unwrap().span, "cannot mix `and` and `or`"));
                        }
                        if is_or && next_is_and.is_some() {
                            return Err(ParseError::new(next_is_and.as_ref().unwrap().span, "cannot mix `and` and `or`"));
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
        } else if let Ok(name) = input.expect_ident() {
            let colon_token = input.expect_colon()?;
            let (value_tokens, refs) = input.resolve_until_semi(vars, scope)?;
            let ts = &mut CssTokenStream::new(input.span(), value_tokens);
            let value = V::parse_value(&name, ts)?;
            ts.expect_ended()?;
            Self::Cond(SupportsCond {
                name,
                colon_token,
                value,
                refs,
            })
        } else {
            return Err(ParseError::new(input.span(), "unexpected token"));
        };
        Ok(ret)
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        match self {
            Self::And(list) | Self::Or(list) => {
                for item in list {
                    item.block.for_each_ref(f);
                }
            }
            Self::Not(item) | Self::Sub(item) => {
                item.for_each_ref(f);
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

pub struct PseudoClass<T: StyleSheetConstructor> {
    pub colon_token: CssColon,
    pub pseudo: crate::pseudo::Pseudo,
    pub content: CssBrace<PseudoClassContent<T>>,
}

impl<T: StyleSheetConstructor> ParseWithVars for PseudoClass<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let colon_token = input.expect_colon()?;
        let pseudo = ParseWithVars::parse_with_vars(input, vars, scope)?;
        let content = ParseWithVars::parse_with_vars(input, vars, scope)?;
        Ok(Self { colon_token, pseudo, content })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        self.content.for_each_ref(f);
    }
}

pub struct PseudoClassContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    pub at_blocks: Vec<AtBlock<T>>,
    pub refs: Vec<CssRef>,
}

impl<T: StyleSheetConstructor> ParseWithVars for PseudoClassContent<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let content: RuleContent<T> = ParseWithVars::parse_with_vars(input, vars, scope)?;
        let RuleContent {
            props,
            at_blocks,
            pseudo_classes,
            refs,
        } = content;
        if let Some(x) = pseudo_classes.get(0) {
            return Err(ParseError::new(
                x.colon_token.span,
                "pseudo classes are not allowed inside pseudo classes",
            ));
        }
        Ok(Self { props, at_blocks, refs })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for r in self.refs.iter() {
            f(r);
        }
        for item in &self.props {
            item.for_each_ref(f);
        }
        for item in &self.at_blocks {
            item.for_each_ref(f);
        }
    }
}
