# Refactoring decision history

## Purpose

This page records the durable outcome of earlier goal changes. Per-commit progress and agent-routing instructions are not current project guidance.

## Decisions that remain relevant

- Preserve the exact design/interaction-polish product behavior and visual output during structural refactoring.
- Use the main workspace direction for reusable library and app package boundaries, while treating its implementation and self-baselines as incomplete product-parity evidence.
- Keep the accepted component, runtime, theme, collection, and app-domain contracts in [shared-components-plan.md](../architecture/shared-components-plan.md) and the sealed task packages.
- Treat earlier oracle revisions, checkpoints, model assignments, and handoff instructions as historical evidence; current authority is defined in [design.md](../design.md), [architecture.md](../architecture.md), and [plan.md](../plan.md).

## Verification

When a historical decision appears to conflict with current requirements, use the explicit authority hierarchy and task inputs above. Do not revive a superseded baseline, schedule, or workflow because it appeared in an earlier goal revision.
