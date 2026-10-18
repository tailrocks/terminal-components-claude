# Junie TUI

## Status

This branch is the reference for the intended terminal interface: visual language, colors, layouts, interactions, and behavior. Its monolithic crate structure is not the target architecture.

## Authority

Use `main` as the architectural direction: separate crates, reusable components, and public APIs. The `main` refactor is incomplete, broken, and not ready to serve as a working implementation. Use this branch for intended UI and behavior. Reuse API ideas where useful, then place them in the split-crate architecture.

## Current evidence

The repository contains one Rust package with four application binaries. The root package manifest and `src/` tree are the evidence for current structure. The design and interaction specifications define intent; they do not prove every item is implemented or verified.

## Remaining work

Preserve all open design, product, audit, and refactoring tasks in `docs/remaining-work.md`, `docs/improvements/`, `docs/audits/`, and `refactoring-tasks/`. Complete the planned UI/UX work against this branch's design authority while carrying reusable APIs into the split-crate architecture. Treat the refactor as unfinished until implementation and its required verification are complete.

## Verification

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets` for code changes. For visual claims, use the current procedures in `docs/visual-testing/README.md`; a plan or historical verification record is not current proof. Rust comment-only removal must leave behavior and rendered output unchanged.

## Applications

- `showcase` demonstrates library widgets and compositions.
- `tablepro` is a deterministic database-workbench preview.
- `jackin-preview` is a deterministic simulated application redesign.
- `holla` is a deterministic context-adaptive action-launcher preview.

Run the showcase with `cargo run --release`; select another binary with `cargo run --release --bin tablepro`, `cargo run --release --bin jackin-preview`, or `cargo run --release --bin holla`. Each binary's README documents its scenarios and controls.
