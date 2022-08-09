use proc_macro2::Span;
use quote::*;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

pub(crate) enum Number {
    Int(i64),
    Float(f64),
}

pub(crate) struct CssIdent {
    pub(crate) span: Span,
    pub(crate) name: String,
}

impl Spanned for CssIdent {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = String::new();
        let mut span = None;
        loop {
            let la = input.lookahead1();
            if la.peek(token::Sub) {
                let t: token::Sub = input.parse()?;
                if span.is_none() { span = Some(t.span()) }
                name.push('-');
            } else if la.peek(Ident) {
                let t: Ident = input.parse()?;
                if span.is_none() { span = Some(t.span()) }
                let s: &str = &input.to_string();
                s.strip_prefix("r#").unwrap_or(s);
                name += s;
            } else {
                Err(la.error())?;
            }
            if !input.peek(token::Sub) {
                break;
            }
        }
        Ok(Self {
            name,
            span: span.unwrap(),
        })
    }
}

pub(crate) struct CssAtKeyword {
    pub(crate) span: Span,
    pub(crate) at_token: token::At,
    pub(crate) name: String,
}

impl Spanned for CssAtKeyword {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssAtKeyword {
    fn parse(input: ParseStream) -> Result<Self> {
        let at_token: token::At = input.parse()?;
        let name = CssIdent::parse(input)?.name;
        Ok(Self {
            span: at_token.span(),
            at_token,
            name,
        })
    }
}

pub(crate) struct CssString {
    pub(crate) s: LitStr,
}

impl Spanned for CssString {
    fn span(&self) -> Span {
        self.s.span()
    }
}

impl Parse for CssString {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            s: input.parse()?,
        })
    }
}

pub(crate) struct CssColon {
    pub(crate) span: Span,
}

impl Spanned for CssColon {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssColon {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Colon = input.parse()?;
        Ok(Self {
            span: x.span(),
        })
    }
}

pub(crate) struct CssSemi {
    pub(crate) span: Span,
}

impl Spanned for CssSemi {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssSemi {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Semi = input.parse()?;
        Ok(Self {
            span: x.span(),
        })
    }
}

pub(crate) struct CssDelim {
    pub(crate) span: Span,
    pub(crate) s: &'static str,
}

impl Spanned for CssDelim {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssDelim {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        macro_rules! parse_delim {
            ($x:tt) => {
                if la.peek(Token![$x]) {
                    let x: Token![$x] = input.parse()?;
                    let span = x.span();
                    return Ok(Self {
                        span,
                        s: stringify!($x),
                    })
                }
            }
        }
        parse_delim!(+);
        parse_delim!(+=);
        parse_delim!(&);
        parse_delim!(&&);
        parse_delim!(&=);
        parse_delim!(@);
        parse_delim!(!);
        parse_delim!(^);
        parse_delim!(^=);
        parse_delim!(/);
        parse_delim!(/=);
        parse_delim!($);
        parse_delim!(.);
        parse_delim!(..);
        parse_delim!(...);
        parse_delim!(..=);
        parse_delim!(=);
        parse_delim!(==);
        parse_delim!(=>);
        parse_delim!(>=);
        parse_delim!(>);
        parse_delim!(<-);
        parse_delim!(<=);
        parse_delim!(<);
        parse_delim!(*=);
        parse_delim!(!=);
        parse_delim!(|);
        parse_delim!(|=);
        parse_delim!(||);
        parse_delim!(#);
        parse_delim!(?);
        parse_delim!(->);
        parse_delim!(%);
        parse_delim!(%=);
        parse_delim!(<<);
        parse_delim!(<<=);
        parse_delim!(>>);
        parse_delim!(>>=);
        parse_delim!(*);
        parse_delim!(-);
        parse_delim!(-=);
        parse_delim!(~);
        Err(la.error())
    }
}

pub(crate) struct CssNumber {
    pub(crate) span: Span,
    pub(crate) num: Number,
}

impl Spanned for CssNumber {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssNumber {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        if la.peek(LitInt) {
            let n: LitInt = input.parse()?;
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
            })
        }
        if la.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
            })
        }
        Err(la.error())
    }
}

pub(crate) struct CssPercentage {
    pub(crate) span: Span,
    pub(crate) num: Number,
}

impl Spanned for CssPercentage {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssPercentage {
    fn parse(input: ParseStream) -> Result<Self> {
        let CssNumber { span, num } = CssNumber::parse(input)?;
        let _: Token![%] = input.parse()?;
        Ok(Self {
            span,
            num,
        })
    }
}

pub(crate) struct CssDimension {
    pub(crate) span: Span,
    pub(crate) num: Number,
    pub(crate) unit: String,
}

impl Spanned for CssDimension {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssDimension {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit: Lit = input.parse()?;
        todo!("Unrecognized CssDimension {:?}", lit.to_token_stream());
    }
}

pub(crate) struct CssFunction<T> {
    pub(crate) span: Span,
    pub(crate) name: String,
    pub(crate) paren_token: token::Paren,
    pub(crate) block: T,
}

impl<T> Spanned for CssFunction<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: Parse> Parse for CssFunction<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let CssIdent { span, name } = CssIdent::parse(input)?;
        let content;
        let paren_token = parenthesized!(content in input);
        let block = content.parse()?;
        Ok(Self {
            span,
            name,
            paren_token,
            block,
        })
    }
}

pub(crate) struct CssParen<T> {
    pub(crate) paren_token: token::Paren,
    pub(crate) block: T,
}

impl<T> Spanned for CssParen<T> {
    fn span(&self) -> Span {
        self.paren_token.span
    }
}

impl<T: Parse> Parse for CssParen<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let block = content.parse()?;
        Ok(Self {
            paren_token,
            block,
        })
    }
}

pub(crate) struct CssBracket<T> {
    pub(crate) bracket_token: token::Bracket,
    pub(crate) block: T,
}

impl<T> Spanned for CssBracket<T> {
    fn span(&self) -> Span {
        self.bracket_token.span
    }
}

impl<T: Parse> Parse for CssBracket<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let bracket_token = bracketed!(content in input);
        let block = content.parse()?;
        Ok(Self {
            bracket_token,
            block,
        })
    }
}

pub(crate) struct CssBrace<T> {
    pub(crate) brace_token: token::Brace,
    pub(crate) block: T,
}

impl<T> Spanned for CssBrace<T> {
    fn span(&self) -> Span {
        self.brace_token.span
    }
}

impl<T: Parse> Parse for CssBrace<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);
        let block = content.parse()?;
        Ok(Self {
            brace_token,
            block,
        })
    }
}

pub(crate) enum CssToken {
    Ident(CssIdent),
    AtKeyword(CssAtKeyword),
    String(CssString),
    Delim(CssDelim),
    Colon(CssColon),
    Semi(CssSemi),
    Function(CssFunction<Vec<CssToken>>),
    Paren(CssParen<Vec<CssToken>>),
    Bracket(CssBracket<Vec<CssToken>>),
    Brace(CssBrace<Vec<CssToken>>),
}
