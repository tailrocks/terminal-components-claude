# Runtime natural-width focus fixture correction

Status: **Test-fixture repair only. No production behavior changed.**

## Finding

The `natural_width_follows_the_themed_glyph` test builds a synthetic `Gutterful` measurement component and sets `LastFrame.snapshot.focus` directly. That fixture does not pass through `Button`'s focus-presentation policy, and it did not set `snapshot.focus_visible`. The theme gutter recipe is conditioned on `FOCUS_VISIBLE`; raw `FOCUSED` remains present for keyboard bindings but does not render the gutter. The measured width of 6 in that fixture was therefore correct. The separate `termrock-controls` Button policy integration test covers real widget focus behavior.

## Repair

The synthetic keyboard-focus case now sets both snapshot fields and asserts `FOCUSED | FOCUS_VISIBLE`. With label `Save`, two one-cell paddings, and custom wide/narrow focus glyphs, natural widths remain different: 8 and 7. A pointer-focus case sets `focus` with `focus_visible = false`, asserts `FOCUSED` alone, and requires both widths to remain 6, proving pointer focus does not manufacture a visible gutter.

## RED/GREEN evidence

- Protected starting commit: `03c3da6a07b7c482f915c0af46a2e78dc27c6b14`.
- Before the fixture edit: `cargo nextest run --locked -p termrock-runtime --lib natural_width_follows_the_themed_glyph` failed at `measure.rs:191` with left `6`, right `7` (1 test run, 0 passed, 234 skipped).
- After the fixture edit, that focused test passed (1 run, 0 skipped).
- `cargo nextest run --locked -p termrock-controls --test button_contract focus_visibility_policies_hold_for_every_button_variant_and_color_level` passed (1 run, 7 skipped).
- `cargo nextest run --locked -p termrock-runtime --all-targets` passed (357 passed, 3 skipped).
- `cargo clippy --locked -p termrock-runtime --all-targets -- -D warnings` passed; `rustfmt --edition 2024 --check crates/termrock-runtime/src/ui/measure.rs` passed.
- The broader palette semantic fixture failure remains independently recorded; this test correction does not waive or alter it.

## Scope and remaining checks

Only `crates/termrock-runtime/src/ui/measure.rs` and this audit record are changed. No theme recipe, Button, Grid, or runtime focus behavior is modified. The separately recorded palette semantic fixture failure is unchanged and remains unwaived.
