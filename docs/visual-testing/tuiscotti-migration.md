# tuiscotti migration status

The visual baseline suite (`tests/visual_baseline/`) runs on
[tuiscotti](https://github.com/tailrocks/tuiscotti) `main`, pinned in
`Cargo.toml` to `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274`. Its predecessor
`tuisnap` (`donbeave/tui-snap` at `2d43458`) is gone from the repository.

## What changed

- **Driver.** Every capture is tuiscotti Workflow 2:
  `tui::Tui::new([CARGO_BIN_EXE_<bin>, args…])` with an explicit
  environment (`env_clear(true)`, `TERM=xterm-256color`,
  `COLORTERM=truecolor`, `LANG`/`LC_ALL=C.UTF-8`, `TZ=UTC`,
  `SHELL=/bin/sh`, a minimal `PATH`, `HOLLA_NO_HISTORY=1`, `NO_COLOR=1`
  only for `nocolor`, and an emptied per-scenario
  `target/tuiscotti/home/<name>` as both `HOME` and cwd; the default-run
  check `isolation::env_isolation` proves nothing else leaks). Readiness is a
  content wait (a text `Locator` on physical rows), then
  `wait_stable_quiet(400 ms)`, then `render::frame_from_screen`. Argv,
  needles, send steps and timeouts are unchanged.
- **Committed format.** One file per scenario:
  `snapshots/<app>/<sub_group>/<cols>x<rows>/<palette>.ansi`. It is
  tuiscotti's `render::ansi_dump` plus one cursor trailer
  (`ESC[?25l` when hidden, else `ESC[?25h ESC[y;xH ESC[Ps q`), because
  `ansi_dump` alone drops the caret. `.txt`, `.png` and `.html` are no
  longer committed. The set is the single constant `COMMITTED` in
  `tests/visual_baseline/support.rs`.
- **Gate.** A local gate replaces `GroupedStore::check_with`, which cannot
  gate a subset. Every check writes `target/tuiscotti/actual/<name>.ansi`
  and `.frame.json`. A mismatch also writes actual PNG/HTML, expected
  PNG/HTML rebuilt from the approved `.ansi`
  (`target/tuiscotti/expected/`) and a diff PNG
  (`target/tuiscotti/diff/`).
- **Approval.** `cargo xtask visual accept '<nextest filterset>'` copies
  the reviewed actual `.ansi` of every matching scenario into `snapshots/`
  and removes that scenario's legacy files; it refuses to run when `CI` is
  set. It replaced the `accept_actuals` ignored test and its
  `VISUAL_ACCEPT` variable.
- **One test per scenario.** The suite is a `harness = false`
  (libtest-mimic) binary. Scenarios are typed table rows
  (`tests/visual_baseline/scenarios/`), expanded by `MATRIX` into one test
  per size and colour, named `<app>::<group>::<leaf>…::<cols>x<rows>::<color>`.
  Every one of the 7,625 stored names is unchanged and resolves to the
  identical binary, argv, geometry, colour, boot needle, steps and timeout
  as the hand-written `Case::new` roots it replaced (both harnesses dumped
  every combo; the diff was empty).
- **Terminal modes.** No snapshot format records them, so
  `modes::*_boot_sets_terminal_modes` asserts 1049, 1003, 1006 and 2004 on
  each app's live emulator state.

## Re-approval status

A scenario is re-approved when its approved files are exactly the
committed set (`support::is_reapproved`). Only re-approved scenarios run
in the gate (`--run-ignored only -E 'binary(visual_baseline)'`). Every
other scenario still holds tuisnap's four files, or has no approval yet,
is skipped, and runs only with `VISUAL_FULL=1`, where it fails until it is
approved. `store_integrity` accepts either file set per stored scenario,
requires every non-pending scenario, and rejects files outside the tables.
`cargo xtask visual list` prints the state of each scenario.

| Group | Scenarios | Status |
|---|---|---|
| `showcase/pages` (18 legacy pages + `selectable`) | 475 | re-approved (pilot): `chrome` for the Shift+F10 hint (abdf895c2), then every page for the tree sidebar, breadcrumb and footer hints (e48c7f75c, ea99e01ec, e0bf4a14b, 3ec122eb2, 85cec55af); gate green on 961abe9b4 |
| `showcase/flows/selectable` (new) | 50 | re-approved with the pilot; gate green on 961abe9b4 |
| every other stored group | 7,100 | pending re-approval (tuisnap files) |
| new coverage rows (`.pending()`: 29 element, 7 overlay and 10 resize roots; 92 overlay dismissals) | 3,450 | no approval yet |
| narrow tier (`40x12`, `60x18`) | 1,056 | no approval yet |

Breakage found while adding coverage, fixed in the tables (the app copy is
unchanged): `jackin/cockpit/cancel_confirm` opens with `x` since
c3515f652 (one meaning per letter), so its `c` step never reached `Cancel
the launch?`; `holla/flows/trust/prompt` ran under reduced motion, where
the screen behind the prompt kept animating and a dismissed frame never
settled. It now runs at `--motion paused --frame 40`, waits for the prompt
title, and has its `-dismissed-esc` / `-dismissed-outside` rows back
(pending; two runs were byte-identical).

The determinism audit (`cargo xtask visual determinism` over the smoke
tier, 890 scenarios captured twice, 2026-10-16) found no run-to-run
difference. It found a harness bug: `ctrl-f` (any modifier + `f`) failed
in tuiscotti's chord parser before reaching the app, which broke
`tablepro/table/filter_editor` and its dismissals; `Session::send_key` now
sends modified characters as `Key::Char`.

A second pass on 1e022b3a5 found 15 roots (42 smoke combos) that never
reached their state. Their steps and needles now follow the current app
(the app is unchanged); every combo of the full tier (950 captures, the
roots plus the other `tablepro/query/*` and `ack/*` rows that shared the
stale `i`) was captured twice byte-identical with `cargo xtask visual
determinism`. They stay pending re-approval.

| Root | Stopped at | Cause | Now |
|---|---|---|---|
| `showcase/flows/chrome/focus` | `wait:← → Menu` | hint reads `←→ Menu` | `wait:←→ Menu` |
| `showcase/flows/chips/select_open` (72x20–100x30 variants) | `wait:500` | Tab Tab reaches Sort by at every size | variants dropped, `wait:customer` everywhere |
| `showcase/flows/editor/completion` (+ dismissals) | `wait:Client` | focus starts the edit (1da20cc01): `i` was typed | `i` dropped |
| `showcase/resize/overview_{shrunk,grown}` | `Foundations / Overview` | tree sidebar crumb | `Design system / Overview` |
| `tablepro/connections/duplicated` | `wait:(Copy)` | Duplicate is Alt+D | `alt-d` |
| `tablepro/query/{results,explain_analyze}` | `wait:25 rows`, `wait:EXPLAIN ANALYZE` | 1da20cc01: `i` typed into the SQL | `i` dropped (also in `query/{error,explain,completion,completion_columns,close_confirm}`, `ack/*`, `workbench/quit_confirm`, which captured the wrong SQL silently) |
| `tablepro/ack/{gate,delete_gate}` (+ dismissals) | `wait:Type orders to confirm`, `wait:DELETE without WHERE` | same | same |
| `tablepro/ack/{armed,executed}` | `wait:Execute` | the token field starts editing and Enter on a match runs (3ec122eb2) | `type:orders`, Tab, Right (armed), then Enter (executed) |
| `tablepro/elements/connections/connect-{hover,pressed}` | `Connect    Edit` | the detail card needs an 80-column pane | `.min_size(100, 30)`: no 72x20 or 80x24 combos |
| `tablepro/overlays/explorer_context` (72x20, + dismissals) | `wait:S audit` | below the fold at 72x20 | `wait:S public` (the cursor row) |
| `jackin/editor/save_preview` | `wait:Save workspace` | arrows no longer leave the editing Name field; Space replaced it | Tab ×3 to Keep awake |
| `jackin/cockpit/debug` | `wait:run-2026` | the debug chip is `t` (c3515f652) | `t` |
| `jackin/accounts/filter` | `wait:Accounts · filt` | the prompt starts editing, so the first Enter applied an empty filter | `/`, `type:work`, Enter |
| `holla/elements/explore/tab-hover` (72x20) | `Disk` | Explore tabs below the fold | `.min_size(80, 24)` |
| `holla/fade/cleanup_list_wheel` | never settles | the review screen redraws every tick under reduced motion | `--motion paused --frame 40` |

`Root::min_size` drops combos where the app does not render a pending
row's target (`scenario_tables` rejects it on a stored row). The narrow
tier (`40x12`, `60x18`, pending) still misses several of these targets
(`jackin/cockpit/debug`, `jackin/accounts/filter` and others); it needs
its own pass before it is approved.

`jackin/manager/quit_confirm` (100 tuisnap files and its `overlay_sites`
row) was removed in 8a8c20253: a clean Ctrl+Q now quits at once, so that
dialog no longer exists. The quit guard is covered instead by pending
`OVERLAYS` rows (open plus both dismissals), each captured twice
byte-identical over the full tier (675 captures):

| Row | Guarded path |
|---|---|
| `jackin/editor/quit_guard` | workspace editor with an unsaved toggle, Ctrl+Q: Save / Discard / Cancel |
| `jackin/settings/quit_guard` | Settings with unsaved edits, Ctrl+Q |
| `holla/flows/quit/confirm_ctrl_c`, `confirm_menu` | live activities, Ctrl+C and File › Quit (Ctrl+Q stays `holla/flows/quit/confirm`) |
| `tablepro/connections/form_leave_{esc,edit,new}` | edited connection form, then Esc, `e` or Ctrl+N |
| `tablepro/connections/form_quit_guard` | edited connection form, Ctrl+C |
| `showcase/overlays/settings_quit_guard` | Settings page with unsaved toggles, `q`: Cancel / Discard |

`overlay_sites` now also counts the shared leave guard as an opener
(`.request_quit(&`, `.request(Leave::`), so `tablepro/connections.rs::leave_form`,
`tablepro/app.rs::request_quit` and `showcase/app.rs::request_quit` are
sites with these rows, and jackin's `editor.rs::leave` / `settings.rs::leave`
are no longer gaps. When these rows were added Ctrl+Q did nothing in
tablepro (its quit keys were hand-coded Ctrl+C and `q`), so its rows use
Ctrl+C.

### Harness-found bugs

| Bug | Root cause | Fix | Status |
|---|---|---|---|
| tablepro ignored Ctrl+Q (even over an edited connection form; its menu advertised it) | the app matched Ctrl+C and `q` by hand instead of the shared `QuitKeys` | `App::quit_keys` (`QuitKeys::DEFAULT`) classifies every key: Ctrl+C / Ctrl+Q before routing (Ctrl+C still cancels a running query first), `q` after the page; help lists `q / Ctrl+Q` | fixed; `app_tests_coverage::ctrl_q_quits_through_the_leave_guard` |
| holla's cleanup review (and the idle finder) redrew an unchanged frame every 200 ms under reduced motion | the simulated world advances every tick, and `CleanupPage::on_tick` / `FinderPage::on_tick` answered every new world tick with `Changed` (the finder's rebuild also restamps live data ages in milliseconds, though only their labels show) | `screens::rebuild_on_tick` rebuilds and reports `Changed` only when the screen's view (rows, candidates, checks, cursor; shown freshness labels) differs | fixed; `app_tests_coverage::an_idle_cleanup_review_requests_no_redraw` (0 of 50 ticks, was 50), `an_idle_finder_requests_redraws_only_when_a_label_moves` (at most 5 of 50, was 50). `holla/fade/cleanup_list_wheel` keeps `--motion paused --frame 40` |

