---
id: 6
title: Rasterize a scene into cells at any glyph tier
type: feature
status: planned
milestone: v0.1
depends_on:
- 5
created: 2026-09-10
updated: 2026-09-10
priority: p0
area: subcell-scene
effort: l
---

## Problem

The scene graph and the quantizers exist but nothing connects them. This is the
join that makes the central claim testable.

## Proposal

Walk the primitives, sample sub-cell coverage into a patch buffer, quantize per
cell. The tier is a parameter throughout; no code path may branch on a specific
tier outside the blitter.

## Acceptance criteria

- [ ] `Scene::rasterize(tier)` produces a cell grid
- [ ] Identical scene renders at every tier without special-casing
- [ ] Damage bounds limit work to changed cells
- [ ] Property test: rasterizing twice is byte-identical
