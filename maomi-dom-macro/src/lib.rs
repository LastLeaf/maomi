#![recursion_limit = "128"]

use maomi_skin::style_sheet::StyleSheet;
use proc_macro::TokenStream;

mod css;
use css::DomStyleSheet;
mod element;
use element::{DomElementDefinition, DomElementDefinitionAttribute, DomDefineAttribute};

#[proc_macro]
pub fn stylesheet(item: TokenStream) -> TokenStream {
    let ss = syn::parse_macro_input!(item as StyleSheet<DomStyleSheet>);
    quote::quote! {
        #ss
    }
    .into()
}

#[proc_macro_attribute]
pub fn dom_element_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = syn::parse_macro_input!(attr as DomElementDefinitionAttribute);
    let def = syn::parse_macro_input!(item as DomElementDefinition);
    quote::quote! {
        #def
    }
    .into()
}

/// Define a custom DOM attribute.
/// 
/// It can be used in `attr:xxx=""` syntax.
#[proc_macro]
pub fn dom_define_attribute(item: TokenStream) -> TokenStream {
    let def = syn::parse_macro_input!(item as DomDefineAttribute);
    quote::quote! {
        #def
    }
    .into()
}
