//! Sub-cell block quantizers.
//!
//! Rendering into a terminal cell is a two-colour block approximation problem,
//! structurally identical to BC1/S3TC texture compression: given a patch of
//! colour, choose a foreground, a background, and the bitmask that says which
//! sub-cell takes which. The glyph is then a lookup on the mask.
//!
//! The important consequence is that this is `O(subcells)`, not `O(glyphs)`.
//! Nothing here searches a candidate glyph table — the clustering produces the
//! mask directly, and the mask indexes the table.
//!
//! # Tiers
//!
//! [`Tier`] ranges from one sample per cell up to eight. Octants (Unicode 16,
//! `U+1CD00`) are the intended top glyph tier and are deliberately **not** here
//! yet: the mask-to-codepoint mapping skips the patterns that existing block
//! elements already cover, and that table has to be generated from Unicode
//! character data rather than typed in by hand. See `docs/blitters.md`.

use subcell_color::{Oklab, Srgb8};

/// A sub-cell sampling density, and the glyph family that expresses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// One sample per cell. Rendered as a space with a background colour.
    Ascii,
    /// `1x2`. Upper and lower half blocks.
    Half,
    /// `2x2`. Quadrant blocks.
    Quadrant,
    /// `2x3`. Sextants, from Unicode 13 Symbols for Legacy Computing.
    Sextant,
    /// `2x4`. Braille patterns. Dots are visually separated and the "off" state
    /// is the cell background, so it carries detail but not two full colours.
    Braille,
}

impl Tier {
    /// Every tier, in ascending order of spatial resolution.
    pub const ALL: [Tier; 5] =
        [Tier::Ascii, Tier::Half, Tier::Quadrant, Tier::Sextant, Tier::Braille];

    /// Sub-cell grid dimensions as `(width, height)`.
    pub const fn sub(self) -> (u8, u8) {
        match self {
            Tier::Ascii => (1, 1),
            Tier::Half => (1, 2),
            Tier::Quadrant => (2, 2),
            Tier::Sextant => (2, 3),
            Tier::Braille => (2, 4),
        }
    }

    /// Number of sub-cells this tier resolves per cell.
    pub const fn samples(self) -> usize {
        let (w, h) = self.sub();
        (w as usize) * (h as usize)
    }

    /// The glyph for a sub-cell bitmask, in row-major order from the top left.
    ///
    /// Returns `None` if `mask` has bits set beyond [`Tier::samples`].
    pub fn glyph(self, mask: u8) -> Option<char> {
        if self != Tier::Ascii && u32::from(mask) >= (1u32 << self.samples()) {
            return None;
        }
        Some(match self {
            Tier::Ascii => ' ',
            Tier::Half => [' ', '\u{2580}', '\u{2584}', '\u{2588}'][mask as usize],
            Tier::Quadrant => QUADRANT[mask as usize],
            Tier::Sextant => sextant(mask),
            Tier::Braille => braille(mask),
        })
    }
}

/// Quadrant blocks indexed by mask: bit 0 top-left, 1 top-right, 2 bottom-left,
/// 3 bottom-right.
const QUADRANT: [char; 16] = [
    ' ', '\u{2598}', '\u{259D}', '\u{2580}', '\u{2596}', '\u{258C}', '\u{259E}', '\u{259B}',
    '\u{2597}', '\u{259A}', '\u{2590}', '\u{259C}', '\u{2584}', '\u{2599}', '\u{259F}', '\u{2588}',
];

/// Maps a 6-bit sextant mask to its codepoint.
///
/// The Unicode block runs `U+1FB00..=U+1FB3B` — sixty characters, not
/// sixty-four, because four patterns already exist as block elements: empty
/// (space), left column (`U+258C`), right column (`U+2590`) and full
/// (`U+2588`). Codepoints are assigned in mask order with those four skipped.
fn sextant(mask: u8) -> char {
    const EMPTY: u8 = 0b000_000;
    const LEFT: u8 = 0b010_101;
    const RIGHT: u8 = 0b101_010;
    const FULL: u8 = 0b111_111;
    match mask {
        EMPTY => ' ',
        LEFT => '\u{258C}',
        RIGHT => '\u{2590}',
        FULL => '\u{2588}',
        m => {
            let skipped = u32::from(m > LEFT) + u32::from(m > RIGHT);
            let idx = u32::from(m) - 1 - skipped;
            char::from_u32(0x1FB00 + idx).expect("sextant index is in range by construction")
        }
    }
}

