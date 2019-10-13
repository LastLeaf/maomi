#![recursion_limit="128"]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;
use syn::parse::*;

mod template;
mod simple_template;
mod xml_template;

#[derive(Clone)]
struct ComponentProperty {
    method: syn::ImplItemMethod,
}
impl Parse for ComponentProperty {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            method: input.parse()?
        })
    }
}
impl ToTokens for ComponentProperty {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut getter = self.method.clone();
        let mut setter = self.method.clone();
        getter.sig.ident = format_ident!("property_{}", getter.sig.ident);
        setter.sig.ident = format_ident!("set_property_{}", setter.sig.ident);
        tokens.append_all(quote! {
            #getter
            #setter
        });
    }
}
#[proc_macro_attribute]
pub fn property(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ComponentProperty);
    let ret = quote! {
        #input
    };
    TokenStream::from(ret)
}

#[derive(Clone)]
enum ComponentStructOrImpl {
    Struct(ItemStruct, Vec<(Ident, Type)>),
    Impl(ItemImpl),
}
impl Parse for ComponentStructOrImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![impl]) {
            Ok(Self::Impl(input.parse()?))
        } else {
            let mut x: ItemStruct = input.parse()?;
            let mut get_set_fields = vec![];
            if let Fields::Named(x) = &mut x.fields {
                for field in x.named.iter_mut() {
                    let mut has_property = None;
                    for (index, attr) in field.attrs.iter().enumerate() {
                        if let AttrStyle::Outer = attr.style {
                            let matched = match attr.path.get_ident() {
                                Some(x) => {
                                    x.to_string() == "property"
                                },
                                None => false
                            };
                            if matched { has_property = Some(index); }
                        }
                    }
                    if let Some(index) = has_property {
                        field.attrs.remove(index);
                        let name = field.ident.clone().unwrap();
                        let ty = field.ty.clone();
                        get_set_fields.push((name, ty));
                    }
                }
            }
            Ok(Self::Struct(x, get_set_fields))
        }
    }
}
impl ToTokens for ComponentStructOrImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Struct(x, get_set_fields) => {
                let struct_name = &x.ident;
                let get_set_fields: Vec<_> = get_set_fields.iter().map(|(name, ty)| {
                    let getter = format_ident!("property_{}", name);
                    let setter = format_ident!("set_property_{}", name);
                    quote! {
                        pub fn #getter(&self) -> &#ty {
                            &self.#name
                        }
                        pub fn #setter<T: Into<#ty>>(&mut self, v: T) {
                            self.#name = v.into();
                        }
                    }
                }).collect();
                tokens.append_all(quote! {
                    #x
                    impl #struct_name {
                        #(#get_set_fields)*
                    }
                });
            },
            Self::Impl(x) => {
                let name = &x.self_ty;
                let mut has_created = false;
                let mut has_attached = false;
                let mut has_ready = false;
                let mut has_moved = false;
                let mut has_detached = false;
                for x in x.items.iter() {
                    if let syn::ImplItem::Method(x) = x {
                        match x.sig.ident.to_string().as_str() {
                            "created" => has_created = true,
                            "attached" => has_attached = true,
                            "ready" => has_ready = true,
                            "moved" => has_moved = true,
                            "detached" => has_detached = true,
                            _ => { }
                        }
                    }
                }
                let created = if has_created {
                    quote! { fn created(&mut self) { self.created() } }
                } else {
                    quote! { }
                };
                let attached = if has_attached {
                    quote! { fn attached(&mut self) { self.attached() } }
                } else {
                    quote! { }
                };
                let ready = if has_ready {
                    quote! { fn ready(&mut self) { self.ready() } }
                } else {
                    quote! { }
                };
                let moved = if has_moved {
                    quote! { fn moved(&mut self) { self.moved() } }
                } else {
                    quote! { }
                };
                let detached = if has_detached {
                    quote! { fn detached(&mut self) { self.detached() } }
                } else {
                    quote! { }
                };
                tokens.append_all(quote! {
                    #x
                    impl Component for #name {
                        fn new() -> Self { Self::new() }
                        #created
                        #attached
                        #ready
                        #moved
                        #detached
                    }
                });
            },
        }
    }
}
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ComponentStructOrImpl);
    let ret = quote! {
        #input
    };
    TokenStream::from(ret)
}

#[derive(Clone)]
struct TemplateDefinition {
    name: syn::Ident,
    root: template::TemplateShadowRoot,
}
impl Parse for TemplateDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let format: syn::Ident = input.parse()?;
        let name = input.parse()?;
        let root = match format.to_string().as_str() {
            "xml" => xml_template::parse_template(input)?,
            "tmpl" => simple_template::parse_template(input)?,
            _ => return Err(Error::new(format.span(), "unrecognized template format"))
        };
        Ok(Self {
            name,
            root,
        })
    }
}
impl ToTokens for TemplateDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { name, root } = self;
        tokens.append_all(quote! {
            impl ComponentTemplate for #name {
                fn template<B: Backend>(__owner: &mut ComponentNodeRefMut<B>, __is_update: bool) -> Option<Vec<NodeRc<B>>> where Self: Sized {
                    // let (init_fn, update_fn) = #root;
                    // if __is_update {
                    //     update_fn(__owner, &__owner.shadow_root_rc().clone());
                    //     None
                    // } else {
                    //     Some(init_fn(__owner))
                    // }
                    __template_sample(__owner, __is_update)
                }
            }
        });
    }
}
#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TemplateDefinition);
    let ret = quote! {
        #input
    };
    TokenStream::from(ret)
}
