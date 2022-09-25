use proc_macro2::Span;
use std::iter::Peekable;
use std::num::NonZeroU64;
use syn::spanned::Spanned;
use syn::*;
use syn::{ext::IdentExt, parse::*};

use super::mac::MacroArgsToken;
use super::{
    write_css::CssWriter, ParseWithVars, ScopeVars, StyleSheetVars, WriteCss, WriteCssSepCond,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(x) => write!(f, "{}", x),
            Self::Float(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CssIdent {
    pub span: Span,
    pub formal_name: String,
}

impl CssIdent {
    pub fn css_name(&self) -> String {
        self.formal_name.replace('_', "-")
    }
}

impl Spanned for CssIdent {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut formal_name = String::new();
        let mut span = None;
        loop {
            let la = input.lookahead1();
            let is_sub = if la.peek(token::Sub) {
                let t: token::Sub = input.parse()?;
                if span.is_none() {
                    span = Some(t.span())
                }
                formal_name.push('_');
                true
            } else if la.peek(Ident::peek_any) {
                let t: Ident = Ident::parse_any(input)?;
                if span.is_none() {
                    span = Some(t.span())
                }
                let s: &str = &t.to_string();
                formal_name += s
                    .strip_prefix("r#")
                    .and_then(|x| x.strip_suffix("#"))
                    .unwrap_or(s);
                false
            } else {
                return Err(la.error());
            };
            if is_sub || input.peek(token::Sub) {
                // empty
            } else {
                break;
            }
        }
        Ok(Self {
            formal_name,
            span: span.unwrap(),
        })
    }
}

impl WriteCss for CssIdent {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_ident(&self.css_name(), true)
    }
}

#[derive(Clone)]
pub struct CssAtKeyword {
    pub span: Span,
    pub at_token: token::At,
    pub formal_name: String,
}

impl CssAtKeyword {
    pub fn css_name(&self) -> String {
        self.formal_name.replace('_', "-")
    }
}

impl Spanned for CssAtKeyword {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssAtKeyword {
    fn parse(input: ParseStream) -> Result<Self> {
        let at_token: token::At = input.parse()?;
        let formal_name = CssIdent::parse(input)?.formal_name;
        Ok(Self {
            span: at_token.span(),
            at_token,
            formal_name,
        })
    }
}

impl WriteCss for CssAtKeyword {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.custom_write(|w, sc, debug_mode| {
            if debug_mode {
                match sc {
                    WriteCssSepCond::BlockStart | WriteCssSepCond::Whitespace => {}
                    _ => {
                        write!(w, " ")?;
                    }
                }
            }
            write!(w, "@{}", self.css_name())?;
            Ok(WriteCssSepCond::NonIdentAlpha)
        })
    }
}

#[derive(Clone)]
pub struct CssString {
    pub s: LitStr,
}

impl CssString {
    pub fn value(&self) -> String {
        self.s.value()
    }
}

impl Spanned for CssString {
    fn span(&self) -> Span {
        self.s.span()
    }
}

impl Parse for CssString {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self { s: input.parse()? })
    }
}

