use syn::{Error, spanned::Spanned};

use maomi_skin::parser::{*, write_css::WriteCss};

pub(crate) enum DomMediaCondValue {
    AspectRatio(CssNumber, CssNumber),
    Orientation(DomMediaOrientation),
    PrefersColorScheme(DomMediaColorScheme),
    Resolution(CssDimension),
    MinResolution(CssDimension),
    MaxResolution(CssDimension),
    Width(CssDimension),
    MinWidth(CssDimension),
    MaxWidth(CssDimension),
    Height(CssDimension),
    MinHeight(CssDimension),
    MaxHeight(CssDimension),
}

pub(crate) enum DomMediaOrientation {
    Landscape,
    Portrait,
}

pub(crate) enum DomMediaColorScheme {
    Light,
    Dark,
}

fn parse_aspect_ratio(tokens: &mut CssTokenStream) -> syn::Result<(CssNumber, CssNumber)> {
    let span = tokens.span();
    let a = tokens.expect_number()?;
    if a.positive_integer().is_none() {
        return Err(Error::new(span, "Expected positive integer"));
    }
    let _ = tokens.expect_delim("/")?;
    let span = tokens.span();
    let b = tokens.expect_number()?;
    if b.positive_integer().is_none() {
        return Err(Error::new(span, "Expected positive integer"));
    }
    Ok((a, b))
}

impl ParseStyleSheetValue for DomMediaCondValue {
    fn parse_value(
        name: &CssIdent,
        tokens: &mut CssTokenStream,
    ) -> syn::Result<Self> where Self: Sized {
        let ret = match name.formal_name.as_str() {
            "aspect_ratio" | "min_aspect_ratio" | "max_aspect_ratio" => {
                let (a, b) = parse_aspect_ratio(tokens)?;
                Self::AspectRatio(a, b)
            }
            // TODO
            _ => {
                return Err(Error::new(name.span(), "Unknown media feature"));
            }
        };
        Ok(ret)
    }
}

impl WriteCss for DomMediaCondValue {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut maomi_skin::parser::write_css::CssWriter<W>,
    ) -> std::fmt::Result {
        match self {
            Self::AspectRatio(a, b) => {
                a.write_css(cssw)?;
                cssw.write_delim("/", true)?;
                b.write_css(cssw)?;
            }
            _ => {
                todo!() // TODO
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test::{setup_env, parse_str};
    use crate::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn aspect_ratio() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(r#"
                .c {
                    @media (aspect-ratio: 16/0) {}
                }
            "#).is_err());
            parse_str(r#"
                @config name_mangling: off;
                .c {
                    @media (aspect-ratio: 16/9), (min-aspect-ratio: 4/3), (max-aspect-ratio: 2/1) {
                        padding: 1px;
                    }
                }
            "#);
            assert_eq!(
                env.read_output(),
                r#"@media(aspect-ratio:16/9),(min-aspect-ratio:4/3),(max-aspect-ratio:2/1){.c{padding:1px}}"#,
            );
        });
    }
}
