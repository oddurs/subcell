The bet this project is making, in order.

**`v0.1` proves the abstraction.** Not the framework — the framework is the last
thing to build. One scene description, rendered through three backends, looked
at honestly. If the octant tier reads as a different product rather than a
lower-fidelity version of the same picture, the abstraction is wrong and that
should surface at week three, not week thirty.

**`v0.2` makes it work on real terminals.** Placeholder encoding, shared-memory
transfer, event decoding, and a support matrix built from actual probe runs
rather than from protocol documentation.

**`v0.3` makes it usable.** Layout, widgets, an event loop. Last on purpose: a
framework built before the renderer is proven bakes in assumptions the renderer
then has to fight.

Two things are deliberately not scheduled early. Widgets, for the reason above.
And `subcell-tiles`, the private-use-area font — it is real, but it asks users
to install something, and the feature has to be worth it before anyone will.

<!-- Everything below this line is generated. Edit the prose above in
     docs/roadmap-intro.md, and the work items in .cairn/items/. -->
