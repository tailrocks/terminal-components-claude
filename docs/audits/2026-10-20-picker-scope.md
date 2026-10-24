# Picker scope caption parity

## Reference

Frozen oracle: `visual-baseline` source commit
`4169783ae883788186d161af09ba9c481bd84e5c`, `src/widgets/picker.rs:653`.
That implementation paints an optional caption on the title row, right aligned,
using muted text. Caption width is `inner.width - width(title) - 3`; captions
are hidden when fewer than six cells remain and use the shared grapheme-safe
right truncation rule otherwise.

## Change

`termrock_overlays::picker::Picker::scope_label` adds the optional caption to
the shared Picker. It uses `Part::META`; no caption keeps the existing title
row unchanged. The application remains responsible for providing domain text.

## Evidence

`crates/termrock-overlays/tests/picker_scope.rs` checks the complete rendered
buffer and cursor for the baseline explicit outer rectangle `(18,15,84,9)`,
title at `(20,16)`, caption ending at column `99`, Unicode grapheme truncation,
`Part::META` override, absent and empty captions, clipping, the six-cell hide
threshold, and Enter selection. The renderer test calls the public `Picker::draw`
with that explicit rectangle to isolate caption painting from layer placement.
Widths 34 and 33 exercise exactly six available cells (visible) and five
(hidden). Expected caption cell styles come from an independent
`Theme::resolve(Family::PICKER, ..., Part::META, ...)` call; a custom META patch
checks the full resulting cell style independently of the rendered cells.

Known separate placement gap: current `ScreenAlign::Center` resolves vertical
position with `free / 3`, so Picker's layer resolver does not produce frozen
rectangle `(18,15,84,9)` on a 120×40 screen. This is tracked for the shared layer
resolver; the caption test does not certify layer placement.

Command: `cargo nextest run -p termrock-overlays --all-targets`

Result: 89 passed, 0 skipped. `cargo clippy -p termrock-overlays --all-targets`,
`cargo fmt --check`, and `git diff --check` also passed.

This component check is not a complete frozen CLI parity run. Application
composition and the paired baseline CLI capture remain separate gates.
