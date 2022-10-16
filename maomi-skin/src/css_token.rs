use std::num::NonZeroU32;
use proc_macro2::Span;

use super::{
    write_css::{CssWriter, WriteCss, WriteCssSepCond},
    ParseWithVars, ScopeVars, StyleSheetVars, ParseError
};

// This function tries to get the byte offset of a span.
// Currently it tries to parse the `Debug` output of the span.
// It may be an unstable behavior of the stdlib or the compiler.
// So a simple check is done and panics if the output is not expected.
fn span_byte_offset(span: Span) -> Option<(usize, usize)> {
    let formatted = format!("{:?}", span);
    let bytes_start = formatted.find("bytes(")?;
    let bytes = &formatted[(bytes_start + 6)..];
    let first_end = bytes.find("..")?;
    let first_str = &bytes[..first_end];
    let first = first_str.parse().ok()?;
    let second_end = bytes.find(")")?;
    let second_str = &bytes[(first_end + 2)..second_end];
    let second = second_str.parse().ok()?;
    Some((first, second))
}

pub(crate) fn detect_byte_offset_compatibility() -> Result<(), ParseError> {
    thread_local! {
        static COMPATIBLE: bool = {
            span_byte_offset(Span::call_site()).is_some()
        };
    }
    COMPATIBLE.with(|x| {
        if *x {
            Ok(())
        } else {
            Err(ParseError::new(Span::call_site(), "failed to parse (probably incompatible with current rust compiler)"))
        }
    })
}

#[derive(Debug, Clone)]
pub struct CssIdent {
    pub span: Span,
    pub formal_name: String,
}

impl CssIdent {
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
    pub formal_name: String,
}

impl CssAtKeyword {
    pub fn is(&self, s: &str) -> bool {
        self.formal_name.as_str() == s
    }

    pub fn css_name(&self) -> String {
        self.formal_name.replace('_', "-")
    }
}

impl std::fmt::Debug for CssAtKeyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssAtKeyword")
            .field(&"span", &self.span)
            .field(&"formal_name", &self.formal_name)
            .finish()
    }
}

impl WriteCss for CssAtKeyword {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_at_keyword(self.formal_name.as_str())
    }
}

#[derive(Clone)]
pub struct CssString {
    pub span: Span,
    pub s: String,
}

