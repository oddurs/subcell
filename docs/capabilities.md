# Capabilities

## Terminals have a device pixel ratio

`CSI 16 t` returns the cell size in pixels. `CSI 14 t` returns the text area in
pixels. `CSI 18 t` returns it in cells. Divide and you have an exact, honest DPR
for the surface you are drawing on.

```
-> CSI 14 t          <- CSI 4 ; 680 ; 960 t      text area is 960x680 px
-> CSI 18 t          <- CSI 8 ;  40 ; 120 t      text area is 120x40 cells
                        so one cell is 8x17 px
```

Note the wire order is **height before width**, which is the opposite of how
everything else in this codebase writes it. `parse_report` normalises it.

Every TUI library assumes the cell is the pixel — it is why `Rect { x: u16 }` is
the universal primitive. That assumption is simply wrong, and the terminal will
tell you so if asked.

## The probe

One batch of queries, one round trip, bounded by a timeout:

```
CSI 16 t          cell size in pixels
CSI 14 t          text area size in pixels
CSI 18 t          text area size in cells
CSI ? u           kitty keyboard flags
CSI ? 2026 $ p    synchronized output support
CSI c             primary device attributes  <- the fence
```

The last one is the trick. Every terminal answers primary device attributes, so
its reply marks the end of the batch: read until you see it, or until the
deadline, and you never have to guess how long to wait for a terminal that
silently ignored a query it did not understand.

Queries that a terminal does not recognise produce no reply rather than garbage,
so an unsupported feature costs nothing but the wait.

**Status:** the parsing and the policy are implemented and tested. Driving a
real terminal — raw mode, writing the batch, reading replies under a deadline —
is `v0.1` work. `subcell-caps` performs no I/O today.

## Degradation policy

| Condition | Consequence |
|---|---|
| Nothing detected | Half blocks. Universally safe. |
| Multiplexer detected | Glyph tier capped at quadrant; pixel tier off entirely. |
| Graphics but no cell size | Pixel tier off — there is no way to rasterize at the right scale. |
| No `SGR 58` | Rules inherit the foreground instead of carrying their own colour. |
| No styled underlines | Every underline becomes solid. |
| No `SGR 1016` | Pointer resolves to cell centres, and says so rather than faking precision. |

Each capability degrades independently. There is no "high" and "low" mode,
because real terminals do not cluster that way — plenty support coloured
underlines and no graphics at all.

## tmux is where this dies

Graphics passthrough through a multiplexer is fragile at best, and a large share
of users live inside one. The policy is to detect it and drop to a glyph tier
immediately rather than to try, fail, and leave the screen corrupted.

Detection reads `TMUX` and `TERM`. It is not authoritative — a nested session
can hide it — but a positive result is reliable enough to degrade on, and the
cost of a false positive is a slightly lower tier rather than a broken display.

## The support matrix

The table in [the proposal](https://github.com/oddurs/subcell) was compiled from
protocol documentation, **not from testing**, and it should not be trusted for
anything except scoping. Building a real one from probe runs across actual
terminals is tracked on the roadmap.

This is also the argument for the probe existing at all: never branch on a
terminal name when you can ask.

## Octant detection

The one capability with no clean query. See
[blitters.md](blitters.md#detection-is-the-weak-point).
