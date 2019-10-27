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
struct TemplateDefinition {
    name: syn::Path,
    generics: Option<syn::Generics>,
    root: template::TemplateShadowRoot,
}
impl Parse for TemplateDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let format: syn::Ident = input.parse()?;
        let lookahead = input.lookahead1();
        let generics = if lookahead.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        let name = input.parse()?;
        let root = match format.to_string().as_str() {
            "xml" => xml_template::parse_template(input)?,
            "tmpl" => simple_template::parse_template(input)?,
            _ => return Err(Error::new(format.span(), "unrecognized template format"))
        };
        Ok(Self {
            name,
            generics,
            root,
        })
    }
}
impl ToTokens for TemplateDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { name, generics, root } = self;
        tokens.append_all(quote! {
            impl #generics #name {
                fn __template<B: Backend>(&self, __owner: &mut ComponentNodeRefMut<B>, __is_update: bool) -> Option<Vec<NodeRc<B>>> {
                    let shadow_root_fn = #root;
                    let sr = __owner.shadow_root_rc().clone();
                    let ret: Vec<NodeRc<B>> = shadow_root_fn(__owner, if __is_update { Some(&sr) } else { None });
                    if __is_update { None } else { Some(ret) }
                }
            }
            impl #generics ComponentTemplate for #name {
                fn template<B: Backend>(__owner: &mut ComponentNodeRefMut<B>, __is_update: bool) -> Option<Vec<NodeRc<B>>> where Self: Sized {
                    let rc = __owner.rc();
                    let __owner2 = unsafe { rc.borrow_mut_unsafe_with(__owner) };
                    __owner2.as_component::<#name>().__template(__owner, __is_update)
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
