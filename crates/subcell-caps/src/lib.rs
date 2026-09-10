//! Terminal capability description and response parsing.
//!
//! Every other crate branches on [`Caps`]. The rule the whole design rests on is
//! that resolution is a *runtime* property: ask the terminal what it can do, then
//! pick a renderer. Nothing downstream is allowed to assume a tier.
//!
//! What lives here today is the vocabulary and the response parsers — the pure,
//! testable half. Driving a real terminal (raw mode, writing the queries,
//! reading replies under a timeout) is the I/O half and is tracked separately;
//! see `docs/capabilities.md`.

use subcell_blit::Tier;

/// Pixel dimensions reported by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    /// Width in pixels.
    pub w: u16,
    /// Height in pixels.
    pub h: u16,
}

impl Size {
    /// Constructs a size.
    pub const fn new(w: u16, h: u16) -> Self {
        Self { w, h }
    }
}

/// Which pixel-graphics protocol the terminal speaks, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Graphics {
    /// No pixel protocol. Glyph tiers only.
    #[default]
    None,
    /// The kitty graphics protocol, including Unicode placeholder placement.
    Kitty,
    /// Sixel, with the number of simultaneously addressable colours.
    Sixel {
        /// Palette size the terminal advertises.
        colors: u16,
    },
}

/// A terminal multiplexer sitting between the application and the terminal.
///
/// Multiplexers break graphics passthrough in ways that are not worth
/// working around, so detecting one is a reason to drop to a glyph tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Muxer {
    /// GNU screen.
    Screen,
    /// tmux.
    Tmux,
}

/// Which underline features are available as a drawing channel.
// A capability set is a bag of independent flags; grouping them to satisfy
// `struct_excessive_bools` would invent a hierarchy that does not exist.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnderlineCaps {
    /// `SGR 58` sets an underline colour independent of the foreground.
    pub colored: bool,
    /// Styled underlines: double, curly, dotted, dashed.
    pub styled: bool,
    /// `SGR 53` overline.
    pub overline: bool,
    /// `SGR 9` strikethrough.
    pub strikethrough: bool,
}

/// Kitty keyboard protocol progressive-enhancement flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KittyKbd {
    /// Keys are disambiguated, so `Ctrl+I` is distinct from `Tab`.
    pub disambiguate: bool,
    /// Key release and repeat events are reported, not just presses.
    pub report_events: bool,
    /// All keys report as escape codes, including plain text keys.
    pub report_all: bool,
}

impl KittyKbd {
    /// Decodes the flag bitfield from a `CSI ? flags u` reply.
    pub const fn from_bits(bits: u8) -> Self {
        Self {
            disambiguate: bits & 0b1 != 0,
            report_events: bits & 0b10 != 0,
            report_all: bits & 0b1000 != 0,
        }
    }
}

/// Everything the renderer needs to choose a backend.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Caps {
    /// Size of one cell in pixels. This is the terminal's device pixel ratio.
    pub cell_px: Option<Size>,
    /// Size of the whole text area in pixels.
    pub area_px: Option<Size>,
    /// Pixel graphics protocol, if any.
    pub graphics: Graphics,
    /// Highest glyph tier believed safe to emit.
    pub blit_max: Option<Tier>,
    /// Rule-drawing channels.
    pub underline: UnderlineCaps,
    /// `SGR 1016`: mouse position reported in pixels rather than cells.
    pub mouse_px: bool,
    /// Kitty keyboard protocol flags.
    pub kbd: KittyKbd,
    /// `DECSET 2026`: synchronized output, for tear-free frames.
    pub sync: bool,
    /// A multiplexer was detected between us and the terminal.
    pub multiplex: Option<Muxer>,
}

