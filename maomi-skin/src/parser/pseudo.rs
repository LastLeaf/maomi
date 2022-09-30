use syn::parse::Parse;
use syn::*;

use crate::parser::CssIdent;

use super::write_css::WriteCss;

/// The supported pseudo classes
///
/// The list is found in [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Pseudo-classes) .
/// Tree-structural pseudo-classes is not included,
/// since it can be handled correctly though template.
pub enum Pseudo {
    Fullscreen,
    Modal,
    PictureInPicture,
    Autofill,
    Enabled,
    Disabled,
    ReadOnly,
    ReadWrite,
    PlaceholderShown,
    Default,
    Checked,
    Blank,
    Valid,
    Invalid,
    InRange,
    OutOfRange,
    Required,
    Optional,
    UserInvalid,
    Dir(PseudoDir),
    Lang(CssIdent),
    AnyLink,
    Link,
    Visited,
    LocalLink,
    Target,
    TargetWithin,
    Scope,
    Playing,
    Paused,
    Current,
    Past,
    Future,
    Hover,
    Active,
    Focus,
    FocusVisible,
    FocusWithin,
}

pub enum PseudoDir {
    Ltr,
    Rtl,
}

impl Parse for Pseudo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: CssIdent = input.parse()?;
        let ret = match ident.formal_name.as_str() {
            "fullscreen" => Self::Fullscreen,
            "modal" => Self::Modal,
            "picture_in_picture" => Self::PictureInPicture,
            "autofill" => Self::Autofill,
            "enabled" => Self::Enabled,
            "disabled" => Self::Disabled,
            "read_only" => Self::ReadOnly,
            "read_write" => Self::ReadWrite,
            "placeholder_shown" => Self::PlaceholderShown,
            "default" => Self::Default,
            "checked" => Self::Checked,
            "blank" => Self::Blank,
            "valid" => Self::Valid,
            "invalid" => Self::Invalid,
            "in_range" => Self::InRange,
            "out_of_range" => Self::OutOfRange,
            "required" => Self::Required,
            "optional" => Self::Optional,
            "user_invalid" => Self::UserInvalid,
            "dir" => {
                let content;
                parenthesized!(content in input);
                let s: CssIdent = content.parse()?;
                match s.formal_name.as_str() {
                    "ltr" => Self::Dir(PseudoDir::Ltr),
                    "rtl" => Self::Dir(PseudoDir::Rtl),
                    _ => {
                        return Err(Error::new(s.span, "Unknown dir"))
                    }
                }
            }
            "lang" => {
                let content;
                parenthesized!(content in input);
                Self::Lang(content.parse()?)
            }
            "any_link" => Self::AnyLink,
            "link" => Self::Link,
            "visited" => Self::Visited,
            "local_link" => Self::LocalLink,
            "target" => Self::Target,
            "target_within" => Self::TargetWithin,
            "scope" => Self::Scope,
            "playing" => Self::Playing,
            "paused" => Self::Paused,
            "current" => Self::Current,
            "past" => Self::Past,
            "future" => Self::Future,
            "hover" => Self::Hover,
            "active" => Self::Active,
            "focus" => Self::Focus,
            "focus_visible" => Self::FocusVisible,
            "focus_within" => Self::FocusWithin,
            _ => {
                return Err(Error::new(ident.span, "Unknown pseudo class"))
            }
        };
        Ok(ret)
    }
}

impl WriteCss for Pseudo {
    fn write_css<W: std::fmt::Write>(&self, cssw: &mut super::write_css::CssWriter<W>) -> std::fmt::Result {
        match self {
            Self::Fullscreen => cssw.write_ident("fullscreen", false),
            Self::Modal => cssw.write_ident("modal", false),
            Self::PictureInPicture => cssw.write_ident("picture-in-picture", false),
            Self::Autofill => cssw.write_ident("autofill", false),
            Self::Enabled => cssw.write_ident("enabled", false),
            Self::Disabled => cssw.write_ident("disabled", false),
            Self::ReadOnly => cssw.write_ident("read-only", false),
            Self::ReadWrite => cssw.write_ident("read-write", false),
            Self::PlaceholderShown => cssw.write_ident("placeholder-shown", false),
            Self::Default => cssw.write_ident("default", false),
            Self::Checked => cssw.write_ident("checked", false),
            Self::Blank => cssw.write_ident("blank", false),
            Self::Valid => cssw.write_ident("valid", false),
            Self::Invalid => cssw.write_ident("invalid", false),
            Self::InRange => cssw.write_ident("in-range", false),
            Self::OutOfRange => cssw.write_ident("out-of-range", false),
            Self::Required => cssw.write_ident("required", false),
            Self::Optional => cssw.write_ident("optional", false),
            Self::UserInvalid => cssw.write_ident("user-invalid", false),
            Self::Dir(dir) => {
                cssw.write_function_block(false, "dir", |cssw| {
                    match dir {
                        PseudoDir::Ltr => cssw.write_ident("ltr", true),
                        PseudoDir::Rtl => cssw.write_ident("rtl", true),
                    }
                })
            }
            Self::Lang(lang) => {
                cssw.write_function_block(false, "lang", |cssw| {
                    cssw.write_ident(lang.css_name().as_str(), true)
                })
            }
            Self::AnyLink => cssw.write_ident("any-link", false),
            Self::Link => cssw.write_ident("link", false),
            Self::Visited => cssw.write_ident("visited", false),
            Self::LocalLink => cssw.write_ident("local-link", false),
            Self::Target => cssw.write_ident("target", false),
            Self::TargetWithin => cssw.write_ident("target-within", false),
            Self::Scope => cssw.write_ident("scope", false),
            Self::Playing => cssw.write_ident("playing", false),
            Self::Paused => cssw.write_ident("paused", false),
            Self::Current => cssw.write_ident("current", false),
            Self::Past => cssw.write_ident("past", false),
            Self::Future => cssw.write_ident("future", false),
            Self::Hover => cssw.write_ident("hover", false),
            Self::Active => cssw.write_ident("active", false),
            Self::Focus => cssw.write_ident("focus", false),
            Self::FocusVisible => cssw.write_ident("focus-visible", false),
            Self::FocusWithin => cssw.write_ident("focus-within", false),
        }
    }
}
