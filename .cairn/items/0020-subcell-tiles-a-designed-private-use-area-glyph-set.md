---
id: 20
title: 'subcell-tiles: a designed private-use-area glyph set'
type: feature
status: backlog
created: 2026-09-10
updated: 2026-09-10
priority: p3
area: subcell-tiles
effort: xl
---

## Problem

Unicode gave us the block glyphs it happened to standardise. Diagonals are
staircases, panels cannot be rounded, and a border can never land between cells.

## Proposal

Design the tile set we actually want in the Private Use Area: 45-degree wedges
at eight phase offsets, quarter-arcs at four radii, anti-aliased thirds, and
half-cell-offset box drawing. OpenType contextual alternates let a *run* of
characters substitute into a single wide vector shape — the Fira Code mechanism,
pointed at drawing instead of at `!=`.

Nerd Fonts proved the distribution channel exists: millions of people will
install a font to make their terminal better.

## Why this is not scheduled

It asks users to install something, so the rest of the renderer has to be worth
it first. Opt-in, detected through `Caps`, degrades to octants when absent.

## Acceptance criteria

- [ ] Glyph set designed and justified against what octants cannot express
- [ ] Detected rather than assumed
- [ ] Degrades to octants with no visible failure
