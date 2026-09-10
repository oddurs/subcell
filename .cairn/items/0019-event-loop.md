---
id: 19
title: Event loop
type: feature
status: backlog
milestone: v0.3
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: subcell
effort: m
---

## Problem

Nothing ties input, layout and rendering into something a program can run.

## Acceptance criteria

- [ ] Input, resize and redraw in one loop
- [ ] Resize re-probes the cell size
- [ ] Terminal restored on drop, on error and on panic
