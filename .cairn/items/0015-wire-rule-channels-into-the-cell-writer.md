---
id: 15
title: Wire rule channels into the cell writer
type: feature
status: backlog
milestone: v0.2
created: 2026-09-10
updated: 2026-09-10
priority: p2
area: subcell-chan
effort: s
---

## Problem

`subcell-chan` builds SGR sequences but nothing emits them as part of a frame.

## Acceptance criteria

- [ ] Cells carry optional rules through the writer
- [ ] Rules degrade per capability, independently
- [ ] Reset emits only what was set
- [ ] An example draws a signal through a row of text
