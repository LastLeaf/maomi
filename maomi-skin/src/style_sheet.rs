use std::{collections::VecDeque, rc::{Rc, Weak}, cell::RefCell};
use proc_macro2::{TokenTree, Span};
use rustc_hash::FxHashMap;
use syn::{Token, parse::ParseStream, Attribute, Visibility, Ident, ext::IdentExt, braced, parenthesized, spanned::Spanned, UseTree};

use crate::{ParseError, css_token::*, ScopeVars, ParseWithVars, write_css::*, ModPath, ScopeVarValue, ArgType, VarDynRef, VarDynValue, MaybeDyn, VarDynValueKind};

// TODO consider a proper way to handle global styling (font, css-reset, etc.)

mod kw {
    syn::custom_keyword!(style);
    syn::custom_keyword!(class);
    syn::custom_keyword!(media);
    syn::custom_keyword!(supports);
    syn::custom_keyword!(only);
    syn::custom_keyword!(not);
    syn::custom_keyword!(and);
    syn::custom_keyword!(or);
}

fn try_parse_until_semi<T>(
    input: ParseStream,
    f: impl FnOnce(ParseStream) -> Result<T, syn::Error>,
) -> Result<T, syn::Error> {
    f(input).and_then(|ret| {
        if !input.peek(Token![;]) {
            return Err(input.error("expected `;`"));
        }
        input.parse::<Token![;]>()?;
        Ok(ret)
    }).or_else(|err| {
        while !input.is_empty() && !input.peek(Token![;]) {
            input.parse::<TokenTree>()?;
        }
        input.parse::<Token![;]>()?;
        Err(err)
    })
}

fn try_parse_paren<T>(
    input: ParseStream,
    f: impl FnOnce(ParseStream) -> Result<T, syn::Error>,
) -> Result<T, syn::Error> {
    let content;
    parenthesized!(content in input);
    f(&content).and_then(|ret| {
        if !content.is_empty() {
            return Err(content.error("unexpected token"));
        }
        Ok(ret)
    })
}

fn try_parse_brace<T>(
    input: ParseStream,
    f: impl FnOnce(ParseStream) -> Result<T, syn::Error>,
) -> Result<T, syn::Error> {
    let content;
    braced!(content in input);
    f(&content).and_then(|ret| {
        if !content.is_empty() {
            return Err(content.error("unexpected token"));
        }
        Ok(ret)
    })
}

struct Paren<T: syn::parse::Parse> {
    inner: T
}

impl<T: syn::parse::Parse> syn::parse::Parse for Paren<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let inner = content.parse()?;
        Ok(Self { inner })
    }
}

/// Handlers for CSS details (varies between backends)
pub trait StyleSheetConstructor: 'static {
    type PropertyValue: ParseStyleSheetValue;
    type MediaCondValue: ParseStyleSheetValue;

    fn new() -> Self
    where
        Self: Sized;

    fn define_key_frames(
        &mut self,
        name: &VarName,
        css_name: &Option<String>,
        content: Vec<KeyFrame<Self::PropertyValue>>,
    ) -> Result<CssToken, ParseError>;

    fn to_tokens(&self, ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized;
}

/// Parse value positions
pub trait ParseStyleSheetValue {
    fn parse_value(name: &CssIdent, tokens: &mut CssTokenStream) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    ssc: T,
    pub items: Vec<Rc<StyleSheetItem<T>>>,
    pub var_context: VarContext<T>,
    pub var_refs: Vec<VarRef>,
    submodules: FxHashMap<Ident, Rc<StyleSheet<T>>>,
}

impl<T: StyleSheetConstructor> StyleSheet<T> {
    pub(crate) fn new_err(err: syn::Error) -> Self {
        let items = vec![
            Rc::new(StyleSheetItem::CompilationError(err)),
        ];
        Self {
            ssc: T::new(),
            items,
            var_context: Default::default(),
            var_refs: Vec::with_capacity(0),
            submodules: FxHashMap::default(),
        }
    }
}

impl<T: StyleSheetConstructor> syn::parse::Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let scope = &mut ScopeVars {
            cur_mod: None,
            vars: Default::default(),
            var_refs: vec![],
        };
        let ss = StyleSheet::parse_with_vars(input, scope)?;
        Ok(ss)
    }
}

impl<T: StyleSheetConstructor> quote::ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ssc.to_tokens(self, tokens)
    }
}

