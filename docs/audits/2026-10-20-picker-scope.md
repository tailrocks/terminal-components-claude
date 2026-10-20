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
buffer and cursor for absent and empty captions, right alignment, clipping,
the six-cell hide threshold, and Enter selection. Existing explicit-width
Picker tests run alongside it.

Command: `cargo nextest run -p termrock-overlays --test picker_scope --test picker_width`

Result: 7 passed, 0 skipped.

This component check is not a complete frozen CLI parity run. Application
composition and the paired baseline CLI capture remain separate gates.
