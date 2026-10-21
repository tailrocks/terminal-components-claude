# Semantic Tabs layout extraction (2026-10-21)

Status: bounded library extraction; independent review and application adoption pending.

Base: `4cc10c3a7158123500fbe38435f2b68874b50561` (`parity/termrock-visual-baseline`). Frozen product source and paired application comparison were not changed by this increment.

## Change

`termrock-navigation::tabs` now exposes an opt-in semantic layout path for `Tabs<T, K, DefaultRow>` through `layout_default`, `TabLayout`, and `draw_layout`. The layout pass accepts immutable area, clip, component state, item data, and an optional `TabWindowReceipt`; it does not accept `Ui`, invoke input/update, paint, or inspect a buffer. It owns formatted `Display` write segments, stable visible row keys, tab and close allocations, overflow/new/status bounds, the active/cursor/first keys, and visible/content counts. The caller can carry `window_receipt()` forward after a nonempty committed draw to preserve existing cold/warm overflow reservation behavior.

The existing draw loop remains shared. Planned drawing consumes the owned segments and geometry; it does not call item `Display`, remeasure a label, or scan rendered cells. The custom `RowFn`/`RowUi` API and callbacks remain on the existing legacy path. The planned path is available only when the component uses `DefaultRow`; custom painters do not gain an inferred semantic layout contract from `intrinsic_label_width`.

The caller must rebuild the plan after state, data, component configuration, viewport, or clip changes, and draw it under the same clip. Empty areas produce no receipt because legacy drawing publishes no layout there. The types are public through the existing public `termrock_navigation::tabs` module; crate-root exports and application adoption are deferred pending review.

## Evidence

- `cargo nextest run -p termrock-navigation --lib`: 106 passed, 0 skipped.
- Differential tests compare complete Ratatui buffers and registered regions for legacy and planned rendering at truecolor, ANSI 256, ANSI 16, and mono levels; widths 0, 1, 2, 8, 13, 24, and 40; active/focus, hover, pressed, and close-part states; empty labels, trailing spaces, wide text, combining marks, control characters, and stable-key reordering.
- Warm-frame test carries the first planned receipt into the next layout and compares it with the legacy runtime's previous-frame counts.
- Split-write `Display` test verifies write segmentation and that planned drawing does not call `Display` again. Custom `RowUi` test verifies legacy callback execution remains intact.
- `cargo clippy -p termrock-navigation --all-targets -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.

These checks prove this component path only. No runtime publication adapter, shared consumer adoption, full application flow, frozen baseline comparison, or full workspace gate was part of this increment.