impl Caps {
    /// The capability set assumed when nothing could be detected.
    ///
    /// Half blocks and 24-bit colour are close enough to universal that this is
    /// a safe floor; everything else is off.
    pub const fn conservative() -> Self {
        Self {
            cell_px: None,
            area_px: None,
            graphics: Graphics::None,
            blit_max: Some(Tier::Half),
            underline: UnderlineCaps {
                colored: false,
                styled: false,
                overline: false,
                strikethrough: false,
            },
            mouse_px: false,
            kbd: KittyKbd { disambiguate: false, report_events: false, report_all: false },
            sync: false,
            multiplex: None,
        }
    }

    /// The tier the renderer should actually use.
    ///
    /// Falls back to [`Tier::Half`] when nothing was detected, and refuses to
    /// exceed [`Tier::Quadrant`] behind a multiplexer, where glyph coverage
    /// depends on a terminal we cannot interrogate directly.
    pub fn effective_tier(&self) -> Tier {
        let tier = self.blit_max.unwrap_or(Tier::Half);
        match self.multiplex {
            Some(_) if tier > Tier::Quadrant => Tier::Quadrant,
            _ => tier,
        }
    }

    /// Whether the pixel tier is usable.
    ///
    /// Requires both a graphics protocol and a known cell size — without the
    /// latter there is no way to rasterize at the right scale.
    pub fn pixel_tier_available(&self) -> bool {
        self.graphics != Graphics::None && self.cell_px.is_some() && self.multiplex.is_none()
    }
}

/// A parsed terminal reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    /// `CSI 4 ; h ; w t` — text area size in pixels, from `CSI 14 t`.
    AreaPixels(Size),
    /// `CSI 6 ; h ; w t` — cell size in pixels, from `CSI 16 t`.
    CellPixels(Size),
    /// `CSI 8 ; rows ; cols t` — text area size in cells, from `CSI 18 t`.
    AreaCells(Size),
}

/// Parses an XTWINOPS reply.
///
/// Returns `None` if the bytes are not a complete, well-formed reply. Note the
/// wire order is height before width, which is the opposite of the way every
/// other part of this codebase writes it.
///
/// ```
/// use subcell_caps::{parse_report, Report, Size};
/// assert_eq!(parse_report(b"\x1b[6;17;8t"), Some(Report::CellPixels(Size::new(8, 17))));
/// ```
pub fn parse_report(bytes: &[u8]) -> Option<Report> {
    let body = bytes.strip_prefix(b"\x1b[")?.strip_suffix(b"t")?;
    let text = std::str::from_utf8(body).ok()?;
    let mut parts = text.split(';');
    let kind: u8 = parts.next()?.parse().ok()?;
    let h: u16 = parts.next()?.parse().ok()?;
    let w: u16 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    match kind {
        4 => Some(Report::AreaPixels(Size::new(w, h))),
        6 => Some(Report::CellPixels(Size::new(w, h))),
        8 => Some(Report::AreaCells(Size::new(w, h))),
        _ => None,
    }
}

/// Detects a multiplexer from the process environment.
///
/// Reads `TERM` and `TMUX` the way every terminal library does. It is not
/// authoritative — a nested session can hide it — but a positive result is
/// reliable enough to degrade on.
pub fn detect_muxer(term: Option<&str>, tmux: Option<&str>) -> Option<Muxer> {
    if tmux.is_some_and(|v| !v.is_empty()) {
        return Some(Muxer::Tmux);
    }
    match term {
        Some(t) if t.starts_with("tmux") => Some(Muxer::Tmux),
        Some(t) if t.starts_with("screen") => Some(Muxer::Screen),
        _ => None,
    }
}

