use std::{num::NonZeroU32, collections::VecDeque};
use proc_macro2::Span;
use quote::TokenStreamExt;

use crate::{ScopeVarValue, VarDynValue, MaybeDyn, Number, style_sheet::ParseStyleSheetValue};

use super::{
    write_css::{CssWriter, WriteCss, WriteCssSepCond},
    ParseWithVars, ScopeVars, ParseError
};

#[derive(Debug, Clone)]
pub struct CssIdent {
    pub span: Span,
    pub formal_name: String,
}

impl CssIdent {
    pub fn new(span: Span, name: &str) -> Self {
        Self {
            span,
            formal_name: name.to_string(),
        }
    }

    pub fn is(&self, s: &str) -> bool {
        self.formal_name.as_str() == s
    }

    pub fn css_name(&self) -> String {
        self.formal_name.replace('_', "-")
    }
}

impl std::hash::Hash for CssIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.formal_name.hash(state);
    }
}

impl PartialEq for CssIdent {
    fn eq(&self, other: &Self) -> bool {
        self.formal_name == other.formal_name
    }
}

impl Eq for CssIdent {}

impl syn::parse::Parse for CssIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let span = ident.span();
        let formal_name = ident.to_string();
        Ok(Self { span, formal_name })
    }
}

impl WriteCss for CssIdent {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        _values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_ident(&self.css_name(), true)
    }
}

#[derive(Clone)]
pub struct Keyword {
    pub span: Span,
    pub formal_name: String,
}

impl Keyword {
    pub fn is(&self, s: &str) -> bool {
        self.formal_name.as_str() == s
    }
}

impl std::fmt::Debug for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyword")
            .field(&"span", &self.span)
            .field(&"formal_name", &self.formal_name)
            .finish()
    }
}

#[derive(Clone)]
pub struct CssString {
    pub span: Span,
    pub s: MaybeDyn<String>,
}

impl CssString {
    pub fn value(&self, values: &[VarDynValue]) -> String {
        self.s.value(values).expect("argument value not enough").to_string()
    }
}

impl std::fmt::Debug for CssString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssString")
            .field(&"span", &self.span)
            .field(&"value", &self.s)
            .finish()
    }
}

impl WriteCss for CssString {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
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
            let s = self.s.value(values).expect("argument value not enough");
            write!(w, "{:?}", s)?;
            Ok(WriteCssSepCond::Other)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssDelim {
    pub span: Span,
    pub s: &'static str,
}

impl CssDelim {
    pub fn is(&self, expect: &str) -> bool {
        self.s == expect
    }
}

impl WriteCss for CssDelim {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        _values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_delim(&self.s, true)
    }
}

#[derive(Debug, Clone)]
pub struct CssNumber {
    pub span: Span,
    pub value: MaybeDyn<Number>,
}

impl CssNumber {
    pub fn integer(&self) -> Option<i32> {
        match &self.value {
            MaybeDyn::Static(Number::I32(x)) => Some(*x),
            _ => None,
        }
    }

    pub fn positive_integer(&self) -> Option<NonZeroU32> {
        self.integer().and_then(|x| {
            let x = x as u32;
            if x <= 0 {
                None
            } else {
                Some(unsafe { NonZeroU32::new_unchecked(x) })
            }
        })
    }
}

