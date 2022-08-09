use proc_macro2::Span;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

use super::css_token::*;

pub(super) struct MacroDefinition {
    brace_token: token::Brace,
    branches: Vec<MacroBranch>,
}

impl Parse for MacroDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        // let block = content.parse()?;
        todo!()
    }
}

pub(super) struct MacroBranch {
    paren_token: token::Paren,
    patterns: Vec<MacroPat>,
    fat_arrow_token: token::FatArrow,
    brace_token: token::Brace,
    body: Vec<MacroContent>,
    semi_token: Option<token::Semi>,
}

pub(super) enum MacroPat {
    Var {
        dollar_token: token::Dollar,
        var_name: CssIdent,
        colon_token: token::Colon,
        ty: MacroPatTy,
    },
    Ident(CssIdent),
    AtKeyword(CssAtKeyword),
    String(CssString),
    Delim(CssDelim),
    Colon(CssColon),
    Semi(CssSemi),
    Function(CssFunction<Vec<MacroPat>>),
    Paren(CssParen<Vec<MacroPat>>),
    Bracket(CssBracket<Vec<MacroPat>>),
    Brace(CssBrace<Vec<MacroPat>>),
}

pub(super) enum MacroPatTy {
    Tt,
    Ident,
}

pub(super) enum MacroContent {
    Var {
        dollar_token: token::Dollar,
        var_name: CssIdent,
    },
    Ident(CssIdent),
    AtKeyword(CssAtKeyword),
    String(CssString),
    Delim(CssDelim),
    Colon(CssColon),
    Semi(CssSemi),
    Function(CssFunction<Vec<MacroContent>>),
    Paren(CssParen<Vec<MacroContent>>),
    Bracket(CssBracket<Vec<MacroContent>>),
    Brace(CssBrace<Vec<MacroContent>>),
}
