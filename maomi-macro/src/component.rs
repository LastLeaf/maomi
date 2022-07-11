use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::*;
use syn::spanned::Spanned;

use super::template::Template;

enum ComponentAttr {
    None,
    ImplBackend {
        _for_token: token::For,
        _impl_token: token::Impl,
        path: Path,
    },
    Backend {
        _for_token: token::For,
        path: Path,
    },
}

impl Parse for ComponentAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ret = if input.is_empty() {
            Self::None
        } else {
            let _for_token = input.parse()?;
            let la = input.lookahead1();
            if la.peek(Ident) || la.peek(token::Colon2) {
                Self::Backend {
                    _for_token,
                    path: input.parse()?,
                }
            } else if la.peek(token::Impl) {
                Self::ImplBackend {
                    _for_token,
                    _impl_token: input.parse()?,
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
    template: Template,
    template_field: Ident,
    template_ty: Type,
}

impl Parse for ComponentBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inner: ItemStruct = input.parse()?;

        // find `template!` invoke
        let mut template = None;
        let mut template_field = None;
        let mut template_ty = None;
        if let Fields::Named(fields) = &mut inner.fields {
            for field in &mut fields.named {
                let mut has_template = false;
                if let Type::Macro(m) = &mut field.ty {
                    if m.mac.path.is_ident("template") {
                        if template.is_some() {
                            return Err(input
                                .error("a component struct should only contain one `template!`"));
                        }
                        has_template = true;
                    }
                }
                if has_template {
                    thread_local! {
                        static EMPTY_TY: Type = parse_str("()").unwrap();
                    }
                    if let Type::Macro(m) = &mut field.ty {
                        let tokens = m.mac.tokens.clone();
                        let t = Template::parse.parse2(tokens)?;
                        field.ty = t.gen_type(&m.mac.delimiter);
                        template = Some(t);
                        template_ty = Some(field.ty.clone());
                        template_field = field.ident.clone();
                    } else {
                        unreachable!()
                    }
                }
            }
        } else {
            return Err(input.error("a component struct must be a named struct"));
        }
        let template = if let Some(t) = template {
            t
        } else {
            return Err(input.error("a component struct must contain a `template!` field"));
        };

        Ok(Self {
            attr: ComponentAttr::None,
            inner,
            template,
            template_field: template_field.unwrap(),
            template_ty: template_ty.unwrap(),
        })
    }
}

impl ToTokens for ComponentBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attr, inner, template, template_field, template_ty } = self;

        // generate backend type params
        let backend_param = match attr {
            ComponentAttr::None => quote! { __Backend },
            ComponentAttr::ImplBackend { path, .. } => {
                let span = path.span();
                quote_spanned! {span=> __Backend }
            }
            ComponentAttr::Backend { path, .. } => {
                let span = path.span();
                quote_spanned! {span=> #path }
            },
        };
        let backend_param_in_impl = match attr {
            ComponentAttr::None => quote! { <__Backend: maomi::backend::Backend> },
            ComponentAttr::ImplBackend { path, .. } => {
                let span = path.span();
                quote_spanned! {span=> <__Backend: #path> }
            }
            ComponentAttr::Backend { .. } => {
                quote! { }
            },
        };

        // impl the component template
        let template_create = template.to_create(&backend_param);
        let template_update = template.to_update(&backend_param);
        let impl_component_template = quote! {
            impl #backend_param_in_impl maomi::component::ComponentTemplate<#backend_param> for HelloWorld {
                type TemplateField = #template_ty;
        
                #[inline]
                fn template(&self) -> &Self::TemplateField {
                    &self.#template_field
                }
        
                #[inline]
                fn template_mut(&mut self) -> &mut Self::TemplateField {
                    &mut self.#template_field
                }
        
                fn create(
                    &mut self,
                    __parent_element: &mut maomi::backend::tree::ForestNodeMut<<#backend_param as maomi::backend::Backend>::GeneralElement>,
                ) -> Result<maomi::backend::tree::ForestNodeRc<<#backend_param as maomi::backend::Backend>::GeneralElement>, maomi::error::Error>
                where
                    Self: Sized {
                    use maomi::backend::BackendGeneralElement;
                    let __backend_element = <#backend_param as maomi::backend::Backend>::GeneralElement::create_virtual_element(__parent_element)?;
                    #template_create
                    self.#template_field = maomi::component::Template::Structure {
                        dirty: false,
                        backend_element_token: __backend_element.token(),
                        backend_element: Box::new(__backend_element.clone()),
                        child_nodes: __child_nodes,
                    };
                    Ok(__backend_element)
                }
        
                fn apply_updates(
                    &mut self,
                    __backend_element: &mut maomi::backend::tree::ForestNodeMut<<#backend_param as maomi::backend::Backend>::GeneralElement>,
                ) -> Result<(), maomi::error::Error> {
                    match self.#template_field {
                        maomi::component::Template::Uninitialized => {
                            Ok(())
                        }
                        maomi::component::Template::Structure {
                            dirty: ref mut __dirty,
                            child_nodes: ref mut __child_nodes,
                            backend_element_token: ref __backend_element_token,
                            ..
                        } => {
                            if *__dirty {
                                *__dirty = false;
                                let mut __backend_element = __backend_element.borrow_mut_token(__backend_element_token);
                                #template_update
                            }
                            Ok(())
                        }
                    }
                }
            }        
        };

        quote! {
            #inner
            #impl_component_template
        }.to_tokens(tokens);
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
