#![recursion_limit = "128"]

use rustc_hash::FxHashMap;
use proc_macro2::{Span, TokenStream};

// pub mod parser;
pub mod css_token;
use css_token::*;
pub mod write_css;
pub mod style_sheet;
pub mod pseudo;
mod module;

#[derive(Debug, Clone)]
pub struct ParseError {
    err: syn::Error,
}

impl ParseError {
    pub fn new(span: Span, message: impl ToString) -> Self {
        Self {
            err: syn::Error::new(span, message.to_string()),
        }
    }

    pub fn into_syn_error(self) -> syn::Error {
        self.err
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.err.to_string())
    }
}

impl From<syn::Error> for ParseError {
    fn from(err: syn::Error) -> Self {
        Self {
            err,
        }
    }
}

pub trait ParseWithVars: Sized {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error>;
}

#[derive(Debug)]
pub struct ScopeVars {
    cur_mod: Option<ModPath>,
    vars: FxHashMap<VarName, ScopeVarValue>,
    var_refs: Vec<VarRef>,
}

#[derive(Debug, Clone)]
pub enum ScopeVarValue {
    Token(CssToken),
    DynStr(VarDynRef),
    DynNum(VarDynRef),
    StyleDefinition(Vec<(VarName, ArgType)>),
}

impl ScopeVarValue {
    fn type_name(&self) -> &'static str {
        match self {
            Self::Token(_) => "value",
            Self::DynStr(_) => "&str",
            Self::DynNum(_) => "{number}",
            Self::StyleDefinition(_) => "StyleDefinition",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArgType {
    Str(Span),
    Num(Span),
}

impl ArgType {
    pub fn type_tokens(self) -> TokenStream {
        match self {
            Self::Str(span) => quote::quote_spanned!(span=> &str ),
            Self::Num(span) => quote::quote_spanned!(span=> f32 ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarDynRef {
    pub span: Span,
    pub index: usize,
}

impl PartialEq for VarDynRef {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[derive(Debug, Clone)]
pub struct VarDynValue {
    pub span: Span,
    pub kind: VarDynValueKind,
}

#[derive(Debug, Clone)]
pub enum VarDynValueKind {
    Str(String),
    Num(Number),
}

impl VarDynValue {
    fn type_name(&self) -> &'static str {
        match &self.kind {
            VarDynValueKind::Str(_) => "&str",
            VarDynValueKind::Num(_) => "{number}",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaybeDyn<T> {
    Static(T),
    Dyn(VarDynRef),
}

impl ParseWithVars for MaybeDyn<String> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        use syn::*;
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
        Ok(value)
    }
}

impl MaybeDyn<String> {
    fn value<'a>(&'a self, values: &'a [VarDynValue]) -> Result<&'a str, syn::Error> {
        match self {
            Self::Static(x) => Ok(x),
            Self::Dyn(x) => {
                let v = values.get(x.index).unwrap();
                match &v.kind {
                    VarDynValueKind::Str(x) => Ok(x),
                    _ => Err(syn::Error::new(x.span, format!("expected &str, found {}", v.type_name()))),
                }
            }
        }
    }
}

impl ParseWithVars for MaybeDyn<Number> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        use syn::*;
        let la = input.lookahead1();
        let value = if la.peek(LitInt) {
            let v: LitInt = input.parse()?;
            let value = v.base10_parse()?;
            MaybeDyn::Static(Number::I32(value))
        } else if la.peek(LitFloat) {
            let v: LitFloat = input.parse()?;
            let value = v.base10_parse()?;
            MaybeDyn::Static(Number::F32(value))
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
        Ok(value)
    }
}

impl MaybeDyn<Number> {
    fn value(&self, values: &[VarDynValue]) -> Result<Number, syn::Error> {
        match self {
            Self::Static(x) => Ok(x.clone()),
            Self::Dyn(x) => {
                let v = values.get(x.index).unwrap();
                match &v.kind {
                    VarDynValueKind::Num(x) => Ok(x.clone()),
                    _ => Err(syn::Error::new(x.span, format!("expected {{number}}, found {}", v.type_name()))),
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    I32(i32),
    F32(f32),
}

#[derive(Debug, Clone, Default)]
pub struct ModPath {
    segs: Vec<syn::Ident>,
}
