# Coverage

## Antialias with colour, not with space

You have 16.7 million colours and roughly 136 pixels per cell. You are
colour-rich and space-poor, so trade accordingly.

To draw a vertical rule at `x = 3.4`, you do not need a glyph that starts 40%
into the cell. You draw at cell 3 with a foreground blended 40% toward the ink.
The eye integrates intensity and reads a shifted edge.

This is LCD subpixel rendering applied to the cell grid, and it is the single
cheapest quality win available. For features narrower than a cell, perceived
position is dominated by intensity, not by which cell you landed in.

## Why the scene graph is `f32`

Because this only works if the geometry never rounded in the first place. A
scene that stores `x: u16` has already thrown away the information coverage
needs. Fractional cells go all the way down, and the single rounding step
happens in the rasterizer.

```rust
// 0.2 cells wide, centred on 3.4: spans 3.3..3.5, entirely inside cell 3.
let line = Rect::new(3.3, 0.0, 0.2, 1.0);
rect_coverage(&line, &Rect::new(3.0, 0.0, 1.0, 1.0));  // 0.2
rect_coverage(&line, &Rect::new(4.0, 0.0, 1.0, 1.0));  // 0.0
```

Two properties are asserted in tests because everything else depends on them:

- **Conservation.** A feature straddling a boundary splits its coverage between
  the two cells and the total is unchanged.
- **Monotonicity.** As geometry slides right, the left cell never gains
  coverage. Without this, tonal positioning judders instead of gliding.

Coverage is computed analytically from geometry, not by supersampling, so cost
does not scale with a sample count.

## Blend in Oklab, not sRGB

sRGB is gamma encoded, so the arithmetic midpoint of two colours is not the
perceptual midpoint. Halfway between black and white is `#808080`, which reads
as roughly 60% of the way up rather than 50%.

That 10% error is not cosmetic. Coverage antialiasing *is* a position encoding —
if the intensity is wrong, the apparent position of the edge is wrong, and a
curve rendered this way wobbles against its own geometry.

Oklab puts the midpoint where the eye expects it. `subcell-color` asserts this
directly: `lerp(black, white, 0.5)` has lightness 0.5, while the sRGB midpoint
measures above 0.57.

A note on a claim that gets repeated and is not quite right: blending two
saturated hues in Oklab does not automatically preserve more chroma than the
sRGB midpoint — for some pairs it preserves slightly less. The defect Oklab
reliably fixes here is **lightness**, and that is the one this crate tests for.

## Where this pays off

A `subcell` sparkline should look categorically better than a naive one at the
*same* glyph tier, with no extra resolution at all. That is the check for
whether this layer is earning its place.
