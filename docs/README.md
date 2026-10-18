# Documentation index

## Status

These documents describe a project with two distinct references: `main` for the intended split-crate architecture and `design/interaction-polish` for intended UI/UX and behavior. The refactor is unfinished, broken, and not ready.

## Authority

- [Architecture](architecture.md): crate structure and reusable API direction.
- [Design](design.md): visual, interaction, color, and behavior authority.
- [Status](status.md): evidence-based current project status.
- [Plan](plan.md): remaining work and verification expectations.
- [Remaining work](remaining-work.md): detailed active design and implementation tasks.
- [Goal](GOAL.md): continuation and handoff procedure, subject to the authority above.

[Safety rationales](../SAFETY.md) preserve the justifications for unchanged unsafe operations outside Rust comments.

## Current evidence

The index catalogs active references and supporting records. `docs/sources/` contains historical source documents. `docs/refactoring/`, `docs/refactoring-plan/`, and `refactoring-tasks/` contain analyses, plans, contracts, and verification records whose claims are scoped to their recorded revisions and dates. `docs/audits/`, `docs/improvements/`, `docs/parity/`, and `docs/visual-testing/` contain design findings, planned improvements, parity evidence, and visual validation procedures. Consult each record for its scope; historical evidence does not establish current completion.

## Remaining work

Keep concrete planned work, open findings, acceptance criteria, and contracts available. Correct contradictions in active guidance and link dated material as historical. Captured terminal `.txt`, `.html`, `.ansi`, image, and other fixture artifacts are data, not prose documentation, and must not be rewritten as prose.

## Verification

For claims about current code, inspect the current source and manifests. For completion, inspect the current task evidence and run its applicable checks. The canonical project checks are `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo nextest run --all-targets`; visual verification procedures live in [visual testing](visual-testing/README.md).
