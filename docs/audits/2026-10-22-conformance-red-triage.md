# Conformance red triage (`termrock-conformance`): per-family verdicts

Date: 2026-10-22. Branch: `parity/termrock-visual-baseline`. Read-only
triage plus bisect spot checks in a detached worktree; no product source or
expected-output change is part of this record.

Question: the suite reports roughly 1200 FAIL lines with the
`local_override_does_not_mutate_the_theme`, tree and too_small families.
For each failure family, is it (a) pre-existing on the merge-base/original
source, (b) caused by recent parity commits, or (c) environmental?

## Verdict summary

| Family (test binary) | Verdict | Evidence |
| --- | --- | --- |
| `render::components::*` digest families — `render_components` 232 FAIL at `bdf62e3be` / 143 FAIL at `a5bb410d4`, including tree and too_small | regression-caused-by-`a848219fd` (+ repair chain `ea4a92429`, `3ba054515`; later frame-matching commits extended the drift; active restoration stream is shrinking it) | Bisect table below: parent `1a947a251` green 332/332, `a848219fd` red 181/332 |
| `conformance` binary: the standing 24 failures, including the only `local_override_does_not_mutate_the_theme` red (`help_overlay`) | pre-existing (proven at parent `ad86aaca1` by the earlier receipt; confirmed unchanged at both tips measured today) | `docs/audits/evidence/2026-10-21-conformance-driver-corrections/ad86-revalidation/parent-24-probe-receipt.md`: 24 run, 0 passed, 24 failed at `ad86aaca1`; exact-name match 24/24 with today's run |
| `control_states`: `w02_button_variants_all_states_and_focus_gutter`, `w08_input_masked_clicks_follow_display`, `w13_filter_backspace_removes_grapheme` | regression-caused-by the same post-`1a947a251` window (exact commit not bisected per test) | 107/107 PASS at `1a947a251`; 123/126 pass, 3 FAIL at `a5bb410d4` |
| `render`, `cursor_provenance`, `parity_burndown`, `button_busy_frames` | green | 0 failures at `a5bb410d4` |
| Named suspects: backdrop regeneration (`ca867254a`) and the holla/jackin bypass removals | exonerated for these families | `ca867254a` touches only tuiscotti snapshot baselines (`.ansi`/JSON) and admission records; the bypass-removal commits touch `crates/{holla,jackin}*, showcase-demos` app layers. Neither path feeds `components.txt` digests, which are source-rendered; `components.txt` is untouched since `032e9faa3` (2026-10-15) |

## Environment and commands

Detached worktree at `/private/tmp/termrock-conformance-parity`, own target
directory, `cargo nextest run --locked -p termrock-conformance` only. All
counts below are from that setup; nothing was blessed and no expected file
was edited.

- `cargo nextest run --locked -p termrock-conformance --test conformance --no-fail-fast`
  - `bdf62e3be`: 935 run, 911 passed, 24 failed (16.7s)
  - `a5bb410d4`: 935 run, 911 passed, 24 failed (228s), identical 24 names
- `cargo nextest run --locked -p termrock-conformance --test render_components --no-fail-fast`
  - `032e9faa3` (2026-10-15 crate split): 332/332 PASS
  - `a97e351df` (2026-10-18 22:24): 332/332 PASS
  - `9e788e545` (2026-10-20 15:53): 332/332 PASS
  - `1a947a251` (2026-10-20 19:16): 332/332 PASS
  - `a848219fd` (2026-10-20 19:34): 151 pass, 181 FAIL
  - `600113898` (2026-10-20 20:23): 128 pass, 204 FAIL
  - `1a073c03f` (2026-10-20 23:52): 128 pass, 204 FAIL
  - `6d1587e16` (2026-10-21 01:54): 119 pass, 213 FAIL
  - `bdf62e3be` (2026-10-22 00:18): 100 pass, 232 FAIL
  - `a5bb410d4` (current tip): 189 pass, 143 FAIL — tree green again after
    `b4b1a3346` ("restore the frozen muted and faint inks and the scrollb…")
- Remaining native binaries at `a5bb410d4`
  (`--test control_states --test render --test cursor_provenance
  --test parity_burndown --test button_busy_frames`):
  126 run, 123 passed, 3 failed, 2 skipped (the 3 are the `control_states`
  cases listed above; the other four binaries are green)

The "~1200 FAIL lines" figure is line-count, not test count: nextest prints
every failure twice (progress line plus summary block), so the native
binaries at `bdf62e3be` produce about 256 failures ≈ 510 FAIL lines, and the
rest of the earlier figure comes from the visual-baseline PTY binaries and
the earlier, larger render_components red.

## Bisect proof for the digest families

First-parent chain, `cargo nextest run --locked -p termrock-conformance
--test render_components --no-fail-fast` per commit:

| Commit | Time (UTC+7) | Result |
| --- | --- | --- |
| `a97e351df` | 10-18 22:24 | 332 pass, 0 fail |
| `9e788e545` | 10-20 15:53 | 332 pass, 0 fail |
| `1a947a251` | 10-20 19:16 | 332 pass, 0 fail |
| `a848219fd` | 10-20 19:34 | 151 pass, 181 fail |
| `600113898` | 10-20 20:23 | 128 pass, 204 fail |
| `6d1587e16` | 10-21 01:54 | 119 pass, 213 fail |
| `bdf62e3be` | 10-22 00:18 | 100 pass, 232 fail |

