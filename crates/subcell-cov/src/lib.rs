//! Analytic coverage rasterization.
//!
//! The cheapest resolution win available in a terminal is to stop treating
//! position as discrete. A vertical rule at `x = 3.4` does not need a glyph that
//! starts 40% into the cell — it needs cell 3 painted with a colour blended 40%
//! of the way toward the ink. The eye integrates intensity and reads a shifted
//! edge. That is all this crate does, and it is why a chart drawn through
//! `subcell` looks smooth at a tier where a naive renderer looks stepped.
//!
//! Coverage is computed analytically from geometry rather than by supersampling,
//! so cost does not scale with the sample count.

use subcell_color::{Oklab, Srgb8};
use subcell_scene::Rect;

/// Fraction of `area` covered by `shape`, in `0.0..=1.0`.
///
/// Both rectangles are in fractional cell space. An `area` with no extent has no
/// coverage rather than dividing by zero.
pub fn rect_coverage(shape: &Rect, area: &Rect) -> f32 {
    if area.is_empty() {
        return 0.0;
    }
    let hit = shape.intersect(area);
    if hit.is_empty() { 0.0 } else { (hit.w * hit.h) / (area.w * area.h) }
}

/// Coverage of a horizontal span over one sub-cell column.
///
/// A convenience for the common case of a rule or an axis: the shape is
/// unbounded vertically, so only the horizontal overlap matters.
pub fn span_coverage(from: f32, to: f32, cell_from: f32, cell_to: f32) -> f32 {
    let width = cell_to - cell_from;
    if width <= 0.0 {
        return 0.0;
    }
    let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
    let overlap = hi.min(cell_to) - lo.max(cell_from);
    (overlap / width).clamp(0.0, 1.0)
}

/// Blends ink over ground by coverage, perceptually.
///
/// Blending in sRGB is what makes antialiased terminal output look dirty; this
/// goes through Oklab so a half-covered edge keeps its chroma.
pub fn shade(ground: Srgb8, ink: Srgb8, coverage: f32) -> Srgb8 {
    let t = coverage.clamp(0.0, 1.0);
    Oklab::from_srgb8(ground).lerp(Oklab::from_srgb8(ink), t).to_srgb8()
}

/// The apparent sub-cell position a coverage value represents.
///
/// The inverse of [`shade`] in geometric terms: given how much of a cell a thin
/// feature covers, where does the eye place it? Used to verify that tonal
/// positioning tracks the geometry it came from.
pub fn apparent_offset(coverage: f32) -> f32 {
    coverage.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    // Several tests assert that endpoints and conserved quantities are
    // *exactly* right; that exactness is the property under test.
    #![allow(clippy::float_cmp)]

    use super::*;

    const GROUND: Srgb8 = Srgb8::new(14, 16, 24);
    const INK: Srgb8 = Srgb8::new(255, 79, 163);

    #[test]
    fn full_and_zero_coverage() {
        let cell = Rect::new(3.0, 0.0, 1.0, 1.0);
        assert_eq!(rect_coverage(&Rect::new(0.0, 0.0, 10.0, 10.0), &cell), 1.0);
        assert_eq!(rect_coverage(&Rect::new(8.0, 8.0, 1.0, 1.0), &cell), 0.0);
    }

    #[test]
    fn partial_coverage_is_the_area_ratio() {
        let cell = Rect::new(3.0, 0.0, 1.0, 1.0);
        let shape = Rect::new(3.0, 0.0, 0.4, 1.0);
        assert!((rect_coverage(&shape, &cell) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn a_hairline_at_three_point_four_lands_in_cell_three() {
        // 0.2 cells wide, centred on 3.4: spans 3.3..3.5, entirely in cell 3.
        let line = Rect::new(3.3, 0.0, 0.2, 1.0);
        assert!((rect_coverage(&line, &Rect::new(3.0, 0.0, 1.0, 1.0)) - 0.2).abs() < 1e-6);
        assert_eq!(rect_coverage(&line, &Rect::new(4.0, 0.0, 1.0, 1.0)), 0.0);
    }

    #[test]
    fn a_hairline_on_a_boundary_splits_between_cells() {
        let line = Rect::new(3.9, 0.0, 0.2, 1.0);
        let left = rect_coverage(&line, &Rect::new(3.0, 0.0, 1.0, 1.0));
        let right = rect_coverage(&line, &Rect::new(4.0, 0.0, 1.0, 1.0));
        assert!((left - 0.1).abs() < 1e-6, "left was {left}");
        assert!((right - 0.1).abs() < 1e-6, "right was {right}");
        assert!((left + right - 0.2).abs() < 1e-6, "coverage must be conserved");
    }

    #[test]
    fn sweeping_a_line_moves_coverage_monotonically() {
        // The property that makes tonal positioning work: as geometry slides
        // right, the left cell must never gain coverage.
        let cell = Rect::new(3.0, 0.0, 1.0, 1.0);
        let mut last = f32::INFINITY;
        for step in 0..=20 {
            let x = 3.0 + step as f32 * 0.05;
            let c = rect_coverage(&Rect::new(x, 0.0, 0.5, 1.0), &cell);
            assert!(c <= last + 1e-6, "coverage rose from {last} to {c} at x={x}");
            last = c;
        }
    }

    #[test]
    fn span_coverage_matches_rect_coverage() {
        let s = span_coverage(3.3, 3.5, 3.0, 4.0);
        let r = rect_coverage(&Rect::new(3.3, 0.0, 0.2, 1.0), &Rect::new(3.0, 0.0, 1.0, 1.0));
        assert!((s - r).abs() < 1e-6);
    }

    #[test]
    fn span_coverage_handles_reversed_input() {
        assert!((span_coverage(3.5, 3.3, 3.0, 4.0) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn zero_width_cell_has_no_coverage() {
        assert_eq!(span_coverage(0.0, 1.0, 2.0, 2.0), 0.0);
    }

    #[test]
    fn shade_endpoints_are_exact() {
        assert_eq!(shade(GROUND, INK, 0.0), GROUND);
        assert_eq!(shade(GROUND, INK, 1.0), INK);
    }

    #[test]
    fn shade_clamps_out_of_range_coverage() {
        assert_eq!(shade(GROUND, INK, -1.0), GROUND);
        assert_eq!(shade(GROUND, INK, 2.0), INK);
    }

    #[test]
    fn shade_is_monotone_in_lightness() {
        let mut last = -1.0;
        for step in 0..=10 {
            let c = shade(GROUND, INK, step as f32 / 10.0);
            let l = Oklab::from_srgb8(c).l;
            assert!(l >= last - 1e-4, "lightness fell from {last} to {l}");
            last = l;
        }
    }

    #[test]
    fn apparent_offset_tracks_coverage() {
        assert!((apparent_offset(0.4) - 0.4).abs() < 1e-6);
        assert_eq!(apparent_offset(1.5), 1.0);
    }
}