The visual-test plan (`docs/visual-testing/plan.md`) decides the final
matrix and the OS that generates baselines (CI runs ubuntu-24.04) before
the rest is re-approved. Re-approve one group at a time:

```sh
VISUAL_FULL=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(/^<app>::<group>::/)'
# review target/tuiscotti/{actual,expected,diff}
cargo xtask visual accept 'test(/^<app>::<group>::/)'
```

Commit the `snapshots/` change on its own, apart from code.

## Harness measurements (one test per scenario)

Same machine and HEAD, gated run (`--run-ignored only -E
'binary(visual_baseline)'`, 525 re-approved scenarios, everything else
skipping), nextest `pty` group at 8 threads:

| Harness | Tests | Wall time |
|---|---|---|
| one test per root, 25 combos in series | 305 | 277 s (the streaming `terminal` root alone: 236 s) |
| one test per scenario | 8,201 | 135 s |
| smoke profile, one test per scenario (42 gated) | 886 | 15 s |

## Pilot measurements (`showcase/pages`, 18 roots × 25 captures)

Local run on macOS arm64, debug test profile, nextest `pty` group (8
threads).

| Run | Wall time | Notes |
|---|---|---|
| tuiscotti, gate passing | 262 s | about 13.5 s per root (0.54 s per capture: boot 15–400 ms, 400 ms quiet window, gate about 20 ms); `terminal` alone 234 s (its boot streams until `7 of 7`) |
| tuiscotti, every capture mismatching | 1,129 s | the mismatch path renders actual and expected PNG/HTML plus a diff, 2–9 s per capture; optimizing the render crates did not change it |
| tuisnap `2d43458`, same source, no approvals | 375 s | about 47 s per root; renders PNG and HTML for every capture |