`a848219fd` ("fix: match tablepro quick-open and tab list to the baseline")
is the first red; its parent `1a947a251` is the last green. It changed
shared rendering far beyond its TablePro scope: junie fg-ladder tokens and
scrollbar glyphs (`crates/termrock-theme/src/builtin/junie.rs`), recipes for
tabs, scrollbar track/thumb, picker title/label/icon/meta/query
(`crates/termrock-theme/src/builtin/mod.rs`), plus
`termrock-runtime` `rowui.rs`/`scroll_region.rs`/`support.rs`/`ui/paint.rs`,
`termrock-navigation` `item.rs`/`filter_list.rs` and
`termrock-overlays/src/picker.rs`. Its own audit
(`docs/audits/2026-10-20-shared-style-regressions.md`, as of `3ba054515`)
records that the shared-style changes were introduced at `a848219fd` and
partially repaired by `94a1215e2` (not first-parent), `ea4a92429` and
`3ba054515`; the repair chain left render_components at 204 red (worse than
the 181 it inherited) while fixing the focused contracts it targeted.

Family-level attribution:

- At `a848219fd` (181 red): brand, chip_bar, code_editor, completion,
  context_menu, diff_view, empty, field, filter_list, grid, hint_bar,
  key_hint, list, nav_list, picker, picker_chain, progress_bar, radio_group,
  scroll_region, select, steps, tabs, text_area, text_input, text_viewport,
  toggle, too_small, tree.
- Added by `600113898`..`1a073c03f` (204): button, dialog, help_overlay.
- Added by later commits through `bdf62e3be` (232): split_pane, menu_bar,
  form (+ one extra state in some families).

Why no fix landed from this lane: the drift is not one hunk. Reverting the
theme recipe files alone breaks the build (`set_mono_rules` is private at
`1a947a251`; the provenance machinery depends on it), and reverting
`rowui.rs`/`scroll_region.rs`/`support.rs`/`ui/paint.rs`/`item.rs`/
`filter_list.rs` to `1a947a251` fails to compile
(`Ui::patch_style`, `Ui::modal_backdrop`, `scroll_bg_counts`,
`scroll_bg_order` are used by code that landed after the repair chain).
Measured shared-layer divergence `1a947a251..a5bb410d4` is 49 files,
+20,896/−1,401. The correct fix is exactly what the standing decision
D-008 (`docs/architecture/termrock-migration-record.md`) already mandates —
"the product differences remain work to fix, not output to bless", no
`BLESS=1`, no edit to `components.txt` — and it is in progress on this
branch: `b4b1a3346` recovered 89 digests (232 → 143) between the two tip
measurements above. Pushing a parallel rewrite of the same files from this
lane would collide with that active stream.

## The 24 `conformance` failures

Unchanged set at both tips (names): completion/dialog/field/filter_list/
form/help_overlay/picker/text_area/text_input `bindings_match_handled_keys`
consumed-key cases, help_overlay family (9 tests, activation/overlay
plumbing), completion/grid/scroll_region `mono_states_are_distinguishable`,
`grid_error_is_real_cell_decor_and_mono_distinct`,
`registry::declared_parts_are_the_parts_actually_styled`,
`filter_list::hover_does_not_steal_focus`,
`picker::hover_does_not_steal_focus`,
`picker::wheel_at_boundary_is_consumed_without_repaint`, and
`help_overlay::local_override_does_not_mutate_the_theme`.

The `help_overlay::local_override_does_not_mutate_the_theme` red is not a
theme mutation. The driver's semantic theme and fingerprint assertions pass;
the failing assertion is "the instance patch had no effect": with the
patched digest equal to the plain digest, `resolved()` for the HelpOverlay
part returns identical `Resolved` values with and without the Warning fg
instance patch (`driver.rs:717`). The mutation-detection hardening
(`9ba5e96d8`, `04fd52fc8`) changed which assertion surfaces first; the
24-name set itself predates both commits and is proven pre-existing at
`ad86aaca1` by the `ad86-revalidation` receipt (24 run, 24 failed there,
24/24 exact-name match with the candidate log).

## Disposition

- No expected output was regenerated, no `BLESS=1`, no assertion weakened.
- The digest families stay open as product work owned by the active
  theme/rendering restoration stream (D-008); this record supplies the
  attribution it needs: `a848219fd` is the regression-introducing commit,
  the repair chain `ea4a92429`+`3ba054515` did not restore the digests, and
  every family red at `a848219fd` is red for that reason, not because of
  the backdrop regeneration or the holla/jackin bypass removals.
- The 24 `conformance` failures remain recorded as pre-existing (parent
  receipt). The three `control_states` failures are green at `1a947a251`
  (107/107 there; the binary has since grown from 107 to 126 tests) and red
  at the tip, so they belong to the same post-`1a947a251` regression window
  as the digest families; per-test commit attribution was not run.
