//! High-resolution terminal input.
//!
//! Resolution is half the story; control is the other half. A slider rendered
//! with sub-cell precision that can only read the mouse to the nearest cell is a
//! control with eight positions. `SGR 1016` reports the pointer in **pixels**,
//! which — divided by the cell size from the capability probe — gives fractional
//! cell coordinates that match the scene graph's own units.
//!
//! The kitty keyboard protocol supplies the other missing piece: key *release*
//! events, without which press-and-hold, drag with proper release semantics, and
//! momentum scrolling cannot be expressed.

use subcell_scene::Pt;

/// Mouse buttons, as reported by SGR mouse encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// Primary button.
    Left,
    /// Middle button.
    Middle,
    /// Secondary button.
    Right,
    /// Wheel up.
    WheelUp,
    /// Wheel down.
    WheelDown,
}

/// Where in a gesture an event falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Button went down.
    Down,
    /// Pointer moved with a button held.
    Drag,
    /// Button came up.
    Up,
    /// Pointer moved with no button held.
    Move,
}

/// Whether a key event is a press, an auto-repeat, or a release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyState {
    /// Initial press.
    Press,
    /// Auto-repeat while held.
    Repeat,
    /// Release. Only reported under the kitty keyboard protocol.
    Release,
}

/// An input event in the scene's own coordinate system.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    /// A pointer event, positioned in fractional cells.
    Mouse {
        /// Position, in cells. Fractional when the terminal reports pixels.
        at: Pt,
        /// Gesture phase.
        phase: Phase,
        /// The button involved, if any.
        button: Option<Button>,
    },
    /// A key event.
    Key {
        /// The character or named key.
        key: String,
        /// Press, repeat or release.
        state: KeyState,
    },
}

/// Converts reported pointer coordinates into fractional cell space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapper {
    cell_w: u16,
    cell_h: u16,
}

impl Mapper {
    /// A mapper for a known cell size in pixels.
    ///
    /// # Errors
    ///
    /// Returns `None` if either dimension is zero.
    pub fn new(cell_w: u16, cell_h: u16) -> Option<Self> {
        (cell_w > 0 && cell_h > 0).then_some(Self { cell_w, cell_h })
    }

    /// Maps pixel coordinates from `SGR 1016` to fractional cells.
    ///
    /// Pixel coordinates are zero-based from the top left of the text area.
    pub fn from_pixels(&self, x: u16, y: u16) -> Pt {
        Pt::new(f32::from(x) / f32::from(self.cell_w), f32::from(y) / f32::from(self.cell_h))
    }

    /// Maps cell coordinates from `SGR 1006` to fractional cells.
    ///
    /// SGR cell coordinates are one-based, and carry no sub-cell information, so
    /// the result lands on the centre of the cell rather than pretending to a
    /// precision that was never reported.
    pub fn from_cells(&self, col: u16, row: u16) -> Pt {
        Pt::new(f32::from(col.saturating_sub(1)) + 0.5, f32::from(row.saturating_sub(1)) + 0.5)
    }

    /// How many distinguishable pointer positions exist across one cell.
    ///
    /// This is the number that makes the case for `SGR 1016`: one without it.
    pub fn resolution_per_cell(&self) -> u32 {
        u32::from(self.cell_w) * u32::from(self.cell_h)
    }
}

/// The sequence enabling pixel-resolution mouse reporting.
///
/// `1002` requests button and drag events, `1016` requests SGR-pixels encoding.
pub const ENABLE_MOUSE_PIXELS: &str = "\x1b[?1002h\x1b[?1016h";

/// The matching disable sequence.
pub const DISABLE_MOUSE_PIXELS: &str = "\x1b[?1016l\x1b[?1002l";

/// Pushes kitty keyboard flags requesting disambiguation and release events.
pub const ENABLE_KITTY_KBD: &str = "\x1b[>3u";

/// Pops the kitty keyboard flags pushed by [`ENABLE_KITTY_KBD`].
pub const DISABLE_KITTY_KBD: &str = "\x1b[<u";

#[cfg(test)]
mod tests {
    // Several tests assert that endpoints and conserved quantities are
    // *exactly* right; that exactness is the property under test.
    #![allow(clippy::float_cmp)]

    use super::*;

    fn mapper() -> Mapper {
        Mapper::new(8, 17).unwrap()
    }

    #[test]
    fn rejects_a_zero_cell_size() {
        assert!(Mapper::new(0, 17).is_none());
        assert!(Mapper::new(8, 0).is_none());
    }

    #[test]
    fn pixel_coordinates_become_fractional_cells() {
        let m = mapper();
        assert_eq!(m.from_pixels(0, 0), Pt::new(0.0, 0.0));
        assert_eq!(m.from_pixels(8, 17), Pt::new(1.0, 1.0));
        let mid = m.from_pixels(12, 17);
        assert!((mid.x - 1.5).abs() < 1e-6, "x was {}", mid.x);
    }

    #[test]
    fn sub_cell_pointer_positions_are_distinguishable() {
        let m = mapper();
        assert_ne!(m.from_pixels(24, 0), m.from_pixels(27, 0));
        assert_eq!(m.resolution_per_cell(), 136);
    }

    #[test]
    fn cell_coordinates_land_mid_cell_and_are_one_based() {
        let m = mapper();
        assert_eq!(m.from_cells(1, 1), Pt::new(0.5, 0.5));
        assert_eq!(m.from_cells(4, 2), Pt::new(3.5, 1.5));
    }

    #[test]
    fn cell_coordinates_survive_a_zero_row() {
        // Some terminals have been known to emit 0; clamp rather than wrap.
        assert_eq!(mapper().from_cells(0, 0), Pt::new(0.5, 0.5));
    }

    #[test]
    fn a_drag_across_one_cell_yields_many_positions() {
        let m = mapper();
        let seen: std::collections::BTreeSet<_> =
            (0..8).map(|px| format!("{:?}", m.from_pixels(px, 0))).collect();
        assert_eq!(seen.len(), 8, "pixel mouse must resolve within a cell");
    }

    #[test]
    fn enable_and_disable_are_symmetric() {
        assert!(ENABLE_MOUSE_PIXELS.contains("1016h"));
        assert!(DISABLE_MOUSE_PIXELS.contains("1016l"));
        assert_eq!(ENABLE_KITTY_KBD, "\x1b[>3u");
        assert_eq!(DISABLE_KITTY_KBD, "\x1b[<u");
    }
}
