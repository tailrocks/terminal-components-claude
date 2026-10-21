# Library reference token oracles

Reference fixed at tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542` and source commit `4169783ae883788186d161af09ba9c481bd84e5c`.

## Meter faint token

The pinned source assigns `Theme::text_faint` to RGB `(115, 115, 115)`, ANSI 256 index `242`, and `Gray` in ANSI 16 and monochrome profiles (`src/theme.rs:754,940,1009,1077`). `Theme::faint` returns that token (`src/theme.rs:1195-1196`). The Meter uses `text_faint` for both `MeterTone::Unknown` and `StaleStyle::Faint` (`src/widgets/progress.rs:385,408-413`). The current Junie theme therefore assigns both reusable meter roles to `TEXT_FAINT`; its explicit ANSI 256 and Mono palettes use index `242` and `Gray`. ANSI 16 continues through the existing projection from the truecolor token. Custom `MeterTokens` remain authoritative through the existing theme projection path.

Before the change, `low_and_stale_tokens_match_reference_roles_but_custom_colors_remain_authoritative` failed at the stale-role assertion: Junie returned RGB `(77, 77, 77)` while the same-level foreground role returned RGB `(115, 115, 115)`. The test now checks that unknown uses the same faint role and that a custom unknown color remains authoritative at all four color levels. Existing expected values and custom-color assertions remain unchanged.

## Scroll track glyph

The pinned source declares the scrollbar track glyph as `┆` (`src/theme.rs:2429`); `│` is the divider glyph (`src/theme.rs:2433`). The full-track test's only mismatch was its stale test oracle at height `2`, initial offset, row `1`: rendering returned `┆`, while the test expected `│`. The test now expects the frozen scroll-track glyph. Its heights, both end positions, thumb start/length calculation, every-cell comparison, custom caps, pointer endpoints, drag assertions, and typed track-replacement checks remain unchanged.

## Evidence

Candidate red-before commit: `ef1d6b37daca2ca895c6b35c3253ebe3e37f57d6`.

Red-before command:

```text
cargo nextest run --locked -p termrock-feedback -p termrock-viewport -E 'test(low_and_stale_tokens_match_reference_roles_but_custom_colors_remain_authoritative) | test(default_thumb_uses_full_track_including_endpoints_and_tiny_bounds)' --no-fail-fast
```

Result: both selected tests failed for the stale color and stale glyph expectations described above. Full output: `/tmp/library-reference-before.log`.

The test was extended with the unknown-role reference and custom-override assertions. With only unknown set temporarily back to its original Junie literal (`WHITE_30`, ANSI 256 `238`, Mono `DarkGray`), the new assertion failed at TrueColor: RGB `(77, 77, 77)` versus the frozen faint role RGB `(115, 115, 115)`. Output: `/tmp/library-reference-unknown-before.log`.

The first post-fix two-package run passed the scrollbar tests and TrueColor meter check, then exposed the separate ANSI 256 stale/unknown palette value `238` versus `242` (182/183 passed). Updating only those two semantic slots in the explicit ANSI 256 and Mono palettes resolved the remaining levels. Final checks:

```text
cargo nextest run --locked -p termrock-feedback -p termrock-viewport --no-fail-fast
cargo clippy --locked -p termrock-theme -p termrock-feedback -p termrock-viewport --all-targets -- -D warnings
rustfmt --edition 2024 --check crates/termrock-theme/src/builtin/junie.rs crates/termrock-theme/src/palettes.rs crates/termrock-feedback/tests/meter_semantic_tokens.rs crates/termrock-viewport/tests/scrollbar_full_track.rs
git diff --check
```

Results: 183/183 tests passed, clippy passed, rustfmt passed, and diff check passed. Logs: `/tmp/library-reference-after-nextest-final.log`, `/tmp/library-reference-clippy.log`.
