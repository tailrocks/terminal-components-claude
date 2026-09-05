# Goal — finish the Rust TUI component-system refactor

Continue the in-progress refactor in this repository. Do not restart it, do not
re-audit what is already recorded, and do not redesign what is already
adjudicated. Everything needed is on disk.

## Read first, in this order

1. `REFACTORING_STATE.md` — the durable ledger: accepted decisions, ownership,
   evidence, unresolved findings, and next action. Older checkpoint sections are
   historical; do not treat them as current measurements.
2. `GOAL.md` and `REFACTORING_GOAL.md` — the contract. The definition of done is
   §29 and the final report is §30.
3. `COMPONENT_ARCHITECTURE.md` — the accepted architecture, **8,559 lines,
   accepted through §74**. **§29 is present.** Any change to a Decision,
   invariant, exact type, or precedence rule requires a fresh `opus-analyst`
   adjudication recorded in that document and in the ledger.
4. `docs/reviews/`, `docs/plans/`, `docs/audit/`, and `docs/guides/` — accepted
   reviews, executable slice plans, binding audits, and compiled API examples.

## Mandatory execution model

The coordinator, implementation role (`fable-builder`), and read-only analyst
run `claude-opus-5` at effort `high`. This is a recorded, user-authorized
exception to the `claude-fable-5-1` requirement: Fable 5.1 credits were
exhausted, and the user directed the work to continue on Opus 5 only. The
`fable-builder` name is historical; its role remains implementation-only. Do
not re-flag this authorized exception as a blocker or pass per-invocation model
overrides.

- Use a fresh, read-only `opus-analyst` for research, adjudication, diagnosis,
  review, and independent verification.
- Use `fable-builder` for repository mutations, with explicit disjoint file
  ownership.
- The coordinator sequences, measures, records, commits, pushes, and reports.
- Never use generic, inheriting, built-in Explore or Plan agents for this goal.

## Current state — measured at the current tip

The current **source payload** is **`b23df21`**
(`b23df21c93a4694a4e71c4e76029bea14e275759`); `origin/main` may be newer from
docs-only checkpoint commits. The latest source lineage is:

- `b23df21` — redact environment values from Jackin debug output;
- `c936d51` — keep Jackin environment drafts masked and persist only after save;
- `5dc310a` — satisfy the strict form clippy gate;
- `c152b97` — register Jackin shell props from `update`;
- `a69a7f1` — retain dynamic Form sensitivity state across updates;
- `e92944f` — clear stale Grid edit errors;
- `ff68306` — expose safe sensitive-text state constructors;
- `85ecac1` — close remaining Jackin preview gates;
- `0369742` — record the §73 adjudication;
- `e49de3f` — close dynamic Form error disclosure;
- `5ba0116`, `316731a`, `bbc48bb`, `40233b7`, and `07bb719` — prior
  source, visual-fixture, picker, clippy, and checkpoint work;
- `de30208`, `2bddd2b`, and `7016f5a` — test cleanup and disabled code-text
  preservation;
- `82158df`, `0f01836`, `776a2ce`, and `5d61076` — architecture-checker,
  props-identity, and grid-validation corrections;
- `69fb70d`, `4cfefea`, `dae0176`, `a10ff36`, `f98ebe1`, `1852338`,
  `06bf0e6`, and `4534a11` — app performance/constructor, error-redaction,
  visual-audit, sensitive-text, and rain-style fixes.

The worktree may contain uncommitted work from another owner; inspect status
before treating the committed tip as the complete source tree.

The workspace is already migrated to the `junie-tui` package/library
(`crates/tui/Cargo.toml`: package `junie-tui`, library `junie_tui`), with the
three application packages under `apps/`. Do not perform the old planned crate
rename or restore a legacy root package.

`xtask` dispatches `doc-check`, `boundary`, `bless-guard`,
`capture-matrix`, and `list`. The guard and matrix commands are real dispatch
paths, not pending work. `bless-guard` must fail closed unless
`BLESS_GUARD_BASE` or `GITHUB_BASE_REF` supplies a comparison base.

Measured at this tip:

- `cargo fmt --all --check`: **PASS**.
- `cargo run -p xtask -- doc-check`: **PASS**, 76 Rust blocks and 864 resolved
  references; the existing not-yet-built allow-list remains explicit.
- `cargo run -p xtask -- boundary`: all named source/contract checks pass except
  the fail-closed `baseline_moves_are_classified` check without a comparison
  base. `props_are_built_once` passes at 131 configured constructions, and
  `every_named_test_exists` reports 388 names, 387 present, and 1 deferred, and
  passes.
- `junie-tui` conformance: 934 passed; library: 735 passed; strict all-target
  clippy: PASS. TablePro route/perf: 1/11 passed; Jackin library/perf: 40/4
  passed. Broad Jackin app tests remain 11 passed / 15 stale journey failures.
