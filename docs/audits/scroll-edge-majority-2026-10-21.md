# Scroll edge majority correction

The runtime now preserves majority-background selection after the first eight
distinct colors. `Ui::majority_bg` keeps its allocation-free eight-color path
and creates a `HashMap` only after a ninth distinct background appears. The
overflow path counts every remaining color and resolves equal counts by the
row-major order of each color's first occurrence, matching the previous table
path's last-entry tie behavior. Scroll fading still changes only foreground
color or the outer-row dim modifier; background, glyph, and underline remain
intact. No color quantizer changed.

The eight-color fast path remains allocation-free. Overflow counts use a
`UiCore`-owned `HashMap`, cleared and reused between calls, so overflow does
not allocate after warm-up. The independent review of the initial correction
found 81,600 allocated bytes across 200 nine-color frames; the unchanged
external allocation test now passes with zero allocations for 8, 9, 12, and
48 colors.

`crates/termrock-runtime/tests/scroll_edges_majority.rs` calls public
`Ui::scroll_edges` through `Harness`. Against the original implementation at
`05d63fe9c8a0e9a327489ae657a4ff6dc6571324`, the ninth-color-majority and
mixed Reset/index/RGB tie tests failed; the narrow and empty-area test passed.
After the correction, all four runtime tests passed. The independent
zero-allocation warm-redraw assertion passes unchanged over 200 measured
frames after 20 warm-up frames. The ninth-color
test uses eight singleton RGB backgrounds plus 32 black cells and verifies that a
bottom-edge cell fades while retaining its background, symbol, and underline.
The tie test verifies that Indexed(42), the later of two equal Reset/index
majorities, receives the dim modifier. The edge-safety test covers a one-cell
wide view and an empty scroll area.

This is a focused runtime regression result. It does not establish complete
workspace validation or application parity.
