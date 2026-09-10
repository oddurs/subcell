---
id: 21
title: Publish the crates to crates.io
type: chore
status: backlog
created: 2026-09-10
updated: 2026-09-10
priority: p2
area: release
effort: s
---

## Problem

Names are unclaimed and the crates are unpublished. `subcell-blit` and
`subcell-caps` are meant to be usable without adopting the framework, which
only works if they are actually on crates.io.

## Acceptance criteria

- [ ] All nine names reserved
- [ ] Publish order respects the dependency graph
- [ ] `cargo publish --dry-run` green for every crate
- [ ] Release workflow tags and publishes from CI
