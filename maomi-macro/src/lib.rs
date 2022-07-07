#![recursion_limit = "128"]

use proc_macro::TokenStream;

mod component;
mod template;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    template::template(input.into()).into()
}
