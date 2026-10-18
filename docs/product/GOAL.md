# Holla product

## Status

Holla exists as a deterministic preview application. This page records current product authority and links to retained design and planning work; it is not an instruction to create a second Holla binary. The repository-wide architecture refactor remains incomplete, broken, and not ready on `main`.

## Authority

Use [`CONCEPT.md`](CONCEPT.md) and [`references/`](references/) for Holla product requirements. Use [`../design/DESIGN.md`](../design/DESIGN.md) and the design-branch specifications for the UI/UX and interaction contract. Use [`../improvements/PLAN.md`](../improvements/PLAN.md) for the retained Holla improvement plan. Use `main` only for split-crate and reusable-API direction.

## Current evidence

The implementation lives in `src/bin/holla/`; the root package declares the `holla` binary. Its fixtures are deterministic and do not execute external commands. Current behavior and completion status require source, test, and visual evidence for the current revision.

## Remaining work

Keep all open Holla design, interaction, showcase coverage, parity, and shared-component tasks in `docs/improvements/PLAN.md`, `docs/remaining-work.md`, the audit records, and their linked task files. Keep production providers and deferred CLI work in their recorded Later scope unless a current project decision changes it. Implement shared behavior in reusable crates when the architecture work reaches it.

## Verification

Use the task-specific acceptance criteria, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets`. Verify rendered behavior through the current procedures in `docs/visual-testing/README.md`. Historical counts and passes apply only to their recorded revision and scope.
