use proc_macro2::Span;
use syn::parse::*;
use syn::spanned::Spanned;
use syn::*;

use super::{WriteCss, WriteCssSepCond};

pub enum Number {
    Int(i64),
    Float(f64),
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(x) => write!(f, "{}", x),
            Self::Float(x) => write!(f, "{}", x),
        }
    }
}

pub struct CssIdent {
    pub span: Span,
    pub name: String,
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
            let is_sub = if la.peek(token::Sub) {
                let t: token::Sub = input.parse()?;
                if span.is_none() {
                    span = Some(t.span())
                }
                name.push('-');
                true
            } else if la.peek(Ident) {
                let t: Ident = input.parse()?;
                if span.is_none() {
                    span = Some(t.span())
                }
                let s: &str = &t.to_string();
                name += s.strip_prefix("r#").unwrap_or(s);
                false
            } else {
                loop {
                    macro_rules! parse_keyword {
                        ($x:tt) => {
                            if la.peek(Token![$x]) {
                                let t: Token![$x] = input.parse()?;
                                if span.is_none() {
                                    span = Some(t.span())
                                }
                                name += stringify!($x);
                                break false;
                            }
                        };
                    }
                    parse_keyword!(abstract);
                    parse_keyword!(as);
                    parse_keyword!(async);
                    parse_keyword!(auto);
                    parse_keyword!(await);
                    parse_keyword!(become);
                    parse_keyword!(box);
                    parse_keyword!(break);
                    parse_keyword!(const);
                    parse_keyword!(continue);
                    parse_keyword!(crate);
                    parse_keyword!(default);
                    parse_keyword!(do);
                    parse_keyword!(dyn);
                    parse_keyword!(else);
                    parse_keyword!(enum);
                    parse_keyword!(extern);
                    parse_keyword!(final);
                    parse_keyword!(fn);
                    parse_keyword!(for);
                    parse_keyword!(if);
                    parse_keyword!(impl);
                    parse_keyword!(in);
                    parse_keyword!(let);
                    parse_keyword!(loop);
                    parse_keyword!(macro);
                    parse_keyword!(match);
                    parse_keyword!(mod);
                    parse_keyword!(move);
                    parse_keyword!(mut);
                    parse_keyword!(override);
                    parse_keyword!(priv);
                    parse_keyword!(pub);
                    parse_keyword!(ref);
                    parse_keyword!(return);
                    parse_keyword!(Self);
                    parse_keyword!(self);
                    parse_keyword!(static);
                    parse_keyword!(struct);
                    parse_keyword!(super);
                    parse_keyword!(trait);
                    parse_keyword!(try);
                    parse_keyword!(type);
                    parse_keyword!(typeof);
                    parse_keyword!(union);
                    parse_keyword!(unsafe);
                    parse_keyword!(unsized);
                    parse_keyword!(use);
                    parse_keyword!(virtual);
                    parse_keyword!(where);
                    parse_keyword!(while);
                    parse_keyword!(yield);
                    return Err(la.error());
                }
            };
            if is_sub || input.peek(token::Sub) {
                // empty
            } else {
                break;
            }
        }
        Ok(Self {
            name,
            span: span.unwrap(),
        })
    }
}

impl WriteCss for CssIdent {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        } else {
            match sc {
                WriteCssSepCond::Ident
                | WriteCssSepCond::NonIdentAlpha
                | WriteCssSepCond::Digit
                | WriteCssSepCond::At => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "{}", self.name)?;
        Ok(WriteCssSepCond::Ident)
    }
}

pub struct CssAtKeyword {
    pub span: Span,
    pub at_token: token::At,
    pub name: String,
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

impl WriteCss for CssAtKeyword {
    fn write_css(
        &self,
        _sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, "@{}", self.name)?;
        Ok(WriteCssSepCond::NonIdentAlpha)
    }
}

pub struct CssString {
    pub s: LitStr,
}

impl Spanned for CssString {
    fn span(&self) -> Span {
        self.s.span()
    }
}

impl Parse for CssString {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self { s: input.parse()? })
    }
}

