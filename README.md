# Junie TUI

Junie TUI is a deterministic Rust terminal UI component and application laboratory. It contains a widget showcase and simulated previews for TablePro, Jackin, and Holla; the preview applications do not perform real external side effects.

## Run

```sh
cargo run --release
cargo run --release --bin tablepro
cargo run --release --bin jackin-preview
cargo run --release --bin holla
```

The root package is `junie-tui`; its library is `junie_tui`. The product source is under `src/`, with the four binaries under `src/bin/`. `xtask` provides repository tooling. This is the `wip/plan-s16` worktree, not the target architecture. `main` gives the correct split-crate direction, but its refactor is incomplete and not ready. `design/interaction-polish` is the exact UI/UX, color, interaction, and behavior target; this WIP retains branch-specific code, specifications, and backlog, and does not imply every target feature is present.

## Project documentation

- [Documentation map](docs/README.md): active references and evidence scope.
- [Architecture direction](docs/architecture.md): current crate shape and migration target.
- [Design contract](docs/design.md): UI/UX authority and focused specifications.
- [Development and verification](docs/development.md): code, visual, and generated-doc workflows.
- [Roadmap](docs/plan.md): detailed open-work index; branch-specific acceptance ledger in [remaining work](docs/remaining-work.md).
- [Visual testing](docs/visual-testing/README.md): scenarios, approval rules, and current commands.
- [Safety rationale](SAFETY.md): explanations for unsafe blocks and invariants.

Run the applicable workspace checks and visual gate before claiming a code or design task complete. Historical reports prove only their recorded revision and scope.