impl WriteCss for CssString {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.custom_write(|w, sc, debug_mode| {
            if debug_mode {
                match sc {
                    WriteCssSepCond::BlockStart | WriteCssSepCond::Whitespace => {}
                    _ => {
                        write!(w, " ")?;
                    }
                }
            }
            write!(w, "{:?}", self.s.value())?;
            Ok(WriteCssSepCond::Other)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssColon {
    pub span: Span,
}

impl Spanned for CssColon {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssColon {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Colon = input.parse()?;
        Ok(Self { span: x.span() })
    }
}

impl WriteCss for CssColon {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        let CssWriter {
            ref mut w,
            ref mut sc,
            ..
        } = cssw;
        write!(w, ":")?;
        *sc = WriteCssSepCond::Other;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CssSemi {
    pub span: Span,
}

impl Spanned for CssSemi {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssSemi {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Semi = input.parse()?;
        Ok(Self { span: x.span() })
    }
}

impl WriteCss for CssSemi {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        let CssWriter {
            ref mut w,
            ref mut sc,
            ..
        } = cssw;
        write!(w, ";")?;
        *sc = WriteCssSepCond::Other;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CssComma {
    pub span: Span,
}

impl Spanned for CssComma {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssComma {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Comma = input.parse()?;
        Ok(Self { span: x.span() })
    }
}

impl WriteCss for CssComma {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        let CssWriter {
            ref mut w,
            ref mut sc,
            ..
        } = cssw;
        write!(w, ",")?;
        *sc = WriteCssSepCond::Other;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CssDelim {
    pub span: Span,
    pub s: &'static str,
}

impl Spanned for CssDelim {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssDelim {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        macro_rules! parse_delim {
            ($x:tt) => {
                if la.peek(Token![$x]) {
                    let x: Token![$x] = input.parse()?;
                    let span = x.span();
                    return Ok(Self {
                        span,
                        s: stringify!($x),
                    });
                }
            };
        }
        parse_delim!(+);
        parse_delim!(+=);
        parse_delim!(&);
        parse_delim!(&&);
        parse_delim!(&=);
        parse_delim!(@);
        parse_delim!(!);
        parse_delim!(^);
        parse_delim!(^=);
        parse_delim!(/);
        parse_delim!(/=);
        parse_delim!($);
        parse_delim!(.);
        parse_delim!(..);
        parse_delim!(...);
        parse_delim!(..=);
        parse_delim!(=);
        parse_delim!(==);
        parse_delim!(=>);
        parse_delim!(>=);
        parse_delim!(>);
        parse_delim!(<-);
        parse_delim!(<=);
        parse_delim!(<);
        parse_delim!(*=);
        parse_delim!(!=);
        parse_delim!(|);
        parse_delim!(|=);
        parse_delim!(||);
        parse_delim!(#);
        parse_delim!(?);
        parse_delim!(->);
        parse_delim!(%);
        parse_delim!(%=);
        parse_delim!(<<);
        parse_delim!(<<=);
        parse_delim!(>>);
        parse_delim!(>>=);
        parse_delim!(*);
        parse_delim!(-);
        parse_delim!(-=);
        parse_delim!(~);
        Err(la.error())
    }
}

impl WriteCss for CssDelim {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_delim(self.s, true)
    }
}

#[derive(Clone)]
pub struct CssNumber {
    pub span: Span,
    pub num: Number,
}

impl CssNumber {
    pub fn integer(&self) -> Option<i64> {
        match self.num {
            Number::Int(x) => Some(x),
            Number::Float(_) => None,
        }
    }

    pub fn positive_integer(&self) -> Option<NonZeroU64> {
        self.integer().and_then(|x| {
            if x <= 0 {
                None
            } else {
                Some(unsafe { NonZeroU64::new_unchecked(x as u64) })
            }
        })
    }
}

impl Spanned for CssNumber {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssNumber {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        if la.peek(LitInt) {
            let n: LitInt = input.parse()?;
            if n.suffix().len() > 0 {
                return Err(Error::new(n.span(), "Illegal number suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
            });
        }
        if la.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            if n.suffix().len() > 0 {
                return Err(Error::new(n.span(), "Illegal number suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
            });
        }
        Err(la.error())
    }
}

impl WriteCss for CssNumber {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.custom_write(|w, sc, debug_mode| {
            if debug_mode {
                match sc {
                    WriteCssSepCond::BlockStart | WriteCssSepCond::Whitespace => {}
                    _ => {
                        write!(w, " ")?;
                    }
                }
            } else {
                match sc {
                    WriteCssSepCond::Ident
                    | WriteCssSepCond::NonIdentAlpha
                    | WriteCssSepCond::Digit
                    | WriteCssSepCond::DotOrPlus => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            write!(w, "{}", self.num)?;
            Ok(WriteCssSepCond::Digit)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssPercentage {
    pub span: Span,
    pub num: Number,
}

impl Spanned for CssPercentage {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssPercentage {
    fn parse(input: ParseStream) -> Result<Self> {
        let CssNumber { span, num } = CssNumber::parse(input)?;
        let _: Token![%] = input.parse()?;
        Ok(Self { span, num })
    }
}

impl WriteCss for CssPercentage {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.custom_write(|w, sc, debug_mode| {
            if debug_mode {
                match sc {
                    WriteCssSepCond::BlockStart | WriteCssSepCond::Whitespace => {}
                    _ => {
                        write!(w, " ")?;
                    }
                }
            } else {
                match sc {
                    WriteCssSepCond::Ident
                    | WriteCssSepCond::NonIdentAlpha
                    | WriteCssSepCond::Digit
                    | WriteCssSepCond::DotOrPlus => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            write!(w, "{}%", self.num)?;
            Ok(WriteCssSepCond::Other)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssDimension {
    pub span: Span,
    pub num: Number,
    pub unit: String,
}

impl Spanned for CssDimension {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssDimension {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        if la.peek(LitInt) {
            let n: LitInt = input.parse()?;
            if n.suffix().len() == 0 {
                return Err(Error::new(n.span(), "Expect dimension suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
                unit: n.suffix().to_string(),
            });
        }
        if la.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            if n.suffix().len() == 0 {
                return Err(Error::new(n.span(), "Expect dimension suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
                unit: n.suffix().to_string(),
            });
        }
        Err(la.error())
    }
}

impl WriteCss for CssDimension {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.custom_write(|w, sc, debug_mode| {
            if debug_mode {
                match sc {
                    WriteCssSepCond::BlockStart | WriteCssSepCond::Whitespace => {}
                    _ => {
                        write!(w, " ")?;
                    }
                }
            } else {
                match sc {
                    WriteCssSepCond::Ident
                    | WriteCssSepCond::NonIdentAlpha
                    | WriteCssSepCond::Digit
                    | WriteCssSepCond::DotOrPlus => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            write!(w, "{}{}", self.num, self.unit)?;
            Ok(WriteCssSepCond::NonIdentAlpha)
        })
    }
}

#[derive(Clone)]
pub struct CssFunction<T> {
    pub span: Span,
    pub formal_name: String,
    pub paren_token: token::Paren,
    pub block: T,
}

impl<T> CssFunction<T> {
    pub fn css_name(&self) -> String {
        self.formal_name.replace('_', "-")
    }
}

impl<T> Spanned for CssFunction<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: ParseWithVars> ParseWithVars for CssFunction<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let CssIdent { span, formal_name } = CssIdent::parse(input)?;
        let content;
        let paren_token = parenthesized!(content in input);
        let block = T::parse_with_vars(&content, vars, scope)?;
        Ok(Self {
            span,
            formal_name,
            paren_token,
            block,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.block.for_each_ref(f)
    }
}

impl<T: WriteCss> WriteCss for CssFunction<T> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_function_block(&self.css_name(), |cssw| self.block.write_css(cssw))
    }
}

#[derive(Clone)]
pub struct CssParen<T> {
    pub paren_token: token::Paren,
    pub block: T,
}

impl<T> Spanned for CssParen<T> {
    fn span(&self) -> Span {
        self.paren_token.span
    }
}

impl<T: Parse> Parse for CssParen<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let block = content.parse()?;
        Ok(Self { paren_token, block })
    }
}

impl<T: ParseWithVars> ParseWithVars for CssParen<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let block = T::parse_with_vars(&content, vars, scope)?;
        Ok(Self { paren_token, block })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.block.for_each_ref(f)
    }
}

impl<T: WriteCss> WriteCss for CssParen<T> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_paren_block(|cssw| self.block.write_css(cssw))
    }
}

#[derive(Clone)]
pub struct CssBracket<T> {
    pub bracket_token: token::Bracket,
    pub block: T,
}

impl<T> Spanned for CssBracket<T> {
    fn span(&self) -> Span {
        self.bracket_token.span
    }
}

impl<T: Parse> Parse for CssBracket<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let bracket_token = bracketed!(content in input);
        let block = content.parse()?;
        Ok(Self {
            bracket_token,
            block,
        })
    }
}

impl<T: ParseWithVars> ParseWithVars for CssBracket<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let content;
        let bracket_token = bracketed!(content in input);
        let block = T::parse_with_vars(&content, vars, scope)?;
        Ok(Self {
            bracket_token,
            block,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.block.for_each_ref(f)
    }
}

impl<T: WriteCss> WriteCss for CssBracket<T> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_bracket_block(|cssw| self.block.write_css(cssw))
    }
}

#[derive(Clone)]
pub struct CssBrace<T> {
    pub brace_token: token::Brace,
    pub block: T,
}

impl<T> Spanned for CssBrace<T> {
    fn span(&self) -> Span {
        self.brace_token.span
    }
}

impl<T: Parse> Parse for CssBrace<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);
        let block = content.parse()?;
        Ok(Self { brace_token, block })
    }
}

impl<T: ParseWithVars> ParseWithVars for CssBrace<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);
        let block = T::parse_with_vars(&content, vars, scope)?;
        Ok(Self { brace_token, block })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.block.for_each_ref(f)
    }
}

impl<T: WriteCss> WriteCss for CssBrace<T> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_brace_block(|cssw| self.block.write_css(cssw))
    }
}

