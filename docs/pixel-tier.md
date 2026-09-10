# The pixel tier

## Unicode placeholders are the whole point

The kitty graphics protocol is the only pixel protocol that composes properly
with a terminal UI, and the reason is virtual placement.

You upload an image once, then reference it with `U+10EEEE` placeholder cells
whose foreground colour encodes the image id and whose combining marks encode
row and column. The consequence is enormous: **your cell-based layout and damage
tracking keep working**. The image is placed *through* the grid, not painted
over it. It scrolls correctly, it clips to panes correctly, and the damage
tracker does not need to know it is there.

This is how Yazi and ratatui-image do previews.

## Why not sixel

Sixel is more widely supported and worse for this purpose:

- No z-order and no placeholder mechanism, so it paints over the grid.
- Palette-limited rather than truecolor.
- Mangled by scrollback and by pane clipping.

It stays as a fallback backend because "worse" still beats "nothing", but it
cannot be the design target.

## Wire format

Commands are APC sequences: `ESC _ G <key=value,...> ; <payload> ESC \`

```
a=t   transmit          f=32  RGBA          i=<id>  image id
a=p   place             s,v   pixel size    c,r     cell extent
a=d   delete            U=1   virtual placement
```

`subcell-px` builds and validates these today, including base64 of the payload,
with dimension and length mismatches rejected rather than trusted.

## Not built yet

**Placeholder encoding.** The row/column diacritics come from a specific list of
combining characters. This is fiddly enough that guessing at it would produce a
subtly broken placement, so it is a tracked `v0.2` item rather than a sketch.

**Shared memory transfer.** 650k pixels per frame base64'd down a pty is real
bandwidth. The protocol supports `t=s` for a shared memory object, which skips
base64 entirely on the hot path. Damage tracking is not optional at this tier.

## The honest limitation

tmux. Graphics passthrough through a multiplexer is fragile, a large share of
users live in one, and the policy is to detect it and not try. Which means: if
the glyph tiers do not look good, nothing else matters. That is why octants are
scheduled ahead of everything here.
