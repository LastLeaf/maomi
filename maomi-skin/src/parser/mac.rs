use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

use super::{css_token::*, ParseWithVars};

pub struct MacroDefinition {
    branches: Repeat<MacroBranch>,
}

impl ParseWithVars for MacroDefinition {
    fn parse_with_vars(input: ParseStream, vars: &super::StyleSheetVars) -> Result<Self> {
        Ok(Self {
            branches: ParseWithVars::parse_with_vars(input, vars)?,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for branch in &self.branches {
            branch.for_each_ref(f);
        }
    }
}

pub(super) struct MacroBranch {
    pattern: CssParen<Repeat<MacroPat>>,
    #[allow(dead_code)]
    fat_arrow_token: token::FatArrow,
    body: CssBrace<Repeat<MacroContent>>,
    #[allow(dead_code)]
    semi_token: Option<token::Semi>,
}

impl ParseWithVars for MacroBranch {
    fn parse_with_vars(input: ParseStream, vars: &super::StyleSheetVars) -> Result<Self> {
        Ok(Self {
            pattern: input.parse()?,
            fat_arrow_token: input.parse()?,
            body: ParseWithVars::parse_with_vars(input, vars)?,
            semi_token: input.parse()?,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.body.block.for_each_ref(f);
    }
}

pub(super) enum MacroPat {
    Var {
        dollar_token: token::Dollar,
        var_name: CssIdent,
        colon_token: token::Colon,
        ty: MacroPatTy,
    },
    Function(CssFunction<Repeat<MacroPat>>),
    Paren(CssParen<Repeat<MacroPat>>),
    Bracket(CssBracket<Repeat<MacroPat>>),
    Brace(CssBrace<Repeat<MacroPat>>),
    Token(CssToken),
}

pub(super) enum MacroPatTy {
    Tt,
    Ident,
    Value,
}

impl Parse for MacroPat {
    fn parse(input: ParseStream) -> Result<Self> {
        let t = if input.peek(token::Dollar) {
            let dollar_token = input.parse()?;
            let la = input.lookahead1();
            if la.peek(token::Dollar) {
                MacroPat::Token(CssToken::Delim(input.parse()?))
            } else if la.peek(Ident) || la.peek(token::Sub) {
                let var_name = input.parse()?;
                let colon_token = input.parse()?;
                let ty_name: Ident = input.parse()?;
                let ty = match ty_name.to_string().as_str() {
                    "tt" => MacroPatTy::Tt,
                    "ident" => MacroPatTy::Ident,
                    "value" => MacroPatTy::Value,
                    _ => {
                        return Err(Error::new(ty_name.span(), "Illegal var type (expected `tt` `ident` or `value`)"));
                    }
                };
                MacroPat::Var {
                    dollar_token,
                    var_name,
                    colon_token,
                    ty,
                }
            } else {
                return Err(la.error());
            }
        } else if input.peek(token::Paren) {
            MacroPat::Paren(input.parse()?)
        } else if input.peek(token::Bracket) {
            MacroPat::Bracket(input.parse()?)
        } else if input.peek(token::Brace) {
            MacroPat::Brace(input.parse()?)
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(token::Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let block = input.parse()?;
                MacroPat::Function(CssFunction {
                    span: x.span,
                    formal_name: x.formal_name,
                    paren_token,
                    block,
                })
            } else {
                MacroPat::Token(CssToken::Ident(x))
            }
        } else if let Ok(x) = input.parse() {
            MacroPat::Token(x)
        } else {
            return Err(input.error("Illegal macro pattern token"));
        };
        Ok(t)
    }
}

pub(super) enum MacroContent {
    VarRef {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        var_name: CssIdent,
    },
    ConstRef {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        var_name: CssIdent,
        tokens: Vec<CssToken>,
    },
    KeyframesRef {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        var_name: CssIdent,
        ident: CssIdent,
    },
    MacroRef {
        name: CssIdent,
        args: CssParen<Repeat<MacroContent>>,
    },
    Function(CssFunction<Repeat<MacroContent>>),
    Paren(CssParen<Repeat<MacroContent>>),
    Bracket(CssBracket<Repeat<MacroContent>>),
    Brace(CssBrace<Repeat<MacroContent>>),
    Token(CssToken),
}

impl ParseWithVars for MacroContent {
    fn parse_with_vars(input: ParseStream, vars: &super::StyleSheetVars) -> Result<Self> {
        let t = if input.peek(Token![$]) {
            let dollar_token = input.parse()?;
            let var_name: CssIdent = input.parse()?;
            if let Some(items) = vars.consts.get(&var_name.formal_name) {
                MacroContent::ConstRef { dollar_token, var_name, tokens: items.clone() }
            } else if let Some(ident) = vars.keyframes.get(&var_name.formal_name).cloned() {
                MacroContent::KeyframesRef { dollar_token, var_name, ident }
            } else {
                return Err(Error::new(
                    var_name.span(),
                    format!("No variable, const or keyframes named {:?}", var_name.formal_name),
                ));
            }
        } else if input.peek(token::Paren) {
            MacroContent::Paren(ParseWithVars::parse_with_vars(
                &input, vars,
            )?)
        } else if input.peek(token::Bracket) {
            MacroContent::Bracket(ParseWithVars::parse_with_vars(
                &input, vars,
            )?)
        } else if input.peek(token::Brace) {
            MacroContent::Brace(ParseWithVars::parse_with_vars(
                &input, vars,
            )?)
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(Token![!]) {
                MacroContent::MacroRef {
                    name: x,
                    args: ParseWithVars::parse_with_vars(&input, vars)?,
                }
            } else if input.peek(token::Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let block = ParseWithVars::parse_with_vars(&content, vars)?;
                MacroContent::Function(CssFunction {
                    span: x.span,
                    formal_name: x.formal_name,
                    paren_token,
                    block,
                })
            } else {
                MacroContent::Token(CssToken::Ident(x))
            }
        } else if let Ok(x) = input.parse() {
            MacroContent::Token(x)
        } else {
            return Err(input.error("Illegal macro content token"));
        };
        Ok(t)
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        match self {
            MacroContent::VarRef { .. } => {}
            MacroContent::ConstRef { var_name, .. } | MacroContent::KeyframesRef { var_name, .. } => {
                f(var_name);
            }
            MacroContent::MacroRef { name, .. } => {
                f(name);
            }
            MacroContent::Function(_) => {}
            MacroContent::Paren(_) => {}
            MacroContent::Bracket(_) => {}
            MacroContent::Brace(_) => {}
            MacroContent::Token(_) => {}
        }
    }
}
