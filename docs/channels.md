# Channels

## Three more rules per cell, in their own colours

Underline, overline and strikethrough are three independently positioned
horizontal rules inside every cell. `SGR 58` gives the underline a colour
unrelated to the foreground, and `SGR 4:n` gives it five styles.

They composite **over** whatever glyph and colours the cell already carries, so
they cost no spatial resolution at all. You can render syntax-highlighted text
and draw a coloured signal trace through the same cells.

```
SGR 53      overline         top of the cell
SGR 9       strikethrough    through the middle
SGR 4       underline        bottom of the cell
SGR 58      underline colour, independent of the foreground
SGR 4:1..5  solid, double, curly, dotted, dashed
```

## What this is actually for

- Diagnostics underlining source in severity colours **while the code keeps its
  syntax highlighting** — two independent colour channels in the same cells.
- A sparkline drawn through a table row without giving up a line of layout.
- Progress as a rule rather than a block, so it does not consume vertical space.

## Independent degradation

Each channel drops out on its own; this is not a mode.

| Missing | Result |
|---|---|
| `SGR 58` | Colour dropped, rule inherits the foreground |
| Styled underlines | Every style becomes solid `SGR 4` |
| `SGR 53` / `SGR 9` | That channel is simply not drawn |

Coloured and styled underlines are widely supported — kitty, Ghostty, WezTerm,
foot, Alacritty and modern VTE. Overline and strikethrough are patchier. Probe
for each rather than assuming they arrive as a set.

## Reset carefully

`Rules::reset` emits only the resets for what was actually set. Blanket `SGR 0`
would clear the cell's colours too, and blanket rule resets on every cell are
wasted bytes on a hot path.

**Status:** SGR construction and the degradation rules are implemented and
tested. Wiring the channels into a cell writer is `v0.2` work, and it needs the
terminal writer from `v0.1` first.
