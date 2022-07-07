use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

struct TemplateDefinition {
    path: Path,
    backend_target: Option<TemplateBackendTarget>,
    brace_token: token::Brace,
    children: Vec<TemplateNode>,
}

impl Parse for TemplateDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        let la = input.lookahead1();
        let backend_target = if la.peek(token::In) {
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
            path,
            backend_target,
            brace_token,
            children,
        })
    }
}

impl ToTokens for TemplateDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            path,
            backend_target,
            children,
            ..
        } = self;

        // get the backend name
        let backend = match backend_target {
            None => {
                let span = path.span();
                quote_spanned! { span => __Backend }
            }
            Some(backend_target) => {
                let TemplateBackendTarget {
                    impl_token,
                    backend_path,
                    ..
                } = backend_target;
                let span = backend_path.span();
                match impl_token {
                    None => quote_spanned! { span => #backend_path },
                    Some(_) => quote_spanned! { span => __Backend },
                }
            }
        };

        // the impl of Component<#backend>
        let content = quote! {
            fn create(
                __backend_element: &mut maomi::backend::tree::ForestNodeMut<'_, <#backend as maomi::backend::Backend>::GeneralElement>,
            ) -> Result<Self, maomi::error::Error> {
                unimplemented!()
            }

            fn apply_updates(
                &mut self,
                __backend_element: &mut maomi::backend::tree::ForestNodeMut<'_, <#backend as maomi::backend::Backend>::GeneralElement>,
            ) -> Result<(), maomi::error::Error> {
                unimplemented!()
            }
        };

        // wrap the trait
        let ret = match backend_target {
            None => quote! {
                impl<__Backend: maomi::backend::Backend> maomi::component::ComponentTemplate<#backend> for #path {
                    #content
                }
            },
            Some(backend_target) => {
                let TemplateBackendTarget {
                    impl_token,
                    backend_path,
                    ..
                } = backend_target;
                match impl_token {
                    None => quote! {
                        impl maomi::component::ComponentTemplate<#backend> for #path {
                            #content
                        }
                    },
                    Some(_) => quote! {
                        impl<__Backend: #backend_path> maomi::component::ComponentTemplate<#backend> for #path {
                            #content
                        }
                    },
                }
            }
        };
        ret.to_tokens(tokens);
    }
}

struct TemplateBackendTarget {
    #[allow(dead_code)]
    in_token: token::In,
    impl_token: Option<token::Impl>,
    backend_path: Path,
}

impl Parse for TemplateBackendTarget {
    fn parse(input: ParseStream) -> Result<Self> {
        let in_token = input.parse()?;
        let la = input.lookahead1();
        let impl_token = if la.peek(token::Impl) {
            Some(input.parse()?)
        } else if la.peek(token::Brace) || la.peek(Ident) || la.peek(token::Colon2) {
            None
        } else {
            return Err(la.error());
        };
        let backend_path = input.parse()?;
        Ok(Self {
            in_token,
            impl_token,
            backend_path,
        })
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

pub(crate) fn template(input: TokenStream) -> TokenStream {
    let template_definition = parse_macro_input!(input as TemplateDefinition);
    quote! {
        #template_definition
    }
    .into()
}