#[derive(Debug, Clone)]
pub struct Repeat<T> {
    inner: Vec<T>,
    refs: Vec<CssIdent>,
}

impl<T> Repeat<T> {
    pub fn get(self) -> (Vec<T>, Vec<CssIdent>) {
        (self.inner, self.refs)
    }

    pub fn into_vec(self) -> Vec<T> {
        self.inner
    }

    pub fn from_vec(v: Vec<T>) -> Self {
        Self {
            inner: v,
            refs: Vec::with_capacity(0),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.inner.iter()
    }
}

impl<T: Parse> Parse for Repeat<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inner = vec![];
        let refs = Vec::with_capacity(0);
        while !input.is_empty() {
            inner.push(input.parse()?);
        }
        Ok(Self { inner, refs })
    }
}

impl ParseWithVars for Repeat<CssToken> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let mut inner = vec![];
        let mut refs = Vec::with_capacity(0);
        while !input.is_empty() {
            parse_token(&mut inner, &mut refs, input, vars, scope)?;
        }
        Ok(Self { inner, refs })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for r in &self.refs {
            f(r)
        }
    }
}

impl<T: ParseWithVars> ParseWithVars for Repeat<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let mut inner = vec![];
        let refs = Vec::with_capacity(0);
        while !input.is_empty() {
            let item = T::parse_with_vars(input, vars, scope)?;
            inner.push(item);
        }
        Ok(Self { inner, refs })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for r in &self.inner {
            r.for_each_ref(f);
        }
    }
}