/// The queries a probe writes, in the order it should write them.
///
/// All of these are answered by a reply or ignored, so a terminal that does not
/// understand one stays silent rather than corrupting the stream. The trailing
/// primary device attributes request is the fence: every terminal answers it, so
/// its reply marks the end of the batch and bounds the wait.
pub const PROBE_SEQUENCE: &[&str] = &[
    "\x1b[16t",     // cell size in pixels
    "\x1b[14t",     // text area size in pixels
    "\x1b[18t",     // text area size in cells
    "\x1b[?u",      // kitty keyboard flags
    "\x1b[?2026$p", // synchronized output support
    "\x1b[c",       // primary device attributes: the fence
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cell_pixels() {
        assert_eq!(parse_report(b"\x1b[6;17;8t"), Some(Report::CellPixels(Size::new(8, 17))));
    }

    #[test]
    fn parses_area_pixels_and_cells() {
        assert_eq!(parse_report(b"\x1b[4;680;960t"), Some(Report::AreaPixels(Size::new(960, 680))));
        assert_eq!(parse_report(b"\x1b[8;40;120t"), Some(Report::AreaCells(Size::new(120, 40))));
    }

    #[test]
    fn rejects_malformed_replies() {
        for bad in [
            &b"\x1b[6;17t"[..],      // too few fields
            &b"\x1b[6;17;8;9t"[..],  // too many
            &b"\x1b[6;17;8"[..],     // unterminated
            &b"6;17;8t"[..],         // no CSI
            &b"\x1b[9;17;8t"[..],    // unknown report kind
            &b"\x1b[6;abc;8t"[..],   // not a number
            &b"\x1b[6;99999;8t"[..], // overflows u16
        ] {
            assert_eq!(parse_report(bad), None, "accepted {bad:?}");
        }
    }

    #[test]
    fn dpr_falls_out_of_two_reports() {
        let Some(Report::AreaPixels(area)) = parse_report(b"\x1b[4;680;960t") else {
            panic!("area reply did not parse")
        };
        let Some(Report::AreaCells(cells)) = parse_report(b"\x1b[8;40;120t") else {
            panic!("cell reply did not parse")
        };
        assert_eq!(area.w / cells.w, 8);
        assert_eq!(area.h / cells.h, 17);
    }

    #[test]
    fn conservative_floor_is_half_blocks() {
        let caps = Caps::conservative();
        assert_eq!(caps.effective_tier(), Tier::Half);
        assert!(!caps.pixel_tier_available());
    }

    #[test]
    fn multiplexer_caps_the_glyph_tier() {
        let caps = Caps {
            blit_max: Some(Tier::Braille),
            multiplex: Some(Muxer::Tmux),
            ..Caps::conservative()
        };
        assert_eq!(caps.effective_tier(), Tier::Quadrant);
    }

    #[test]
    fn multiplexer_disables_the_pixel_tier() {
        let caps = Caps {
            graphics: Graphics::Kitty,
            cell_px: Some(Size::new(8, 17)),
            multiplex: Some(Muxer::Tmux),
            ..Caps::conservative()
        };
        assert!(!caps.pixel_tier_available());
    }

    #[test]
    fn pixel_tier_needs_a_known_cell_size() {
        let no_size = Caps { graphics: Graphics::Kitty, ..Caps::conservative() };
        assert!(!no_size.pixel_tier_available());
        let sized = Caps { cell_px: Some(Size::new(8, 17)), ..no_size };
        assert!(sized.pixel_tier_available());
    }

    #[test]
    fn muxer_detection() {
        assert_eq!(detect_muxer(Some("xterm-256color"), None), None);
        assert_eq!(detect_muxer(Some("xterm-256color"), Some("/tmp/x,1,0")), Some(Muxer::Tmux));
        assert_eq!(detect_muxer(Some("tmux-256color"), None), Some(Muxer::Tmux));
        assert_eq!(detect_muxer(Some("screen.xterm"), None), Some(Muxer::Screen));
        assert_eq!(detect_muxer(None, Some("")), None);
    }

    #[test]
    fn kbd_flag_decoding() {
        assert_eq!(KittyKbd::from_bits(0), KittyKbd::default());
        let all = KittyKbd::from_bits(0b1011);
        assert!(all.disambiguate && all.report_events && all.report_all);
    }

    #[test]
    fn probe_sequence_ends_with_the_fence() {
        assert_eq!(PROBE_SEQUENCE.last(), Some(&"\x1b[c"));
    }
}