Churn against the committed tuisnap approvals: all 450 `.ansi` differed.
On the same source, the tuisnap and tuiscotti engines produce
byte-identical `.ansi` for all 450 captures (cursor trailer excluded). So
none of the churn comes from the engine; it is product work on the branch
since the approval (the `↑↓` footer hint in 445, scrollbar gap and
horizontal bar, list counters, sidebar indicator, theme colours). The
cursor is hidden in all 450 pilot captures. A second gate run against the
new approvals matched all 450.

Approvals go stale fast while the UI is moving: after one rebase (54 new
commits on the branch) all 450 pilot scenarios differed again, again from
product changes only, and were re-approved on that HEAD (a later gate run
matched all 525 pilot and selectable scenarios in 278 s). Whoever changes
a re-approved surface re-runs the group and approves it with the change;
`cargo xtask visual check-touched` (README, "Gate for UI changes") finds
the groups a branch touched. With `VISUAL_REVIEW=0` the whole pilot runs
in about 115 s even when every capture mismatches.

`accept` copies only actuals of the current run. Every scenario test
first deletes its old actual files (a skipped scenario or one that fails
before its capture leaves none), and every capture writes a
`<name>.run` stamp next to its `.ansi` (`run=` the nextest run id,
`fnv1a64=` a hash of the `.ansi`). `accept` takes the run of the newest
stamp in its selection and refuses, writing nothing, when any selected
actual is unstamped, rewritten since, or from an earlier run (such as a
pending row run with `VISUAL_FULL=1` before), listing those scenarios.
Re-run them, narrow the filterset, or delete them.