impl<T: WriteCss> WriteCss for Repeat<T> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        for item in self.inner.iter() {
            item.write_css(cssw)?;
        }
        Ok(())
    }
}

impl<'a, T> IntoIterator for &'a Repeat<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[derive(Clone)]
pub enum CssToken {
    Ident(CssIdent),
    AtKeyword(CssAtKeyword),
    String(CssString),
    Colon(CssColon),
    Semi(CssSemi),
    Comma(CssComma),
    Delim(CssDelim),
    Number(CssNumber),
    Percentage(CssPercentage),
    Dimension(CssDimension),
    Function(CssFunction<Repeat<CssToken>>),
    Paren(CssParen<Repeat<CssToken>>),
    Bracket(CssBracket<Repeat<CssToken>>),
    Brace(CssBrace<Repeat<CssToken>>),
}

impl CssToken {
    pub(crate) fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Ident(x) => {
                if let Self::Ident(y) = other {
                    x.formal_name == y.formal_name
                } else {
                    false
                }
            }
            Self::AtKeyword(x) => {
                if let Self::AtKeyword(y) = other {
                    x.formal_name == y.formal_name
                } else {
                    false
                }
            }
            Self::String(x) => {
                if let Self::String(y) = other {
                    x.s.value() == y.s.value()
                } else {
                    false
                }
            }
            Self::Colon(_) => {
                if let Self::Colon(_) = other {
                    true
                } else {
                    false
                }
            }
            Self::Semi(_) => {
                if let Self::Semi(_) = other {
                    true
                } else {
                    false
                }
            }
            Self::Comma(_) => {
                if let Self::Comma(_) = other {
                    true
                } else {
                    false
                }
            }
            Self::Delim(x) => {
                if let Self::Delim(y) = other {
                    x.s == y.s
                } else {
                    false
                }
            }
            Self::Number(x) => {
                if let Self::Number(y) = other {
                    x.num == y.num
                } else {
                    false
                }
            }
            Self::Percentage(x) => {
                if let Self::Percentage(y) = other {
                    x.num == y.num
                } else {
                    false
                }
            }
            Self::Dimension(x) => {
                if let Self::Dimension(y) = other {
                    x.num == y.num && x.unit == y.unit
                } else {
                    false
                }
            }
            Self::Function(x) => {
                if let Self::Function(y) = other {
                    let mut eq = x.formal_name == y.formal_name;
                    if eq {
                        for (x, y) in x.block.iter().zip(y.block.iter()) {
                            if !x.content_eq(y) {
                                eq = false;
                                break;
                            }
                        }
                    }
                    eq
                } else {
                    false
                }
            }
            Self::Paren(x) => {
                if let Self::Paren(y) = other {
                    let mut eq = true;
                    for (x, y) in x.block.iter().zip(y.block.iter()) {
                        if !x.content_eq(y) {
                            eq = false;
                            break;
                        }
                    }
                    eq
                } else {
                    false
                }
            }
            Self::Bracket(x) => {
                if let Self::Bracket(y) = other {
                    let mut eq = true;
                    for (x, y) in x.block.iter().zip(y.block.iter()) {
                        if !x.content_eq(y) {
                            eq = false;
                            break;
                        }
                    }
                    eq
                } else {
                    false
                }
            }
            Self::Brace(x) => {
                if let Self::Brace(y) = other {
                    let mut eq = true;
                    for (x, y) in x.block.iter().zip(y.block.iter()) {
                        if !x.content_eq(y) {
                            eq = false;
                            break;
                        }
                    }
                    eq
                } else {
                    false
                }
            }
        }
    }
}

