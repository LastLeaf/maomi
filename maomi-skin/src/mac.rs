use proc_macro2::Span;
use rustc_hash::FxHashMap;

use crate::{css_token::*, ParseWithVars, ScopeVars, StyleSheetVars, ParseError};

pub struct MacroDefinition {
    branches: CssBrace<Repeat<MacroBranch>>,
}

impl MacroDefinition {
    pub(crate) fn expand_recursive(
        &self,
        ret: &mut Vec<CssToken>,
        span: Span,
        call: &Vec<CssToken>,
        vars: &StyleSheetVars,
    ) -> Result<(), ParseError> {
        for branch in self.branches.block.iter() {
            let call = &mut CssTokenStream::new(span, call.clone());
            if let Some(x) = branch.match_and_expand(ret, call, vars) {
                return x;
            }
        }
        Err(ParseError::new(span, "illegal macro arguments"))
    }
}

impl ParseWithVars for MacroDefinition {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        Ok(Self {
            branches: ParseWithVars::parse_with_vars(input, vars, scope)?,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssRef)) {
        for branch in &self.branches.block {
            branch.for_each_ref(f);
        }
    }
}

pub(super) struct MacroBranch {
    pattern: CssParen<Repeat<MacroPat>>,
    body: CssBrace<Vec<CssToken>>,
}

impl MacroBranch {
    fn match_and_expand(
        &self,
        ret: &mut Vec<CssToken>,
        call: &mut CssTokenStream,
        vars: &StyleSheetVars,
    ) -> Option<Result<(), ParseError>> {
        let mut pat_vars = PatVarValues::default();
        MacroPat::try_match(
            self.pattern.block.as_slice(),
            call,
            true,
            &mut pat_vars,
        ).ok()?;
        let scope = &mut ScopeVars { pat_var_values: Some(&mut pat_vars) };
        Some(self.expand(ret, vars, scope))
    }

    fn expand(
        &self,
        ret: &mut Vec<CssToken>,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<(), ParseError> {
        for token in self.body.block.iter() {
            match token {
                CssToken::VarRef(x) => x.resolve_append(ret, vars, scope)?,
                CssToken::VarListRef(x) => x.resolve_append(ret, vars, scope)?,
                CssToken::MacroRef(x) => x.resolve_append(ret, vars, scope)?,
                x => ret.push(x.clone()),
            };
        }
        Ok(())
    }
}

impl ParseWithVars for MacroBranch {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let pattern: CssParen<Repeat<MacroPat>> = ParseWithVars::parse_with_vars(input, vars, scope)?;
        input.expect_delim("=>")?;
        let body = input.parse_brace(|input| {
            let mut tokens = vec![];
            while let Ok(x) = input.next() {
                tokens.push(x);
            }
            Ok(tokens)
        })?;
        input.expect_semi()?;
        Ok(Self {
            pattern,
            body,
        })
    }

    fn for_each_ref(&self, _f: &mut impl FnMut(&CssRef)) {
        // empty
    }
}

