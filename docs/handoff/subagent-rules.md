# Subagent rules (binding for every subagent)

The coordinator pastes or links this file in every subagent prompt.
`AGENTS.md` is authoritative and wins on any conflict; read it fully
first, especially "Way of working". This file merges the rules the
2026-10-17/18 coordinator gave every agent (common rules, showcase page
rules, Priority 0 and Priority 0b briefs).

## Setup

- Repo: `/Users/donbeave/Projects/terminal-components-claude`, branch
  `design/interaction-polish`. Each agent works in its own git worktree.
- Setup, exactly:
  `git fetch origin && git switch --detach origin/design/interaction-polish && git switch --no-track -C <your-branch> && git status --porcelain`
  (must be empty, else `git reset --hard origin/design/interaction-polish`).
  If hook wrappers block plain `git` in the worktree, call it by full path
  (`/usr/bin/git` or `/opt/homebrew/bin/git`), one plain command per call.
- Resuming a WIP branch instead: `git switch --no-track -C <your-branch> origin/wip/<name>`,
  then rebase onto `origin/design/interaction-polish` (see
  [README.md](README.md) for each branch's next steps).
- Read `docs/remaining-work.md` (your rows and the docs they cite) and
  `docs/architecture/shared-components-plan.md` (your step row, earlier
  "outcome" and "notes for <step>" sections).
- Tools: `mise install` once; plain `cargo`. If cargo still reports an
  older rustc than `rust-toolchain.toml`, prefix
  `RUSTUP_TOOLCHAIN=<that version>`. Use the mise tuiscotti CLI
  (`mise which tuiscotti`), never `~/.cargo/bin`. `cargo nextest` only,
  never `cargo test`.
- Development builds that must not disturb the gate archive go to a
  separate target dir: `CARGO_TARGET_DIR=target-dev cargo nextest run ...`
  (see README "target-dev").
- Scripts live only in your own scratch dir (never in the repo unless they
  are handoff scripts under `docs/handoff/scripts/`); run only scripts you
  wrote or copied from `docs/handoff/scripts/` after reading them. Check
  `git rev-parse --show-toplevel` is your worktree before running one.
- Never `pkill -f` or `killall`; stop only PIDs you started.

## Time and load

- Every foreground command has an explicit timeout of at most 240000 ms.
  A job that takes longer is split so each part fits: nextest
  `--partition hash:k/N`, visual groups by filter, clippy separately from
  tests. If one times out, split it further; never re-run it whole.
- 5-minute activity rule (owner): never leave a command (foreground or
  background) running longer than 5 minutes without acting on its
  progress. Background long jobs, keep working, check each part as it
  finishes. Never `sleep` longer than 180 s.
- Never purely wait. If the push lock is held, keep working and retry at
  the next checkpoint. If only waiting remains, end the run and report
  exactly where to resume.
- Machine load (load average ~29 with several agents): at most 4 parallel
  visual partitions, never 8. Run a full visual check-touched only inside
  the push lock or just before taking it; never several full visual runs
  "for checking". Prefer targeted visual runs of the groups your change
  touches while developing.

## Push

- Commit each coherent step as soon as it passes the gate; small portions.
  Commit messages end with a blank line, then
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Order: take the push lock FIRST, then fetch and rebase, then gate once
  inside the lock, push, release. Never gate a push candidate outside the
  lock (it repeats after every upstream push).
- Push lock: `mkdir "$(git rev-parse --git-common-dir)/push.lock"` and
  write `<branch> <date>` into `push.lock/owner`. If it is held: keep
  working, retry at the next checkpoint; remove it only when its owner
  file is older than 50 minutes. Always `rm -rf` it after the push or on
  failure. If the coordinator says the lock was claimed for you, do not
  `mkdir`; it is already yours.
- Inside the lock: fetch; rebase onto `origin/design/interaction-polish`;
  `git diff origin/design/interaction-polish HEAD --stat` shows only
  intended files; full gate on the exact HEAD, checking exit codes:
  `cargo nextest run --workspace --all-targets`, again with
  `JUNIE_TUI_CLIENT_PLATFORM=linux`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt --all --check`, `cargo xtask visual check-touched`; then
  `git push origin HEAD:design/interaction-polish`. Never force-push.
- Fast gate: build ONE nextest archive and run every partition from it
  (`--archive-file`), at most 4 visual partitions in parallel, each part
  under 4 minutes. `docs/handoff/scripts/gate.sh` does exactly this.
- Stopping before the gate passes: push the work to `wip/<name>` (never
  to `design/interaction-polish`) and report the tip and next steps so
  the coordinator lists it in `docs/handoff/README.md`.

## Behaviour and visual rules

- Every behaviour fix: a regression test through the real app or widget,
  shown failing on the parent.
- Fix behaviour in shared code (`src/core`, `src/ui`, `src/widgets`);
  apps and showcase are thin consumers. Every finding is fixed or recorded
  in `docs/audits` with a reason.
- `tests/showcase_baseline.txt`: regenerate only in the causing commit
  (`UPDATE_BASELINE=1 cargo nextest run --all-targets showcase_visual_baseline`)
  and verify by script that only intended rows changed.
- Re-approve captures (`tuiscotti accept`, see `docs/visual-testing/`)
  only after a script attributes every changed cell to named commits
  (`docs/handoff/scripts/cells.py`, `classify.py`), in snapshot-only
  commits `test(visual): <scope> re-approval`. All four apps and the
  showcase pages are gated.
- Legacy (unapproved) captures are compared against the `visual-baseline`
  tag look, read-only (`git archive visual-baseline ...`); record the
  classification in
  `docs/audits/design-review-2026-10/visual-reapproval.md` before
  accepting. Never move, delete or recreate the tag.
- Known flakes: `docs/visual-testing/README.md` "Known flakes". Re-run a
  listed flake alone before treating it as a failure; never re-approve
  from a flaky run.
- Never post anything publicly.

## Showcase page steps

- Each page = template Playground | Options | States
  (`src/bin/showcase/pages/template.rs`, SC2). Options: the page's arm in
  `options::page_options`, `Page::option_value`/`set_option`, every knob
  drives the real library builder (`--opt <page>.<key>=<value>` works
  through `App::apply_opt`; `Template::sync_options`, `Request::SetOption`).
  States: `Specimen` (`src/ui/specimen.rs`) and `StateGrid`; one live `▎`
  only (samples depict focus with `▏`). Playground: realistic demo data;
  interaction is the widget's own.
- Read `AGENTS.md` (Showcase section and coverage gate) and plan §7 fully
  (principles, page tree and slugs, shared acceptance, your SC rows, §7.8
  outcomes, "How to satisfy the coverage gate"), plus
  `docs/audits/design-review-2026-10/showcase-coverage-library.md` rows for
  your widgets.
- Coverage gate: declare `Demo` / `Option` / `StateRow` (incl.
  `Shown::Via`) in `src/bin/showcase/coverage.rs`, delete the matching
  lines from `tests/showcase_coverage_allow.txt`, register new public API
  in `src/catalog.rs`. `cargo nextest run --bin showcase remaining_gaps --no-capture`
  lists what is left per step; reach 0 lines or re-point each remaining
  line to the right later step with a reason.
- Tests: Harness tests through the real showcase for each page (options
  change rendering, States rows present, key interactions).
- Add pending tuiscotti scenarios for each new page or section.
- Other agents run in parallel: touch only your pages' files plus shared
  files strictly needed (options arm, coverage entries, allowlist lines,
  catalog). Rebase often; keep others' changes. Update the §7.8 step
  status table and your `docs/remaining-work.md` rows in your docs commit.

### Priority 0: no stub pages

- No page may render the "<Page> arrives with SCnn" stub. If the library
  component a page is about does not exist, extract it now (its S-step
  part only: generic, configurable, standard traits, builders, events,
  tests), migrate the apps' hand-rolled copies onto it (behaviour-
  identical; attribute any capture change) and register it in the catalog.
- Screens pages compose only library components; where one is missing and
  out of scope, compose the closest and record the gap row in
  `docs/remaining-work.md`, never draw it in the page.

### Priority 0b: every Playground fully reachable

- `src/bin/showcase/app_tests_reach.rs` walks every page at 80x24,
  100x25, 106x29 and 120x40 and checks every word visible at 220x70 is
  reachable. Failures are listed in `tests/showcase_reach_allow.txt` and
  as rows R1–R54 in `docs/remaining-work.md`.
- Fix: implement `Page::measure` (natural size) and lay the Playground
  out at it so content overflows into the template's `ScrollView`. Use
  library `Measure`; a widget that cannot measure itself gets `Measure`
  in the library (shared, tested), never a page-local hack. Content an
  inner scroller holds is reachable by the shared inner-scroller rule,
  not a per-page exception.
- Remove the page's lines from `tests/showcase_reach_allow.txt` (a fixed
  page keeping its line fails as stale) and mark its R row done. Layouts
  stay readable at 80x24; focus-follow keeps focused controls visible.
- `docs/handoff/scripts/reach-page-filter.patch` adds a one-page filter
  and a dump test for development (apply locally, never commit).

## Final message

Per item: status and commit hashes; behaviour changes with their tests;
visual changes with attribution; deviations; findings for other steps;
anything recorded instead of fixed; and, if unfinished, the `wip/<name>`
branch, its tip and exactly where to resume.
