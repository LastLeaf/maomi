use quote::*;
use syn::parse::*;
use syn::*;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

fn add_global_attrs(
    fields: &mut Punctuated<Field, token::Comma>,
) {
    let vis: Visibility = parse_quote! { pub };
    let span = vis.span();
    let mut add_attr = |field_name, ty| {
        let ident = Ident::new(field_name, span);
        fields.push(Field {
            attrs: Vec::with_capacity(0),
            vis: vis.clone(),
            ident: Some(ident.clone()),
            colon_token: Default::default(),
            ty,
        });
    };
    add_attr("id", parse_quote! { attribute!(&str in web_sys::Element) });
    add_attr("title", parse_quote! { attribute!(&str in web_sys::HtmlElement) }); // FIXME use LocaleStr
    add_attr("hidden", parse_quote! { attribute!(bool in web_sys::HtmlElement) });
    add_attr("touch_start", parse_quote! { event!(event::touch::TouchStart) });
    add_attr("touch_move", parse_quote! { event!(event::touch::TouchMove) });
    add_attr("touch_end", parse_quote! { event!(event::touch::TouchEnd) });
    add_attr("touch_cancel", parse_quote! { event!(event::touch::TouchCancel) });
    add_attr("mouse_down", parse_quote! { event!(event::mouse::MouseDown) });
    add_attr("mouse_up", parse_quote! { event!(event::mouse::MouseUp) });
    add_attr("mouse_move", parse_quote! { event!(event::mouse::MouseMove) });
    add_attr("mouse_enter", parse_quote! { event!(event::mouse::MouseEnter) });
    add_attr("mouse_leave", parse_quote! { event!(event::mouse::MouseLeave) });
    add_attr("click", parse_quote! { event!(event::mouse::Click) });
    add_attr("tap", parse_quote! { event!(event::tap::Tap) });
    add_attr("long_tap", parse_quote! { event!(event::tap::LongTap) });
    add_attr("cancel_tap", parse_quote! { event!(event::tap::CancelTap) });
    add_attr("scroll", parse_quote! { event!(event::scroll::Scroll) });
    add_attr("animation_start", parse_quote! { event!(event::animation::AnimationStart) });
    add_attr("animation_iteration", parse_quote! { event!(event::animation::AnimationIteration) });
    add_attr("animation_end", parse_quote! { event!(event::animation::AnimationEnd) });
    add_attr("animation_cancel", parse_quote! { event!(event::animation::AnimationCancel) });
    add_attr("transition_run", parse_quote! { event!(event::transition::TransitionRun) });
    add_attr("transition_start", parse_quote! { event!(event::transition::TransitionStart) });
    add_attr("transition_end", parse_quote! { event!(event::transition::TransitionEnd) });
    add_attr("transition_cancel", parse_quote! { event!(event::transition::TransitionCancel) });
    // FIXME add aria properties
    add_attr("aria_hidden", parse_quote! { attribute!(&str) });
}

enum Attr {
    Normal {
        ty_name: Type,
        dom_element_name: Path,
        ty: Path,
    },
    Raw {
        ty_name: Type,
        ty: Path,
    },
    Binding {
        ty_name: Type,
        dom_element_name: Path,
        ty: Path,
        event: LitStr,
        cb: ExprClosure,
    },
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty_name: Type = input.parse()?;
        let span = ty_name.span();
        let s = ty_name.to_token_stream().to_string();
        if input.is_empty() {
            let ty = match s.as_str() {
                "& str" => parse_quote_spanned! {span=> DomStrAttr },
                _ => {
                    return Err(Error::new(span, "unknown raw attribute type"))
                }
            };
            return Ok(Self::Raw { ty_name, ty });
        }
        let _: token::In = input.parse()?;
        let dom_element_name = input.parse()?;
        let ty = match s.as_str() {
            "& str" => parse_quote_spanned! {span=> DomStrAttr },
            "bool" => parse_quote_spanned! {span=> DomBoolAttr },
            "u32" => parse_quote_spanned! {span=> DomU32Attr },
            "i32" => parse_quote_spanned! {span=> DomI32Attr },
            "f64" => parse_quote_spanned! {span=> DomF64Attr },
            _ => {
                return Err(Error::new(span, "unknown attribute type"))
            }
        };
        if input.is_empty() {
            return Ok(Self::Normal {
                ty_name,
                dom_element_name,
                ty,
            });
        }
        let _: token::While = input.parse()?;
        let event = input.parse()?;
        let cb = input.parse()?;
        let span = ty.span();
        let ty = match s.as_str() {
            "& str" => parse_quote_spanned! {span=> DomBindingStrAttr },
            "bool" => parse_quote_spanned! {span=> DomBindingBoolAttr },
            "f64" => parse_quote_spanned! {span=> DomBindingF64Attr },
            _ => {
                return Err(Error::new(span, "unknown binding attribute type"))
            }
        };
        Ok(Self::Binding {
            ty_name,
            dom_element_name,
            ty,
            event,
            cb,
        })
    }
}

