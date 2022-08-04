use proc_macro2::TokenStream;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

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
    pub(super) fn gen_type(
        &self,
        backend_param: &TokenStream,
        md: &MacroDelimiter,
    ) -> Type {
        let Self { children } = self;
        let span = match md {
            MacroDelimiter::Paren(x) => x.span,
            MacroDelimiter::Brace(x) => x.span,
            MacroDelimiter::Bracket(x) => x.span,
        };
        let children = children.iter().map(|c| c.gen_type(backend_param));
        parse_quote_spanned!(span=> (#(#children,)*) )
    }

    pub(super) fn to_create<'a>(&'a self, backend_param: &'a TokenStream) -> TemplateCreate<'a> {
        TemplateCreate {
            template: self,
            backend_param,
        }
    }

    pub(super) fn to_update<'a>(&'a self, backend_param: &'a TokenStream) -> TemplateUpdate<'a> {
        TemplateUpdate {
            template: self,
            backend_param,
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
    SelfCloseTag {
        tag_lt_token: token::Lt,
        tag_name: Path,
        attrs: Vec<TemplateAttribute>,
        #[allow(dead_code)]
        close_token: token::Div,
        #[allow(dead_code)]
        tag_gt_token: token::Gt,
    },
    Tag {
        tag_lt_token: token::Lt,
        tag_name: Path,
        attrs: Vec<TemplateAttribute>,
        #[allow(dead_code)]
        tag_gt_token: token::Gt,
        children: Vec<TemplateNode>,
        #[allow(dead_code)]
        end_tag_lt_token: token::Lt,
        #[allow(dead_code)]
        close_token: token::Div,
        #[allow(dead_code)]
        end_tag_name: Path,
        #[allow(dead_code)]
        end_tag_gt_token: token::Gt,
    },
    IfElse {
        branches: Vec<TemplateIfElse>,
    },
    Match {
        match_token: token::Match,
        expr: Box<Expr>,
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
    fn gen_type(
        &self,
        backend_param: &TokenStream,
    ) -> Type {
        match self {
            Self::StaticText { content } => {
                let span = content.span();
                parse_quote_spanned!(span=> maomi::text_node::TextNode )
            }
            Self::DynamicText { brace_token, .. } => {
                let span = brace_token.span;
                parse_quote_spanned!(span=> maomi::text_node::TextNode )
            }
            Self::SelfCloseTag { tag_name, .. } => {
                let span = tag_name.span();
                parse_quote_spanned!(span=> maomi::node::Node<#backend_param, #tag_name, ()> )
            }
            Self::Tag { tag_name, children, .. } => {
                let span = tag_name.span();
                let children = children.iter().map(|c| c.gen_type(backend_param));
                parse_quote_spanned!(span=> maomi::node::Node<#backend_param, #tag_name, (#(#children,)*)> )
            }
            Self::IfElse { branches } => {
                let branch_ty = get_branch_ty(branches.len());
                let branches = branches.iter().map(|x| {
                    let span = x.brace_token.span;
                    let children = x.children.iter().map(|c| c.gen_type(backend_param));
                    quote_spanned!(span=> (#(#children,)*) )
                });
                parse_quote!(maomi::node::ControlNode<maomi::node::#branch_ty<#(#branches),*>> )
            }
            Self::Match { arms, .. } => {
                let branch_ty = get_branch_ty(arms.len());
                let branches = arms.iter().map(|x| {
                    let span = x.brace_token.span;
                    let children = x.children.iter().map(|c| c.gen_type(backend_param));
                    quote_spanned!(span=> (#(#children,)*) )
                });
                parse_quote!(maomi::node::ControlNode<maomi::node::#branch_ty<#(#branches),*>> )
            }
            Self::ForLoop { brace_token, children, key, .. } => {
                let span = brace_token.span;
                let children = children.iter().map(|c| c.gen_type(backend_param));
                let key_ty = match key {
                    Some((_, _, _, key_ty)) => quote!(maomi::diff::key::ListKeyAlgo<#backend_param, #key_ty>),
                    None => quote!(),
                };
                parse_quote_spanned!(span=> maomi::node::ControlNode<maomi::node::Loop<#key_ty, (#(#children,)*)>> )
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
            // parse a tag
            let tag_lt_token = input.parse()?;
            let tag_name = input.parse()?;
            let mut attrs = vec![];
            let mut la = input.lookahead1();
            loop {
                if la.peek(Ident) {
                    let attr = input.parse()?;
                    attrs.push(attr);
                    la = input.lookahead1();
                } else {
                    break;
                }
            }
            if la.peek(token::Div) {
                let close_token = input.parse()?;
                let tag_gt_token = input.parse()?;
                TemplateNode::SelfCloseTag {
                    tag_lt_token,
                    tag_name,
                    attrs,
                    close_token,
                    tag_gt_token,
                }
            } else if la.peek(token::Gt) {
                let tag_gt_token = input.parse()?;
                let mut children = vec![];
                while !input.peek(token::Lt) || !input.peek2(token::Div) {
                    let child = input.parse()?;
                    children.push(child);
                }
                let end_tag_lt_token = input.parse()?;
                let close_token = input.parse()?;
                let end_tag_name: Path = input.parse()?;
                let short_tag_name = match tag_name.segments.last() {
                    None => Ident::new("", tag_name.span()),
                    Some(x) => x.ident.clone(),
                };
                if !end_tag_name.is_ident(&short_tag_name) {
                    return Err(Error::new(
                        end_tag_name.span(),
                        "End tag name does not match the start tag name",
                    ));
                }
                let end_tag_gt_token = input.parse()?;
                TemplateNode::Tag {
                    tag_lt_token,
                    tag_name,
                    attrs,
                    tag_gt_token,
                    children,
                    end_tag_lt_token,
                    close_token,
                    end_tag_name,
                    end_tag_gt_token,
                }
            } else {
                return Err(la.error());
            }
        } else if la.peek(token::If) {
            // parse if expr
            let mut branches = vec![];
            let mut else_token = None;
            loop {
                let if_cond = if input.peek(token::If) {
                    Some((input.parse()?, input.parse()?))
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
                    break
                }
            }
            TemplateNode::IfElse { branches }
        } else if la.peek(token::Match) {
            // parse match expr
            let match_token = input.parse()?;
            let expr = input.parse()?;
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
            TemplateNode::Match { match_token, expr, brace_token, arms }
        } else if la.peek(token::For) {
            // parse for expr
            let for_token = input.parse()?;
            let pat = input.parse()?;
            let in_token = input.parse()?;
            let expr = input.parse()?;
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
            TemplateNode::ForLoop { for_token, pat, in_token, expr, key, brace_token, children }
        } else {
            return Err(la.error());
        };
        Ok(ret)
    }
}

pub(super) enum TemplateAttribute {
    StaticProperty {
        name: Ident,
        #[allow(dead_code)]
        eq_token: token::Eq,
        value: Lit,
    },
    DynamicProperty {
        ref_token: Option<token::And>,
        name: Ident,
        #[allow(dead_code)]
        eq_token: token::Eq,
        #[allow(dead_code)]
        brace_token: token::Brace,
        expr: Box<Expr>,
    },
    Event {
        #[allow(dead_code)]
        at_token: token::At,
        name: Ident,
        #[allow(dead_code)]
        eq_token: token::Eq,
        #[allow(dead_code)]
        brace_token: token::Brace,
        expr: Box<Expr>,
    },
}

impl Parse for TemplateAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let ret = if la.peek(Ident) {
            let name = input.parse()?;
            let eq_token = input.parse()?;
            let la = input.lookahead1();
            if la.peek(Lit) {
                let value = input.parse()?;
                TemplateAttribute::StaticProperty {
                    name,
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
                    eq_token,
                    brace_token,
                    expr,
                }
            } else {
                return Err(la.error());
            }
        } else if la.peek(token::At) {
            let at_token = input.parse()?;
            let name = input.parse()?;
            let eq_token = input.parse()?;
            let content;
            let brace_token = braced!(content in input);
            let expr = content.parse()?;
            TemplateAttribute::Event {
                at_token,
                name,
                eq_token,
                brace_token,
                expr,
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
}

impl<'a> ToTokens for TemplateCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            template,
            backend_param,
        } = self;
        let Template { children } = template;
        let children = children.into_iter().map(|x| TemplateNodeCreate {
            template_node: x,
            backend_param,
        });
        quote! {
            (#({#children},)*)
        }
        .to_tokens(tokens)
    }
}

struct TemplateNodeCreate<'a> {
    template_node: &'a TemplateNode,
    backend_param: &'a TokenStream,
}

impl<'a> ToTokens for TemplateNodeCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            template_node,
            backend_param,
        } = self;
        match template_node {
            TemplateNode::StaticText { content } => {
                let span = content.span();
                quote_spanned! {span=>
                    let (__m_child, __m_backend_element) =
                        maomi::text_node::TextNode::create::<#backend_param>(
                            __m_parent_element,
                            #content,
                        )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                    __m_child
                }
            }
            TemplateNode::DynamicText { brace_token, expr } => {
                let span = brace_token.span;
                quote_spanned! {span=>
                    let (__m_child, __m_backend_element) =
                        maomi::text_node::TextNode::create::<#backend_param>(
                            __m_parent_element,
                            #expr,
                        )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                    __m_child
                }
            }
            TemplateNode::SelfCloseTag { tag_lt_token, tag_name, attrs, .. } => {
                let span = tag_lt_token.span();
                let attrs = attrs.into_iter().map(|attr| TemplateAttributeCreate { attr });
                quote_spanned! {span=>
                    let (mut __m_child, __m_backend_element) =
                        <<#tag_name as maomi::backend::SupportBackend<#backend_param>>::Target as maomi::backend::BackendComponent<#backend_param>>::init(
                            __m_backend_context,
                            __m_parent_element,
                        )?;
                    let __m_slot_children = <<#tag_name as maomi::backend::SupportBackend<#backend_param>>::Target as maomi::backend::BackendComponent<#backend_param>>::create(
                        &mut __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        |__m_child, __m_update_ctx| {
                            #(#attrs)*
                        },
                        |__m_parent_element, __m_scope| Ok(()),
                    )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                    maomi::node::Node {
                        tag: __m_child,
                        child_nodes: __m_slot_children,
                    }
                }
            }
            TemplateNode::Tag { tag_lt_token, tag_name, attrs, children, .. } => {
                let span = tag_lt_token.span();
                let attrs = attrs.into_iter().map(|attr| TemplateAttributeCreate { attr });
                let children = children.into_iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                quote_spanned! {span=>
                    let (mut __m_child, __m_backend_element) =
                        <<#tag_name as maomi::backend::SupportBackend<#backend_param>>::Target as maomi::backend::BackendComponent<#backend_param>>::init(
                            __m_backend_context,
                            __m_parent_element,
                        )?;
                    let __m_slot_children = <<#tag_name as maomi::backend::SupportBackend<#backend_param>>::Target as maomi::backend::BackendComponent<#backend_param>>::create(
                        &mut __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        |__m_child, __m_update_ctx| {
                            #(#attrs)*
                        },
                        |__m_parent_element, __m_scope| {
                            Ok((#({#children},)*))
                        },
                    )?;
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                    maomi::node::Node {
                        tag: __m_child,
                        child_nodes: __m_slot_children,
                    }
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
                    let children = children.iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                    quote! {
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
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                    maomi::node::ControlNode {
                        forest_token: __m_backend_element_token,
                        content: __m_slot_children,
                    }
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
                    let children = children.iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                    quote! {
                        #pat #guard #fat_arrow_token {
                            maomi::node::#branch_ty::#branch_selected((#({#children},)*))
                        } #comma
                    }
                });
                quote! {
                    let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                    let __m_slot_children = {
                        let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                        #match_token #expr {
                            #(#branches)*
                        }
                    };
                    let __m_backend_element_token = __m_backend_element.token();
                    <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                    maomi::node::ControlNode {
                        forest_token: __m_backend_element_token,
                        content: __m_slot_children,
                    }
                }
            }
            TemplateNode::ForLoop { for_token, pat, in_token, expr, key, children, .. } => {
                let children = children.iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                if let Some((_, __m_list_update_iter, key_expr, key_ty)) = key.as_ref() {
                    quote! {
                        let mut __m_list = std::iter::IntoIterator::into_iter(#expr);
                        let __m_size_hint = {
                            let size_hint = __m_list.size_hint();
                            size_hint.1.unwrap_or(size_hint.0)
                        };
                        let mut __m_slot_children = Vec::with_capacity(__m_size_hint);
                        let mut __m_list_diff_algo = maomi::diff::key::ListKeyAlgo::<#backend_param, #key_ty>::list_diff_new();
                        let __m_backend_element = <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::create_virtual_element(__m_parent_element)?;
                        {
                            let __m_parent_element = &mut __m_parent_element.borrow_mut(&__m_backend_element);
                            let mut __m_list_update_iter = __m_list_diff_algo.list_diff_update(
                                &mut __m_slot_children,
                                __m_parent_element,
                                __m_size_hint,
                            );
                            #for_token #pat #in_token __m_list {
                                __m_list_update_iter.next(
                                    #key_expr,
                                    |__m_parent_element| {
                                        Ok((#({#children},)*))
                                    },
                                    |_, __m_parent_element| {
                                        unreachable!()
                                    },
                                )?;
                            }
                            __m_list_update_iter.end()?;
                        }
                        let __m_backend_element_token = __m_backend_element.token();
                        <<#backend_param as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                        maomi::node::ControlNode {
                            forest_token: __m_backend_element_token,
                            content: maomi::node::Loop {
                                list_diff_algo: __m_list_diff_algo,
                                items: __m_slot_children,
                            },
                        }
                    }
                } else {
                    todo!()
                }
            }
        }.to_tokens(tokens);
    }
}

pub(super) struct TemplateUpdate<'a> {
    template: &'a Template,
    backend_param: &'a TokenStream,
}

impl<'a> ToTokens for TemplateUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            template,
            backend_param,
        } = self;
        let Template { children } = template;
        let children = children
            .into_iter()
            .enumerate()
            .map(|(index, x)| TemplateNodeUpdate {
                child_index: Index::from(index),
                template_node: x,
                backend_param,
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
}

impl<'a> ToTokens for TemplateNodeUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            child_index,
            template_node,
            backend_param,
        } = self;
        match template_node {
            TemplateNode::StaticText { .. } => {
                quote! {}
            }
            TemplateNode::DynamicText { brace_token, expr } => {
                let span = brace_token.span;
                quote_spanned! {span=>
                    let __m_child = &mut __m_children.#child_index;
                    __m_child.set_text::<#backend_param>(__m_parent_element, #expr)?;
                }
            }
            TemplateNode::SelfCloseTag { tag_lt_token, tag_name, attrs, .. } => {
                let span = tag_lt_token.span();
                let attrs = attrs.into_iter().map(|attr| TemplateAttributeUpdate { attr });
                quote_spanned! {span=>
                    let maomi::node::Node {
                        tag: ref mut __m_child,
                        child_nodes: ref mut __m_slot_children,
                    } = __m_children.#child_index;
                    let mut __m_children_i = 0usize;
                    <<#tag_name as maomi::backend::SupportBackend<#backend_param>>::Target as maomi::backend::BackendComponent<#backend_param>>::apply_updates(
                        __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        |__m_child, __m_update_ctx| {
                            #(#attrs)*
                        },
                        |__m_slot_change| {
                            Ok(
                                match __m_slot_change {
                                    maomi::diff::ListItemChange::Added(__m_parent_element, __m_scope) => {
                                        __m_slot_children.add(__m_children_i, ())?;
                                        __m_children_i += 1;
                                    }
                                    maomi::diff::ListItemChange::Unchanged(__m_parent_element, __m_scope) => {
                                        __m_children_i += 1;
                                    }
                                    maomi::diff::ListItemChange::Removed(__m_parent_element) => {
                                        __m_slot_children.remove(__m_children_i)?;
                                    }
                                }
                            )
                        },
                    )?;
                }
            }
            TemplateNode::Tag { tag_lt_token, tag_name, attrs, children, .. } => {
                let span = tag_lt_token.span();
                let attrs = attrs.into_iter().map(|attr| TemplateAttributeUpdate { attr });
                let create_children = children.into_iter()
                    .map(|x| TemplateNodeCreate {
                        template_node: x,
                        backend_param,
                    });
                let update_children = children.into_iter()
                    .enumerate()
                    .map(|(index, x)| TemplateNodeUpdate {
                        child_index: Index::from(index),
                        template_node: x,
                        backend_param,
                    });
                quote_spanned! {span=>
                    let maomi::node::Node {
                        tag: ref mut __m_child,
                        child_nodes: ref mut __m_slot_children,
                    } = __m_children.#child_index;
                    let mut __m_children_i = 0usize;
                    <<#tag_name as maomi::backend::SupportBackend<#backend_param>>::Target as maomi::backend::BackendComponent<#backend_param>>::apply_updates(
                        __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        |__m_child, __m_update_ctx| {
                            #(#attrs)*
                        },
                        |__m_slot_change| {
                            Ok(
                                match __m_slot_change {
                                    maomi::diff::ListItemChange::Added(__m_parent_element, __m_scope) => {
                                        let __m_children = (#({#create_children},)*);
                                        __m_slot_children.add(__m_children_i, __m_children)?;
                                        __m_children_i += 1;
                                    }
                                    maomi::diff::ListItemChange::Unchanged(__m_parent_element, __m_scope) => {
                                        // TODO handling relationship correctly
                                        let __m_children =
                                            __m_slot_children.get_mut(__m_children_i)?;
                                        #({#update_children})*
                                        __m_children_i += 1;
                                    }
                                    maomi::diff::ListItemChange::Removed(__m_parent_element) => {
                                        __m_slot_children.remove(__m_children_i)?;
                                    }
                                }
                            )
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
                    let create_children = children.iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                    let update_children = children.iter()
                        .enumerate()
                        .map(|(index, x)| TemplateNodeUpdate {
                            child_index: Index::from(index),
                            template_node: x,
                            backend_param,
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
                    let mut __m_backend_element = __m_parent_element.borrow_mut_token(&__m_backend_element_token);
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
                    let create_children = children.iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                    let update_children = children.iter()
                        .enumerate()
                        .map(|(index, x)| TemplateNodeUpdate {
                            child_index: Index::from(index),
                            template_node: x,
                            backend_param,
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
                    let mut __m_backend_element = __m_parent_element.borrow_mut_token(&__m_backend_element_token);
                    #match_token #expr {
                        #(#branches)*
                    }
                }
            }
            TemplateNode::ForLoop { for_token, pat, in_token, expr, key, children, .. } => {
                quote! {
                    todo!();
                }
            }
        }.to_tokens(tokens);
    }
}

pub(super) struct TemplateAttributeCreate<'a> {
    attr: &'a TemplateAttribute,
}

impl<'a> ToTokens for TemplateAttributeCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attr } = self;
        match attr {
            TemplateAttribute::StaticProperty { name, value, .. } => {
                let span = value.span();
                let ref_sign = match value {
                    Lit::Str(_) | Lit::ByteStr(_) => quote! {},
                    _ => quote!{ & },
                };
                quote_spanned! {span=>
                    maomi::prop::PropertyUpdate::compare_and_set_ref(&mut __m_child.#name, #ref_sign #value, __m_update_ctx);
                }
            }
            TemplateAttribute::DynamicProperty { ref_token, name, expr, .. } => {
                let span = expr.span();
                if ref_token.is_some() {
                    quote_spanned! {span=>
                        maomi::prop::PropertyUpdate::compare_and_set_ref(&mut __m_child.#name, &(#expr), __m_update_ctx);
                    }
                } else {
                    quote_spanned! {span=>
                        maomi::prop::PropertyUpdate::compare_and_set_ref(&mut __m_child.#name, #expr, __m_update_ctx);
                    }
                }
            }
            TemplateAttribute::Event { name, expr, .. } => {
                let span = expr.span();
                quote_spanned! {span=>
                    maomi::prop::PropertyUpdate::compare_and_set_ref(&mut __m_child.#name, #expr, __m_update_ctx);
                }
            }
        }
        .to_tokens(tokens)
    }
}

pub(super) struct TemplateAttributeUpdate<'a> {
    attr: &'a TemplateAttribute,
}

impl<'a> ToTokens for TemplateAttributeUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attr } = self;
        match attr {
            TemplateAttribute::StaticProperty { .. } => {
                quote! {}
            }
            TemplateAttribute::DynamicProperty { ref_token, name, expr, .. } => {
                let span = expr.span();
                quote_spanned! {span=>
                    maomi::prop::PropertyUpdate::compare_and_set_ref(&mut __m_child.#name, #ref_token #expr, __m_update_ctx);
                }
            }
            TemplateAttribute::Event { .. } => {
                quote! {}
            }
        }
        .to_tokens(tokens)
    }
}
