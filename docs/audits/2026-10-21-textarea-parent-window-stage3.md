# TextArea parent-window adapter — stage 3

Status: **Fields adapter implemented; Form integration and full acceptance remain incomplete.**

## Change

`TextArea::draw_window` now paints original document rows through the prepared `VerticalWindow`; the existing `draw` and form draw methods retain zero-offset behavior and delegate through a full window. Window-aware form/secret-form adapters are additive. A shared private `TextAreaGeometry` resolves physical `TEXT` bounds, parent source-row mapping, document coordinates, visible cursor placement, and width from the same prepared window. Update maps pointer press/drag/secondary-click using the last presented child-scroll offsets and only when the point belongs to the actual visible body. Missing geometry leaves coordinate-dependent input inert without bypassing FocusOut or keyboard handling. `caret_source_row` is a read-only accessor for a future Form reveal integration.

The child ScrollState remains the full logical viewport. Drawing and measuring do not mutate it or the editing draft. A focused editable caret is published only when it maps inside the visible `TEXT` rectangle; no placeholder control is registered.

## Verification

- `cargo nextest run -p termrock-fields --test textarea_window`: 4 passed.
- `cargo nextest run -p termrock-fields --all-targets`: 74 passed, 0 skipped.
- `cargo clippy -p termrock-fields --all-targets -- -D warnings`: passed.
- `rustfmt --edition 2024 crates/termrock-fields/src/textarea.rs crates/termrock-fields/tests/textarea_window.rs`: passed.
- Workspace `cargo fmt --check` is **not clean** due pre-existing formatting differences in `crates/showcase-demos/src/pages/chrome.rs`, an out-of-scope file on the integrated branch. No workspace formatting was applied.

The new public consumer tests exercise top-only, bottom-only and both-edge clipping, wide/combining text cell output, nonzero child plus parent offsets, pointer insertion into the original document row, caret exclusion for a clipped focused editor, and logical five-row PageDown despite a two-row physical window. Initial probe failures were retained as evidence in the work record: the wide glyph's continuation cell is represented as a space in Ratatui's Buffer, and focusing the fixture moves its child offset to the document end before first draw. Assertions now encode those established semantics; no production output was changed to satisfy the probe.

## Explicit gaps

- Form has not yet passed its parent window to these adapters; no Form changes are included here.
- Form does not currently route TextArea edit-menu requests. Standalone keyboard menu behavior for an initially parent-clipped caret still requires a reference-tested update/prepare ordering decision; this change emits no invented anchor when the caret is clipped.
- Resize-before-input, pointer drag selection over wide/combining graphemes, repeated draw/measure purity assertions, and partial scrollbar-thumb drag under parent clipping are not covered by this Fields increment.
- No application baseline capture or 1:1 application parity is claimed.