impl WriteCss for CssNumber {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
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
                    | WriteCssSepCond::Dot
                    | WriteCssSepCond::PlusOrMinus => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            match &self.value {
                MaybeDyn::Static(Number::I32(x)) => {
                    write!(w, "{}", x)?;
                }
                MaybeDyn::Static(Number::F32(x)) => {
                    write!(w, "{}", x)?;
                }
                MaybeDyn::Dyn(_) => {

                }
            }
            match self.value.value(values).expect("argument value not enough") {
                Number::I32(x) => write!(w, "{}", x)?,
                Number::F32(x) => write!(w, "{}", x)?,
            }
            Ok(WriteCssSepCond::Digit)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssPercentage {
    pub span: Span,
    pub value: MaybeDyn<Number>,
}

impl CssPercentage {
    pub(crate) fn new_int(span: Span, value: i32) -> Self {
        Self {
            span,
            value: MaybeDyn::Static(Number::I32(value)),
        }
    }
}

impl WriteCss for CssPercentage {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
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
                    | WriteCssSepCond::Dot
                    | WriteCssSepCond::PlusOrMinus => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            match self.value.value(values).expect("argument value not enough") {
                Number::I32(x) => write!(w, "{}%", x)?,
                Number::F32(x) => write!(w, "{}%", x)?,
            }
            Ok(WriteCssSepCond::Other)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssDimension {
    pub span: Span,
    pub value: MaybeDyn<Number>,
    pub unit: String,
}

impl WriteCss for CssDimension {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
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
                    | WriteCssSepCond::Dot
                    | WriteCssSepCond::PlusOrMinus => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            match self.value.value(values).expect("argument value not enough") {
                Number::I32(x) => write!(w, "{}{}", x, self.unit)?,
                Number::F32(x) => write!(w, "{}{}", x, self.unit)?,
            }
            Ok(WriteCssSepCond::NonIdentAlpha)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssColor {
    pub span: Span,
    pub value: MaybeDyn<String>,
}

impl WriteCss for CssColor {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
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
            let v = self.value.value(values).expect("argument value not enough");
            write!(w, "#{}", v)?;
            Ok(WriteCssSepCond::Digit)
        })
    }
}

#[derive(Clone)]
pub struct CssFunction<T> {
    pub span: Span,
    pub formal_name: String,
    pub block: T,
}

impl<T> CssFunction<T> {
    pub fn css_name(&self) -> String {
        self.formal_name.replace('_', "-")
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for CssFunction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssFunction")
            .field(&"span", &self.span)
            .field(&"formal_name", &self.formal_name)
            .field(&"block", &self.block)
            .finish()
    }
}

impl<T: WriteCss> WriteCss for CssFunction<T> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_function_block(true, &self.css_name(), |cssw| self.block.write_css_with_args(cssw, values))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssFunction<T> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let ident: syn::Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let block = T::parse_with_vars(&content, scope)?;
        Ok(Self {
            span: ident.span(),
            formal_name: ident.to_string(),
            block,
        })
    }
}

#[derive(Clone)]
pub struct CssParen<T> {
    pub span: Span,
    pub block: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for CssParen<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssParen")
            .field(&"block", &self.block)
            .finish()
    }
}

impl<T: WriteCss> WriteCss for CssParen<T> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_paren_block(|cssw| self.block.write_css_with_args(cssw, values))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssParen<T> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let block = T::parse_with_vars(&content, scope)?;
        Ok(Self {
            span: paren_token.span,
            block,
        })
    }
}

#[derive(Clone)]
pub struct CssBracket<T> {
    pub span: Span,
    pub block: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for CssBracket<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssBracket")
            .field(&"block", &self.block)
            .finish()
    }
}

impl<T: WriteCss> WriteCss for CssBracket<T> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_bracket_block(|cssw| self.block.write_css_with_args(cssw, values))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssBracket<T> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let content;
        let paren_token = syn::bracketed!(content in input);
        let block = T::parse_with_vars(&content, scope)?;
        Ok(Self {
            span: paren_token.span,
            block,
        })
    }
}

#[derive(Clone)]
pub struct CssBrace<T> {
    pub span: Span,
    pub block: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for CssBrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssBrace")
            .field(&"block", &self.block)
            .finish()
    }
}

impl<T: WriteCss> WriteCss for CssBrace<T> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_brace_block(|cssw| self.block.write_css_with_args(cssw, values))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssBrace<T> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let content;
        let paren_token = syn::braced!(content in input);
        let block = T::parse_with_vars(&content, scope)?;
        Ok(Self {
            span: paren_token.span,
            block,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VarName {
    pub ident: syn::Ident,
}

impl VarName {
    pub fn into_ref(self) -> VarRef {
        VarRef { ident: self.ident.clone() }
    }

    pub fn span(&self) -> Span {
        self.ident.span()
    }

    pub fn formal_name(&self) -> String {
        self.ident.to_string()
    }

    pub fn css_name(&self) -> String {
        let s = self.ident.to_string();
        s.strip_prefix("r#").unwrap_or(s.as_str()).to_string()
    }
}

impl std::hash::Hash for VarName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}

impl PartialEq for VarName {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for VarName {}

impl syn::parse::Parse for VarName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        Ok(Self { ident })
    }
}

impl quote::ToTokens for VarName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        tokens.append_all(quote::quote! { #ident });
    }
}

#[derive(Debug, Clone)]
pub struct VarRef {
    pub ident: syn::Ident,
}

