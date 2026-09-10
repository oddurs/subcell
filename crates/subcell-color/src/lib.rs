//! Perceptual colour for terminal rendering.
//!
//! Sub-cell rendering spends colour precision to buy spatial precision, so every
//! blend and every clustering decision has to happen in a space where distance
//! means what the eye thinks it means. sRGB is not that space: it is gamma
//! encoded, so the arithmetic midpoint of two colours is not the perceptual
//! midpoint. Halfway between black and white in sRGB is `#808080`, which the eye
//! reads as about 60% of the way up, not 50%. Antialiasing on that basis puts
//! edges in the wrong place. This crate provides [`Oklab`] and the operations
//! the rasterizers need on top of it.

/// An 8-bit sRGB colour, the form a terminal actually accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Srgb8 {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Srgb8 {
    /// Constructs a colour from its three 8-bit channels.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// A colour in the Oklab perceptual space.
///
/// Euclidean distance in this space approximates perceived colour difference,
/// which is what makes it the right space for both [`Oklab::lerp`] and the
/// two-means clustering in `subcell-blit`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Oklab {
    /// Perceptual lightness, roughly 0.0 to 1.0.
    pub l: f32,
    /// Green-to-red axis.
    pub a: f32,
    /// Blue-to-yellow axis.
    pub b: f32,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

impl Oklab {
    /// Constructs a colour from its three axes.
    pub const fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }

    /// Converts from 8-bit sRGB.
    pub fn from_srgb8(c: Srgb8) -> Self {
        Self::from_linear(
            srgb_to_linear(f32::from(c.r) / 255.0),
            srgb_to_linear(f32::from(c.g) / 255.0),
            srgb_to_linear(f32::from(c.b) / 255.0),
        )
    }

    /// Converts from linear-light RGB channels in `0.0..=1.0`.
    // `l`/`m`/`s` and `r`/`g`/`b` are the reference names for these channels;
    // spelling them out would make the transform harder to check.
    #[allow(clippy::many_single_char_names)]
    pub fn from_linear(r: f32, g: f32, b: f32) -> Self {
        let l = 0.412_221_5 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
        let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
        Self {
            l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        }
    }

    /// Converts back to linear-light RGB channels, unclamped.
    #[allow(clippy::many_single_char_names)]
    pub fn to_linear(self) -> (f32, f32, f32) {
        let l = self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b;
        let m = self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b;
        let s = self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b;
        let (l, m, s) = (l * l * l, m * m * m, s * s * s);
        (
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        )
    }

    /// Converts back to 8-bit sRGB, clamping out-of-gamut results.
    pub fn to_srgb8(self) -> Srgb8 {
        let (r, g, b) = self.to_linear();
        let q = |c: f32| (linear_to_srgb(c).clamp(0.0, 1.0) * 255.0).round() as u8;
        Srgb8::new(q(r), q(g), q(b))
    }

    /// Blends perceptually toward `other`, where `t == 0.0` is `self`.
    ///
    /// This is the primitive behind coverage antialiasing: a feature covering
    /// 40% of a cell is drawn by blending 40% of the way from ground to ink,
    /// which the eye integrates as a sub-cell shift in position.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            l: self.l + (other.l - self.l) * t,
            a: self.a + (other.a - self.a) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }

    /// Squared Euclidean distance, for comparisons that never need the root.
    pub fn distance_sq(self, other: Self) -> f32 {
        let (dl, da, db) = (self.l - other.l, self.a - other.a, self.b - other.b);
        dl * dl + da * da + db * db
    }
}

impl From<Srgb8> for Oklab {
    fn from(c: Srgb8) -> Self {
        Self::from_srgb8(c)
    }
}

impl From<Oklab> for Srgb8 {
    fn from(c: Oklab) -> Self {
        c.to_srgb8()
    }
}

#[cfg(test)]
mod tests {
    // Several tests assert that endpoints and conserved quantities are
    // *exactly* right; that exactness is the property under test.
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn white_is_lightness_one() {
        let w = Oklab::from_srgb8(Srgb8::new(255, 255, 255));
        assert!((w.l - 1.0).abs() < 1e-3, "l was {}", w.l);
        assert!(w.a.abs() < 1e-3 && w.b.abs() < 1e-3);
    }

    #[test]
    fn black_is_origin() {
        let k = Oklab::from_srgb8(Srgb8::new(0, 0, 0));
        assert!(k.l.abs() < 1e-4 && k.a.abs() < 1e-4 && k.b.abs() < 1e-4);
    }

    #[test]
    fn round_trips_through_oklab() {
        for c in [
            Srgb8::new(0, 0, 0),
            Srgb8::new(255, 255, 255),
            Srgb8::new(196, 24, 122),
            Srgb8::new(10, 115, 134),
            Srgb8::new(17, 19, 30),
        ] {
            let back = Oklab::from_srgb8(c).to_srgb8();
            assert_eq!(c, back, "round trip changed {c:?}");
        }
    }

    #[test]
    fn lerp_endpoints_are_exact() {
        let a = Oklab::from_srgb8(Srgb8::new(196, 24, 122));
        let b = Oklab::from_srgb8(Srgb8::new(10, 115, 134));
        assert_eq!(a.lerp(b, 0.0).to_srgb8(), a.to_srgb8());
        assert_eq!(a.lerp(b, 1.0).to_srgb8(), b.to_srgb8());
    }

    #[test]
    fn midpoint_blend_is_perceptually_centred() {
        // The defect Oklab exists to fix: sRGB is gamma encoded, so the
        // arithmetic midpoint of black and white sits well above perceptual
        // middle grey. Coverage antialiasing built on that puts edges in the
        // wrong place, which is exactly what makes terminal curves look wrong.
        let black = Oklab::from_srgb8(Srgb8::new(0, 0, 0));
        let white = Oklab::from_srgb8(Srgb8::new(255, 255, 255));
        let perceptual = black.lerp(white, 0.5);
        let naive = Oklab::from_srgb8(Srgb8::new(128, 128, 128));

        assert!((perceptual.l - 0.5).abs() < 1e-3, "perceptual midpoint L was {}", perceptual.l);
        assert!(naive.l > 0.57, "sRGB midpoint should read too light, L was {}", naive.l);
        assert!(
            (perceptual.l - 0.5).abs() < (naive.l - 0.5).abs(),
            "Oklab must be closer to perceptual centre"
        );
    }

    #[test]
    fn coverage_blending_is_monotone_and_bounded() {
        let ground = Oklab::from_srgb8(Srgb8::new(14, 16, 24));
        let ink = Oklab::from_srgb8(Srgb8::new(255, 79, 163));
        let mut last = f32::NEG_INFINITY;
        for step in 0..=20 {
            let t = step as f32 / 20.0;
            let l = ground.lerp(ink, t).l;
            assert!(l >= last - 1e-6, "lightness fell from {last} to {l} at t={t}");
            assert!(l >= ground.l - 1e-4 && l <= ink.l + 1e-4, "left the endpoints at t={t}");
            last = l;
        }
    }

    #[test]
    fn distance_is_zero_for_identical_colours() {
        let a = Oklab::from_srgb8(Srgb8::new(40, 80, 120));
        assert!(a.distance_sq(a).abs() < 1e-9);
    }
}