impl<T: StyleSheetConstructor> ParseWithVars for StyleSheet<T> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let mut ssc = T::new();
        let mut items = vec![];
        let var_context = VarContext::default();
        let mut submodules = FxHashMap::default();
        while !input.is_empty() {
            if let Some(item) = StyleSheetItem::parse_with_vars(input, scope, &var_context, &mut submodules, &mut ssc)? {
                items.push(item);
            }
        }
        Ok(Self {
            ssc,
            items,
            var_context,
            var_refs: std::mem::replace(&mut scope.var_refs, Vec::with_capacity(0)),
            submodules,
        })
    }
}

pub struct VarContext<T: StyleSheetConstructor> {
    map: Rc<RefCell<FxHashMap<VarName, Weak<StyleSheetItem<T>>>>>,
}

impl<T: StyleSheetConstructor> Clone for VarContext<T> {
    fn clone(&self) -> Self {
        Self { map: self.map.clone() }
    }
}

impl<T: StyleSheetConstructor> Default for VarContext<T> {
    fn default() -> Self {
        Self { map: Rc::new(RefCell::new(FxHashMap::default())) }
    }
}

impl<T: StyleSheetConstructor> VarContext<T> {
    pub fn get(&self, var_name: &VarName) -> Option<Rc<StyleSheetItem<T>>> {
        self.map.borrow().get(var_name).and_then(|x| x.upgrade())
    }
}

pub enum StyleSheetItem<T: StyleSheetConstructor> {
    CompilationError(syn::Error),
    ConstValue(ConstValueDefinition),
    KeyFrames(KeyFramesDefinition),
    StyleFn(StyleFnDefinition<T>),
    Style(StyleDefinition<T>),
    Class(ClassDefinition<T>),
}

pub struct KeyFrame<V: ParseStyleSheetValue> {
    pub progress: CssPercentage,
    pub items: Vec<StyleContentItem<V>>,
}

impl<T: StyleSheetConstructor> StyleSheetItem<T> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
        var_context: &VarContext<T>,
        submodules: &mut FxHashMap<Ident, Rc<StyleSheet<T>>>,
        ssc: &mut T,
    ) -> Result<Option<Rc<Self>>, syn::Error> {
        // `#[xxx(xxx)]`
        let attrs = Attribute::parse_outer(input)?;

        // `pub(xxx)`
        let (vis, extern_vis): (Option<ModPath>, Option<Visibility>) = if input.peek(Token![pub]) {
            let extern_vis: Visibility = input.parse()?;
            let vis = if let Some(cur_mod_path) = &scope.cur_mod {
                match &extern_vis {
                    Visibility::Inherited => Some(cur_mod_path.clone()),
                    Visibility::Public(_) => Some(ModPath::default()),
                    Visibility::Crate(_) => Some(ModPath::default()),
                    Visibility::Restricted(x) => {
                        let segs = x.path.segments.iter().map(|seg| {
                            assert!(seg.arguments.is_empty());
                            seg.ident.clone()
                        }).collect();
                        Some(ModPath { segs })
                    },
                }
            } else {
                None
            };
            (vis, Some(extern_vis))
        } else {
            (None, None)
        };

        let la = input.lookahead1();
        if la.peek(Token![mod]) {
            if let Some(cur_mod_path) = &scope.cur_mod {
                input.parse::<Token![mod]>()?;
                for attr in attrs {
                    return Err(syn::Error::new_spanned(attr, "unknown attribute"));
                }
                // `mod xxx;`
                unimplemented!()
            } else {
                return Err(input.error("`mod` cannot be used inside inline stylesheets"));
            }
        } else if la.peek(Token![use]) {
            let use_token: Token![use] = input.parse()?;
            for attr in attrs {
                return Err(syn::Error::new_spanned(attr, "unknown attribute"));
            }
            let mut cur_module = crate::module::root_module::<T>().ok_or_else(|| {
                syn::Error::new_spanned(use_token, "root module not found (`MAOMI_STYLESHEET_MOD_ROOT` or `CARGO_MANIFEST_DIR/src/lib.mcss` not exists?)")
            })?;
            let use_tree: UseTree = input.parse()?;
            match &use_tree {
                UseTree::Path(syn::UsePath { ident, tree, .. }) => {
                    if ident.to_string().as_str() != "crate" {
                        return Err(syn::Error::new_spanned(use_tree, "path must be started with `crate::`"));
                    }
                    fn rec<T: StyleSheetConstructor>(
                        tree: &UseTree,
                        cur_module: &Rc<StyleSheet<T>>,
                    ) -> Result<(), syn::Error> {
                        match tree {
                            UseTree::Path(syn::UsePath { ident, tree, .. }) => {
                                let m = cur_module.submodules.get(ident).ok_or_else(|| {
                                    syn::Error::new_spanned(ident, "module not found")
                                })?;
                                return rec(tree, m);
                            }
                            UseTree::Name(syn::UseName { ident }) => {

                            }
                            UseTree::Rename(syn::UseRename { ident, rename, .. }) => {

                            }
                            UseTree::Glob(_) => {

                            }
                            UseTree::Group(_) => {

                            }
                        }
                        Ok(())
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(use_tree, "path must be started with `crate::`"));
                }
            }
            unimplemented!("!!!")
        } else if la.peek(Token![const]) {
            // `const xxx: xxx = xxx;`
            let parsed = ConstValueDefinition::parse_with_vars(input, scope, attrs, vis, extern_vis, ssc)?;
            let name = match &parsed {
                StyleSheetItem::ConstValue(x) => x.name.clone(),
                StyleSheetItem::KeyFrames(x) => x.name.clone(),
                _ => unreachable!(),
            };
            let item = Rc::new(parsed);
            var_context.map.borrow_mut().insert(name, Rc::downgrade(&item));
            Ok(Some(item))
        } else if la.peek(Token![fn]) {
            // `fn xxx(xxx: xxx) { xxx }`
            if let Some(x) = extern_vis {
                if vis.is_none() {
                    return Err(syn::Error::new_spanned(x, "functions are always private in inline stylesheets"));
                }
            }
            let parsed = StyleFnDefinition::parse_with_vars(input, scope, attrs, vis, var_context)?;
            let name = parsed.name.clone();
            let item = Rc::new(Self::StyleFn(parsed));
            var_context.map.borrow_mut().insert(name, Rc::downgrade(&item));
            Ok(Some(item))
        } else if la.peek(kw::style) {
            // `style xxx(xxx: xxx) { xxx }`
            if vis.is_some() {
                return Err(input.error("style definition cannot be used inside independent stylesheet modules"));
            }
            let parsed = StyleDefinition::parse_with_vars(input, scope, attrs, extern_vis)?;
            let name = parsed.name.clone();
            let item = Rc::new(Self::Style(parsed));
            var_context.map.borrow_mut().insert(name, Rc::downgrade(&item));
            Ok(Some(item))
        } else if la.peek(kw::class) {
            // `class xxx { xxx }`
            if vis.is_some() {
                return Err(input.error("class definition cannot be used inside independent stylesheet modules"));
            }
            let parsed = ClassDefinition::parse_with_vars(input, scope, attrs, extern_vis)?;
            let name = parsed.name.clone();
            let item = Rc::new(Self::Class(parsed));
            var_context.map.borrow_mut().insert(name, Rc::downgrade(&item));
            Ok(Some(item))
        } else {
            return Err(la.error());
        }
    }
}

