# Retired production container-path design

> Historical record only. This file is non-executable and is not a campaign
> runbook. Never create containers, mounts, firmlinks, fixed root namespaces,
> or host lifecycle services for this refactoring.

## Status

The former production verification design coupled task packages to fixed
container namespaces and a host-owned lifecycle. That design is rejected. The
current contract is [campaign-policy.md](campaign-policy.md): an
implementer, verifier, and reviewer subagent use isolated host-local paths;
standalone latest taskfmt is limited to one task's lint and verify checks.

The current path rules are in [path-contract.md](path-contract.md). The
readiness report remains the authority for whether any task may dispatch.

## Preserved finding

Production task packages once assumed that the checker executable and frozen
context files would appear through external runtime namespaces. Passing a
worktree root or task directory to taskfmt could not rewrite those embedded
arguments. Verification therefore depended on runtime setup that the current
policy forbids.

That dependency was the reason for the old readiness blocker. It is retained
here as provenance only; it is not a supported workaround and cannot authorize
task dispatch.

## Replacement contract

For each task, the verifier subagent receives:

- TASK_DIR: one catalog package;
- WORKTREE: one isolated candidate worktree;
- RUN_DIR: one external evidence directory;
- TASKFMT: the exact latest standalone taskfmt binary;
- SCOPE_BASE: the recorded scope-base commit.

The verifier runs only the per-task taskfmt lint and taskfmt verify commands
defined by the current policy. The coordinator reviews raw evidence and
reviewer findings before serial integration. No historical lifecycle operation
from this document is part of that flow.

## Cross-reference policy

Existing planning documents may link here because the filename records the old
design. Readers must follow the current policy links above. Do not copy a
command, path, or lifecycle sequence from this historical record into a task
package, script, or operator prompt.
