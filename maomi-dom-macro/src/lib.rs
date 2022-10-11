#![recursion_limit = "128"]

use proc_macro::TokenStream;
use maomi_skin::style_sheet::StyleSheet;

mod css;
use css::DomStyleSheet;
mod element;
use element::{DomElementDefinition, DomElementDefinitionAttribute};

#[proc_macro]
pub fn dom_css(item: TokenStream) -> TokenStream {
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
