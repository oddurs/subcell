---
id: 4
title: Generate the octant glyph table from Unicode data
type: feature
status: planned
milestone: v0.1
created: 2026-09-10
updated: 2026-09-10
priority: p0
area: subcell-blit
effort: m
---

## Problem

Octants (Unicode 16.0, around `U+1CD00`) are the intended top glyph tier: 2x4
like braille, but solid fills with independent fg/bg, and synthesized by kitty,
Ghostty and foot so they are pixel-exact at any font size.

The block omits mask patterns that existing block elements already cover. That
exclusion set is larger than the sextant case and does not follow from the same
rule, so a hand-written table is exactly the kind of change that ships a subtly
wrong mapping and passes review.

## Proposal

Check in a generator that reads Unicode character data and emits the
mask-to-codepoint table. Assert independently verified anchors in tests, plus
the bijection property already used for sextants and braille.

## Acceptance criteria

- [ ] Generator is checked in and reproducible offline
- [ ] `Tier::Octant` exists with `sub() == (2, 4)`
- [ ] Block endpoints asserted against Unicode character data
- [ ] Mapping is a bijection over all 256 masks
- [ ] `docs/blitters.md` octant section updated to describe what shipped
