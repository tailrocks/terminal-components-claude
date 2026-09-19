# Refactoring plan navigation

Status: **NO-GO. Preparation only. The campaign ledger remains `armed: false`.**

This directory is the canonical preparation package for `refactor/holla-parity`.
It does not authorize the implementation `/goal`, production task dispatch,
ledger arming, a push, or a merge. The sole readiness authority is
[`execution-readiness-report.md`](execution-readiness-report.md). A NO-GO in
that report is fail-closed.

## Final source payload and documentation seal

The final source payload being documented is:

| item | identity |
| --- | --- |
| branch | `refactor/holla-parity` |
| source payload commit | `74e4ec458e2d8b41257232900bdf511bfa335730` |
| source payload tree | `f14b129677ccc493de52cca25d85d14c0f413823` |
| source payload parent | `4a95fcdeedb8f7a3a132162e536ca28c2404b823` |

The fresh preparation roots below bind this commit/tree unless explicitly
marked otherwise. This documentation-only reconciliation creates a new Git
commit and tree; it is not included in those roots. After this commit, a fresh
readiness seal must bind the post-documentation HEAD/tree before acceptance.
Any further relevant source, documentation, contract, tool, oracle, or
environment change invalidates affected evidence. The pre-documentation
inspection at `bb574d84` / `6e464830` is historical provenance only.

## Trust-path repair and independent review

Bernoulli (`gpt-5.6-luna`, max reasoning effort) independently rejected the
prior preparation tree `21f875318f61c30f55bf0a47b8daa4326d48540a7` after
reproducing two trust defects: taskfmt accepted a symlinked final or parent
alias, and the Rust verifier accepted symlinked parent components. Raw review
evidence is retained at:

```text
/tmp/campaign-review-evidence-21f87531.7EkYOO/
```

The defects were repaired, but this historical repair evidence is not
final-payload approval:

- `8e783592afd0a2c2f08076858a386a091a35e712` rejects taskfmt final and parent
  aliases in preflight/dispatch and adds adversarial coverage. Evidence:
  /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-path-repair-2026-09-19.
- `bb574d84bf25ff9179b42e951fca52068f2ab623` rejects symlinked Rust verifier
  trust-path parents while preserving the documented macOS system symlinks.
  Evidence:
  /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/symlink-parent-repair-2026-09-19.
  `final-identity.txt` binds that repair to `bb574d84` and tree `6e464830`;
  sealed nextest, Clippy, and rustdoc exit files are all zero.

The independent rejection is closed as a preparation defect only. A separate
final verifier and reviewer must still evaluate the exact post-documentation
tree and return `VERIFIED`; no such approval exists.

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

Current source-payload evidence is raw preparation evidence, not an accepted
receipt for the post-documentation tree:

- proof build:
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/proof-final-74e4ec45-2026-09-19`;
  receipt JSON binds `74e4ec458e2d8b41257232900bdf511bfa335730` and
  `f14b129677ccc493de52cca25d85d14c0f413823`; binary SHA-256
  `88c5340476e1fbaa2e424d97db75b4c323735f4bf3bde7dea1d915b940d6c012`;
  receipt SHA-256
  `98e6fe9d090391c9cb28b8cb6fb710ac5c94d2120a2bf39190b143fcb064a86f`;
- proof nextest:
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/proof-nextest-74e4ec45-2026-09-19`;
  identity binds the final payload and `nextest.log` records `cargo nextest:
  28 passed (2 binaries, 21.527s)`, exit 0;
- static:
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/static-74e4ec45-2026-09-19`;
  fmt, Clippy, and rustdoc each exit 0;
- shell:
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/shell-74e4ec45-2026-09-19`;
  Bash syntax, ShellCheck, shfmt, preparation guards, proof-path guards, and
  dispatch authorization each exit 0;
- catalog:
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/catalog-74e4ec45-2026-09-19`;
  plan exit 0, 73 packages linted with qualified taskfmt and zero failures,
  rebundle exit 0;
- documentation:
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/docs-74e4ec45-2026-09-19`;
  actionlint and Lychee each exit 0.

Workspace nextest is separate raw evidence:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/workspace-nextest-sealed-final-2026-09-19`
records `3,342 passed` and exit 0, but its identity file binds the previous
source commit `4a95fcdeedb8f7a3a132162e536ca28c2404b823` and tree
`f98f4908506f55026a35ea4e2701b78c240d147c`. It ran before this documentation
change; the docs-only edit does not change source behavior, but this is not a
final-tree receipt and must not be relabeled as one. The older `3,359 passed,
1 failed, 6 skipped` result is historical only. No preparation receipt exists.

Superseded pre-documentation paths remain useful only as historical provenance:
the old `27/27` and `27 passed` proof-nextest logs, proof build
`proof-preparation-final-2026-09-19` with binary SHA-256
`f85400a16dc131fe0f59bfc90b5ec22cadf97de02833a216ed81385bf70a1b44`,
`static-final-2026-09-19`, `shell-final-current-2026-09-19`,
`docs-final-current-retry-2026-09-19`, and
`taskfmt-lints-final-retry-2026-09-19` are not current evidence. Full taskfmt
source integration tests remain unqualified because the Docker-only path was
not run and fails closed without its opt-in. Native Linux evidence is
unavailable. `xtask boundary` still fails because `parity/evidence.tsv` is
missing.

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
native proof/result/receipt qualification; a clean deterministic 892-source
calibration and altered-output negative control; current behavioral/PTY
evidence; the missing parity evidence contract; full workspace and platform
gates; Linux; absent `TASK-071`/`TASK-072` dependency receipts; and unresolved
product refactoring. The 892 control covers 302 cases with 298 passed, 4
failed, 2 skipped, 1 leaky, exit 100, 7,550 ANSI/plain/PNG/HTML artifacts
each (30,200 total), and 17 diff files. This is coverage evidence, not a
calibration pass. The final preparation remains NO-GO, no accepted preparation
receipt exists, the ledger stays `armed=false`, and the next goal remains
**NOT AUTHORIZED FOR EXECUTION**.

**Decision: NO-GO.**
