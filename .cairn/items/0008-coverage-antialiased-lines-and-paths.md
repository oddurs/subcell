---
id: 8
title: Coverage-antialiased lines and paths
type: feature
status: backlog
milestone: v0.1
depends_on:
- 6
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: subcell-cov
effort: m
---

## Problem

`subcell-cov` computes coverage for rectangles. Lines and curves are where tonal
positioning actually pays off, and they are what a chart is made of.

## Proposal

Analytic coverage for a stroked segment against a sub-cell, then paths as
sequences of segments. No supersampling.

## Acceptance criteria

- [ ] Sub-cell-width strokes render as tonal shifts, not stepped blocks
- [ ] Conservation and monotonicity properties hold, as for rectangles
- [ ] Re-render the tier comparison and confirm middle tiers improve visibly
