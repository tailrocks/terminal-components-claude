# tuisnap snapshot baseline — coverage design

Provenance: designed 2026-09-12 from the completed capturable-surface inventory
(apps/CLI, existing `shots/` corpus, known gaps) and the tui-snap 0.2.0
documentation (`~/Projects/tui-snap/docs/USAGE.md`). CLI behavior quoted below
was validated against the installed `tuisnap` 0.2.0 binary on live captures of
`showcase`, `holla` and `tablepro` (see "Validated CLI facts").

The baseline is the artifact that later proves the refactoring did not change
UI/UX: every capture is a real binary running in a PTY, gated cell-exact and
pixel-exact against an approved snapshot in the store at `shots/tuisnap/`.

## Validated CLI facts (probed 2026-09-12)

- `tuisnap run --store shots/tuisnap --name N` writes `actual/N.frame.json` +
  `actual/N.png` FIRST, then gates. Missing approval exits 1 with
  `requires review (missing-approval)`; content drift exits 1 with
  `(cells-differ)`; spawn/timeout errors exit 1 with a different message
  (`spawn PTY`, wait-timeout). The summary classifier keys on these strings.
- `--format`/`--out` are IGNORED in store mode — loose artifacts are only
  written when no `--store` is given. Loose per-frame artifacts are therefore
  produced by a second, offline step:
  `tuisnap render --input actual/N.frame.json --format ansi --format txt
  --format png --format html --out frames/N` (byte-identical re-render of the
  canonical frame; no second PTY session).
- `--wait-for T` is appended AFTER all `--send` steps. A boot wait must be the
  first `--send wait:<needle>` step instead.
- Step syntax: key names (`enter escape tab backtab backspace insert delete up
  down left right home end pageup pagedown space f1..f12 ctrl-<c> alt-<c>`,
  any single printable char), `type:<text>`, `sleep:<ms>`, `wait:<needle>`.
  `alt-` accepts a single character only — `Alt+Enter`/`Alt+0..9` chords
  cannot be expressed (see Honest gaps).
- `argv[0]` is resolved via PATH: pass absolute binary paths.
  `env NO_COLOR=1 <bin>` works as the command for real NO_COLOR captures.
- With `--motion paused --frame N`, holla/jackin captures are deterministic:
  an accepted holla frame re-ran `matched` (seeded sim, no wall-clock
  randomness; `HOLLA_NO_HISTORY=1` suppresses history side effects).
- Apps can draw a stable blank frame before first content: every capture
  carries a `wait:` needle for a fully-rendered-screen string.
- `run_once` begins with a boot `wait_idle(200ms)` BEFORE any `--send` step,
  and it shares `--timeout-ms` with every other wait. A screen that repaints
  more often than every 200 ms from boot can never pass it, no matter the
  timeout (verified against termlens `wait_idle_deadline`: it needs one
  ≥200 ms output-silence window). Two consequences: boot-streaming screens
  (scrolling, terminal) get a per-capture `CAP_TIMEOUT` long enough to reach
  their idle end state; a screen animated forever from boot (showcase
  progress, `animating()==true`) is structurally uncapturable via
  `tuisnap run` — proven with a 30 s timeout probe.

## Naming scheme

`<app>_<surface>_<state>_<cols>x<rows>_<color>`

- `<state>` is `default` for the freshly-opened surface, otherwise a short
  slug of the driven state (`editing`, `invalid`, `sorted`, `gate-1`, ...).
- `<color>` ∈ `truecolor 256 16 none nocolor`; `nocolor` = real `NO_COLOR=1`
  in the environment (no `--color` flag), everything else = `--color <v>`.
- Examples: `showcase_buttons_default_120x40_truecolor`,
  `holla_rust-dirty_default_100x30_none`,
  `tablepro_ack_gate_120x40_truecolor`, `jackin_first-use_f300_120x40_truecolor`.

## Geometry and colour justification

- `80x24` — the canonical default terminal; most of the legacy corpus uses it.
- `120x40` — primary review geometry; all interactive states are captured here.
- `100x30` — holla's legacy scripted mono size, and below tablepro's
  explorer→drawer breakpoint (<100 cols), so one size exercises both contracts.
- `160x50` — wide-screen holla layout (legacy scripted matrix included it).
- `72x20` — documented minimum; holla renders "Terminal too small / Need
  72×20" below it, so exactly-72×20 is the boundary worth gating (spot only).
