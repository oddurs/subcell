//! Underline, overline and strikethrough as drawing channels.
//!
//! These three attributes are three independently positioned horizontal rules
//! inside every cell, and `SGR 58` gives the underline a colour unrelated to the
//! foreground. They composite *over* whatever glyph and colours the cell already
//! carries, so they cost no spatial resolution at all — you can render syntax
//! highlighted text and draw a coloured signal through the same cells.
//!
//! Each channel degrades independently: without `SGR 58` a rule inherits the
//! foreground; without styled underlines it falls back to solid.

use std::fmt::Write as _;

use subcell_color::Srgb8;

/// Which of the three rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rule {
    /// `SGR 53`, drawn at the top of the cell.
    Over,
    /// `SGR 9`, drawn through the middle.
    Strike,
    /// `SGR 4`, drawn at the bottom.
    Under,
}

/// Underline styles from the extended `SGR 4:n` form.
///
/// Only the under rule supports styles; over and strike are always solid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Style {
    /// `4:1`, a single line.
    #[default]
    Solid,
    /// `4:2`, two lines.
    Double,
    /// `4:3`, a wave.
    Curly,
    /// `4:4`, dots.
    Dotted,
    /// `4:5`, dashes.
    Dashed,
}

impl Style {
    /// The numeric parameter in `SGR 4:n`.
    pub const fn param(self) -> u8 {
        match self {
            Style::Solid => 1,
            Style::Double => 2,
            Style::Curly => 3,
            Style::Dotted => 4,
            Style::Dashed => 5,
        }
    }
}

/// The rules attached to one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rules {
    /// Underline: style and optional independent colour.
    pub under: Option<(Style, Option<Srgb8>)>,
    /// Overline, if drawn.
    pub over: bool,
    /// Strikethrough, if drawn.
    pub strike: bool,
}

impl Rules {
    /// No rules.
    pub const NONE: Rules = Rules { under: None, over: false, strike: false };

    /// Whether any rule is set.
    pub fn is_empty(&self) -> bool {
        *self == Rules::NONE
    }

    /// Emits the SGR parameters for these rules.
    ///
    /// `colored` and `styled` come from the capability probe. When `styled` is
    /// false every underline is emitted as a plain `4`; when `colored` is false
    /// the underline colour is dropped and the rule inherits the foreground.
    pub fn sgr(&self, colored: bool, styled: bool) -> String {
        let mut out = String::new();
        if let Some((style, color)) = self.under {
            if styled && style != Style::Solid {
                let _ = write!(out, "\x1b[4:{}m", style.param());
            } else {
                out.push_str("\x1b[4m");
            }
            if let (true, Some(c)) = (colored, color) {
                let _ = write!(out, "\x1b[58:2::{}:{}:{}m", c.r, c.g, c.b);
            }
        }
        if self.over {
            out.push_str("\x1b[53m");
        }
        if self.strike {
            out.push_str("\x1b[9m");
        }
        out
    }

    /// The reset sequence for whatever this cell set.
    pub fn reset(&self) -> String {
        let mut out = String::new();
        if self.under.is_some() {
            out.push_str("\x1b[24m\x1b[59m");
        }
        if self.over {
            out.push_str("\x1b[55m");
        }
        if self.strike {
            out.push_str("\x1b[29m");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CYAN: Srgb8 = Srgb8::new(63, 199, 220);

    #[test]
    fn empty_rules_emit_nothing() {
        assert_eq!(Rules::NONE.sgr(true, true), "");
        assert_eq!(Rules::NONE.reset(), "");
        assert!(Rules::NONE.is_empty());
    }

    #[test]
    fn curly_coloured_underline() {
        let r = Rules { under: Some((Style::Curly, Some(CYAN))), ..Rules::NONE };
        assert_eq!(r.sgr(true, true), "\x1b[4:3m\x1b[58:2::63:199:220m");
    }

    #[test]
    fn falls_back_to_solid_without_styled_support() {
        let r = Rules { under: Some((Style::Curly, Some(CYAN))), ..Rules::NONE };
        assert_eq!(r.sgr(true, false), "\x1b[4m\x1b[58:2::63:199:220m");
    }

    #[test]
    fn drops_colour_without_sgr58() {
        let r = Rules { under: Some((Style::Curly, Some(CYAN))), ..Rules::NONE };
        assert_eq!(r.sgr(false, true), "\x1b[4:3m");
    }

    #[test]
    fn three_channels_are_independent() {
        let r = Rules { under: Some((Style::Dotted, None)), over: true, strike: true };
        let s = r.sgr(true, true);
        assert!(s.contains("\x1b[4:4m") && s.contains("\x1b[53m") && s.contains("\x1b[9m"));
        let reset = r.reset();
        assert!(
            reset.contains("\x1b[24m") && reset.contains("\x1b[55m") && reset.contains("\x1b[29m")
        );
    }

    #[test]
    fn reset_only_undoes_what_was_set() {
        let r = Rules { over: true, ..Rules::NONE };
        assert_eq!(r.reset(), "\x1b[55m");
    }

    #[test]
    fn style_parameters() {
        assert_eq!(Style::default(), Style::Solid);
        assert_eq!(Style::Curly.param(), 3);
        assert_eq!(Style::Dashed.param(), 5);
    }

    #[test]
    fn rules_are_ordered_top_to_bottom() {
        assert!(Rule::Over < Rule::Strike && Rule::Strike < Rule::Under);
    }
}