impl syn::parse::Parse for VarRef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        Ok(Self { ident })
    }
}

impl quote::ToTokens for VarRef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        tokens.append_all(quote::quote! { #ident });
    }
}

#[derive(Debug, Clone)]
pub enum CssToken {
    Ident(CssIdent),
    String(CssString),
    Delim(CssDelim),
    Number(CssNumber),
    Percentage(CssPercentage),
    Dimension(CssDimension),
    Color(CssColor),
    Function(CssFunction<CssTokenStream>),
    Paren(CssParen<CssTokenStream>),
    Bracket(CssBracket<CssTokenStream>),
    Brace(CssBrace<CssTokenStream>),
}

impl CssToken {
    pub fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Ident(x) => {
                if let Self::Ident(y) = other {
                    x.formal_name == y.formal_name
                } else {
                    false
                }
            }
            Self::String(x) => {
                if let Self::String(y) = other {
                    x.s == y.s
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
                    x.value == y.value
                } else {
                    false
                }
            }
            Self::Percentage(x) => {
                if let Self::Percentage(y) = other {
                    x.value == y.value
                } else {
                    false
                }
            }
            Self::Dimension(x) => {
                if let Self::Dimension(y) = other {
                    x.value == y.value && x.unit == y.unit
                } else {
                    false
                }
            }
            Self::Color(x) => {
                if let Self::Color(y) = other {
                    x.value == y.value
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

    pub fn span(&self) -> Span {
        match self {
            Self::Ident(x) => x.span,
            Self::String(x) => x.span,
            Self::Delim(x) => x.span,
            Self::Number(x) => x.span,
            Self::Percentage(x) => x.span,
            Self::Dimension(x) => x.span,
            Self::Color(x) => x.span,
            Self::Function(x) => x.span,
            Self::Paren(x) => x.span,
            Self::Bracket(x) => x.span,
            Self::Brace(x) => x.span,
        }
    }
}

impl WriteCss for CssToken {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        let sc = match self {
            Self::Ident(x) => x.write_css_with_args(cssw, values)?,
            Self::String(x) => x.write_css_with_args(cssw, values)?,
            Self::Delim(x) => x.write_css_with_args(cssw, values)?,
            Self::Number(x) => x.write_css_with_args(cssw, values)?,
            Self::Percentage(x) => x.write_css_with_args(cssw, values)?,
            Self::Dimension(x) => x.write_css_with_args(cssw, values)?,
            Self::Color(x) => x.write_css_with_args(cssw, values)?,
            Self::Function(x) => x.write_css_with_args(cssw, values)?,
            Self::Paren(x) => x.write_css_with_args(cssw, values)?,
            Self::Bracket(x) => x.write_css_with_args(cssw, values)?,
            Self::Brace(x) => x.write_css_with_args(cssw, values)?,
        };
        Ok(sc)
    }
}

impl ParseWithVars for CssToken {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        use syn::{*, spanned::Spanned};

