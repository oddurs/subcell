---
id: 9
title: Benchmark the quantizer to sub-millisecond at 200x60
type: chore
status: backlog
milestone: v0.1
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: subcell-blit
effort: s
---

## Problem

Quantization runs per cell per frame. At 200x60 that is 12,000 cells; if it is
not comfortably sub-millisecond the tiered design is not viable at interactive
frame rates.

## Proposal

Criterion benchmarks per tier. Establish the number before optimising anything,
so later work has a baseline to argue against.

## Acceptance criteria

- [ ] Benchmarks for every tier
- [ ] 200x60 full-frame quantization under 1ms on a laptop core
- [ ] Result recorded in `docs/blitters.md`
