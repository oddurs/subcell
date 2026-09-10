# Blitters

## Sub-cell rendering is texture compression

The naive implementation of a glyph tier scores every candidate glyph against a
cell — 64 masks for sextants, 256 for braille. Don't.

Choosing a glyph, a foreground and a background to approximate a patch of colour
is **block truncation coding**: the same problem BC1/S3TC solves for texture
blocks. Average each sub-cell, cluster the samples into two groups, and the
cluster assignment *is* the bitmask. The glyph falls out for free.

```rust
let s = patch.means();                    // sub-cell means, in Oklab
let (bg, fg, mask) = two_means(s);        // O(sub-cells), not O(glyphs)
let glyph = tier.glyph(mask).unwrap();
```

Clustering happens in Oklab, not sRGB — see [coverage.md](coverage.md) for why
that matters.

One stability detail worth keeping: the **darker centroid always becomes the
background**. Without that rule, foreground and background swap between frames
on nearly-tied clusters, and a static image visibly shimmers.

## The tiers

| Tier | Sub-cells | Glyphs | 120x40 becomes |
|---|---|---|---|
| Ascii | 1x1 | space + background colour | 120 x 40 |
| Half | 1x2 | two block elements | 120 x 80 |
| Quadrant | 2x2 | sixteen block elements | 240 x 80 |
| Sextant | 2x3 | `U+1FB00..U+1FB3B` + 4 reused | 240 x 120 |
| Braille | 2x4 | `U+2800..U+28FF` | 240 x 160 |

Masks are row-major from the top left, so bit 0 is always the top-left
sub-cell.

## Sextants: sixty, not sixty-four

The Unicode block runs `U+1FB00..=U+1FB3B` — sixty characters. Four of the
sixty-four possible patterns already exist as block elements and were not
duplicated:

| Mask | Character |
|---|---|
| `000000` | space |
| `010101` | `U+258C` left half |
| `101010` | `U+2590` right half |
| `111111` | `U+2588` full |

Codepoints are assigned in mask order with those four skipped, so the offset for
a given mask has to subtract the exclusions **cumulatively**. Getting this wrong
shifts every glyph past the first exclusion by one codepoint, and it will not
show up as a test failure unless the test pins independently verified anchors —
which is why `subcell-blit` asserts both endpoints of the block and that the
mapping is a bijection.

## Braille: not raster order

Braille dot numbering predates raster graphics. The left column top to bottom is
dots 1, 2, 3, 7; the right column is 4, 5, 6, 8. The mask must be permuted
before offsetting from `U+2800`:

```
sub-cell (col, row)   dot   bit
    (0, 0)             1    0x01
    (0, 1)             2    0x02
    (0, 2)             3    0x04
    (0, 3)             7    0x40
    (1, 0)             4    0x08
    (1, 1)             5    0x10
    (1, 2)             6    0x20
    (1, 3)             8    0x80
```

Braille carries 2x4 detail but only one colour: the "off" state is the cell
background, not a second foreground. It also renders as visually separated dots,
with spacing that varies by font. It is the right choice for line art and the
wrong choice for filled graphics.

## Octants: the missing tier

Unicode 16.0 (September 2024) added octant blocks around `U+1CD00`. They are
2x4 like braille but **solid fills with independent foreground and background**,
so no dot gaps and colour composites properly. kitty, Ghostty and foot
*synthesize* them programmatically rather than pulling from the font, which
makes them seam-free and pixel-exact at any font size.

This is the intended top glyph tier and it is **not implemented yet**, on
purpose. Like sextants, the block omits patterns that existing block elements
already cover, but the exclusion set is larger and does not follow from the same
rule as the sextant case. Hand-writing that table is exactly the kind of change
that produces a subtly wrong mapping which passes review.

The table must be **generated from Unicode character data**, with the generator
checked in and the anchors asserted in tests. Tracked as the first `v0.1` item.

## Detection is the weak point

There is no escape sequence that asks "do you render octants?". Support has to
be inferred from the terminal identity string, which is genuinely ugly and will
occasionally be wrong. Two consequences are designed in:

1. The conservative default is a tier lower than the guess.
2. There is a manual override, because a user staring at tofu should not have to
   wait for a release.

## Prior art

[chafa](https://hpjansson.org/chafa/) is the state of the art in glyph
quantization — read its scoring before changing anything here.
[notcurses](https://github.com/dankamongmen/notcurses) pioneered the tiered
blitter architecture in C.
