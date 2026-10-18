# Termrock

Termrock is a Rust terminal UI component library with deterministic demonstration applications. The workspaces share one product target, but their implementations differ in maturity. The modular refactor is incomplete; no branch should be treated as product-ready from compilation, tests, or documentation alone.

Start with [direction](docs/direction.md), [current state](docs/current-state.md), [architecture](docs/architecture.md), [product](docs/product.md), [requirements](docs/requirements.md), [development](docs/development.md), and [decisions](docs/decisions.md). The repository's `AGENTS.md` remains the instruction file for agent workflow.

| Workspace | What it represents |
|---|---|
| `main` | The intended modular crate direction and incomplete implementation. |
| `design/interaction-polish` | The exact product behavior, visual language, and interaction target, in the former monolithic package. |
| `wip/v1-showcase` | Monolithic snapshot with a scoped Holla activity-timing correction for paused ticks and stopped-job prompts. |
| `wip/s14-followups` | Monolithic snapshot with platform-capability-aware OSC 52 copy status in Holla and Jackin. |
| `wip/plan-s16` | Monolithic snapshot with list/tree metadata and row/column marker API work. |
| `wip/reach-pages-a` | Monolithic snapshot with natural-size measurement and additional measured text/query/connection/finder pages. |
| `wip/rapps-3-7-14` | Monolithic snapshot with metadata folding and hover reveal for dropped app-table columns. |
| `wip/reach-pages-a-2` | Monolithic snapshot with natural-size measurement, scroll-view horizontal-bar retry, and panel/step metadata reveal. |

These documents are intentionally branch-identical. They describe the shared target and known constraints; inspect the checked-out source, manifest, and tests for branch-specific implementation facts. `AGENTS.md` and `CLAUDE.md` are preserved instruction files and can still contain legacy links; this README is the navigation entrypoint for the replacement docs.
