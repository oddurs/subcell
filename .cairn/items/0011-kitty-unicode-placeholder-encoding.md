---
id: 11
title: Kitty Unicode placeholder encoding
type: feature
status: backlog
milestone: v0.2
depends_on:
- 10
created: 2026-09-10
updated: 2026-09-10
priority: p0
area: subcell-px
effort: m
---

## Problem

Virtual placement is the only reason to prefer kitty graphics over sixel: the
image is placed through the grid, so layout and damage tracking keep working.
`subcell-px` builds the APC commands but cannot yet emit placeholder cells.

Row and column are encoded in combining diacritics from a specific list. Fiddly
enough that guessing produces a subtly broken placement.

## Acceptance criteria

- [ ] Placeholder cells encode image id, row and column correctly
- [ ] Verified against a real kitty and a real Ghostty
- [ ] Images clip and scroll with the pane that contains them
- [ ] Damage tracker needs no special case for image cells
