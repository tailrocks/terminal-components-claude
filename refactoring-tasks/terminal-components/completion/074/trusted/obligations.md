# TASK-074 binding obligations

Each row maps one adapter-task worker demand site to the prepared-context
binding that must satisfy it. All bindings are emitted by `tc-proof prepare`
from the task manifest, the oracle, and the candidate worktree. Field
details live in `trusted/context-bind/context-bind-protocol.md`.

## O-001 — Native oracle contract (CHK-002, R-001, AC-001)

Demand: `tools/refactor-proof/runner/operations.py::run_oracle` native path
and `tools/refactor-proof/runner/validate.py::validate_native_extension`.
Today prepare binds only `qualification.family=native`; the worker then
requires `source_sha256`, `mapping_owner == oracle_commit`,
`mapping == {footer_row, pointer}`, and `resized_sizes`, which are absent.

Obligation: the oracle context for a native namespace (here `showcase`)
carries the complete native contract alongside `family=native` and
`observer_sequence=[oracle]`.

## O-002 — Nested comparator context (CHK-003, R-001, AC-006)

Demand: `tools/refactor-proof/src/compare.rs::parse_compare_value` /
`parse_runner_value`. Today `bind_compare_qualification` emits only
schema/run/task/check/oracle-commit/tree/report-path; the comparator then
requires `oracle_root`, `candidate_root`, both manifest digests,
`required_sha256`, `actions_sha256`, `required_ids`/`required_count`, the
tool digest, both adapter digests, and the host-bound
`<runtime>/CHK-003.compare.json` report path with outer-identity agreement.

Obligation: the compare context carries the complete nested comparator
context with absolute distinct existing roots, digest shapes, required-set
coherence (`required_count == len(required_ids)`), and tool agreement.

## O-003 — Accounting preparation register (CHK-004, R-002, AC-003)

Demand: `tools/refactor-proof/accounting/dispatch.py` accounting-family path
and `tools/refactor-proof/accounting/extension.py::validate_extension_event`.
Today prepare binds only the template-declared `family` (plus common trust);
the worker then requires the lifted declarations (`mode=preparation`,
`register_kind=source-derived`, `requires_*=false`) and the derived
`original{source_sha256, non_test_sha256, assertions}`, `required`, and
`preparation_register={required, future}` with no accepted receipts.

Obligation: the account-tests context lifts the CHK-004 template
declarations and derives the complete preparation register; `worker_context`
and `template_sha256` continue to bind the template bytes exactly.

## O-004 — Architecture bindings (CHK-005, R-002, AC-002)

Demand: `tools/refactor-proof/architecture/dispatch.py::run_architecture`
base path (`validate_schema`, `observer_sequence=[architecture]`,
`configuration` with integer `seed` and `seed_steps` list) and
`runner/context.py::validate_required_members` for execution matching.

Obligation: the architecture context carries a well-formed configuration
and task-derived membership (O-005). Provider-payload agreement stays with
TASK-072 prerequisite evidence and the adapter end-to-end gates.

## O-005 — Task-derived membership (CHK-002..CHK-005, R-003)

Demand: `runner/context.py::validate_required_members` plus observer
execution matching on every worker path. Today `build_context` hardcodes
the tiny `direct/pty x 8/12 x blue/yellow` fixture axes/members for every
check; real adapter-task observations cannot match fixture membership.

Obligation: every worker-path context binds axes/members derived from the
task manifest, oracle, and worktree — well-formed (non-empty unique member
strings, non-empty axis value lists) and not the hardcoded fixture default.
Preflight/close contexts keep shape only. Exact observer-provider agreement
on member identity stays with the adapter end-to-end gates.

## O-006 — Preserved bindings (all checks, R-003)

Demand: proof-contract § native layout, `runner/__main__.py`
`bind_native_environment`, and the existing `preflight`/`required`/`close`
workers. The repair must not regress identity, trust-manifest, tool,
dependency, index, result, template-flow, or manifest-agreement bindings.

Obligation: CHK-001 re-proves the full common binding set on the current
tree, and every profile re-asserts it alongside its new bindings.

## O-007 — Forbidden redirections (CHK-001/CHK-006, R-004, AC-004)

The fix lives in prepare only: `tools/refactor-proof/src/verifier.rs` and
the `tools/refactor-proof/runner/{__main__,context,operations,validate}.py`
glue. Forbidden: any `refactoring-tasks/**` change (in particular no
template edits inside TASK-002..007 packages), any
`tools/refactor-proof/{accounting,architecture,bin}/**` change owned by
TASK-071/TASK-072, and any demand-side weakening
(`src/compare.rs`, `src/bin/`, runner/accounting/architecture validators).
Taskfmt scope + forbidden-path gates enforce this; the reviewer confirms it.
