# Snapshots v2 — grouped tuisnap baseline

The visual-baseline suite (`tests/visual_baseline/`) stores approved frames in
the **grouped multi-artifact store** (`tuisnap::grouped::GroupedStore`, tui-snap
rev `c4b5975764b61fb0bc82aff83253ebab68ab0694`). It replaces the classic
single-PNG store at `shots/tuisnap/` (v1, frozen legacy corpus under `shots/`).
This document is the taxonomy + workflow reference; capture rationale and the
honest-gaps list stay in `docs/baseline/tuisnap-coverage.md`.

## Taxonomy

Capture name = `<app>/<sub_group>/<leaf>`; `group` = app, `sub_group` =
scenario family. The leaf drops the app prefix and keeps
`<surface>_<state>_<cols>x<rows>_<color>`; the v1 `no_color` spelling is
`nocolor` everywhere.

| group | sub_groups |
|---|---|
| `showcase/` | `pages/` (18 non-audit pages, static defaults; progress via `--motion paused`), `audit/` (5 audit pages × 5×5 matrix), `flows/` (keyboard/mouse states), `hover/`, `fade/` (wheel scroll-fade), `resize/` |
| `holla/` | `concept/` (9 non-audit concept scenarios static), `parity/` (23 parity scenarios static), `audit/` (rust + upgrade fixtures × 5×5), `flows/` (journeys, chords, gates, menus), `fade/`, `resize/` |
| `jackin/` | `scenarios/` (6 non-audit scenarios static), `audit/` (accounts + capsule × 5×5), `intro/` (phase frames), `manager/`, `cockpit/`, `capsule/`, `editor/`, `accounts/`, `usage/`, `settings/` (route sub_groups for interactive) |
| `tablepro/` | `audit/` (production workbench × 5×5), `connections/`, `workbench/`, `query/`, `table/`, `ack/`, `overlays/`, `fade/`, `resize/` |

**Dedupe rule:** if a surface is one of the 10 audit fixtures, ALL its
static-default captures live only in `<app>/audit/` (full 5×5 matrix);
otherwise statics live in the app's static sub_group at standard combos. No
capture exists twice.

**Audit matrix (250):** the 10 fixtures × 5 sizes
{72x20,80x24,100x30,120x40,160x50} × 5 colours {truecolor,256,16,none,nocolor},
generated data-drivenly in `tests/visual_baseline/audit.rs` (one test fn per
fixture loops the 25 combos, so each combo exists exactly once):

| fixture | bin | args | needle |
|---|---|---|---|
| holla-rust | holla | `--scenario rust-dirty --motion paused --frame 40` | `holla❯` |
| holla-upgrade | holla | `--scenario upgrade-plan --motion paused --frame 40` | `holla❯` |
| jackin-accounts | jackin-preview | `--scenario accounts-mixed --motion paused --frame 40` | `jackin❯` |
| jackin-capsule | jackin-preview | `--scenario capsule-multi --motion paused --frame 40` | `jackin❯` |
| showcase-{buttons,diff,forms,inputs,textareas} | showcase | `--page <slug>` | `Junie Design system` |
| tablepro-production | tablepro | `--connect Production` | `Query 1` |

Determinism notes (learned during the v2 regeneration):

- **jackin-accounts** drives a live session and, at sizes where the Accounts
  list overflows (72x20/80x24/100x30), waits for the scroll badge text
  (`of 23`) before settling. The badge needs the list viewport, learned on
  render one, so it first appears in a later repaint; accounts-mixed never
  finishes its boot refresh under paused motion (`refreshing 1` →
  `busy_rows` non-empty → a repaint every 500 ms tick), while
  `wait_stable(400 ms)` can fire at t≈400 ms, before the badge repaint.
  Waiting for the badge text pins the capture after it (120x40/160x50 do not
  overflow — no badge exists there).
- **holla/flows/finder_query** runs `--motion paused --frame 40` (not
  reduced): parity-history renders a wall-clock relative-time label
  (`live · N s ago`) that reduced-motion ticks advance.
- **showcase/flows/scrolling_scrolled** prepends `wait:739.63s` (the boot
  stream's end-state marker, the 2000th log line's timestamp) to its sends:
  `wait_idle(200ms)` can fire early during a mid-stream stall under load, and
  sends racing the stream tail leave the frame's final cursor position —
  embedded in the gated HTML — dependent on flush timing.

