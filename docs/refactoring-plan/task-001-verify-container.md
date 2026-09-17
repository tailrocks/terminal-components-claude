# Retired TASK-001 path simulation

> Historical record only. The former root-path and container simulation is
> retired. Do not run it, recreate its mounts, or use it as verification
> authority.

## Current authority

Use [campaign-policy.md](campaign-policy.md),
[path-contract.md](path-contract.md), and the current
[execution-readiness-report.md](execution-readiness-report.md). They require
host-local subagent worktrees and limit standalone latest taskfmt to per-task
lint and verify validation.

## What this record preserves

An earlier TASK-001 qualification attempt showed that its checker arguments
were bound to fixed runtime namespaces. Worktree and task-directory flags did
not translate those arguments. The attempted root-path simulation also needed
privileged filesystem setup, so the observed result was not an accepted
completion gate.

The useful conclusion is negative: runtime path fabrication is not a valid
verification strategy. The campaign now treats any legacy fixed namespace in a
task verification configuration as a migration blocker and fails closed.

The old qualification notes remain linked for provenance:
[task-001-taskfmt-verify-notes.md](task-001-taskfmt-verify-notes.md).
They are historical evidence, not current instructions.

## Frozen invariants

- The visual-baseline tag is immutable.
- No historical simulation result authorizes TASK-001 or any later task.
- Current evidence must come from a fresh verifier subagent run with explicit
  host-local paths and current taskfmt identity.
- The coordinator cannot upgrade historical evidence into completion authority.
