# Workspace structure and migration evidence

## Current authority

`main` supplies the split-crate structure and reusable API direction; its refactor remains incomplete, broken, and not ready. `design/interaction-polish` supplies the exact product UI/UX and behavior target. Current implementation claims must be checked against current source and task-specific gates.

## Current structural evidence

The `main` root manifest declares 46 workspace members grouped as reusable `termrock-*` crates, Showcase/Jackin/Holla/TablePro app-family crates, and test/proof/tooling crates. The repository contains 50 Cargo manifests including the root and three package manifests outside workspace membership. Use the current root and member manifests as the exact inventory; older workspace proposals and branch snapshots are historical.

The WIP worktrees remain a single `junie-tui` package with four binaries under `src/bin/`. Their application and shared modules have not yet moved into the split packages. Manifest presence proves a package boundary exists in the tree; it does not establish that behavior has been migrated or verified.

## Migration evidence to consult

- [Architecture and migration requirements](../../architecture/component-architecture.md) describes current WIP boundaries, the split direction, required API and state contracts, and migration order.
- [Shared component plan](../../architecture/shared-components-plan.md) and the task packages own detailed API and crate-specific acceptance.
- [Remaining work](../../remaining-work.md) owns branch-specific task IDs, dependencies, and recorded scoped closures.
- [Visual testing](../README.md) defines the current scenario and approval process against `design/interaction-polish`.

Historical branch comparisons and defect ledgers remain useful for their cited clauses only. They do not provide current source status, authorize cherry-picks, or establish behavioral parity.
