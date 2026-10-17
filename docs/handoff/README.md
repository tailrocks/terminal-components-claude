# Handoff: how to continue unfinished work

Every restart reads this file first (see [../GOAL.md](../GOAL.md),
"Continue from a handoff"). It lists every unfinished branch on `origin`,
what is on it and exactly what to do next, plus the operational knowledge
that is not obvious from `AGENTS.md`.

Rule (AGENTS.md "Way of working"): a coordinator or agent that stops with
unfinished work pushes it to `wip/<name>` on `origin` and lists it in the
table below in the same change. Nothing lives only in a local worktree or
a scratch dir. When a WIP branch lands, delete its row (and, once the
owner agrees, the remote branch).

Last updated: 2026-10-18, base `origin/design/interaction-polish` at
`6f8a4b3af` plus this docs commit.

## Start of every session

1. `git fetch origin && git ls-remote --heads origin 'wip/*'`: compare
   with the table. A branch missing from the table: read its log and add
   a row before anything else.
2. Push lock: `ls "$(git rev-parse --git-common-dir)/push.lock"`. If it
   exists and its `owner` file is older than 50 minutes (check
   `stat -f %Sm` on macOS), no live agent holds it: `rm -rf` it. A
   younger lock belongs to a live agent: leave it.
3. Land or resume the WIP branches (table order) before new work. Each
   resuming subagent gets [subagent-rules.md](subagent-rules.md) and the
   branch's resume note.

## WIP branches

None of these is merged into `design/interaction-polish`. "Behind" is
the number of commits on `design/interaction-polish` the branch lacks at
the time of writing; every branch needs a rebase first.

| Branch | Tip | Item | State | Exact next steps |
|---|---|---|---|---|
| `wip/reach-pages-a` | `f5b538137` (7 commits, from `5b73c5b41`) | Priority 0b R34, R40–R44 (Scrolling, Terminal, TextUtilities, QueryWorkbench, Connections, Finder Playgrounds) | Another agent was landing it when this handoff was written. Committed on the branch (not yet on `design/interaction-polish` at writing): library `5b73c5b41` (panel meta reveal), R34 (`0081e8df9`, `4825f44ea` re-approval), `33072c20f` (StepRail cut-label reveal), `b4314e3f5` (scroll_view sideways-only bar), R40 (`1ac32e248`); `f5b538137` is a WIP commit with R41–R44 measures | Check `git log origin/design/interaction-polish..origin/wip/reach-pages-a`. If empty, the branch landed: delete this row. Otherwise: rebase; finish R41 TextUtilities, R42 QueryWorkbench, R43 Connections, R44 Finder (`Page::measure`, delete their lines in `tests/showcase_reach_allow.txt`, mark rows done in `docs/remaining-work.md`); regenerate `tests/showcase_baseline.txt` per page and check by script only that page's rows changed; gate; re-approve changed captures with cell attribution. Expect a conflict with `wip/rapps-3-7-14` on the `QueryWorkbench \| Playground \| 120x40` reach line |
| `wip/rapps-3-7-14` | `81be99a79` (1 commit, behind 4) | R-apps-3: Tree/List meta folds into the cut label with `…`; `ui::columns::dropped_text` + `reveal_dropped` for hand-rolled tables (holla files, jackin mounts) | Not gated. R-apps-7 and R-apps-14 from the same agent landed (`6e19572c0`, `5629775d9`, `a2f397f1b`, `3144afba9`) | [resume/rapps-3-7-14.md](resume/rapps-3-7-14.md) "Left": 1) fix tablepro `app_tests::explorer_opens_table_and_grid_navigates` (`organizations` missing; likely tree `label_cell` width with folded meta in the narrow sidebar); 2) check every regenerated `tests/showcase_baseline.txt` row is intended; 3) full gate + visual check-touched, attribute cells, snapshot-only re-approval; 4) mark R-apps-3 done in `docs/remaining-work.md` and `docs/audits/design-review-2026-10/responsive.md`; 5) record that hand-rolled tables use `reveal_dropped`, not `widgets::table` |
| `wip/plan-s16` | `b249a81be` (1 commit, behind 179) | S16a library: `RowMark`, `RowSource::is_header`, `ListItem::{header, status, mark}`, `ListBox::dirty_lane`, `Column::marker`, `Cell::strong`, `TreeNode::{meta_tone, trailing, cells}`, `TreeView::columns` | Builds; nothing else run. 20 catalog lines added | [resume/resume-plan-s16.md](resume/resume-plan-s16.md) "Not done": 1) rebase (expect conflicts in `src/widgets/list.rs`, `tree.rs`, `src/catalog.rs`); declare or allowlist (`id \| reason \| SC35`) the 20 new catalog ids; re-add the unit tests `row_extension_tests` (list.rs) and `tree_table_tests` (tree.rs); gate; push the library; 2) S16d tablepro EXPLAIN on `TreeView::columns` (attribute capture changes); 3) showcase P0-G1 (Disk cleanup partial check), P0-G2 (size column), EXPLAIN tree-table on Query workbench, SC35 demos; 4) S16b holla and S16c jackin rows, CO1, the 11 `y Copy` literals, tablepro history `!` onto `ListItem::status` |
| `wip/s14-followups` | `95f62ca33` (1 commit, behind 176) | S14 follow-ups task 2: `shell::run` installs the real clipboard for every app; holla and jackin copies are truthful (OSC 52) | nextest (macOS), clippy, fmt passed before the rebase; linux nextest and visual not run. Task 1 (SI17 copy-first) landed as `95ac0ca05` | [resume/resume-s14-followups.md](resume/resume-s14-followups.md): 1) rebase; 2) visual check of holla captures with a copy status (wording changes), attribute every cell to `95f62ca33`, holla snapshot-only re-approval; jackin wording must not change; 3) full gate incl. linux, push; 4) docs: SI17 done in `docs/remaining-work.md` and plan "S14 deviations" 1–3, status row in `docs/audits/design-review-2026-10/showcase-pages-navigation.md` item 4, record the three findings; 5) task 3 (K6: tablepro asks before quit or disconnect while a query runs, through the leave guard) and task 4 (P0-G6: Read-only report adopts `PageHeader` / `Heading` / `ActionRow`) |
| `wip/v1-showcase` | `c1a49af0f` (1 commit, behind 275) | D10 holla flake fix: a task's shown duration counts script time only (prompt waits excluded); lines after a program-ending answer are dropped. V1 itself not started (0 of 66 roots) | Test `domain::activity::tests::duration_does_not_depend_on_how_long_a_prompt_waited` fails on the parent, passes on the tip; holla tests and fmt pass. No full gate, no re-approval | [resume/resume-v1-showcase.md](resume/resume-v1-showcase.md): 1) owner D10 is recommended "yes"; rebase; 2) re-approve `holla/flows/task/input_cancelled` (and `input_killed` if it changes) in a snapshot-only commit naming the fix (`0 s` instead of `1 s`, the `rolling out` line gone); 3) 20x determinism run on that scenario; full gate; push; update "Known flakes" and `docs/audits`; record the residual `input_killed` wall-pace risk; 4) V1 proper: `scripts/v1/` (`base.sh`, `mkfilter.py`, `det.sh <group filter>`, `report.py`, `effect.py`), group by group, `tuiscotti accept` per group, "Showcase legacy (V1)" section in `visual-reapproval.md` |

