#![recursion_limit = "128"]

use proc_macro::TokenStream;

mod component;
mod template;
mod i18n;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn i18n(item: TokenStream) -> TokenStream {
    let content = syn::parse_macro_input!(item as i18n::mac::I18nArgs);
    quote::quote!(#content).into()
}
