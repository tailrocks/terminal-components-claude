# Project operating goal

## Product and architecture direction

This `wip/v1-showcase` worktree contains branch-specific interaction and visual implementation evidence. `design/interaction-polish` is the exact product target; this WIP does not imply that every target feature is present. `main` supplies the correct split-crate architecture direction, but that refactor remains incomplete and not ready.

## Work to preserve

Keep open requirements in the detailed design contract, audit findings, improvement specifications, parity records, shared-component plan, visual-testing plan, and refactoring task packages. Preserve branch-specific task IDs, dependencies, and recorded scoped closures. Retain useful APIs where they fit the split-crate boundaries, and port behavior against the design authority rather than treating the current main implementation as complete.

Start with [README](README.md) and the [documentation map](docs/README.md). Use the [architecture guide](docs/architecture.md), [design contract](docs/design.md), [development guide](docs/development.md), and [roadmap](docs/plan.md) as active references. The detailed worktree ledger is [docs/remaining-work.md](docs/remaining-work.md).

## Completion evidence

Verify structure from current manifests and imports, behavior from current source and task gates, and visual claims against `design/interaction-polish` using [the visual testing guide](docs/visual-testing/README.md). Standard code gates are `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets`. A passing check proves only the revision and scope it exercised.
