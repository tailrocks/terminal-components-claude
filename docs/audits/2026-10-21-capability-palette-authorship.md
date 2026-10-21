# Capability palette authorship and shared grid fade

## Decision

Generic RGB conversion stays nearest-color based. Removed the five exact-RGB exceptions from `termrock-theme::downgrade_color`; custom colors such as `#3a3a3a`, `#737373`, `#474747`, `#808080`, and `#173f1c` now follow the generic xterm palette conversion.

Junie semantic colors retain their authored capability values in `CapabilityPalettes`. The fixed `visual-baseline` source commit `4169783ae883788186d161af09ba9c481bd84e5c`, `src/theme.rs:921-962`, assigns the ANSI-256 `text_on_accent` index 232 and `border_strong` index 242. The same function assigns `disabled` index 239. The capability comparison now preserves those three semantic assignments while deriving all remaining RGB values through the generic oracle.

## Evidence

The frozen source is unchanged. The comparison still covers all 73 Junie semantic color slots at truecolor, ANSI-256, ANSI-16, and monochrome. Custom-color assertions remain in the downgrade unit suite and now prove nearest-color behavior for the exact values that had been globally special-cased.

Focused verification on PR #38 source `5c0075a09eec51b805f15d79ad945d4fb51fb5bf`:

```text
CARGO_TARGET_DIR=/private/tmp/termrock-pr38-theme-grid-20261021-target cargo nextest run --locked -p termrock-theme -E 'test(downgrade_is_deterministic_per_level) | test(all_73_junie_slots_match_reference_capability_output)' --no-fail-fast
2 tests passed; 121 skipped by the expression.
```

This verifies palette ownership and custom conversion only. It does not establish application parity or the full visual acceptance gate.

## Split seam semantic recipe

The table workbench's extra vertical seam now asks the shared `Family::SPLIT` / `Part::SEAM` resolver for its resting style instead of using `Role::DisabledFg`. Junie's seam ladder is supplied by optional per-color-level family recipes: truecolor RGB `(77,77,77)` / `(128,128,128)`, ANSI-256 indices 239 / 245, ANSI-16 DarkGray / Gray, and Mono DarkGray / Gray before monochrome conversion. Pressed remains the existing accent recipe state. Custom family edits take precedence per channel, including explicit Clear; changing disabled or border tokens does not alias the seam. `StyleCache` keys now include color level and the recipe mutation revision so a changed profile cannot reuse a stale patch.

Focused regression tests cover all four levels, rest/hover/pressed resolution, disabled-token independence, base and state customization, Clear, unrelated-channel customization, ordinary family overrides, and cache identity. The complete `termrock-theme` suite passed 126/126 after this change. `tablepro-ui` remains 167/180 with the same 13 failing cases observed at preservation tip `148c32c1` before the seam-profile/app-style edit; this narrows the seam change as a cause but does not attribute those failures against the fixed 5c source. Its visual-baseline test still fails and requires a paired frozen-binary comparison. The public `define_family_for_level` API is additive and keeps `ColorTokens` and `Role` unchanged; add it to the public API catalog and Showcase coverage before considering the library gate complete.

Independent API review found two stale-cache cases: public `Recipes::iter_mut` did not advance recipe identity, and separately edited clones could have equal local revisions. The cache revision is now globally unique on every mutable recipe access (including `iter_mut`), with regressions for both paths. After these fixes, `cargo nextest run --locked -p termrock-theme` passed 128/128.

Follow-up comparison against frozen source `4169783ae883788186d161af09ba9c481bd84e5c:src/theme.rs` at `apply_mono` showed its seam ladder is explicitly `DarkGray / Gray / White`. The capability-specific Mono recipe now preserves its authored custom colors through final style binding, while ordinary `Role::Custom` colors still use generic color downgrade. Coverage checks all four Junie color levels, a custom Mono profile through `Theme::downgrade`, and an ordinary custom RGB channel through the generic path. The updated full theme suite passes 129/129. Independent review of this follow-up is pending; it is not application parity approval.

