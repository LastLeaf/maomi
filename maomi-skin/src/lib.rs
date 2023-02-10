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
    Str(syn::Ident),
    U32(syn::Ident),
    I32(syn::Ident),
    F32(syn::Ident),
    StyleDefinition(Vec<(VarName, ArgType)>),
}

impl ScopeVarValue {
    fn type_name(&self) -> &'static str {
        match self {
            Self::Token(_) => "value",
            Self::Str(_) => "&str",
            Self::U32(_) => "u32",
            Self::I32(_) => "i32",
            Self::F32(_) => "f32",
            Self::StyleDefinition(_) => "StyleDefinition",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArgType {
    Str,
    U32,
    I32,
    F32,
}

#[derive(Debug, Clone, Default)]
pub struct ModPath {
    segs: Vec<syn::Ident>,
}
