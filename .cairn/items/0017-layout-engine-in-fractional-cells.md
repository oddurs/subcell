---
id: 17
title: Layout engine in fractional cells
type: feature
status: backlog
milestone: v0.3
created: 2026-09-10
updated: 2026-09-10
priority: p0
area: subcell
effort: l
---

## Problem

Every TUI layout engine rounds to whole cells, which throws away exactly the
precision this renderer exists to preserve. A split at 50% of an odd width
should be allowed to land mid-cell.

## Acceptance criteria

- [ ] Constraints resolve in `f32` cells
- [ ] A 50% split of an odd width lands mid-cell and renders as such
- [ ] Rounding happens once, in the rasterizer
