# Refactoring plan navigation

Status: **NO-GO. Preparation only. The campaign ledger remains `armed: false`.**

This directory is the canonical preparation package for `refactor/holla-parity`.
It does not authorize the implementation `/goal`, production task dispatch,
ledger arming, a push, or a merge. The sole readiness authority is
[`execution-readiness-report.md`](execution-readiness-report.md). A NO-GO in
that report is fail-closed.

## Source identity at documentation start

The four-document repair was started only after reading the live checkout:

| item | identity |
| --- | --- |
| branch | `refactor/holla-parity` |
| HEAD | `f6f94dc995f5b6451800d739174d7f23802a40c3` |
| HEAD tree | `61563f7b48d0fe7b8e5bdddae57072e96f6a6f12` |
| parent | `e8c4950928b0ab6cc1268777dbed6f96cb0309ba` |
| preparation commits | `e8c49509` proof static repair; `f6f94dc9` shell/preparation guards |
| local `main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| `origin/main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| `origin/refactor/holla-parity` | `f5013f609aed1ba32ce60352b38fd0b1b11b063c` |
| merge-base with `main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| merge-base with protected baseline | `cc14dd6beae526884aabdf897e309be837b4f504` |

At that point `main` was an ancestor of the campaign, the campaign was 75
commits ahead of local `main` and 31 commits ahead of its remote campaign ref,
and no push or ref mutation had occurred. `.worktrees/main` is not used to
identify `main`; Git refs are authoritative. This document commit changes the
tree, so the post-commit source identity must be sealed externally and any
pre-commit evidence must be re-bound before acceptance.

## Authority and navigation

Resolve conflict in this order: repository policy and user scope; the frozen
visual oracle; current source, tests, contracts, and Git evidence; this report
and the campaign contracts; historical notes. Historical evidence cannot
override current source or authorize execution.

- [`execution-readiness-report.md`](execution-readiness-report.md) — current
  verdict, blockers, and acceptance gate.
- [`evidence/current-preparation-2026-09-19.md`](evidence/current-preparation-2026-09-19.md)
  — command/evidence index and provenance ledger.
- [`next-implementation-goal.md`](next-implementation-goal.md) — complete
  future prompt, explicitly **NOT AUTHORIZED FOR EXECUTION** while this report
  is NO-GO.
- [`campaign-policy.md`](campaign-policy.md) — branch, scope, and protected
  ref rules.
- [`subagent-only-policy.md`](subagent-only-policy.md) and
  [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — native
  roles, isolation, integration, and evidence sequence.
- [`path-contract.md`](path-contract.md) and
  [`proof-contract.md`](proof-contract.md) — native paths, trust inputs, and
  result/receipt binding.
- [`campaign-ledger.schema.json`](campaign-ledger.schema.json) — ledger shape.
- [`task-graph.json`](task-graph.json) — generated structure only; it contains
  no authorization or acceptance result.
- [`task-graph.md`](task-graph.md) and [`task-index.tsv`](task-index.tsv) —
  reconciled catalog and dependencies.
- [`../../refactoring-tasks/visual-validation.md`](../../refactoring-tasks/visual-validation.md)
  — exact visual/behavioral comparison contract.
- [`architecture.md`](architecture.md), the branch-diff reports, and the root
  architecture documents — product and target-architecture obligations.

## Frozen baseline and oracle

The protected identity is the peeled tag, not a branch name or release label:

| artifact | immutable identity |
| --- | --- |
| `refs/tags/visual-baseline^{commit}` | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| baseline commit tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| `snapshots/` tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| matrix keys | 7,550 |
| artifacts | 30,200: 7,550 each ANSI, plain text, PNG, HTML |

The exact read-only import and its independent manifest are external:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19
```

The import records the tag commit/tree, snapshot tree, 30,200-file hash
manifest, four 7,550-key lists, and zero symlinks. It is read-only. No
candidate output has been used as an expected artifact.

Three identities must not be conflated:

1. `4a79c0a2` is the protected artifact/oracle commit.
2. `89218626011f2f82c4e87c4dfd5868a4c5f3e284`, tree
   `6fccf997cd742071ebcff0e0a00e89404ef95ca8`, is the Git source identified
   as the snapshot producer. Its `snapshots/` tree is the same
   `3f0261c32849e26feda24d87697de4a7ce6b8375`, with all 30,200 artifacts. Its
   external control root is
   `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19`.
3. `02f5294bfdbf38004cc49130d0aff1d01f31434c` (tree
   `efa2b409b77077caf5c639f7f4c6154cbadbce5`) remains the distinct
   architecture/source oracle. It is not the visual artifact tag.

The earlier `3570a2ed23444dddf1eddcdcc49b654b169038fe` claim is corrected, not
authoritative: its tree is `77d6a559536d6a1d733b52d3b78b5b49112315cb` and its
snapshot corpus is only 28,580 artifacts / 7,145 keys. It cannot calibrate the
complete gate.

## Qualified tools and checks

The only qualified standalone taskfmt is:

```text
source: /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
version: 0.2.0
binary: /tmp/taskfmt-latest-install/bin/taskfmt
binary SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Only `taskfmt lint "$TASK_DIR"` and the documented standalone `taskfmt verify`
form are permitted. Taskfmt is not an orchestrator, workspace manager, ref
manager, container launcher, or acceptance authority.

Observed preparation payload checks include plan/DAG validation, 73/73
standalone lints, 27/27 `refactor-proof` nextest tests after `e8c49509`, strict
proof-code Clippy/rustdoc after `e8c49509`, and the shell guard/path checks in
`f6f94dc9`. These are source-payload observations, not a final receipt for the
post-documentation tree. Full workspace nextest had a parallel reliability
failure (`3,359 passed, 1 failed, 6 skipped`); an isolated rerun passed only
the affected architecture test. `xtask boundary` still fails because
`parity/evidence.tsv` is absent. Full taskfmt source integration tests are not
qualified: the Docker-only test path was not run and fails closed when its
required opt-in is absent. Native Linux evidence is unavailable.

## Task and architecture state

The generated catalog has 73 packages (`001`–`073`), 506 checks (27 `argv`,
479 `shell`), 276 dependency edges, and maximum dependency depth 35. Plan
validation reports 1,174 source obligations, 3,256 traceability rows, and zero
structural errors. All package lints passed; no production task is accepted.

- `TASK-001` and `TASK-070` are retired/non-qualifying fail-closed historical
  lifecycle tasks. They are not dispatchable implementation work.
- `TASK-071` and `TASK-072` remain blocked qualification prerequisites until
  fresh, independently reviewed receipts exist.
- `TASK-002`–`TASK-069` and `TASK-073` contain the remaining implementation,
  ownership, behavior, visual, performance, and closure obligations.

The target architecture is caller-owned state, borrowed props, read-only draw,
runtime-owned routing/focus/layers/pointer/cursor, and one reusable owner per
component family. The current product still has duplicate/compatibility
painters and missing application behavior. Examples are the Showcase grid and
chrome, Jackin projections, and TablePro tree/filter/frame painters. These are
future implementation obligations, not preparation changes.

## Evidence and blockers

The evidence index preserves raw paths, rejected historical verifier/reviewer
roots, calibration roots, and invalidation rules. Important rejected roots are:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed-proof-requal-b20ca5c6
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/reviewer-final-proof-requal-b20ca5c6
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-211c29ad-independent
```

They are rejected evidence only. They do not attest to this tree, issue a
receipt, or authorize GO. No independent final verifier/reviewer has returned
`VERIFIED` for the post-documentation tree.

Preparation blockers remain: a clean post-documentation final seal; complete
native proof/result/receipt qualification; a complete 892-source control
replay with exact 30,200-artifact coverage and negative controls; current
behavioral/PTY evidence; the missing parity evidence contract; full workspace
and platform gates; Linux; and unresolved product refactoring. The ledger stays
disarmed and no production task has been dispatched.

**Decision: NO-GO.**