/// Maps an 8-bit sub-cell mask to a braille pattern.
///
/// Braille dot numbering is not raster order: the left column top-to-bottom is
/// dots 1, 2, 3, 7 and the right column is 4, 5, 6, 8. This permutes the mask
/// before offsetting from `U+2800`.
fn braille(mask: u8) -> char {
    // Sub-cell index (row-major, 2 wide) to braille dot bit.
    const DOT: [u8; 8] = [0x01, 0x08, 0x02, 0x10, 0x04, 0x20, 0x40, 0x80];
    let mut bits = 0u8;
    for (i, dot) in DOT.iter().enumerate() {
        if mask & (1 << i) != 0 {
            bits |= dot;
        }
    }
    char::from_u32(0x2800 + u32::from(bits)).expect("braille offset is always in range")
}

/// One terminal cell: a glyph and the two colours it selects between.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// The glyph to emit.
    pub glyph: char,
    /// Colour of the set bits.
    pub fg: Srgb8,
    /// Colour of the clear bits.
    pub bg: Srgb8,
}

/// The sub-cell means of one cell, in row-major order from the top left.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    tier: Tier,
    means: Vec<Oklab>,
}

impl Patch {
    /// Builds a patch by sampling `f` at each sub-cell of `tier`.
    ///
    /// `f` receives sub-cell coordinates and returns that sub-cell's mean colour.
    pub fn from_fn(tier: Tier, mut f: impl FnMut(u8, u8) -> Oklab) -> Self {
        let (w, h) = tier.sub();
        let mut means = Vec::with_capacity(tier.samples());
        for y in 0..h {
            for x in 0..w {
                means.push(f(x, y));
            }
        }
        Self { tier, means }
    }

    /// Builds a patch from pre-computed sub-cell means in row-major order.
    ///
    /// # Errors
    ///
    /// Returns [`PatchError`] if `means` is not exactly [`Tier::samples`] long.
    pub fn from_means(tier: Tier, means: Vec<Oklab>) -> Result<Self, PatchError> {
        if means.len() == tier.samples() {
            Ok(Self { tier, means })
        } else {
            Err(PatchError { tier, got: means.len() })
        }
    }

    /// The tier this patch was sampled at.
    pub fn tier(&self) -> Tier {
        self.tier
    }

    /// The sub-cell means, row-major from the top left.
    pub fn means(&self) -> &[Oklab] {
        &self.means
    }
}

/// A patch was built with the wrong number of sub-cell samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatchError {
    /// The tier the patch was declared at.
    pub tier: Tier,
    /// How many samples were supplied.
    pub got: usize,
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (w, h) = self.tier.sub();
        write!(
            f,
            "{:?} needs {} sub-cell samples ({w}x{h}), got {}",
            self.tier,
            self.tier.samples(),
            self.got
        )
    }
}

impl std::error::Error for PatchError {}

/// Quantizes a patch into a single cell.
///
/// Runs two-means over the sub-cell colours in Oklab. The cluster assignment is
/// the glyph's bitmask; the two centroids are the cell's colours. The darker
/// centroid becomes the background, which keeps output stable frame to frame
/// instead of letting foreground and background swap on noise.
///
/// # Panics
///
/// Never, for a [`Patch`] built through its constructors: those guarantee the
/// sample count matches the tier, which bounds the mask to a valid glyph.
pub fn quantize(patch: &Patch) -> Cell {
    let tier = patch.tier;
    let means = patch.means();

    if tier == Tier::Ascii {
        let c = means[0].to_srgb8();
        return Cell { glyph: ' ', fg: c, bg: c };
    }

    let (bg, fg, mask) = two_means(means);
    let glyph = tier.glyph(mask).expect("mask is bounded by the tier's sample count");
    Cell { glyph, fg: fg.to_srgb8(), bg: bg.to_srgb8() }
}

