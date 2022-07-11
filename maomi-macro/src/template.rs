use proc_macro2::TokenStream;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

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
        parse_quote_spanned!(span=> maomi::component::Template<(#(#children,)*)> )
    }

    pub(super) fn to_create<'a>(&'a self, backend_param: &'a TokenStream) -> TemplateCreate<'a> {
        TemplateCreate { template: self, backend_param }
    }

    pub(super) fn to_update<'a>(&'a self, backend_param: &'a TokenStream) -> TemplateUpdate<'a> {
        TemplateUpdate { template: self, backend_param }
    }
}

impl Parse for Template {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = vec![];
        while !input.is_empty() {
            let child = input.parse()?;
            children.push(child);
        }
        Ok(Self {
            children,
        })
    }
}

pub(super) enum TemplateNode {
    StaticText {
        content: LitStr,
    },
    DynamicText {
        brace_token: token::Brace,
        expr: Expr,
    },
    SelfCloseTag {
        tag_lt_token: token::Lt,
        tag_name: Path,
        attrs: Vec<TemplateAttribute>,
        close_token: token::Div,
        tag_gt_token: token::Gt,
    },
    Tag {
        tag_lt_token: token::Lt,
        tag_name: Path,
        attrs: Vec<TemplateAttribute>,
        tag_gt_token: token::Gt,
        children: Vec<TemplateNode>,
        end_tag_lt_token: token::Lt,
        close_token: token::Div,
        end_tag_name: Path,
        end_tag_gt_token: token::Gt,
    },
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
            Self::SelfCloseTag { tag_name, .. } => {
                let span = tag_name.span();
                parse_quote_spanned!(span=> maomi::component::Node<#tag_name, ()> )
            }
            Self::Tag {
                tag_name, children, ..
            } => {
                let span = tag_name.span();
                let children = children.iter().map(|c| c.gen_type());
                parse_quote_spanned!(span=> maomi::component::Node<#tag_name, (#(#children,)*)> )
            }
        }
    }
}

impl Parse for TemplateNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let ret = if la.peek(LitStr) {
            TemplateNode::StaticText {
                content: input.parse()?,
            }
        } else if la.peek(token::Brace) {
            let content;
            let brace_token = braced!(content in input);
            let expr = content.parse()?;
            TemplateNode::DynamicText { brace_token, expr }
        } else if la.peek(token::Lt) {
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
                while !input.peek(token::Lt) && !input.peek2(token::Div) {
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
                    return Err(Error::new(end_tag_name.span(), "End tag name does not match the start tag name"));
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
        } else {
            return Err(la.error());
        };
        Ok(ret)
    }
}

pub(super) enum TemplateAttribute {
    StaticProperty {
        name: Ident,
        eq_token: token::Eq,
        value: Lit,
    },
    DynamicProperty {
        name: Ident,
        eq_token: token::Eq,
        brace_token: token::Brace,
        expr: Expr,
    },
    Event {
        at_token: token::At,
        name: Ident,
        eq_token: token::Eq,
        brace_token: token::Brace,
        expr: Expr,
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
            } else if la.peek(token::Brace) {
                let content;
                let brace_token = braced!(content in input);
                let expr = content.parse()?;
                TemplateAttribute::DynamicProperty {
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
        let Self { template, backend_param } = self;
        let Template { children } = template;
        let children = children.into_iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
        quote! {
            let __child_nodes = {
                let mut __parent_element = __parent_element.borrow_mut(&__backend_element);
                (
                    #({ #children },)*
                )
            };
        }.to_tokens(tokens)
    }
}

struct TemplateNodeCreate<'a> {
    template_node: &'a TemplateNode,
    backend_param: &'a TokenStream,
}