Historical resume notes (landed, kept for context): S13
([resume/resume-plan-s13.md](resume/resume-plan-s13.md)), S14
([resume/resume-plan-s14.md](resume/resume-plan-s14.md)), SC group H
([resume/resume-sc-group-h.md](resume/resume-sc-group-h.md)), reach R45–R48
and R51 ([resume/reach-pages-b.md](resume/reach-pages-b.md)). Paths into a
`/private/tmp/...scratchpad` inside the notes are gone; the scripts that
matter are in [scripts/](scripts/).

## Notes for the owner (no decision needed)

- **DiskCleanup and Workspaces** scroll sideways by about 5 columns at
  120x40, since their Playgrounds lay out at natural size (R45, R47).
- **Drawer demos** no longer
  collapse into a summary and drawer at 80 and 100 columns by default:
  the page scrolls instead. The `collapse_below` option still shows the
  collapsed mode.
- **Dialog SQL** (R-apps-14): a write gate's code preview now joins the
  body scroll, so on short screens the SQL sits below the facts and needs
  a scroll to see.
- Activity's Steps and Run cards stack under the log at 120x40 (R46).
- TextArea placeholders wrap over the rows (`8b63fed89`, deliberate
  visual change; Text areas, Forms, tablepro connection forms).

## Operational knowledge

### Push lock and lock relay

- Take the lock before fetch, rebase and gate:
  `mkdir "$(git rev-parse --git-common-dir)/push.lock"` and write
  `<branch> <date>` into `push.lock/owner`. `mkdir` is atomic, so a
  failure means it is held. Inside: fetch, rebase onto
  `origin/design/interaction-polish`, check
  `git diff origin/design/interaction-polish HEAD --stat`, gate on the
  exact HEAD, `git push origin HEAD:design/interaction-polish`, then
  `rm -rf` the lock, on failure too. Never force-push.
- A held lock whose owner file is older than 50 minutes is stale and may
  be removed. All worktrees share one lock (it lives in the common git
  dir).
