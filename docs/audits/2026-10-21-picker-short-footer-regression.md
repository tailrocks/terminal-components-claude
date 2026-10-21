# Picker short and empty footer regression

The existing zero-footer test used 100 rows. The list repaint covered the
footer location, so that case could pass while the footer still painted over
available result space. Added independent one-result and empty-data cases in
`crates/termrock-overlays/tests/picker_layout.rs`. Each checks both
`footer_rows(0)` and `footer_rows(1)` at truecolor, 256-color, 16-color, and
monochrome levels through the public `Harness`.

The one-result case fails on `ed5bda4abb7873acf6b546ec45b2f35b8fbc1e63`:
with a zero-row footer, `VISIBLE_FOOTER` appears in the spare result row. The
empty-data case passes on that revision; it remains coverage for preserving
empty-state and footer behavior at every color level. Both cases pass on
`6cd63d2270b85dac30c63018070e00ed80510925`.

Validation:

- `cargo nextest run -p termrock-overlays --test picker_layout`: 7 passed,
  0 skipped on `6cd63d2`.
- On `ed5bda4`, `zero_footer_hides_hints_with_one_result_at_every_color_level`
  failed as expected; `zero_footer_hides_hints_with_empty_results_at_every_color_level`
  passed.
- Validation used Rust 1.99.1 and separate target directories for each
  worktree.
