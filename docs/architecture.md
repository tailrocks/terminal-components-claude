# Architecture

## Status

Target direction established; the `main` refactor is incomplete, broken, and not ready. This branch remains a single Rust package and is not the desired final structure.

## Authority

`main` is the architectural reference: split crates with reusable component APIs. `design/interaction-polish` is the UI/UX reference: intended visuals, colors, layouts, and behavior. Its monolithic structure is not authoritative. Some API designs from this branch may be retained when they fit the crate boundaries.

## Current evidence

`Cargo.toml` declares the root package and workspace member `xtask`; library code and four application binaries live under `src/`. This is direct evidence of the present structure. Branch history and plans are not evidence that the refactor works. See `docs/refactoring/main-port-analysis.md` and `docs/refactoring/state.md` for recorded branch analysis, and re-check source before relying on their dated findings.

## Remaining work

Continue the planned decomposition recorded in `docs/architecture/shared-components-plan.md`, `docs/remaining-work.md`, and `refactoring-tasks/`. Preserve reusable APIs and contracts while moving ownership into crates. Port the intended interaction and visual behavior from this branch. Keep every uncompleted task open; do not describe planned, partially ported, or historically verified work as complete.

## Verification

Verify crate boundaries from current manifests and imports. Verify each migrated capability with its required tests and gates. Use `cargo nextest run --all-targets`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`; use the procedures in `docs/visual-testing/README.md` for UI parity. Passing checks for an earlier revision do not establish current completion.