        fn parse_css_delim(input: syn::parse::ParseStream) -> Result<CssDelim> {
            let la = input.lookahead1();
            macro_rules! parse_delim {
                ($x:tt) => {
                    if la.peek(Token![$x]) {
                        let x: Token![$x] = input.parse()?;
                        let span = x.span();
                        return Ok(CssDelim {
                            span,
                            s: stringify!($x),
                        });
                    }
                };
            }
            parse_delim!(:);
            parse_delim!(;);
            parse_delim!(,);
            parse_delim!(+=);
            parse_delim!(+);
            parse_delim!(&&);
            parse_delim!(&=);
            parse_delim!(&);
            parse_delim!(@);
            parse_delim!(^=);
            parse_delim!(^);
            parse_delim!(/=);
            parse_delim!(/);
            parse_delim!($);
            parse_delim!(..=);
            parse_delim!(...);
            parse_delim!(..);
            parse_delim!(.);
            parse_delim!(*=);
            parse_delim!(*);
            parse_delim!(!=);
            parse_delim!(!);
            parse_delim!(||);
            parse_delim!(|=);
            parse_delim!(|);
            parse_delim!(#);
            parse_delim!(?);
            parse_delim!(->);
            parse_delim!(%=);
            parse_delim!(%);
            parse_delim!(<<=);
            parse_delim!(>>=);
            parse_delim!(<<);
            parse_delim!(>>);
            parse_delim!(==);
            parse_delim!(=>);
            parse_delim!(=);
            parse_delim!(>=);
            parse_delim!(>);
            parse_delim!(<-);
            parse_delim!(<=);
            parse_delim!(<);
            parse_delim!(-=);
            parse_delim!(-);
            parse_delim!(~);
            Err(la.error())
        }

        let css_token = if input.peek(LitStr) {
            let ls: LitStr = input.parse()?;
            let s = ls.value();
            CssToken::String(CssString {
                span: ls.span(),
                s: MaybeDyn::Static(s),
            })
        } else if input.peek(LitInt) || input.peek(LitFloat) {
            let span;
            let value;
            let lit: Lit = input.parse()?;
            match &lit {
                Lit::Int(num) => {
                    span = num.span();
                    let v = num.base10_parse()?;
                    value = MaybeDyn::Static(Number::I32(v));
                }
                Lit::Float(num) => {
                    span = num.span();
                    let v = num.base10_parse()?;
                    value = MaybeDyn::Static(Number::F32(v));
                }
                _ => unreachable!()
            }
            if input.peek(Token![%]) {
                let _: Token![%] = input.parse()?;
                CssToken::Percentage(CssPercentage { span, value })
            } else {
                CssToken::Number(CssNumber { span, value })
            }
        } else if input.peek(token::Paren) {
            let content;
            let t = parenthesized!(content in input);
            CssToken::Paren(CssParen {
                span: t.span,
                block: ParseWithVars::parse_with_vars(&content, scope)?,
            })
        } else if input.peek(token::Bracket) {
            let content;
            let t = bracketed!(content in input);
            CssToken::Bracket(CssBracket {
                span: t.span,
                block: ParseWithVars::parse_with_vars(&content, scope)?,
            })
        } else if input.peek(token::Brace) {
            let content;
            let t = braced!(content in input);
            CssToken::Brace(CssBrace {
                span: t.span,
                block: ParseWithVars::parse_with_vars(&content, scope)?,
            })
        } else if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            let css_ident = CssIdent {
                span: ident.span(),
                formal_name: ident.to_string(),
            };
            if input.peek(token::Paren) {
                let content;
                parenthesized!(content in input);
                let is_uppercase = {
                    let first_char = *css_ident.formal_name.as_bytes().get(0).unwrap_or(&0);
                    'A' as u8 <= first_char && first_char <= 'Z' as u8
                };
                if is_uppercase {
                    let input = &content;
                    if css_ident.is("Color") {
                        let la = input.lookahead1();
                        let value = if la.peek(LitStr) {
                            let s: LitStr = input.parse()?;
                            MaybeDyn::Static(s.value())
                        } else if la.peek(Ident) {
                            let var_name: VarName = input.parse()?;
                            if let Some(v) = scope.vars.get(&var_name) {
                                match v {
                                    ScopeVarValue::DynStr(x) => {
                                        scope.var_refs.push(var_name.into_ref());
                                        MaybeDyn::Dyn(x.clone())
                                    }
                                    x => {
                                        return Err(syn::Error::new(var_name.span(), format!("expected &str, found {}", x.type_name())));
                                    }
                                }
                            } else {
                                return Err(syn::Error::new(var_name.span(), "variable not declared"));
                            }
                        } else {
                            return Err(la.error());
                        };
                        CssToken::Color(CssColor {
                            span: css_ident.span,
                            value,
                        })
                    } else if css_ident.is("Percent") {
                        let la = input.lookahead1();
                        let value = if la.peek(LitInt) {
                            let s: LitInt = input.parse()?;
                            MaybeDyn::Static(Number::I32(s.base10_parse()?))
                        } else if la.peek(LitFloat) {
                            let s: LitFloat = input.parse()?;
                            MaybeDyn::Static(Number::F32(s.base10_parse()?))
                        } else if la.peek(Ident) {
                            let var_name: VarName = input.parse()?;
                            if let Some(v) = scope.vars.get(&var_name) {
                                match v {
                                    ScopeVarValue::DynNum(x) => {
                                        scope.var_refs.push(var_name.into_ref());
                                        MaybeDyn::Dyn(x.clone())
                                    }
                                    x => {
                                        return Err(syn::Error::new(var_name.span(), format!("expected i32 or f32, found {}", x.type_name())));
                                    }
                                }
                            } else {
                                return Err(syn::Error::new(var_name.span(), "variable not declared"));
                            }
                        } else {
                            return Err(la.error());
                        };
                        CssToken::Percentage(CssPercentage {
                            span: css_ident.span,
                            value,
                        })
                    } else {
                        let la = input.lookahead1();
                        if la.peek(LitInt) {
                            let v: LitInt = input.parse()?;
                            let value = v.base10_parse()?;
                            CssToken::Dimension(CssDimension {
                                span: css_ident.span,
                                value: MaybeDyn::Static(Number::I32(value)),
                                unit: css_ident.formal_name.to_ascii_lowercase(),
                            })
                        } else if la.peek(LitFloat) {
                            let v: LitFloat = input.parse()?;
                            let value = v.base10_parse()?;
                            CssToken::Dimension(CssDimension {
                                span: css_ident.span,
                                value: MaybeDyn::Static(Number::F32(value)),
                                unit: css_ident.formal_name.to_ascii_lowercase(),
                            })
                        } else {
                            return Err(la.error());
                        }
                    }
                } else {
                    CssToken::Function(CssFunction {
                        span: css_ident.span,
                        formal_name: css_ident.formal_name,
                        block: ParseWithVars::parse_with_vars(&content, scope)?,
                    })
                }
            } else {
                let var_name = VarName { ident };
                if let Some(v) = scope.vars.get(&var_name) {
                    scope.var_refs.push(var_name.into_ref());
                    match v {
                        ScopeVarValue::Token(x) => {
                            x.clone()
                        }
                        ScopeVarValue::DynStr(x) => {
                            CssToken::String(CssString { span: x.span, s: MaybeDyn::Dyn(x.clone()) })
                        }
                        ScopeVarValue::DynNum(x) => {
                            CssToken::Number(CssNumber { span: x.span, value: MaybeDyn::Dyn(x.clone()) })
                        }
                        x => {
                            return Err(syn::Error::new(css_ident.span, format!("expected value, found {}", x.type_name())));
                        }
                    }
                } else {
                    CssToken::Ident(css_ident)
                }
            }
        } else if let Ok(delim) = parse_css_delim(input) {
            CssToken::Delim(delim)
        } else {
            return Err(input.error("unexpected token"));
        };

