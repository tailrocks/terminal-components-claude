# TASK-078 binding obligations

Each row maps one production account-tests failure to the prepare binding
that must satisfy it. All bindings are emitted by `tc-proof prepare` from
the task manifest, the oracle, and the candidate worktree; the fail-closed
negative path is evidenced by a colocated Rust unit test because prepare
must refuse to emit a context there. Field details live in the trusted
`receipt-bind` driver, which is the exact independent judge.

## O-001 — Production receipt projection (CHK-002, R-001, AC-001)

Demand: `tools/refactor-proof/accounting/extension.py::validate_extension_event`
production branch. Today `bind_accounting_preparation` never inserts
`accepted_inventory`/`accepted_disposition`; production-mode accounting
checks (catalog tasks 009-069 plus 073 CHK-005 declare
`mode=production` with both requires flags true) then fail with
`TEST_ACCOUNTING` on the truthiness gate.

Obligation: the production account-tests context lifts the CHK-002
template declarations exactly (`worker_context`, `template_sha256`,
`mode`/`family`/`register_kind`/`requires_*`), keeps the accepted
TASK-075 `original`/`required` inventory intact, and — only in
production mode, only when the matching requires flag is true — binds
`accepted_inventory` (for `requires_inventory_receipt`) and
`accepted_disposition` (for `requires_disposition_receipt`) as
non-empty provenance objects. Each claim carries a `receipts` list of
exactly `{task_id, sha256, integration_commit}` entries (no `path`, no
extra keys) whose set equals the bound `qualification.common.dependencies`
stubs: same task ids, same receipt `sha256`, same `integration_commit`.
Any entry outside the bound stub set, any digest/commit mismatch, or any
accepted claim without its requires flag fails.

## O-002 — Fail closed when required but unbound (reviewer nextest, R-002, AC-001)

Demand: a required receipt that is not bound must never be fabricated or
defaulted; prepare must refuse the context. No prepared context can
exhibit this path, so the driver cannot assert it.

Obligation: the implementer colocates a Rust `#[test]` in
`tools/refactor-proof/src/verifier.rs` `mod tests` that drives the
production accounting binding with a requires flag true and the
corresponding dependency receipt unbound, and asserts prepare returns an
error (no context, no accepted claim). The reviewer runs
`cargo nextest run -p refactor-proof --lib <filter>` from the candidate
worktree with a filter matching the new test, and records the command
plus its passing output as R-002 evidence. No verify.toml shell runs
cargo; the deterministic driver shell stays read-only on contexts.

## O-003 — Preparation mode unchanged (CHK-003, R-003, AC-002)

Demand: `bind_accounting_preparation` preparation branch plus the
TASK-075 accepted inventory. The repair must not change any binding
outside the production accepted-claim projection.

Obligation: the preparation account-tests context lifts the CHK-003
template declarations exactly, keeps `original` (64-hex digests plus
`assertions`), the five-key source-derived `required` inventory, an
empty `future`, and `preparation_register == {required, future}`
byte-identical in behavior, and still strips both accepted claims.
Every other binding prepare already emits — identity, trust, tool,
dependency, index, result, template-flow, manifest-agreement, and
task-derived membership — is re-asserted unchanged by every profile.

## O-004 — Forbidden redirections (CHK-001/CHK-004, R-004, AC-003)

The fix lives in `tools/refactor-proof/src/verifier.rs`
`bind_accounting_preparation` (and its direct receipt-projection
helpers) plus narrow directly necessary colocated tests only.
Forbidden: any `refactoring-tasks/**` change (in particular no
template, driver, loader, or manifest edits inside this or any other
task package), any `tools/refactor-proof/{accounting,architecture,bin}/**`
change owned by TASK-071/TASK-072, any `runner/*.py` change (TASK-074's
accepted glue), any demand-side weakening (`src/compare.rs`, `src/bin/`,
validators), any hardcoding of digests, receipts, commits, frames,
states, or membership to satisfy the judge, and any fabrication of an
accepted claim from anything other than the bound `common.dependencies`
stubs. Taskfmt scope + forbidden-path gates enforce this; the reviewer
confirms it.