impl Attr {
    fn ty(&self) -> Path {
        match self {
            Self::Normal { ty, .. } => ty.clone(),
            Self::Raw { ty, .. } => ty.clone(),
            Self::Binding { ty, .. } => ty.clone(),
        }
    }

    fn generate_dom_setter(
        &self,
        tag_name: &Ident,
        field_name: &Ident,
        tokens: &mut proc_macro2::TokenStream,
    ) -> Ident {
        match self {
            Self::Normal { ty_name, dom_element_name, .. } | Self::Binding { ty_name, dom_element_name, .. } => {
                let span = field_name.span();
                let dom_setter_name = Ident::new(&format!("dom_setter_{}_{}", tag_name.to_string(), field_name.to_string().trim_start_matches("r#")), span);
                let dom_element_fn_name = Ident::new(&format!("set_{}", field_name.to_string().trim_start_matches("r#")), span);
                tokens.append_all(quote_spanned! {span=>
                    #[inline]
                    #[allow(non_snake_case)]
                    fn #dom_setter_name(elem: &web_sys::HtmlElement, v: #ty_name) {
                        #dom_element_name::#dom_element_fn_name(elem.unchecked_ref(), v.into());
                    }
                });
                dom_setter_name
            }
            Self::Raw { ty_name, .. } => {
                let span = field_name.span();
                let field_name_str = field_name.to_string();
                let dom_setter_name = Ident::new(&format!("dom_setter_{}_{}", tag_name.to_string(), field_name.to_string().trim_start_matches("r#")), span);
                tokens.append_all(quote_spanned! {span=>
                    #[inline]
                    #[allow(non_snake_case)]
                    fn #dom_setter_name(elem: &web_sys::HtmlElement, v: #ty_name) {
                        elem.set_attribute(#field_name_str, v.into()).ok();
                    }
                });
                dom_setter_name
            }
        }
    }
}

pub(crate) struct DomElementDefinitionAttribute {
    // empty
}

impl Parse for DomElementDefinitionAttribute {
    fn parse(_: ParseStream) -> Result<Self> {
        Ok(Self {})
    }
}

pub(crate) struct DomElementDefinition {
    s: ItemStruct,
    attrs: Vec<(Ident, String, Attr)>,
    events: Vec<(Ident, String)>,
}

