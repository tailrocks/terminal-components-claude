# Architecture direction and current shape

## Target boundaries

`main` supplies the correct structural direction: split crates with reusable component APIs. Its refactor is incomplete and not ready. `design/interaction-polish` supplies the product behavior and visual target. Keep useful API proposals from this WIP where they fit the target crate boundaries; its monolithic source layout is not an architecture requirement.

## Current package

`Cargo.toml` defines one `junie-tui` package and the `junie_tui` library at `src/lib.rs`, with four binaries under `src/bin/` and `xtask` as a workspace member. Widgets, shared state, shell, and runtime code remain under `src/`. This describes the `wip/reach-pages-a-2` checkout, not the readiness of the `main` migration.

## Migration requirements

Move reusable state, input, rendering, focus, accessibility, terminal lifecycle, and safety contracts into the appropriate shared crates. Keep app routes and product composition in app-facing layers. Preserve typed public APIs where possible, and make any compatibility change explicit in the owning task contract. Port behavior against `design/interaction-polish`; do not infer target behavior from the current main implementation.

The detailed candidate API inventory and migration steps are in [shared components plan](architecture/shared-components-plan.md). Branch-specific acceptance items remain in [remaining work](remaining-work.md), task packages, and [the source-first refactoring plan](refactoring/plan.md).

## Architecture verification

For each migrated boundary, inspect current manifests, imports, ownership, and public APIs. Verify behavior at both component and app integration layers, then run the task's required gates. Passing compilation alone does not prove a crate boundary is complete or that the UI matches its design authority.
