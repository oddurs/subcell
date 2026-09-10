# Architecture

## The one rule

**Resolution is a runtime property.** Probe the terminal, then pick a renderer.
No crate below the facade is allowed to assume a tier, and no geometry is stored
in whole cells.

Every other decision here follows from that. The web settled this in 2010 with
device pixel ratio; terminals have had the equivalent query since the 1980s and
almost nobody calls it.

## Why nine crates

The split is not organisational tidiness. Two of these layers are useful to
people who will never adopt the framework, and keeping them independently
depend-able is how the ideas spread faster than `subcell` does:

- `subcell-blit` takes colour and emits cells. A ratatui user can use it today.
- `subcell-caps` is a vocabulary and a parser. Any TUI library can use it.

Everything else exists to keep those two honest.

```
                    subcell  (facade: Plan, re-exports)
                       |
     +--------+--------+--------+--------+---------+
     |        |        |        |        |         |
  subcell- subcell- subcell- subcell- subcell-  subcell-
   caps      blit      cov      px      chan       in
     |        |        |        |        |         |
     |        +--------+----+---+--------+         |
     |                      |                      |
     |               subcell-color          subcell-scene
     |                                             |
     +----------- (caps depends on blit for Tier) -+
```

Dependency direction is strict and acyclic: `color` and `scene` are leaves,
`caps` sits above `blit` only to name a tier, and the facade sits above all.

## The layers

| Crate | Question it answers |
|---|---|
| `subcell-caps` | What can this terminal do? |
| `subcell-color` | What does this colour look like to a person? |
| `subcell-scene` | Where is this thing, in continuous coordinates? |
| `subcell-cov` | How much of this cell does it cover? |
| `subcell-blit` | Which glyph and two colours best approximate this patch? |
| `subcell-px` | How do I get real pixels onto the screen? |
| `subcell-chan` | What else can this cell carry for free? |
| `subcell-in` | Where exactly is the pointer? |
| `subcell` | Given all of the above, what should I render with? |

## The seam that matters

The scene graph speaks in `f32` cells. The rasterizers speak in sub-cells. The
terminal speaks in cells. The conversion happens once, at the bottom, and it is
the only place a tier is known.

```rust
// Continuous in, discrete out. The tier is a parameter, never an assumption.
scene.rasterize(backend_for(&caps))
```

This is what makes the central claim testable: if one scene description cannot
survive being rendered at five different densities without looking like five
different products, the abstraction is wrong. See [the roadmap](roadmap.md) —
proving that is the whole of `v0.1`, and it is deliberately scheduled before any
widget work.

## What is not here yet

No layout engine, no widgets, no event loop. Those are `v0.3`, and they are last
on purpose: a framework built before the renderer is proven would bake in
assumptions the renderer then has to fight.

Nothing performs terminal I/O. `subcell-caps` parses replies but does not read
them; `subcell-px` builds escape sequences but does not write them. The pure
half is built and tested; the I/O half is `v0.1` and `v0.2` work.
