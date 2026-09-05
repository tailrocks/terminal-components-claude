# Resume prompt — Complete Rust TUI Component-System Refactor

Paste this as the first message of a new session.

Continue the in-progress refactor. Do not restart from scratch or treat old
checkpoint summaries as current measurements.

## Read first

1. `REFACTORING_STATE.md` — the durable ledger. Older checkpoint sections are
   historical evidence; use the measured tip below until the ledger is refreshed.
2. `GOAL.md` and `REFACTORING_GOAL.md` — definition of done §29 and final report
   §30.
3. `COMPONENT_ARCHITECTURE.md` — **8,559 lines, accepted through §74; §29 is
   present**. Changes to decisions, invariants, exact types, or precedence rules
   require fresh `opus-analyst` adjudication recorded in the architecture and
   ledger.
4. `docs/reviews/`, `docs/plans/`, `docs/audit/`, and `docs/guides/` — accepted
   evidence and executable plans.

## Routing exception

Coordinator, `fable-builder` implementation workers, and `opus-analyst` all run
`claude-opus-5` at effort `high`. Fable 5.1 was unavailable because its monthly
credits were exhausted; the user explicitly authorized continuing on Opus 5.
This is a known, authorized exception, not an active blocker. The
`fable-builder` label is historical; retain its implementation-only role and do
not pass model or effort overrides.

## Recorded checkpoint — measured at source payload `b23df21`

This checkpoint is historical until refreshed against the current worktree and
committed tip. Do not treat its gate counts or capture metadata as current
measurements.

The current **source payload** is `b23df21`
(`b23df21c93a4694a4e71c4e76029bea14e275759`); `origin/main` may be newer from
docs-only checkpoint commits. Its immediate source lineage is
`b23df21` (Jackin environment debug redaction), `c936d51` (masked environment
drafts), `5dc310a` (form clippy gate), `c152b97` (Jackin shell props registration),
`e92944f` (stale Grid edit errors),
`ff68306` (safe sensitive-text state
constructors), `85ecac1` (Jackin preview gates), `0369742` (§73 adjudication),
and `e49de3f` (dynamic Form error disclosure), following the prior capture,
architecture-checker, props/grid, and rain-style fixes. The worktree may contain
uncommitted work from another owner; inspect status before treating the committed
tip as the complete source tree.

For current Jackin security behavior, inspect
`apps/jackin-preview/src/domain/workspace.rs`: transient plain environment
input is masked and enters the pending workspace only after key validation and
Save; persisted key-shaped values use a mask that intentionally retains the
final four characters. Older “no secret ever reaches a frame” wording is not a
valid current claim.

The library is already package `junie-tui` / library `junie_tui` in
`crates/tui`; the workspace has `apps/showcase`, `apps/tablepro`, and
`apps/jackin-preview`. Do not apply the obsolete planned crate rename or restore
the removed root package.

`xtask` dispatches `doc-check`, `boundary`, `bless-guard`, `capture-matrix`,
and `list`. Current named checks:

- `cargo fmt --all --check`: PASS.
- `doc-check`: PASS — 76 Rust blocks and 863 resolved references; the existing
  not-yet-built allow-list remains explicit.
- `boundary`: all named checks pass except the fail-closed
  `baseline_moves_are_classified` check needs `BLESS_GUARD_BASE` or
-  `GITHUB_BASE_REF`; `props_are_built_once` passes at 131 configured
  constructions. `every_named_test_exists` is 388/387 with one deferred name
  and passes.
- `junie-tui` conformance/library tests: 934/735 passed; strict all-target
  clippy passes. TablePro route/perf: 1/11; Jackin library/perf: 40/4. Broad
  Jackin app tests remain 11 passed / 15 stale journey failures.
- Tracked capture provenance is schema 1 at revision `a358272…` with 96 cells,
  stale against `b23df21`; the latest independent audit still records the
  TablePro `connections` digest regression and Jackin
  `accounts-1password-step-1` fixture failure. No baseline edit or blessing is
  authorized until fresh provenance-backed captures and independent review exist.
- The full workspace §26 gate set has not been re-run at `b23df21`.

## First actions

1. Run `git status --short`, `git log --oneline -5`, and the gate relevant to the
   assigned work. Preserve unrelated worktree changes.
2. Run `cargo run -p xtask -- doc-check` and `cargo run -p xtask -- boundary`
   after documentation or contract changes; provide the exact results.
3. For visual work, run `capture-matrix` only with the required authorization,
   classify moved keys before blessing, and provide a comparison base to
   `bless-guard`.
4. Do not claim full completion until the entire §26 gate set and required fresh
   architecture/visual reviews are green.

## Historical record — superseded session-2 prompt

The old resume text is retained as provenance, not as an action list:

- It described a post-token-limit tree with an `E0502`, last green commit
  `0f66160`, and interrupted Slice-4 builders. That was repaired before the
  current tip.
- It recorded an old `HEAD` of `69fcdca`, a temporary `junie-tui` crate name,
  an un-migrated root package, and 247 legacy tests. Those facts belong to the
  pre-migration history.
- It correctly records the origin of the model exception: three Fable builders
  were terminated by HTTP 429 and the user authorized Opus 5 continuation.

### Historical session-2 details

That old prompt also recorded a first action of building `junie-tui`, running
the package tests and boundary check, and keeping the root package's 247 tests
green. It described the interrupted prototype components and architecture
amendments as WIP. Those instructions applied to the old tree only; use the
current tip and gate state above.
