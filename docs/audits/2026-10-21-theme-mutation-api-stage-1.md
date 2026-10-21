# Theme mutation API migration: stage 1

This increment implements the first stage of the independent
`/private/tmp/theme-mutation-api-decision-20261021/decision.md` decision.
The decision selects a pre-1.0 migration from mutable access to installed
recipes toward typed authoring APIs. Its reason is that writes through a
retained `&mut Recipe` cannot be distinguished from no-op access or
same-value writes without guessing from equality.

## This stage

`PartEdit::glyph_slot` and `PartEdit::size_slot` now accept `Slot::Inherit`,
`Slot::Set`, and `Slot::Clear` explicitly. Existing `glyph` and `size` methods
delegate to those methods and retain their `Set` behavior. Runtime glyph
fixtures in `rowui.rs` and `rowui_glyph_contract.rs` now use public
`Theme::define_family` / `RecipeEdit::part` / `PartEdit::glyph_slot`, so the
integration test compiles that consumer path against the exported API and
continues to check the real rendered cells.

## Compatibility boundary

No live `Recipes` mutation API is removed or deprecated in this increment.
`Recipes::get_mut` and `iter_mut` remain public pending the separate,
explicitly reviewed stage 2 boundary migration. Detached `Recipe`,
`PartRecipe`, `PartMap`, and `StylePatch` values remain unchanged. This stage
does not claim to resolve the installed-recipe mutation ambiguity, the
MenuBar precedence HOLD, or application parity.

## Verification

- `CARGO_TARGET_DIR=/private/tmp/cargo-target-theme-slots-stage1 CARGO_BUILD_JOBS=4 cargo nextest run -p termrock-theme` — 182 passed.
- `CARGO_TARGET_DIR=/private/tmp/cargo-target-theme-slots-stage1 CARGO_BUILD_JOBS=4 cargo nextest run -p termrock-runtime --test rowui_glyph_contract` — 2 passed. The integration test is a separate consumer using only the public typed API and checks rendered cells for Inherit, Set, and Clear.
- `CARGO_TARGET_DIR=/private/tmp/cargo-target-theme-slots-stage1 CARGO_BUILD_JOBS=4 cargo clippy -p termrock-theme -p termrock-runtime --all-targets -- -D warnings` — passed.
- `cargo fmt --check` — passed.

No expected output or frozen baseline artifacts were changed.
