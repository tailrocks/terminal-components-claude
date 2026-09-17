# Archival synthetic fixture instructions

> This directory is historical evidence, not a live refactoring task. Do not
> dispatch it, edit protected fixture inputs, start containers, create mounts,
> or invoke host/taskfmt lifecycle helpers.

## Authority

The fixture README defines the synthetic payload and protected sentinel. The
driver, observer, vectors, and hashes are planner-owned evidence. A candidate
must never change those inputs or claim that this fixture proves a production
task.

## Preservation rules

- Keep the protected sentinel and checker bytes unchanged.
- Keep the fixture package, schemas, and expected outcomes internally
  consistent.
- Treat any historical result, receipt, progress file, or report as
  non-authoritative.
- Do not place fixture output in the campaign ledger or a task run directory.

## Current campaign boundary

Current refactoring work uses isolated implementer, verifier, and reviewer
subagents with host-local paths. Latest standalone taskfmt is limited to the
per-task lint and verify checks defined by the current campaign policy. This
archival fixture is not one of those task packages and is not a substitute for
fresh verifier-subagent evidence.

If the fixture is inspected for historical review, preserve raw bytes and
record observations outside the repository's campaign authority. No edit here
can authorize campaign dispatch or integration.
