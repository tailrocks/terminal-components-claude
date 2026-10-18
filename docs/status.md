# Project status

## Status

The intended UI/UX is represented by `design/interaction-polish`. The reusable, split-crate architecture is represented by the direction of `main`. That refactor is unfinished, broken, and not ready. Neither a plan nor a historical completion label changes this status without current implementation evidence.

## Authority

Architecture: `main` direction. Visuals, colors, interactions, behavior, and planned design work: `design/interaction-polish`. For the authoritative open-work list, see `docs/remaining-work.md` and the linked task records. For code state, inspect the current branch and manifests.

## Current evidence

At this revision, the root `Cargo.toml` defines package `junie-tui` and the `xtask` workspace member; application and library source remain under `src/`. This establishes that this checkout has not adopted a split-crate layout. It does not establish whether every planned UI behavior is implemented. Historical branch comparisons in `docs/refactoring/` are dated evidence and must be revalidated before use as current status.

## Remaining work

Complete the planned crate decomposition without losing this branch's UI/UX. Carry forward useful reusable API designs. Retain the existing open work in `docs/remaining-work.md`, audits, improvement plans, parity plans, and `refactoring-tasks/`; update completion state only when its stated evidence exists.

## Verification

Check current manifests and source for architecture claims. Check current test and visual-gate results for behavior claims. Required Rust checks: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets`. A successful check proves only the revision and coverage it actually ran.
