#![recursion_limit = "128"]

use rustc_hash::FxHashMap;
use proc_macro2::Span;

// pub mod parser;
pub mod css_token;
use css_token::*;
pub mod write_css;
pub mod style_sheet;
pub mod pseudo;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArgType {
    Str,
    Num,
}

#[derive(Debug, Clone)]
pub struct VarDynRef {
    span: Span,
    index: usize,
}

impl PartialEq for VarDynRef {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[derive(Debug, Clone)]
pub struct VarDynValue {
    span: Span,
    kind: VarDynValueKind,
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
