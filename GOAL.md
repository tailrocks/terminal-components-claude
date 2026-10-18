# Active project goal

## Status

Active. Documentation and code-comment cleanup underway; refactor remains incomplete and not ready.

## Authority

Use [docs/README.md](docs/README.md), [docs/architecture.md](docs/architecture.md), [docs/design.md](docs/design.md), [docs/status.md](docs/status.md), and [docs/plan.md](docs/plan.md). `main` defines structure; `design/interaction-polish` defines product behavior and appearance.

## Current evidence

The workspace has a split crate structure. The current offline workspace check passed after comment removal. No such result proves full behavior or parity.

## Remaining work

Compare all branches named in [the plan](docs/plan.md), remove comments from every Rust crate without changing behavior, preserve all planned design-branch work, and rewrite documentation to the shared structure and current authority. Keep unresolved requirements open.

## Verification

Audit all tracked prose docs and Rust crate comments. Validate documentation links and use behavior-relevant evidence to confirm comment removal caused no runtime, visual, or functional change.
