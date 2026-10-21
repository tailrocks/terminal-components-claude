# Showcase false-parity audit (showcase streams)

Date: 2026-10-22. Branch `parity/termrock-visual-baseline` at `7e2597710a` (parent audited: `ef23c3303c`). Lane mirrors the completed Holla (`2026-10-21-holla-plate-removal.md`) and Jackin removal lanes. Worktree: `/private/tmp/termrock-showcase-fp-showcase`.

## Inventory

| Mechanism | Where | Disposition |
| --- | --- | --- |
| `historical_*` replay paint | `crates/showcase-ui/src`, `crates/showcase-demos/src`, `crates/showcase/src`, `crates/showcase-data/src` | already-clean at `ef23c3303c` (0 hits; only the guard test `showcase-demos/tests/no_false_parity.rs` names it). Selectable ANSI replay replaced by live widgets in `3f2fca010e`; editor baseline replay purged in `fb6ed4f839`. |
| Size-keyed `*_plate` / `*_frame` cell dispatchers | none left | already-clean; the last two exact-size screens (`selectable.rs` `wide_page` exact `134x46`, `settings.rs` `narrow_frame`/`compact_frame` exact height equality) were removed at tip in `7e2597710a`. Remaining `wide_page` (`>= 120 && >= 40`) and settings width bands (`<= 59`, `<= 51`) are width-band responsive layout, same policy as the Holla home lane; no exact test size, no cell tables. |
| Fixed `RUNS` cell tables | showcase prod | already-clean (0 hits). |
| `include_str!` / `include_bytes!` of captured ANSI in production | showcase prod | already-clean (0 hits; the only `include_str!` is the guard test reading page sources to assert absence). |
| Screen paths keyed by test identity / fixture labels / exact test sizes / captured clock values | showcase prod | already-clean (no `cfg!(test)`, `is_terminal`, `env::var`, literal size equality, fixed clock or timestamp selectors). |
| Invisible control registration to force cursor output | showcase prod | already-clean (no `set_cursor`/`hide_cursor`/`ESC[` literals/cursor forcing). |
| Selectable delete-confirm ANSI stamp | `crates/showcase-demos/src/pages/selectable.rs` | already-clean: paints the shared `termrock_overlays::dialog::Dialog` (`DELETE_DIALOG`) in the overlay layer. |
| Selectable bulk-mixed ANSI stamp | `crates/showcase-demos/src/pages/selectable.rs` | already-clean: live chips/fields; the `bulk_mixed` flows are PTY scenarios, not painted cells. |

No production replay mechanism remains in the showcase streams. Nothing was removed by this audit; `7e2597710a` (parallel session, 2026-10-22 05:25 +0700) removed the final exact-size checks and was verified here.

## Byte-neutrality proof for `7e2597710a` (paired before/after)

Paired captures on the branch harness (`crates/visual-harness` `visual_baseline`, tuiscotti), same base build method, `--run-ignored only`, filter `showcase::flows::selectable` / `showcase::flows::settings` / `showcase::sections::settings` / `showcase::elements::selectable` / `showcase::overlays::selectable`:

- Parent `ef23c3303c` binary sha256 `d79ad9372fb437114edc5e63856acd920f36b118`; tip `7e2597710a` binary sha256 `c4db38411207774b2dad47b481c3539a8607602a`. Logs and captured frames: `/private/tmp/fp-bins/` (`parent-run.log`, `tip-run.log`, `actual-parent/`, `actual-tip/`).
- 473 scenarios (5 sizes x 5 color levels): 423 passed / 50 failed on BOTH builds; the 50 failing case ids are byte-identical sets (`diff` of sorted id lists: empty). The 50 are the recorded review-required showcase set (e.g. `cells-differ` on `flows/selectable/delete_confirm/72x20/truecolor`), pre-existing at the parent, not touched by this lane. No expected output was edited and no BLESS was run.
- Captured frames: `diff -rq` of the two `actual` trees shows zero differing frame artifacts (`.ansi` / `.frame.json` / `.png` / `.txt`); only `*.manifest.json` differ, exclusively `run_id` and `written_unix_ns`.