impl Parse for DomElementDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut s: ItemStruct = input.parse()?;
        let mut attrs = vec![];
        let mut events = vec![];
        if let Fields::Named(fields) = &mut s.fields {
            add_global_attrs(&mut fields.named);
            for field in &mut fields.named {
                if let Type::Macro(m) = field.ty.clone() {
                    let field_name = field.ident.clone().unwrap();
                    let attr_name = field_name.to_string().trim_start_matches("r#").to_string();
                    field.ident = Some(field_name.clone());
                    if m.mac.path.is_ident("attribute") {
                        let field_doc_comment = format!(r#"The `{}` attribute."#, attr_name);
                        field.attrs.push(parse_quote! {
                            #[doc = #field_doc_comment]
                        });
                        let tokens = m.mac.tokens.clone();
                        let attr = Attr::parse.parse2(tokens)?;
                        field.ty = Type::Path(TypePath { qself: None, path: attr.ty() });
                        attrs.push((field_name, attr_name, attr));
                    } else if m.mac.path.is_ident("event") {
                        let field_doc_comment = format!(r#"The `{}` event."#, attr_name.replace('_', ""));
                        field.attrs.push(parse_quote! {
                            #[doc = #field_doc_comment]
                        });
                        let span = m.mac.span();
                        let tokens = m.mac.tokens.clone();
                        let p = Path::parse.parse2(tokens)?;
                        let ty = parse_quote_spanned! {span=>
                            DomEvent<#p>
                        };
                        field.ty = Type::Path(ty);
                        events.push((field_name, attr_name));
                    } else {
                        return Err(Error::new(m.mac.span(), "unknown macro"))
                    }
                }
            }
            let span = s.ident.span();
            fields.named.push(Field {
                attrs: Vec::with_capacity(0),
                vis: Visibility::Inherited,
                ident: Some(Ident::new("backend_element_token", span)),
                colon_token: Default::default(),
                ty: parse_quote! { maomi::backend::tree::ForestToken },
            });
            fields.named.push(Field {
                attrs: vec![parse_quote! {
                    #[doc = "The `class` of the element."]
                }],
                vis: parse_quote! { pub },
                ident: Some(Ident::new("class", span)),
                colon_token: Default::default(),
                ty: parse_quote! { DomClassList },
            });
            fields.named.push(Field {
                attrs: vec![parse_quote! {
                    #[doc = "The `style` of the element."]
                }],
                vis: parse_quote! { pub },
                ident: Some(Ident::new("style", span)),
                colon_token: Default::default(),
                ty: parse_quote! { DomStyleList },
            });
            fields.named.push(Field {
                attrs: Vec::with_capacity(0),
                vis: Visibility::Inherited,
                ident: Some(Ident::new("dom_elem_lazy", span)),
                colon_token: Default::default(),
                ty: parse_quote! { std::cell::UnsafeCell<dom_state_ty!(web_sys::Element, (), RematchedDomElem)> },
            });
        } else {
            return Err(Error::new(s.span(), "expected named struct"));
        }
        Ok(Self {
            s,
            attrs,
            events,
        })
    }
}

