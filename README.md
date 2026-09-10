# subcell

[![CI](https://github.com/oddurs/subcell/actions/workflows/ci.yml/badge.svg)](https://github.com/oddurs/subcell/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A terminal renderer that describes the screen in continuous coordinates and
quantizes down to whatever the terminal can actually draw.

A terminal cell offers 16.7 million colours and roughly 136 pixels. That ratio
is the whole design space: you are colour-rich and space-poor, so you spend
colour precision to buy back spatial precision. `subcell` is built on one rule —
**resolution is a runtime property.** Ask the terminal what it can do, then pick
a renderer. Nothing is allowed to assume a tier.

```
 tier        sub-cells   120x40 window becomes
 ---------   ---------   ---------------------
 cell            1x1     120 x  40
 half            1x2     120 x  80
 quadrant        2x2     240 x  80
 sextant         2x3     240 x 120
 braille         2x4     240 x 160
 pixel          8x17     960 x 680     (kitty graphics / sixel)
```

## Status

Pre-release, `0.0.1`. This is a young repository and the honest summary is that
the pure, testable half is built and the terminal I/O half is not.

**Working today**

- `subcell-color` — Oklab conversion, perceptual blending, round-trip exact.
- `subcell-blit` — two-colour block quantization via two-means in Oklab, with
  verified glyph tables for half, quadrant, sextant and braille tiers.
- `subcell-cov` — analytic coverage, so a hairline at `x = 3.4` renders as a
  tonal shift rather than snapping to a cell boundary.
- `subcell-scene` — fractional-cell geometry and whole-cell damage bounds.
- `subcell-caps` — the capability vocabulary and XTWINOPS reply parsing.
- `subcell-chan` — underline, overline and strikethrough SGR construction with
  independent degradation.
- `subcell-px` — kitty graphics APC command construction.
- `subcell-in` — pixel-to-fractional-cell pointer mapping.

**Not built yet** — see [the roadmap](docs/roadmap.md)

- Driving a real terminal: raw mode, writing probe queries, reading replies.
  Everything in `subcell-caps` is parsing and policy; nothing does I/O.
- Octant glyphs (Unicode 16, `U+1CD00`). The mask-to-codepoint mapping skips
  patterns that existing block elements already cover, and that table has to be
  generated from Unicode character data rather than typed in by hand.
- Kitty Unicode placeholder encoding, and shared-memory image transfer.
- Widgets, layout, and an event loop.

## Install

```sh
cargo add subcell
```

Or use a single layer directly — the crates are designed to be useful on their
own, without adopting the framework:

```sh
cargo add subcell-blit    # just the quantizers
cargo add subcell-caps    # just the capability vocabulary
```

## Quickstart

Report what `subcell` would render with in the current terminal:

```sh
cargo run --bin subcell-probe
```

Pick a renderer from a capability set:

```rust
use subcell::{Plan, caps::Caps};

// With nothing detected, there is still a safe floor.
let plan = Plan::for_caps(&Caps::conservative());
assert_eq!(plan.tier, subcell::blit::Tier::Half);
assert!(!plan.pixel_tier);
```

Quantize a patch of colour into one cell:

```rust
use subcell::blit::{Patch, Tier, quantize};
use subcell::color::{Oklab, Srgb8};

let ground = Oklab::from_srgb8(Srgb8::new(14, 16, 24));
let ink = Oklab::from_srgb8(Srgb8::new(255, 79, 163));

// Left column dark, right column light.
let patch = Patch::from_fn(Tier::Quadrant, |x, _| if x == 0 { ground } else { ink });
let cell = quantize(&patch);

assert_eq!(cell.glyph, '▐');
```

## Documentation

| Document | What it covers |
|---|---|
| [`docs/architecture.md`](docs/architecture.md) | How the nine crates fit together, and why the seams sit where they do |
| [`docs/blitters.md`](docs/blitters.md) | Sub-cell quantization, the glyph tables, and the octant problem |
| [`docs/capabilities.md`](docs/capabilities.md) | The probe, terminal support, and how degradation is decided |
| [`docs/coverage.md`](docs/coverage.md) | Antialiasing with colour instead of space |
| [`docs/channels.md`](docs/channels.md) | Underline, overline and strikethrough as drawing channels |
| [`docs/pixel-tier.md`](docs/pixel-tier.md) | Kitty graphics, virtual placement, and why sixel is the fallback |
| [`docs/input.md`](docs/input.md) | Pixel mouse and the kitty keyboard protocol |
| [`docs/roadmap.md`](docs/roadmap.md) | Generated from the tracked backlog |

## Development

```sh
scripts/setup           # wire git hooks, once after cloning
scripts/agent doctor    # check the environment
scripts/task check      # format, lint, test, build
```

All automation goes through `scripts/task`, so CI, the git hooks and
`scripts/agent` can never drift from what you run by hand.

Work happens one branch to a worktree:

```sh
scripts/agent start feat/octant-table
cd ../.worktrees/subcell/feat-octant-table
# ... edit, then:
scripts/agent commit "feat(blit): add the octant glyph table"
scripts/agent pr
```

`main` only ever advances through a merged pull request. See
[CONTRIBUTING.md](CONTRIBUTING.md).

The backlog lives in the repository as Markdown, managed with
[cairn](https://github.com/oddurs/cairn):

```sh
cairn next      # what is ready to work on
cairn board     # kanban view
cairn roadmap   # milestone progress
```

## Prior art

[chafa](https://hpjansson.org/chafa/) is the state of the art in glyph
quantization and worth reading before touching `subcell-blit`.
[notcurses](https://github.com/dankamongmen/notcurses) pioneered capability-
tiered "blitters" in C. [ratatui](https://ratatui.rs) is the Rust TUI ecosystem
`subcell` expects to interoperate with rather than replace.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in this work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any additional
terms or conditions.