Byte-neutral: accepted. The frozen-oracle `tools/frozen_baseline.py compare` route was attempted first but aborted read-only: the prepared root at `/private/tmp/termrock-backdrop-adj-20261021` cannot be cloned without cargo rebuilding (and thereby changing) the frozen baseline binaries against the recorded `build.json` hashes, so the branch-harness paired capture above is the proof of record.

## Other gate results at tip (all confirmed pre-existing by rerunning the parent)

- `cargo nextest run -p showcase-demos -p showcase-ui -p showcase`: 143 run, 138 passed, 5 failed. `keyboard_navigation_between_pages`, `hidden_job_ages_but_publishes_completion_only_on_eligible_tick`, `shell_chrome_hit_regions_are_unchanged` fail identically at the parent; `all_page_headings_match_the_reference_capture_matrix` and `showcase_visual_baseline` are the independently attributed pre-existing red set (`Overview 80 24 junie truecolor` digest `80780cbbac55284d` baseline vs `8b54edf362c1d6fc` actual, identical at parent).
- `cargo nextest run -p termrock-controls --test button_contract --test showcase_buttons`: 19/23 `button_contract` passed; `showcase_buttons` 22/22. The 4 failures are adjudicated below.

## STOPPED for adjudication: Button focus-visible while pressed

`37f55475f7` ("fix(conformance): achieve 100% control-state and baseline parity") added `&& !style_flags.contains(StateFlags::PRESSED)` to the `ButtonFocusVisibility::Always` promotion in `crates/termrock-controls/src/button.rs` (draw), so a pressed focused button no longer resolves `Part::GUTTER` to `GlyphRole::FocusBar`. The button contract tests still encode the macOS-convention behavior (focus indicator stays while pressed; `focus_theme()` maps `FOCUS_VISIBLE` to `FocusBar`):

- `pointer_focus_visibility_obeys_always_and_keyboard_only_policies` (`button_contract.rs:580`): `Set(RuleQuiet)` actual vs `Set(FocusBar)` expected.
- `always_policy_does_not_promote_focus_in_an_inert_reference_scope` (`:735`): `Set(FocusBar)` vs `Set(RuleQuiet)`.
- `focus_visibility_policies_hold_for_every_button_variant_and_color_level`, `owner_wide_reference_styles_every_part_without_live_registration_or_cursor`: same root cause (gutter/foreground resolved without the promoted flag).

Both results are recorded, neither side was edited:

- At tip `7e2597710a`: `button_contract` 15 passed / 4 failed; conformance `control_states` suite green (it was made green by the same commit, which also reversed the W02-01 expectation text from "disabled must not change the focused gutter" to "disabled must suppress the focused gutter").
- At parent-of-`37f55475f7` semantics (the pre-`37f55475f7` behavior): the contract tests pass (the `2b3` review at `docs/audits/2026-10-21-button-2b3-review.json` records `default_pointer_focus` `RuleQuiet` -> `FocusBar` as red-before/green-after), while the conformance control-state parity that `37f55475f7` chased would regress.

The owner decision owed: whether the pressed focused button keeps its `▎` focus gutter (contract tests, macOS HIG: focus indicator stays while pressed) or yields to the pressed state (conformance expectation authored by `37f55475f7`). Fixing either side changes `crates/termrock-conformance/tests/baselines/components.txt` (2120 lines changed by that commit) or the contract tests, so per the expectation-protection rule this lane stopped instead of picking a side. Note the `2b3` P2 `BUTTON-KEYBOARD-ONLY-FIRST-KEY-LOST` finding appears fixed at tip: bindings publish from the pre-policy `live` flags (`button.rs` "Bindings consume the actual runtime focus state").

## Pins unchanged

Baseline tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, baseline source `4169783ae883788186d161af09ba9c481bd84e5c`, tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad`. No snapshots, expected stores, or harness gates were modified.