- `shots/capture-provenance.json` is schema 1 at revision `a358272…`; the TSV
  contains 96 cells. Those captures predate the current tip and are stale
  evidence. The latest recorded independent audit still has the TablePro
  `connections` digest regression and the Jackin
  `accounts-1password-step-1` fixture failure. No baseline edit or visual
  blessing is authorized from them.
- The full workspace §26 gate set has not been re-run at `b23df21`; do not claim
  Slice or goal completion from the named checks above.

## Resume actions

1. Measure `git status --short`, `git log --oneline -5`, and the exact gate needed
   for the assigned slice. Preserve unrelated in-progress changes.
2. Keep architecture changes behind fresh adjudication and record accepted
   results in the ledger. Do not edit `COMPONENT_ARCHITECTURE.md` §73 or
   `REFACTORING_STATE.md` from a non-owner lane.
3. Before any baseline blessing, run a provenance-backed `capture-matrix`, obtain
   the required independent visual review, classify every moved key, and supply
   the guard's comparison base.
4. Report measured evidence, unresolved criteria, and blockers. Commit only
   completed owned files and push the current branch.

## Quality gates

Every command must exit 0 before a slice is called done:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --doc --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo build --workspace --all-targets --all-features
cargo build -p junie-tui --examples
cargo test  -p junie-tui --test render --test render_components
cargo test  --workspace --test perf --release -- --test-threads=1
cargo check -p junie-tui --no-default-features
cargo +1.88.0 check --workspace --all-targets --all-features
cargo run   -p xtask -- doc-check
cargo run   -p xtask -- boundary
cargo test  --all-targets
```

Name both render targets. Any moved baseline needs a `docs/visual-changes.md`
entry classified against a numbered §20.10 item before blessing. `boundary` and
`bless-guard` are fail-closed checks, not proof that the complete workspace gate
set is green.

## Rules and evidence discipline

Follow modern-API rules R‑1…R‑20 literally. Use `#[expect(lint, reason =
"…")]`, never `#[allow]`; no stubs, unsafe code in `crates/tui`, literal
palette colours outside `theme/builtin`, raw `bg: Color` parameters, or
TablePro/Jackin vocabulary in the library. Preserve edition 2024 and verified
MSRV 1.88.

Do not claim completion without command output and review evidence. When a
measurement contradicts an earlier record, correct the record plainly. Keep
the durable ledger current enough for a cold session to resume.

The goal is complete only when every item in `REFACTORING_GOAL.md` §29 is
proven, the §26 gates exit 0 after final corrections, fresh architecture and
visual reviews find no unresolved material issue, the ledger reports no
remaining work, and the §30 final report contains all required items.

## Historical record — superseded pre-migration state

The following facts are retained for provenance only; they are not active
instructions or current measurements.

- An earlier prompt described the architecture as approximately 6,300 lines,
  ending at §28.8, and claimed §29 did not exist. §29 and later sections were
  subsequently appended; the architecture now ends at §74.
- An earlier checkpoint called `junie-tui` / `junie_tui` temporary until Slice 5,
  described Slice 4 wave 1 as incomplete, and listed old fmt, clippy,
  conformance, and render-matrix failures. Those statements predate the current
  workspace and must not be used as current state.
- Earlier open findings included the Q residuals, discarded glyph slots, custom
  mono fallback coverage, Scenario G, app tick/grid questions, incorrect test
  counts, and missing `bless-guard`. They are historical findings; consult the
  architecture and ledger for their accepted dispositions and any remaining
  work.
- The earlier ordered plan (finish Slice 4, rename the crate in Slice 5, then
  migrate TablePro and Jackin) is retained as historical sequencing. The source
  tree has already reached the app packages; follow the current ledger and
  architecture instead.

### Historical checkpoint details

At the earlier checkpoint, the recorded state was:

- Slices 1–3 were described as complete, with six audits, 499 before-refactor
  captures, TablePro/Jackin digests, the `perf/baseline` tag, architecture
  Adjudications A–P, foundations, and a seven-component prototype.
- Slice 4 wave 1 was described as partially complete: 4B and 4G components
  existed, while registrations, the render matrix, and the gate were incomplete;
  4A, 4C, 4D, 4E, 4F, 4H, and 4I were described as unstarted.
- The old gate snapshot listed fmt failures on five files, roughly 18 clippy
  errors in `junie-tui`, two in `xtask`, a failing Tabs mono test, and a
  44-pass/116-fail render matrix. It also listed `boundary` failing on public
  component coverage and a `PERF_STRICT` wall-clock regression with no
  allocation or byte failures.
- The old open-decision list covered Q's missing architecture section and
  `Slot<GlyphRole>`, custom-family mono fallbacks, Scenario G, Grid questions,
  Jackin tick/time questions, `Screen::strip_right`, missing `bless-guard`,
  recorded app/test-count discrepancies, and six documentation findings.
- The old execution plan called for closing Slice 4, then a scripted crate
  rename in Slice 5, then parallel TablePro/Jackin migration and Slice 8
  cleanup/review. That plan is retained as historical sequencing only.
