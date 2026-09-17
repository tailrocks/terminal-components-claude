# Refactoring plan pointer

This file is a navigation pointer, not an execution plan. The sole current
readiness authority is [`docs/refactoring-plan/execution-readiness-report.md`](../refactoring-plan/execution-readiness-report.md),
currently **NO-GO**.

Current references:

- Product and architecture intent: [`goal.md`](goal.md), [`../GOAL.md`](../GOAL.md),
  [`../COMPONENT_ARCHITECTURE.md`](../COMPONENT_ARCHITECTURE.md), and
  [`../DESIGN.md`](../DESIGN.md).
- Campaign policy: [`../refactoring-plan/campaign-policy.md`](../refactoring-plan/campaign-policy.md).
- Subagent-only execution: [`../refactoring-plan/subagent-only-policy.md`](../refactoring-plan/subagent-only-policy.md),
  [`../refactoring-plan/campaign-executor-protocol.md`](../refactoring-plan/campaign-executor-protocol.md),
  and [`../refactoring-plan/path-contract.md`](../refactoring-plan/path-contract.md).
- Latest taskfmt contract: [`../refactoring-plan/task-format.md`](../refactoring-plan/task-format.md).

The old host/container executor, lifecycle commands, root-firmlink layout, and
historical taskfmt pins are retired. Do not replay commands from archived
reports. Task work uses isolated host-local subagent worktrees; standalone
taskfmt is limited to per-task `lint` and `verify` validation.

Historical planning evidence remains under [`../refactoring-plan/`](../refactoring-plan/)
and is explicitly non-authoritative unless the current readiness report links
to it.
