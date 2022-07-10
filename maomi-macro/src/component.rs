use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::*;

use super::template::TemplateNode;

enum ComponentAttr {
    None,
    ImplBackend {
        impl_token: token::Impl,
        path: Path,
    },
    Backend {
        path: Path,
    },
}

impl Parse for ComponentAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ret = if input.is_empty() {
            Self::None
        } else {
            let la = input.lookahead1();
            if la.peek(Ident) || la.peek(token::Colon2) {
                Self::Backend {
                    path: input.parse()?,
                }
            } else if la.peek(token::Impl) {
                Self::ImplBackend {
                    impl_token: input.parse()?,
                    path: input.parse()?,
                }
            } else {
                return Err(la.error());
            }
        };
        Ok(ret)
    }
}

struct ComponentBody {
    attr: ComponentAttr,
    inner: ItemStruct,
    template: TemplateNode,
}

impl Parse for ComponentBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inner: ItemStruct = input.parse()?;

        // find `template!` invoke
        let mut template = None;
        if let Fields::Named(fields) = &mut inner.fields {
            for field in &mut fields.named {
                let mut has_template = false;
                if let Type::Macro(m) = &mut field.ty {
                    if m.mac.path.is_ident("template") {
                        if template.is_some() {
                            return Err(input.error("A component struct should only contain one `template!`"));
                        }
                        has_template = true;
                    }
                }
                if has_template {
                    thread_local! {
                        static EMPTY_TY: Type = parse_str("()").unwrap();
                    }
                    let ty = std::mem::replace(&mut field.ty, EMPTY_TY.with(|x| x.clone()));
                    if let Type::Macro(m) = ty {
                        let tokens = m.mac.tokens.clone();
                        let t = TemplateNode::parse.parse2(tokens)?;
                        field.ty = t.gen_type();
                        template = Some(t);
                    } else {
                        unreachable!()
                    }
                }
            }
        } else {
            return Err(input.error("A component struct must be a named struct"));
        }
        let template = if let Some(t) = template {
            t
        } else {
            return Err(input.error("A component struct must contain a `template!` field"));
        };

        Ok(Self {
            attr: ComponentAttr::None,
            inner,
            template,
        })
    }
}

impl ToTokens for ComponentBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let component_attr = parse_macro_input!(attr as ComponentAttr);
    let mut component_body = parse_macro_input!(item as ComponentBody);
    component_body.attr = component_attr;
    quote! {
        #component_body
    }
    .into()
}
