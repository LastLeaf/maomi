//! Parsing details of the stylesheets

use quote::ToTokens;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

mod css_token;
pub use css_token::*;
mod mac;
use mac::MacroDefinition;

/// Handlers for CSS details (varies between backends)
pub trait StyleSheetConstructor {
    type PropertyValue: Parse;
    type FontFacePropertyValue: Parse;

    fn to_tokens(
        ss: &StyleSheet<Self>,
        tokens: &mut proc_macro2::TokenStream,
    ) where Self: Sized;
}

/// A CSS property (name-value pair)
pub struct Property<V> {
    pub name: CssIdent,
    pub colon_token: token::Colon,
    pub value: V,
    pub semi_token: token::Semi,
}

impl<V: Parse> Parse for Property<V> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon_token: input.parse()?,
            value: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

pub enum PropertyOrSubRule<T: StyleSheetConstructor> {
    Property(Property<T::PropertyValue>),
    SubClass {
        and_token: token::And,
        ident: CssIdent,
        items: CssBrace<Repeat<PropertyOrSubRule<T>>>,
    },
    PseudoClass {
        colon_token: token::Colon,
        ident: CssIdent,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
    Media {
        at_keyword: CssAtKeyword,
        expr: Repeat<CssToken>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
    Supports {
        at_keyword: CssAtKeyword,
        expr: Repeat<CssToken>,
        items: CssBrace<Repeat<Property<T::PropertyValue>>>,
    },
}

impl<T: StyleSheetConstructor> Parse for PropertyOrSubRule<T>  {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let item = if la.peek(Ident) || la.peek(token::Sub) {
            Self::Property(input.parse()?)
        } else if la.peek(token::And) {
            Self::SubClass {
                and_token: input.parse()?,
                ident: input.parse()?,
                items: input.parse()?,
            }
        } else if la.peek(token::Colon) {
            Self::PseudoClass {
                colon_token: input.parse()?,
                ident: input.parse()?,
                items: input.parse()?,
            }
        } else if la.peek(token::At) {
            let at_keyword: CssAtKeyword = input.parse()?;
            match at_keyword.name.as_str() {
                "media" => Self::Media {
                    at_keyword,
                    expr: Repeat::parse_while(input, |input| !input.peek(token::Brace) && !input.peek(token::Semi))?,
                    items: input.parse()?,
                },
                "supports" => Self::Supports {
                    at_keyword,
                    expr: Repeat::parse_while(input, |input| !input.peek(token::Brace) && !input.peek(token::Semi))?,
                    items: input.parse()?,
                },
                _ => {
                    return Err(Error::new(at_keyword.span(), "Unknown at-keyword"));
                }
            }
        } else {
            return Err(la.error());
        };
        Ok(item)
    }
}

pub enum StyleSheetItem<T: StyleSheetConstructor> {
    Macro {
        at_keyword: CssAtKeyword,
        export: bool,
        name: CssIdent,
        mac: MacroDefinition,
    },
    Import {
        at_keyword: CssAtKeyword,
        src: CssString,
        semi_token: token::Semi,
    },
    KeyFrames {
        at_keyword: CssAtKeyword,
        name: CssIdent,
        brace_token: token::Brace,
        content: Repeat<(CssPercentage, CssBrace<Repeat<Property<T::PropertyValue>>>)>,
    },
    FontFaceRule {
        at_keyword: CssAtKeyword,
        items: CssBrace<Repeat<Property<T::FontFacePropertyValue>>>,
    },
    Rule {
        dot_token: token::Dot,
        ident: CssIdent,
        items: CssBrace<Repeat<PropertyOrSubRule<T>>>,
    },
}

impl<T: StyleSheetConstructor> Parse for StyleSheetItem<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        let item = if la.peek(token::At) {
            let at_keyword: CssAtKeyword = input.parse()?;
            match at_keyword.name.as_str() {
                "macro" => StyleSheetItem::Macro {
                    at_keyword,
                    export: false,
                    name: input.parse()?,
                    mac: input.parse()?,
                },
                "macro-export" => StyleSheetItem::Macro {
                    at_keyword,
                    export: true,
                    name: input.parse()?,
                    mac: input.parse()?,
                },
                "import" => StyleSheetItem::Import {
                    at_keyword,
                    src: input.parse()?,
                    semi_token: input.parse()?,
                },
                "key-frames" => {
                    let name = input.parse()?;
                    let content;
                    let brace_token = braced!(content in input);
                    let input = content;
                    let mut content = vec![];
                    while !content.is_empty() {
                        let la = input.lookahead1();
                        let percentage = if la.peek(Ident) {
                            let s: CssIdent = input.parse()?;
                            match s.name.as_str() {
                                "from" => CssPercentage {
                                    span: s.span(),
                                    num: Number::Int(0),
                                },
                                "to" => CssPercentage {
                                    span: s.span(),
                                    num: Number::Int(100),
                                },
                                _ => return Err(Error::new(s.span(), "Illegal ident"))
                            }
                        } else if la.peek(Lit) {
                            input.parse()?
                        } else {
                            return Err(la.error());
                        };
                        let props = input.parse()?;
                        content.push((percentage, props));
                    }
                    StyleSheetItem::KeyFrames {
                        at_keyword,
                        name,
                        brace_token,
                        content: content.into(),
                    }
                }
                "font-face" => StyleSheetItem::FontFaceRule {
                    at_keyword,
                    items: input.parse()?,
                },
                _ => {
                    return Err(Error::new(at_keyword.span(), "Unknown at-keyword"));
                }
            }
        } else if la.peek(token::Dot) {
            let dot_token = input.parse()?;
            let ident = input.parse()?;
            let items = input.parse()?;
            StyleSheetItem::Rule {
                dot_token,
                ident,
                items,
            }
        } else {
            return Err(la.error());
        };
        Ok(item)
    }
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    pub items: Repeat<StyleSheetItem<T>>,
}

impl<T: StyleSheetConstructor> Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            items: input.parse()?,
        })
    }
}

impl<T: StyleSheetConstructor> ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        T::to_tokens(self, tokens)
    }
}
