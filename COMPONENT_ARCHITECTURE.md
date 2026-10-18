# Architecture decisions

## Status

Current structural direction. Implementation is incomplete and not ready.

## Authority

[docs/architecture.md](docs/architecture.md) defines crate structure and API goals. [docs/design.md](docs/design.md) defines the product reference. The branch `main` is the structural model; `design/interaction-polish` is the visual and behavior model.

## Current evidence

`Cargo.toml` declares focused reusable library crates, app-specific domain and presentation crates, and application entry crates. This establishes that the split exists in the workspace; it does not establish that boundaries or public APIs work correctly.

## Remaining work

- Keep reusable components and interaction primitives in reusable crates; keep product/domain decisions in app crates.
- Make public APIs composable and usable across crates without exposing implementation-only internals.
- Adapt better API ideas from `design/interaction-polish` only when they fit the split architecture.
- Preserve stable identity, caller-owned state, deterministic input/output, and testable rendering where current source and design evidence support those contracts.
- Do not adopt old Slice 1–7 sequencing or historical acceptance claims from retired plans.

## Verification

Inspect manifests and dependency edges for structure. Compile independent consumers and test APIs across crate boundaries. Compare behavior and captures with `design/interaction-polish`. Compilation alone does not establish readiness.
