# Termrock migration record — coordinator ledger

Single committed ledger for the Termrock refactor delivered through PR #38
(`parity/termrock-visual-baseline`). Progress, decisions, coverage gaps and
evidence links live here. Detailed artefacts stay in their own files; this
record links them instead of restating them.

Historical narrative and per-increment receipts stay in
[termrock-migration-matrix.md](./termrock-migration-matrix.md). Where this
record and the matrix disagree, this record states the current decision and
the matrix section is marked superseded by it.

## Delivery path

| Item | Value |
| --- | --- |
| Delivery branch | `parity/termrock-visual-baseline` (PR #38) |
| Delivery PR | https://github.com/tailrocks/terminal-components-claude/pull/38 |
| Base branch | `main` (never pushed to directly) |
| Read-only product reference | `design/interaction-polish` (PR #26) |
| Frozen visual oracle | tag `visual-baseline` |

## Fixed reference revisions

Unchanged. A change to any of these needs a documented reference decision
recorded in this file.

| Reference | Revision |
| --- | --- |
| PR #26 source | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` |
| Baseline commit | `4169783ae883788186d161af09ba9c481bd84e5c` |
| Baseline tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` |
| Tuiscotti | `a1ea27f757d75d07b05499276cd932e9f8e067ad` |

## Starting state of this coordinator run

| Record | Value |
| --- | --- |
| PR #38 tip when work started | `3196f4ef6` (chore(xtask): register shell OWN-14 invariants, clear stale editor exceptions, fix doc-inventory fences) |
| Audit base commit used by this run's read-only auditors | `40bc297e5` (docs: record independent form chooser review) |
| Branch tip observed at coordinator resume | `5478ecfc9` (Merge PR #38 chooser fix into picker verification) |

`origin/parity/termrock-visual-baseline` advanced during this run from
concurrent writers already active on the branch. Every increment in this
record is rebased onto the then-current remote tip before it is pushed, and
every push takes the shared push lock described in `AGENTS.md`.

## Acceptance gates

Both requirements must pass. Neither substitutes for the other.

| Requirement | Gate | Status |
| --- | --- | --- |
| Exact product match | Paired reference capture, reference replay, candidate comparison over the full reference inventory of all four applications, twice, same result | pending |
| Reusable library | Termrock builds and tests with no application dependency; a separate consumer compiles against public APIs only; shared components demonstrated in real application flows | pending |

## Reference inventory (the acceptance surface)

From `baselines/tuiscotti-v1/corpus-index.json` (schema
`termrock-spec/tuiscotti-corpus-index-v1`):

| Application | Screens | Captures | Artefacts |
| --- | --- | --- | --- |
| holla | 125 | 3125 | 3125 x 10 |
| showcase | 84 | 2100 | 2100 x 10 |
| jackin | 49 | 1225 | 1225 x 10 |
| tablepro | 44 | 1100 | 1100 x 10 |
| Total | 302 | 7550 | 75500 |

Every screen is captured at 5 sizes (`72x20`, `80x24`, `100x30`, `120x40`,
`160x50`) and 5 colour modes (`truecolor`, `256`, `16`, `nocolor`, `none`).
Layout of one capture directory:
`baselines/tuiscotti-v1/<app>/<screen>/<variant>/<size>/<mode>.{ansi,ascii,frame.json,html,manifest.json,observations.json,png,txt}`.

Oracle provenance recorded in the index: `historical_oracle_tag =
visual-baseline`, `historical_oracle_commit = 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`,
`reference_app_sha = 7bd6a331721737514a2477c894d922cb262ef07b`,
`tuiscotti_source_sha = a47c9aaefb34e4c00026f99d8a8dd7ee5916b274`.

`baselines/oracle/ORACLE.toml` is still `pending = true` with zero imported
scenarios; the frozen baseline therefore still resolves through `snapshots/`.
Closing that gap is part of the "frozen artefacts stay outside production
dependencies" decision D-003 and the oracle import path
`cargo xtask visual import --oracle | seal`.

## Machine-checked acceptance state

`python3 tools/acceptance_architecture.py` is the fail-closed architecture
gate. At revision `64afb8a1f` it reports:

| Field | Value |
| --- | --- |
| `architecture_checks_passed` | true |
| `findings` | 0 |
| `dependency_violations` | 0 |
| `unaccounted_inputs` | 0 |
| `source_inputs` | 631 |
| `application_shared_closure_missing` | 0 |
| `acceptance_complete` | false (the bounded checker never attests acceptance) |
| `unresolved` | 5 |

The five unresolved requirements are declared in
`tools/acceptance_policy.json` and are the concrete work plan for this run:

1. full real-CLI workflow inventory and exact paired comparisons
2. independent source-bound semantic review of application thinness and
   behavior ownership
3. full workspace nextest, clippy, and formatting results bound to this source
   manifest
4. isolated production-library build and nextest result
5. reviewed complete production-input coverage including generated inputs

Requirement 1 is bounded by the reference inventory above: 302 screens across
four applications, each at 5 sizes and 5 colour modes.

The paired comparison harness lives in
`crates/termrock-conformance/tests/visual_baseline/`. Its store integrity
check (`main.rs`), the ten-artefact bundle writer, the decoded-frame
comparison (`support.rs::compare_bundle_dirs`, `check_baseline_bundle`) and
the per-application journey and pending suites are the mechanism. The
`#[ignore = "… run with --ignored"]` marker on a test means the case exists
in the inventory but has not been admitted yet.

Current inventory backlog: 351 ignored tests across
`crates/termrock-conformance/tests/visual_baseline/`, of which the per-app
pending suites are:

| App | Pending files | Ignored tests | Reference screens |
| --- | --- | --- | --- |
| holla | `holla_pending_h1..h7` | 124 | 125 |
| jackin | `jackin_pending_j1..j2` | 47 | 49 |
| showcase | `showcase_pending_s1..s5` | 94 | 84 |
| tablepro | `tablepro_pending_t1..t2` | 28 | 44 |

The remaining ignores are journeys, pointer, audit and support checks.

## Coverage gaps

Filled from the audit round; see the sections below.

## Decisions

See `## Decisions log`.

## Evidence

See `## Evidence index`.

## Decisions log

### D-001 — Delivery stays on PR #38

All increments land on `parity/termrock-visual-baseline` through the shared
push lock. `main` is never pushed to directly; `design/interaction-polish`
(PR #26) and the `visual-baseline` tag stay unchanged.

### D-002 — False parity is removed, not renamed

Application paths that reproduce a reference screen by replaying captured
output, by branching on exact test sizes, fixture labels or captured clock
values, or by repainting a shared widget, are deleted and replaced by real
component composition. They are not moved into Termrock and not renamed.

### D-003 — Frozen artefacts are test-only

Reference frames stay outside production dependencies. A production build
must not be able to consume reference frames through source, generated code,
resources or build scripts.

## Evidence index

Populated as increments land.