pub struct ConstValueDefinition {
    vis: Option<ModPath>,
    pub name: VarName,
    converted_token: CssToken,
}

impl ConstValueDefinition {
    fn parse_with_vars<T: StyleSheetConstructor>(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
        attrs: Vec<Attribute>,
        vis: Option<ModPath>,
        extern_vis: Option<Visibility>,
        ssc: &mut T,
    ) -> Result<StyleSheetItem<T>, syn::Error> {
        input.parse::<Token![const]>()?;
        let name: VarName = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = Ident::parse_any(input)?;
        input.parse::<Token![=]>()?;
        try_parse_until_semi(input, |input| {
            if let Some(x) = extern_vis {
                if vis.is_none() {
                    return Err(syn::Error::new_spanned(x, "constants are always private in inline stylesheets"));
                }
                if let Visibility::Public(_) = x {
                    return Err(syn::Error::new_spanned(x, "constants cannot be visited by other crates, use `pub(crate)` instead"));
                }
            }
            match ty.to_string().as_str() {
                "value" => {
                    for attr in attrs {
                        return Err(syn::Error::new_spanned(attr, "unknown attribute"));
                    }
                    let converted_token = CssToken::parse_with_vars(input, scope)?;
                    if scope.vars.insert(name.clone(), ScopeVarValue::Token(converted_token.clone())).is_some() {
                        return Err(syn::Error::new(name.span(), "duplicated identifier"));
                    }
                    Ok(StyleSheetItem::ConstValue(Self { vis, name, converted_token }))
                }
                "keyframes" => {
                    let mut css_name = None;
                    for attr in attrs {
                        if attr.path.is_ident("css_name") {
                            let name = syn::parse2::<Paren<syn::LitStr>>(attr.tokens)?;
                            css_name = Some(name.inner.value());
                        } else {
                            return Err(syn::Error::new_spanned(attr, "unknown attribute"));
                        }
                    }
                    let content;
                    braced!(content in input);
                    let input = &content;
                    let mut frames = vec![];
                    let var_refs = std::mem::replace(&mut scope.var_refs, vec![]);
                    let result = (|input: ParseStream| {
                        while !input.is_empty() {
                            let token: CssToken = ParseWithVars::parse_with_vars(input, scope)?;
                            let progress = match token {
                                CssToken::Ident(x) => {
                                    match x.formal_name.as_str() {
                                        "from" => CssPercentage::new_int(x.span, 0),
                                        "to" => CssPercentage::new_int(x.span, 100),
                                        _ => {
                                            return Err(syn::Error::new(x.span, "invalid keyframe progress token"));
                                        }
                                    }
                                }
                                CssToken::Percentage(x) => x,
                                x => {
                                    return Err(syn::Error::new(x.span(), "invalid keyframe progress token"));
                                }
                            };
                            let content;
                            braced!(content in input);
                            let input = &content;
                            let items = StyleContentItem::parse_with_vars(input, scope, true)?;
                            frames.push(KeyFrame { progress, items });
                        }
                        Ok(())
                    })(input);
                    let sub_var_refs = std::mem::replace(&mut scope.var_refs, var_refs);
                    result?;
                    let converted_token = ssc.define_key_frames(&name, &css_name, frames).map_err(|e| e.into_syn_error())?;
                    if scope.vars.insert(name.clone(), ScopeVarValue::Token(converted_token.clone())).is_some() {
                        return Err(syn::Error::new(name.span(), "duplicated identifier"));
                    }
                    Ok(StyleSheetItem::KeyFrames(KeyFramesDefinition { vis, name, css_name, converted_token, sub_var_refs }))
                }
                _ => Err(syn::Error::new_spanned(ty, "invalid type")),
            }
        })
    }
}

