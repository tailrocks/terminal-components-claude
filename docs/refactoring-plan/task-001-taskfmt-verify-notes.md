# Retired TASK-001 taskfmt qualification notes

> Historical snapshot. This file records a failed preparation attempt on the
> retired planning worktree. It is non-executable and does not authorize
> TASK-001, a receipt, or campaign dispatch.

## Current rule

The current workflow is [campaign-policy.md](campaign-policy.md):
subagents own implementation, verification, and review; latest standalone
taskfmt is used only for one task's lint and verify validation. Follow
[path-contract.md](path-contract.md) for host-local paths.

## Recorded attempt

The 2026-09-15 preparation attempt used an older taskfmt revision and a
retired path-bound verification layout. The process itself returned exit zero,
but its terminal result was RESULT FAIL, not DONE. The attempt therefore did
not satisfy the historical completion gate.

The failure was environmental and structural: checker arguments referenced
fixed runtime namespaces, and privileged path setup was unavailable. Worktree,
task-directory, and environment overrides did not rewrite those checker
arguments. An advisory run of selected checks passed after path substitution,
but that was not a valid standalone taskfmt verification result.

## Retained evidence

- The old taskfmt executable, source revision, and hashes are historical
  observations only; they are not the current taskfmt identity.
- The observed pass=5 fail=5 ceiling is evidence of the retired layout's
  limitation, not a current task result.
- The old progress stream reached its terminal marker, but that marker cannot
  substitute for current verifier-subagent evidence.
- The old root-path simulation and its sandbox helper are retired. Do not
  recreate their filesystem setup or invoke their lifecycle operations.

## Decision

This record remains because several planning reports cite its blocker and
verdict. Those links are safe provenance links only. A current task may be
considered only after its verification configuration is host-local and a fresh
verifier subagent records successful latest-taskfmt lint/verify evidence.
