# Active plan and acceptance

## Goal

Complete the structural refactor while reproducing the design/interaction-polish branch's exact product behavior and visual output. The current checkout supplies the product reference and detailed requirements. The separate main workspace is the structural starting point for an eventual integration; its existing implementation does not establish completion or parity.

## Work inventory

- [Remaining work](remaining-work.md) tracks current open product, component, parity, evidence, and verification obligations.
- [Shared component plan](architecture/shared-components-plan.md) contains detailed state, API, rendering, input, identity, theme, collection, and package contracts.
- [Refactoring completion plan](sources/REFACTORING_COMPLETION_PLAN.md) and [planning goal](sources/PLANNING_GOAL.md) define the execution boundary, accepted oracle, and completion gate.
- [`refactoring-plan/`](refactoring-plan/) contains the task graph, task index, traceability, branch-difference analysis, and proof requirements.
- [`refactoring-tasks/`](../refactoring-tasks/) contains sealed, independently verifiable task packages. Their task files and structured inputs are requirements, not historical reports.
- [`improvements/PLAN.md`](improvements/PLAN.md) and the per-feature files retain product improvement scope and acceptance criteria.

## Completion gates

1. Every retained requirement maps to an owning task and an executable acceptance check.
2. The task graph and source inventories are complete and independently reviewed before execution begins.
3. Structural boundaries, public APIs, state ownership, and safety contracts pass their dedicated checks.
4. The four applications preserve the specified product flows and states.
5. Visual and interaction output matches the immutable approved reference across required scenarios, dimensions, and color modes.
6. Required build, test, lint, documentation, architecture, and evidence-integrity checks pass at the final integration revision.

The refactor is not complete merely because a plan, task package, or earlier verification report says it passed. Current readiness and execution authorization are determined by the live task inputs and their verified evidence.