pub struct KeyFramesDefinition {
    vis: Option<ModPath>,
    pub name: VarName,
    pub css_name: Option<String>,
    converted_token: CssToken,
    pub sub_var_refs: Vec<VarRef>,
}

pub struct StyleFnDefinition<T: StyleSheetConstructor> {
    vis: Option<ModPath>,
    pub name: VarName,
    pub args: Vec<(VarName, ArgType)>,
    pub content: Vec<StyleContentItem<T::PropertyValue>>,
    pub sub_var_refs: Vec<VarRef>,
    pub var_context: VarContext<T>,
}

impl<T: StyleSheetConstructor> StyleFnDefinition<T> {
    fn parse_arg_list(
        input: syn::parse::ParseStream,
        _scope: &mut ScopeVars,
    ) -> Result<Vec<(VarName, ArgType)>, syn::Error> {
        try_parse_paren(input, |input| {
            let mut args = vec![];
            while !input.is_empty() {
                let var_name: VarName = input.parse()?;
                input.parse::<Token![:]>()?;
                let ty: syn::Type = input.parse()?;
                let span = ty.span();
                let arg_type: ArgType = match &ty {
                    syn::Type::Reference(r) if r.lifetime.is_none() && r.mutability.is_none() => {
                        match &*r.elem {
                            syn::Type::Path(p) if p.qself.is_none() && p.path.is_ident("str") => {
                                ArgType::Str(span)
                            }
                            _ => Err(syn::Error::new_spanned(ty, "invalid type, possible types: &str, f32"))?
                        }
                    }
                    syn::Type::Path(p) if p.qself.is_none() => {
                        if p.path.is_ident("f32") {
                            ArgType::Num(span)
                        } else {
                            Err(syn::Error::new_spanned(ty, "invalid type, possible types: &str, f32"))?
                        }
                    }
                    _ => Err(syn::Error::new_spanned(ty, "invalid type, possible types: &str, f32"))?
                };
                args.push((var_name, arg_type));
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }
            Ok(args)
        })
    }