impl WriteCss for CssString {
    fn write_css(
        &self,
        _sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, "{:?}", self.s.value())?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssColon {
    pub span: Span,
}

impl Spanned for CssColon {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssColon {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Colon = input.parse()?;
        Ok(Self { span: x.span() })
    }
}

impl WriteCss for CssColon {
    fn write_css(
        &self,
        _sc: super::WriteCssSepCond,
        _debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        write!(w, ":")?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssSemi {
    pub span: Span,
}

impl Spanned for CssSemi {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssSemi {
    fn parse(input: ParseStream) -> Result<Self> {
        let x: token::Semi = input.parse()?;
        Ok(Self { span: x.span() })
    }
}

impl WriteCss for CssSemi {
    fn write_css(
        &self,
        _sc: super::WriteCssSepCond,
        _debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        write!(w, ";")?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssDelim {
    pub span: Span,
    pub s: &'static str,
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
                    });
                }
            };
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

impl WriteCss for CssDelim {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        let need_sep = debug_mode
            || match sc {
                WriteCssSepCond::Ident => self.s == "-",
                WriteCssSepCond::NonIdentAlpha => self.s == "-",
                WriteCssSepCond::Digit => self.s == "." || self.s == "-" || self.s == "%",
                WriteCssSepCond::At => self.s == "-",
                WriteCssSepCond::Equalable => self.s == "=",
                WriteCssSepCond::Bar => self.s == "=" || self.s == "|" || self.s == "|=",
                WriteCssSepCond::Slash => self.s == "*" || self.s == "*=",
                _ => false,
            };
        if need_sep {
            write!(w, " ")?;
        }
        write!(w, "{}", self.s)?;
        Ok(match self.s {
            "@" => WriteCssSepCond::At,
            "." | "+" => WriteCssSepCond::DotOrPlus,
            "$" | "^" | "~" | "*" => WriteCssSepCond::Equalable,
            "|" => WriteCssSepCond::Bar,
            "/" => WriteCssSepCond::Slash,
            _ => WriteCssSepCond::Other,
        })
    }
}

pub struct CssNumber {
    pub span: Span,
    pub num: Number,
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
            if n.suffix().len() > 0 {
                return Err(Error::new(n.span(), "Illegal number suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
            });
        }
        if la.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            if n.suffix().len() > 0 {
                return Err(Error::new(n.span(), "Illegal number suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
            });
        }
        Err(la.error())
    }
}

impl WriteCss for CssNumber {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        } else {
            match sc {
                WriteCssSepCond::Ident
                | WriteCssSepCond::NonIdentAlpha
                | WriteCssSepCond::Digit
                | WriteCssSepCond::DotOrPlus => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "{}", self.num)?;
        Ok(WriteCssSepCond::Digit)
    }
}

pub struct CssPercentage {
    pub span: Span,
    pub num: Number,
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
        Ok(Self { span, num })
    }
}

impl WriteCss for CssPercentage {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        } else {
            match sc {
                WriteCssSepCond::Ident
                | WriteCssSepCond::NonIdentAlpha
                | WriteCssSepCond::Digit
                | WriteCssSepCond::DotOrPlus => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "{}%", self.num)?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssDimension {
    pub span: Span,
    pub num: Number,
    pub unit: String,
}

impl Spanned for CssDimension {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for CssDimension {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        if la.peek(LitInt) {
            let n: LitInt = input.parse()?;
            if n.suffix().len() == 0 {
                return Err(Error::new(n.span(), "Expect dimension suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
                unit: n.suffix().to_string(),
            });
        }
        if la.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            if n.suffix().len() == 0 {
                return Err(Error::new(n.span(), "Expect dimension suffix"));
            }
            return Ok(Self {
                span: n.span(),
                num: Number::Int(n.base10_parse()?),
                unit: n.suffix().to_string(),
            });
        }
        Err(la.error())
    }
}

impl WriteCss for CssDimension {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        } else {
            match sc {
                WriteCssSepCond::Ident
                | WriteCssSepCond::NonIdentAlpha
                | WriteCssSepCond::Digit
                | WriteCssSepCond::DotOrPlus => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "{}{}", self.num, self.unit)?;
        Ok(WriteCssSepCond::NonIdentAlpha)
    }
}

pub struct CssFunction<T> {
    pub span: Span,
    pub name: String,
    pub paren_token: token::Paren,
    pub block: T,
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

impl<T: WriteCss> WriteCss for CssFunction<T> {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        } else {
            match sc {
                WriteCssSepCond::Ident
                | WriteCssSepCond::NonIdentAlpha
                | WriteCssSepCond::Digit
                | WriteCssSepCond::At => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "{}(", self.name)?;
        self.block
            .write_css(WriteCssSepCond::Other, debug_mode, w)?;
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, ")")?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssParen<T> {
    pub paren_token: token::Paren,
    pub block: T,
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
        Ok(Self { paren_token, block })
    }
}

impl<T: WriteCss> WriteCss for CssParen<T> {
    fn write_css(
        &self,
        sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        } else {
            match sc {
                WriteCssSepCond::Ident => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "(")?;
        self.block
            .write_css(WriteCssSepCond::Other, debug_mode, w)?;
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, ")")?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssBracket<T> {
    pub bracket_token: token::Bracket,
    pub block: T,
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

impl<T: WriteCss> WriteCss for CssBracket<T> {
    fn write_css(
        &self,
        _sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, "[")?;
        self.block
            .write_css(WriteCssSepCond::Other, debug_mode, w)?;
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, "]")?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct CssBrace<T> {
    pub brace_token: token::Brace,
    pub block: T,
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
        Ok(Self { brace_token, block })
    }
}

impl<T: WriteCss> WriteCss for CssBrace<T> {
    fn write_css(
        &self,
        _sc: super::WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<super::WriteCssSepCond, std::fmt::Error> {
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, "{{")?;
        self.block
            .write_css(WriteCssSepCond::Other, debug_mode, w)?;
        if debug_mode {
            write!(w, " ")?;
        }
        write!(w, "}}")?;
        Ok(WriteCssSepCond::Other)
    }
}

pub struct Repeat<T> {
    inner: Vec<T>,
}

impl<T: Parse> Repeat<T> {
    pub fn parse_while(input: ParseStream, mut f: impl FnMut(ParseStream) -> bool) -> Result<Self> {
        let mut inner = vec![];
        while f(input) {
            let x = input.parse()?;
            inner.push(x);
        }
        Ok(Self { inner })
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.inner.iter()
    }
}

impl<T: Parse> Parse for Repeat<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_while(input, |input| !input.is_empty())
    }
}

