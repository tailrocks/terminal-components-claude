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

The current pushed preparation payload before this documentation update is:

```text
branch:   refactor/holla-parity
commit:   247e47d5c00f83685211c98cdef1a281b2af86b4
tree:     40d76b003cfb7e6f19a818942762d9d9f82e11f6
parents:  9346c104d9a544c36bd61e48002f037fdc13423d, 28a75786
local main:   7b27732a8c3c131760ec3438f641cb3c11343a42
remote main:  7b27732a8c3c131760ec3438f641cb3c11343a42
remote campaign tip: 247e47d5c00f83685211c98cdef1a281b2af86b4
merge-base with main: 7b27732a8c3c131760ec3438f641cb3c11343a42
campaign commits ahead of origin/refactor/holla-parity: 0
```

The campaign is a descendant of actual local and remote `main`; the remote
campaign ref equals this pushed payload. No ref was reset, rewritten,
force-pushed, pruned, or moved. The directory name `.worktrees/main` was not
used to identify `main`.

The four canonical documents are source-controlled preparation metadata. Their
next commit changes the tested payload, so the final verifier must bind the
exact post-documentation HEAD/tree. Evidence is invalidated by any relevant
source, documentation, task contract, schema, script, tool, oracle,
environment, or generated-output change. The evidence-sealing protocol is:

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

At current campaign HEAD `247e47d5`, `tests/visual_baseline/` and
`.config/nextest.toml` exist. The candidate branch still has no `snapshots/`
directory and no `parity/evidence.tsv`; those remain protected external oracle
inputs, not candidate-generated expected output.

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

The native Rust launcher/materializer has implementation and contract claims
for source/tree, scope, oracle, contracts, tool hashes, observer transport,
results, and receipts. Those claims are not qualified: the independent
launcher review rejected the current visual control for protected-target use,
missing source/output freshness and link controls, accepted target/config/
manifest overrides, incomplete HTML/provenance validation, incomplete tool
identity, destructive scratch cleanup, and failed `shfmt -d`. The native
threat model remains limited to integrity/detection controls; it does not
claim isolation from a hostile same-user process.

Current and historical qualification evidence:

- Current taskfmt lint at pushed `247e47d5`: 73/73 packages passed, exit 0;
  log `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/taskfmt-lint-current-2/all.log`,
  SHA-256 `afecad09ba800c74fc841fd4d830fd8932d80e84894bd18c5bf5660522134124`.
- Current proof run at `247e47d5`: `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/refactor-proof-current-clean/nextest.log`,
  SHA-256 `629750c16dcab5fd69bc6a8c082aadbcdc10526f105f2a7f7b2db603f10b4cc6`;
  2/3 selected tests passed and the provider-hang test failed, exit 100.
- `proof-full-bc4e5980`, `native-preparation-bc4e5980`,
  `adversarial-preparation-bc4e5980`, `taskfmt-lints-bc4e5980`, and
  `final-checks-bc4e5980` are historical runs bound to superseded source;
  they remain provenance only.

The adversarial audit at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/adversarial-fe802/adversarial-proof-contract-audit-fe802.md`
(SHA-256 `e92d7f517c05992afefc0d80bc3b4aef5250b22cb17865ddfb136e12b4ab7f19`)
is rejected with AP-01–AP-05 open: observer provenance, result closure,
taskfmt sealing, trust-path consistency, and observer request-count binding.

Raw runs are under
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19`.
They are supporting evidence, not acceptance receipts.

The clean pre-arm preflight run
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/preflight-current-clean`
exited 1 with the correct NO-GO refusal. Its result SHA-256 is
`ad1e78e8c2c5c14fd068befd42fa7774d115dfd83e91f41784ca2a22e56adf12`; it is
bound to historical `9346c104`/`00958932`, so it cannot authorize the current
or post-documentation tree. Fresh pre-documentation Lychee log
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/lychee-docs-247e47d5/lychee.log`
passed with SHA-256 `2e3b6e8d8f1f9a9740000b37cb83cec558485df985dadcfd0642052b968c956c`;
qualified actionlint 1.7.12 log
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/actionlint-docs-247e47d5/actionlint.log`
passed with SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
No final-tree verifier or reviewer receipt exists.

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