    fn parse_fn_body(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
        args: &[(VarName, ArgType)],
    ) -> Result<(Vec<StyleContentItem<T::PropertyValue>>, Vec<VarRef>), syn::Error> {
        try_parse_brace(input, |input| {
            for (index, (var_name, ty)) in args.iter().enumerate() {
                let r = VarDynRef { span: var_name.span(), index };
                if scope.vars.insert(var_name.clone(), match ty {
                    ArgType::Str(_) => ScopeVarValue::DynStr(r),
                    ArgType::Num(_) => ScopeVarValue::DynNum(r),
                }).is_some() {
                    return Err(syn::Error::new(var_name.span(), "duplicated identifier"));
                };
            }
            let var_refs = std::mem::replace(&mut scope.var_refs, vec![]);
            let content_result = StyleContentItem::parse_with_vars(input, scope, true);
            let sub_var_refs = std::mem::replace(&mut scope.var_refs, var_refs);
            for (var_name, _) in args.iter() {
                scope.vars.remove(var_name);
            }
            Ok((content_result?, sub_var_refs))
        })
    }

    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
        attrs: Vec<Attribute>,
        vis: Option<ModPath>,
        var_context: &VarContext<T>,
    ) -> Result<Self, syn::Error> {
        input.parse::<Token![fn]>()?;
        let name: VarName = input.parse()?;
        for attr in attrs {
            return Err(syn::Error::new_spanned(attr, "unknown attribute"));
        }
        let args = Self::parse_arg_list(input, scope)?;
        let (content, sub_var_refs) = Self::parse_fn_body(input, scope, &args)?;
        if scope.vars.insert(name.clone(), ScopeVarValue::StyleDefinition(args.clone())).is_some() {
            return Err(syn::Error::new(name.span(), "duplicated identifier"));
        }
        Ok(Self { vis, name, args, content, sub_var_refs, var_context: var_context.clone() })
    }
}

pub struct StyleDefinition<T: StyleSheetConstructor> {
    pub extern_vis: Option<Visibility>,
    pub name: VarName,
    pub args: Vec<(VarName, ArgType)>,
    pub content: Vec<StyleContentItem<T::PropertyValue>>,
    pub sub_var_refs: Vec<VarRef>,
}

impl<T: StyleSheetConstructor> StyleDefinition<T> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
        attrs: Vec<Attribute>,
        extern_vis: Option<Visibility>,
    ) -> Result<Self, syn::Error> {
        input.parse::<kw::style>()?;
        let name: VarName = input.parse()?;
        for attr in attrs {
            return Err(syn::Error::new_spanned(attr, "unknown attribute"));
        }
        let args = StyleFnDefinition::<T>::parse_arg_list(input, scope)?;
        let (content, sub_var_refs) = StyleFnDefinition::<T>::parse_fn_body(input, scope, &args)?;
        if scope.vars.insert(name.clone(), ScopeVarValue::StyleDefinition(args.clone())).is_some() {
            return Err(syn::Error::new(name.span(), "duplicated identifier"));
        }
        Ok(Self { extern_vis, name, args, content, sub_var_refs })
    }
}

pub struct ClassDefinition<T: StyleSheetConstructor> {
    pub extern_vis: Option<Visibility>,
    pub error_css_output: Option<Span>,
    pub css_name: Option<String>,
    pub name: VarName,
    pub content: RuleContent<T>,
    pub sub_var_refs: Vec<VarRef>,
}

impl<T: StyleSheetConstructor> ClassDefinition<T> {
    fn parse_with_vars(
        input: syn::parse::ParseStream,
        scope: &mut ScopeVars,
        attrs: Vec<Attribute>,
        extern_vis: Option<Visibility>,
    ) -> Result<Self, syn::Error> {
        input.parse::<kw::class>()?;
            let name = input.parse()?;
            let mut error_css_output = None;
            let mut css_name = None;
            for attr in attrs {
                if attr.path.is_ident("error_css_output") {
                    if !attr.tokens.is_empty() {
                        return Err(syn::Error::new_spanned(attr.tokens, "unknown attribute arguments"));
                    }
                    error_css_output = Some(attr.path.span());
                } else if attr.path.is_ident("css_name") {
                    let name = syn::parse2::<Paren<syn::LitStr>>(attr.tokens)?;
                    css_name = Some(name.inner.value());
                } else {
                    return Err(syn::Error::new_spanned(attr, "unknown attribute"));
                }
            }
            let var_refs = std::mem::replace(&mut scope.var_refs, vec![]);
            let content = try_parse_brace(input, |input| {
                RuleContent::parse_with_vars(input, scope, false)
            })?;
            let sub_var_refs = std::mem::replace(&mut scope.var_refs, var_refs);
            Ok(Self { extern_vis, error_css_output, css_name, name, content, sub_var_refs })
    }
}

#[derive(Debug, Clone)]
pub enum StyleContentItem<V: ParseStyleSheetValue> {
    CompilationError(syn::Error),
    Property(Property<V>),
    StyleRef(VarName, Vec<MaybeDyn<VarDynValue>>),
}

