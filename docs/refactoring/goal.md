# Refactoring objective

## Objective

Complete the split-crate refactor while preserving the exact intended experience in `design/interaction-polish`: appearance, colors, layout, interaction, and behavior across Showcase, Holla, Jackin, and TablePro. Keep all planned work in the detailed design specifications, audits, improvements, parity records, task contracts, and branch-specific remaining-work ledger.

## Authority

- `main` defines the split-crate structure and reusable API direction. Its refactor remains incomplete, broken, and not ready.
- `design/interaction-polish` defines the product UI/UX, color, layout, interaction, and behavior target. Its monolithic structure is not the architecture target.
- Current source and task-specific verification determine implementation status for the exact revision and scope tested. Historical reports, successful compilation, and branch snapshots do not establish parity or readiness.

## Required outcomes

- Reusable crates own shared state, input, rendering, focus, accessibility, terminal lifecycle, safety, layout, text, theme, collection, and component contracts. App-family crates own product models, simulations, workflows, and screens.
- Every application uses the intended reusable public APIs. Do not finish with parallel legacy APIs, duplicated shared behavior, or application-local patches that bypass component ownership.
- Preserve visible and interactive behavior against the design authority. Do not redesign to make a port easier or to match current implementation differences.
- Keep theme roles semantic and capability-aware, including a distinct paper theme alongside Junie; preserve the color, state, focus, keyboard, pointer, resize, overlay, and terminal-size contracts in the design and component specifications.
- Treat expected visual evidence as independently controlled input. Candidate output cannot bless or redefine the expected result. Record exact revisions, scenarios, dimensions, and verification scope for each closure.

## Work and closure

Use [the roadmap](../plan.md), [the branch-specific remaining-work ledger](../remaining-work.md), and the linked task packages as owners of IDs, dependencies, acceptance criteria, and recorded scoped closures. A task is complete only when its current implementation and required tests or visual evidence pass at the stated scope. An individual green build, old checkpoint, or neighboring task closure does not close it.

## Current checkout

This document is the active objective for the WIP worktrees. The root package and embedded applications remain monolithic here; see [the architecture and migration requirements](../architecture/component-architecture.md) for the observed source shape, split-workspace direction, package boundaries, and migration sequence. See [the design contract](../design.md) for product authority.
