---
id: 5
title: 'Drive a real terminal: raw mode, write queries, read replies'
type: feature
status: planned
milestone: v0.1
created: 2026-09-10
updated: 2026-09-10
priority: p0
area: subcell-caps
effort: l
---

## Problem

`subcell-caps` parses replies and decides policy, but performs no I/O. Nothing
can actually detect anything yet, which makes every downstream tier selection
hypothetical.

## Proposal

Write the probe batch, read until the primary device attributes fence or the
deadline, parse what arrived. Terminals that ignore a query stay silent, so the
fence is what bounds the wait.

Must not corrupt the terminal on timeout, on a hostile reply, or on a panic
elsewhere in the process.

## Acceptance criteria

- [ ] Raw mode entered and restored, including on panic
- [ ] Deadline bounded; a silent terminal costs the timeout, not a hang
- [ ] Malformed and adversarial replies neither panic nor allocate unboundedly
- [ ] `subcell-probe` reports real cell size, graphics and keyboard flags
- [ ] Works when stdin is not a tty (returns conservative caps, no error)
