# Task status and acceptance

## Status

The task catalog is a set of future execution contracts. Previous audit tables and pass counts are historical and do not establish present readiness.

## Authority

Each package's `task.toml`, `verify.toml`, structured inputs, and README define that task's scope and acceptance. The current task graph and cross-requirement joins are indexed in [`refactoring-plan/task-index.tsv`](../refactoring-plan/task-index.tsv), [`task-graph.md`](../refactoring-plan/task-graph.md), and [`traceability.tsv`](../refactoring-plan/traceability.tsv). Do not infer task status from old narrative audits.

## Current evidence

The catalog is under [`refactoring-tasks/terminal-components/completion/`](../../refactoring-tasks/terminal-components/completion/). Its task files are operational inputs and remain intact. A task is complete only when its exact acceptance checks pass on the intended source revision and the required independent review and integration evidence exist.

## Remaining work

Resolve the open requirements in [remaining-work.md](../remaining-work.md), preserve all task dependencies, and update status only from current evidence. Missing source, skipped tests, self-generated baselines, and historical receipts cannot satisfy acceptance.

## Verification

Follow the task-format and protected-evidence rules in [`refactoring-plan/proof-contract.md`](../refactoring-plan/proof-contract.md). The repository-wide completion gates are in [plan.md](../plan.md).
