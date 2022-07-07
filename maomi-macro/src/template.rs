use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::*;

struct TemplateDefinition {
    backend_target: Option<TemplateBackendTarget>,
    brace_token: token::Brace,
    children: Vec<TemplateNode>,
}

impl Parse for TemplateDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let backend_target = if la.peek(token::For) {
            Some(input.parse()?)
        } else if la.peek(token::Brace) {
            None
        } else {
            return Err(la.error());
        };
        let content;
        let brace_token = braced!(content in input);
        let mut children = vec![];
        while !content.is_empty() {
            let child = content.parse()?;
            children.push(child);
        }
        Ok(Self {
            backend_target,
            brace_token,
            children,
        })
    }
}

impl ToTokens for TemplateDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

struct TemplateBackendTarget {
    for_token: token::For,
    impl_token: Option<token::Impl>,
    path: Path,
}

impl Parse for TemplateBackendTarget {
    fn parse(input: ParseStream) -> Result<Self> {
        let for_token = input.parse()?;
        let la = input.lookahead1();
        let impl_token = if la.peek(token::Impl) {
            Some(input.parse()?)
        } else if la.peek(token::Brace) || la.peek(Ident) || la.peek(token::Colon2) {
            None
        } else {
            return Err(la.error());
        };
        let path = input.parse()?;
        Ok(Self {
            for_token,
            impl_token,
            path,
        })
    }
}

impl ToTokens for TemplateBackendTarget {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

enum TemplateNode {
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
            TemplateNode::DynamicText {
                brace_token,
                expr,
            }
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
        return Ok(ret);
    }
}

impl ToTokens for TemplateNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

enum TemplateAttribute {
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
        todo!()
    }
}

impl ToTokens for TemplateAttribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

pub(crate) fn template(input: TokenStream) -> TokenStream {
    let template_definition = parse_macro_input!(input as TemplateDefinition);
    quote! {
        #template_definition
    }.into()
}