- `truecolor` is the primary palette; `none` (mono) proves structure survives
  hue loss; `256`/`16` are spot-checked on the most palette-sensitive surfaces
  (the retired 5×5 audit sweep covered every combo — its frames are now
  frozen historical evidence, see the supersession table);
  `nocolor` proves real backend suppression, which `--color none` cannot
  (backend rule vs app rule).

Two sizes × two colours for every static surface keeps the matrix tractable
while touching each page/scenario; the wider size×colour net lives on as the
showcase hash baseline plus the frozen legacy frames listed in the
supersession table.

## The baseline matrix (367 captures)

The runner `tools/tuisnap_baseline.sh` is the executable source of truth;
every entry below appears there verbatim.

### showcase — 118 captures

Static defaults: 22 of 23 pages × {80x24, 120x40} × {truecolor, none} = **88**
Name: `showcase_<page>_default_<size>_<color>`; argv `--page <slug> --color <c>`;
boot needle `Junie Design system`.

The progress page is EXCLUDED (was 4 captures): `ProgressPage::animating()`
is hardcoded `true`, so the page repaints every 80 ms from boot and the boot
`wait_idle(200ms)` inside `tuisnap run` can never observe an output-silence
window — proven with a 30 s timeout probe. No timeout/settle/send can fix
this on the current binary; see Honest gaps.

Scrolling and terminal are captured at their deterministic idle END states
via per-capture `CAP_TIMEOUT` overrides (the boot `wait_idle` shares
`--timeout-ms`): scrolling streams ~1600 demo log lines (400→2000, one per
80 ms tick ≈ 128 s) before follow-tail stops — each scrolling capture takes
~2.5 min; terminal boots into its staged demo run (`reset()` sets
`running=true`) which finishes in under 20 s. The gated frames show the full
finished scrollback / completed step rail, not a mid-stream phase.

Pages (name-slug → argv slug): overview, buttons, inputs, textareas, forms,
lists, trees, tables, editable→`editabletables`, panels, sidebars, dialogs,
scrolling, terminal, codeeditor, diff, datagrid,
chips→`chipsselects`, pickers, chrome, settings, taskrunner.
(`PageId::from_name` requires full normalized-label equality, so the argv
slug for "Editable tables" is `editabletables` and for "Chips & selects" is
`chipsselects` — verified live that `--page editable` is rejected.)

Palette spots: {buttons, forms, datagrid, diff} × 120x40 × {256, 16} = **8**
NO_COLOR spots: {overview, inputs} × 120x40 nocolor = **2**
Minimum-size spots: {overview, buttons} × 72x20 truecolor = **2**

Interactive states, all 120x40 truecolor, needle after the boot wait = **18**:

| Capture name | Sends after boot wait | Provenance |
|---|---|---|
| showcase_inputs_editing_120x40_truecolor | tab, enter, wait:EDIT | legacy audit_flows (retired) |
| showcase_inputs_selected_120x40_truecolor | tab, enter, ctrl-l, wait:EDIT | legacy audit_flows (retired) |
| showcase_forms_invalid_120x40_truecolor | tab, ctrl-s, wait:Required | legacy audit_flows (retired) |
| showcase_diff_review_120x40_truecolor | tab, enter, wait:● Review | legacy audit_flows (retired) |
| showcase_diff_empty_120x40_truecolor | tab, enter, tab, enter, wait:No file selected | focus ring [review, empty, view]; legacy audit_flows (retired) used backtab only because a mouse drag had focused the view first |
| showcase_buttons_focus_120x40_truecolor | tab | new |
| showcase_lists_moved_120x40_truecolor | tab, down, down | new |
| showcase_trees_expanded_120x40_truecolor | tab, right | new (trees had ZERO legacy frames) |
| showcase_tables_selected_120x40_truecolor | tab, down, down | new |
| showcase_editable_editing_120x40_truecolor | tab, enter | new (editable had ZERO legacy frames) |
| showcase_datagrid_selected_120x40_truecolor | tab, down, right | new |
| showcase_dialogs_open_120x40_truecolor | tab, enter | new |
| showcase_pickers_open_120x40_truecolor | tab, enter | new |
| showcase_chips_toggled_120x40_truecolor | tab, space | new |
| showcase_scrolling_scrolled_120x40_truecolor | tab, down, down, down (after the ~128 s boot stream, CAP_TIMEOUT=180000) | new |
| showcase_settings_toggled_120x40_truecolor | tab, space | new |
| showcase_help_overlay_120x40_truecolor | ? (on overview) | new |
| showcase_inspector_open_120x40_truecolor | i (on overview) | new |

