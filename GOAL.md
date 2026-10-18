# Project goal

## Status

The Rust refactor on `main` has started but is incomplete, broken, and not ready. `design/interaction-polish` is the authority for intended UI/UX, colors, interactions, and behavior.

## Authority

Use `main` as the direction for split crates and reusable APIs. Use this branch for the design target and all its planned work. Carry the intended behavior into the split-crate architecture; retain useful API designs where they fit. Do not treat historical planning prompts as current instructions.

## Current evidence

Inspect current branch source, manifests, task records, and verification results before asserting implementation status. The documentation index is [docs/README.md](docs/README.md); canonical status, architecture, design, and plan live in `docs/status.md`, `docs/architecture.md`, `docs/design.md`, and `docs/plan.md`.

## Remaining work

Preserve and complete all open UI/UX, product, audit, parity, improvement, and refactoring work in `docs/remaining-work.md`, `docs/audits/`, `docs/improvements/`, `docs/parity/`, `docs/architecture/shared-components-plan.md`, and `refactoring-tasks/`. Remove Rust source comments without changing code behavior or project visuals. Reconcile outdated prose with current evidence and the branch authorities above.

## Verification

For code changes, run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets`. For visual claims, follow `docs/visual-testing/README.md`. Comment removal must leave behavior and rendered output unchanged. Historical checks prove only their recorded revision and scope.