| Size | Before (4 files) | After (`.ansi`) |
|---|---|---|
| `showcase/pages` on disk | 801 MB | 5.1 MB |
| per scenario, average | `.ansi` 9.6 KB, `.txt` 2.0 KB, `.png` 390 KB, `.html` 1.46 MB | 9.8 KB |
| scratch per mismatch (not committed) | | `.frame.json` 913 KB, `.html` 1.46 MB, `.png` 390 KB |

## Known tuiscotti main issues (at `a47c9aa`)

- A lone `f`/`F` key fails as a malformed function key:
  `crates/tuiscotti-runtime/src/tui/input_types.rs:165` strips the `F`
  prefix before the single-character case. The suite sends bare
  characters as `Key::Char`.
- Exact dependency pins (`Cargo.toml:22,31,32`: `crossterm =0.28.1`,
  `ratatui =0.29.0`, `unicode-width =0.2.0`) force this crate's
  `unicode-width` down from 0.2.2 to 0.2.0 and add a second
  ratatui/crossterm to the dev graph.
- `GroupedStore` always reads, renders and accepts all four artifacts
  (`grouped/check.rs:84-104,132-135`, `grouped/mutate.rs:22-27`), and
  `render::ansi_dump` (`render/text.rs:46`) writes no cursor. An upstream
  `ArtifactSet::AnsiCursor` mode would let the local gate go.
- `BoundLocator::expect_visible` requires a unique match
  (`bound_locator.rs:162-181`); boot needles that repeat on screen need
  `Locator::resolve` inside `wait_predicate` instead.
- `WaitError`'s `Display` (`tui/error.rs:107`) names only the revision;
  the suite prints the evidence screen itself.
- Rendering review PNG/HTML costs seconds per large frame, so a run where
  many captures mismatch is slow.
