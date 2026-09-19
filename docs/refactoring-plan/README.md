# Refactoring plan navigation

Status: **NO-GO; preparation only; `.campaign/ledger.json` is `armed: false`**
(2026-09-19).

This directory is an evidence-only preparation package for
`refactor/holla-parity`. It does not authorize `/goal`, task dispatch,
implementation work, ledger arming, push, or merge.

The current source candidate at the start of this documentation repair is:

- HEAD `211c29adca147d42f8bfce428af1394f0213a22d`;
- tree `3c3b7ea695c5e56d6d7f5c6783456382dff0e434`;
- parent `6a57d2b0bf461a520d22bc0b840ab17dc6ad52f8`.

Commit `211c29ad` is the current proof lifecycle fix. This repair is a later
documentation commit, so neither this file nor the other canonical documents
self-attest the final HEAD/tree.

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

Layer 2 is this evidence-only docs package. Its package ancestor is commit
`3b79d3403d52ededca087d62dccb9ad4474c105b` with tree
`d6b85e1feb8e89f25cae6db360b92f071a4c4f43`; the preceding sealed candidate
was `6a57d2b0bf461a520d22bc0b840ab17dc6ad52f8` with tree
`1ef2734ebb58b65fba670dfdd460bf189ce15cfa`. The old final-seal root binds
that preceding candidate and is invalid after `211c29ad`:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`

Final sealing for the current source line is external and predeclared at:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed-211c29ad`

Only the final verifier’s manifest from that run root may bind the actual
final HEAD/tree, task contracts, qualified tools, oracle identities, and
receipts. This docs package does not self-attest the final tree identity. Any
subsequent relevant edit invalidates the final sealing run and requires a new
external verification.

The complete evidence index is
[`evidence/current-preparation-2026-09-19.md`](evidence/current-preparation-2026-09-19.md).

This documentation update owns only the four canonical documents and generated
`planning-artifacts.tsv`. It does not change product source, task manifests,
proof code, task semantics, the ledger, protected baseline refs or snapshots,
run evidence, or generated expected artifacts.

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

## Current supplemental evidence — not receipts

Popper’s external closeout for the current candidate remains explicitly
**REJECTED** for the stale final seal, readiness preflight, dispatcher, and
calibration:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-211c29ad-independent`

The `211c29ad` proof repair closes the observer FD/process lifecycle. Popper’s
parallel raw nextest reported exit `0`, 27 passed, and one historical `LEAK` at
`compare::tests::runner_context_rejects_comparator_report_outside_runtime_outputs`;
its serialized raw run reported 27/27 clean. Keep that old LEAK disclosed as
non-reproducible raw evidence, not a pass. Franklin’s fresh supplemental
default-parallel and serialized refactor-proof nextest runs both exited `0`
with 27/27 passed. These runs do not create a receipt or clear the rejected
closeout:

- `/tmp/refactor-proof-raw.wpWzhX/default-parallel-raw.log`
- `/tmp/refactor-proof-combo.BfVzDT/parallel-combo.log`
- `/tmp/refactor-proof-combo-serialized.1jbqpR/serialized.log`

Hume established that STYLE_TIMING intentionally freezes 11 assets and that
runner index/extensions belong to the separate RUNNER corpus of 33 assets.
Both freeze checks reported `changed: 0` and `written: false`, and TASK-072
standalone taskfmt lint passed. The copied TASK-072 style-timing runner
self-test still failed with `FileNotFoundError` for sibling
`runner-bootstrap-index.py`; preserve that raw failure and never call it a
pass. It is an auxiliary retired/out-of-scope diagnostic, not a TASK-072 check
or readiness blocker. The old-root raw evidence is supplemental only.

## Current blockers

- TASK-001 dispatcher verify exits `1` (`RESULT FAIL`) on scope/forbidden-path
  checks and the intentional NO-GO checks; preflight exits `1` fail-closed.
- No accepted current final-sealing manifest is claimed here. The old
  `verifier-final-sealed` root binds `6a57d2b` and is stale; final HEAD/tree
  identity remains for the external verifier at
  `verifier-final-sealed-211c29ad`.
- The copied TASK-072 style-timing runner self-test remains a visible,
  unqualified, auxiliary retired/out-of-scope diagnostic. It is not hidden or
  counted as a pass, and it is not a readiness blocker.
- Popper’s current-candidate closeout is **REJECTED**. Franklin’s supplemental
  no-leak replay is non-receipt evidence and does not change that verdict.
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
