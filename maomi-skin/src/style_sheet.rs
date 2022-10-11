use crate::{ParseError, css_token::*, StyleSheetVars, ScopeVars, ParseWithVars, write_css::*};

/// Handlers for CSS details (varies between backends)
pub trait StyleSheetConstructor {
    type PropertyValue: ParseStyleSheetValue;
    type FontFacePropertyValue: ParseStyleSheetValue;
    type MediaCondValue: ParseStyleSheetValue;

    fn new() -> Self
    where
        Self: Sized;

    fn set_config(&mut self, name: &CssIdent, tokens: &mut CssTokenStream) -> Result<(), ParseError>;

    fn define_key_frames(
        &mut self,
        name: &CssIdent,
        content: &Vec<(
            CssPercentage,
            CssBrace<Repeat<Property<Self::PropertyValue>>>,
        )>,
    ) -> CssIdent;

    fn to_tokens(&self, ss: &StyleSheet<Self>, tokens: &mut proc_macro2::TokenStream)
    where
        Self: Sized;
}

/// Parse value positions
pub trait ParseStyleSheetValue {
    fn parse_value(name: &CssIdent, tokens: &mut CssTokenStream) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub struct StyleSheet<T: StyleSheetConstructor> {
    ssc: T,
    pub items: Vec<StyleSheetItem<T>>,
    vars: StyleSheetVars,
}

pub enum StyleSheetItem<T: StyleSheetConstructor> {
    // MacroDefinition {
    //     at_keyword: CssAtKeyword,
    //     name: CssIdent,
    //     refs: Vec<CssIdent>,
    // },
    // ConstDefinition {
    //     at_keyword: CssAtKeyword,
    //     name: CssIdent,
    //     refs: Vec<CssIdent>,
    // },
    // KeyFramesDefinition {
    //     at_keyword: CssAtKeyword,
    //     dollar_token: token::Dollar,
    //     name: CssIdent,
    //     brace_token: token::Brace,
    //     content: Vec<(CssPercentage, CssBrace<Repeat<Property<T::PropertyValue>>>)>,
    //     def: CssIdent,
    // },
    Rule {
        dot_token: CssDelim,
        ident: CssIdent,
        content: CssBrace<RuleContent<T>>,
    },
}

impl<T: StyleSheetConstructor> ParseWithVars for StyleSheet<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let mut ssc = T::new();
        let mut items = vec![];
        let mut vars = StyleSheetVars {};
        while !input.is_ended() {
            if let Ok(dot_token) = input.expect_delim(".") {
                let item = StyleSheetItem::Rule {
                    dot_token,
                    ident: input.expect_ident()?,
                    content: ParseWithVars::parse_with_vars(input, &mut vars, scope)?,
                };
                items.push(item);
            } else if let Ok(at_keyword) = input.expect_at_keyword() {
                match at_keyword.formal_name.as_str() {
                    "config" => {
                        let name = input.expect_ident()?;
                        let mut tokens = input.sub_until_semi();
                        ssc.set_config(&name, &mut tokens);
                        tokens.expect_ended()?;
                        input.expect_semi()?;
                    }
                    "macro" => {
                        todo!() // TODO
                    }
                    "const" => {
                        todo!() // TODO
                    }
                    "keyframes" => {
                        todo!() // TODO
                    }
                    _ => {
                        return Err(ParseError::new(at_keyword.span, "unknown at-keyword"));
                    }
                }
            } else {
                return Err(ParseError::new(input.span(), "unexpected CSS token"));
            };
        }
        Ok(Self { ssc, items, vars })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        // TODO
        for item in self.items.iter() {
            match item {
                StyleSheetItem::Rule { content, .. } => {
                    content.block.for_each_ref(f);
                }
            }
        }
    }
}

pub struct RuleContent<T: StyleSheetConstructor> {
    pub props: Vec<Property<T::PropertyValue>>,
    // pub at_blocks: Vec<AtBlock<T>>,
    // pub pseudo_classes: Vec<PseudoClass<T>>,
    // pub sub_classes: Vec<SubClass<T>>,
    pub refs: Vec<CssIdent>,
}

/// A CSS property (name-value pair)
pub struct Property<V> {
    pub name: CssIdent,
    pub colon_token: CssColon,
    pub value: V,
    pub semi_token: CssSemi,
}

impl<V: WriteCss> WriteCss for Property<V> {
    fn write_css<W: std::fmt::Write>(&self, cssw: &mut CssWriter<W>) -> std::fmt::Result {
        self.name.write_css(cssw)?;
        self.colon_token.write_css(cssw)?;
        self.value.write_css(cssw)?;
        self.semi_token.write_css(cssw)?;
        Ok(())
    }
}

impl<T: StyleSheetConstructor> ParseWithVars for RuleContent<T> {
    fn parse_with_vars(
        input: &mut CssTokenStream,
        vars: &mut StyleSheetVars,
        scope: &mut ScopeVars,
    ) -> Result<Self, ParseError> {
        let mut props = vec![];
        let mut refs = vec![];
        while !input.is_ended() {
            let next = input.next()?;
            match next {
                CssToken::Ident(name) => {
                    let colon_token = input.expect_colon()?;
                    let mut value_tokens = input.sub_until_semi(); // TODO resolve vars
                    let value = T::PropertyValue::parse_value(&name, &mut value_tokens)?;
                    value_tokens.expect_ended()?;
                    let semi_token = input.expect_semi()?;
                    props.push(Property { name, colon_token, value, semi_token })
                }
                x => {
                    return Err(ParseError::new(x.span(), "unexpected token"));
                }
            }
        }
        Ok(Self {
            props,
            refs,
        })
    }

    fn for_each_ref(&self, f: &mut impl FnMut(&CssIdent)) {
        for r in &self.refs {
            f(r);
        }
    }
}

impl<T: StyleSheetConstructor> syn::parse::Parse for StyleSheet<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tokens = &mut CssTokenStream::parse(input)?;
        let ss = StyleSheet::parse_with_vars(tokens, &mut StyleSheetVars {}, &mut ScopeVars {})
            .map_err(|x| x.into_syn_error())?;
        tokens.expect_ended().map_err(|x| x.into_syn_error())?;
        Ok(ss)
    }
}

impl<T: StyleSheetConstructor> quote::ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ssc.to_tokens(self, tokens)
    }
}