impl Spanned for CssToken {
    fn span(&self) -> Span {
        match self {
            Self::Ident(x) => x.span(),
            Self::AtKeyword(x) => x.span(),
            Self::String(x) => x.span(),
            Self::Colon(x) => x.span(),
            Self::Semi(x) => x.span(),
            Self::Comma(x) => x.span(),
            Self::Delim(x) => x.span(),
            Self::Number(x) => x.span(),
            Self::Percentage(x) => x.span(),
            Self::Dimension(x) => x.span(),
            Self::Function(x) => x.span(),
            Self::Paren(x) => x.span(),
            Self::Bracket(x) => x.span(),
            Self::Brace(x) => x.span(),
        }
    }
}

impl Parse for CssToken {
    fn parse(input: ParseStream) -> Result<Self> {
        let t = if input.peek(token::At) {
            CssToken::AtKeyword(input.parse()?)
        } else if input.peek(LitStr) {
            CssToken::String(input.parse()?)
        } else if input.peek(token::Colon) {
            CssToken::Colon(input.parse()?)
        } else if input.peek(token::Semi) {
            CssToken::Semi(input.parse()?)
        } else if input.peek(token::Comma) {
            CssToken::Comma(input.parse()?)
        } else if input.peek(token::Paren) {
            CssToken::Paren(input.parse()?)
        } else if input.peek(token::Bracket) {
            CssToken::Bracket(input.parse()?)
        } else if input.peek(token::Brace) {
            CssToken::Brace(input.parse()?)
        } else if input.peek(LitInt) {
            let n: LitInt = input.parse()?;
            let span = n.span();
            let num = Number::Int(n.base10_parse()?);
            if n.suffix().len() == 0 {
                if input.peek(Token![%]) {
                    let _: Token![%] = input.parse()?;
                    CssToken::Percentage(CssPercentage { span, num })
                } else {
                    CssToken::Number(CssNumber { span, num })
                }
            } else {
                CssToken::Dimension(CssDimension {
                    span,
                    num,
                    unit: n.suffix().to_string(),
                })
            }
        } else if input.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            let span = n.span();
            let num = Number::Float(n.base10_parse()?);
            if n.suffix().len() == 0 {
                if input.peek(Token![%]) {
                    let _: Token![%] = input.parse()?;
                    CssToken::Percentage(CssPercentage { span, num })
                } else {
                    CssToken::Number(CssNumber { span, num })
                }
            } else {
                CssToken::Dimension(CssDimension {
                    span,
                    num,
                    unit: n.suffix().to_string(),
                })
            }
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(token::Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let block = content.parse()?;
                CssToken::Function(CssFunction {
                    span: x.span,
                    formal_name: x.formal_name,
                    paren_token,
                    block,
                })
            } else {
                CssToken::Ident(x)
            }
        } else if let Ok(x) = input.parse() {
            CssToken::Delim(x)
        } else {
            return Err(input.error("Illegal CSS token"));
        };
        Ok(t)
    }
}

