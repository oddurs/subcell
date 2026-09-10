---
id: 12
title: Shared-memory image transfer
type: feature
status: backlog
milestone: v0.2
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: subcell-px
effort: m
---

## Problem

650k pixels per frame base64-encoded down a pty is real bandwidth. The protocol
supports `t=s` for a shared memory object, skipping base64 entirely on the hot
path.

## Acceptance criteria

- [ ] `t=s` transfer path with a base64 fallback
- [ ] Shared memory objects are cleaned up, including on panic
- [ ] Benchmark against the base64 path at a realistic frame size
