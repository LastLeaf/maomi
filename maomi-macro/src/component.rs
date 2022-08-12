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
    inner: ItemStruct,
    component_name: proc_macro2::TokenStream,
    backend_param: proc_macro2::TokenStream,
    backend_param_in_impl: Option<GenericParam>,
    template: Result<Template>,
    template_field: Ident,
    template_ty: Type,
}

impl ComponentBody {
    fn new(attr: ComponentAttr, mut inner: ItemStruct) -> Result<Self> {
        // generate backend type params
        let backend_param = match &attr {
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
        let backend_param_in_impl = match &attr {
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
                            Err(syn::Error::new(
                                m.span(),
                                "a component struct can only contain one `template!` field",
                            ))?;
                            continue;
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
                        let t = Template::parse.parse2(tokens);
                        let structure_ty = match t.as_ref() {
                            Ok(x) => x.gen_type(&backend_param, &m.mac.delimiter),
                            Err(_) => parse_quote! { () },
                        };
                        field.ty = parse_quote! {
                            maomi::template::Template<#component_name, #structure_ty, ()> // TODO slot data ty
                        };
                        template_ty = Some(structure_ty);
                        template = Some(t);
                        template_field = field.ident.clone();
                    } else {
                        unreachable!()
                    }
                }
            }
        } else {
            Err(syn::Error::new(
                inner.span(),
                "a component struct must be a named struct",
            ))?;
        }
        let template = if let Some(t) = template {
            t
        } else {
            return Err(syn::Error::new(
                inner.span(),
                "a component struct must contain a `template!` field",
            ));
        };

        Ok(Self {
            inner,
            component_name,
            backend_param,
            backend_param_in_impl,
            template,
            template_field: template_field.unwrap(),
            template_ty: template_ty.unwrap(),
        })
    }
}

impl ToTokens for ComponentBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            inner,
            component_name,
            backend_param,
            backend_param_in_impl,
            template,
            template_field,
            template_ty,
        } = self;

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
        let impl_component_template = match template.as_ref() {
            Ok(template) => {
                let template_create = template.to_create(&backend_param);
                let template_update = template.to_update(&backend_param);
                quote! {
                    impl #impl_type_params maomi::template::ComponentTemplate<#backend_param> for #component_name {
                        type TemplateField = maomi::template::Template<Self, Self::TemplateStructure, Self::SlotData>;
                        type TemplateStructure = #template_ty;
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
                            let __m_parent_element = __m_backend_element;
                            let __m_subtree_status = &self.#template_field.__m_root_subtree_status;
                            self.#template_field.__m_structure = Some(#template_create);
                            Ok(__m_slot)
                        }

                        #[inline]
                        fn template_update<'__m_b>(
                            &'__m_b mut self,
                            __m_is_subtree_update: bool,
                            __m_backend_context: &'__m_b maomi::BackendContext<#backend_param>,
                            __m_backend_element: &'__m_b mut maomi::backend::tree::ForestNodeMut<
                                <#backend_param as maomi::backend::Backend>::GeneralElement,
                            >,
                            __m_slot_fn: impl FnMut(
                                maomi::node::SlotChange<
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
                            let __m_parent_element = __m_backend_element;
                            let __m_children = self
                                .#template_field
                                .__m_structure
                                .as_mut()
                                .ok_or(maomi::error::Error::TreeNotCreated)?;
                            #template_update
                            Ok(())
                        }
                    }
                }
            }
            Err(err) => err.to_compile_error(),
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
    match ComponentBody::new(component_attr, parse_macro_input!(item as ItemStruct)) {
        Ok(component_body) => quote! {
            #component_body
        }
        .into(),
        Err(err) => err.to_compile_error().into(),
    }
}
