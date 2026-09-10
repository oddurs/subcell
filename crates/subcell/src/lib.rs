//! A resolution-independent terminal UI renderer.
//!
//! `subcell` describes the screen in continuous coordinates and quantizes down
//! to whatever the terminal can actually draw. A terminal cell offers 16.7
//! million colours and roughly 136 pixels; every technique in this workspace is
//! a way of spending colour precision to buy back spatial precision.
//!
//! The rule the design rests on: **resolution is a runtime property.** Probe the
//! terminal, then pick a renderer. Nothing is allowed to assume a tier.
//!
//! # Layers
//!
//! | Crate | Responsibility |
//! |---|---|
//! | [`caps`] | What can this terminal do? |
//! | [`color`] | Oklab, perceptual blending, clustering |
//! | [`scene`] | Fractional-cell scene graph |
//! | [`cov`] | Analytic coverage, sub-cell antialiasing |
//! | [`blit`] | Glyph-tier quantizers |
//! | [`px`] | Kitty graphics and sixel |
//! | [`chan`] | Underline, overline, strikethrough as rules |
//! | [`input`] | Pixel mouse and the kitty keyboard protocol |
//!
//! # Example
//!
//! ```
//! use subcell::{Plan, caps::Caps};
//!
//! // With nothing detected, the renderer still has a safe floor.
//! let plan = Plan::for_caps(&Caps::conservative());
//! assert!(!plan.pixel_tier);
//! ```

pub use subcell_blit as blit;
pub use subcell_caps as caps;
pub use subcell_chan as chan;
pub use subcell_color as color;
pub use subcell_cov as cov;
pub use subcell_in as input;
pub use subcell_px as px;
pub use subcell_scene as scene;

use caps::Caps;

/// The renderer configuration chosen for a given terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// Glyph tier to quantize into.
    pub tier: blit::Tier,
    /// Whether a pixel backend is usable.
    pub pixel_tier: bool,
    /// The selected pixel backend, if any.
    pub backend: Option<px::Backend>,
    /// Whether rules can carry their own colour.
    pub colored_rules: bool,
    /// Whether the pointer resolves inside a cell.
    pub analog_pointer: bool,
}

impl Plan {
    /// Derives a plan from a capability set.
    pub fn for_caps(caps: &Caps) -> Self {
        Self {
            tier: caps.effective_tier(),
            pixel_tier: caps.pixel_tier_available(),
            backend: px::backend_for(caps),
            colored_rules: caps.underline.colored,
            analog_pointer: caps.mouse_px && caps.cell_px.is_some(),
        }
    }

    /// A one-line human-readable summary, for `subcell-probe` and bug reports.
    pub fn summary(&self) -> String {
        let pixels = match self.backend {
            Some(px::Backend::Kitty) => "kitty",
            Some(px::Backend::Sixel { .. }) => "sixel",
            None => "none",
        };
        format!(
            "tier={:?} pixels={pixels} rules={} pointer={}",
            self.tier,
            if self.colored_rules { "colored" } else { "inherited" },
            if self.analog_pointer { "sub-cell" } else { "cell" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caps::{Graphics, Muxer, Size, UnderlineCaps};

    #[test]
    fn conservative_plan_is_the_floor() {
        let plan = Plan::for_caps(&Caps::conservative());
        assert_eq!(plan.tier, blit::Tier::Half);
        assert!(!plan.pixel_tier && plan.backend.is_none());
        assert!(!plan.colored_rules && !plan.analog_pointer);
    }

    #[test]
    fn a_capable_terminal_gets_everything() {
        let caps = Caps {
            cell_px: Some(Size::new(8, 17)),
            graphics: Graphics::Kitty,
            blit_max: Some(blit::Tier::Braille),
            underline: UnderlineCaps { colored: true, styled: true, ..UnderlineCaps::default() },
            mouse_px: true,
            ..Caps::conservative()
        };
        let plan = Plan::for_caps(&caps);
        assert_eq!(plan.tier, blit::Tier::Braille);
        assert_eq!(plan.backend, Some(px::Backend::Kitty));
        assert!(plan.colored_rules && plan.analog_pointer);
        assert!(plan.summary().contains("pixels=kitty"));
    }

    #[test]
    fn one_scene_survives_every_tier() {
        // The load-bearing property: the same description quantizes at any tier.
        use color::{Oklab, Srgb8};
        let ink = Oklab::from_srgb8(Srgb8::new(255, 79, 163));
        let ground = Oklab::from_srgb8(Srgb8::new(14, 16, 24));
        for tier in blit::Tier::ALL {
            let patch = blit::Patch::from_fn(tier, |x, _| if x == 0 { ground } else { ink });
            let cell = blit::quantize(&patch);
            assert!(cell.glyph != '\0', "{tier:?} produced no glyph");
        }
    }

    #[test]
    fn a_multiplexer_degrades_the_plan_without_failing() {
        let caps = Caps {
            cell_px: Some(Size::new(8, 17)),
            graphics: Graphics::Kitty,
            blit_max: Some(blit::Tier::Braille),
            multiplex: Some(Muxer::Tmux),
            ..Caps::conservative()
        };
        let plan = Plan::for_caps(&caps);
        assert_eq!(plan.tier, blit::Tier::Quadrant);
        assert!(!plan.pixel_tier, "graphics must not be attempted through tmux");
    }

    #[test]
    fn analog_pointer_needs_both_pixel_mouse_and_a_cell_size() {
        let no_size = Caps { mouse_px: true, ..Caps::conservative() };
        assert!(!Plan::for_caps(&no_size).analog_pointer);
        let sized = Caps { cell_px: Some(Size::new(8, 17)), ..no_size };
        assert!(Plan::for_caps(&sized).analog_pointer);
    }
}
