# Execution record: danger-token tag-wins (D-009/D-010)

Date: 2026-10-22. Branch: `wip/danger-token-tag-wins-20261022`, cut from
`parity/termrock-visual-baseline` tip `2b99f81f67`. Decision basis:
coordinator decision D-009/D-010 (2026-10-22) — the frozen
`visual-baseline` tag source `4169783ae883788186d161af09ba9c481bd84e5c`
is the oracle; **tag-wins**. This executes the danger-token half of the
open owner decision package
`docs/audits/2026-10-23-danger-token-reference-decision.md` (§1 option A,
sub-decisions 4.1 and 4). The owner-decision document was executed by the
coordinator per autonomous mode; this record is the implementation
receipt, not a new owner decision.

## Scope

Only slots the frozen tag source actually defines. The tag's danger
family (`git show 4169783ae:src/theme.rs`): truecolor `RED`
(`:521`, `#f05d5d` — already restored by `740374edb2`), `RED_SOFT`
(`:531`, `#e8a0a0`, mapped to `error_soft` at `:747`), 256-tier
`error = i(203)` / `error_strong = i(210)` / `error_soft = i(174)`
(`:956`–`:964`), 16-tier `error_soft = LightRed` (`:1034`), mono
`error_soft = White` (`:1096`). `paper.rs` was left untouched: its
`danger_soft` is `Color::Reset` and the frozen tag does not define a
paper palette.

## Value changes (old → new)

| Slot | Old | New | Tag evidence |
|---|---|---|---|
| `junie.rs` `RED_SOFT` (truecolor `danger_soft`) | `0xd98a8a` | `0xe8a0a0` | tag `src/theme.rs:531` |
| `palettes.rs` `junie_256` `danger_soft` | `Indexed(181)` | `Indexed(174)` | tag `src/theme.rs:964` (`error_soft = i(174)`) |
| `palettes.rs` `junie_mono` `danger_soft` | `Color::Gray` | `Color::White` | tag `src/theme.rs:1096` (`error_soft = White`) |
| `empty_style_inheritance.rs:289` fingerprint | `312_677_554_172_290_677` | `3_030_540_740_827_194_175` | derived recipe output after the above |

No change was needed for the 16 tier: the generic `nearest_16` ladder
already maps both the old `0xd98a8a` and the tag's `0xe8a0a0` to
`LightRed`, which is the tag's authored `apply_16` value (`:1034`).

## 256-tier danger index `Indexed(203)` — ratified, no code change

The downgrade ladder already produces 203 from the frozen-source recipe:
`baseline_ansi256` (`crates/termrock-theme/src/downgrade.rs:90-91`) maps
`(0xf0, 0x5d, 0x5d)` → `203` (the frozen `apply_256` `error` slot,
`#ff5f5f`, deliberately over the cube-nearest 210), and the authored
`junie_256` table plus the `all_73_junie_slots_match_reference_capability_output`
overrides (capability_palettes.rs) pin `danger`, `syntax.invalid`,
`syntax.diagnostic_error`, `meter.high` and `meter.series[2]` to
`Indexed(203)` — the state authored by `1c17d8f262`. Verified, not
changed.

## Red-first evidence

Tests were moved to the tag values first and run against the unchanged
sources (all red), then the sources were changed (all green):

- `termrock-theme::capability_palettes::all_73_junie_slots_match_reference_capability_output`
  — red: Ansi256 authored `danger_soft: Indexed(181)` vs expected
  `Indexed(174)`.
- `termrock-theme::capability_palettes::danger_family_matches_frozen_tag_reference`
  (new regression test, asserts the whole danger family through resolved
  per-tier styles plus the ladder recipe
  `downgrade_color(Rgb(0xf0, 0x5d, 0x5d), Ansi256) == Indexed(203)`) —
  red: `Rgb(217, 138, 138)` vs `Rgb(232, 160, 160)`.
- `termrock-theme::builtin::tests::junie_tokens_equal_the_legacy_palette_exactly`
  — red: `Rgb(217, 138, 138)` vs `Rgb(232, 160, 160)`.

The fingerprint test `standalone_empty_cells_match_pre_inheritance_reference`
was already red at the base commit (pre-existing, recorded in the
reference-decision package §2.3); it was re-pinned to the post-change
recipe output `3_030_540_740_827_194_175`, which is a derived-recipe
re-pin, not a bless.

## Gate

- `cargo nextest run -p termrock-theme`: 197 tests, 196 passed, 1
  failed — the one failure is the pre-existing, unrelated
  `builtin::tests::a_hovered_disabled_part_keeps_every_slot_the_disabled_rule_writes`
  (BUTTON/SUBTLE disabled-bg-vs-hover allowlist, red at the base commit
  before any change here; belongs to the hover family, not danger). The
  fingerprint is green after the re-pin.
- `cargo clippy --all-targets` and `cargo fmt --check` on the exact HEAD:
  clean.

## Not done here (stays with its own decision)

- No visual baseline blessed; no expected digest edited other than the
  fingerprint above. The held `jackin.txt` digest, the danger-scoped
  `components.txt` lines and the `baselines/tuiscotti-v1` corpus
  re-approval remain with the reference-decision package §1.
- The stale `DISABLED_BG_LOST_TO_HOVER` allowlist entry surfaced by the
  pre-existing red is recorded here and left to the hover-family owner.
- Downstream rendered-output suites outside `-p termrock-theme` see the
  new `danger_soft` inks and are expected to move only through their own
  scoped re-pin decisions.
