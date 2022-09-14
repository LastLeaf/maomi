use std::num::NonZeroU32;
use syn::{Error, spanned::Spanned};

use maomi_skin::parser::{CssDimension, ParseStyleSheetValue, CssIdent, CssTokenStream};

pub(crate) enum DomMediaCondValue {
    AspectRatio(NonZeroU32, NonZeroU32),
    MinAspectRatio(NonZeroU32, NonZeroU32),
    MaxAspectRatio(NonZeroU32, NonZeroU32),
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

fn parse_aspect_ratio(tokens: &mut CssTokenStream) -> syn::Result<(NonZeroU32, NonZeroU32)> {
    let span = tokens.span();
    let a = tokens.expect_integer()?;
    if a < 0 || a > u32::MAX as i64 {
        return Err(Error::new(span, "Expected positive integer"));
    }
    let _ = tokens.expect_delim("/")?;
    let span = tokens.span();
    let b = tokens.expect_integer()?;
    if b < 0 || b > u32::MAX as i64 {
        return Err(Error::new(span, "Expected positive integer"));
    }
    Ok((NonZeroU32::new(a as u32).unwrap(), NonZeroU32::new(b as u32).unwrap()))
}

impl ParseStyleSheetValue for DomMediaCondValue {
    fn parse_value(
        name: &CssIdent,
        tokens: &mut CssTokenStream,
    ) -> syn::Result<Self> where Self: Sized {
        let ret = match name.formal_name.as_str() {
            "aspect_ratio" => {
                let (a, b) = parse_aspect_ratio(tokens)?;
                Self::AspectRatio(a, b)
            }
            "min_aspect_ratio" => {
                let (a, b) = parse_aspect_ratio(tokens)?;
                Self::MinAspectRatio(a, b)
            }
            // TODO
            _ => {
                return Err(Error::new(name.span(), "Unknown media feature"));
            }
        };
        Ok(ret)
    }
}
