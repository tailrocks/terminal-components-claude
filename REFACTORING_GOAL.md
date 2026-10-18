# Modular architecture goal

## Status

Active target. The migration has started but does not yet meet the intended behavior and is not ready.

## Authority

`main` defines the required split-crate structure and reusable API direction. `design/interaction-polish` defines the target appearance, colors, UI/UX, behavior, and planned product work. The design branch's monolithic implementation is not the target structure; its API ideas may be adapted to focused crates.

## Current evidence

The workspace manifest declares many reusable and application-specific crates. `cargo check --workspace --offline` passed after the Rust comment-removal pass. That proves the current workspace compiles; it does not prove application behavior, visual fidelity, API quality, or refactor readiness.

## Remaining work

- Complete and repair the current modular implementation.
- Preserve the design branch's complete planned-work list and map each item to its responsible crate or application.
- Reconcile candidate API designs from the reference branch with focused, reusable public APIs.
- Keep domain behavior in applications and generic interaction/presentation behavior in reusable crates.
- Remove obsolete migration layers only through an intentional compatibility-safe migration.
- Prove required user-visible behavior and design parity with current captures and interaction evidence.

## Verification

Use current-source crate boundary checks, independent consumer builds, focused behavior tests, and reviewed captures against the design branch. Report each result with its exact revision and scope.
