//! Pixel-tier backends.
//!
//! The kitty graphics protocol is the only pixel protocol that composes properly
//! with a terminal UI, because of Unicode placeholders: an image is uploaded
//! once and then *referenced* by placeholder cells, so the layout engine and the
//! damage tracker keep owning the grid. The image is placed through the grid,
//! not painted over it, which means it scrolls and clips correctly for free.
//!
//! What is implemented here is the wire format — building and escaping the APC
//! commands, which is pure and testable. Placeholder diacritic encoding and the
//! shared-memory transfer path are tracked separately; see `docs/pixel-tier.md`.

use subcell_caps::{Caps, Graphics};

/// An image handle assigned by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(pub u32);

/// How an image should be placed once transmitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Placed at the current cursor position and owned by the terminal.
    Cursor,
    /// Referenced by Unicode placeholder cells, so the grid keeps ownership.
    ///
    /// This is the mode a terminal UI wants: the renderer emits placeholder
    /// cells as part of its normal cell output and the terminal composites the
    /// image behind them.
    Virtual,
}

/// The placeholder codepoint that references a virtually placed image.
pub const PLACEHOLDER: char = '\u{10EEEE}';

/// Builds the APC command that transmits an RGBA image.
///
/// `width` and `height` are in pixels. The payload is base64 of tightly packed
/// RGBA bytes, which the protocol calls format 32.
///
/// # Errors
///
/// Returns [`PxError::BadDimensions`] if either dimension is zero, or
/// [`PxError::PayloadMismatch`] if `rgba` is not exactly `width * height * 4`
/// bytes.
pub fn transmit(id: ImageId, rgba: &[u8], width: u32, height: u32) -> Result<String, PxError> {
    if width == 0 || height == 0 {
        return Err(PxError::BadDimensions { width, height });
    }
    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() != expected {
        return Err(PxError::PayloadMismatch { expected, got: rgba.len() });
    }
    Ok(format!("\x1b_Ga=t,f=32,t=d,i={},s={width},v={height};{}\x1b\\", id.0, base64(rgba)))
}

/// Builds the APC command that places an already-transmitted image.
///
/// `cols` and `rows` are the cell extent the image should occupy.
pub fn place(id: ImageId, cols: u16, rows: u16, placement: Placement) -> String {
    let unicode = u8::from(placement == Placement::Virtual);
    format!("\x1b_Ga=p,i={},c={cols},r={rows},U={unicode};\x1b\\", id.0)
}

/// Builds the APC command that deletes an image and frees its storage.
pub fn delete(id: ImageId) -> String {
    format!("\x1b_Ga=d,d=I,i={};\x1b\\", id.0)
}

/// Chooses a pixel backend for a capability set.
///
/// Returns `None` whenever the pixel tier is unavailable — no protocol, no known
/// cell size, or a multiplexer in the way — which is the caller's cue to fall
/// back to a glyph tier rather than to try and fail.
pub fn backend_for(caps: &Caps) -> Option<Backend> {
    if !caps.pixel_tier_available() {
        return None;
    }
    match caps.graphics {
        Graphics::Kitty => Some(Backend::Kitty),
        Graphics::Sixel { colors } => Some(Backend::Sixel { colors }),
        Graphics::None => None,
    }
}

/// A selected pixel backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Kitty graphics, with virtual placement available.
    Kitty,
    /// Sixel, palette-limited and without placeholders.
    Sixel {
        /// Advertised palette size.
        colors: u16,
    },
}

impl Backend {
    /// Whether this backend can place images through the cell grid.
    ///
    /// Sixel cannot: it has no z-order and no placeholder mechanism, so it
    /// paints over the grid and fights the damage tracker.
    pub fn supports_virtual_placement(self) -> bool {
        matches!(self, Backend::Kitty)
    }
}

/// Something went wrong building a graphics command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PxError {
    /// An image dimension was zero.
    BadDimensions {
        /// Requested width.
        width: u32,
        /// Requested height.
        height: u32,
    },
    /// The payload length did not match the declared dimensions.
    PayloadMismatch {
        /// Bytes implied by width, height and RGBA.
        expected: usize,
        /// Bytes supplied.
        got: usize,
    },
}

impl std::fmt::Display for PxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PxError::BadDimensions { width, height } => {
                write!(f, "image dimensions must be non-zero, got {width}x{height}")
            }
            PxError::PayloadMismatch { expected, got } => {
                write!(f, "RGBA payload should be {expected} bytes, got {got}")
            }
        }
    }
}

impl std::error::Error for PxError {}

/// Standard base64, no line breaks, as the graphics protocol expects.
fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let q = [n >> 18 & 63, n >> 12 & 63, n >> 6 & 63, n & 63];
        for (i, idx) in q.iter().enumerate() {
            if i <= chunk.len() {
                out.push(T[*idx as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use subcell_caps::{Muxer, Size};

    fn kitty_caps() -> Caps {
        Caps { graphics: Graphics::Kitty, cell_px: Some(Size::new(8, 17)), ..Caps::conservative() }
    }

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn transmit_builds_a_well_formed_apc() {
        let cmd = transmit(ImageId(7), &[0u8; 16], 2, 2).unwrap();
        assert!(cmd.starts_with("\x1b_G"), "must open with APC G");
        assert!(cmd.ends_with("\x1b\\"), "must close with ST");
        assert!(cmd.contains("i=7") && cmd.contains("s=2") && cmd.contains("v=2"));
        assert!(cmd.contains("f=32"), "RGBA is format 32");
    }

    #[test]
    fn transmit_rejects_a_mismatched_payload() {
        let err = transmit(ImageId(1), &[0u8; 15], 2, 2).unwrap_err();
        assert_eq!(err, PxError::PayloadMismatch { expected: 16, got: 15 });
        assert!(err.to_string().contains("16 bytes"));
    }

    #[test]
    fn transmit_rejects_zero_dimensions() {
        assert!(matches!(
            transmit(ImageId(1), &[], 0, 4),
            Err(PxError::BadDimensions { width: 0, height: 4 })
        ));
    }

    #[test]
    fn virtual_placement_sets_the_unicode_flag() {
        assert!(place(ImageId(3), 40, 12, Placement::Virtual).contains("U=1"));
        assert!(place(ImageId(3), 40, 12, Placement::Cursor).contains("U=0"));
    }

    #[test]
    fn delete_targets_the_image_id() {
        assert_eq!(delete(ImageId(9)), "\x1b_Ga=d,d=I,i=9;\x1b\\");
    }

    #[test]
    fn placeholder_is_the_protocol_codepoint() {
        assert_eq!(PLACEHOLDER as u32, 0x0010_EEEE);
    }

    #[test]
    fn backend_selection_follows_capabilities() {
        assert_eq!(backend_for(&kitty_caps()), Some(Backend::Kitty));
        assert_eq!(backend_for(&Caps::conservative()), None);
        let sixel = Caps { graphics: Graphics::Sixel { colors: 256 }, ..kitty_caps() };
        assert_eq!(backend_for(&sixel), Some(Backend::Sixel { colors: 256 }));
    }

    #[test]
    fn a_multiplexer_removes_the_pixel_backend() {
        let muxed = Caps { multiplex: Some(Muxer::Tmux), ..kitty_caps() };
        assert_eq!(backend_for(&muxed), None);
    }

    #[test]
    fn only_kitty_places_through_the_grid() {
        assert!(Backend::Kitty.supports_virtual_placement());
        assert!(!Backend::Sixel { colors: 256 }.supports_virtual_placement());
    }
}
