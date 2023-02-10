use maomi_skin::ParseError;
use maomi_skin::write_css::*;
use maomi_skin::css_token::*;
use maomi_skin::style_sheet::*;

pub(crate) struct DomCssProperty {
    // TODO really parse the value
    inner: Vec<CssToken>,
}

impl ParseStyleSheetValue for DomCssProperty {
    fn parse_value(_: &CssIdent, tokens: &mut CssTokenStream) -> Result<Self, ParseError> {
        let mut v = vec![];
        while tokens.peek().is_ok() {
            v.push(tokens.next().unwrap())
        }
        Ok(Self {
            inner: v,
        })
    }
}

impl WriteCss for DomCssProperty {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::fmt::Result {
        for token in &self.inner {
            token.write_css(cssw)?;
        }
        Ok(())
    }
}
