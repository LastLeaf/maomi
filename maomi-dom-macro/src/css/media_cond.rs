use maomi_skin::{write_css::*, css_token::*, style_sheet::ParseStyleSheetValue, ParseError};

pub(crate) enum DomMediaCondValue {
    AspectRatio(CssNumber, CssNumber),
    Orientation(DomMediaOrientation),
    PrefersColorScheme(DomMediaColorScheme),
    Resolution(CssDimension),
    Width(CssDimension),
    Height(CssDimension),
}

pub(crate) enum DomMediaOrientation {
    Landscape,
    Portrait,
}

pub(crate) enum DomMediaColorScheme {
    Light,
    Dark,
}

fn parse_aspect_ratio(tokens: &mut CssTokenStream) -> Result<(CssNumber, CssNumber), ParseError> {
    let span = tokens.span();
    let a = tokens.expect_number()?;
    if a.positive_integer().is_none() {
        return Err(ParseError::new(span, "Expected positive integer"));
    }
    let _ = tokens.expect_delim("/")?;
    let span = tokens.span();
    let b = tokens.expect_number()?;
    if b.positive_integer().is_none() {
        return Err(ParseError::new(span, "Expected positive integer"));
    }
    Ok((a, b))
}

impl ParseStyleSheetValue for DomMediaCondValue {
    fn parse_value(name: &CssIdent, tokens: &mut CssTokenStream) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let ret = match name.formal_name.as_str() {
            "aspect_ratio" | "min_aspect_ratio" | "max_aspect_ratio" => {
                let (a, b) = parse_aspect_ratio(tokens)?;
                Self::AspectRatio(a, b)
            }
            "orientation" => Self::Orientation(match tokens.expect_ident()?.formal_name.as_str() {
                "landscape" => DomMediaOrientation::Landscape,
                "portrait" => DomMediaOrientation::Portrait,
                _ => Err(ParseError::new(
                    name.span,
                    "Expected `landscape` or `portrait`",
                ))?,
            }),
            "prefers_color_scheme" => {
                Self::PrefersColorScheme(match tokens.expect_ident()?.formal_name.as_str() {
                    "light" => DomMediaColorScheme::Light,
                    "dark" => DomMediaColorScheme::Dark,
                    _ => Err(ParseError::new(name.span, "Expected `light` or `dark`"))?,
                })
            }
            "resolution" | "min_resolution" | "max_resolution" => {
                let x = tokens.expect_dimension()?;
                if x.unit.as_str() != "dpi" {
                    return Err(ParseError::new(name.span, "Expected `dpi` unit"));
                }
                Self::Resolution(x)
            }
            "width" | "min_width" | "max_width" => {
                let x = tokens.expect_dimension()?;
                if x.unit.as_str() != "px" {
                    return Err(ParseError::new(name.span, "Expected `px` unit"));
                }
                Self::Width(x)
            }
            "height" | "min_height" | "max_height" => {
                let x = tokens.expect_dimension()?;
                if x.unit.as_str() != "px" {
                    return Err(ParseError::new(name.span, "Expected `px` unit"));
                }
                Self::Height(x)
            }
            _ => {
                return Err(ParseError::new(name.span, "Unknown media feature"));
            }
        };
        Ok(ret)
    }
}

impl WriteCss for DomMediaCondValue {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::fmt::Result {
        match self {
            Self::AspectRatio(a, b) => {
                a.write_css(cssw)?;
                cssw.write_delim("/", true)?;
                b.write_css(cssw)?;
            }
            Self::Orientation(x) => {
                let s = match x {
                    DomMediaOrientation::Landscape => "landscape",
                    DomMediaOrientation::Portrait => "portrait",
                };
                cssw.write_ident(s, true)?;
            }
            Self::PrefersColorScheme(x) => {
                let s = match x {
                    DomMediaColorScheme::Light => "light",
                    DomMediaColorScheme::Dark => "dark",
                };
                cssw.write_ident(s, true)?;
            }
            Self::Resolution(x) => x.write_css(cssw)?,
            Self::Width(x) => x.write_css(cssw)?,
            Self::Height(x) => x.write_css(cssw)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{parse_str, setup_env};
    use super::super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn only() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media only (resolution: 1.dpi) {}
                    }
                "#,
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media only all and (resolution: 1.dpi) {
                            padding: 1.px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media only all and (resolution:1dpi){.c{padding:1px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn media_type() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media xxx {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media screen {
                            padding: 1.px;
                        }
                        @media all {
                            padding: 2.px;
                        }
                        @media print and not (resolution: 1.dpi) {
                            padding: 3.px;
                        }
                        @media all and (resolution: 2.dpi) {
                            padding: 4.px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media screen{.c{padding:1px}}@media all{.c{padding:2px}}@media print and not (resolution:1dpi){.c{padding:3px}}@media(resolution:2dpi){.c{padding:4px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn aspect_ratio() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media (aspect-ratio: 16/0) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media (aspect-ratio: 16/9), (min-aspect-ratio: 4/3), (max-aspect-ratio: 2/1) {
                            padding: 1px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media(aspect-ratio:16/9),(min-aspect-ratio:4/3),(max-aspect-ratio:2/1){.c{padding:1px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn orientation() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media (orientation: xxx) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media not (orientation: landscape) and (orientation: portrait) {
                            padding: 1px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media not (orientation:landscape)and (orientation:portrait){.c{padding:1px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn prefers_color_scheme() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media (prefers-color-scheme: xxx) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media (prefers-color-scheme: light), not (prefers-color-scheme: dark) {
                            padding: 1px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media(prefers-color-scheme:light),not (prefers-color-scheme:dark){.c{padding:1px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn resolution() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media (resolution: 1px) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media (resolution: 1dpi), (min-resolution: 1.1dpi), (max-resolution: 2dpi) {
                            padding: 1px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media(resolution:1dpi),(min-resolution:1.1dpi),(max-resolution:2dpi){.c{padding:1px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn width() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media (width: 1dpi) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media (width: 1px), (min-width: 2px), (max-width: 3px) {
                            padding: 1px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media(width:1px),(min-width:2px),(max-width:3px){.c{padding:1px}}"#,
            );
        });
    }

    #[test]
    #[serial]
    fn height() {
        setup_env(false, |env| {
            assert!(syn::parse_str::<StyleSheet<DomStyleSheet>>(
                r#"
                    .c {
                        @media (height: 1dpi) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    @config name_mangling: off;
                    .c {
                        @media (height: 1px), (min-height: 2px), (max-height: 3px) {
                            padding: 1px;
                        }
                    }
                "#,
            );
            assert_eq!(
                env.read_output(),
                r#"@media(height:1px),(min-height:2px),(max-height:3px){.c{padding:1px}}"#,
            );
        });
    }
}