impl ToTokens for DomElementDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = &self.s;
        let tag_name = &s.ident;
        let tag_name_str = tag_name.to_string();
        let struct_doc_comment = format!("The HTML `<{}>` element.", tag_name);
        let attrs_init = self.attrs.iter().map(|(field_name, attr_name, attr)| {
            let dom_setter_name = attr.generate_dom_setter(tag_name, field_name, tokens);
            let ty = attr.ty();
            quote! {
                #field_name: #ty {
                    inner: Default::default(),
                    f: #dom_setter_name,
                    #[cfg(feature = "prerendering")]
                    attr_name: #attr_name,
                },
            }
        }).collect::<Box<_>>();
        let events_init = self.events.iter().map(|(ev, _)| {
            quote! {
                #ev: Default::default(),
            }
        });
        let binding_props_init = self.attrs.iter().filter_map(|(field_name, _attr_name, attr)| {
            if let Attr::Binding { ty_name, event, cb, .. } = attr {
                let span = ty_name.span();
                Some(quote_spanned! {span=>
                    let binding_value_rc = self.#field_name.inner.clone();
                    let cb = #cb;
                    init_binding_prop(dom_element, #event, move |ev: web_sys::Event| {
                        if let Some(target) = ev.target() {
                            let binding_value: &mut BindingValue<_> = &mut binding_value_rc.borrow_mut();
                            cb(binding_value, ev.unchecked_ref(), target.unchecked_ref());
                        }
                    });
                })
            } else {
                None
            }
        });
        tokens.append_all(quote! {
            #[doc = #struct_doc_comment]
            #[allow(non_camel_case_types)]
            #s

            impl #tag_name {
                #[inline]
                fn init_binding_props(&mut self, dom_element: &mut DomElement) {
                    #(#binding_props_init)*
                }
            }

            impl DomElementBase for #tag_name {
                #[inline]
                fn dom_element_lazy(&self) -> &std::cell::UnsafeCell<dom_state_ty!(web_sys::Element, (), RematchedDomElem)> {
                    &self.dom_elem_lazy
                }
            }

            impl BackendComponent<DomBackend> for #tag_name {
                type SlotData = ();
                type UpdateTarget = Self;
                type UpdateContext = DomElement;
            
                #[inline]
                fn init<'b>(
                    _backend_context: &'b BackendContext<DomBackend>,
                    owner: &'b mut ForestNodeMut<DomGeneralElement>,
                    _owner_weak: &'b Box<dyn OwnerWeak>,
                ) -> Result<(Self, ForestNodeRc<DomGeneralElement>), Error>
                where
                    Self: Sized,
                {
                    thread_local! {
                        static tag_name: &'static MaybeJsStr = MaybeJsStr::new_leaked(#tag_name_str);
                    }
                    let elem = tag_name.with(|m| owner.create_dom_element_by_tag_name(m));
                    let backend_element = crate::DomGeneralElement::wrap_dom_element(owner, &elem);
                    let this = Self {
                        backend_element_token: backend_element.token(),
                        class: DomClassList::new(match &elem {
                            DomState::Normal(x) => DomState::Normal(x.class_list()),
                            #[cfg(feature = "prerendering")]
                            DomState::Prerendering(_) => DomState::Prerendering(()),
                            #[cfg(feature = "prerendering-apply")]
                            DomState::PrerenderingApply(_) => DomState::PrerenderingApply(()),
                        }),
                        style: DomStyleList::new(),
                        #(#attrs_init)*
                        #(#events_init)*
                        dom_elem_lazy: std::cell::UnsafeCell::new(DomGeneralElement::to_lazy(elem)),
                    };
                    Ok((this, backend_element))
                }
            
                #[inline]
                fn create<'b>(
                    &'b mut self,
                    _backend_context: &'b BackendContext<DomBackend>,
                    owner: &'b mut ForestNodeMut<DomGeneralElement>,
                    update_fn: Box<dyn 'b + FnOnce(&mut Self, &mut Self::UpdateContext)>,
                    slot_fn: &mut dyn FnMut(
                        &mut ForestNodeMut<DomGeneralElement>,
                        &ForestToken,
                        &Self::SlotData,
                    ) -> Result<(), Error>,
                ) -> Result<(), Error> {
                    let mut node = owner.borrow_mut_token(&self.backend_element_token).ok_or(Error::TreeNodeReleased)?;
                    let dom_element = &mut DomGeneralElement::as_dom_element_mut(&mut node).unwrap();
                    self.init_binding_props(dom_element);
                    update_fn(self, dom_element);
                    slot_fn(&mut node, &self.backend_element_token, &())?;
                    Ok(())
                }
            
                #[inline]
                fn apply_updates<'b>(
                    &'b mut self,
                    _backend_context: &'b BackendContext<DomBackend>,
                    owner: &'b mut ForestNodeMut<<DomBackend as maomi::backend::Backend>::GeneralElement>,
                    update_fn: Box<dyn 'b + FnOnce(&mut Self, &mut Self::UpdateContext)>,
                    slot_fn: &mut dyn FnMut(
                        SlotChange<&mut ForestNodeMut<DomGeneralElement>, &ForestToken, &Self::SlotData>,
                    ) -> Result<(), Error>,
                ) -> Result<(), Error> {
                    let mut node = owner.borrow_mut_token(&self.backend_element_token).ok_or(Error::TreeNodeReleased)?;
                    update_fn(self, &mut DomGeneralElement::as_dom_element_mut(&mut node).unwrap());
                    slot_fn(SlotChange::Unchanged(&mut node, &self.backend_element_token, &()))?;
                    Ok(())
                }
            }

            impl SupportBackend for #tag_name {
                type Target = Box<Self>;
                type SlotChildren<C> = StaticSingleSlot<ForestTokenAddr, C>;
            }
        });
    }
}
