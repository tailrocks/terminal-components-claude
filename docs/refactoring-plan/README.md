# Refactoring preparation authority

Status: **NO-GO.** Preparation only. The ledger must remain `armed: false`.
No production task may be dispatched, and this branch must not be merged into
`main`, while the readiness report is NO-GO.

This directory is the canonical preparation package for
`refactor/holla-parity`. The authority order is:

1. repository policy and the user’s preparation boundary;
2. the immutable `visual-baseline` oracle;
3. current source, tests, contracts, Git history, and executable evidence;
4. [`execution-readiness-report.md`](execution-readiness-report.md);
5. task descriptions and historical reports.

Historical evidence never authorizes execution. The generated structural graph
contains no status, acceptance result, or dispatch authority.

## Current candidate and branch truth

The latest tested preparation payload before this final documentation freeze is:

```text
branch:   refactor/holla-parity
commit:   bc4e5980256f1fa2c66d673790c99610b610fd16
tree:     1a388126806c701ff4420f17023a8b792bfd4b98
parent:   1da58a09f420b653195b5d8015ed5ca22deb8a6a
local main:   7b27732a8c3c131760ec3438f641cb3c11343a42
remote main:  7b27732a8c3c131760ec3438f641cb3c11343a42
remote campaign tip: 1da58a09f420b653195b5d8015ed5ca22deb8a6a
merge-base with main: 7b27732a8c3c131760ec3438f641cb3c11343a42
campaign commits ahead of origin/refactor/holla-parity: 1
```

The campaign is a descendant of actual local and remote `main`; the remote
campaign ref is stale relative to this local preparation branch. No ref was
reset, rewritten, force-pushed, pruned, or moved. The directory name
`.worktrees/main` was not used to identify `main`.

The four canonical documents are source-controlled preparation metadata. Their
commit changes the tested payload, so the final verifier must bind the exact
post-documentation HEAD/tree. Evidence is invalidated by any relevant source,
documentation, task contract, schema, script, tool, oracle, environment, or
generated-output change. The evidence-sealing protocol is therefore:

```text
commit preparation payload
→ commit canonical documentation
→ freeze the final tree
→ generate external verifier/reviewer reports for that exact tree
→ record their paths and hashes without another tracked edit
```

## Frozen visual authority

The protected oracle is the peeled tag, not a branch or release label:

| identity | value |
| --- | --- |
| `refs/tags/visual-baseline^{commit}` | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| tag commit tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| `snapshots/` tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| matrix keys | 7,550 |
| artifacts | 30,200: 7,550 ANSI, 7,550 plain, 7,550 PNG, 7,550 HTML |

The exact read-only import is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`.
Its independent manifest is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19/sha256.manifest`;
SHA-256:
`95e1f38220bd2fd09da44d3b98590543d1f03837b50e0069bf53bd1f73893637`.
The import has no symlinks. Protected refs, snapshots, grouped stores,
fixtures, and expected artifacts were not changed.

The complete snapshot-producing source is commit
`89218626011f2f82c4e87c4dfd5868a4c5f3e284`, tree
`6fccf997cd742071ebcff0e0a00e89404ef95ca8`, with the same snapshot tree.
Its control run is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19`.
That run executed 302 cases and produced the complete count, but was not
clean: 298 passed, 4 failed, 2 skipped, 1 leaky, exit 100. No output was
blessed or normalized.

Parity is exact at cells/styles/layout/ANSI/plain/PNG/HTML and interactive
state transitions. PTY checks must include setup, input, resize, settled state,
exit, restoration, and cleanup. A compatibility painter, copied oracle frame,
or duplicate renderer cannot satisfy the gate.

## Catalog, architecture, and remaining product scope

Machine-checked catalog at the tested payload:

| measure | value |
| --- | ---: |
| direct task packages | 73 |
| recursive `verify.toml` files | 77 |
| direct checks | 506: 27 argv, 479 shell |
| recursive checks | 526 |
| dependency edges | 276 |
| maximum dependency depth | 35 |
| file-conflict pairs | 193 |
| serialization pairs | 0 |
| source obligations | 1,174 |
| traceability rows | 3,256 |
| accepted production tasks | 0 |

`TASK-001` and `TASK-070` are retired fail-closed lifecycle/bootstrap
contracts. `TASK-071` and `TASK-072` remain qualification prerequisites.
`TASK-002`–`TASK-069` and `TASK-073` remain valid implementation obligations,
subject to fresh reconciliation by the next goal.

The target architecture is caller-owned state, borrowed props, read-only draw,
runtime-owned routing/focus/layers/pointer/cursor, and one reusable
implementation per component family. Remaining obligations include Showcase
compatibility painting and fixed-grid text, Jackin projections, TablePro
legacy painters, reduced Holla route/scenario coverage, ownership migration,
application integration, behavior, visual parity, performance, and API/static
closure. Preparation did not implement product refactoring.

## Native proof and tools

Qualified standalone taskfmt:

```text
source:   /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
tree:     b7d90bd8adbe6c341a08fc485099ee8cf1584431
version:  0.2.0
binary:   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-qualified-2026-09-19/install/bin/taskfmt
SHA-256:  f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Only standalone `taskfmt lint "$TASK_DIR"` and the documented standalone
`taskfmt verify --root ... --task-dir ... --base ... --progress "" --log-dir ...`
are allowed. Taskfmt is not an orchestrator, workspace manager, ref manager,
container launcher, or acceptance authority.

