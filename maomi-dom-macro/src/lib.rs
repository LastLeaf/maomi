#![recursion_limit = "128"]

use proc_macro::TokenStream;
use maomi_skin::parser::StyleSheet;

mod css;
use css::DomStyleSheet;

#[proc_macro]
pub fn dom_css(item: TokenStream) -> TokenStream {
    let ss = syn::parse_macro_input!(item as StyleSheet<DomStyleSheet>);
    quote::quote! {
        #ss
    }
    .into()
}
