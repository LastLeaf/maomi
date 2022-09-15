use maomi_skin::parser::{Repeat, CssToken, ParseStyleSheetValue, CssIdent, CssTokenStream};
use maomi_skin::parser::write_css::WriteCss;

pub(crate) struct DomCssProperty {
    // TODO really parse the value
    inner: Repeat<CssToken>,
}

impl ParseStyleSheetValue for DomCssProperty {
    fn parse_value(_: &CssIdent, tokens: &mut CssTokenStream) -> syn::Result<Self> {
        let mut v = vec![];
        while tokens.peek().is_ok() {
            v.push(tokens.next().unwrap())
        }
        Ok(Self {
            inner: Repeat::from_vec(v),
        })
    }
}

impl WriteCss for DomCssProperty {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut maomi_skin::parser::write_css::CssWriter<W>,
    ) -> std::fmt::Result {
        self.inner.write_css(cssw)
    }
}