impl<V: ParseStyleSheetValue> StyleContentItem<V> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
        parse_to_end: bool,
    ) -> Result<Vec<Self>, syn::Error> {
        let mut compilation_errors = vec![];
        let mut items = vec![];
        while !input.is_empty() {
            if !parse_to_end && !input.peek(Ident) {
                break
            }
            if input.peek2(Token![=]) {
                match Property::parse_with_vars(input, scope) {
                    Ok(prop) => {
                        items.push(Self::Property(prop));
                    }
                    Err(err) => {
                        compilation_errors.push(err);
                    }
                }
            } else if input.peek2(syn::token::Paren) {
                let v: VarName = input.parse()?;
                if let Some(x) = scope.vars.get(&v).cloned() {
                    if let ScopeVarValue::StyleDefinition(args) = x {
                        scope.var_refs.push(v.clone().into_ref());
                        let var_dyn_values = try_parse_paren(input, |input| {
                            let mut var_dyn_values = vec![];
                            let mut args_iter = args.into_iter();
                            while !input.is_empty() {
                                let token: CssToken = ParseWithVars::parse_with_vars(input, scope)?;
                                let v = if let Some((_, ty)) = args_iter.next() {
                                    match ty {
                                        ArgType::Str(_) => match token {
                                            CssToken::String(s) => match s.s {
                                                MaybeDyn::Static(x) => MaybeDyn::Static(VarDynValue {
                                                    span: s.span,
                                                    kind: VarDynValueKind::Str(x),
                                                }),
                                                MaybeDyn::Dyn(x) => MaybeDyn::Dyn(x),
                                            },
                                            _ => {
                                                return Err(syn::Error::new(token.span(), "expected &str"));
                                            }
                                        },
                                        ArgType::Num(_) => match token {
                                            CssToken::Number(s) => match s.value {
                                                MaybeDyn::Static(x) => MaybeDyn::Static(VarDynValue {
                                                    span: s.span,
                                                    kind: VarDynValueKind::Num(x),
                                                }),
                                                MaybeDyn::Dyn(x) => MaybeDyn::Dyn(x),
                                            },
                                            _ => {
                                                return Err(syn::Error::new(token.span(), "expected {number}"));
                                            }
                                        }
                                    }
                                } else {
                                    return Err(syn::Error::new(token.span(), "unnecessary argument"));
                                };
                                var_dyn_values.push(v);
                                if input.is_empty() {
                                    break;
                                }
                                input.parse::<Token![,]>()?;
                            }
                            if args_iter.next().is_some() {
                                return Err(input.error("arguments not enough"));
                            }
                            Ok(var_dyn_values)
                        })?;
                        input.parse::<Token![;]>()?;
                        items.push(Self::StyleRef(v, var_dyn_values));
                    } else {
                        return Err(syn::Error::new_spanned(&v.ident, format!("expected StyleDefinition, found {}", x.type_name())));
                    }
                } else {
                    return Err(syn::Error::new_spanned(&v.ident, "variable not declared"));
                }
            } else {
                input.parse::<Ident>()?;
                return Err(input.error("expected `=` (as property) or `(...)` (as style reference)"));
            }
        }
        Ok(items)
    }
}

pub struct RuleContent<T: StyleSheetConstructor> {
    pub items: Vec<StyleContentItem<T::PropertyValue>>,
    pub at_blocks: Vec<AtBlock<T>>,
    pub pseudo_classes: Vec<PseudoClass<T>>,
}

impl<T: StyleSheetConstructor> RuleContent<T> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
        inside_sub_rule: bool,
    ) -> Result<Self, syn::Error> {
        let items = StyleContentItem::parse_with_vars(input, scope, false)?;
        if !input.is_empty() && !input.peek(Token![if]) {
            return Err(input.error("expected property, style reference, or `if` conditions"));
        }
        let mut at_blocks = vec![];
        let mut pseudo_classes = vec![];
        while !input.is_empty() {
            input.parse::<Token![if]>()?;
            if input.peek(kw::media) {
                input.parse::<kw::media>()?;
                let mut expr = vec![];
                loop {
                    expr.push(ParseWithVars::parse_with_vars(input, scope)?);
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                        continue;
                    }
                    break;
                }
                let content = {
                    let content;
                    braced!(content in input);
                    RuleContent::parse_with_vars(&content, scope, true)?
                };
                if pseudo_classes.len() > 0 {
                    return Err(input.error("media conditions should be put before pseudo conditions"));
                } else {
                    at_blocks.push(AtBlock::Media {
                        expr,
                        content,
                    })
                }
            } else if input.peek(kw::supports) {
                input.parse::<kw::supports>()?;
                let expr = ParseWithVars::parse_with_vars(input, scope)?;
                let content = {
                    let content;
                    braced!(content in input);
                    RuleContent::parse_with_vars(&content, scope, true)?
                };
                if pseudo_classes.len() > 0 {
                    return Err(input.error("media conditions should be put before pseudo conditions"));
                } else {
                    at_blocks.push(AtBlock::Supports {
                        expr,
                        content,
                    })
                }
            } else {
                let p = ParseWithVars::parse_with_vars(input, scope)?;
                if inside_sub_rule {
                    return Err(input.error("pseudo conditions should not be put inside other conditions"));
                } else {
                    pseudo_classes.push(p);
                }
            }
        }
        Ok(Self {
            items,
            at_blocks,
            pseudo_classes,
        })
    }
}

