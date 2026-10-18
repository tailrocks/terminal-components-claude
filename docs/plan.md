# Project plan

## Status

Architecture refactor remains unfinished and not ready. Planned design work on `design/interaction-polish` remains part of the target and is not removed by this documentation cleanup.

## Authority

Use `main` as the target for crate separation and reusable APIs. Use `design/interaction-polish` as the target for UI/UX, visual design, colors, and behavior. Integrate design behavior into the split crates; reuse useful API ideas when they fit. Do not treat the current `main` refactor as complete or working.

## Current evidence

`docs/remaining-work.md` is the branch's detailed active work list. `docs/architecture/shared-components-plan.md`, `docs/improvements/`, `docs/audits/`, and `refactoring-tasks/` contain plans, contracts, and recorded evidence at different dates. Their task-level status and acceptance criteria remain meaningful; claims of completion require current evidence. Historical snapshots under `docs/sources/` are context, not active instructions.

## Remaining work

1. Preserve and finish all open design, audit, parity, improvement, and refactoring work.
2. Decompose the implementation into reusable crates following the architectural direction of `main`.
3. Port the intended UI/UX and behavior from `design/interaction-polish` into those crates.
4. Reconcile old documentation against the current code and task evidence. Retain historical records with clear scope; remove false current claims and obsolete instructions.
5. Verify each completion against its own acceptance criteria and applicable code and visual gates.

## Verification

For each task, use its recorded acceptance criteria and current source evidence. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets` for code milestones. Use the current visual procedures in `docs/visual-testing/README.md` for UI milestones. Record exact revision and result; do not infer completion from branch names or old reports.
