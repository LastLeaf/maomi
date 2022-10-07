#![recursion_limit = "128"]

// pub mod parser;
pub use cssparser;
pub mod style_sheet;

pub fn parse<T: StyleSheetConstructor>(s: &str) -> style_sheet::StyleSheet<T> {
    let input = &mut cssparser::ParserInput::new(s);
    let parser = &mut cssparser::Parser::new(input);
    StyleSheet::parse_css(parser)
}

type Error<'i> = cssparser::ParseError<'i, ParseCssError>;

pub enum ParseCssError {
}

pub trait ParseCss {
    fn parse_css<'i>(parser: &mut cssparser::Parser<'i>) -> Result<Self, Error<'i>> where Self: Sized;
}