impl CssString {
    pub fn value(&self) -> String {
        self.s.to_string()
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
            write!(w, "{:?}", self.s)?;
            Ok(WriteCssSepCond::Other)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssColon {
    pub span: Span,
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

impl CssDelim {
    pub fn is(&self, expect: &str) -> bool {
        self.s == expect
    }
}

impl WriteCss for CssDelim {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_delim(&self.s, true)
    }
}

#[derive(Debug, Clone)]
pub struct CssNumber {
    pub span: Span,
    pub value: f32,
    pub int_value: Option<i32>,
}

impl CssNumber {
    pub fn integer(&self) -> Option<i32> {
        self.int_value
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
            if let Some(x) = self.int_value {
                write!(w, "{}", x)?;
            } else {
                write!(w, "{}", self.value)?;
            }
            Ok(WriteCssSepCond::Digit)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssPercentage {
    pub span: Span,
    pub value: f32,
    pub int_value: Option<i32>,
}

impl CssPercentage {
    pub(crate) fn new_int(span: Span, value: i32) -> Self {
        Self {
            span,
            value: value as f32,
            int_value: Some(value),
        }
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
            if let Some(x) = self.int_value {
                write!(w, "{}%", x)?;
            } else {
                write!(w, "{}%", self.value)?;
            }
            Ok(WriteCssSepCond::Other)
        })
    }
}

#[derive(Debug, Clone)]
pub struct CssDimension {
    pub span: Span,
    pub value: f32,
    pub int_value: Option<i32>,
    pub unit: String,
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
            if let Some(x) = self.int_value {
                write!(w, "{}{}", x, self.unit)?;
            } else {
                write!(w, "{}{}", self.value, self.unit)?;
            }
            Ok(WriteCssSepCond::NonIdentAlpha)
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
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_function_block(true, &self.css_name(), |cssw| self.block.write_css(cssw))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssFunction<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        input.parse_function(|_, input| {
            T::parse_with_vars(input, vars, scope)
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        self.block.for_each_ref(f)
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
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_paren_block(|cssw| self.block.write_css(cssw))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssParen<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        input.parse_paren(|input| {
            T::parse_with_vars(input, vars, scope)
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        self.block.for_each_ref(f)
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
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_bracket_block(|cssw| self.block.write_css(cssw))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssBracket<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        input.parse_bracket(|input| {
            T::parse_with_vars(input, vars, scope)
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        self.block.for_each_ref(f)
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
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        cssw.write_brace_block(|cssw| self.block.write_css(cssw))
    }
}

impl<T: ParseWithVars> ParseWithVars for CssBrace<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        input.parse_brace(|input| {
            T::parse_with_vars(input, vars, scope)
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        self.block.for_each_ref(f)
    }
}

#[derive(Debug, Clone)]
pub struct CssVarRef {
    pub ident: CssIdent,
}

impl CssVarRef {
    pub fn into_ref(self) -> CssRef {
        CssRef {
            span: self.ident.span,
            formal_name: self.ident.formal_name,
        }
    }

    pub fn resolve_append(
        &self,
        ret: &mut Vec<CssToken>,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<(), ParseError> {
        if let Some(p) = scope.pat_var_values.as_ref() {
            if p.expand_append(ret, self).is_some() {
                return Ok(());
            }
        }
        if let Some(x) = vars.consts.get(self) {
            for x in x.tokens.iter() {
                ret.push(x.clone());
            }
            return Ok(());
        }
        return Err(ParseError::new(self.ident.span, "no such const, keyframes, or pattern variable"));
    }
}

impl std::hash::Hash for CssVarRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.formal_name.hash(state);
    }
}

impl PartialEq for CssVarRef {
    fn eq(&self, other: &Self) -> bool {
        self.ident.formal_name == other.ident.formal_name
    }
}

impl Eq for CssVarRef {}

#[derive(Clone)]
pub struct CssListRef<T> {
    pub span: Span,
    pub block: T,
    pub sep: Option<Box<CssToken>>,
}

impl CssListRef<Repeat<CssToken>> {
    pub fn resolve_append(
        &self,
        ret: &mut Vec<CssToken>,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<(), ParseError> {
        if let Some(sub_list) = scope.pat_var_values.as_mut() {
            for item in sub_list.sub.iter_mut() {
                let sub_scope = &mut ScopeVars { pat_var_values: Some(item) };
                for token in self.block.iter() {
                    token.clone().resolve_append(ret, None, vars, sub_scope)?;
                }
            }
            if let Some(sep) = self.sep.as_ref() {
                let sep: &CssToken = &sep;
                ret.push(sep.clone());
            }
            Ok(())
        } else {
            return Err(ParseError::new(self.span, "not in macro"));
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for CssListRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssListRef")
            .field(&"span", &self.span)
            .field(&"block", &self.block)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct CssMacroRef<T> {
    pub ident: CssIdent,
    pub block: T,
    pub is_brace: bool,
}

impl<T> CssMacroRef<T> {
    pub fn into_ref(self) -> CssRef {
        CssRef { span: self.ident.span, formal_name: self.ident.formal_name }
    }
}

impl CssMacroRef<Repeat<CssToken>> {
    pub fn resolve_append(
        &self,
        ret: &mut Vec<CssToken>,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<(), ParseError> {
        if let Some(x) = vars.macros.get(&self.ident) {
            let mut resolved_inner = vec![];
            for token in self.block.iter() {
                token.clone().resolve_append(&mut resolved_inner, None, vars, scope)?;
            }
            x.expand_recursive(ret, self.ident.span, &resolved_inner, vars)?;
            return Ok(());
        }
        return Err(ParseError::new(self.ident.span, "no such macro"));
    }
}

#[derive(Clone)]
pub struct CssRef {
    pub span: Span,
    pub formal_name: String,
}

#[derive(Debug, Clone)]
pub struct Repeat<T> {
    inner: Vec<T>,
}

impl<T> Repeat<T> {
    pub fn into_vec(self) -> Vec<T> {
        self.inner
    }

    pub fn from_vec(v: Vec<T>) -> Self {
        Self {
            inner: v,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.inner.iter()
    }
}

impl<T: syn::parse::Parse> syn::parse::Parse for Repeat<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut inner = vec![];
        while !input.is_empty() {
            let v = input.parse()?;
            inner.push(v);
        }
        Ok(Self { inner })
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

impl<T: ParseWithVars> ParseWithVars for Repeat<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let mut inner = vec![];
        while !input.is_ended() {
            let i = T::parse_with_vars(input, vars, scope)?;
            inner.push(i);
        }
        Ok(Self { inner })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for i in &self.inner {
            i.for_each_ref(f);
        }
    }
}

impl<'a, T> IntoIterator for &'a Repeat<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[derive(Debug, Clone)]
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
    Function(CssFunction<CssTokenStream>),
    Paren(CssParen<CssTokenStream>),
    Bracket(CssBracket<CssTokenStream>),
    Brace(CssBrace<CssTokenStream>),
    VarRef(CssVarRef),
    VarListRef(CssListRef<Repeat<CssToken>>),
    MacroRef(CssMacroRef<Repeat<CssToken>>),
}

impl CssToken {
    fn resolve_append(
        self,
        ret: &mut Vec<CssToken>,
        mut refs: Option<&mut Vec<CssRef>>,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<(), ParseError> {
        match self {
            CssToken::VarRef(x) => {
                x.resolve_append(ret, vars, scope)?;
                if let Some(refs) = refs {
                    refs.push(x.into_ref());
                }
            }
            CssToken::VarListRef(x) => {
                x.resolve_append(ret, vars, scope)?;
            }
            CssToken::MacroRef(x) => {
                x.resolve_append(ret, vars, scope)?;
                if let Some(refs) = refs {
                    refs.push(x.into_ref());
                }
            }
            CssToken::Function(mut x) => {
                let mut resolved_inner = vec![];
                while let Ok(token) = x.block.next() {
                    match refs.as_mut() {
                        Some(refs) => {
                            token.resolve_append(&mut resolved_inner, Some(refs), vars, scope)?;
                        }
                        None => {
                            token.resolve_append(&mut resolved_inner, None, vars, scope)?;
                        }
                    }
                }
                ret.push(CssToken::Function(CssFunction {
                    span: x.span,
                    formal_name: x.formal_name,
                    block: CssTokenStream::new(x.block.span(), resolved_inner),
                }));
            }
            CssToken::Paren(mut x) => {
                let mut resolved_inner = vec![];
                while let Ok(token) = x.block.next() {
                    match refs.as_mut() {
                        Some(refs) => {
                            token.resolve_append(&mut resolved_inner, Some(refs), vars, scope)?;
                        }
                        None => {
                            token.resolve_append(&mut resolved_inner, None, vars, scope)?;
                        }
                    }
                }
                ret.push(CssToken::Paren(CssParen {
                    span: x.span,
                    block: CssTokenStream::new(x.block.span(), resolved_inner),
                }));
            }
            CssToken::Bracket(mut x) => {
                let mut resolved_inner = vec![];
                while let Ok(token) = x.block.next() {
                    match refs.as_mut() {
                        Some(refs) => {
                            token.resolve_append(&mut resolved_inner, Some(refs), vars, scope)?;
                        }
                        None => {
                            token.resolve_append(&mut resolved_inner, None, vars, scope)?;
                        }
                    }
                }
                ret.push(CssToken::Bracket(CssBracket {
                    span: x.span,
                    block: CssTokenStream::new(x.block.span(), resolved_inner),
                }));
            }
            CssToken::Brace(mut x) => {
                let mut resolved_inner = vec![];
                while let Ok(token) = x.block.next() {
                    match refs.as_mut() {
                        Some(refs) => {
                            token.resolve_append(&mut resolved_inner, Some(refs), vars, scope)?;
                        }
                        None => {
                            token.resolve_append(&mut resolved_inner, None, vars, scope)?;
                        }
                    }
                }
                ret.push(CssToken::Brace(CssBrace {
                    span: x.span,
                    block: CssTokenStream::new(x.block.span(), resolved_inner),
                }));
            }
            x => {
                ret.push(x);
            }
        }
        Ok(())
    }

    pub fn content_eq(&self, other: &Self) -> bool {
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
                    x.s == y.s
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
            Self::VarRef(x) => {
                if let Self::VarRef(y) = other {
                    x.ident.formal_name == y.ident.formal_name
                } else {
                    false
                }
            }
            Self::VarListRef(x) => {
                if let Self::VarListRef(y) = other {
                    let mut eq = true;
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
            Self::MacroRef(x) => {
                if let Self::MacroRef(y) = other {
                    let mut eq = x.ident.formal_name == y.ident.formal_name;
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
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Ident(x) => x.span,
            Self::AtKeyword(x) => x.span,
            Self::String(x) => x.span,
            Self::Colon(x) => x.span,
            Self::Semi(x) => x.span,
            Self::Comma(x) => x.span,
            Self::Delim(x) => x.span,
            Self::Number(x) => x.span,
            Self::Percentage(x) => x.span,
            Self::Dimension(x) => x.span,
            Self::Function(x) => x.span,
            Self::Paren(x) => x.span,
            Self::Bracket(x) => x.span,
            Self::Brace(x) => x.span,
            Self::VarRef(x) => x.ident.span,
            Self::VarListRef(x) => x.span,
            Self::MacroRef(x) => x.ident.span,
        }
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
            Self::VarRef(_) | Self::VarListRef(_) | Self::MacroRef(_) => {
                panic!("cannot write unresolved ref");
            }
        };
        Ok(sc)
    }
}

impl syn::parse::Parse for CssToken {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{*, ext::IdentExt, spanned::Spanned};

        fn parse_css_ident_with_last_span(input: syn::parse::ParseStream) -> Result<(CssIdent, Span)> {
            let mut formal_name = String::new();
            let span;
            if input.peek(Ident::peek_any) {
                let s = Ident::parse_any(input)?;
                formal_name += &s.to_string();
                span = s.span();
            } else if input.peek(Token![-]) && input.peek2(Ident::peek_any) {
                let sub_token: Token![-] = input.parse()?;
                formal_name.push('_');
                span = sub_token.span();
                let s = Ident::parse_any(input)?;
                formal_name += &s.to_string();
            } else {
                return Err(input.error("expected CSS identifier"));
            }
            let mut last_span = span;
            if input.peek(Token![-]) {
                let mut last_span_end_offset = span_byte_offset(span).unwrap_or_default().1;
                loop {
                    let cur_span_offset = span_byte_offset(input.span()).unwrap_or_default();
                    if cur_span_offset.0 != last_span_end_offset {
                        break;
                    }
                    let t: Token![-] = input.parse()?;
                    formal_name.push('_');
                    last_span = t.span;
                    last_span_end_offset = cur_span_offset.1;
                    if input.peek(Ident::peek_any) {
                        let cur_span_offset = span_byte_offset(input.span()).unwrap_or_default();
                        if cur_span_offset.0 == last_span_end_offset {
                            let s = Ident::parse_any(input)?;
                            formal_name += &s.to_string();
                            last_span = s.span();
                            last_span_end_offset = cur_span_offset.1;
                        }
                    }
                    if !input.peek(Token![-]) {
                        break;
                    }
                }
            }
            let ident = CssIdent {
                span,
                formal_name,
            };
            Ok((ident, last_span))
        }

        fn parse_css_ident(input: syn::parse::ParseStream) -> Result<CssIdent> {
            let (ident, _) = parse_css_ident_with_last_span(input)?;
            Ok(ident)
        }

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

        let css_token = if input.peek(Token![@]) {
            let at_token: Token![@] = input.parse()?;
            let span = at_token.span();
            let css_ident = parse_css_ident(input)?;
            CssToken::AtKeyword(CssAtKeyword {
                span,
                formal_name: css_ident.formal_name,
            })
        } else if input.peek(LitStr) {
            let ls: LitStr = input.parse()?;
            let s = ls.value();
            CssToken::String(CssString {
                span: ls.span(),
                s,
            })
        } else if input.peek(Token![:]) {
            let t: Token![:] = input.parse()?;
            CssToken::Colon(CssColon {
                span: t.span(),
            })
        } else if input.peek(Token![;]) {
            let t: Token![;] = input.parse()?;
            CssToken::Semi(CssSemi {
                span: t.span(),
            })
        } else if input.peek(Token![,]) {
            let t: Token![,] = input.parse()?;
            CssToken::Comma(CssComma {
                span: t.span(),
            })
        } else if input.peek(LitInt) || input.peek(LitFloat) {
            let span;
            let value;
            let int_value;
            let suffix;
            let lit: Lit = input.parse()?;
            match &lit {
                Lit::Int(num) => {
                    span = num.span();
                    let v: i32 = num.base10_parse()?;
                    value = v as f32;
                    int_value = Some(v);
                    suffix = num.suffix();
                }
                Lit::Float(num) => {
                    span = num.span();
                    value = num.base10_parse()?;
                    int_value = None;
                    suffix = num.suffix();
                }
                _ => unreachable!()
            }
            if suffix.len() > 0 {
                return Err(Error::new(
                    span,
                    format!("`.` should be added before units, i.e. `{}.{}`", value, suffix),
                ));
            }
            if input.peek(Token![.]) {
                let _: Token![.] = input.parse()?;
                let unit = Ident::parse_any(input)?.to_string();
                CssToken::Dimension(CssDimension { span, value, int_value, unit })
            } else if input.peek(Token![%]) {
                let _: Token![%] = input.parse()?;
                CssToken::Percentage(CssPercentage { span, value, int_value })
            } else {
                CssToken::Number(CssNumber { span, value, int_value })
            }
        } else if input.peek(token::Paren) {
            let content;
            let t = parenthesized!(content in input);
            CssToken::Paren(CssParen {
                span: t.span,
                block: content.parse()?,
            })
        } else if input.peek(token::Bracket) {
            let content;
            let t = bracketed!(content in input);
            CssToken::Bracket(CssBracket {
                span: t.span,
                block: content.parse()?,
            })
        } else if input.peek(token::Brace) {
            let content;
            let t = braced!(content in input);
            CssToken::Brace(CssBrace {
                span: t.span,
                block: content.parse()?,
            })
        } else if input.peek(Token![$]) {
            let _: Token![$] = input.parse()?;
            if input.peek(Token![$]) {
                CssToken::Delim(parse_css_delim(input)?)
            } else if input.peek(token::Paren) {
                let content;
                let t = parenthesized!(content in input);
                let sep = if input.peek(Token![*]) {
                    let _: Token![*] = input.parse()?;
                    None
                } else {
                    let sep = input.parse()?;
                    let _: Token![*] = input.parse()?;
                    Some(sep)
                };
                CssToken::VarListRef(CssListRef {
                    span: t.span,
                    block: content.parse()?,
                    sep,
                })
            } else {
                let ident = parse_css_ident(input)?;
                CssToken::VarRef(CssVarRef {
                    ident,
                })
            }
        } else if input.peek(Token![#]) {
            return Err(input.error("`#` values are not supported currently (consider other forms instead)"));
        } else if let Ok((ident, last_span)) = parse_css_ident_with_last_span(input) {
            if input.peek(Token![!]) {
                let _: Token![!] = input.parse()?;
                let la = input.lookahead1();
                if la.peek(token::Paren) {
                    let content;
                    parenthesized!(content in input);
                    CssToken::MacroRef(CssMacroRef {
                        ident,
                        block: content.parse()?,
                        is_brace: false,
                    })
                } else if la.peek(token::Bracket) {
                    let content;
                    bracketed!(content in input);
                    CssToken::MacroRef(CssMacroRef {
                        ident,
                        block: content.parse()?,
                        is_brace: false,
                    })
                } else if la.peek(token::Brace) {
                    let content;
                    braced!(content in input);
                    CssToken::MacroRef(CssMacroRef {
                        ident,
                        block: content.parse()?,
                        is_brace: true,
                    })
                } else {
                    return Err(la.error());
                }
            } else if input.peek(token::Paren) {
                let paren_start = span_byte_offset(input.span()).unwrap_or_default().0;
                let ident_end = span_byte_offset(last_span).unwrap_or_default().1;
                if paren_start == ident_end {
                    let content;
                    parenthesized!(content in input);
                    CssToken::Function(CssFunction {
                        span: ident.span,
                        formal_name: ident.formal_name,
                        block: content.parse()?,
                    })
                } else {
                    CssToken::Ident(ident)
                }
            } else {
                CssToken::Ident(ident)
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
    rev_tokens: Vec<CssToken>,
}

impl CssTokenStream {
    fn iter(&self) -> impl Iterator<Item = &CssToken> {
        self.rev_tokens.iter().rev()
    }

    #[inline]
    pub fn new(last_span: Span, mut tokens: Vec<CssToken>) -> Self {
        tokens.reverse();
        Self {
            last_span,
            rev_tokens: tokens,
        }
    }

    #[inline]
    pub fn is_ended(&self) -> bool {
        self.rev_tokens.last().is_none()
    }

    #[inline]
    pub fn expect_ended(&self) -> Result<(), ParseError> {
        if let Some(x) = self.rev_tokens.last() {
            Err(ParseError::new(x.span(), "expected end"))
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn span(&self) -> Span {
        if let Some(x) = self.rev_tokens.last() {
            x.span()
        } else {
            self.last_span
        }
    }

    #[inline]
    pub fn sub_until_semi(&mut self) -> Self {
        if let Some((index, _)) = self.rev_tokens.iter().enumerate().rfind(|(_, t)| {
            if let CssToken::Semi(_) = t {
                return true;
            }
            false
        }) {
            let sub_rev_tokens = self.rev_tokens.drain(index..).collect();
            Self {
                last_span: self.span(),
                rev_tokens: sub_rev_tokens,
            }
        } else {
            let sub_rev_tokens = self.rev_tokens.drain(..).collect();
            Self {
                last_span: self.span(),
                rev_tokens: sub_rev_tokens,
            }
        }
    }

    #[inline]
    pub fn resolve_until_semi(
        &mut self,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<(Vec<CssToken>, Vec<CssRef>), ParseError> {
        let mut tokens = vec![];
        let mut refs = vec![];
        while !self.is_ended() {
            if let CssToken::Semi(_) = self.peek()? {
                break;
            }
            let next = self.next()?;
            next.resolve_append(&mut tokens, Some(&mut refs), vars, scope)?;
        }
        Ok((tokens, refs))
    }

    #[inline]
    pub fn next(&mut self) -> Result<CssToken, ParseError> {
        if let Some(x) = self.rev_tokens.pop() {
            Ok(x)
        } else {
            Err(ParseError::new(self.span(), "unexpected end"))
        }
    }

    #[inline]
    pub fn peek(&self) -> Result<&CssToken, ParseError> {
        if let Some(x) = self.rev_tokens.last() {
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
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected CSS identifier"))
        }
    }

    #[inline]
    pub fn expect_at_keyword(&mut self) -> Result<CssAtKeyword, ParseError> {
        let next = self.next()?;
        if let CssToken::AtKeyword(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected CSS at-keyword"))
        }
    }

    #[inline]
    pub fn expect_string(&mut self) -> Result<CssString, ParseError> {
        let next = self.next()?;
        if let CssToken::String(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected CSS string literal"))
        }
    }

    #[inline]
    pub fn expect_colon(&mut self) -> Result<CssColon, ParseError> {
        let next = self.next()?;
        if let CssToken::Colon(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected `:`"))
        }
    }

    #[inline]
    pub fn expect_semi(&mut self) -> Result<CssSemi, ParseError> {
        let next = self.next()?;
        if let CssToken::Semi(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected `;`"))
        }
    }

    #[inline]
    pub fn expect_comma(&mut self) -> Result<CssComma, ParseError> {
        let next = self.next()?;
        if let CssToken::Comma(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected `,`"))
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
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected number"))
        }
    }

    #[inline]
    pub fn expect_percentage(&mut self) -> Result<CssPercentage, ParseError> {
        let next = self.next()?;
        if let CssToken::Percentage(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected percentage (number with `%`)"))
        }
    }

    #[inline]
    pub fn expect_dimension(&mut self) -> Result<CssDimension, ParseError> {
        let next = self.next()?;
        if let CssToken::Dimension(x) = next {
            Ok(x)
        } else {
            self.rev_tokens.push(next);
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
            self.rev_tokens.push(next);
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
            self.rev_tokens.push(next);
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
            self.rev_tokens.push(next);
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
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), "expected `{...}`"))
        }
    }

    #[inline]
    pub fn expect_var_ref(&mut self) -> Result<CssVarRef, ParseError> {
        let next = self.next()?;
        if let CssToken::VarRef(x) = next {
            Ok(x)
        } else {
            let hint = if let CssToken::Ident(x) = &next {
                format!("expected variable name, i.e. `${}`", x.formal_name)
            } else {
                "expected variable name".to_string()
            };
            self.rev_tokens.push(next);
            Err(ParseError::new(self.span(), hint))
        }
    }
}

impl WriteCss for CssTokenStream {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        for token in self.rev_tokens.iter() {
            token.write_css(cssw)?;
        }
        Ok(())
    }
}

impl syn::parse::Parse for CssTokenStream {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ret = vec![];
        while !input.is_empty() {
            ret.push(input.parse()?);
        }
        Ok(Self::new(input.span(), ret))
    }
}
