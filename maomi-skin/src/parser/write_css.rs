use std::fmt::{Write, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineStatus {
    BlockStart,
    LineStart,
    Other,
}

/// A CSS writer
pub struct CssWriter<'a, W: Write> {
    pub(crate) w: &'a mut W,
    pub(crate) sc: WriteCssSepCond,
    pub(crate) debug_mode: bool,
    tab_count: usize,
    line_status: LineStatus,
}

impl<'a, W: Write> CssWriter<'a, W> {
    pub fn new(w: &'a mut W, debug_mode: bool) -> Self {
        Self {
            w,
            sc: WriteCssSepCond::BlockStart,
            debug_mode,
            tab_count: 0,
            line_status: LineStatus::BlockStart,
        }
    }

    pub fn line_wrap(&mut self) -> Result {
        if !self.debug_mode {
            return Ok(());
        }
        if self.line_status == LineStatus::Other {
            self.line_status = LineStatus::LineStart;
        }
        write!(self.w, "\n")
    }

    fn prepare_write(&mut self) -> Result {
        if !self.debug_mode {
            return Ok(());
        }
        if self.line_status == LineStatus::BlockStart {
            self.line_status = LineStatus::LineStart;
            write!(self.w, "\n")?;
        }
        if self.line_status == LineStatus::LineStart {
            for _ in 0..self.tab_count {
                write!(self.w, "    ")?;
            }
            self.line_status = LineStatus::Other;
        }
        Ok(())
    }

    pub(crate) fn custom_write(
        &mut self,
        f: impl FnOnce(&mut W, WriteCssSepCond, bool) -> std::result::Result<WriteCssSepCond, std::fmt::Error>,
    ) -> Result {
        self.prepare_write()?;
        let CssWriter { ref mut w, ref mut sc, debug_mode, .. } = self;
        *sc = f(w, *sc, *debug_mode)?;
        Ok(())
    }

    pub fn write_ident(&mut self, ident: &str) -> Result {
        self.prepare_write()?;
        let CssWriter { ref mut w, ref mut sc, debug_mode, .. } = self;
        if *debug_mode {
            match sc {
                WriteCssSepCond::BlockStart => {}
                _ => {
                    write!(w, " ")?;
                }
            }
        } else {
            match sc {
                WriteCssSepCond::Ident
                | WriteCssSepCond::NonIdentAlpha
                | WriteCssSepCond::Digit
                | WriteCssSepCond::At => {
                    write!(w, " ")?;
                }
                _ => {}
            }
        }
        write!(w, "{}", ident)?;
        *sc = WriteCssSepCond::Ident;
        Ok(())
    }

    pub fn write_function_block(&mut self, name: &str, f: impl FnOnce(&mut Self) -> Result) -> Result {
        self.prepare_write()?;
        {
            let CssWriter { ref mut w, ref mut sc, debug_mode, .. } = self;
            if *debug_mode {
                write!(w, " ")?;
            } else {
                match sc {
                    WriteCssSepCond::Ident
                    | WriteCssSepCond::NonIdentAlpha
                    | WriteCssSepCond::Digit
                    | WriteCssSepCond::At => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            write!(w, "{}(", name)?;
            *sc = WriteCssSepCond::BlockStart;
        }
        f(self)?;
        {
            let CssWriter { ref mut w, ref mut sc, .. } = self;
            write!(w, ")")?;
            *sc = WriteCssSepCond::Other;
        }
        Ok(())
    }

    pub fn write_paren_block(&mut self, f: impl FnOnce(&mut Self) -> Result) -> Result {
        self.prepare_write()?;
        {
            let CssWriter { ref mut w, ref mut sc, debug_mode, .. } = self;
            if *debug_mode {
                write!(w, " ")?;
            } else {
                match sc {
                    WriteCssSepCond::Ident => {
                        write!(w, " ")?;
                    }
                    _ => {}
                }
            }
            write!(w, "(")?;
            *sc = WriteCssSepCond::BlockStart;
        }
        f(self)?;
        {
            let CssWriter { ref mut w, ref mut sc, .. } = self;
            write!(w, ")")?;
            *sc = WriteCssSepCond::Other;
        }
        Ok(())
    }

    pub fn write_bracket_block(&mut self, f: impl FnOnce(&mut Self) -> Result) -> Result {
        self.prepare_write()?;
        {
            let CssWriter { ref mut w, ref mut sc, debug_mode, .. } = self;
            if *debug_mode {
                write!(w, " ")?;
            }
            write!(w, "[")?;
            *sc = WriteCssSepCond::BlockStart;
        }
        f(self)?;
        {
            let CssWriter { ref mut w, ref mut sc, .. } = self;
            write!(w, "]")?;
            *sc = WriteCssSepCond::Other;
        }
        Ok(())
    }

    pub fn write_brace_block(&mut self, f: impl FnOnce(&mut Self) -> Result) -> Result {
        self.prepare_write()?;
        {
            let CssWriter { ref mut w, ref mut sc, debug_mode, .. } = self;
            if *debug_mode {
                write!(w, " ")?;
            }
            write!(w, "{{")?;
            *sc = WriteCssSepCond::BlockStart;
        }
        self.tab_count += 1;
        self.line_wrap()?;
        f(self)?;
        self.tab_count -= 1;
        self.line_wrap()?;
        {
            let CssWriter { ref mut w, ref mut sc, .. } = self;
            write!(w, "}}")?;
            *sc = WriteCssSepCond::Other;
        }
        self.line_wrap()?;
        self.line_status = LineStatus::BlockStart;
        Ok(())
    }
}

/// Display as CSS text
pub trait WriteCss {
    /// Write CSS text
    fn write_css<W: Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> Result;
}

/// Separator indicator for `WriteCss`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WriteCssSepCond {
    /// The CSS string ends with `CssIdent`
    ///
    /// It should not be followed by alphabets, digits, `-`, or `(` .
    Ident,
    /// The CSS string ends with alphabets or digits (but not an ident nor number), `-` or `#`
    ///
    /// It should not be followed by alphabets, digits, or `-` .
    NonIdentAlpha,
    /// The CSS string ends with `CssNumber`
    ///
    /// It should not be followed by alphabets, digits, `.`, `-`, or `%` .
    Digit,
    /// The CSS string ends with `@`
    ///
    /// It should not be followed by alphabets or `-` .
    At,
    /// The CSS string ends with `.` `+`
    ///
    /// It should not be followed by digits.
    DotOrPlus,
    /// The CSS string ends with `$` `^` `~` `*`
    ///
    /// It should not be followed by `=` .
    Equalable,
    /// The CSS string ends with `|`
    ///
    /// It should not be followed by `=` `|` `|=` .
    Bar,
    /// The CSS string ends with `/`
    ///
    /// It should not be followed by `*` `*=` .
    Slash,
    /// The CSS string ends with `(` `[` `{`
    ///
    /// Always no separators needed, but separators may be added in debug mode.
    BlockStart,
    /// Other cases
    ///
    /// Always no separators needed, but separators may be added in debug mode.
    Other,
}

impl<V: WriteCss> WriteCss for Option<V> {
    fn write_css<W: std::fmt::Write>(
        &self,
        cssw: &mut CssWriter<W>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Some(x) => x.write_css(cssw)?,
            None => {}
        }
        Ok(())
    }
}
