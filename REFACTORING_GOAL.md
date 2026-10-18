# Project goal and historical capture contract

## Current goal

Complete the split-crate architecture on `main` while preserving the exact product experience and future requirements defined by `design/interaction-polish`. The architecture is incomplete and not ready until current public APIs, applications, tests, and reviewed visual behavior satisfy their scoped acceptance criteria. The design branch is the UI, color, interaction, and behavior authority; it does not define crate placement. The main branch defines crate placement; it does not prove design parity.

Use [architecture](docs/architecture.md) for current boundaries, [component contracts](COMPONENT_ARCHITECTURE.md) for API and ownership requirements, [design](docs/design.md) for product authority, and [plan](docs/plan.md) for open work. Do not infer completion from historical reports, compilation alone, or a checked-in reference capture.

## §6 Before-refactor visual evidence

`tools/baseline_capture.sh` and the historical capture manifest emit this heading as provenance. The harness writes to `baseline/before/` by default when run; its generated captures describe only the source revision, commands, and terminal state recorded for that run. They are historical comparison inputs, not current product behavior, approval snapshots, or authorization to replace baselines. Use the design reference and current plan for product acceptance. Baseline updates require a separately authorized, reviewed change.
