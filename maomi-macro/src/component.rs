use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::*;

pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    // collect special attributes
    let mut properties = vec![];
    let mut events = vec![];
    if let Fields::Named(fields) = &mut input.fields {
        for field in &mut fields.named {
            if let Some(index) = field.attrs.iter().position(|attr| {
                if let AttrStyle::Outer = attr.style {
                    if attr.path.is_ident("property") {
                        return true;
                    }
                }
                false
            }) {
                properties.push(field.attrs.remove(index));
            }
            if let Some(index) = field.attrs.iter().position(|attr| {
                if let AttrStyle::Outer = attr.style {
                    if attr.path.is_ident("event") {
                        return true;
                    }
                }
                false
            }) {
                events.push(field.attrs.remove(index));
            }
        }
    }

    // generate property impls
    // TODO

    // concat output
    quote! {
        #input
    }
    .into()
}
