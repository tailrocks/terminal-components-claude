# Continue project work

## Status

The design branch defines intended UI/UX. The `main` refactor defines architectural direction but remains incomplete, broken, and not ready.

## Authority

Use [status.md](status.md), [architecture.md](architecture.md), and [design.md](design.md) for current authority. Use [remaining-work.md](remaining-work.md) as the detailed open-task register. Preserve the individual plans, audits, contracts, and acceptance criteria they reference.

## Current evidence

Branch state, manifests, source, current task records, and current check results are authoritative evidence. Historical reports describe the revisions and states they record; re-check before treating them as current.

## Remaining work

Continue every open item in `docs/remaining-work.md`, the improvement and audit records, parity plans, and `refactoring-tasks/`. Keep design work from `design/interaction-polish` in scope. Implement the intended UI/UX within reusable crates following `main`'s architectural direction. Record exact evidence before marking work complete.

## Verification

Use each task's acceptance criteria and the applicable gates. Standard Rust checks are `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets`. Visual checks follow `docs/visual-testing/README.md`. Historical checks apply only to their recorded revision and scope.