impl<T: WriteCss> WriteCss for Repeat<T> {
    fn write_css(
        &self,
        mut sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error> {
        for item in self.inner.iter() {
            sc = item.write_css(sc, debug_mode, w)?;
        }
        Ok(sc)
    }
}

impl<T> From<Vec<T>> for Repeat<T> {
    fn from(inner: Vec<T>) -> Self {
        Self { inner }
    }
}

impl<'a, T> IntoIterator for &'a Repeat<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

pub enum CssToken {
    Ident(CssIdent),
    AtKeyword(CssAtKeyword),
    String(CssString),
    Colon(CssColon),
    Semi(CssSemi),
    Delim(CssDelim),
    Number(CssNumber),
    Percentage(CssPercentage),
    Dimension(CssDimension),
    Function(CssFunction<Repeat<CssToken>>),
    Paren(CssParen<Repeat<CssToken>>),
    Bracket(CssBracket<Repeat<CssToken>>),
    Brace(CssBrace<Repeat<CssToken>>),
}

impl Parse for CssToken {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = if input.peek(token::At) {
            Self::AtKeyword(input.parse()?)
        } else if input.peek(LitStr) {
            Self::String(input.parse()?)
        } else if input.peek(token::Colon) {
            Self::Colon(input.parse()?)
        } else if input.peek(token::Semi) {
            Self::Semi(input.parse()?)
        } else if input.peek(token::Paren) {
            Self::Paren(input.parse()?)
        } else if input.peek(token::Bracket) {
            Self::Bracket(input.parse()?)
        } else if input.peek(token::Brace) {
            Self::Brace(input.parse()?)
        } else if input.peek(LitInt) {
            let n: LitInt = input.parse()?;
            let span = n.span();
            let num = Number::Int(n.base10_parse()?);
            if n.suffix().len() == 0 {
                if input.peek(Token![%]) {
                    let _: Token![%] = input.parse()?;
                    Self::Percentage(CssPercentage { span, num })
                } else {
                    Self::Number(CssNumber { span, num })
                }
            } else {
                Self::Dimension(CssDimension {
                    span,
                    num,
                    unit: n.suffix().to_string(),
                })
            }
        } else if input.peek(LitFloat) {
            let n: LitFloat = input.parse()?;
            let span = n.span();
            let num = Number::Float(n.base10_parse()?);
            if n.suffix().len() == 0 {
                if input.peek(Token![%]) {
                    let _: Token![%] = input.parse()?;
                    Self::Percentage(CssPercentage { span, num })
                } else {
                    Self::Number(CssNumber { span, num })
                }
            } else {
                Self::Dimension(CssDimension {
                    span,
                    num,
                    unit: n.suffix().to_string(),
                })
            }
        } else if let Ok(x) = input.parse::<CssIdent>() {
            if input.peek(token::Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let block = content.parse()?;
                Self::Function(CssFunction {
                    span: x.span,
                    name: x.name,
                    paren_token,
                    block,
                })
            } else {
                Self::Ident(x)
            }
        } else if let Ok(x) = input.parse() {
            Self::Delim(x)
        } else {
            return Err(input.error("Illegal CSS token"));
        };
        Ok(item)
    }
}

impl WriteCss for CssToken {
    fn write_css(
        &self,
        sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error> {
        let sc = match self {
            Self::Ident(x) => x.write_css(sc, debug_mode, w)?,
            Self::AtKeyword(x) => x.write_css(sc, debug_mode, w)?,
            Self::String(x) => x.write_css(sc, debug_mode, w)?,
            Self::Colon(x) => x.write_css(sc, debug_mode, w)?,
            Self::Semi(x) => x.write_css(sc, debug_mode, w)?,
            Self::Delim(x) => x.write_css(sc, debug_mode, w)?,
            Self::Number(x) => x.write_css(sc, debug_mode, w)?,
            Self::Percentage(x) => x.write_css(sc, debug_mode, w)?,
            Self::Dimension(x) => x.write_css(sc, debug_mode, w)?,
            Self::Function(x) => x.write_css(sc, debug_mode, w)?,
            Self::Paren(x) => x.write_css(sc, debug_mode, w)?,
            Self::Bracket(x) => x.write_css(sc, debug_mode, w)?,
            Self::Brace(x) => x.write_css(sc, debug_mode, w)?,
        };
        Ok(sc)
    }
}
