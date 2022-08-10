#![recursion_limit = "128"]

use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt, quote_spanned};
use syn::parse::Parse;

use maomi_skin::parser::{StyleSheet, StyleSheetItem};
use maomi_skin::parser::{StyleSheetConstructor, CssToken, Repeat};

struct DomCssProperty {
    inner: Repeat<CssToken>,
}

impl Parse for DomCssProperty {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: Repeat::parse_while(input, |x| !x.peek(syn::token::Semi))?,
        })
    }
}

struct DomStyleSheet {}

impl StyleSheetConstructor for DomStyleSheet {
    type PropertyValue = DomCssProperty;
    type FontFacePropertyValue = DomCssProperty;

    fn to_tokens(
        ss: &StyleSheet<Self>,
        tokens: &mut proc_macro2::TokenStream,
    ) where Self: Sized {
        for item in ss.items.iter() {
            match item {
                StyleSheetItem::Macro { .. } => unreachable!(),
                StyleSheetItem::Import { .. } => unreachable!(),
                StyleSheetItem::KeyFrames { at_keyword, name, brace_token, content } => {
                    // TODO
                }
                StyleSheetItem::FontFaceRule { at_keyword, items } => {
                    // TODO
                }
                StyleSheetItem::Rule { dot_token, ident, items } => {
                    let span = ident.span;
                    let str_name = &ident.name;
                    let struct_name = syn::Ident::new(&ident.name.replace('-', "_"), span);
                    tokens.append_all(quote_spanned! {span=>
                        #[allow(non_camel_case_types)]
                        struct #struct_name {}
                        impl maomi::prop::ListPropertyItem<maomi_dom::class_list::DomClassList, bool> for #struct_name {
                            type Value = &'static str;
                            #[inline(always)]
                            fn item_value(
                                _dest: &mut maomi_dom::class_list::DomClassList,
                                _index: usize,
                                _s: &bool,
                                _ctx: &mut <maomi_dom::class_list::DomClassList as maomi::prop::ListPropertyUpdate<bool>>::UpdateContext,
                            ) -> Self::Value {
                                #str_name
                            }
                        }
                    })
                    // TODO
                }
            }
        }
    }
}

#[proc_macro]
pub fn dom_css(item: TokenStream) -> TokenStream {
    let ss = syn::parse_macro_input!(item as StyleSheet<DomStyleSheet>);
    quote! {
        #ss
    }.into()
}
