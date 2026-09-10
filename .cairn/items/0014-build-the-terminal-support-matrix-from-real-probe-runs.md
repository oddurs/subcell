---
id: 14
title: Build the terminal support matrix from real probe runs
type: docs
status: backlog
milestone: v0.2
created: 2026-09-10
updated: 2026-09-10
priority: p1
area: docs
effort: m
---

## Problem

The matrix published with the proposal was compiled from protocol documentation,
not from testing, and it is labelled as a planning estimate for that reason.
Shipping it as fact would be dishonest and would also undercut the argument for
probing at all.

## Proposal

Run `subcell-probe` across real terminals, record what actually came back.
Invite contributors to submit their own output.

## Acceptance criteria

- [ ] kitty, Ghostty, WezTerm, foot, Alacritty, iTerm2, Windows Terminal, xterm
- [ ] Each row cites the version tested
- [ ] Divergences from the documentation-based estimate called out explicitly
