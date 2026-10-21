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

## Independent review and PR #38 synchronization

Independent review `CLEAR_BOUNDED_STAGE_1` covers exact implementation commit
`312b8f38a7d946edddddf640e32db47cbde99e24`; its evidence is preserved in
[`independent-review.json`](evidence/2026-10-21-theme-typed-slots/independent-review.json)
(SHA-256 `46cdc54359662c37ac2edc0c70928a884f306aeae48c53808c00a1930882a2be`).
The reviewer notes there is no public runtime output test for `size_slot`; its
three slot values have unit coverage, while a public runtime size contract is
follow-up work. The review does not clear raw mutation retirement, MenuBar
precedence, application parity, or the full acceptance requirements.

This WIP was synchronized by a normal merge of PR #38 source
`bfefea9ce4b3ab91f846239d5f315534ccc2ff43` into stage-1 commit 312b. That
preserves a fast-forward path back to PR #38. All seven held expectation blobs
in `docs/audits/evidence/2026-10-21-form-dialog-integration/held-expectations.json`
were checked at the merge tree and matched their pinned object IDs exactly;
no candidate expectation was refreshed. The source synchronization itself
does not imply independent review of the incoming PR #38 changes.

After synchronization, `cargo nextest run -p termrock-theme --no-fail-fast`
ran 182 tests: 181 passed, one failed, zero skipped. The failure is
`capability_palettes::all_73_junie_slots_match_reference_capability_output`,
already recorded as red in the PR #38 migration matrix before this merge. It
is an inherited reference-output conflict, not waived or changed here. The
assigned Runtime glyph contract still passes 2/2; strict scoped Clippy,
formatting, and diff checks pass on the synchronized WIP. The review's
182/182 theme result applies to pre-merge 312b only.