Rows marked "new" use the library's documented key semantics (Tab enters the
page, arrows/space/enter act on the focused widget); they are confirmed
visually at accept time, like every first-run capture.

### holla — 194 captures

All static holla captures run `--motion paused --frame 40` with
`HOLLA_NO_HISTORY=1`; boot needle `holla❯` (row 0 of every scenario).

Core matrix: all 34 scenarios × {80x24, 100x30, 120x40, 160x50} truecolor =
**136**. Name: `holla_<scenario>_default_<size>_truecolor`.
Scenarios: first-use, rust-dirty, monorepo-root, monorepo-child,
docker-cleanup, disk-cleanup, upgrade-plan, activities-multi, remote-host,
launch-failure, hard-cases, parity-discovery, parity-history, parity-files,
parity-browser, parity-git-current, parity-git-batch, parity-task-sources,
parity-cargo, parity-docker, parity-brew-services, parity-gradle, parity-idea,
parity-upgrade-managers, parity-executor, parity-task-input,
parity-custom-actions, parity-disk-scan, parity-disk-navigation,
parity-insights, parity-delete-safety, parity-cleanup-results,
parity-platforms, parity-platforms-linux.

Mono: all 34 scenarios × 100x30 none = **34** (matches the legacy scripted
mono contract: lose nothing but hue).
16-colour: {upgrade-plan, disk-cleanup} × 100x30, `--frame 80` = **2**
(the two screens whose tinted rows the legacy 16-colour frames proved).
256-colour (gap closure — no legacy 256 holla captures existed):
{rust-dirty, upgrade-plan, disk-cleanup, hard-cases} × 100x30 = **4**
NO_COLOR (gap closure): {rust-dirty, upgrade-plan} × 100x30 nocolor = **2**
Minimum-size: {first-use, hard-cases} × 72x20 truecolor = **2**

Journeys, all 120x40 truecolor, `--motion reduced` (as the legacy flow
scripts drove them) except where noted = **14**. Reduced motion keeps
activities/plans advancing, so its boot `wait_idle` is a race — two journeys
already lost it on the first full run and are paused now (marked); if a
reduced journey flakes at a later re-verify, switch it to paused the same
way (the steady-state finder/files screens are motion-independent):

| Capture name | Scenario | Sends after boot wait |
|---|---|---|
| holla_finder_query_120x40_truecolor | parity-history | type:pull |
| holla_finder_query-selected_120x40_truecolor | parity-history (paused, frame 40) | type:pull, ctrl-a |
| holla_trust_prompt_120x40_truecolor | monorepo-child | type:test, enter |
| holla_files_results_120x40_truecolor | parity-files | type:Find files under home, enter, type:readme |
| holla_files_unicode_120x40_truecolor | parity-files | type:Find files under home, enter, ctrl-u, type:café |
| holla_browser_hidden_120x40_truecolor | parity-browser | type:Browse ~/work/site, enter, ctrl-h |
| holla_browser_preview_120x40_truecolor | parity-browser | type:Browse ~/work/site, enter, down, down, down |
| holla_cleanup_plan_120x40_truecolor | disk-cleanup (paused, frame 80) | c |
| holla_cleanup_gate-1_120x40_truecolor | disk-cleanup (paused, frame 80) | c, c |
| holla_upgrade_excluded_120x40_truecolor | upgrade-plan | down×5, space |
| holla_upgrade_confirm_120x40_truecolor | upgrade-plan | down×5, space, c |
| holla_remote_gate-1_120x40_truecolor | remote-host | type:restart payments, enter |
| holla_help_overlay_120x40_truecolor | rust-dirty (paused, frame 40) | f1 |
| holla_activities_overlay_120x40_truecolor | activities-multi (paused, frame 40 — reduced keeps a live spinner in the tab strip) | ctrl-g |

### tablepro — 29 captures

Boot needles: `TablePro` (connections), `Explorer` (workbench). Workbench
captures pass `--connect Production` (in-memory fixture, no drivers); the
settle wait rides out the simulated connect ticks.

