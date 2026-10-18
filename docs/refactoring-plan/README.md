# Refactoring planning records

This directory contains the task graph, obligation and traceability ledgers, proof contracts, source-difference analyses, and historical planning evidence for the component refactor. The [active plan](../plan.md) and the sealed packages under [`../../refactoring-tasks/`](../../refactoring-tasks/) define current task navigation and acceptance.

## Use

- Use `task-index.tsv`, `task-graph.*`, and `traceability*.tsv` to navigate task identity, dependencies, and requirement ownership.
- Use `proof-contract.md` and task package `verify.toml` files for planned verification contracts.
- Treat `PROGRESS.md`, `planning-verification.md`, `planning-acceptance.md`, review reports, and dated re-audit snapshots as evidence of work recorded at their cited source revision. Their pass, failure, and readiness statements are not current measurements.
- Preserve structured task inputs and evidence artifacts byte-for-byte when task contracts or integrity records bind to them. This index is explanatory and does not replace those inputs.

The product behavior authority is the design/interaction-polish contract and its approved captures. Structural direction is described in [architecture](../architecture.md). No report in this directory can independently close an implementation or parity requirement.
