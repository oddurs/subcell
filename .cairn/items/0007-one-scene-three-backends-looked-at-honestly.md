---
id: 7
title: One scene, three backends, looked at honestly
type: feature
status: backlog
milestone: v0.1
depends_on:
- 4
- 6
- 10
created: 2026-09-10
updated: 2026-09-10
priority: p0
area: subcell
effort: m
---

## Problem

The entire bet is that one scene description degrades across tiers without
looking like three different products. That has to be checked with eyes, early,
on a scene with the failure modes in it.

## Proposal

An example rendering a chart with curves, a thin diagonal and a colour gradient,
through the pixel tier, the octant tier and half blocks. Capture all three.

If the octant tier reads as a real downgrade rather than a lower-fidelity
version of the same picture, the abstraction is wrong and we want to know now.

## Acceptance criteria

- [ ] `examples/tiers.rs` renders the same scene at every available tier
- [ ] Runs on a terminal without graphics support and still looks deliberate
- [ ] Captured output committed under `docs/`
- [ ] A written judgement recorded in the PR: does it hold up?