/// A CSS property (name-value pair)
#[derive(Debug, Clone)]
pub struct Property<V> {
    pub name: CssIdent,
    pub value: V,
}

impl<V: ParseStyleSheetValue> Property<V> {
    fn parse_value(
        input: ParseStream,
        scope: &mut ScopeVars,
        name: &CssIdent,
    ) -> Result<V, syn::Error> {
        let mut tokens = VecDeque::new();
        while !input.is_empty() && !input.peek(Token![;]) {
            let token: CssToken = ParseWithVars::parse_with_vars(input, scope)?;
            tokens.push_back(token);
        }
        let mut tokens = CssTokenStream::new(input.span(), tokens);
        let value = V::parse_value(&name, &mut tokens).map_err(|err| err.into_syn_error())?;
        tokens.expect_ended().map_err(|err| err.into_syn_error())?;
        Ok(value)
    }
}

impl<V: WriteCss> WriteCss for Property<V> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::fmt::Result {
        self.name.write_css_with_args(cssw, values)?;
        cssw.write_colon()?;
        self.value.write_css_with_args(cssw, values)?;
        cssw.write_semi()?;
        Ok(())
    }
}

impl<V: ParseStyleSheetValue> ParseWithVars for Property<V> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        try_parse_until_semi(input, |input| {
            let name: CssIdent = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = Self::parse_value(input, scope, &name)?;
            Ok(Self { name, value })
        })
    }
}

/// A CSS at-rule inside a class
pub enum AtBlock<T: StyleSheetConstructor> {
    Media {
        expr: Vec<MediaQuery<T::MediaCondValue>>,
        content: RuleContent<T>,
    },
    Supports {
        expr: SupportsQuery<T::PropertyValue>,
        content: RuleContent<T>,
    },
}

pub struct MediaQuery<V> {
    pub only: Option<CssIdent>,
    pub media_type: MediaType,
    pub cond_list: Vec<MediaCond<V>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    All,
    Screen,
    Print,
}

pub struct MediaCond<V> {
    pub not: Option<CssIdent>,
    pub name: CssIdent,
    pub cond: V,
}

impl<V: ParseStyleSheetValue> ParseWithVars for MediaQuery<V> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let only = if input.peek(kw::only) {
            Some(input.parse()?)
        } else {
            None
        };
        let (media_type, has_media_feature) = {
            let need_media_type = if only.is_some() {
                true
            } else {
                input.peek(Ident) && !input.peek(kw::not)
            };
            if need_media_type {
                let ident: CssIdent = input.parse()?;
                let media_type = match ident.formal_name.as_str() {
                    "all" => MediaType::All,
                    "screen" => MediaType::Screen,
                    "print" => MediaType::Print,
                    _ => {
                        return Err(syn::Error::new(ident.span, "unknown media type"));
                    }
                };
                let has_media_feature = input.peek(kw::and);
                if has_media_feature {
                    input.parse::<kw::and>()?;
                }
                (media_type, has_media_feature)
            } else {
                (MediaType::All, true)
            }
        };
        let mut cond_list = vec![];
        if has_media_feature {
            loop {
                let not = if input.peek(kw::not) {
                    Some(input.parse()?)
                } else {
                    None
                };
                let cond = {
                    let content;
                    parenthesized!(content in input);
                    let input = &content;
                    let name = input.parse()?;
                    input.parse::<Token![=]>()?;
                    let cond = Property::<V>::parse_value(input, scope, &name)?;
                    MediaCond {
                        not,
                        name,
                        cond,
                    }
                };
                cond_list.push(cond);
                if input.peek(kw::and) {
                    input.parse::<kw::and>()?;
                } else {
                    break;
                }
            }
        }
        Ok(MediaQuery {
            only,
            media_type,
            cond_list,
        })
    }
}

