use proc_macro::TokenStream;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

use crate::StyleSheetConstructor;
mod css_token;
pub(crate) use css_token::*;
mod mac;
use mac::MacroDefinition;

pub(crate) struct Property<V> {
    name: CssIdent,
    value: V,
}

pub(crate) enum PropertyOrSubRule<T: StyleSheetConstructor> {
    Property(Property<T::PropertyValue>),
    SubClass {
        and_token: token::And,
        ident: CssIdent,
        items: CssBrace<Vec<PropertyOrSubRule<T>>>,
    },
    PseudoClass {
        colon_token: token::Colon,
        ident: CssIdent,
        items: CssBrace<Vec<Property<T::PropertyValue>>>,
    },
    Media {
        at_keyword: CssAtKeyword,
        expr: Vec<CssToken>,
        items: CssBrace<Vec<Property<T::PropertyValue>>>,
    },
    Supports {
        at_keyword: CssAtKeyword,
        expr: Vec<CssToken>,
        items: CssBrace<Vec<Property<T::PropertyValue>>>,
    },
}

pub(crate) struct StyleSheet<T: StyleSheetConstructor> {
    items: Vec<StyleSheetItem<T>>,
}

pub(crate) enum StyleSheetItem<T: StyleSheetConstructor> {
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
        content: Vec<(CssPercentage, CssBrace<Vec<Property<T::PropertyValue>>>)>,
    },
    FontFaceRule {
        at_keyword: CssAtKeyword,
        items: CssBrace<Vec<Property<T::FontFacePropertyValue>>>,
    },
    Rule {
        and_token: token::Dot,
        ident: CssIdent,
        items: CssBrace<Vec<PropertyOrSubRule<T>>>,
    },
}

impl<T: StyleSheetConstructor> Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = vec![];
        while input.is_empty() {
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
                        let mut content;
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
                            content,
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
                StyleSheetItem::Rule {
                    and_token: input.parse()?,
                    ident: input.parse()?,
                    items: input.parse()?,
                }
            } else {
                return Err(la.error());
            };
            items.push(item);
        }
        Ok(StyleSheet { items })
    }
}

pub fn parse<T: StyleSheetConstructor>(item: TokenStream) -> TokenStream {
    let ss = parse_macro_input!(item as StyleSheet<T>);
    quote! {
        #ss
    }.into()
}
