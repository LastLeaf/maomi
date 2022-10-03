use std::fmt::Display;

/// Marker for a type that is i18n friendly
///
/// When i18n support is enabled,
/// only types which implemented this trait can be used in text node.
pub trait ToLocaleStr {
    fn to_locale_str(&self) -> &str;
}

/// A translated static str
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct LocaleStaticStr(&'static str);

impl LocaleStaticStr {
    /// Wraps a translated str
    pub const fn translated(s: &'static str) -> Self {
        Self(s)
    }

    /// Convert to a `LocaleString`
    pub fn to_locale_string(&self) -> LocaleString {
        LocaleString(self.0.to_string())
    }
}

impl ToLocaleStr for LocaleStaticStr {
    fn to_locale_str(&self) -> &str {
        self.0
    }
}

impl Display for LocaleStaticStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A translated string
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct LocaleString(String);

impl LocaleString {
    /// Wraps a translated string
    pub fn translated(s: &impl ToString) -> Self {
        Self(s.to_string())
    }
}

impl ToLocaleStr for LocaleString {
    fn to_locale_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T: ?Sized + ToLocaleStr> ToLocaleStr for &T {
    fn to_locale_str(&self) -> &str {
        (*self).to_locale_str()
    }
}

impl Display for LocaleString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}