impl<V: WriteCss> WriteCss for MediaQuery<V> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::fmt::Result {
        self.only.write_css_with_args(cssw, values)?;
        let mut need_and = match self.media_type {
            MediaType::All => {
                if self.only.is_some() || self.cond_list.is_empty() {
                    cssw.write_ident("all", true)?;
                    true
                } else {
                    false
                }
            }
            MediaType::Print => {
                cssw.write_ident("print", true)?;
                true
            }
            MediaType::Screen => {
                cssw.write_ident("screen", true)?;
                true
            }
        };
        for item in self.cond_list.iter() {
            if need_and {
                cssw.write_ident("and", true)?;
            } else {
                need_and = true;
            }
            item.not.write_css_with_args(cssw, values)?;
            cssw.write_paren_block(|cssw| {
                item.name.write_css_with_args(cssw, values)?;
                cssw.write_colon()?;
                item.cond.write_css_with_args(cssw, values)?;
                Ok(())
            })?;
        }
        Ok(())
    }
}

pub enum SupportsQuery<V> {
    Cond(SupportsCond<V>),
    And(Vec<CssParen<SupportsQuery<V>>>),
    Or(Vec<CssParen<SupportsQuery<V>>>),
    Not(Box<CssParen<SupportsQuery<V>>>),
    Sub(Box<CssParen<SupportsQuery<V>>>),
}

pub struct SupportsCond<V> {
    pub name: CssIdent,
    pub value: V,
}

impl<V: ParseStyleSheetValue> ParseWithVars for SupportsQuery<V> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let la = input.lookahead1();
        let ret = if la.peek(kw::not) {
            input.parse::<kw::not>()?;
            let item: CssParen<SupportsQuery<V>> =
                ParseWithVars::parse_with_vars(input, scope)?;
            if let Self::Sub(item) = item.block {
                Self::Not(item)
            } else {
                Self::Not(Box::new(item))
            }
        } else if la.peek(syn::token::Paren) {
            let first: CssParen<SupportsQuery<V>> =
                ParseWithVars::parse_with_vars(input, scope)?;
            let next_is_and = input.peek(kw::and);
            let next_is_or = input.peek(kw::or);
            if next_is_and || next_is_or {
                if next_is_and { input.parse::<kw::and>()?; }
                if next_is_or { input.parse::<kw::or>()?; }
                let mut list = vec![if let Self::Sub(item) = first.block {
                    *item
                } else {
                    first
                }];
                loop {
                    let item: CssParen<SupportsQuery<V>> =
                        ParseWithVars::parse_with_vars(input, scope)?;
                    if let Self::Sub(item) = item.block {
                        list.push(*item);
                    } else {
                        list.push(item);
                    }
                    if next_is_and && input.peek(kw::and) {
                        // empty
                    } else if next_is_or && input.peek(kw::or) {
                        // empty
                    } else {
                        break;
                    }
                }
                if next_is_and {
                    Self::And(list)
                } else {
                    Self::Or(list)
                }
            } else {
                if let Self::Sub(item) = first.block {
                    Self::Sub(item)
                } else {
                    Self::Sub(Box::new(first))
                }
            }
        } else if la.peek(Ident) {
            let name = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = Property::<V>::parse_value(input, scope, &name)?;
            Self::Cond(SupportsCond {
                name,
                value,
            })
        } else {
            return Err(la.error());
        };
        Ok(ret)
    }
}

impl<V: WriteCss> WriteCss for SupportsQuery<V> {
    fn write_css_with_args<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::fmt::Result {
        match self {
            Self::Cond(cond) => {
                cond.name.write_css_with_args(cssw, values)?;
                cssw.write_colon()?;
                cond.value.write_css_with_args(cssw, values)?;
            }
            Self::And(list) => {
                for (index, item) in list.iter().enumerate() {
                    if index > 0 {
                        cssw.write_ident("and", true)?;
                    }
                    item.write_css_with_args(cssw, values)?;
                }
            }
            Self::Or(list) => {
                for (index, item) in list.iter().enumerate() {
                    if index > 0 {
                        cssw.write_ident("or", true)?;
                    }
                    item.write_css_with_args(cssw, values)?;
                }
            }
            Self::Not(item) => {
                cssw.write_ident("not", true)?;
                item.write_css_with_args(cssw, values)?;
            }
            Self::Sub(item) => {
                item.write_css_with_args(cssw, values)?;
            }
        }
        Ok(())
    }
}

pub struct PseudoClass<T: StyleSheetConstructor> {
    pub pseudo: crate::pseudo::Pseudo,
    pub content: RuleContent<T>,
}

impl<T: StyleSheetConstructor> ParseWithVars for PseudoClass<T> {
    fn parse_with_vars(
        input: ParseStream,
        scope: &mut ScopeVars,
    ) -> Result<Self, syn::Error> {
        let pseudo = ParseWithVars::parse_with_vars(input, scope)?;
        try_parse_brace(input, |input| {
            let content = RuleContent::parse_with_vars(input, scope, true)?;
            Ok(Self { pseudo, content })
        })
    }
}