Static = **9**: `tablepro_connections_default_{80x24,120x40}_truecolor` (2);
`tablepro_workbench_default_{80x24,120x40}_truecolor` (2 — 80x24 exercises
the <100-col drawer mode); `tablepro_workbench_default_120x40_{256,16,none}`
(3); `tablepro_workbench_default_120x40_nocolor` (1);
`tablepro_connections_default_120x40_none` (1).

Interactive, all 120x40 truecolor = **20** (sequences traced from
`src/bin/tablepro/app_tests.rs` and the retired tools/audit_flows.sh; "open orders" below
= down×5, enter from the fresh workbench):

| Capture name | Sends after boot wait |
|---|---|
| tablepro_form_new_120x40_truecolor | (connections) ctrl-n |
| tablepro_form_new-filled_120x40_truecolor | (connections) ctrl-n, type:Staging replica |
| tablepro_table_data_120x40_truecolor | open orders, wait:public › orders |
| tablepro_table_structure_120x40_truecolor | open orders, ctrl-d, wait:Columns |
| tablepro_table_sorted_120x40_truecolor | open orders, right×12, s, wait:sort created_at ▴ |
| tablepro_table_filtered_120x40_truecolor | open orders, home, right×4, f, backtab, backtab, enter, ctrl-l, type:pending, enter, wait:filtered (1) |
| tablepro_cell_editing_120x40_truecolor | open orders, home, right×4, enter |
| tablepro_row_duplicated_120x40_truecolor | open orders, alt-d (pending-change queue) |
| tablepro_query_completion_120x40_truecolor | tab, i, type:SELECT * FROM ord, wait:order_items |
| tablepro_query_results_120x40_truecolor | tab, i, type:SELECT * FROM ord, enter, type:" WHERE st", tab, type:" = 'pending' ORDER BY created_at DESC LIMIT 25", escape, ctrl-r, wait:25 rows |
| tablepro_query_error_120x40_truecolor | tab, i, type:SELECT * FROM missing_table, escape, ctrl-r, sleep:800 |
| tablepro_query_explain_120x40_truecolor | tab, i, type:SELECT * FROM orders, escape, ctrl-x, sleep:500 |
| tablepro_ack_gate_120x40_truecolor | tab, i, type:UPDATE orders SET status = 'paid' WHERE id = 'x', escape, ctrl-r, wait:Type orders to confirm |
| tablepro_ack_armed_120x40_truecolor | gate + enter, type:orders, enter, right, wait:Execute |
| tablepro_ack_executed_120x40_truecolor | armed + enter, wait:rows affected |
| tablepro_history_tab_120x40_truecolor | ctrl-y, wait:History |
| tablepro_picker_open_120x40_truecolor | ctrl-o, wait:Open Quickly |
| tablepro_tablist_open_120x40_truecolor | ctrl-g |
| tablepro_safemode_picker_120x40_truecolor | ctrl-l, wait:Safe Mode |
| tablepro_help_overlay_120x40_truecolor | ?, wait:Keyboard |

### jackin-preview — 26 captures

All static jackin captures run `--motion paused`; boot needle `jackin❯`.
Rain/motion is seeded (`MOTION_SEED`, no wall-clock randomness), so a paused
frame is exact.

Scenarios × {80x24, 120x40} truecolor, frame 40 = **16**:
first-use, returning, accounts-mixed, launch-running, launch-failure,
capsule-multi, outro-last, hard-cases →
`jackin_<scenario>_default_<size>_truecolor`.
Needle exception: the outro-last starfield and the first-use warp frame
(f300) carry no `jackin❯` brand line; both wait on the persistent
`Enter Skip` hint instead (verified present on the timeout screens of the
first full run). The outro caption ("You were in the Construct for …") is
seed-glitched under paused motion, so it is not a reliable needle.

first-use intro phases at 120x40 (INTRO_END = tick 308): frame 300 (warp) +
frame 400 (post-intro manager) = **2** → `jackin_first-use_f{300,400}_120x40_truecolor`.
Mono: {first-use, capsule-multi, accounts-mixed, hard-cases} × 120x40 none = **4**
Palette spots: capsule-multi 256 + accounts-mixed 16, 120x40 = **2**
NO_COLOR: first-use 120x40 nocolor = **1**
Minimum-size: first-use 72x20 truecolor = **1**

## Legacy supersession table