impl<'a> ToTokens for TemplateNodeCreate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { template_node, backend_param } = self;
        match template_node {
            TemplateNode::StaticText { content } => {
                let span = content.span();
                quote_spanned! {span=>
                    let (__node, __backend_element) = maomi::text_node::TextNode::create::<#backend_param>(
                        &mut __parent_element,
                        #content,
                    )?;
                    <#backend_param as maomi::backend::Backend>::GeneralElement::append(&mut __parent_element, __backend_element);
                    __node
                }
            }
            TemplateNode::DynamicText { brace_token, expr } => {
                let span = brace_token.span;
                quote_spanned! {span=>
                    let (__node, __backend_element) = maomi::text_node::TextNode::create::<#backend_param>(
                        &mut __parent_element,
                        #expr,
                    )?;
                    <#backend_param as maomi::backend::Backend>::GeneralElement::append(&mut __parent_element, __backend_element);
                    __node
                }
            }
            TemplateNode::SelfCloseTag { tag_lt_token, tag_name, attrs, .. } => {
                let span = tag_lt_token.span();
                quote_spanned! {span=>
                    let (__node, __backend_element) =
                        <#tag_name as maomi::backend::SupportBackend<#backend_param>>::create(&mut __parent_element, |__node| {
                            // TODO set property
                            Ok(())
                        })?;
                    let __child_nodes = ();
                    <#backend_param as maomi::backend::Backend>::GeneralElement::append(&mut __parent_element, __backend_element);
                    maomi::component::Node { node: __node, child_nodes: __child_nodes }
                }
            }
            TemplateNode::Tag { tag_lt_token, tag_name, attrs, children, .. } => {
                let span = tag_lt_token.span();
                let children = children.into_iter().map(|x| TemplateNodeCreate { template_node: x, backend_param });
                quote_spanned! {span=>
                    let (__node, __backend_element) =
                        <#tag_name as maomi::backend::SupportBackend<#backend_param>>::create(&mut __parent_element, |__node| {
                            // TODO set property
                            Ok(())
                        })?;
                    let __child_nodes = {
                        let mut __parent_element = __parent_element.borrow_mut(&__backend_element);
                        (
                            #({ #children },)*
                        )
                    };
                    <#backend_param as maomi::backend::Backend>::GeneralElement::append(&mut __parent_element, __backend_element);
                    maomi::component::Node { node: __node, child_nodes: __child_nodes }
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
        let Self { template, backend_param } = self;
        let Template { children } = template;
        let children = children.into_iter()
            .enumerate()
            .map(|(index, x)| TemplateNodeUpdate {
                child_index: Index::from(index),
                template_node: x,
                backend_param,
            });
        quote! {
            {
                #({ #children })*
            }
        }.to_tokens(tokens);
    }
}

struct TemplateNodeUpdate<'a> {
    child_index: Index,
    template_node: &'a TemplateNode,
    backend_param: &'a TokenStream,
}

impl<'a> ToTokens for TemplateNodeUpdate<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { child_index, template_node, backend_param } = self;
        match template_node {
            TemplateNode::StaticText { .. } => {
                quote! {}
            }
            TemplateNode::DynamicText { brace_token, expr } => {
                let span = brace_token.span;
                quote_spanned! {span=>
                    let __node = &mut __child_nodes.#child_index;
                    __node.set_text(#expr);
                    __node.apply_updates::<#backend_param>(&mut __backend_element)?;
                }
            }
            TemplateNode::SelfCloseTag { tag_lt_token, tag_name, attrs, .. } => {
                let span = tag_lt_token.span();
                quote_spanned! {span=>
                    let maomi::component::Node { node: ref mut __node, child_nodes: ref mut __child_nodes } = __child_nodes.#child_index;
                    {
                        // TODO set property
                    }
                    <#tag_name as maomi::backend::SupportBackend<#backend_param>>::apply_updates(__node, &mut __backend_element)?;
                }
            }
            TemplateNode::Tag { tag_lt_token, tag_name, attrs, children, .. } => {
                let span = tag_lt_token.span();
                let children = children.into_iter()
                    .enumerate()
                    .map(|(index, x)| TemplateNodeUpdate {
                        child_index: Index::from(index),
                        template_node: x,
                        backend_param,
                    });
                quote_spanned! {span=>
                    let maomi::component::Node { node: ref mut __node, child_nodes: ref mut __child_nodes } = __child_nodes.#child_index;
                    {
                        // TODO set property
                    }
                    <#tag_name as maomi::backend::SupportBackend<#backend_param>>::apply_updates(__node, &mut __backend_element)?;
                    #({ #children })*
                }
            }
        }.to_tokens(tokens);
    }
}
