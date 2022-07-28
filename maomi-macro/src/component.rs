use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

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
        let Self {
            attr,
            inner,
            template,
            template_field,
            template_ty,
        } = self;

        // generate backend type params
        let backend_param = match attr {
            ComponentAttr::None => quote! { __MBackend },
            ComponentAttr::ImplBackend { path, .. } => {
                let span = path.span();
                quote_spanned! {span=> __MBackend }
            }
            ComponentAttr::Backend { path, .. } => {
                let span = path.span();
                quote_spanned! {span=> #path }
            }
        };
        let backend_param_in_impl = match attr {
            ComponentAttr::None => Some(parse_quote! { __MBackend: maomi::backend::Backend }),
            ComponentAttr::ImplBackend { path, .. } => {
                let span = path.span();
                Some(parse_quote_spanned! {span=> __MBackend: #path })
            }
            ComponentAttr::Backend { .. } => None,
        };

        // find component name and type params
        let component_name = {
            let component_name_ident = &inner.ident;
            let component_type_params = inner.generics.params.iter().map(|x| {
                let span = x.span();
                match x {
                    GenericParam::Type(x) => {
                        let x = x.ident.clone();
                        quote_spanned! {span=> #x }
                    }
                    GenericParam::Lifetime(x) => {
                        let x = x.lifetime.clone();
                        quote_spanned! {span=> #x }
                    }
                    GenericParam::Const(x) => {
                        let x = x.ident.clone();
                        quote_spanned! {span=> #x }
                    }
                }
            });
            quote! {
                #component_name_ident<#(#component_type_params),*>
            }
        };

        // find generics for impl
        let impl_type_params = {
            let items = inner
                .generics
                .params
                .iter()
                .chain(backend_param_in_impl.as_ref());
            quote! {
                <#(#items),*>
            }
        };

        // impl the component template
        let template_create = template.to_create(&backend_param);
        let template_update = template.to_update(&backend_param);
        let impl_component_template = quote! {
            impl #impl_type_params maomi::template::ComponentTemplate<#backend_param> for #component_name {
                type TemplateField = #template_ty;
                type SlotData = ();

                #[inline]
                fn template(&self) -> &Self::TemplateField {
                    &self.#template_field
                }

                #[inline]
                fn template_mut(&mut self) -> &mut Self::TemplateField {
                    &mut self.#template_field
                }

                #[inline]
                fn template_init(&mut self, __m_init: maomi::template::TemplateInit<#component_name>) {
                    self.#template_field.init(__m_init);
                }

                #[inline]
                fn template_create<'__m_b, __MSlot>(
                    &'__m_b mut self,
                    __m_backend_context: &'__m_b maomi::BackendContext<#backend_param>,
                    __m_backend_element: &'__m_b mut maomi::backend::tree::ForestNodeMut<
                        <#backend_param as maomi::backend::Backend>::GeneralElement,
                    >,
                    __m_slot_fn: impl FnMut(
                        &mut maomi::backend::tree::ForestNodeMut<
                            <#backend_param as maomi::backend::Backend>::GeneralElement,
                        >,
                        &Self::SlotData,
                    ) -> Result<__MSlot, maomi::error::Error>,
                ) -> Result<maomi::node::SlotChildren<__MSlot>, maomi::error::Error>
                where
                    Self: Sized,
                {
                    let mut __m_slot: maomi::node::SlotChildren<__MSlot> = maomi::node::SlotChildren::None;
                    let mut __m_parent_element = __m_backend_element;
                    self.#template_field.structure = Some(#template_create);
                    Ok(__m_slot)
                }

                #[inline]
                fn template_update<'__m_b>(
                    &'__m_b mut self,
                    __m_backend_context: &'__m_b maomi::BackendContext<#backend_param>,
                    __m_backend_element: &'__m_b mut maomi::backend::tree::ForestNodeMut<
                        <#backend_param as maomi::backend::Backend>::GeneralElement,
                    >,
                    __m_slot_fn: impl FnMut(
                        maomi::diff::ListItemChange<
                            &mut maomi::backend::tree::ForestNodeMut<
                                <#backend_param as maomi::backend::Backend>::GeneralElement,
                            >,
                            &Self::SlotData,
                        >,
                    ) -> Result<(), maomi::error::Error>,
                ) -> Result<(), maomi::error::Error>
                where
                    Self: Sized,
                {
                    // update tree
                    let mut __m_parent_element = __m_backend_element;
                    let __m_children = self
                        .#template_field
                        .structure
                        .as_mut()
                        .ok_or(maomi::error::Error::TreeNotCreated)?;
                    #template_update
                    Ok(())
                }
            }
        };

        quote! {
            #inner
            #impl_component_template
        }
        .to_tokens(tokens);
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