**Audit-flows (51):** the five showcase flows (`diff_review`, `diff_empty`,
`diff_drag-selected`, `forms_invalid`, `inputs_selected`) at 9 combos each
({80x24,120x40,160x50} × {truecolor,none,nocolor}) + the tablepro ack chain
(`gate`/`armed`/`executed`) × {truecolor,nocolor}. The keyboard variants are
data-driven in `showcase.rs`, the drag variants in `pointer.rs` (live-session
drag technique).

## Store layout

```text
snapshots/<group>/<sub_group>/<name>.ansi   colored terminal text (cell-exact gate)
snapshots/<group>/<sub_group>/<name>.txt    plain text (content gate)
snapshots/<group>/<sub_group>/<name>.png    colored image (pixel gate, threshold 1.0)
snapshots/<group>/<sub_group>/<name>.html   standalone HTML render (render-level gate)
```

The approved tree is committed and holds **exactly those four artifacts per
scenario** — no `.frame.json`, no `.cursor` sidecars. Scratch state lives
under `target/` (gitignored):

```text
target/tuisnap/actual/<name>.{ansi,txt,png,html}   latest capture (written BEFORE gating)
target/tuisnap/actual/<name>.frame.json            debug sidecar (report re-verification)
target/tuisnap/actual/<name>.png.fidelity.json     missing-glyph sidecar
target/tuisnap/diff/<name>.png                     red-overlay diff, on mismatch
target/tuisnap/report.html                         expected/actual/diff report
```

## Regenerate / bless workflow

```sh
# 1. regenerate from scratch (every capture logs PENDING — nothing approved yet)
rm -rf snapshots/ target/tuisnap/
cargo test --test visual_baseline -- --ignored --skip rebuild_review_html

# 2. bridge the suite's target/ actual root to the CLI's default sibling
#    roots (one-time; gitignored). The CLI's grouped accept/report derive
#    actuals from <store>.actual next to --store.
ln -sfn target/tuisnap/actual snapshots.actual

# 3. bless: approved = current actuals (the ONLY bless; the test never approves)
cargo run --manifest-path ~/Projects/tui-snap/Cargo.toml --release -- \
  accept --grouped --store snapshots --all          # or: --name <group/leaf>

# 4. re-run until every capture reports Matched (0 PENDING, 0 failures)
cargo test --test visual_baseline -- --ignored --skip rebuild_review_html

# 5. rebuild the HTML report (re-verifies every actual against its approval;
#    asserts 0 failed)
cargo test --test visual_baseline report -- --ignored
# CLI equivalent:
# cargo run --manifest-path ~/Projects/tui-snap/Cargo.toml --release -- \
#   report --grouped --store snapshots --report-path target/tuisnap/report.html
open target/tuisnap/report.html
```

On drift after approval the gate fails closed: review
`target/tuisnap/diff/<name>.png` plus the first-difference notes in the test
panic, then either fix the app or re-bless with steps 2–3 (the suite rewrites
actuals on every run).

## Intentional drops (v1 → v2)

Every legacy scenario has a v2 counterpart or a documented drop. The drops,
with reasons (from the v2 coverage plan §4):

**Generational duplicates (jackin):** all `j2_*` (7), `j3_*` (10), `j4_*`
(11), `j5_*` (17), `j6_*` (4) — superseded design generations of the same
surfaces; current-design counterparts are the `j_*` states covered by the
jackin interactive plan (§3.4 there).

**`j_*` residual drops:** `j_intro_warp_{early,late,reveal}`,
`j_intro_phrase`, `j_outro_{warp,caption}` — wall-clock rain phases /
seed-glitched text; covered at boundaries by `jackin/intro/f300,f400` +
`jackin/scenarios/outro-last`. `j_prelude_1..5(_error)` — tick-driven
transition stages; boundary captured (launch_prelude), stages
nondeterministic. `j_accounts_op1..4` — mid-operation progression
(Modal::Op animates); end state only.

