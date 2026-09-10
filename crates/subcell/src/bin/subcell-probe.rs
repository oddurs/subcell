//! Reports what `subcell` would render with in the current terminal.
//!
//! This reads the process environment only. It does not yet write queries to the
//! terminal and read replies, so it cannot report a cell size or a graphics
//! protocol — those need the interactive probe. What it does report is real, and
//! it is the fastest way to see how the tier selection behaves.

use subcell::{
    Plan,
    caps::{Caps, detect_muxer},
};

fn main() {
    let term = std::env::var("TERM").ok();
    let tmux = std::env::var("TMUX").ok();
    let muxer = detect_muxer(term.as_deref(), tmux.as_deref());

    let caps = Caps { multiplex: muxer, ..Caps::conservative() };
    let plan = Plan::for_caps(&caps);

    println!("TERM             {}", term.as_deref().unwrap_or("(unset)"));
    println!("multiplexer      {}", muxer.map_or("none".to_string(), |m| format!("{m:?}")));
    println!("plan             {}", plan.summary());
    println!();
    println!("Environment only. Cell size, graphics protocol and underline");
    println!("support need the interactive probe; see docs/capabilities.md.");
}
