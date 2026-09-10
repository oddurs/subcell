---
id: 13
title: Decode pixel mouse and kitty keyboard event streams
type: feature
status: backlog
milestone: v0.2
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: subcell-in
effort: m
---

## Problem

`Mapper` converts coordinates but nothing parses the events they come from.

## Proposal

Incremental parser over a byte stream: SGR 1006 and 1016 mouse, kitty keyboard
with release and repeat. Must handle split reads, since escape sequences arrive
in fragments.

## Acceptance criteria

- [ ] Handles sequences split across reads
- [ ] Distinguishes `Ctrl+I` from `Tab` under the kitty protocol
- [ ] Release events produce `KeyState::Release`
- [ ] Fuzzed against random bytes without panicking