- Lock relay (coordinator): when an agent stops because the lock is held,
  the coordinator runs a background
  `until mkdir <lock> 2>/dev/null; do sleep 5; done`, writes the waiting
  agent's name into the owner file, then resumes that agent and tells it
  the lock is already its own (it must not `mkdir` again). This keeps
  agents from polling and keeps the queue fair.

### Archive-first visual gating

- Build ONE nextest archive, then run every partition from it with
  `--archive-file ... --workspace-remap .`. Concurrent cargo builds
  otherwise replace app binaries mid-boot (ENOENT, blank-screen flakes).
- At most 4 visual partitions in parallel (`--partition hash:k/4`,
  `--test-threads 4`), each part under 4 minutes; the unit and linux runs
  as 2 partitions each. Check each part's summary as it finishes.
- [scripts/gate.sh](scripts/gate.sh) `archive | unit | linux | visual`
  does exactly this (`GATE_DIR` = your scratch dir). The visual phase
  runs the `cargo xtask visual check-touched --dry-run` filter that the
  archive phase saved.
- Run the full visual check only inside the push lock (or just before
  taking it); while developing, run only the groups the change touches.

### Cell attribution before `tuiscotti accept`

- Never accept a failing capture on sight. First run
  [scripts/cells.py](scripts/cells.py) `<worktree> <GATE_DIR>/vis-*.txt`
  (per capture: text cells and style-only cells that changed) and
  [scripts/classify.py](scripts/classify.py) (blank / undim / other
  totals, the first 20 "other" cells). Every changed cell must be
  explained by a named commit; anything unexplained is a bug or a flake.
- Then [scripts/accept.sh](scripts/accept.sh) accepts the failing names
  one per call with the mise tuiscotti CLI (`mise which tuiscotti`), in a
  snapshot-only commit `test(visual): <scope> re-approval` that names the
  causing commit. Legacy captures compare against the `visual-baseline`
  tag read-only ([scripts/v1/base.sh](scripts/v1/base.sh)).

### `target-dev/` as a separate target dir

- Use `CARGO_TARGET_DIR=target-dev cargo nextest run ...` for quick
  development runs (fail-first checks, one test) while a gate archive or
  visual run uses `target/`. Sharing `target/` makes the two builds
  replace each other's binaries mid-run.
- `/target-dev/` is ignored through the shared `.git/info/exclude`, not
  `.gitignore`; never commit it. Gate runs still use the default
  `target/` (the visual harness reads `target/tuiscotti/actual`).

### Known flakes

Authoritative list: `docs/visual-testing/README.md` "Known flakes".
Re-run a listed flake alone before treating it as a failure; never
re-approve from a flaky run.

- `holla/flows/task/input_cancelled`: duration counts prompt wait time;
  fixed on `wip/v1-showcase` (`c1a49af0f`, D10).
- `holla/flows/brew/services_stop_failed`: a wall-clock failure notice in
  the footer under load.
- `jackin/accounts/add_form` (160x50): stale bold attributes on the last
  row.
- `jackin/capsule/new_tab`: failed only under load (dim state); passes
  alone.
- tablepro unit test `the_write_gate_refuses_an_outside_click_and_cancels_on_esc`:
  failed once in a full macOS run, passed on re-run.

### Machine load

- Several agents share one machine (load average around 29). At most 5
  subagents at once, helpers included; at most 4 parallel visual
  partitions; no full visual runs "for checking" outside the lock.
- Every foreground command has a timeout of at most 240000 ms; no
  command runs more than 5 minutes without the agent acting on it.
- Never `pkill -f` or `killall`; stop only PIDs you started.
- Hook wrappers may block plain `git` in agent worktrees; call
  `/usr/bin/git` or `/opt/homebrew/bin/git`, one plain command per call.

## Scripts

All in [scripts/](scripts/); each has a one-line header. Copy into your
scratch dir before running, set the env var they name, run from the
worktree root.

| Script | What it does |
|---|---|
| `gate.sh` | Archive-first gate: `archive`, `unit`, `linux`, `visual` (4 parts) |
| `accept.sh` | `tuiscotti accept` every failing capture from the visual logs, one per call |
| `cells.py` | Per-capture changed-cell attribution, approved vs actual |
| `classify.py` | Classifies every changed cell (blank / undim / other) |
| `reach-page-filter.patch` | Dev-only: one-page reach sweep (`REACH_PAGE`) and a Playground dump test (`DUMP_PAGE`) |
| `v1/base.sh` | Extracts the `visual-baseline` tag's showcase snapshots read-only |
| `v1/mkfilter.py` | Lists the legacy showcase roots and writes per-group filters |
| `v1/det.sh` | Determinism capture for one group filter |
| `v1/report.py` | Per-root change summary against the tag |
| `v1/effect.py` | Interaction effect then vs now, per root |
| `v1/ansi.py` | `.ansi` parser used by the V1 scripts |
