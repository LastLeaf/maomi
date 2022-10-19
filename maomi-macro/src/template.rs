use std::collections::HashMap;
use proc_macro2::TokenStream;
use quote::*;
use syn::parse::*;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;

use crate::i18n::{LocaleGroup, TransRes};

fn get_branch_ty(len: usize) -> Ident {
    Ident::new(&format!("Branch{}", len), proc_macro2::Span::call_site())
}

fn get_branch_selected(index: usize) -> Ident {
    Ident::new(&format!("B{}", index), proc_macro2::Span::call_site())
}

pub(super) struct Template {
    children: Vec<TemplateNode>,
}

impl Template {
    pub(super) fn gen_type(&self, md: &MacroDelimiter) -> Type {
        let Self { children } = self;
        let span = match md {
            MacroDelimiter::Paren(x) => x.span,
            MacroDelimiter::Brace(x) => x.span,
            MacroDelimiter::Bracket(x) => x.span,
        };
        let children = children.iter().map(|c| c.gen_type());
        parse_quote_spanned!(span=> (#(#children,)*) )
    }

    pub(super) fn to_create<'a>(&'a self, backend_param: &'a TokenStream, locale_group: &'a LocaleGroup) -> TemplateCreate<'a> {
        TemplateCreate {
            template: self,
            backend_param,
            locale_group,
        }
    }

    pub(super) fn to_update<'a>(&'a self, backend_param: &'a TokenStream, locale_group: &'a LocaleGroup) -> TemplateUpdate<'a> {
        TemplateUpdate {
            template: self,
            backend_param,
            locale_group,
        }
    }
}

impl Parse for Template {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = vec![];
        while !input.is_empty() {
            let child = input.parse()?;
            children.push(child);
        }
        Ok(Self { children })
    }
}

pub(super) enum TemplateNode {
    StaticText {
        content: LitStr,
    },
    DynamicText {
        brace_token: token::Brace,
        expr: Box<Expr>,
    },
    Slot {
        tag_lt_token: token::Lt,
        tag_name: Path,
        data: Option<TemplateAttribute>,
        #[allow(dead_code)]
        tag_gt_token: token::Gt,
        #[allow(dead_code)]
        close_token: token::Div,
    },
    Tag {
        tag_lt_token: token::Lt,
        tag_name: Path,
        slot_var_name: Option<Ident>,
        attrs: Vec<TemplateAttribute>,
        #[allow(dead_code)]
        tag_gt_token: token::Gt,
        children: Vec<TemplateNode>,
        #[allow(dead_code)]
        close_token: token::Div,
    },
    IfElse {
        branches: Vec<TemplateIfElse>,
    },
    Match {
        match_token: token::Match,
        expr: Box<Expr>,
        #[allow(dead_code)]
        brace_token: token::Brace,
        arms: Vec<TemplateMatchArm>,
    },
    ForLoop {
        for_token: token::For,
        pat: Pat,
        in_token: token::In,
        expr: Box<Expr>,
        key: Option<(token::Use, Option<token::Paren>, Box<Expr>, Path)>,
        brace_token: token::Brace,
        children: Vec<TemplateNode>,
    },
}

pub(super) struct TemplateIfElse {
    else_token: Option<token::Else>,
    if_cond: Option<(token::If, Box<Expr>)>,
    brace_token: token::Brace,
    children: Vec<TemplateNode>,
}

