# Refactoring task catalog

## Status

Historical task-format catalog. Its packages are pinned contract artifacts pending reconciliation with current scope and authority; do not dispatch them as current active plans.

## Authority

`main` supplies the split-crate architecture and reusable API direction. `design/interaction-polish` supplies the current UI/UX, colors, and behavior. Historical package inputs and oracle pins do not override that design authority. The current readiness record is [docs/status.md](../docs/status.md), and open work is tracked in [docs/remaining-work.md](../docs/remaining-work.md).

## Current evidence

The catalog contains 73 task packages and associated trusted source-obligation data. Package formats and their requirement/acceptance/check identifiers are preserved byte-for-byte; their current reconciliation state is pending. The task index and traceability tables are historical inventories, not proof of readiness.

## Remaining work

Retain every package requirement, acceptance criterion, check, task dependency, and trusted source obligation while reconciling the catalog against the current split-crate architecture and design branch. Record any supersession explicitly; do not drop planned work.

## Verification

The package schema is `task/v5`, `verify/v2`, and `task-meta/v1` at task-format revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`. Each task README follows the schema’s fixed sections. The [task artifact disposition inventory](../docs/refactoring-task-disposition.tsv) classifies and hashes every Markdown artifact in this tree. Partition review ledgers also record per-task README SHA-256 and semantic coverage. Changing pinned task files requires a newly reviewed contract set and updated inventories. Schema lint alone does not establish current authority or implementation readiness.

Read the [historical proof contract](../docs/refactoring-plan/proof-contract.md), [task index](../docs/refactoring-plan/task-index.tsv), and [bidirectional traceability](../docs/refactoring-plan/traceability.tsv) as source records only. Do not use the old dispatcher or promotion lifecycle as current workflow.
