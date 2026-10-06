# Termrock

This repository holds the Termrock Rust terminal UI library and its four preview applications.
The refactor proceeds on `termrock-implementation` from the frozen `visual-baseline` commit.
The workspace is a multi-crate Cargo layout under `crates/`, with the Termrock library crates
and the `showcase`, `tablepro`, `jackin-preview`, and `holla` preview applications.
Crate counts alone never prove reusable component adoption; the composition contract below owns that rule.

Some current source and package names still predate the Termrock identity.
They remain implementation names until a later implementation change.
This documentation update does not rename the repository, Cargo packages, crates, source paths, or applications.

## Preview applications

`showcase`, `tablepro`, `jackin-preview`, and `holla` are preview UI applications.
They demonstrate the library with deterministic fixtures, scenarios, keyboard and pointer behavior.
Their existing output and observable interactions are protected.
Future Termrock implementations must preserve them one-for-one; product changes require a separate, explicitly approved change.
The previews show how a real interface uses the library; they are not separate product redevelopment projects.

The frozen source and expected output are pinned to commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b), the commit resolved by the annotated `visual-baseline` tag. Candidate code cannot create, update, or approve its own expected baseline.

## Run the current applications

Run these commands from the repository root. They describe the current baseline implementation:

```sh
cargo run --release
cargo run --release --bin tablepro
cargo run --release --bin tablepro -- --connect Production
cargo run --release --bin jackin-preview
cargo run --release --bin jackin-preview -- --scenario accounts-mixed
cargo run --release --bin holla
cargo run --release --bin holla -- --scenario docker-cleanup
```

Showcase is the default binary. Its pages, and the preview scenarios for the other applications, provide the frozen reference scenes. Approved artifacts live under `snapshots/`; the source capture harness lives under `tests/visual_baseline/`. The current complete visual-baseline command is:

```sh
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

This is a verification command for implementation work. Expected artifacts stay protected; do not run a snapshot acceptance or regeneration command against them.

## Documentation map

- [GOAL.md](GOAL.md) — current mission, boundaries, and success conditions.
- [AGENTS.md](AGENTS.md) — agent entry rules, ownership, review, and the path allowlist.
- [docs/README.md](docs/README.md) — canonical documentation map and authority rules.
- [Architecture](docs/architecture/overview.md) — target ownership and runtime shape.
- [Component composition](docs/architecture/component-composition.md) — the component-only interface contract.
- [Public API](docs/api/public-api.md) and [component contracts](docs/components/README.md) — target Rust API and all documented surfaces.
- [Consumer recipes](docs/api/consumer-recipes.md) — ordinary consumer examples with verified status labels.
- [Per-commit review](docs/process/per-commit-review.md) — independent work-item and every-commit review.
- [Implementation plan](docs/implementation/plan.md) — future in-place phases.
- [Visual verification](docs/verification/visual-parity.md) — oracle, comparison, and parity gates.
- [Ownership and parity](docs/verification/ownership-and-parity.md) — ownership proof with appearance and behavior.
- [Preserved applications](docs/applications/README.md) — commands and conformance role for each preview consumer.
- [Task catalog](refactoring-tasks/README.md) — executable future work packages and dependencies.

The long-term product is a small reusable Rust TUI library with caller-owned state, borrowed props, explicit update/draw/measure phases, typed actions, stable identities, shared runtime interaction, and semantic styling. Component consolidation reduces duplicated mechanisms while keeping each frozen visual recipe and interaction distinct.