The native Rust launcher/materializer now binds task/check/run/source/tree,
scope base, dependency receipts, oracle, contracts, comparator/tool hashes,
environment, observer nonce/FD transport, result locations, and immutable
receipts. It rejects forged, stale, duplicate, missing, symlinked, hard-linked,
mutated, cross-run, wrong-tree, wrong-tool, wrong-oracle, wrong-nonce, and
incomplete inputs. Observer shutdown is bounded; canonical native targets are
named `target`. Preflight and dispatch require exactly one canonical readiness
verdict and reject legacy or ambiguous markers. The supported threat model
detects integrity substitution but does not claim isolation from a hostile
same-user process.

Current qualification evidence:

- `proof-full-bc4e5980`: unfiltered `cargo nextest` exit 100; 37 passed and
  the bounded observer-hang test failed under the unfiltered run. The raw
  failure is retained; a focused rerun cannot erase it.
- `native-preparation-bc4e5980`: build, prepare, validate, and explicit
  proof-preparation preflight all exit 0; external build receipt, binary,
  contexts, results, observer, and index bind to `bc4e5980` / `1a388126`.
- `adversarial-preparation-bc4e5980`: preparation guards, proof paths,
  dispatch authorization, and ledger contract checks all exit 0.
- `taskfmt-lints-bc4e5980`: 73/73 package lints passed, 0 failed.

Raw runs are under
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19`.
They are supporting evidence, not acceptance receipts.

## Readiness and navigation

- [`execution-readiness-report.md`](execution-readiness-report.md) — sole
  current verdict and blocker register.
- [`evidence/current-preparation-2026-09-19.md`](evidence/current-preparation-2026-09-19.md)
  — command, identity, oracle, calibration, and evidence index.
- [`next-implementation-goal.md`](next-implementation-goal.md) — complete
  future prompt, explicitly **NOT AUTHORIZED FOR EXECUTION**.
- [`task-graph.json`](task-graph.json), [`task-graph.md`](task-graph.md), and
  [`task-index.tsv`](task-index.tsv) — structural catalog only.
- [`campaign-policy.md`](campaign-policy.md), [`path-contract.md`](path-contract.md),
  [`proof-contract.md`](proof-contract.md), and
  [`../../refactoring-tasks/visual-validation.md`](../../refactoring-tasks/visual-validation.md)
  — execution, trust, and parity contracts.
- [`architecture.md`](architecture.md), `COMPONENT_ARCHITECTURE.md`, `DESIGN.md`,
  `GOAL.md` — architecture/product obligations, not execution authority.

The final independent verifier and separate reviewer must inspect the clean
post-documentation tree, rerun decisive checks, inspect raw evidence, and
return explicit decisions. Their predetermined external paths are:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
```

Because exact-tag calibration, the unfiltered proof suite, platform evidence,
and product parity remain failed or unavailable, no honest preparation receipt
or GO can exist in this goal. The next goal must revalidate readiness before
any authorization or arming operation.
