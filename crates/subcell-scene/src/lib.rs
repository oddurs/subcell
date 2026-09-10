//! A scene graph in fractional cell coordinates.
//!
//! Every geometric type here is `f32` and measured in **cells**, not characters.
//! That single choice is what lets one scene description be rasterized at any
//! tier: the scene says a rule sits at `x = 3.4`, and it is the rasterizer's job
//! to decide whether that becomes a shifted glyph, a blended colour, or a real
//! pixel.

use subcell_color::Srgb8;

/// A point in fractional cell space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pt {
    /// Horizontal position, in cells.
    pub x: f32,
    /// Vertical position, in cells.
    pub y: f32,
}

impl Pt {
    /// Constructs a point.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// An axis-aligned rectangle in fractional cell space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width, in cells.
    pub w: f32,
    /// Height, in cells.
    pub h: f32,
}

impl Rect {
    /// Constructs a rectangle.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Right edge.
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Whether the rectangle has no area.
    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    /// The overlapping region of two rectangles, or an empty rect.
    #[must_use]
    pub fn intersect(&self, other: &Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let w = self.right().min(other.right()) - x;
        let h = self.bottom().min(other.bottom()) - y;
        if w <= 0.0 || h <= 0.0 { Rect::default() } else { Rect::new(x, y, w, h) }
    }

    /// The smallest rectangle containing both.
    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Rect::new(x, y, self.right().max(other.right()) - x, self.bottom().max(other.bottom()) - y)
    }

    /// The whole-cell rectangle that covers this one, as `(col, row, cols, rows)`.
    ///
    /// This is the damage region: the cells a rasterizer has to redraw.
    pub fn cell_bounds(&self) -> (u16, u16, u16, u16) {
        if self.is_empty() {
            return (0, 0, 0, 0);
        }
        let x0 = self.x.floor().max(0.0) as u16;
        let y0 = self.y.floor().max(0.0) as u16;
        let x1 = self.right().ceil().max(0.0) as u16;
        let y1 = self.bottom().ceil().max(0.0) as u16;
        (x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
    }
}

/// How a primitive is filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    /// A flat colour.
    Solid(Srgb8),
    /// A linear gradient between two colours, interpolated in Oklab.
    Linear {
        /// Colour at the start of the run.
        from: Srgb8,
        /// Colour at the end.
        to: Srgb8,
        /// Direction in radians, clockwise from the positive x axis.
        angle_rad_milli: i32,
    },
}

/// A stroke style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Line width in cells. Values below one are the interesting case: they are
    /// expressed tonally rather than spatially.
    pub width: f32,
    /// Stroke colour.
    pub paint: Paint,
}

/// One drawable.
#[derive(Debug, Clone, PartialEq)]
pub enum Prim {
    /// A filled rectangle.
    Rect {
        /// Bounds.
        r: Rect,
        /// Fill.
        fill: Paint,
    },
    /// A straight line segment.
    Line {
        /// Start point.
        a: Pt,
        /// End point.
        b: Pt,
        /// Stroke style.
        stroke: Stroke,
    },
    /// A run of text anchored at a point.
    Text {
        /// Anchor, at the text baseline's cell origin.
        at: Pt,
        /// The text itself.
        text: String,
        /// Foreground colour.
        fg: Srgb8,
    },
}

impl Prim {
    /// The rectangle this primitive can touch.
    pub fn bounds(&self) -> Rect {
        match self {
            Prim::Rect { r, .. } => *r,
            Prim::Line { a, b, stroke } => {
                let pad = (stroke.width / 2.0).max(0.0);
                let x = a.x.min(b.x) - pad;
                let y = a.y.min(b.y) - pad;
                Rect::new(x, y, (a.x - b.x).abs() + pad * 2.0, (a.y - b.y).abs() + pad * 2.0)
            }
            Prim::Text { at, text, .. } => Rect::new(at.x, at.y, text.chars().count() as f32, 1.0),
        }
    }
}

/// An ordered list of primitives, painted back to front.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scene {
    prims: Vec<Prim>,
}

impl Scene {
    /// An empty scene.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a primitive.
    pub fn push(&mut self, p: Prim) -> &mut Self {
        self.prims.push(p);
        self
    }

    /// The primitives, in paint order.
    pub fn prims(&self) -> &[Prim] {
        &self.prims
    }

    /// Whether the scene has nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.prims.is_empty()
    }

    /// The union of every primitive's bounds.
    pub fn bounds(&self) -> Rect {
        self.prims.iter().fold(Rect::default(), |acc, p| acc.union(&p.bounds()))
    }

    /// Removes every primitive, keeping the allocation.
    pub fn clear(&mut self) {
        self.prims.clear();
    }
}

#[cfg(test)]
mod tests {
    // Several tests assert that endpoints and conserved quantities are
    // *exactly* right; that exactness is the property under test.
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn empty_scene_has_empty_bounds() {
        assert!(Scene::new().bounds().is_empty());
    }

    #[test]
    fn bounds_are_the_union_of_primitives() {
        let mut s = Scene::new();
        s.push(Prim::Rect {
            r: Rect::new(1.0, 1.0, 2.0, 2.0),
            fill: Paint::Solid(Srgb8::default()),
        });
        s.push(Prim::Rect {
            r: Rect::new(5.0, 0.0, 1.0, 1.0),
            fill: Paint::Solid(Srgb8::default()),
        });
        assert_eq!(s.bounds(), Rect::new(1.0, 0.0, 5.0, 3.0));
    }

    #[test]
    fn fractional_bounds_expand_to_whole_cells() {
        // A hairline at x = 3.4 damages cell 3, not cell 4.
        let r = Rect::new(3.4, 0.2, 0.1, 0.5);
        assert_eq!(r.cell_bounds(), (3, 0, 1, 1));
    }

    #[test]
    fn a_feature_spanning_a_boundary_damages_both_cells() {
        let r = Rect::new(3.9, 0.0, 0.3, 1.0);
        assert_eq!(r.cell_bounds(), (3, 0, 2, 1));
    }

    #[test]
    fn intersection_of_disjoint_rects_is_empty() {
        let a = Rect::new(0.0, 0.0, 1.0, 1.0);
        let b = Rect::new(4.0, 4.0, 1.0, 1.0);
        assert!(a.intersect(&b).is_empty());
    }

    #[test]
    fn thin_line_bounds_include_the_stroke_width() {
        let p = Prim::Line {
            a: Pt::new(1.0, 1.0),
            b: Pt::new(4.0, 1.0),
            stroke: Stroke { width: 0.4, paint: Paint::Solid(Srgb8::default()) },
        };
        let b = p.bounds();
        assert!((b.y - 0.8).abs() < 1e-6, "y was {}", b.y);
        assert!((b.h - 0.4).abs() < 1e-6, "h was {}", b.h);
    }

    #[test]
    fn text_bounds_count_characters_not_bytes() {
        let p =
            Prim::Text { at: Pt::new(0.0, 0.0), text: "héllo".to_string(), fg: Srgb8::default() };
        assert_eq!(p.bounds().w, 5.0);
    }

    #[test]
    fn clear_keeps_the_scene_usable() {
        let mut s = Scene::new();
        s.push(Prim::Rect {
            r: Rect::new(0.0, 0.0, 1.0, 1.0),
            fill: Paint::Solid(Srgb8::default()),
        });
        s.clear();
        assert!(s.is_empty());
    }
}
