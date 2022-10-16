#![recursion_limit = "128"]

use rustc_hash::FxHashMap;
use proc_macro2::Span;

// pub mod parser;
pub mod css_token;
use css_token::*;
pub mod write_css;
pub mod style_sheet;
pub mod pseudo;
pub mod mac;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn new(span: Span, message: impl ToString) -> Self {
        Self {
            span,
            message: message.to_string(),
        }
    }

    pub fn into_syn_error(self) -> syn::Error {
        syn::Error::new(self.span, self.message)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub trait ParseWithVars: Sized {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError>;
    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef));
}

#[derive(Default)]
pub struct StyleSheetVars {
    macros: FxHashMap<CssIdent, mac::MacroDefinition>,
    consts: FxHashMap<CssVarRef, ConstOrKeyframe>,
}

#[derive(Debug, Clone)]
pub struct ConstOrKeyframe {
    pub tokens: Vec<CssToken>,
}

#[derive(Default)]
pub struct ScopeVars<'a> {
    pat_var_values: Option<&'a mut mac::PatVarValues>,
}