impl WriteCss for CssToken {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        let sc = match self {
            Self::Ident(x) => x.write_css(cssw)?,
            Self::AtKeyword(x) => x.write_css(cssw)?,
            Self::String(x) => x.write_css(cssw)?,
            Self::Colon(x) => x.write_css(cssw)?,
            Self::Semi(x) => x.write_css(cssw)?,
            Self::Comma(x) => x.write_css(cssw)?,
            Self::Delim(x) => x.write_css(cssw)?,
            Self::Number(x) => x.write_css(cssw)?,
            Self::Percentage(x) => x.write_css(cssw)?,
            Self::Dimension(x) => x.write_css(cssw)?,
            Self::Function(x) => x.write_css(cssw)?,
            Self::Paren(x) => x.write_css(cssw)?,
            Self::Bracket(x) => x.write_css(cssw)?,
            Self::Brace(x) => x.write_css(cssw)?,
        };
        Ok(sc)
    }
}

pub(crate) fn parse_token(
    ret: &mut Vec<CssToken>,
    refs: &mut Vec<CssIdent>,
    input: ParseStream,
    vars: &StyleSheetVars,
    scope: &mut ScopeVars,
) -> Result<()> {
    MacroArgsToken::parse_input_and_write(ret, refs, input, vars, scope)
}

pub(crate) struct ParseTokenUntilSemi {
    inner: Vec<CssToken>,
    refs: Vec<CssIdent>,
}

impl ParseTokenUntilSemi {
    pub(crate) fn get(self) -> (Vec<CssToken>, Vec<CssIdent>) {
        (self.inner, self.refs)
    }
}

impl ParseWithVars for ParseTokenUntilSemi {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let mut inner = vec![];
        let mut refs = Vec::with_capacity(0);
        while !input.is_empty() {
            if input.peek(token::Semi) {
                break;
            }
            parse_token(&mut inner, &mut refs, input, vars, scope)?;
        }
        Ok(Self { inner, refs })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for r in &self.refs {
            f(r);
        }
    }
}

pub struct CssTokenStream {
    span: Span,
    inner: Peekable<std::vec::IntoIter<CssToken>>,
}

impl CssTokenStream {
    #[inline]
    pub fn new(span: Span, v: Vec<CssToken>) -> Self {
        Self {
            span,
            inner: v.into_iter().peekable(),
        }
    }

