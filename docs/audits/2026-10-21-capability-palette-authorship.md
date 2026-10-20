# Capability palette authorship correction

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
