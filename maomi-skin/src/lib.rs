#![recursion_limit = "128"]

use proc_macro::TokenStream;

mod parser;
mod dom;

trait StyleSheetConstructor {
    type PropertyValue;
    type FontFacePropertyValue;

    fn construct_sheet();
}

#[proc_macro]
pub fn dom_css(item: TokenStream) -> TokenStream {
    parser::parse::<dom::DomStyleSheet>(item.into()).into()
}
