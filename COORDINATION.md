# Multi-lead coordination contract

Three lead agents are working the same goal (`GOAL.md`, `REFACTORING_GOAL.md`,
`CONTINUE_PROMPT.md`) against this one working tree. This file is the authoritative
lane and file-ownership contract between the leads. It is the only cross-lead channel.

**Main lead: Lane A.** Lane A owns sequencing, the shared ledger, the architecture
document, and the final §30 report. Lanes B and C report results by appending to
their own status file (below); Lane A folds them into `REFACTORING_STATE.md`.

## Single-writer files (Lane A only — never edit from Lane B or C)

- `REFACTORING_STATE.md`
- `COMPONENT_ARCHITECTURE.md`
- `COORDINATION.md`
- `Cargo.toml` (workspace root), `Cargo.lock`
- `.github/workflows/ci.yml`, `.claude/**`

Lane B and Lane C record architectural findings as new files under `docs/reviews/`
using a lane-prefixed name (`laneB-*.md`, `laneC-*.md`) and a status file
(`docs/status/laneB.md`, `docs/status/laneC.md`). Lane A transcribes accepted
decisions into `COMPONENT_ARCHITECTURE.md` and the ledger.

## Lanes

### Lane A — library, foundations, Slice 4 and Slice 5 (main lead)

Exclusive ownership:
- `crates/tui/**`
- `crates/tui-testing/**`
- `xtask/**`
- `tools/**`
- root `src/**` removal, `tools/capture.sh`, the `tui-next` -> `junie-tui` rename

Deliverables: close Slice 4 wave 1, then packages 4A/4C/4E, then wave 2
(4D/4F/4H/4I), then Slice 5 (rename, showcase migration, Adjudication P1).

### Lane B — Slice 6, TablePro

Exclusive ownership:
- `src/bin/tablepro/**` and the TablePro app tree wherever it lands
- `tests/baselines/tablepro.txt`
- `docs/plans/slice6-tablepro.md`, `docs/reviews/laneB-*.md`, `docs/status/laneB.md`

Blocked-on-Lane-A items (raise in `docs/status/laneB.md`, do NOT implement in
`crates/**` yourself): Slice 6 Q1 (does the library `Grid` own a sort permutation
and on what comparison), Q2 (what `GridState` exposes; eleven TablePro chords and
three migrated tests need the cursor), Q3 (may a control register a zero-area focus
entry). Research and adjudicate them read-only with a fresh `opus-analyst` and post
the adjudication as `docs/reviews/laneB-grid-contract.md`; Lane A records it in the
architecture document and implements the library half.

Recorded counts to honour: TablePro has **23** app tests and **42** digests.

### Lane C — Slice 7, Jackin

Exclusive ownership:
- `src/bin/jackin_preview/**` and the Jackin app tree wherever it lands
- `tests/baselines/jackin.txt`
- `docs/plans/slice7-jackin.md`, `docs/reviews/laneC-*.md`, `docs/status/laneC.md`

Blocked-on-Lane-A items: Slice 7 Q1 (`Screen::strip_right` removed with no named
replacement; `HintLayer.status` cannot carry its priorities and tones) and Q2 (the
accepted `App` trait has no tick hook and no accessor for elapsed virtual time,
yet Jackin's clock, both rain state machines, the launch stage machine,
`world.jobs` and the status timeout are all tick-driven). Adjudicate read-only,
post as `docs/reviews/laneC-app-tick.md`; Lane A implements the library half.

Highest risk, stated so it is not rediscovered: Jackin's virtual clock advances by
the **route's** interval (`Route::tick_ms`). Re-basing it on a uniform token or a
wall-clock delta breaks ~40 tick counts, every fixture timestamp and the outro
caption at once.

Recorded counts to honour: Jackin has **28** app tests, **36** digests, and the
Jackin `Screen` trait has **23** methods.

## Contended files — minimal-insertion protocol

`crates/tui/src/components/mod.rs`, `crates/tui/src/lib.rs`, `crates/tui/src/author.rs`,
the suite list in `crates/tui/tests/conformance.rs`, and `xtask/named_tests_allow.txt`
are Lane A's, and within Lane A are edited only by single-line insertions in
alphabetical position, re-read immediately before each edit, retried on failure.
`xtask/named_tests_allow.txt` may only shrink.

## Repository-wide operations — Lane A only, announced first

`cargo fmt --all`, any baseline blessing (`BLESS=1`), the crate rename, dependency
bumps, and `git rebase`/`git push --force` are repository-wide. Only Lane A runs
them. Lane B and Lane C format with an explicit file list scoped to their own
ownership (`cargo fmt -- <files>`), never `--all`.

## Baseline discipline

Any moved baseline needs a `docs/visual-changes.md` entry classified against a
numbered §20.10 item **before** blessing. `docs/visual-changes.md` is append-only;
each lane appends its own entries and never rewrites another lane's.

## Commit discipline

Commit only the files your own lane owns. Never `git add -A`, never `git commit -a`.
Pull with `git pull --rebase` before pushing. If a rebase conflicts in a
single-writer file, drop your side and tell Lane A.
