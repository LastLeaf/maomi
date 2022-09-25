use rustc_hash::FxHashMap;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

use super::{css_token::*, ParseWithVars, ScopeVars, StyleSheetVars};

struct DroppedRefs();

impl Extend<CssIdent> for DroppedRefs {
    fn extend<T: IntoIterator<Item = CssIdent>>(&mut self, _iter: T) {
        // empty
    }
}

pub struct MacroDefinition {
    branches: Repeat<MacroBranch>,
}

impl MacroDefinition {
    pub(crate) fn expand_recursive(
        &self,
        ret: &mut Vec<CssToken>,
        call: &MacroCall<MacroArgsToken>,
        vars: &StyleSheetVars,
    ) -> Option<Result<()>> {
        for branch in self.branches.iter() {
            if let Some(x) = branch.match_and_expand(ret, call, vars) {
                return Some(x);
            }
        }
        None
    }
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

impl MacroBranch {
    fn match_and_expand(
        &self,
        ret: &mut Vec<CssToken>,
        call: &MacroCall<MacroArgsToken>,
        vars: &StyleSheetVars,
    ) -> Option<Result<()>> {
        let mut pat_vars = PatVarValues {
            map: FxHashMap::default(),
            sub: Vec::with_capacity(0),
        };
        MacroPat::try_match(
            self.pattern.block.as_slice(),
            &mut call.tokens.iter(),
            &mut pat_vars,
        )?;
        Some(MacroContent::expand(
            ret,
            self.body.block.as_slice(),
            &pat_vars,
            vars,
        ))
    }
}

impl ParseWithVars for MacroBranch {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        _scope: &mut ScopeVars,
    ) -> Result<Self> {
        let pattern: CssParen<Repeat<MacroPat>> = input.parse()?;
        let fat_arrow_token = input.parse()?;
        let body = {
            let mut pat_vars = Box::new(MacroPatVars::new());
            for x in pattern.block.as_slice() {
                x.collect_vars(&mut pat_vars)?;
            }
            let mut scope = ScopeVars {
                macro_pat_vars: Some(&mut pat_vars),
            };
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

struct PatVarValues {
    map: FxHashMap<String, PatVarValueTokens>,
    sub: Vec<PatVarValues>,
}

enum PatVarValueTokens {
    Single(MacroArgsToken),
    Multi(Vec<MacroArgsToken>),
}

impl MacroPat {
    fn try_match<'a>(
        self_list: &[Self],
        call: &mut impl Iterator<Item = &'a MacroArgsToken>,
        ret: &mut PatVarValues,
    ) -> Option<()> {
        for pat_item in self_list {
            match pat_item {
                MacroPat::Var { var_name, ty, .. } => match ty {
                    MacroPatTy::Tt => {
                        let v = call.next()?;
                        ret.map.insert(
                            var_name.formal_name.clone(),
                            PatVarValueTokens::Single(v.clone()),
                        );
                    }
                    MacroPatTy::Ident => {
                        let v = call.next()?;
                        let matched = match v {
                            MacroArgsToken::Token(CssToken::Ident(_)) => true,
                            _ => false,
                        };
                        if matched {
                            ret.map.insert(
                                var_name.formal_name.clone(),
                                PatVarValueTokens::Single(v.clone()),
                            );
                        }
                    }
                    MacroPatTy::Value => {
                        let mut list = vec![];
                        while let Some(v) = call.next() {
                            let matched = match v {
                                MacroArgsToken::Token(CssToken::Semi(_)) => false,
                                _ => true,
                            };
                            if !matched {
                                break;
                            }
                            list.push(v.clone());
                        }
                        if list.len() == 0 {
                            return None;
                        }
                        ret.map
                            .insert(var_name.formal_name.clone(), PatVarValueTokens::Multi(list));
                    }
                },
                MacroPat::ListScope { inner, sep, .. } => {
                    loop {
                        let mut sub_vars = PatVarValues {
                            map: FxHashMap::default(),
                            sub: Vec::with_capacity(0),
                        };
                        Self::try_match(inner.block.as_slice(), call, &mut sub_vars)?;
                        ret.sub.push(sub_vars);
                        if let Some(sep) = sep.as_ref() {
                            let matched = match call.next() {
                                Some(MacroArgsToken::Token(x)) => x.content_eq(sep),
                                _ => false,
                            };
                            if !matched {
                                break;
                            }
                        } else {
                            todo!(); // TODO
                        }
                    }
                }
                MacroPat::Function(x) => {
                    if let MacroArgsToken::Function(v) = call.next()? {
                        if v.formal_name != x.formal_name {
                            return None;
                        }
                        Self::try_match(
                            x.block.as_slice(),
                            &mut v.block.as_slice().into_iter(),
                            ret,
                        )?;
                    } else {
                        return None;
                    }
                }
                MacroPat::Paren(x) => {
                    if let MacroArgsToken::Paren(v) = call.next()? {
                        Self::try_match(
                            x.block.as_slice(),
                            &mut v.block.as_slice().into_iter(),
                            ret,
                        )?;
                    } else {
                        return None;
                    }
                }
                MacroPat::Bracket(x) => {
                    if let MacroArgsToken::Bracket(v) = call.next()? {
                        Self::try_match(
                            x.block.as_slice(),
                            &mut v.block.as_slice().into_iter(),
                            ret,
                        )?;
                    } else {
                        return None;
                    }
                }
                MacroPat::Brace(x) => {
                    if let MacroArgsToken::Brace(v) = call.next()? {
                        Self::try_match(
                            x.block.as_slice(),
                            &mut v.block.as_slice().into_iter(),
                            ret,
                        )?;
                    } else {
                        return None;
                    }
                }
                MacroPat::Token(x) => {
                    if let MacroArgsToken::Token(v) = call.next()? {
                        if !x.content_eq(v) {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
            }
        }
        if call.next().is_some() {
            return None;
        }
        Some({})
    }

    fn collect_vars(&self, vars: &mut MacroPatVars) -> Result<()> {
        match self {
            Self::Var { var_name, .. } => {
                if let Some(_) = vars.self_scope.insert(var_name.formal_name.clone(), ()) {
                    return Err(Error::new(
                        var_name.span,
                        format!("Duplicated `${}`", var_name.formal_name),
                    ));
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
            }
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
                MacroPat::ListScope {
                    dollar_token,
                    inner,
                    sep,
                    star_token,
                }
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
        args: MacroCall<Self>,
    },
    Function(CssFunction<Repeat<MacroContent>>),
    Paren(CssParen<Repeat<MacroContent>>),
    Bracket(CssBracket<Repeat<MacroContent>>),
    Brace(CssBrace<Repeat<MacroContent>>),
    Token(CssToken),
}

impl MacroContent {
    fn shallow_expand(
        ret: &mut Vec<MacroArgsToken>,
        self_list: &[Self],
        pat_vars: &PatVarValues,
        vars: &StyleSheetVars,
    ) -> Result<()> {
        for item in self_list {
            match item {
                Self::VarRef { var_name, .. } => {
                    let t = pat_vars.map.get(&var_name.formal_name).unwrap();
                    match t {
                        PatVarValueTokens::Single(x) => {
                            ret.push(x.clone());
                        }
                        PatVarValueTokens::Multi(x) => {
                            for item in x {
                                ret.push(item.clone());
                            }
                        }
                    }
                }
                Self::ListScope { inner, sep, .. } => {
                    for (index, v) in pat_vars.sub.iter().enumerate() {
                        if index > 0 {
                            if let Some(x) = sep.as_ref() {
                                ret.push(MacroArgsToken::Token(x.clone()));
                            }
                        }
                        Self::shallow_expand(ret, inner.block.as_slice(), v, vars)?;
                    }
                }
                Self::ConstRef { tokens, .. } => {
                    for x in tokens {
                        ret.push(MacroArgsToken::Token(x.clone()));
                    }
                }
                Self::KeyframesRef { ident, .. } => {
                    ret.push(MacroArgsToken::Token(CssToken::Ident(ident.clone())));
                }
                Self::MacroRef { name, args } => {
                    let mut expanded: Vec<MacroArgsToken> = vec![];
                    MacroContent::shallow_expand(
                        &mut expanded,
                        args.tokens.as_slice(),
                        pat_vars,
                        vars,
                    )?;
                    ret.push(MacroArgsToken::MacroRef {
                        name: name.clone(),
                        args: MacroCall {
                            tokens: Repeat::from_vec(expanded),
                            is_braced: args.is_braced,
                        },
                    })
                }
                Self::Function(x) => {
                    let mut sub = vec![];
                    Self::shallow_expand(&mut sub, x.block.as_slice(), pat_vars, vars)?;
                    let block = Repeat::from_vec(sub);
                    ret.push(MacroArgsToken::Function(CssFunction {
                        span: x.span,
                        formal_name: x.formal_name.clone(),
                        paren_token: x.paren_token,
                        block,
                    }));
                }
                Self::Paren(x) => {
                    let mut sub = vec![];
                    Self::shallow_expand(&mut sub, x.block.as_slice(), pat_vars, vars)?;
                    let block = Repeat::from_vec(sub);
                    ret.push(MacroArgsToken::Paren(CssParen {
                        paren_token: x.paren_token,
                        block,
                    }));
                }
                Self::Bracket(x) => {
                    let mut sub = vec![];
                    Self::shallow_expand(&mut sub, x.block.as_slice(), pat_vars, vars)?;
                    let block = Repeat::from_vec(sub);
                    ret.push(MacroArgsToken::Bracket(CssBracket {
                        bracket_token: x.bracket_token,
                        block,
                    }));
                }
                Self::Brace(x) => {
                    let mut sub = vec![];
                    Self::shallow_expand(&mut sub, x.block.as_slice(), pat_vars, vars)?;
                    let block = Repeat::from_vec(sub);
                    ret.push(MacroArgsToken::Brace(CssBrace {
                        brace_token: x.brace_token,
                        block,
                    }));
                }
                Self::Token(x) => {
                    ret.push(MacroArgsToken::Token(x.clone()));
                }
            }
        }
        Ok(())
    }

    fn expand(
        ret: &mut Vec<CssToken>,
        self_list: &[Self],
        pat_vars: &PatVarValues,
        vars: &StyleSheetVars,
    ) -> Result<()> {
        let refs = &mut DroppedRefs();
        let scope = &mut ScopeVars::new();
        let mut tokens = vec![];
        Self::shallow_expand(&mut tokens, self_list, pat_vars, vars)?;
        for item in tokens {
            item.write_self(ret, refs, vars, scope)?;
        }
        Ok(())
    }
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
                if let Some(_) = scope
                    .macro_pat_vars
                    .as_ref()
                    .and_then(|x| x.self_scope.get(&var_name.formal_name))
                {
                    MacroContent::VarRef {
                        dollar_token,
                        var_name,
                    }
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
                    macro_pat_vars: scope
                        .macro_pat_vars
                        .as_mut()
                        .and_then(|x| x.list_scope.as_mut().map(|x| &mut **x)),
                };
                let inner = ParseWithVars::parse_with_vars(input, vars, &mut scope)?;
                let (sep, star_token) = if input.peek(token::Star) {
                    (None, input.parse()?)
                } else {
                    (Some(input.parse()?), input.parse()?)
                };
                MacroContent::ListScope {
                    dollar_token,
                    inner,
                    sep,
                    star_token,
                }
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
                MacroContent::MacroRef {
                    name: x,
                    args: ParseWithVars::parse_with_vars(input, vars, scope)?,
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

#[derive(Clone)]
pub(crate) struct MacroCall<T> {
    tokens: Repeat<T>,
    is_braced: bool,
}

impl<T> MacroCall<T> {
    pub(crate) fn is_braced(&self) -> bool {
        self.is_braced
    }
}

impl<T: ParseWithVars> ParseWithVars for MacroCall<T> {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let _: Token![!] = input.parse()?;
        let la = input.lookahead1();
        let (tokens, is_braced) = if la.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let args = ParseWithVars::parse_with_vars(&content, vars, scope)?;
            (args, false)
        } else if la.peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            let args = ParseWithVars::parse_with_vars(&content, vars, scope)?;
            (args, false)
        } else if la.peek(token::Brace) {
            let content;
            braced!(content in input);
            let args = ParseWithVars::parse_with_vars(&content, vars, scope)?;
            (args, true)
        } else {
            return Err(la.error());
        };
        Ok(Self { tokens, is_braced })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for t in self.tokens.iter() {
            t.for_each_ref(f);
        }
    }
}

#[derive(Clone)]
pub(crate) enum MacroArgsToken {
    Ref {
        #[allow(dead_code)]
        dollar_token: token::Dollar,
        var_name: CssIdent,
    },
    MacroRef {
        name: CssIdent,
        args: MacroCall<MacroArgsToken>,
    },
    Function(CssFunction<Repeat<MacroArgsToken>>),
    Paren(CssParen<Repeat<MacroArgsToken>>),
    Bracket(CssBracket<Repeat<MacroArgsToken>>),
    Brace(CssBrace<Repeat<MacroArgsToken>>),
    Token(CssToken),
}

impl MacroArgsToken {
    fn write_ref(
        ret: &mut Vec<CssToken>,
        refs: &mut impl Extend<CssIdent>,
        var_name: CssIdent,
        vars: &StyleSheetVars,
    ) -> Result<()> {
        if let Some(items) = vars.consts.get(&var_name.formal_name) {
            for item in items {
                ret.push(item.clone());
            }
        } else if let Some(ident) = vars.keyframes.get(&var_name.formal_name) {
            ret.push(CssToken::Ident(ident.clone()));
        } else {
            return Err(Error::new(
                var_name.span(),
                format!("no const or keyframes named {:?}", var_name.formal_name),
            ));
        }
        refs.extend(Some(var_name));
        Ok(())
    }

    pub(crate) fn write_macro_ref(
        ret: &mut Vec<CssToken>,
        refs: &mut impl Extend<CssIdent>,
        name: &CssIdent,
        args: &MacroCall<Self>,
        vars: &StyleSheetVars,
    ) -> Result<()> {
        let mac = vars.macros.get(&name.formal_name).ok_or_else(|| {
            Error::new(
                name.span(),
                format!("no macro named {:?}", name.formal_name),
            )
        })?;
        if mac.expand_recursive(ret, args, vars).is_none() {
            return Err(Error::new(name.span(), "no macro rule matched"));
        }
        args.for_each_ref(&mut |x| refs.extend(Some(x.clone())));
        Ok(())
    }

    fn write_function<T: Extend<CssIdent>>(
        ret: &mut Vec<CssToken>,
        refs: &mut T,
        name: CssIdent,
        input: ParseStream,
        f: impl FnOnce(&mut Vec<CssToken>, &mut T, ParseStream) -> Result<()>,
    ) -> Result<()> {
        let content;
        let paren_token = parenthesized!(content in input);
        let mut sub = vec![];
        f(&mut sub, refs, &content)?;
        ret.push(CssToken::Function(CssFunction {
            span: name.span,
            formal_name: name.formal_name,
            paren_token,
            block: Repeat::from_vec(sub),
        }));
        Ok(())
    }

    fn write_paren<T: Extend<CssIdent>>(
        ret: &mut Vec<CssToken>,
        refs: &mut T,
        input: ParseStream,
        f: impl FnOnce(&mut Vec<CssToken>, &mut T, ParseStream) -> Result<()>,
    ) -> Result<()> {
        let content;
        let paren_token = parenthesized!(content in input);
        let mut sub = vec![];
        f(&mut sub, refs, &content)?;
        ret.push(CssToken::Paren(CssParen {
            paren_token,
            block: Repeat::from_vec(sub),
        }));
        Ok(())
    }

    fn write_bracket<T: Extend<CssIdent>>(
        ret: &mut Vec<CssToken>,
        refs: &mut T,
        input: ParseStream,
        f: impl FnOnce(&mut Vec<CssToken>, &mut T, ParseStream) -> Result<()>,
    ) -> Result<()> {
        let content;
        let bracket_token = bracketed!(content in input);
        let mut sub = vec![];
        f(&mut sub, refs, &content)?;
        ret.push(CssToken::Bracket(CssBracket {
            bracket_token,
            block: Repeat::from_vec(sub),
        }));
        Ok(())
    }

    fn write_braced<T: Extend<CssIdent>>(
        ret: &mut Vec<CssToken>,
        refs: &mut T,
        input: ParseStream,
        f: impl FnOnce(&mut Vec<CssToken>, &mut T, ParseStream) -> Result<()>,
    ) -> Result<()> {
        let content;
        let brace_token = braced!(content in input);
        let mut sub = vec![];
        f(&mut sub, refs, &content)?;
        ret.push(CssToken::Brace(CssBrace {
            brace_token,
            block: Repeat::from_vec(sub),
        }));
        Ok(())
    }

    fn parse_all_input_and_write(
        ret: &mut Vec<CssToken>,
        refs: &mut Vec<CssIdent>,
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<()> {
        while !input.is_empty() {
            Self::parse_input_and_write(ret, refs, input, vars, scope)?;
        }
        Ok(())
    }

    pub(crate) fn parse_input_and_write(
        ret: &mut Vec<CssToken>,
        refs: &mut Vec<CssIdent>,
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<()> {
        if input.peek(Token![$]) {
            let _: Token![$] = input.parse()?;
            let var_name: CssIdent = input.parse()?;
            Self::write_ref(ret, refs, var_name, vars)?;
        } else if input.peek(token::Paren) {
            Self::write_paren(ret, refs, input, |ret, refs, input| {
                Self::parse_all_input_and_write(ret, refs, input, vars, scope)
            })?;
        } else if input.peek(token::Bracket) {
            Self::write_bracket(ret, refs, input, |ret, refs, input| {
                Self::parse_all_input_and_write(ret, refs, input, vars, scope)
            })?;
        } else if input.peek(token::Brace) {
            Self::write_braced(ret, refs, input, |ret, refs, input| {
                Self::parse_all_input_and_write(ret, refs, input, vars, scope)
            })?;
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(Token![!]) {
                let args = ParseWithVars::parse_with_vars(input, vars, scope)?;
                Self::write_macro_ref(ret, refs, &x, &args, vars)?;
            } else if input.peek(token::Paren) {
                Self::write_function(ret, refs, x, input, |ret, refs, input| {
                    Self::parse_all_input_and_write(ret, refs, input, vars, scope)
                })?;
            } else {
                ret.push(CssToken::Ident(x));
            }
        } else if let Ok(x) = input.parse() {
            ret.push(x);
        } else {
            return Err(input.error("unexpected token"));
        }
        Ok(())
    }

    fn write_self(
        self,
        ret: &mut Vec<CssToken>,
        refs: &mut impl Extend<CssIdent>,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<()> {
        match self {
            Self::Ref { var_name, .. } => Self::write_ref(ret, refs, var_name, vars)?,
            Self::MacroRef { name, args } => Self::write_macro_ref(ret, refs, &name, &args, vars)?,
            Self::Function(x) => {
                let mut list = vec![];
                for item in x.block.into_vec().drain(..) {
                    item.write_self(&mut list, refs, vars, scope)?;
                }
                ret.push(CssToken::Function(CssFunction {
                    span: x.span,
                    formal_name: x.formal_name,
                    paren_token: x.paren_token,
                    block: Repeat::from_vec(list),
                }));
            }
            Self::Paren(x) => {
                let mut list = vec![];
                for item in x.block.into_vec().drain(..) {
                    item.write_self(&mut list, refs, vars, scope)?;
                }
                ret.push(CssToken::Paren(CssParen {
                    paren_token: x.paren_token,
                    block: Repeat::from_vec(list),
                }));
            }
            Self::Bracket(x) => {
                let mut list = vec![];
                for item in x.block.into_vec().drain(..) {
                    item.write_self(&mut list, refs, vars, scope)?;
                }
                ret.push(CssToken::Bracket(CssBracket {
                    bracket_token: x.bracket_token,
                    block: Repeat::from_vec(list),
                }));
            }
            Self::Brace(x) => {
                let mut list = vec![];
                for item in x.block.into_vec().drain(..) {
                    item.write_self(&mut list, refs, vars, scope)?;
                }
                ret.push(CssToken::Brace(CssBrace {
                    brace_token: x.brace_token,
                    block: Repeat::from_vec(list),
                }));
            }
            Self::Token(x) => {
                ret.push(x.clone());
            }
        }
        Ok(())
    }
}

impl ParseWithVars for MacroArgsToken {
    fn parse_with_vars(
        input: ParseStream,
        vars: &StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self> {
        let ret = if input.peek(Token![$]) {
            Self::Ref {
                dollar_token: input.parse()?,
                var_name: input.parse()?,
            }
        } else if input.peek(token::Paren) {
            Self::Paren(ParseWithVars::parse_with_vars(&input, vars, scope)?)
        } else if input.peek(token::Bracket) {
            Self::Bracket(ParseWithVars::parse_with_vars(&input, vars, scope)?)
        } else if input.peek(token::Brace) {
            Self::Brace(ParseWithVars::parse_with_vars(&input, vars, scope)?)
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(Token![!]) {
                Self::MacroRef {
                    name: x,
                    args: ParseWithVars::parse_with_vars(input, vars, scope)?,
                }
            } else if input.peek(token::Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let block = ParseWithVars::parse_with_vars(&content, vars, scope)?;
                Self::Function(CssFunction {
                    span: x.span,
                    formal_name: x.formal_name,
                    paren_token,
                    block,
                })
            } else {
                Self::Token(CssToken::Ident(x))
            }
        } else if let Ok(x) = input.parse() {
            Self::Token(x)
        } else {
            return Err(input.error("Illegal macro content token"));
        };
        Ok(ret)
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        match self {
            Self::Ref { var_name, .. } => {
                f(var_name);
            }
            Self::MacroRef { name, .. } => {
                f(name);
            }
            Self::Function(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            Self::Paren(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            Self::Bracket(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            Self::Brace(x) => {
                for x in x.block.as_slice() {
                    x.for_each_ref(f);
                }
            }
            Self::Token(_) => {}
        }
    }
}
