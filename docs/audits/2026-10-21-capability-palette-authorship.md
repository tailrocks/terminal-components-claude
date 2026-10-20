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