/// Two-means clustering over sub-cell colours.
///
/// Returns `(background, foreground, mask)`, where a set bit in `mask` means
/// that sub-cell belongs to the foreground cluster. Seeded from the darkest and
/// lightest samples, which converges in a handful of passes for eight points.
pub fn two_means(means: &[Oklab]) -> (Oklab, Oklab, u8) {
    const ITERATIONS: usize = 4;
    debug_assert!(!means.is_empty() && means.len() <= 8);

    let mut dark = means[0];
    let mut light = means[0];
    for m in means {
        if m.l < dark.l {
            dark = *m;
        }
        if m.l > light.l {
            light = *m;
        }
    }

    let mut mask = 0u8;
    for _ in 0..ITERATIONS {
        mask = 0;
        let (mut ds, mut dn) = (Oklab::default(), 0u32);
        let (mut ls, mut ln) = (Oklab::default(), 0u32);
        for (i, m) in means.iter().enumerate() {
            if m.distance_sq(light) < m.distance_sq(dark) {
                mask |= 1 << i;
                ls = add(ls, *m);
                ln += 1;
            } else {
                ds = add(ds, *m);
                dn += 1;
            }
        }
        if dn > 0 {
            dark = scale(ds, 1.0 / dn as f32);
        }
        if ln > 0 {
            light = scale(ls, 1.0 / ln as f32);
        }
    }

    (dark, light, mask)
}

fn add(a: Oklab, b: Oklab) -> Oklab {
    Oklab::new(a.l + b.l, a.a + b.a, a.b + b.b)
}

