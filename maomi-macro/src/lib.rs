#![recursion_limit = "128"]

use proc_macro::TokenStream;

mod component;
mod template;
mod i18n;

/// Define a component struct.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component(attr.into(), item.into()).into()
}

/// Translate a string with default translation group.
/// 
/// The basic usage:
/// 
/// ```rust
/// i18n!("The string to translate.");
/// ```
/// 
/// Furthermore, this macro works like `println!` and `format!` .
/// However, the dynamic components must also be translated.
/// 
/// ```rust
/// let my_name = LocaleString::translated("Alice");
/// i18n!("My name is {}.", my_name);
/// ```
/// 
#[proc_macro]
pub fn i18n(item: TokenStream) -> TokenStream {
    let content = syn::parse_macro_input!(item as i18n::mac::I18nArgs);
    quote::quote!(#content).into()
}

/// Define translation group.
/// 
/// This will define another macro similar to `i18n!` but use another translation group.
/// Usage:
/// 
/// ```rust
/// i18n_group!(my_group_name as my_macro_name);
/// my_macro_name!("The string to translate with group `my_group_name`.");
/// ```
/// 
#[proc_macro]
pub fn i18n_group(item: TokenStream) -> TokenStream {
    let content = syn::parse_macro_input!(item as i18n::mac::I18nGroupArgs);
    quote::quote!(#content).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn i18n_group_format(item: TokenStream) -> TokenStream {
    let content = syn::parse_macro_input!(item as i18n::mac::I18nGroupFormatArgs);
    quote::quote!(#content).into()
}
