# Table horizontal scrollbar

The first table scrollbar implementation registered its horizontal track and thumb under the grid owner. The grid's vertical scrollbar uses the same parts, so the later registration overwrote the earlier hit region. It also painted one glyph per bar, hid the bar when the visible suffix fit despite hidden columns on the left, and allowed keyboard scrolling past the last valid viewport.

`Table` now registers horizontal scrollbar parts under a stable child ID. Its state uses shared `ScrollState` layout, thumb, press, drag, and clamp calculations. The grid receives the viewport's maximum horizontal offset. Bar glyphs repeat across the full track and thumb, including width-two glyphs. Independent regressions cover left-side overflow, disjoint hit owners, keyboard clamping, track clicks, thumb dragging, and full-width glyph output.

Validation: `cargo nextest run -p termrock-grid --lib --no-fail-fast` passed 45 tests. Full-workspace formatting remains blocked by pre-existing formatting drift in `crates/showcase-demos/src/pages/lists.rs:705`; changed Rust files formatted directly with rustfmt.