        Ok(css_token)
    }
}

#[derive(Debug, Clone)]
pub struct CssTokenStream {
    last_span: Span,
    tokens: VecDeque<CssToken>,
}

impl CssTokenStream {
    fn iter(&self) -> impl Iterator<Item = &CssToken> {
        self.tokens.iter()
    }

    #[inline]
    pub fn new(last_span: Span, tokens: VecDeque<CssToken>) -> Self {
        Self {
            last_span,
            tokens,
        }
    }

    #[inline]
    pub fn is_ended(&self) -> bool {
        self.tokens.is_empty()
    }

    #[inline]
    pub fn expect_ended(&self) -> Result<(), ParseError> {
        if let Some(x) = self.tokens.front() {
            Err(ParseError::new(x.span(), "expected end"))
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn span(&self) -> Span {
        if let Some(x) = self.tokens.front() {
            x.span()
        } else {
            self.last_span
        }
    }

    #[inline]
    pub fn next(&mut self) -> Result<CssToken, ParseError> {
        if let Some(x) = self.tokens.pop_front() {
            Ok(x)
        } else {
            Err(ParseError::new(self.span(), "unexpected end"))
        }
    }

    #[inline]
    pub fn peek(&self) -> Result<&CssToken, ParseError> {
        if let Some(x) = self.tokens.front() {
            Ok(x)
        } else {
            Err(ParseError::new(self.span(), "unexpected end"))
        }
    }

    #[inline]
    pub fn expect_keyword(&mut self, keyword: &str) -> Result<CssIdent, ParseError> {
        let peek = self.peek()?;
        let matched = if let CssToken::Ident(x) = peek {
            if x.is(keyword) {
                true
            } else {
                false
            }
        } else {
            false
        };
        if matched {
            if let Ok(CssToken::Ident(x)) = self.next() {
                Ok(x)
            } else {
                unreachable!()
            }
        } else {
            Err(ParseError::new(self.span(), format!("expected `{}`", keyword)))
        }
    }

    #[inline]
    pub fn expect_ident(&mut self) -> Result<CssIdent, ParseError> {
        let next = self.next()?;
        if let CssToken::Ident(x) = next {
            Ok(x)
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected CSS identifier"))
        }
    }

    #[inline]
    pub fn expect_string(&mut self) -> Result<CssString, ParseError> {
        let next = self.next()?;
        if let CssToken::String(x) = next {
            Ok(x)
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected CSS string literal"))
        }
    }

    #[inline]
    pub fn expect_delim(&mut self, s: &str) -> Result<CssDelim, ParseError> {
        let peek = self.peek()?;
        let matched = if let CssToken::Delim(x) = peek {
            if x.is(s) {
                true
            } else {
                false
            }
        } else {
            false
        };
        if matched {
            if let Ok(CssToken::Delim(x)) = self.next() {
                Ok(x)
            } else {
                unreachable!()
            }
        } else {
            Err(ParseError::new(self.span(), format!("expected `{}`", s)))
        }
    }

    #[inline]
    pub fn expect_number(&mut self) -> Result<CssNumber, ParseError> {
        let next = self.next()?;
        if let CssToken::Number(x) = next {
            Ok(x)
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected number"))
        }
    }

    #[inline]
    pub fn expect_percentage(&mut self) -> Result<CssPercentage, ParseError> {
        let next = self.next()?;
        if let CssToken::Percentage(x) = next {
            Ok(x)
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected percentage (number with `%`)"))
        }
    }

    #[inline]
    pub fn expect_dimension(&mut self) -> Result<CssDimension, ParseError> {
        let next = self.next()?;
        if let CssToken::Dimension(x) = next {
            Ok(x)
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected dimension (number with unit)"))
        }
    }

    #[inline]
    pub fn parse_function<R>(
        &mut self,
        f: impl FnOnce(&str, &mut CssTokenStream) -> Result<R, ParseError>,
    ) -> Result<CssFunction<R>, ParseError> {
        let next = self.next()?;
        if let CssToken::Function(mut x) = next {
            let block = f(&x.formal_name, &mut x.block)?;
            x.block.expect_ended()?;
            Ok(CssFunction { span: x.span, formal_name: x.formal_name, block })
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected CSS function"))
        }
    }

    #[inline]
    pub fn parse_paren<R>(
        &mut self,
        f: impl FnOnce(&mut CssTokenStream) -> Result<R, ParseError>,
    ) -> Result<CssParen<R>, ParseError> {
        let next = self.next()?;
        if let CssToken::Paren(mut x) = next {
            let block = f(&mut x.block)?;
            x.block.expect_ended()?;
            Ok(CssParen { span: x.span, block })
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected `(...)`"))
        }
    }

    #[inline]
    pub fn parse_bracket<R>(
        &mut self,
        f: impl FnOnce(&mut CssTokenStream) -> Result<R, ParseError>,
    ) -> Result<CssBracket<R>, ParseError> {
        let next = self.next()?;
        if let CssToken::Bracket(mut x) = next {
            let block = f(&mut x.block)?;
            x.block.expect_ended()?;
            Ok(CssBracket { span: x.span, block })
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected `[...]`"))
        }
    }

    #[inline]
    pub fn parse_brace<R>(
        &mut self,
        f: impl FnOnce(&mut CssTokenStream) -> Result<R, ParseError>,
    ) -> Result<CssBrace<R>, ParseError> {
        let next = self.next()?;
        if let CssToken::Brace(mut x) = next {
            let block = f(&mut x.block)?;
            x.block.expect_ended()?;
            Ok(CssBrace { span: x.span, block })
        } else {
            self.tokens.push_front(next);
            Err(ParseError::new(self.span(), "expected `{...}`"))
        }
    }
}

impl WriteCss for CssTokenStream {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::result::Result<(), std::fmt::Error> {
        for token in self.tokens.iter() {
            token.write_css_with_args(cssw, values)?;
        }
        Ok(())
    }
}

impl ParseWithVars for CssTokenStream {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let mut ret = VecDeque::new();
        while !input.is_empty() {
            ret.push_back(ParseWithVars::parse_with_vars(input, scope)?);
        }
        Ok(Self::new(input.span(), ret))
    }
}

impl ParseStyleSheetValue for CssTokenStream {
    fn parse_value(_name: &CssIdent, tokens: &mut CssTokenStream) -> Result<Self, ParseError>
    where
        Self: Sized {
        Ok(Self {
            last_span: tokens.last_span,
            tokens: tokens.tokens.drain(..).collect(),
        })
    }
}