fn scale(a: Oklab, k: f32) -> Oklab {
    Oklab::new(a.l * k, a.a * k, a.b * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(r: u8, g: u8, b: u8) -> Oklab {
        Oklab::from_srgb8(Srgb8::new(r, g, b))
    }

    #[test]
    fn tier_sample_counts() {
        assert_eq!(Tier::Ascii.samples(), 1);
        assert_eq!(Tier::Half.samples(), 2);
        assert_eq!(Tier::Quadrant.samples(), 4);
        assert_eq!(Tier::Sextant.samples(), 6);
        assert_eq!(Tier::Braille.samples(), 8);
    }

    #[test]
    fn half_and_quadrant_anchors() {
        assert_eq!(Tier::Half.glyph(0b01), Some('\u{2580}')); // upper half
        assert_eq!(Tier::Half.glyph(0b10), Some('\u{2584}')); // lower half
        assert_eq!(Tier::Quadrant.glyph(0b0001), Some('\u{2598}')); // upper left
        assert_eq!(Tier::Quadrant.glyph(0b0101), Some('\u{258C}')); // left half
        assert_eq!(Tier::Quadrant.glyph(0b1010), Some('\u{2590}')); // right half
        assert_eq!(Tier::Quadrant.glyph(0b1111), Some('\u{2588}')); // full
    }

    #[test]
    fn out_of_range_masks_are_rejected() {
        assert_eq!(Tier::Half.glyph(0b100), None);
        assert_eq!(Tier::Quadrant.glyph(0b1_0000), None);
        assert_eq!(Tier::Sextant.glyph(0b100_0000), None);
    }

    #[test]
    fn sextant_block_endpoints() {
        // U+1FB00 is the first sextant; U+1FB3B is the last.
        assert_eq!(Tier::Sextant.glyph(0b000_001), Some('\u{1FB00}'));
        assert_eq!(Tier::Sextant.glyph(0b111_110), Some('\u{1FB3B}'));
    }

    #[test]
    fn sextant_reuses_existing_block_elements() {
        assert_eq!(Tier::Sextant.glyph(0b000_000), Some(' '));
        assert_eq!(Tier::Sextant.glyph(0b010_101), Some('\u{258C}'));
        assert_eq!(Tier::Sextant.glyph(0b101_010), Some('\u{2590}'));
        assert_eq!(Tier::Sextant.glyph(0b111_111), Some('\u{2588}'));
    }

    #[test]
    fn sextant_mapping_is_a_bijection_over_the_block() {
        let mut seen = std::collections::BTreeSet::new();
        for m in 0u8..64 {
            seen.insert(Tier::Sextant.glyph(m).unwrap());
        }
        assert_eq!(seen.len(), 64, "every mask must map to a distinct glyph");
        let in_block = seen.iter().filter(|c| ('\u{1FB00}'..='\u{1FB3B}').contains(c)).count();
        assert_eq!(in_block, 60, "sixty of the sixty-four live in the sextant block");
    }

    #[test]
    fn braille_dot_numbering() {
        assert_eq!(Tier::Braille.glyph(0b0000_0000), Some('\u{2800}'));
        assert_eq!(Tier::Braille.glyph(0b1111_1111), Some('\u{28FF}'));
        assert_eq!(Tier::Braille.glyph(0b0000_0001), Some('\u{2801}')); // dot 1
        assert_eq!(Tier::Braille.glyph(0b0000_0010), Some('\u{2808}')); // dot 4
        assert_eq!(Tier::Braille.glyph(0b0100_0000), Some('\u{2840}')); // dot 7
        assert_eq!(Tier::Braille.glyph(0b1000_0000), Some('\u{2880}')); // dot 8
    }

    #[test]
    fn braille_mapping_is_a_bijection() {
        let mut seen = std::collections::BTreeSet::new();
        for m in 0u8..=255 {
            seen.insert(Tier::Braille.glyph(m).unwrap());
        }
        assert_eq!(seen.len(), 256);
    }

    #[test]
    fn flat_patch_quantizes_to_one_colour() {
        let grey = ok(128, 128, 128);
        let patch = Patch::from_fn(Tier::Quadrant, |_, _| grey);
        let cell = quantize(&patch);
        assert_eq!(cell.fg, cell.bg, "a flat patch has nothing to separate");
    }

    #[test]
    fn split_patch_finds_the_edge() {
        // Left column dark, right column light: expect the left-half glyph or
        // its complement, and two clearly different colours.
        let patch =
            Patch::from_fn(
                Tier::Quadrant,
                |x, _| {
                    if x == 0 { ok(10, 10, 14) } else { ok(240, 240, 250) }
                },
            );
        let cell = quantize(&patch);
        assert_eq!(cell.glyph, '\u{2590}', "right half should be the set bits");
        assert!(cell.bg.r < 40 && cell.fg.r > 200);
    }

    #[test]
    fn darker_cluster_is_always_the_background() {
        for tier in Tier::ALL.into_iter().filter(|t| *t != Tier::Ascii) {
            let patch =
                Patch::from_fn(tier, |_, y| if y == 0 { ok(250, 250, 250) } else { ok(5, 5, 5) });
            let cell = quantize(&patch);
            let lum = |c: Srgb8| u32::from(c.r) + u32::from(c.g) + u32::from(c.b);
            assert!(lum(cell.bg) < lum(cell.fg), "{tier:?} put the light cluster in bg");
        }
    }

    #[test]
    fn ascii_tier_is_background_only() {
        let patch = Patch::from_fn(Tier::Ascii, |_, _| ok(196, 24, 122));
        let cell = quantize(&patch);
        assert_eq!(cell.glyph, ' ');
        assert_eq!(cell.bg, Srgb8::new(196, 24, 122));
    }

    #[test]
    fn patch_rejects_wrong_sample_count() {
        let err = Patch::from_means(Tier::Sextant, vec![Oklab::default(); 4]).unwrap_err();
        assert_eq!(err.got, 4);
        assert!(err.to_string().contains("2x3"));
    }
}
