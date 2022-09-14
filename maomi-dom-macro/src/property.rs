use maomi_skin::parser::{Repeat, CssToken, ParseStyleSheetValue, CssIdent, CssTokenStream, WriteCss, WriteCssSepCond};

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
    fn write_css(
        &self,
        sc: WriteCssSepCond,
        debug_mode: bool,
        w: &mut impl std::fmt::Write,
    ) -> std::result::Result<WriteCssSepCond, std::fmt::Error> {
        self.inner.write_css(sc, debug_mode, w)
    }
}