    #[inline]
    pub fn expect_ended(&mut self) -> Result<()> {
        let peek = self.inner.peek();
        if let Some(x) = peek {
            Err(Error::new(x.span(), "unexpected token"))
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn span(&mut self) -> Span {
        self.inner.peek().map(|x| x.span()).unwrap_or(self.span)
    }

    #[inline]
    pub fn next(&mut self) -> Result<CssToken> {
        self.inner
            .next()
            .ok_or_else(|| Error::new(self.span, "unexpected end of the value"))
    }

    #[inline]
    pub fn peek(&mut self) -> Result<&CssToken> {
        self.inner
            .peek()
            .ok_or_else(|| Error::new(self.span, "unexpected end of the value"))
    }

    #[inline]
    pub fn expect_ident(&mut self) -> Result<CssIdent> {
        let x = self.next()?;
        if let CssToken::Ident(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected identifier"))
        }
    }

    #[inline]
    pub fn expect_at_keyword(&mut self) -> Result<CssAtKeyword> {
        let x = self.next()?;
        if let CssToken::AtKeyword(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected at-keyword"))
        }
    }

    #[inline]
    pub fn expect_string(&mut self) -> Result<CssString> {
        let x = self.next()?;
        if let CssToken::String(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected quoted string"))
        }
    }

    #[inline]
    pub fn expect_colon(&mut self) -> Result<CssColon> {
        let x = self.next()?;
        if let CssToken::Colon(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected `:`"))
        }
    }

    #[inline]
    pub fn expect_semi(&mut self) -> Result<CssSemi> {
        let x = self.next()?;
        if let CssToken::Semi(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected `;`"))
        }
    }

    #[inline]
    pub fn expect_delim(&mut self, delim: &'static str) -> Result<CssDelim> {
        let x = self.next()?;
        let span = x.span();
        if let CssToken::Delim(x) = x {
            if x.s == delim {
                return Ok(x);
            }
        }
        Err(Error::new(span, format_args!("expected `{}`", delim)))
    }

    #[inline]
    pub fn expect_number(&mut self) -> Result<CssNumber> {
        let x = self.next()?;
        if let CssToken::Number(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected number"))
        }
    }

    #[inline]
    pub fn expect_percentage(&mut self) -> Result<CssPercentage> {
        let x = self.next()?;
        if let CssToken::Percentage(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected percentage"))
        }
    }

    #[inline]
    pub fn expect_dimension(&mut self) -> Result<CssDimension> {
        let x = self.next()?;
        if let CssToken::Dimension(x) = x {
            Ok(x)
        } else {
            Err(Error::new(x.span(), "expected dimension"))
        }
    }

    #[inline]
    pub fn parse_function<R>(&mut self, f: impl FnOnce(&str, Self) -> Result<R>) -> Result<R> {
        let x = self.next()?;
        if let CssToken::Function(x) = x {
            Ok(f(
                x.formal_name.as_str(),
                Self::new(x.span(), x.block.into_vec()),
            )?)
        } else {
            Err(Error::new(x.span(), "expected function"))
        }
    }

    #[inline]
    pub fn parse_paren<R>(&mut self, f: impl FnOnce(Self) -> Result<R>) -> Result<R> {
        let x = self.next()?;
        if let CssToken::Paren(x) = x {
            Ok(f(Self::new(x.span(), x.block.into_vec()))?)
        } else {
            Err(Error::new(x.span(), "expected `(...)`"))
        }
    }

    #[inline]
    pub fn parse_bracket<R>(&mut self, f: impl FnOnce(Self) -> Result<R>) -> Result<R> {
        let x = self.next()?;
        if let CssToken::Bracket(x) = x {
            Ok(f(Self::new(x.span(), x.block.into_vec()))?)
        } else {
            Err(Error::new(x.span(), "expected `[...]`"))
        }
    }

    #[inline]
    pub fn parse_brace<R>(&mut self, f: impl FnOnce(Self) -> Result<R>) -> Result<R> {
        let x = self.next()?;
        if let CssToken::Brace(x) = x {
            Ok(f(Self::new(x.span(), x.block.into_vec()))?)
        } else {
            Err(Error::new(x.span(), "expected `{...}`"))
        }
    }
}
