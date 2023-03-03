use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;

use crate::template::SlotType;

use super::template::Template;
use super::i18n::LocaleGroup;

struct ComponentAttr {
    items: Punctuated<ComponentAttrItem, token::Comma>,
}

impl Parse for ComponentAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let items = Punctuated::parse_terminated(input)?;
        Ok(Self { items })
    }
}

enum ComponentAttrItem {
    Backend {
        attr_name: Ident,
        #[allow(dead_code)]
        equal_token: token::Eq,
        impl_token: Option<token::Impl>,
        path: Path,
    },
    SlotData {
        attr_name: Ident,
        #[allow(dead_code)]
        equal_token: token::Eq,
        path: Path,
    },
    Translation {
        attr_name: Ident,
        #[allow(dead_code)]
        equal_token: token::Eq,
        name: Ident,
    },
}

impl Parse for ComponentAttrItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr_name: Ident = input.parse()?;
        let ret = match attr_name.to_string().as_str() {
            "Backend" => Self::Backend {
                attr_name,
                equal_token: input.parse()?,
                impl_token: input.parse()?,
                path: input.parse()?,
            },
            "SlotData" => Self::SlotData {
                attr_name,
                equal_token: input.parse()?,
                path: input.parse()?,
            },
            "Translation" => Self::Translation {
                attr_name,
                equal_token: input.parse()?,
                name: input.parse()?,
            },
            _ => {
                return Err(Error::new(attr_name.span(), "Unknown attribute parameter"));
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
    slot_kind: proc_macro2::TokenStream,
    slot_data_ty: proc_macro2::TokenStream,
    template: Result<Template>,
    template_field: Ident,
    template_ty: Type,
    locale_group: LocaleGroup,
}

impl ComponentBody {
    fn new(attr: ComponentAttr, mut inner: ItemStruct) -> Result<Self> {
        // generate backend type params and slot params
        let mut backend_attr = None;
        let mut slot_data_attr = None;
        let mut locale_group_name = None;
        for item in attr.items {
            match item {
                ComponentAttrItem::Backend {
                    attr_name,
                    impl_token,
                    path,
                    ..
                } => {
                    if backend_attr.is_some() {
                        return Err(Error::new(
                            attr_name.span(),
                            "Duplicated attribute parameter",
                        ));
                    }
                    backend_attr = Some((impl_token, path));
                }
                ComponentAttrItem::SlotData {
                    attr_name, path, ..
                } => {
                    if slot_data_attr.is_some() {
                        return Err(Error::new(
                            attr_name.span(),
                            "Duplicated attribute parameter",
                        ));
                    }
                    slot_data_attr = Some(path);
                }
                ComponentAttrItem::Translation { attr_name, name, .. } => {
                    if locale_group_name.is_some() {
                        return Err(Error::new(
                            attr_name.span(),
                            "Duplicated attribute parameter",
                        ));
                    }
                    locale_group_name = Some(name);
                }
            }
        }
        let backend_param = match &backend_attr {
            None => quote! { __MBackend },
            Some((Some(_), path)) => {
                let span = path.span();
                quote_spanned! {span=> __MBackend }
            }
            Some((None, path)) => {
                let span = path.span();
                quote_spanned! {span=> #path }
            }
        };
        let backend_param_in_impl = match backend_attr {
            None => Some(parse_quote! { __MBackend: maomi::backend::Backend }),
            Some((Some(_), path)) => {
                let span = path.span();
                Some(parse_quote_spanned! {span=> __MBackend: #path })
            }
            Some((None, _)) => None,
        };
        let slot_data_ty = match slot_data_attr {
            None => quote! { () },
            Some(path) => {
                let span = path.span();
                quote_spanned! {span=> #path }
            }
        };
        let locale_group = match locale_group_name {
            None => LocaleGroup::get_default(),
            Some(x) => LocaleGroup::get(&x.to_string()),
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

        // set a default component slot kind
        let mut slot_kind = quote! {
            maomi::node::NoneSlot
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
                            Ok(x) => x.gen_type(&m.mac.delimiter),
                            Err(_) => parse_quote! { () },
                        };
                        if let Ok(x) = &t {
                            match x.slot_type() {
                                SlotType::None => {}
                                SlotType::StaticSingle => {
                                    slot_kind = quote! {
                                        maomi::node::StaticSingleSlot
                                    };
                                }
                                SlotType::Dynamic => {
                                    slot_kind = quote! {
                                        maomi::node::DynamicSlot
                                    };
                                }
                            }
                        }
                        field.ty = parse_quote! {
                            maomi::template::Template<
                                #component_name,
                                #structure_ty,
                                #slot_kind<maomi::backend::tree::ForestTokenAddr, (maomi::backend::tree::ForestToken, maomi::prop::Prop<#slot_data_ty>)>,
                            >
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
            slot_kind,
            slot_data_ty,
            template,
            template_field: template_field.unwrap(),
            template_ty: template_ty.unwrap(),
            locale_group,
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
            slot_kind,
            slot_data_ty,
            template,
            template_field,
            template_ty,
            locale_group,
        } = self;

        // write the struct
        inner.to_tokens(tokens);

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
        let impl_type_params_without_backend_param = {
            let items = inner
                .generics
                .params
                .iter();
            quote! {
                <#(#items),*>
            }
        };

        // write the component template
        match template.as_ref() {
            Ok(template) => {
                let template_children = template.to_children(backend_param, locale_group);
                quote! {
                    impl #impl_type_params_without_backend_param maomi::template::ComponentSlotKind for #component_name {
                        type SlotChildren<C> = #slot_kind<maomi::backend::tree::ForestTokenAddr, C>;
                        type SlotData = #slot_data_ty;
                    }
                }.to_tokens(tokens);
                quote! {
                    impl #impl_type_params maomi::template::ComponentTemplate<#backend_param> for #component_name {
                        type TemplateField = maomi::template::Template<
                            Self,
                            Self::TemplateStructure,
                            Self::SlotChildren<(maomi::backend::tree::ForestToken, maomi::prop::Prop<Self::SlotData>)>,
                        >;
                        type TemplateStructure = #template_ty;

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
                        fn template_create_or_update<'__m_b>(
                            &'__m_b mut self,
                            __m_backend_context: &'__m_b maomi::BackendContext<#backend_param>,
                            __m_backend_element: &'__m_b mut maomi::backend::tree::ForestNodeMut<
                                <#backend_param as maomi::backend::Backend>::GeneralElement,
                            >,
                            __m_slot_fn: &mut dyn FnMut(
                                maomi::node::SlotChange<
                                    &mut maomi::backend::tree::ForestNodeMut<
                                        <#backend_param as maomi::backend::Backend>::GeneralElement,
                                    >,
                                    &maomi::backend::tree::ForestToken,
                                    &Self::SlotData,
                                >,
                            ) -> Result<(), maomi::error::Error>,
                        ) -> Result<(), maomi::error::Error>
                        where
                            Self: Sized,
                        {
                            let __m_event_self_weak = maomi::template::TemplateHelper::component_weak(
                                &self.#template_field,
                            ).unwrap();
                            let mut __m_slot_scopes = maomi::node::SlotKindTrait::update(&mut self.#template_field.__m_slot_scopes);
                            {
                                let __m_slot_scopes = &mut __m_slot_scopes;
                                let __m_self_owner_weak = self.#template_field.__m_self_owner_weak.as_ref().unwrap();
                                let __m_parent_element = __m_backend_element;
                                let __m_children = self.#template_field.__m_structure.as_mut();
                                if let Some(__m_children) = (#template_children)(__m_parent_element, __m_children)? {
                                    self.#template_field.__m_structure = Some(__m_children);
                                }
                            }
                            maomi::node::SlotKindUpdateTrait::finish(__m_slot_scopes, |_, (n, _)| {
                                __m_slot_fn(maomi::node::SlotChange::Removed(&n))?;
                                Ok(())
                            })?;
                            Ok(())
                        }
                    }
                }.to_tokens(tokens);
            }
            Err(err) => {
                err.to_compile_error().to_tokens(tokens);
            },
        }
    }
}

pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let component_attr = parse_macro_input!(attr as ComponentAttr);
    match ComponentBody::new(component_attr, parse_macro_input!(item as ItemStruct)) {
        Ok(component_body) => {
            quote! {
                #component_body
            }.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}
