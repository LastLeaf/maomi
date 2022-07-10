use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

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
    pub(super) fn gen_type(&self) -> Type {
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
                parse_quote_spanned!(span=> maomi::component::Node<#tag_name, (#(#children)*)> )
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
                let end_tag_name = input.parse()?;
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

impl ToTokens for TemplateNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
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

impl ToTokens for TemplateAttribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}