pub(super) struct TemplateMatchArm {
    pat: Pat,
    guard: Option<(token::If, Expr)>,
    fat_arrow_token: token::FatArrow,
    brace_token: token::Brace,
    children: Vec<TemplateNode>,
    comma: Option<token::Comma>,
}
impl TemplateNode {
    fn gen_type(&self) -> Type {
        match self {
            Self::StaticText { content } => {
                let span = content.span();
                parse_quote_spanned!(span=> maomi::text_node::TextNode )
            }
            Self::DynamicText { brace_token, .. } => {
                let span = brace_token.span;
                parse_quote_spanned!(span=> maomi::text_node::TextNode )
            }
            Self::Slot { tag_name, .. } => {
                let span = tag_name.span();
                parse_quote_spanned!(span=> maomi::node::ControlNode<()> )
            }
            Self::Tag {
                tag_name, children, ..
            } => {
                let span = tag_name.span();
                let children = children.iter().map(|c| c.gen_type());
                parse_quote_spanned!(span=> maomi::node::Node<#tag_name, (#(#children,)*)> )
            }
            Self::IfElse { branches } => {
                let branch_ty = get_branch_ty(branches.len());
                let branches = branches.iter().map(|x| {
                    let span = x.brace_token.span;
                    let children = x.children.iter().map(|c| c.gen_type());
                    quote_spanned!(span=> (#(#children,)*) )
                });
                parse_quote!(maomi::node::ControlNode<maomi::node::#branch_ty<#(#branches),*>> )
            }
            Self::Match { arms, .. } => {
                let branch_ty = get_branch_ty(arms.len());
                let branches = arms.iter().map(|x| {
                    let span = x.brace_token.span;
                    let children = x.children.iter().map(|c| c.gen_type());
                    quote_spanned!(span=> (#(#children,)*) )
                });
                parse_quote!(maomi::node::ControlNode<maomi::node::#branch_ty<#(#branches),*>> )
            }
            Self::ForLoop {
                brace_token,
                children,
                key,
                ..
            } => {
                let span = brace_token.span;
                let children = children.iter().map(|c| c.gen_type());
                let ty = if let Some((_, _, _, key_ty)) = key.as_ref() {
                    quote!(maomi::diff::key::KeyList<#key_ty, (#(#children,)*)>)
                } else {
                    quote!(maomi::diff::keyless::KeylessList<(#(#children,)*)>)
                };
                parse_quote_spanned!(span=> maomi::node::ControlNode<#ty> )
            }
        }
    }
}

impl Parse for TemplateNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let ret = if la.peek(LitStr) {
            // parse static text node
            TemplateNode::StaticText {
                content: input.parse()?,
            }
        } else if la.peek(token::Brace) {
            // parse dynamic text node
            let content;
            let brace_token = braced!(content in input);
            let expr = content.parse()?;
            TemplateNode::DynamicText { brace_token, expr }
        } else if la.peek(token::Lt) {
            let tag_lt_token = input.parse()?;
            let tag_name: Path = input.parse()?;
            if tag_name.is_ident("slot") {
                // parse slot tag
                let data = if input.peek(Ident) {
                    let attr: TemplateAttribute = input.parse()?;
                    match &attr {
                        TemplateAttribute::StaticProperty {
                            name, list_updater, ..
                        }
                        | TemplateAttribute::DynamicProperty {
                            name, list_updater, ..
                        } => {
                            if name.to_string().as_str() != "data" {
                                Err(Error::new(
                                    name.span(),
                                    "The slot element cannot contain attributes other than `data`",
                                ))?;
                            }
                            if list_updater.is_some() {
                                Err(Error::new(name.span(), "Illegal slot `data` attribute"))?;
                            }
                        }
                        TemplateAttribute::EventHandler { name, .. }
                        | TemplateAttribute::Slot { name, .. } => {
                            Err(Error::new(
                                name.span(),
                                "Illegal slot element attribute value",
                            ))?;
                        }
                    }
                    Some(attr)
                } else {
                    None
                };
                let la = input.lookahead1();
                if la.peek(token::Div) {
                    let close_token = input.parse()?;
                    let tag_gt_token = input.parse()?;
                    TemplateNode::Slot {
                        tag_lt_token,
                        tag_name,
                        data,
                        tag_gt_token,
                        close_token,
                    }
                } else if la.peek(token::Gt) {
                    let tag_gt_token = input.parse()?;
                    let _: token::Lt = input.parse()?;
                    let close_token = input.parse()?;
                    if input.parse::<Token![_]>().is_err() {
                        let end_tag_name: Path = input.parse()?;
                        if !end_tag_name.is_ident("slot") {
                            return Err(Error::new(
                                end_tag_name.span(),
                                "End tag name does not match the start tag name (consider use `_` instead?)",
                            ));
                        }
                    }
                    let _: token::Gt = input.parse()?;
                    TemplateNode::Slot {
                        tag_lt_token,
                        tag_name,
                        data,
                        tag_gt_token,
                        close_token,
                    }
                } else {
                    return Err(la.error());
                }
            } else {
                // parse element
                let mut slot_var_name = None;
                let mut attrs = vec![];
                let mut la = input.lookahead1();
                loop {
                    if la.peek(Ident) {
                        let attr = input.parse()?;
                        if let TemplateAttribute::Slot { name, var_name, .. } = attr {
                            if slot_var_name.is_some() {
                                return Err(Error::new(name.span(), "Duplicated `slot` attribute"));
                            }
                            slot_var_name = Some(var_name);
                        } else {
                            attrs.push(attr);
                        }
                        la = input.lookahead1();
                    } else {
                        break;
                    }
                }
                if la.peek(token::Div) {
                    let close_token = input.parse()?;
                    let tag_gt_token = input.parse()?;
                    TemplateNode::Tag {
                        tag_lt_token,
                        tag_name,
                        slot_var_name,
                        attrs,
                        tag_gt_token,
                        children: Vec::with_capacity(0),
                        close_token,
                    }
                } else if la.peek(token::Gt) {
                    let tag_gt_token = input.parse()?;
                    let mut children = vec![];
                    while !input.peek(token::Lt) || !input.peek2(token::Div) {
                        let child = input.parse()?;
                        children.push(child);
                    }
                    let _: token::Lt = input.parse()?;
                    let close_token = input.parse()?;
                    if input.parse::<Token![_]>().is_err() {
                        let end_tag_name: Path = input.parse()?;
                        let short_tag_name = match tag_name.segments.last() {
                            None => Ident::new("", tag_name.span()),
                            Some(x) => x.ident.clone(),
                        };
                        if !end_tag_name.is_ident(&short_tag_name) {
                            return Err(Error::new(
                                end_tag_name.span(),
                                "End tag name does not match the start tag name (consider use `_` instead?)",
                            ));
                        }
                    }
                    let _: token::Gt = input.parse()?;
                    TemplateNode::Tag {
                        tag_lt_token,
                        tag_name,
                        slot_var_name,
                        attrs,
                        tag_gt_token,
                        children,
                        close_token,
                    }
                } else {
                    return Err(la.error());
                }
            }
        } else if la.peek(token::If) {
            // parse if expr
            let mut branches = vec![];
            let mut else_token = None;
            loop {
                let if_cond = if input.peek(token::If) {
                    Some((
                        input.parse()?,
                        Box::new(Expr::parse_without_eager_brace(input)?),
                    ))
                } else {
                    None
                };
                let has_if = if_cond.is_some();
                let content;
                let brace_token = braced!(content in input);
                let mut children = vec![];
                while !content.is_empty() {
                    children.push(content.parse()?);
                }
                if branches.len() >= 16 {
                    Err(Error::new(
                        brace_token.span,
                        "`if` and `else` group cannot contain more than 16 branches",
                    ))?;
                }
                branches.push(TemplateIfElse {
                    else_token,
                    if_cond,
                    brace_token,
                    children,
                });
                if input.peek(token::Else) {
                    else_token = Some(input.parse()?);
                } else {
                    // add an else branch if it is not ended with
                    if has_if {
                        branches.push(TemplateIfElse {
                            else_token: Some(Default::default()),
                            if_cond: None,
                            brace_token: Default::default(),
                            children: vec![],
                        })
                    }
                    break;
                }
            }
            TemplateNode::IfElse { branches }
        } else if la.peek(token::Match) {
            // parse match expr
            let match_token = input.parse()?;
            let expr = Box::new(Expr::parse_without_eager_brace(input)?);
            let content;
            let brace_token = braced!(content in input);
            let mut arms = vec![];
            {
                let input = content;
                while !input.is_empty() {
                    let pat = input.parse()?;
                    let guard = if input.peek(token::If) {
                        Some((input.parse()?, input.parse()?))
                    } else {
                        None
                    };
                    let fat_arrow_token = input.parse()?;
                    let content;
                    let brace_token = braced!(content in input);
                    let mut children = vec![];
                    while !content.is_empty() {
                        children.push(content.parse()?);
                    }
                    let comma = input.parse()?;
                    if arms.len() >= 16 {
                        Err(Error::new(
                            brace_token.span,
                            "`match` cannot contain more than 16 branches",
                        ))?;
                    }
                    arms.push(TemplateMatchArm {
                        pat,
                        guard,
                        fat_arrow_token,
                        brace_token,
                        children,
                        comma,
                    })
                }
            }
            TemplateNode::Match {
                match_token,
                expr,
                brace_token,
                arms,
            }
        } else if la.peek(token::For) {
            // parse for expr
            let for_token = input.parse()?;
            let pat = input.parse()?;
            let in_token = input.parse()?;
            let expr = Box::new(Expr::parse_without_eager_brace(input)?);
            let key = if input.peek(token::Use) {
                let use_token = input.parse()?;
                let la = input.lookahead1();
                let (paren, key_expr) = if la.peek(token::Paren) {
                    let content;
                    let paren = parenthesized!(content in input);
                    (Some(paren), content.parse()?)
                } else if let Pat::Ident(x) = &pat {
                    let ident = &x.ident;
                    let span = ident.span();
                    (None, parse_quote_spanned! {span=> #ident })
                } else {
                    return Err(la.error());
                };
                let path = input.parse()?;
                Some((use_token, paren, key_expr, path))
            } else {
                None
            };
            let content;
            let brace_token = braced!(content in input);
            let mut children = vec![];
            while !content.is_empty() {
                children.push(content.parse()?);
            }
            TemplateNode::ForLoop {
                for_token,
                pat,
                in_token,
                expr,
                key,
                brace_token,
                children,
            }
        } else {
            return Err(la.error());
        };
        Ok(ret)
    }
}

pub(super) enum TemplateAttribute {
    StaticProperty {
        name: Ident,
        list_updater: Option<(token::Colon, Path)>,
        #[allow(dead_code)]
        eq_token: token::Eq,
        value: Lit,
    },
    DynamicProperty {
        name: Ident,
        list_updater: Option<(token::Colon, Path)>,
        #[allow(dead_code)]
        eq_token: token::Eq,
        ref_token: Option<token::And>,
        #[allow(dead_code)]
        brace_token: token::Brace,
        expr: Box<Expr>,
    },
    EventHandler {
        name: Ident,
        #[allow(dead_code)]
        eq_token: token::Eq,
        at_token: token::At,
        fn_name: Ident,
        #[allow(dead_code)]
        paren_token: token::Paren,
        args: Punctuated<Box<Expr>, token::Comma>,
    },
    Slot {
        name: Ident,
        #[allow(dead_code)]
        colon_token: token::Colon,
        var_name: Ident,
    },
}

impl TemplateAttribute {
    fn list_ident(&self) -> Option<&Ident> {
        match self {
            Self::StaticProperty {
                name, list_updater, ..
            } => list_updater.as_ref().map(|_| name),
            Self::DynamicProperty {
                name, list_updater, ..
            } => list_updater.as_ref().map(|_| name),
            Self::EventHandler { .. } => None,
            Self::Slot { .. } => None,
        }
    }
}

impl Parse for TemplateAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let ret = if la.peek(Ident) {
            let name: Ident = input.parse()?;
            if name.to_string().as_str() == "slot" {
                TemplateAttribute::Slot {
                    name,
                    colon_token: input.parse()?,
                    var_name: input.parse()?,
                }
            } else {
                let list_updater = if input.peek(token::Colon) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                };
                if input.peek(token::Eq) {
                    let eq_token = input.parse()?;
                    let la = input.lookahead1();
                    if la.peek(Lit) {
                        let value = input.parse()?;
                        TemplateAttribute::StaticProperty {
                            name,
                            list_updater,
                            eq_token,
                            value,
                        }
                    } else if la.peek(token::And) {
                        let ref_token = input.parse()?;
                        let content;
                        let brace_token = braced!(content in input);
                        let expr = content.parse()?;
                        TemplateAttribute::DynamicProperty {
                            ref_token: Some(ref_token),
                            name,
                            list_updater,
                            eq_token,
                            brace_token,
                            expr,
                        }
                    } else if la.peek(token::Brace) {
                        let content;
                        let brace_token = braced!(content in input);
                        let expr = content.parse()?;
                        TemplateAttribute::DynamicProperty {
                            ref_token: None,
                            name,
                            list_updater,
                            eq_token,
                            brace_token,
                            expr,
                        }
                    } else if la.peek(token::At) {
                        let at_token = input.parse()?;
                        let fn_name = input.parse()?;
                        let content;
                        let paren_token = parenthesized!(content in input);
                        let args = Punctuated::parse_terminated(&content)?;
                        TemplateAttribute::EventHandler {
                            name,
                            eq_token,
                            at_token,
                            fn_name,
                            paren_token,
                            args,
                        }
                    } else {
                        return Err(la.error());
                    }
                } else {
                    let span = name.span();
                    TemplateAttribute::StaticProperty {
                        name,
                        list_updater,
                        eq_token: parse_quote_spanned! {span=> = },
                        value: parse_quote_spanned! {span=> true },
                    }
                }
            }
        } else {
            return Err(la.error());
        };
        Ok(ret)
    }
}

pub(super) struct TemplateCreate<'a> {
    template: &'a Template,
    backend_param: &'a TokenStream,
    locale_group: &'a LocaleGroup,
}

impl<'a> ToTokens for TemplateCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            template,
            backend_param,
            locale_group,
        } = self;
        let Template { children } = template;
        let children = children.into_iter().map(|x| TemplateNodeCreate {
            inside_update: false,
            template_node: x,
            backend_param,
            locale_group,
        });
        quote! {
            (#({#children},)*)
        }
        .to_tokens(tokens)
    }
}

struct TemplateNodeCreate<'a> {
    inside_update: bool,
    template_node: &'a TemplateNode,
    backend_param: &'a TokenStream,
    locale_group: &'a LocaleGroup,
}

impl<'a> ToTokens for TemplateNodeCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            inside_update,
            template_node,
            backend_param,
            locale_group,
        } = self;
        let inside_update = *inside_update;
        match template_node {
            TemplateNode::StaticText { content } => {
                let span = content.span();
                let translated = match locale_group.trans(&content.value()) {
                    TransRes::LackTrans => quote_spanned! {span=> compile_error!("lacks translation") },
                    TransRes::Done(x) => {
                        let s = LitStr::new(x, span);
                        quote! { maomi::locale_string::LocaleStaticStr::translated(#s) }
                    }
                    TransRes::NotNeeded => quote_spanned! {span=> maomi::locale_string::LocaleStaticStr::translated(#content) },
                };
                quote_spanned! {span=>
                    let (__m_child, __m_backend_element) =
                        maomi::text_node::TextNode::create::<#backend_param>(
                            __m_parent_element,
                            #translated,
                        )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    __m_child
                }
            }
            TemplateNode::DynamicText { brace_token, expr } => {
                let span = brace_token.span;
                let translated = match locale_group.need_trans() {
                    true => quote_spanned! {span=> #expr },
                    false => quote_spanned! {span=> maomi::locale_string::LocaleString::translated(#expr) },
                };
                quote_spanned! {span=>
                    let (__m_child, __m_backend_element) =
                        maomi::text_node::TextNode::create::<#backend_param>(
                            __m_parent_element,
                            #translated,
                        )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    __m_child
                }
            }
            TemplateNode::Slot { tag_lt_token, data, .. } => {
                let span = tag_lt_token.span();
                let data_expr = match data {
                    None => quote_spanned! {span=> &() },
                    Some(attr) => {
                        match attr {
                            TemplateAttribute::StaticProperty { value, .. } => {
                                let span = value.span();
                                match value {
                                    Lit::Str(_) | Lit::ByteStr(_) => quote! {span=> #value },
                                    _ => quote_spanned! {span=> & #value },
                                }
                            }
                            TemplateAttribute::DynamicProperty { expr, ref_token, .. } => {
                                let span = expr.span();
                                match ref_token {
                                    Some(ref_sign) => quote_spanned!(span=> #ref_sign(#expr)),
                                    None => quote_spanned!(span=> #expr),
                                }
                            }
                            TemplateAttribute::EventHandler { .. } => unreachable!(),
                            TemplateAttribute::Slot { .. } => unreachable!(),
                        }
                    },
                };
                let create_slot = if inside_update {
                    quote! { __m_slot_fn(maomi::node::SlotChange::Added(__m_parent_element, &__m_backend_element_token, &__m_slot_data))?; }
                } else {
                    quote! { __m_slot_fn(__m_parent_element, &__m_backend_element_token, &__m_slot_data)?; }
                };
                quote_spanned! {span=>
                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                    {
                        let __m_backend_element_token = __m_backend_element.token();
                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                        let __m_slot_data = maomi::prop::Prop::new(maomi::prop::PropAsRef::property_to_owned(#data_expr));
                        #create_slot
                        __m_slot_scopes.add(__m_backend_element_token.stable_addr(), (__m_backend_element_token, __m_slot_data));
                    }
                    let __m_backend_element_token = __m_backend_element.token();
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    maomi::node::ControlNode::new(
                        __m_backend_element_token,
                        (),
                    )
                }
            }
            TemplateNode::Tag { tag_lt_token, tag_name, slot_var_name, attrs, children, .. } => {
                let span = tag_lt_token.span();
                let mut list_prop_count = HashMap::new();
                let attrs = attrs.into_iter().map(|attr| {
                    let list_index = if let Some(x) = attr.list_ident() {
                        *list_prop_count.entry(x)
                            .and_modify(|x| *x += 1)
                            .or_insert(1) - 1
                    } else {
                        0
                    };
                    TemplateAttributeCreate { attr, list_index }
                }).collect::<Vec<_>>();
                let (list_prop_name, list_prop_count): (Vec<&Ident>, Vec<usize>) = list_prop_count.iter().unzip();
                let attrs_list_init = quote_spanned! {span=>
                    #(
                        maomi::prop::ListPropertyInit::init_list(
                            &mut __m_child.#list_prop_name,
                            #list_prop_count,
                            __m_update_ctx,
                        );
                    )*
                };
                let children = children.into_iter().map(|x| TemplateNodeCreate { inside_update, template_node: x, backend_param, locale_group });
                let slot_var_name = match slot_var_name {
                    Some(x) => quote! { #x },
                    None => quote! { __m_slot_data },
                };
                quote_spanned! {span=>
                    let (mut __m_child, __m_backend_element) =
                        <<#tag_name as maomi::backend::SupportBackend>::Target as maomi::backend::BackendComponent<#backend_param>>::init(
                            __m_backend_context,
                            __m_parent_element,
                            __m_self_owner_weak,
                        )?;
                    let mut __m_slot_children = maomi::node::SlotChildren::None;
                    <<#tag_name as maomi::backend::SupportBackend>::Target as maomi::backend::BackendComponent<#backend_param>>::create(
                        &mut __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        #[inline] |__m_child, __m_update_ctx| {
                            #attrs_list_init
                            #(#attrs)*
                        },
                        #[inline] |__m_parent_element, __m_backend_element_token, #slot_var_name| {
                            __m_slot_children.add(__m_backend_element_token.stable_addr(), (#({#children},)*));
                            Ok(())
                        },
                    )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    maomi::node::Node::new(
                        __m_child,
                        __m_slot_children,
                    )
                }
            }
            TemplateNode::IfElse { branches } => {
                let branch_ty = get_branch_ty(branches.len());
                let branches = branches.iter().enumerate().map(|(index, x)| {
                    let branch_selected = get_branch_selected(index);
                    let TemplateIfElse { else_token, if_cond, children, .. } = x;
                    let span = else_token.as_ref().map(|x| x.span()).or_else(|| if_cond.as_ref().map(|(if_token, _)| if_token.span())).unwrap();
                    let if_cond = match if_cond {
                        Some((if_token, cond)) => quote! { #if_token #cond },
                        None => quote! {},
                    };
                    let children = children.iter().map(|x| TemplateNodeCreate { inside_update, template_node: x, backend_param, locale_group });
                    quote_spanned! {span=>
                        #else_token #if_cond {
                            maomi::node::#branch_ty::#branch_selected((#({#children},)*))
                        }
                    }
                });
                quote! {
                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                    let __m_slot_children = {
                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                        #(#branches)*
                    };
                    let __m_backend_element_token = __m_backend_element.token();
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    maomi::node::ControlNode::new(
                        __m_backend_element_token,
                        __m_slot_children,
                    )
                }
            }
            TemplateNode::Match { match_token, expr, arms, .. } => {
                let branch_ty = get_branch_ty(arms.len());
                let branches = arms.iter().enumerate().map(|(index, x)| {
                    let branch_selected = get_branch_selected(index);
                    let TemplateMatchArm { pat, guard, fat_arrow_token, children, comma, .. } = x;
                    let guard = match guard {
                        Some((if_token, cond)) => quote! { #if_token #cond },
                        None => quote! {},
                    };
                    let children = children.iter().map(|x| TemplateNodeCreate { inside_update, template_node: x, backend_param, locale_group });
                    quote! {
                        #pat #guard #fat_arrow_token {
                            maomi::node::#branch_ty::#branch_selected((#({#children},)*))
                        } #comma
                    }
                });
                let span = match_token.span();
                quote_spanned! {span=>
                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                    let __m_slot_children = {
                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                        #match_token #expr {
                            #(#branches)*
                        }
                    };
                    let __m_backend_element_token = __m_backend_element.token();
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    maomi::node::ControlNode::new(
                        __m_backend_element_token,
                        __m_slot_children,
                    )
                }
            }
            TemplateNode::ForLoop { for_token, pat, in_token, expr, key, children, .. } => {
                let span = for_token.span();
                let children = children.iter().map(|x| TemplateNodeCreate { inside_update, template_node: x, backend_param, locale_group });
                let (algo, next_arg) = if let Some((_, _, key_expr, key_ty)) = key.as_ref() {
                    (
                        quote!(maomi::diff::key::KeyList::<#key_ty, _>),
                        quote!(#key_expr,),
                    )
                } else {
                    (
                        quote!(maomi::diff::keyless::KeylessList::<_>),
                        quote!(),
                    )
                };
                quote_spanned! {span=>
                    let mut __m_list = std::iter::IntoIterator::into_iter(#expr);
                    let __m_size_hint = {
                        let size_hint = std::iter::Iterator::size_hint(&__m_list);
                        size_hint.1.unwrap_or(size_hint.0)
                    };
                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                    let __m_list_diff_algo = {
                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                        let mut __m_list_update_iter = #algo::list_diff_new::<#backend_param>(
                            __m_parent_element,
                            __m_size_hint,
                        );
                        #for_token #pat #in_token __m_list {
                            __m_list_update_iter.next(
                                #next_arg
                                #[inline] |__m_parent_element| {
                                    Ok((#({#children},)*))
                                },
                            )?;
                        }
                        __m_list_update_iter.end()
                    };
                    let __m_backend_element_token = __m_backend_element.token();
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, &__m_backend_element);
                    maomi::node::ControlNode::new(
                        __m_backend_element_token,
                        __m_list_diff_algo,
                    )
                }
            }
        }.to_tokens(tokens);
    }
}

pub(super) struct TemplateUpdate<'a> {
    template: &'a Template,
    backend_param: &'a TokenStream,
    locale_group: &'a LocaleGroup,
}

impl<'a> ToTokens for TemplateUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            template,
            backend_param,
            locale_group,
        } = self;
        let Template { children } = template;
        let children = children
            .into_iter()
            .enumerate()
            .map(|(index, x)| TemplateNodeUpdate {
                child_index: Index::from(index),
                template_node: x,
                backend_param,
                locale_group,
            });
        quote! {
            #({#children})*
        }
        .to_tokens(tokens);
    }
}

struct TemplateNodeUpdate<'a> {
    child_index: Index,
    template_node: &'a TemplateNode,
    backend_param: &'a TokenStream,
    locale_group: &'a LocaleGroup,
}

impl<'a> ToTokens for TemplateNodeUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            child_index,
            template_node,
            backend_param,
            locale_group,
        } = self;
        match template_node {
            TemplateNode::StaticText { .. } => {
                quote! {}
            }
            TemplateNode::DynamicText { brace_token, expr } => {
                let span = brace_token.span;
                let translated = match locale_group.need_trans() {
                    true => quote_spanned! {span=> #expr },
                    false => quote_spanned! {span=> maomi::locale_string::LocaleString::translated(#expr) },
                };
                quote_spanned! {span=>
                    let __m_child = &mut __m_children.#child_index;
                    __m_child.set_text::<#backend_param>(__m_parent_element, #translated)?;
                }
            }
            TemplateNode::Slot { tag_lt_token, data, .. } => {
                let span = tag_lt_token.span();
                let data_expr = match data {
                    None => quote_spanned! {span=> &() },
                    Some(attr) => {
                        match attr {
                            TemplateAttribute::StaticProperty { value, .. } => {
                                let span = value.span();
                                match value {
                                    Lit::Str(_) | Lit::ByteStr(_) => quote! {span=> #value },
                                    _ => quote_spanned! {span=> & #value },
                                }
                            }
                            TemplateAttribute::DynamicProperty { expr, ref_token, .. } => {
                                let span = expr.span();
                                match ref_token {
                                    Some(ref_sign) => quote_spanned!(span=> #ref_sign(#expr)),
                                    None => quote_spanned!(span=> #expr),
                                }
                            }
                            TemplateAttribute::EventHandler { .. } => unreachable!(),
                            TemplateAttribute::Slot { .. } => unreachable!(),
                        }
                    },
                };
                quote_spanned! {span=>
                    let maomi::node::ControlNode {
                        forest_token: ref mut __m_backend_element_token,
                        content: ref mut __m_slot_children,
                    } = __m_children.#child_index;
                    let mut __m_backend_element = __m_parent_element.borrow_mut_token(&__m_backend_element_token)
                        .ok_or(maomi::error::Error::ListChangeWrong)?;
                    let __m_slot_data = &mut __m_slot_scopes.reuse(__m_backend_element_token.stable_addr())?.1;
                    let mut __m_slot_data_changed = false;
                    maomi::prop::PropertyUpdate::compare_and_set_ref(
                        __m_slot_data,
                        #data_expr,
                        &mut __m_slot_data_changed,
                    );
                    if __m_slot_data_changed {
                        __m_slot_fn(maomi::node::SlotChange::DataChanged(&mut __m_backend_element, &__m_backend_element_token, __m_slot_data))?;
                    } else {
                        __m_slot_fn(maomi::node::SlotChange::Unchanged(&mut __m_backend_element, &__m_backend_element_token, __m_slot_data))?;
                    }
                }
            }
            TemplateNode::Tag { tag_lt_token, tag_name, slot_var_name, attrs, children, .. } => {
                let span = tag_lt_token.span();
                let mut list_prop_count = HashMap::new();
                let attrs = attrs.into_iter().map(|attr| {
                    let list_index = if let Some(x) = attr.list_ident() {
                        *list_prop_count.entry(x)
                            .and_modify(|x| *x += 1)
                            .or_insert(1) - 1
                    } else {
                        0
                    };
                    TemplateAttributeUpdate { attr, list_index }
                });
                let create_children = children.into_iter()
                    .map(|x| TemplateNodeCreate {
                        inside_update: true,
                        template_node: x,
                        backend_param,
                        locale_group,
                    });
                let update_children = children.into_iter()
                    .enumerate()
                    .map(|(index, x)| TemplateNodeUpdate {
                        child_index: Index::from(index),
                        template_node: x,
                        backend_param,
                        locale_group,
                    });
                let slot_var_name = match slot_var_name {
                    Some(x) => quote! { #x },
                    None => quote! { __m_slot_data },
                };
                quote_spanned! {span=>
                    let maomi::node::Node {
                        tag: ref mut __m_child,
                        child_nodes: ref mut __m_slot_children,
                    } = __m_children.#child_index;
                    <<#tag_name as maomi::backend::SupportBackend>::Target as maomi::backend::BackendComponent<#backend_param>>::apply_updates(
                        __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        #[inline] |
                            __m_child: &mut <<#tag_name as maomi::backend::SupportBackend>::Target as maomi::backend::BackendComponent<#backend_param>>::UpdateTarget,
                            __m_update_ctx: &mut <<#tag_name as maomi::backend::SupportBackend>::Target as maomi::backend::BackendComponent<#backend_param>>::UpdateContext,
                        | {
                            #(#attrs)*
                        },
                        #[inline] |__m_slot_change| {
                            match __m_slot_change {
                                maomi::node::SlotChange::Added(__m_parent_element, __m_backend_element_token, #slot_var_name) => {
                                    let __m_children = (#({#create_children},)*);
                                    __m_slot_children.add(__m_backend_element_token.stable_addr(), __m_children);
                                }
                                maomi::node::SlotChange::DataChanged(__m_parent_element, __m_backend_element_token, #slot_var_name)
                                    | maomi::node::SlotChange::Unchanged(__m_parent_element, __m_backend_element_token, #slot_var_name)
                                    => {
                                    let __m_children =
                                        __m_slot_children.get_mut(__m_backend_element_token.stable_addr())?;
                                    #({#update_children})*
                                }
                                maomi::node::SlotChange::Removed(__m_backend_element_token) => {
                                    __m_slot_children.remove(__m_backend_element_token.stable_addr())?;
                                }
                            }
                            Ok(())
                        },
                    )?;
                }
            }
            TemplateNode::IfElse { branches } => {
                let branch_ty = get_branch_ty(branches.len());
                let branches = branches.iter().enumerate().map(|(index, x)| {
                    let branch_selected = get_branch_selected(index);
                    let TemplateIfElse { else_token, if_cond, children, .. } = x;
                    let if_cond = match if_cond {
                        Some((if_token, cond)) => quote! { #if_token #cond },
                        None => quote! {},
                    };
                    let create_children = children.iter().map(|x| TemplateNodeCreate { inside_update: true, template_node: x, backend_param, locale_group });
                    let update_children = children.iter()
                        .enumerate()
                        .map(|(index, x)| TemplateNodeUpdate {
                            child_index: Index::from(index),
                            template_node: x,
                            backend_param,
                            locale_group,
                        });
                    quote! {
                        #else_token #if_cond {
                            if let maomi::node::#branch_ty::#branch_selected(__m_children) = __m_slot_children {
                                let __m_parent_element = &mut __m_backend_element;
                                #({#update_children})*
                            } else {
                                let __m_backend_element_new = {
                                    let __m_parent_element = &mut __m_backend_element;
                                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                                    *__m_backend_element_token = __m_backend_element.token();
                                    *__m_slot_children = {
                                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                                        maomi::node::#branch_ty::#branch_selected((#({#create_children},)*))
                                    };
                                    __m_backend_element
                                };
                                <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::replace_with(__m_backend_element, __m_backend_element_new);
                            }
                        }
                    }
                });
                quote! {
                    let maomi::node::ControlNode {
                        forest_token: ref mut __m_backend_element_token,
                        content: ref mut __m_slot_children,
                    } = __m_children.#child_index;
                    let mut __m_backend_element = __m_parent_element.borrow_mut_token(&__m_backend_element_token)
                        .ok_or(maomi::error::Error::ListChangeWrong)?;
                    #(#branches)*
                }
            }
            TemplateNode::Match { match_token, expr, arms, .. } => {
                let branch_ty = get_branch_ty(arms.len());
                let branches = arms.iter().enumerate().map(|(index, x)| {
                    let branch_selected = get_branch_selected(index);
                    let TemplateMatchArm { pat, guard, fat_arrow_token, children, comma, .. } = x;
                    let guard = match guard {
                        Some((if_token, cond)) => quote! { #if_token #cond },
                        None => quote! {},
                    };
                    let create_children = children.iter().map(|x| TemplateNodeCreate { inside_update: true, template_node: x, backend_param, locale_group });
                    let update_children = children.iter()
                        .enumerate()
                        .map(|(index, x)| TemplateNodeUpdate {
                            child_index: Index::from(index),
                            template_node: x,
                            backend_param,
                            locale_group,
                        });
                    quote! {
                        #pat #guard #fat_arrow_token {
                            if let maomi::node::#branch_ty::#branch_selected(__m_children) = __m_slot_children {
                                let __m_parent_element = &mut __m_backend_element;
                                #({#update_children})*
                            } else {
                                let __m_backend_element_new = {
                                    let __m_parent_element = &mut __m_backend_element;
                                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                                    *__m_backend_element_token = __m_backend_element.token();
                                    *__m_slot_children = {
                                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                                        maomi::node::#branch_ty::#branch_selected((#({#create_children},)*))
                                    };
                                    __m_backend_element
                                };
                                <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::replace_with(__m_backend_element, __m_backend_element_new);
                            }
                        } #comma
                    }
                });
                quote! {
                    let maomi::node::ControlNode {
                        forest_token: ref mut __m_backend_element_token,
                        content: ref mut __m_slot_children,
                    } = __m_children.#child_index;
                    let mut __m_backend_element = __m_parent_element.borrow_mut_token(&__m_backend_element_token)
                        .ok_or(maomi::error::Error::ListChangeWrong)?;
                    #match_token #expr {
                        #(#branches)*
                    }
                }
            }
            TemplateNode::ForLoop { for_token, pat, in_token, expr, key, children, .. } => {
                let span = for_token.span();
                let create_children = children.into_iter()
                    .map(|x| TemplateNodeCreate {
                        inside_update: true,
                        template_node: x,
                        backend_param,
                        locale_group,
                    });
                let update_children = children.into_iter()
                    .enumerate()
                    .map(|(index, x)| TemplateNodeUpdate {
                        child_index: Index::from(index),
                        template_node: x,
                        backend_param,
                        locale_group,
                    });
                let next_arg = if let Some((_, _, key_expr, _)) = key.as_ref() {
                    quote!(#key_expr,)
                } else {
                    quote!()
                };
                quote_spanned! {span=>
                    let maomi::node::ControlNode {
                        forest_token: ref mut __m_backend_element_token,
                        content: ref mut __m_list_diff_algo,
                    } = __m_children.#child_index;
                    let mut __m_parent_element = &mut __m_parent_element.borrow_mut_token(&__m_backend_element_token)
                        .ok_or(maomi::error::Error::ListChangeWrong)?;
                    let mut __m_list = std::iter::IntoIterator::into_iter(#expr);
                    let __m_size_hint = {
                        let size_hint = std::iter::Iterator::size_hint(&__m_list);
                        size_hint.1.unwrap_or(size_hint.0)
                    };
                    let mut __m_list_update_iter = __m_list_diff_algo.list_diff_update::<#backend_param>(
                        __m_parent_element,
                        __m_size_hint,
                    );
                    #for_token #pat #in_token __m_list {
                        __m_list_update_iter.next(
                            #next_arg
                            #[inline] |__m_children, __m_parent_element| {
                                if let Some(__m_children) = __m_children {
                                    #({#update_children})*
                                    Ok(None)
                                } else {
                                    Ok(Some((#({#create_children},)*)))
                                }
                            },
                        )?;
                    }
                    __m_list_update_iter.end()?;
                }
            }
        }.to_tokens(tokens);
    }
}

pub(super) struct TemplateAttributeCreate<'a> {
    attr: &'a TemplateAttribute,
    list_index: usize,
}

impl<'a> ToTokens for TemplateAttributeCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attr, list_index } = self;
        match attr {
            TemplateAttribute::StaticProperty { name, list_updater, value, .. } => {
                let span = value.span();
                let ref_sign = match value {
                    Lit::Str(_) | Lit::ByteStr(_) => quote! {},
                    _ => quote!{ & },
                };
                if let Some((_, updater)) = list_updater {
                    let index = Index::from(*list_index);
                    quote_spanned! {span=>
                        maomi::prop::ListPropertyUpdate::compare_and_set_item_ref::<#updater>(
                            &mut __m_child.#name,
                            #index,
                            #ref_sign #value,
                            __m_update_ctx,
                        );
                    }
                } else {
                    quote_spanned! {span=>
                        maomi::prop::PropertyUpdate::compare_and_set_ref(
                            &mut __m_child.#name,
                            #ref_sign #value,
                            __m_update_ctx,
                        );
                    }
                }
            }
            TemplateAttribute::DynamicProperty { ref_token, name, list_updater, expr, .. } => {
                let span = expr.span();
                let expr = match ref_token {
                    Some(ref_sign) => quote_spanned!(span=> #ref_sign(#expr)),
                    None => quote_spanned!(span=> #expr),
                };
                if let Some((_, updater)) = list_updater {
                    let index = Index::from(*list_index);
                    quote_spanned! {span=>
                        maomi::prop::ListPropertyUpdate::compare_and_set_item_ref::<#updater>(
                            &mut __m_child.#name,
                            #index,
                            #expr,
                            __m_update_ctx,
                        );
                    }
                } else {
                    quote_spanned! {span=>
                        maomi::prop::PropertyUpdate::compare_and_set_ref(
                            &mut __m_child.#name,
                            #expr,
                            __m_update_ctx,
                        );
                    }
                }
            }
            TemplateAttribute::EventHandler { name, at_token, fn_name, args, .. } => {
                let span = at_token.span();
                let (args_ref, args_expr): (Vec<_>, Vec<_>) = args.iter().enumerate().map(|(index, expr)| {
                    let span = expr.span();
                    let arg_name = Ident::new(&format!("__m_arg{}", index), span);
                    let arg_expr = quote_spanned! {span=> let #arg_name = std::borrow::ToOwned::to_owned(#expr); };
                    let arg_ref = quote_spanned! {span=> std::borrow::Borrow::borrow(&#arg_name) };
                    (arg_ref, arg_expr)
                }).unzip();
                quote_spanned! {span=>
                    let __m_event_self = __m_event_self_weak.clone();
                    #(#args_expr)*
                    maomi::event::EventHandler::set_handler_fn(
                        &mut __m_child.#name,
                        Box::new(move |__m_event_detail| {
                            if let Some(__m_event_self) = __m_event_self.upgrade() {
                                Self::#fn_name(__m_event_self, __m_event_detail, #(#args_ref),*)
                            }
                        }),
                        __m_update_ctx,
                    );
                }
            }
            TemplateAttribute::Slot { .. } => unreachable!(),
        }
        .to_tokens(tokens)
    }
}

pub(super) struct TemplateAttributeUpdate<'a> {
    attr: &'a TemplateAttribute,
    list_index: usize,
}

impl<'a> ToTokens for TemplateAttributeUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attr, list_index } = self;
        match attr {
            TemplateAttribute::StaticProperty { .. } => {
                quote! {}
            }
            TemplateAttribute::DynamicProperty {
                ref_token,
                name,
                list_updater,
                expr,
                ..
            } => {
                let span = expr.span();
                let expr = match ref_token {
                    Some(ref_sign) => quote_spanned!(span=> #ref_sign(#expr)),
                    None => quote_spanned!(span=> #expr),
                };
                if let Some((_, updater)) = list_updater {
                    let index = Index::from(*list_index);
                    quote_spanned! {span=>
                        maomi::prop::ListPropertyUpdate::compare_and_set_item_ref::<#updater>(
                            &mut __m_child.#name,
                            #index,
                            #expr,
                            __m_update_ctx,
                        );
                    }
                } else {
                    quote_spanned! {span=>
                        maomi::prop::PropertyUpdate::compare_and_set_ref(
                            &mut __m_child.#name,
                            #expr,
                            __m_update_ctx,
                        );
                    }
                }
            }
            TemplateAttribute::EventHandler { name, at_token, fn_name, args, .. } => {
                if args.len() > 0 {
                    let span = at_token.span();
                    let (args_ref, args_expr): (Vec<_>, Vec<_>) = args.iter().enumerate().map(|(index, expr)| {
                        let span = expr.span();
                        let arg_name = Ident::new(&format!("__m_arg{}", index), span);
                        let arg_expr = quote_spanned! {span=> let #arg_name = std::borrow::ToOwned::to_owned(#expr); };
                        let arg_ref = quote_spanned! {span=> std::borrow::Borrow::borrow(&#arg_name) };
                        (arg_ref, arg_expr)
                    }).unzip();
                    quote_spanned! {span=>
                        let __m_event_self = __m_event_self_weak.clone();
                        #(#args_expr)*
                        maomi::event::EventHandler::set_handler_fn(
                            &mut __m_child.#name,
                            Box::new(move |__m_event_detail| {
                                if let Some(__m_event_self) = __m_event_self.upgrade() {
                                    Self::#fn_name(__m_event_self, __m_event_detail, #(#args_ref),*)
                                }
                            }),
                            __m_update_ctx,
                        );
                    }
                } else {
                    quote! {}
                }
            }
            TemplateAttribute::Slot { .. } => unreachable!(),
        }
        .to_tokens(tokens)
    }
}
