# Input

## An analog renderer deserves analog input

`SGR 1016` reports the pointer in **pixels**, not cells. Divided by the cell size
from the capability probe, that gives fractional cell coordinates in exactly the
units the scene graph already uses.

```rust
let m = Mapper::new(8, 17).unwrap();   // cell size from CSI 16 t
m.from_pixels(12, 17);                 // Pt { x: 1.5, y: 1.0 }
m.resolution_per_cell();               // 136
```

Resolution is half the story; control is the other half. A slider rendered with
sub-cell precision that can only read the mouse to the nearest cell is a control
with eight positions. With mode 1016 it has a hundred and thirty-six.

## Honest fallback

Without `SGR 1016`, `from_cells` maps to the **centre** of the cell rather than
its corner. The pointer really was somewhere in that cell and the terminal did
not say where; reporting the centre is the least wrong answer, and it does not
pretend to a precision that was never measured.

SGR cell coordinates are one-based. Some terminals have been known to emit zero,
so the mapper clamps rather than wrapping.

## Key release events

The kitty keyboard protocol reports real key-down **and key-up** events, with
modifiers on every key, and distinguishes `Ctrl+I` from `Tab`.

Key-up is the one that unlocks a category rather than a convenience:
press-and-hold, drag with proper release semantics, chords, and momentum
scrolling. These are the interactions that make a GUI feel responsive, and whose
absence is a large part of why TUIs feel like forms.

Enable with `CSI > 3 u` — which *pushes* flags — and pop them with `CSI < u` on
the way out. Pushing and popping rather than setting means nesting works and a
crash does not leave the terminal in a strange state for the next program.

**Status:** the coordinate mapping is implemented and tested. Decoding actual
event streams is `v0.2` work.
