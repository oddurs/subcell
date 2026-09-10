---
id: 10
title: Terminal writer with damage tracking and synchronized output
type: feature
status: backlog
milestone: v0.1
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: subcell
effort: l
---

## Problem

Rasterizing to a cell grid is useless without something that writes the diff to
a terminal without tearing.

## Proposal

Double-buffered cell grid, emit only changed runs, wrap frames in `DECSET 2026`
where supported. Coalesce SGR changes across runs rather than re-emitting per
cell.

## Acceptance criteria

- [ ] Only changed cells are written
- [ ] Frames are wrapped in synchronized output when available
- [ ] SGR state is tracked, not re-emitted per cell
- [ ] Terminal is restored on drop and on panic
