#![recursion_limit = "128"]

use proc_macro::TokenStream;

mod component;
mod template;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component(attr.into(), item.into()).into()
}