The tmux/Python harness that produced the legacy corpus was removed
2026-09-12. Every "frozen" verdict below means the frames in `shots/` remain
as historical evidence but can no longer be regenerated — what was lost is
the regeneration mechanism, not evidence (the hover/fade frames were ad-hoc
and never scripted, and no resize-sequence corpus ever existed). The
successor path for the lost lanes is tuisnap's Rust PTY API
(`Session::click/drag/resize`) driven from a Rust test, plus a future
showcase `--motion` flag for the progress page.

| Legacy category | Replaced by | Verdict |
|---|---|---|
| `h_parity_*` (115: 23 parity scenarios × 4 sizes + mono) | holla core matrix (136) + mono (34) — same 4 sizes, same mono@100x30 contract | **Fully superseded** (adds store gating) |
| `h_<concept>_*` (57) | holla core matrix covers all 11 concept scenarios at 4 sizes + mono | **Fully superseded** |
| `h_flow_*` + `h_p3_*` (45 journey frames) | holla journeys (14) | **Partially** — the Alt+Enter alternatives, Alt+0..9 activity tab jumps, and wall-clock runs (upgrade run-to-failure ≈14 s sleeps, discovery progression) that the CLI cannot express deterministically are frozen frames; successor: Rust test via the tuisnap Rust PTY API |
| `h_hp01..23_*` (69 parity slices) | holla parity scenarios in core matrix + finder/files/browser/cleanup/upgrade/remote journeys | **Partially** — time-progression frames (hp01 discovering under full motion + sleep 4) and multi-step wizard states beyond the 14 baseline journeys are frozen frames; successor: Rust test via the tuisnap Rust PTY API |
| `j_*` (~116, ad-hoc, no script) | jackin matrix (26) incl. first-use phase frames f40/f300/f400 | **Partially** — static per-scenario coverage superseded; interactive cockpit/capsule menu/pane/tab states driven by unproven key sequences are frozen frames (they were ad-hoc, never scripted, so no regeneration mechanism was lost) |
| `f_*` + `s_*` + `s2_*` (~43 showcase, ad-hoc) | showcase matrix (118) — 22 of 23 pages, 2 sizes × 2 colours, 18 states; closes the trees/editable zero-frame gap | **Partially** — fully supersedes the keyboard/static corpus EXCEPT progress-page frames: progress is structurally uncapturable via `tuisnap run` (see Honest gaps #0); the legacy progress frames are frozen until a showcase `--motion` flag lands |
| `t_*` (38 tablepro, ad-hoc) | tablepro matrix (29) | **Partially** — connection-form Advanced tab and Duplicate flow (reachable only via fragile multi-Tab focus walks) and EXPLAIN ANALYZE variants are frozen frames |
| `shots/audit/` (250: 10 fixtures × 5 sizes × 5 colours) | per-app static matrices | **Partially** — baseline covers more surfaces but fewer size×colour combos; the 5×5 sweep (72x20/160x50 for tablepro/jackin, full 256/16 everywhere) is frozen wide-net evidence; successor for a live wide net: extend the tuisnap matrix |
| `shots/audit-flows/` (51) | showcase/tablepro interactive captures | **Partially** — keyboard flows superseded; the diff text-drag (mouse) frames and NO_COLOR reverse-video buffer assertions are frozen (the flow script was removed with the harness); successor: `Session::click/drag` from a Rust test |
| `shots/fade/` (17 scroll-fade) | none | **Frozen** — scroll fade appears under live wheel/scroll; these frames were ad-hoc (never scripted), so only the frames remain; successor: `Session::click/drag` + wheel from a Rust test |
| `tests/showcase_baseline.txt` (460 hashes: 23 pages × 5 sizes × 4 palettes) | showcase matrix | **Partially** — retained as the cheap hash-level wide net (5 sizes × 4 palettes) complementing the pixel-gated subset |

## Honest gaps — what tuisnap CLI cannot capture

0. **Screens that never go quiet from boot cannot be captured at all.**
   `tuisnap run` starts with a boot `wait_idle(200ms)` before any send step,
   and it shares `--timeout-ms` with every later wait. Learned from the first
   full run (19 failures, all diagnosed from the per-capture logs):
   - **showcase progress (4 captures): permanently uncapturable.**
     `ProgressPage::animating()` is hardcoded `true` → 80 ms repaints forever
     → no 200 ms output-silence window ever occurs. Proven with a 30 s
     `--timeout-ms` probe (same boot-wait timeout as at 8 s). No
     timeout/settle/send/flag fixes this on the current binary; capturing a
     blank or mid-animation frame would violate the no-weakening rule, so the
     4 progress captures are REMOVED from the matrix. Disposition: the
     legacy `f_*`/`s_*` progress frames are frozen historical evidence (the
     capture tooling that made them was removed 2026-09-12); the proper fix
     is a showcase `--motion paused` flag (holla/jackin already have one) —
     restore the 4 captures when it lands.
   - **showcase scrolling / terminal (8 captures): boot-streaming, fixed.**
     Scrolling streams ~1600 demo log lines (one per 80 ms tick, ≈128 s)
     before follow-tail stops; terminal boots into its staged demo run
     (<20 s). Both reach a deterministic idle END state, so they are captured
     with per-capture `CAP_TIMEOUT` overrides (180000/30000 ms) and gate that
     end state — each scrolling capture costs ~2.5 min of wall time.
   - **holla reduced-motion journeys (2 of 7 failed once): boot race.**
     Under `--motion reduced` activities/plans keep advancing; if any visible
     element repaints within every 200 ms window (activities-multi tab-strip
     spinner did), the boot wait loses the race. `activities_overlay` and
     `finder_query-selected` moved to `--motion paused --frame 40` (their
     steady-state screens are motion-independent). The five journeys still on
     reduced passed, but the race is inherent — if one flakes at re-verify,
     switch it to paused too rather than re-running blindly.
1. **Mouse states** (hover/pressed rows of the showcase state matrices, diff
   text-drag selection, scroll fade under wheel, hover evidence in j_*/t_*).
   CLI has no click/drag/move. Disposition: the existing frames (audit-flows
   diff_drag, shots/fade, hover evidence in j_*/t_*) are frozen historical
   evidence — the tmux harness was removed 2026-09-12, and the hover/fade
   frames were ad-hoc (never scripted), so what is lost is the regeneration
   mechanism, not evidence. Successor: tuisnap's Rust PTY API
   (`Session::click/drag`) from a Rust test.
2. **Mid-session resize sequences.** CLI geometry is fixed per run; the
   baseline gates layout at each static size (incl. the 100-col tablepro
   drawer breakpoint and 72x20 minimum) but not reflow behaviour.
   Disposition: no resize-sequence corpus ever existed (the legacy harness's
   resize command was a capability, never run into `shots/`), so only the
   capability is gone. Successor: `Session::resize` from a Rust test.
3. **Wall-clock motion phases** (holla hp01 discovery progression, upgrade
   run-to-failure after ~14 s, jackin live rain). Nondeterministic under a
   cell/pixel gate. Disposition: deterministic `--motion paused --frame N`
   captures in the baseline (seeded sim); live-motion frames are frozen
   legacy evidence. Successor: a Rust-side driver via tuisnap's Rust PTY API.
4. **Alt+Enter and Alt+0..9 chords** (`alt-` takes a single character only):
   h_flow_alternatives and the activity-tab jumps cannot be expressed.
   Disposition: the legacy h_flow frames for these chords are frozen
   historical evidence. Successor: drive the chords from a Rust test via
   tuisnap's Rust PTY API.
5. **Spinner/indeterminate mid-animation frames** (showcase taskrunner live
   rows once started, tablepro connect ticks). Residual nondeterminism:
   the baseline captures these surfaces in their static initial state after
   `wait_stable` (--settle-ms 400); a spinner glyph cannot be pinned. If one
   flakes at re-verify time, re-run once before suspecting a regression;
   deliberate relaxation = `tuisnap report --pixel-threshold` for review.
   (The progress page is NOT covered by this item — see gap 0: it never
   reaches a static state at all.)
6. **tablepro form Advanced tab + Duplicate button flow** — only reachable
   via long focus walks that a baseline should not depend on.
   Disposition: the t_* legacy frames covering these are frozen historical
   evidence (the capture tooling was removed 2026-09-12).
7. **Glitched text is not needle-stable.** jackin's outro caption is drawn
   through a seed-glitched renderer under paused motion, so a text needle
   may legitimately never match; the outro/warp captures wait on the
   persistent `Enter Skip` hint instead. General rule: needles must be
   verified against the actual paused frame, not assumed from live motion.

## Regeneration procedure

```bash
# 1. full rebuild from scratch (store is disposable until accepted)
rm -rf shots/tuisnap
tools/tuisnap_baseline.sh
#    - cargo build --bins (skip with SKIP_BUILD=1)
#    - runs all 367 captures; first run: every entry lands in actual/ and is
#      reported as pending (missing-approval) — expected, not a failure
#    - note: the 5 scrolling captures take ~2.5 min each (boot stream)
#    - renders frames/<name>.{ansi,txt,png,html} per capture
#    - ends with a captured/pending/drift/failed summary (also:
#      tools/tuisnap_baseline.sh summary)
# 2. review every actual
open shots/tuisnap/report.html
# 3. approve after review (explicit; no env var approves anything)
tuisnap accept --store shots/tuisnap --all
# 4. re-verify: every approved frame must report matched
tuisnap report --store shots/tuisnap
```

Subset re-runs (after touching one surface):

```bash
APPS=tablepro SKIP_BUILD=1 tools/tuisnap_baseline.sh
ONLY='holla_(rust-dirty|upgrade-plan)' SKIP_BUILD=1 tools/tuisnap_baseline.sh
APPS=showcase SIZES=120x40 COLORS=truecolor tools/tuisnap_baseline.sh
```

## How the baseline is used later (refactoring gate)

- After any refactoring change: `tools/tuisnap_baseline.sh` (rebuilds, then
  re-runs the PTY for every approved name). `matched` = no UI/UX drift.
  `cells-differ`/`pixels-differ` = drift: inspect `shots/tuisnap/diff/<name>.png`
  (red overlay) + the first-100 cell diagnostics, fix the regression or — only
  for an intended visual change — re-accept that name explicitly.
- `tuisnap report --store shots/tuisnap` re-verifies every
  `actual/*.frame.json` against `approved/` and rewrites `report.html`
  (publishable CI artifact).
- Single capture check without a PTY:
  `tuisnap check --store shots/tuisnap --name N --input shots/tuisnap/actual/N.frame.json`.
- Store layout: `shots/tuisnap/actual/` (latest frames + PNGs),
  `shots/tuisnap/approved/` (approved PNG baselines), `shots/tuisnap/diff/`
  (mismatch overlays), `shots/tuisnap/report.html`,
  `shots/tuisnap/frames/` (loose ansi/txt/png/html review aids — derived,
  never gated).

## Git tracking policy

The store is ~3.2 GB on disk; only the canonical, non-regenerable baseline is
tracked in git (~490 MB):

- **Tracked:** `shots/tuisnap/approved/` (367 × frame.json + approved PNG +
  `.png.fidelity.json` — the gate authority), `shots/tuisnap/frames/*.ansi`
  and `frames/*.txt` (small, and `ansi` is an explicitly required persisted
  format).
- **Ignored (`.gitignore`):** `actual/`, `diff/`, `.baseline-run/`,
  `report.html` (~1.4 GB — embeds every frame), `frames/*.html`,
  `frames/*.png`. All ignored artifacts are deterministic re-renders of the
  tracked approved frames: `tuisnap render --input
  shots/tuisnap/approved/<name>.frame.json --format png --format html --out
  <prefix>`, and `tuisnap report --store shots/tuisnap` rebuilds the report
  after any run.
- The legacy `shots/` corpus outside `shots/tuisnap/` is left in place as
  frozen historical evidence; the tmux/Python harness that produced it was
  removed 2026-09-12. Supersession per category is mapped above. Physical
  removal is a separate decision: partially superseded categories still carry
  mouse-driven evidence (hover/drag) that tuisnap CLI cannot reproduce — the
  regeneration path for those lanes is a Rust test driving tuisnap's Rust
  PTY API (`Session::click/drag/resize`).
- Renderer pin: approved PNGs were rendered by tuisnap built from
  tui-snap@263eeeb (JetBrainsMono Nerd Font Mono faces, HiDPI rasterization,
  10×21 cells at 16px). A different tuisnap build pixel-drifts by design;
  after an intentional renderer upgrade: `tuisnap report --store
  shots/tuisnap`, review, `tuisnap accept --store shots/tuisnap --all`.
- Capture environment: the runner strips `NO_COLOR` from its environment
  (`PRESERVE_NO_COLOR=1` opts out). An exported NO_COLOR (agent and CI shells
  set it) silently poisons every colour capture — crossterm suppresses colour
  SGR by *presence*, even under `--color truecolor`: frames claim truecolor in
  their header yet record all-Default cells. The 2026-09-12 first baseline was
  re-captured for exactly this reason.