pub(super) enum MacroPat {
    Var {
        var_name: CssVarRef,
        ty: MacroPatTy,
    },
    ListScope {
        inner: Repeat<MacroPat>,
        sep: Option<Box<CssToken>>,
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

#[derive(Debug, Default)]
pub(super) struct PatVarValues {
    map: FxHashMap<CssVarRef, PatVarValueTokens>,
    pub(super) sub: Vec<PatVarValues>,
}

#[derive(Debug)]
enum PatVarValueTokens {
    Single(CssToken),
    Multi(Vec<CssToken>),
}

impl PatVarValues {
    pub(crate) fn expand_append(&self, ret: &mut Vec<CssToken>, var_name: &CssVarRef) -> Option<()> {
        if let Some(x) = self.map.get(var_name) {
            match x {
                PatVarValueTokens::Single(x) => {
                    ret.push(x.clone());
                }
                PatVarValueTokens::Multi(x) => {
                    for i in x.iter() {
                        ret.push(i.clone());
                    }
                }
            }
            Some(())
        } else {
            None
        }
    }
}

impl MacroPat {
    fn try_match<'a>(
        self_list: &[Self],
        call: &mut CssTokenStream,
        expect_ended: bool,
        ret: &mut PatVarValues,
    ) -> Result<(), ParseError> {
        for pat_item in self_list {
            match pat_item {
                MacroPat::Var { var_name, ty, .. } => match ty {
                    MacroPatTy::Tt => {
                        let v = call.next()?;
                        ret.map.insert(
                            var_name.clone(),
                            PatVarValueTokens::Single(v.clone()),
                        );
                    }
                    MacroPatTy::Ident => {
                        let v = call.expect_ident()?;
                        ret.map.insert(
                            var_name.clone(),
                            PatVarValueTokens::Single(CssToken::Ident(v.clone())),
                        );
                    }
                    MacroPatTy::Value => {
                        let mut tokens = vec![];
                        while !call.is_ended() {
                            if let CssToken::Semi(_) = call.peek()? {
                                break;
                            };
                            let next = call.next()?;
                            tokens.push(next);
                        }
                        if tokens.len() == 0 {
                            return Err(ParseError::new(call.span(), "unexpected token"));
                        }
                        ret.map.insert(
                            var_name.clone(),
                            PatVarValueTokens::Multi(tokens),
                        );
                    }
                },
                MacroPat::ListScope { inner, sep } => {
                    let count = ret.sub.len();
                    let mut cur_index = 0;
                    loop {
                        let mut sub_vars = if count == 0 {
                            ret.sub.push(Default::default());
                            ret.sub.last_mut().unwrap()
                        } else {
                            let sub_vars = ret.sub.get_mut(cur_index).ok_or_else(|| {
                                ParseError::new(call.span(), "unexpected token")
                            })?;
                            cur_index += 1;
                            sub_vars
                        };
                        Self::try_match(inner.as_slice(), call, false, &mut sub_vars)?;
                        if call.is_ended() {
                            break;
                        }
                        if let Some(sep) = sep.as_ref() {
                            let peek = call.peek()?;
                            if !peek.content_eq(sep) {
                                break;
                            }
                            call.next().unwrap();
                        }
                    }
                    if cur_index != count {
                        return Err(ParseError::new(call.span(), "unexpected token"));
                    }
                }
                MacroPat::Function(x) => {
                    call.parse_function(|name, call| {
                        if name != x.formal_name.as_str() {
                            return Err(ParseError::new(call.span(), "unexpected token"));
                        }
                        Self::try_match(
                            x.block.as_slice(),
                            call,
                            true,
                            ret,
                        )?;
                        Ok(())
                    })?;
                }
                MacroPat::Paren(x) => {
                    call.parse_paren(|call| {
                        Self::try_match(
                            x.block.as_slice(),
                            call,
                            true,
                            ret,
                        )?;
                        Ok(())
                    })?;
                }
                MacroPat::Bracket(x) => {
                    call.parse_bracket(|call| {
                        Self::try_match(
                            x.block.as_slice(),
                            call,
                            true,
                            ret,
                        )?;
                        Ok(())
                    })?;
                }
                MacroPat::Brace(x) => {
                    call.parse_brace(|call| {
                        Self::try_match(
                            x.block.as_slice(),
                            call,
                            true,
                            ret,
                        )?;
                        Ok(())
                    })?;
                }
                MacroPat::Token(x) => {
                    let next = call.next()?;
                    if !next.content_eq(x) {
                        return Err(ParseError::new(call.span(), "unexpected token"));
                    }
                }
            }
        }
        if expect_ended && !call.is_ended() {
            return Err(ParseError::new(call.span(), "unexpected token"));
        }
        Ok(())
    }
}

impl ParseWithVars for MacroPat {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let next = input.next()?;
        let t = if let CssToken::VarRef(x) = next {
            let _ = input.expect_colon()?;
            let ident = input.expect_ident()?;
            let ty = match ident.formal_name.as_str() {
                "tt" => MacroPatTy::Tt,
                "ident" => MacroPatTy::Ident,
                "value" => MacroPatTy::Value,
                _ => {
                    return Err(ParseError::new(
                        ident.span,
                        "Illegal var type (expected `tt` `ident` or `value`)",
                    ));
                }
            };
            MacroPat::Var {
                var_name: x,
                ty,
            }
        } else if let CssToken::VarListRef(x) = next {
            let mut s = CssTokenStream::new(x.span, x.block.into_vec());
            MacroPat::ListScope {
                inner: ParseWithVars::parse_with_vars(&mut s, vars, scope)?,
                sep: x.sep,
            }
        } else if let CssToken::Function(mut x) = next {
            MacroPat::Function(CssFunction {
                span: x.span,
                formal_name: x.formal_name,
                block: ParseWithVars::parse_with_vars(&mut x.block, vars, scope)?
            })
        } else if let CssToken::Paren(mut x) = next {
            MacroPat::Paren(CssParen {
                span: x.span,
                block: ParseWithVars::parse_with_vars(&mut x.block, vars, scope)?
            })
        } else if let CssToken::Bracket(mut x) = next {
            MacroPat::Bracket(CssBracket {
                span: x.span,
                block: ParseWithVars::parse_with_vars(&mut x.block, vars, scope)?
            })
        } else if let CssToken::Brace(mut x) = next {
            MacroPat::Brace(CssBrace {
                span: x.span,
                block: ParseWithVars::parse_with_vars(&mut x.block, vars, scope)?
            })
        } else {
            MacroPat::Token(next)
        };
        Ok(t)
    }

    fn for_each_ref(&self, _f: &mut impl FnMut(&CssRef)) {
        // empty
    }
}
