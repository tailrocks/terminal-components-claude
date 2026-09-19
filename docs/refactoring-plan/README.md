# Refactoring plan navigation

Status: **NO-GO; preparation only; `.campaign/ledger.json` is `armed: false`**
(2026-09-19).

This directory is an evidence-only preparation package for
`refactor/holla-parity`. It does not authorize `/goal`, task dispatch,
implementation work, ledger arming, push, or merge.

## Two-layer sealing protocol

Layer 1 is the verified preparation payload, retained as historical/tested
evidence only. It is **never** the current final-tree binding:

- payload commit `14aa8ed0469219ff8f6570be7824e5ade39240cc`;
- payload tree `f3ef0f6badd01161bc24cdfef5161db55ba5d579`;
- payload parent `96c6c475b5d22193b7539565fa5b4612a88ea007`;
- tested verifier run
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-14aa8ed-lagrange`;
- tested verdict `VERDICT.md`: **REJECTED**.

Its proof qualification subchecks pass, but TASK-001 dispatcher verification
exits `1` with `RESULT FAIL`, readiness preflight exits `1` at the explicit
NO-GO gate, and the existing full calibration is not clean. These facts are
historical/tested payload evidence, not current final-tree evidence.

Layer 2 is this evidence-only docs package. Its package parent is commit
`3b79d3403d52ededca087d62dccb9ad4474c105b` with tree
`d6b85e1feb8e89f25cae6db360b92f071a4c4f43`. Final sealing is external and
predeclared at:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`

Only the final verifier’s manifest from that run root may bind the actual
final HEAD/tree, task contracts, qualified tools, oracle identities, and
receipts. This docs package does not self-attest the final tree identity. Any
subsequent relevant edit invalidates the final sealing run and requires a new
external verification.

The complete evidence index is
[`evidence/current-preparation-2026-09-19.md`](evidence/current-preparation-2026-09-19.md).

This documentation update changes only the four canonical documents named in
the preparation request. It does not change product source, task manifests,
proof code, the ledger, protected baseline refs or snapshots, or generated
expected artifacts.

## Authority

[`execution-readiness-report.md`](execution-readiness-report.md) is the sole
current readiness authority. Its **NO-GO** controls whether the campaign can be
armed or executed.

[`next-implementation-goal.md`](next-implementation-goal.md) is the complete
future `/goal` prompt. It is labeled **NOT AUTHORIZED FOR EXECUTION** and must
reject NO-GO, stale evidence, unqualified calibration, and `armed: false` at
startup.

Current contracts:

- [`campaign-policy.md`](campaign-policy.md) — branch, scope, and protection
- [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — isolated roles and sequence
- [`path-contract.md`](path-contract.md) — native paths and external run data
- [`proof-contract.md`](proof-contract.md) — context, observer, result, and receipt rules
- [`task-format.md`](task-format.md) — qualified standalone taskfmt
- [`subagent-only-policy.md`](subagent-only-policy.md) — native macOS subagents
- [`campaign-ledger.schema.json`](campaign-ledger.schema.json) — ledger schema
- [`task-graph.json`](task-graph.json) — generated dependency structure only
- [`../../refactoring-tasks/visual-validation.md`](../../refactoring-tasks/visual-validation.md) — visual comparison contract

Lychee is the documentation verifier. It does not orchestrate agents, arm the
ledger, start containers, or grant execution authority.

## Fixed preparation identities

The protected oracle is the peeled `visual-baseline` tag commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, tag ref object
`1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, baseline tree
`0b1f13431fdfd6060cf9f45a114afa5a99cc6c26`, and snapshot tree
`3f0261c32849e26feda24d87697de4a7ce6b8375`. Its unchanged inventory is
7,550 matrix keys and 30,200 artifacts: 7,550 each of ANSI, plain text, PNG,
and HTML, across five terminal sizes and five color modes.

The only qualified taskfmt is `/Users/donbeave/Projects/taskfmt/task-format` at
revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, version `0.2.0`, binary
`/tmp/taskfmt-latest-install/bin/taskfmt`, SHA-256
`f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`.
Only standalone `lint` and `verify` are permitted.

The Layer-1 tested native proof build is schema `tc-proof-native-build/v1`,
binary SHA-256
`a3b7712ab7c3ea22940ffe915a2328d25767d35e5253e9df82776db0d3b70fcc`, bound
to the historical/tested Layer-1 payload, not the final docs tree. Its
qualification run
passed 26/26 locked nextest tests, 72/72 comparator cases over 141 invocations
with 0 failures, all 7 external-binding negative controls, native observer
launch/validate, plan/DAG/path/freeze checks, and 73/73 taskfmt lints. Those
passes do not override the Layer-1 **REJECTED** verdict or seal Layer 2.

## Current blockers

- TASK-001 dispatcher verify exits `1` (`RESULT FAIL`) on scope/forbidden-path
  checks and the intentional NO-GO checks; preflight exits `1` fail-closed.
- No accepted Layer-2 final-sealing manifest is claimed here; final HEAD/tree
  identity remains for the external verifier at `verifier-final-sealed`.
- The auxiliary copied-runner style-timing self-test fails with
  `FileNotFoundError` for sibling `runner-bootstrap-index.py`. It is recorded
  as **unqualified**, visible evidence; it is not hidden or counted as a pass.
- Calibration remains 302 selected, 301 passed, 1 failed, and 2 skipped. The
  sole failure is
  `tablepro_connections_form_advanced_120x40_truecolor`: 24/100 artifacts
  match and 76/100 mismatch. Independent history establishes a mixed frozen
  oracle state; no oracle output may be changed to reconcile it.
- The ledger remains `armed: false`; no task was dispatched, no receipt was
  issued, and no push or merge occurred.
- All 73 task manifests remain unaccepted. The catalog has 506 checks, 276
  dependency edges, and maximum DAG depth 35. Consumer migration, duplicate
  renderer/compatibility-painter removal, component ownership, behavior, PTY,
  performance, API, static, documentation, platform, and complete visual
  parity remain obligations.

## Historical records

Retired reports, prompts, runbooks, and coordination files are provenance
only. They cannot override this report, current contracts, deterministic proof,
or the protected oracle. Do not replay their old taskfmt, container, or
baseline commands. The old `docs/sources/PLANNING_GOAL.md` path is absent; the
immutable baseline copy is the only historical reference when needed.

Never move, retarget, recreate, or write to the `visual-baseline` tag, branch,
release, grouped store, snapshots, fixtures, or expected artifacts. Provider
immutability is not assumed; repository policy is the protection boundary.
