extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::parse::*;
use syn::*;

mod skin;

#[derive(Clone)]
struct StyleSheetDefinition {
    visibility: Option<Visibility>,
    namespace: Option<Ident>,
    style: LitStr,
}
impl Parse for StyleSheetDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let visibility = if lookahead.peek(Token![pub]) {
            Some(input.parse()?)
        } else {
            None
        };
        let lookahead = input.lookahead1();
        let namespace = if lookahead.peek(Ident) {
            let namespace = Some(input.parse()?);
            input.parse::<Token![=]>()?;
            namespace
        } else {
            None
        };
        let original_style: LitStr = input.parse()?;
        let original_style_str = original_style.value();
        let namespace_str = namespace.as_ref().map(|x: &Ident| x.to_string() + "-");
        let style = skin::compile(
            namespace_str.as_ref().map(|x| x.as_str()),
            original_style_str.as_str(),
        );
        match style {
            Ok(style) => Ok(Self {
                visibility,
                namespace,
                style: LitStr::new(&style, proc_macro2::Span::call_site()),
            }),
            Err(e) => {
                let msg = format!("Parse style sheet failed: {:?}", e);
                Err(syn::Error::new(proc_macro2::Span::call_site(), msg))
            }
        }
    }
}
impl ToTokens for StyleSheetDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            visibility,
            namespace,
            style,
        } = self;
        match namespace {
            Some(namespace) => {
                tokens.append_all(quote! { #visibility const #namespace: &'static str = #style; });
            }
            None => {
                tokens.append_all(quote! { #style });
            }
        }
    }
}
#[proc_macro]
pub fn skin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StyleSheetDefinition);
    let ret = quote! {
        #input
    };
    TokenStream::from(ret)
}
