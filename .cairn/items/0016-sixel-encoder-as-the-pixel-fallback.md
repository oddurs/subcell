---
id: 16
title: Sixel encoder as the pixel fallback
type: feature
status: backlog
milestone: v0.2
created: 2026-09-10
updated: 2026-09-10
priority: p2
area: subcell-px
effort: m
---

## Problem

Sixel is worse than kitty graphics in every way that matters here — no z-order,
palette-limited, mangled by scrollback — but it is more widely supported, and
worse still beats nothing.

## Acceptance criteria

- [ ] Palette quantization in Oklab, reusing `subcell-color`
- [ ] Respects the palette size the terminal advertises
- [ ] Documented as unable to support virtual placement
