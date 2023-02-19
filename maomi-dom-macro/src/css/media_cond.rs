use maomi_skin::{write_css::*, css_token::*, style_sheet::ParseStyleSheetValue, ParseError, VarDynValue};

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
    fn write_css_with_args<W: CssWriteTarget>(
        &self,
        cssw: &mut CssWriter<W>,
        values: &[VarDynValue],
    ) -> std::fmt::Result {
        match self {
            Self::AspectRatio(a, b) => {
                a.write_css_with_args(cssw, values)?;
                cssw.write_delim("/", true)?;
                b.write_css_with_args(cssw, values)?;
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
            Self::Resolution(x) => x.write_css_with_args(cssw, values)?,
            Self::Width(x) => x.write_css_with_args(cssw, values)?,
            Self::Height(x) => x.write_css_with_args(cssw, values)?,
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
                    class c {
                        if media only (resolution = Dpi(1)) {}
                    }
                "#,
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media only all and (resolution = Dpi(1)) {
                            padding = Px(1);
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
                    class c {
                        if media xxx {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media screen {
                            padding = Px(1);
                        }
                        if media all {
                            padding = Px(2);
                        }
                        if media print and not (resolution = Dpi(1)) {
                            padding = Px(3);
                        }
                        if media all and (resolution = Dpi(2)) {
                            padding = Px(4);
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
                    class c {
                        if media (aspect_ratio = 16/0) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media (aspect_ratio = 16/9), (min_aspect_ratio = 4/3), (max_aspect_ratio = 2/1) {
                            padding = Px(1);
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
                    class c {
                        if media (orientation = xxx) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media not (orientation = landscape) and (orientation = portrait) {
                            padding = Px(1);
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
                    class c {
                        if media (prefers_color_scheme = xxx) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media (prefers_color_scheme = light), not (prefers_color_scheme = dark) {
                            padding = Px(1);
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
                    class c {
                        if media (resolution = Px(1)) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media (resolution = Dpi(1)), (min_resolution = Dpi(1.1)), (max_resolution = Dpi(2)) {
                            padding = Px(1);
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
                    class c {
                        if media (width = Dpi(1)) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media (width = Px(1)), (min_width = Px(2)), (max_width = Px(3)) {
                            padding = Px(1);
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
                    class c {
                        if media (height = Dpi(1)) {}
                    }
                "#
            )
            .is_err());
            parse_str(
                r#"
                    #[css_name("c")]
                    class c {
                        if media (height = Px(1)), (min_height = Px(2)), (max_height = Px(3)) {
                            padding = Px(1);
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
