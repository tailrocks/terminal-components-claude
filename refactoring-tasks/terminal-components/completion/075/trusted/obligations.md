# TASK-075 binding obligations

Each row maps one TASK-002 verify failure (candidate `fc955f56`, scope base
`eacea930`) to the prepare/branch binding that must satisfy it. All bindings
are emitted by `tc-proof prepare` from the task manifest, the oracle, and the
candidate worktree; the supervision result is emitted by the fixed
`runner/__main__.py` compare branch. Field details live in
`trusted/bind-repair/bind-repair-protocol.md`.

## O-001 — Real compare roots (CHK-002, R-001, AC-001)

Demand: `tools/refactor-proof/src/compare.rs::compare_roots` /
`verify_manifest` / `check_required_set` / `compare_scenario`. Today
`bind_compare_qualification` binds `oracle_root` to the git object store
(`--git-common-dir`, the `.git` dir) and `candidate_root` to the whole
candidate worktree, with domain-derived manifest digests; the comparator
then fails `CHK-004` with `UNSAFE_PATH` (symlinks under the worktree) and
missing `manifest.json`.

Obligation: the compare context carries roots that are absolute, distinct,
existing real directories, recursively symlink-free, never the worktree
root and never the git common dir. Each root holds a `manifest.json`
(`tc-proof-artifacts/v1`) whose recomputed SHA-256 equals the bound
manifest digest, `scenario_ids` equal to `required_ids`, and every listed
file present with matching size and digest. The oracle root additionally
holds `required.json` (`tc-proof-required/v1`) whose canonical SHA-256
equals bound `required_sha256`, with scenarios covering `required_ids`;
every scenario's frame/state/provenance files exist under both roots with
cross-consistent provenance (`source_tree`, `scenario_id`, `lane`,
`checkpoint`, `actions_sha256`, `tool_sha256`, side adapter digest,
candidate `run_id`/`task_id`, recomputed `binary_sha256`). No `approved`
entry exists under `candidate_root`.

## O-002 — Five-key accounting inventory (CHK-003, R-002, AC-002)

Demand: `tools/refactor-proof/accounting/extension.py::validate_extension_event`
seen-keys accounting. Today `bind_accounting_preparation` binds `required`
as one-key `[{id: member}]` rows; the worker builds seen identities as
`{package, target, profile, source_commit, name}` and fails `CHK-005` with
`TEST_ACCOUNTING` on the canonical-set mismatch.

Obligation: the account-tests context binds `required` as a non-empty
unique list of exact five-key identities
`{package, target, profile, source_commit, name}` with non-empty strings
and `source_commit` equal to the bound oracle commit (source-derived).
`preparation_register` equals `{required, future}` with an empty future,
no accepted receipts are claimed, and the CHK-003 template declarations
stay lifted exactly (`worker_context`, `template_sha256`,
`mode`/`family`/`register_kind`/`requires_*`).

## O-003 — Compare supervision emits a runner result (CHK-004, R-003, AC-003)

Demand: `tools/refactor-proof/runner/validate.py` close validation. Today
the `runner/__main__.py` compare branch `os.execv`s the native comparator,
which writes only the comparison report; `outputs/CHK-004.result.json` is
never emitted, so `CHK-007` close fails.

Obligation: the source `runner/__main__.py` compare branch supervises the
native comparator as a subprocess and emits
`outputs/<CHK>.result.json` via `finish()` (`tc-proof-runner-result/v1`)
bound to the executed context (`run_id`, `operation=compare`,
`context_sha256`), with coherent status/exit (`passed` exactly when the
comparator exits 0), a non-empty 64-hex observation-digest list, and the
comparison report present at the nested report path. The branch derives
everything from `--context` with no `TC_PROOF_*` environment preset, and
returns to its caller (no `execv`). The judge executes the exact candidate
source bytes in place and attests the loaded path.

## O-004 — Preserved bindings (all checks, R-004)

Demand: proof-contract § native layout, the TASK-074 accepted bindings,
and `runner/context.py` membership validation. The repair must not regress
identity, trust-manifest, tool, dependency, index, result, template-flow,
or manifest-agreement bindings, the nested comparator identity/report/tool
agreement, or task-derived (non-fixture-default) membership.

Obligation: CHK-001 re-proves the full common binding set on the current
tree (now also requiring the accepted+integrated TASK-074 receipt), and
every profile re-asserts it alongside its new bindings.

## O-005 — Forbidden redirections (CHK-001/CHK-005, R-005, AC-004)

The fix lives in prepare's named bind functions
(`tools/refactor-proof/src/verifier.rs` `bind_compare_qualification`,
`bind_accounting_preparation`, and their direct root/inventory helpers)
plus the `tools/refactor-proof/runner/__main__.py` compare branch only.
Forbidden: any `refactoring-tasks/**` change (in particular no template,
driver, or manifest edits inside TASK-002..007 or any other task package),
any `tools/refactor-proof/{accounting,architecture,bin}/**` change owned
by TASK-071/TASK-072, any other `runner/*.py` change (TASK-074's accepted
glue), any demand-side weakening (`src/compare.rs`, `src/bin/`,
validators), and any hardcoding of digests, manifests, frames, states, or
membership to satisfy the judge. Taskfmt scope + forbidden-path gates
enforce this; the reviewer confirms it.
