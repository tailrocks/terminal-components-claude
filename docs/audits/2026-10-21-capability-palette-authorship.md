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

## Shared grid edge fade

The frozen reference's grid uses the shared scroll-edge fade only when content is hidden above or below the viewport. Its policy preserves the cursor row, reversed cells, and cells whose background differs from the grid container. The candidate `Grid::draw` now applies the existing shared `Ui::scroll_edges` policy to its resolved content viewport after drawing rows. The TablePro-only `paint_bottom_fade` and indexed edge repaint were removed instead of moving the row-specific repaint into the library.

`termrock-grid::grid::tests::overflowing_grid_fades_only_real_scroll_edges` checks an overflowing grid's bottom edge and confirms a fitting grid does not fade its visible content. The test failed when the shared `Grid::draw` call was temporarily removed (the overflowing edge returned from RGB 140 to RGB 255), then passed after restoration. Existing `termrock-runtime::ui::paint` tests cover cursor protection, reversed cells, container-background eligibility, and edge selection.

The focused `cargo nextest run --locked -p termrock-grid` run passed 125/125. The broader `cargo nextest run --locked -p tablepro-ui` run compiled after adding its required snapshot paths to this sparse worktree, then failed 13/180 tests (167 passed). Failures include existing TablePro interaction/duplicate-ID/performance cases and the stored visual baseline case; this is not a passing product comparison. The paired frozen-binary run remains necessary to decide exact application parity.

The existing application-level `draw_painted_grid` / `paint_scrolled_cells` path still repaints the shared Grid and remains architecture debt. It is recorded here as unresolved rather than expanded into this change without a component-contract migration map.