**Wall-clock motion (holla):** `h_flow_discovering`, `h_flow_upgrade_running`,
`h_flow_upgrade_failed(_step)` (~14 s run-to-failure), `h_hp01_discovering`,
`h_hp14_burst`, `h_hp14_stream`, `h_fade_burst_mid`, `h_flow_trusted_running`,
`h_p3_docker_running` — tick-phase frames, nondeterministic under a cell/pixel
gate; deterministic boundary/end states captured instead (static matrices,
end-state flows). Same class, documented in `tests/visual_baseline/holla.rs`:
`h_hp15_prompt`, `h_hp15_input_mode`, `h_hp15_answered`, `h_hp15_stopping`
(wall-clock spinner / `running · N s`; `animating()` stays true so
`wait_stable` never holds), `h_hp18_scanning`, `h_hp18_cancelled` (mid-scan
percentage is tick-count; paused pins 0%, reduced lands on a different
phase every run). Terminal hp15/hp18 states are captured instead.

**Wall-clock motion (tablepro):** `t_running` — live query execution animates;
end states captured (`tablepro/query/results`, `tablepro/query/error`).

**Harness contract:** `h_hp02_recent` — `HOLLA_NO_HISTORY=1` (suite
`opts_for`) makes the usage-populated Recent list ("used N times here")
unreachable; every capture shows "nothing used here yet".

**Streaming/attached child TTY (holla):** `h_flow_btm`, `h_flow_btm_attached`,
`h_flow_logs`, `h_flow_logs_80`, `h_flow_pg_blocking`, `h_flow_logs_chips`,
`h_flow_logs_hidden` — attached/streaming content repaints forever
(progress-page-class structural gap); the detach chord (`ctrl-]`) state may
be added later if a settle proves reachable.

**Ad-hoc orphans:** `shot.*`, `t1.*`, `t2.*`, `source_f_lists_{move,tab}.*`,
`stderr.log`, `.args.junie_fable_*`, `.DS_Store` — debug artifacts with no
scenario contract (`source_f_lists_*` states are covered by
`showcase/flows/lists_moved`).

**Harness sidecars:** `shots/audit/stderr.*`, `shots/audit-flows/{README,stderr}.*`,
`shots/fade/stderr.*`, all `*.cursor` — not snapshots; cursor positions live
in the tuisnap `.frame.json` metadata (scratch sidecar).

**Superseded duplicates:** `t_80*/t_100*/t_160*` (the tablepro/audit matrix
covers all five sizes), `t_conn_dup`/`t_conn_form_adv` (superseded by the
connections duplicate/advanced-form flows), `h_flow_help` (covered:
`holla/flows/help_overlay`), `h_flow_trust` (covered:
`holla/flows/trust_prompt`), `h_flow_cleanup_{plan,gate1}`,
`h_flow_remote_gate1`, `h_flow_upgrade_{excluded,confirm}`,
`h_flow_upgrade_cleanup_excluded` (covered by existing journeys),
`s2_progress`/`f_progress` (showcase `--motion paused` landed; progress
statics live under `showcase/pages/progress_default_*`, mid/done under
`showcase/flows/progress_{mid,done}_*`), `f_taskrunner_running` (now
`showcase/flows/taskrunner_running_120x40_truecolor`), `s2_pickers`/`s2_tabs`
(duplicates of pickers/chrome pages coverage),
`h_p3_docker_{root,plan,gate1}` (covered by docker-cleanup static + the
docker gate chain).

**Legacy size/color variants outside matrices:** remaining `h_*`/`j_*` ad-hoc
mono/256/16 variants — the audit matrices are strictly wider.

**Filled LIVE holes (not drops):** these six now have v2 counterparts:

| v1 | v2 |
|---|---|
| `h_flow_args` | `holla/flows/args_clone_120x40_truecolor` |
| `h_hp04_preview_control` | `holla/flows/files_preview_control_120x40_truecolor` |
| `h_hp23_linux_report` | `holla/flows/cleanup_linux_report_120x40_truecolor` |
| `j_accounts_form_ref` | `jackin/accounts/add_form_required_120x40_truecolor` |
| `t_dirty` | `tablepro/table/dirty_120x40_truecolor` |
| `t_sorted_filtered` | `tablepro/table/sorted-filtered_120x40_truecolor` |
