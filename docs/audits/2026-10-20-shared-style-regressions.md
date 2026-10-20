# Shared style and painting regression audit

## Scope and fixed evidence

This audit isolates five shared rendering regressions found while reviewing
TablePro's live picker consumer. The frozen product reference remains
`visual-baseline` source `4169783ae883788186d161af09ba9c481bd84e5c` (tag object
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`); Tuiscotti remains pinned to
`a1ea27f757d75d07b05499276cd932e9f8e067ad`.

Reviewed source points:

- `a848219fd858ed6352fcb403b2cc5d5953083e6f`: TablePro picker migration and
  the introduced shared-style changes.
- `1a947a251ab9b045610b47729bf481419b9fca8d`: shared paired-runner base used
  for integration.
- `99b719d85a0714c88484240dbc9184f110b518d`: isolated fix branch base.
- Read-only suite records: `/tmp/termrock-current-source-review-a848-nextest.log`
  and `/tmp/termrock-current-source-review-1a-nextest.log`.

The review records the source suite at `a848` as 538 passing, 2 failing, and 3
skipped; the two allocation/style regressions below were red there. At `1a`,
the same two regression tests passed. New focused tests below exercise the
remaining explicit-style and opaque-surface contracts.

## Findings and disposition

| Finding | Evidence before fix | Disposition | Evidence after fix |
|---|---|---|---|
| `ScrollRegion` removed explicit scrollbar `BOLD` together with its default bold. | `cargo nextest run -p termrock-navigation every_override_channel_is_forwarded_to_scrollbar` failed at `steps.rs:1402`: track cell lacked `BOLD`. | Fixed in `scroll_region.rs`: strip theme/default bold, then restore bold only when the merged explicit part patch adds it. | The existing Steps regression passes; part-specific explicit style stays distinct from default styling. |
| Modal backdrop allocated a `String` for every cell, and cloned the theme while projecting each modal. | `cargo nextest run -p termrock-runtime warmed_modal_snapshot_projection_performs_zero_allocations` failed at `a848` with 548 allocations. | Fixed in `ui/paint.rs`: compare the borrowed cell symbol and borrow the UI theme/page fields directly. | The warmed modal projection test passes with zero allocations. |
| `FilterList::draw` skipped its container fill whenever incoming cells happened to have non-reset backgrounds, even when `preserve_surface` was false. Explicit container patches were also skipped under an intentionally preserved opaque surface. | Source branch `99b` checked opacity without consulting the preserve policy or the explicit `CONTAINER` patch. | Fixed in `filter_list.rs`: fill non-preserved areas regardless of incoming opacity; preserve an opaque surface by default; apply an explicit container patch even when that preserved surface is opaque. | New tests cover opaque non-preserved cells and explicit danger background on an opaque preserved surface; both pass. |
| Focused `RowUi` selection fill replaced built-in owner role/modifier patches with the selected-row defaults. | Source branch `99b`, `resolve_row_part`, forced the selected patch after resolving the owner patch; existing owner-ink exception only covered `Role::Custom`. | Fixed in `rowui.rs`: merge explicit row-part patch over selection defaults for all style channels. | New focused-fill test verifies built-in `Role::Danger` and underline modifier survive. |
| Picker overlay chrome discarded explicit `CONTAINER` foreground and modifiers while backdrop-mapping the inherited foreground. | Source branch `99b`, `overlay_chrome`, always used mapped backdrop foreground and cleared all modifiers. | Fixed in `support.rs`: map inherited foreground as before, but preserve explicit foreground/clear and modifiers from the resolved container patch. | New picker-chrome test checks resolved overlay foreground and underline modifier on an interior cell. |

No app-level paint workaround or test-specific visual path was added. Picker
title/query geometry and TablePro composition remain outside this change.

## Verification and review status

Before-fix red checks:

- Scrollbar: `cargo nextest run -p termrock-navigation every_override_channel_is_forwarded_to_scrollbar` failed on the missing track `BOLD` modifier.
- Modal projection: `cargo nextest run -p termrock-runtime warmed_modal_snapshot_projection_performs_zero_allocations` failed with 548 allocations on `a848`.
- Opaque FilterList surface: with the new tests temporarily run against the old opacity-only condition, `cargo nextest run -p termrock-navigation -E 'test(explicit_container_patch_reaches_an_opaque_preserved_surface) | test(container_fill_is_not_skipped_for_an_opaque_unpreserved_area)'` failed both checks (danger patch ignored; magenta backing left unchanged).
- Focused RowUi owner patch: with the new test temporarily run against the old selection-default replacement, `cargo nextest run -p termrock-runtime focused_fill_keeps_explicit_builtin_role_and_modifier` failed because highlight blue replaced danger red.
- Picker chrome owner patch: with the new test temporarily run against the old backdrop-only foreground and empty-modifier behavior, `cargo nextest run -p termrock-runtime picker_chrome_keeps_explicit_container_foreground_and_modifier` failed because the explicit danger foreground was replaced by the background sample.

After-fix checks on the isolated branch:

- Focused set: `cargo nextest run -p termrock-runtime -p termrock-navigation -p termrock-overlays -E 'test(every_override_channel_is_forwarded_to_scrollbar) | test(explicit_container_patch_reaches_an_opaque_preserved_surface) | test(container_fill_is_not_skipped_for_an_opaque_unpreserved_area) | test(focused_fill_keeps_explicit_builtin_role_and_modifier) | test(picker_chrome_keeps_explicit_container_foreground_and_modifier) | test(warmed_modal_snapshot_projection_performs_zero_allocations)'` — 6 passed, 0 failed.
- Package suites: `cargo nextest run -p termrock-runtime -p termrock-navigation -p termrock-overlays --status-level fail` — 544 passed, 3 skipped.
- Clippy: `cargo clippy -p termrock-runtime -p termrock-navigation -p termrock-overlays --all-targets -- -D warnings` — passed.
- Formatting: `rustfmt --check --edition 2024` on the five changed Rust files — passed. Workspace `cargo fmt --all --check` remains blocked by existing formatting differences in untouched `crates/showcase-demos/src/pages/chrome.rs`; that file was left unchanged.
- `git diff --check` — passed.
- Independent code review: pending exact-commit review.
- Full application parity and library acceptance: pending; this audit is not an acceptance claim.