The first Mono follow-up passed only `Theme::resolve` and did not preserve author provenance through the real cached `Ui::style` path; an explicit same-value `Role::Custom(DarkGray)` override also incorrectly inherited the profile bypass. The resolver cache now carries private per-channel authored-profile provenance to the binding call, and explicit overlays/overrides/local patches clear that provenance without comparing color values. The real `SplitPane` renderer verifies rest-cell colors across all four capability levels and the same-value explicit Custom override. On the corrected follow-up, `cargo nextest run --locked -p termrock-theme -p termrock-runtime -p termrock-controls` passes 546 tests, with 3 skipped.
## Shared grid edge fade

The frozen reference's grid uses the shared scroll-edge fade only when content is hidden above or below the viewport. Its policy preserves the cursor row, reversed cells, and cells whose background differs from the grid container. The candidate `Grid::draw` now applies the existing shared `Ui::scroll_edges` policy to its resolved content viewport after drawing rows. The TablePro-only `paint_bottom_fade` and indexed edge repaint were removed instead of moving the row-specific repaint into the library.

`termrock-grid::grid::tests::overflowing_grid_fades_only_real_scroll_edges` checks an overflowing grid's bottom edge and confirms a fitting grid does not fade its visible content. The test failed when the shared `Grid::draw` call was temporarily removed (the overflowing edge returned from RGB 140 to RGB 255), then passed after restoration. Existing `termrock-runtime::ui::paint` tests cover cursor protection, reversed cells, container-background eligibility, and edge selection.

The focused `cargo nextest run --locked -p termrock-grid` run passed 125/125. The broader `cargo nextest run --locked -p tablepro-ui` run compiled after adding its required snapshot paths to this sparse worktree, then failed 13/180 tests (167 passed). The observed failures include TablePro interaction, duplicate-ID, performance, and stored visual-baseline cases; attribution against the pre-change tree has not been established. This is not a passing product comparison. The paired frozen-binary run remains necessary to decide exact application parity.

The existing application-level `draw_painted_grid` / `paint_scrolled_cells` path still repaints the shared Grid and remains architecture debt. Because that repaint follows `Grid::draw`, it can overwrite the new shared cell fade in TablePro. The shared Grid behavior is covered for direct consumers, but this change does not claim the application path now displays the same edge fade. Removal is recorded as unresolved rather than expanded into this change without a component-contract migration map.

## Split seam semantic recipe

The table workbench's extra vertical seam now asks the shared `Family::SPLIT` / `Part::SEAM` resolver for its resting style instead of using `Role::DisabledFg`. Junie's seam ladder is supplied by optional per-color-level family recipes: truecolor RGB `(77,77,77)` / `(128,128,128)`, ANSI-256 indices 239 / 245, ANSI-16 DarkGray / Gray, and Mono DarkGray / Gray before monochrome conversion. Pressed remains the existing accent recipe state. Custom family edits take precedence per channel, including explicit Clear; changing disabled or border tokens does not alias the seam. `StyleCache` keys now include color level and the recipe mutation revision so a changed profile cannot reuse a stale patch.

Focused regression tests cover all four levels, rest/hover/pressed resolution, disabled-token independence, base and state customization, Clear, unrelated-channel customization, ordinary family overrides, and cache identity. The complete `termrock-theme` suite passed 126/126 after this change. `tablepro-ui` remains 167/180 with the same 13 failing cases observed at preservation tip `148c32c1` before the seam-profile/app-style edit; this narrows the seam change as a cause but does not attribute those failures against the fixed 5c source. Its visual-baseline test still fails and requires a paired frozen-binary comparison. The public `define_family_for_level` API is additive and keeps `ColorTokens` and `Role` unchanged; add it to the public API catalog and Showcase coverage before considering the library gate complete.
