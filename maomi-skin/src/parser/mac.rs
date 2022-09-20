use rustc_hash::FxHashMap;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

use super::{css_token::*, ParseWithVars, StyleSheetVars, ScopeVars};

pub struct MacroDefinition {
    branches: Repeat<MacroBranch>,
}

impl ParseWithVars for MacroDefinition {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        Ok(Self {
            branches: ParseWithVars::parse_with_vars(input, vars, scope)?,
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
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let pattern: CssParen<Repeat<MacroPat>> = input.parse()?;
        let fat_arrow_token = input.parse()?;
        let body = {
            let mut pat_vars = Box::new(MacroPatVars::new());
            for x in pattern.block.as_slice() {
                x.collect_vars(&mut pat_vars);
            }
            let mut scope = ScopeVars { macro_pat_vars: Some(&mut pat_vars) };
            ParseWithVars::parse_with_vars(input, vars, &mut scope)?
        };
        let semi_token = input.parse()?;
        Ok(Self {
            pattern,
            fat_arrow_token,
            body,
            semi_token,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        self.body.block.for_each_ref(f);
    }
}

pub(super) struct MacroPatVars {
    self_scope: FxHashMap<String, ()>,
    list_scope: Option<Box<MacroPatVars>>,
}

impl MacroPatVars {
    fn new() -> Self {
        Self {
            self_scope: FxHashMap::default(),
            list_scope: None,
        }
    }
}

pub(super) enum MacroPat {
    Var {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        var_name: CssIdent,
        #[allow(dead_code)]
        colon_token: token::Colon,
        ty: MacroPatTy,
    },
    ListScope {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        inner: CssParen<Repeat<MacroPat>>,
        sep: Option<CssToken>,
        #[allow(dead_code)]
        star_token: token::Star,
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

impl MacroPat {
    fn collect_vars(&self, vars: &mut MacroPatVars) -> Result<()> {
        match self {
            Self::Var { var_name, .. } => {
                if let Some(_) = vars.self_scope.insert(var_name.formal_name.clone(), ()) {
                    return Err(Error::new(var_name.span, format!("Duplicated `${}`", var_name.formal_name)))
                }
            }
            Self::ListScope { inner, .. } => {
                for list in inner.block.as_slice() {
                    if let Some(x) = vars.list_scope.as_mut() {
                        list.collect_vars(x)?;
                    } else {
                        let mut x = Box::new(MacroPatVars::new());
                        list.collect_vars(&mut x)?;
                        vars.list_scope = Some(x);
                    }
                }
            },
            Self::Function(x) => {
                for x in x.block.as_slice() {
                    x.collect_vars(vars)?;
                }
            }
            Self::Paren(x) => {
                for x in x.block.as_slice() {
                    x.collect_vars(vars)?;
                }
            }
            Self::Bracket(x) => {
                for x in x.block.as_slice() {
                    x.collect_vars(vars)?;
                }
            }
            Self::Brace(x) => {
                for x in x.block.as_slice() {
                    x.collect_vars(vars)?;
                }
            }
            Self::Token(_) => {}
        }
        Ok(())
    }
}

impl Parse for MacroPat {
    fn parse(input: ParseStream) -> Result<Self> {
        let t = if input.peek(token::Dollar) {
            let dollar_token = input.parse()?;
            let la = input.lookahead1();
            if la.peek(Ident) || la.peek(token::Sub) {
                let var_name = input.parse()?;
                let colon_token = input.parse()?;
                let ty_name: Ident = input.parse()?;
                let ty = match ty_name.to_string().as_str() {
                    "tt" => MacroPatTy::Tt,
                    "ident" => MacroPatTy::Ident,
                    "value" => MacroPatTy::Value,
                    _ => {
                        return Err(Error::new(
                            ty_name.span(),
                            "Illegal var type (expected `tt` `ident` or `value`)",
                        ));
                    }
                };
                MacroPat::Var {
                    dollar_token,
                    var_name,
                    colon_token,
                    ty,
                }
            } else if la.peek(token::Paren) {
                let inner = input.parse()?;
                let (sep, star_token) = if input.peek(token::Star) {
                    (None, input.parse()?)
                } else {
                    (Some(input.parse()?), input.parse()?)
                };
                MacroPat::ListScope { dollar_token, inner, sep, star_token }
            } else if la.peek(token::Dollar) {
                MacroPat::Token(CssToken::Delim(input.parse()?))
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
                let block = content.parse()?;
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
    ListScope {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        inner: CssParen<Repeat<MacroContent>>,
        sep: Option<CssToken>,
        #[allow(dead_code)]
        star_token: token::Star,
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
        #[allow(dead_code)]
        bang_token: Token![!],
        args: Repeat<MacroContent>,
    },
    Function(CssFunction<Repeat<MacroContent>>),
    Paren(CssParen<Repeat<MacroContent>>),
    Bracket(CssBracket<Repeat<MacroContent>>),
    Brace(CssBrace<Repeat<MacroContent>>),
    Token(CssToken),
}

impl ParseWithVars for MacroContent {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let t = if input.peek(Token![$]) {
            let dollar_token = input.parse()?;
            let la = input.lookahead1();
            if la.peek(Ident) || la.peek(token::Sub) {
                let var_name: CssIdent = input.parse()?;
                if let Some(_) = scope.macro_pat_vars.as_ref().and_then(|x| x.self_scope.get(&var_name.formal_name)) {
                    MacroContent::VarRef { dollar_token, var_name }
                } else if let Some(items) = vars.consts.get(&var_name.formal_name) {
                    MacroContent::ConstRef {
                        dollar_token,
                        var_name,
                        tokens: items.clone(),
                    }
                } else if let Some(ident) = vars.keyframes.get(&var_name.formal_name).cloned() {
                    MacroContent::KeyframesRef {
                        dollar_token,
                        var_name,
                        ident,
                    }
                } else {
                    return Err(Error::new(
                        var_name.span(),
                        format!(
                            "No variable, const or keyframes named {:?}",
                            var_name.formal_name
                        ),
                    ));
                }
            } else if la.peek(token::Paren) {
                let mut scope = ScopeVars {
                    macro_pat_vars: scope.macro_pat_vars
                        .as_mut()
                        .and_then(|x| x.list_scope.as_mut().map(|x| &mut **x)),
                };
                let inner = ParseWithVars::parse_with_vars(input, vars, &mut scope)?;
                let (sep, star_token) = if input.peek(token::Star) {
                    (None, input.parse()?)
                } else {
                    (Some(input.parse()?), input.parse()?)
                };
                MacroContent::ListScope { dollar_token, inner, sep, star_token }
            } else if la.peek(token::Dollar) {
                MacroContent::Token(CssToken::Delim(input.parse()?))
            } else {
                return Err(la.error());
            }
        } else if input.peek(token::Paren) {
            MacroContent::Paren(ParseWithVars::parse_with_vars(&input, vars, scope)?)
        } else if input.peek(token::Bracket) {
            MacroContent::Bracket(ParseWithVars::parse_with_vars(&input, vars, scope)?)
        } else if input.peek(token::Brace) {
            MacroContent::Brace(ParseWithVars::parse_with_vars(&input, vars, scope)?)
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(Token![!]) {
                let name = x;
                let bang_token = input.parse()?;
                let la = input.lookahead1();
                let args = if la.peek(token::Paren) {
                    let content;
                    parenthesized!(content in input);
                    let args = ParseWithVars::parse_with_vars(&content, vars, scope)?;
                    let _: token::Semi = input.parse()?;
                    args
                } else if la.peek(token::Bracket) {
                    let content;
                    bracketed!(content in input);
                    let args = ParseWithVars::parse_with_vars(&content, vars, scope)?;
                    let _: token::Semi = input.parse()?;
                    args
                } else if la.peek(token::Brace) {
                    let content;
                    braced!(content in input);
                    ParseWithVars::parse_with_vars(&content, vars, scope)?
                } else {
                    return Err(la.error());
                };
                MacroContent::MacroRef {
                    name,
                    bang_token,
                    args,
                }
            } else if input.peek(token::Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let block = ParseWithVars::parse_with_vars(&content, vars, scope)?;
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
            MacroContent::ListScope { inner, .. } => {
                for x in inner.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            MacroContent::ConstRef { var_name, .. }
            | MacroContent::KeyframesRef { var_name, .. } => {
                f(var_name);
            }
            MacroContent::MacroRef { name, .. } => {
                f(name);
            }
            MacroContent::Function(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            MacroContent::Paren(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            MacroContent::Bracket(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            MacroContent::Brace(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            MacroContent::Token(_) => {}
        }
    }
}
