# Documentation

This documentation describes a Rust terminal UI library whose modular rewrite is incomplete and not ready for product use.

## Architecture

- [Architecture](architecture.md) describes crate boundaries and API direction. `main` is the structural reference.
- [Design](design.md) identifies `design/interaction-polish` as the product reference for visuals, colors, UI/UX, behavior, and planned work.
- [Component contracts](components-contracts.md) records current public API locations and requirements still needing consumer or behavior verification.

## Work and development

- [Plan](plan.md) is the live implementation backlog and links the detailed, immutable design-branch requirement inventory.
- [Development](development.md) lists workspace checks and the no-new-code-comments rule.
- [Quick start](guides/quickstart.md) is a current in-workspace facade example; check its APIs against source before depending on them externally.
- [Safety rationales](../SAFETY.md) retains justifications removed from Rust comments.

The design work index preserves original acceptance detail and branch provenance. Its recorded checklist counts are historical inventory, not current completion claims. Other branch audits, reviews, prompts, and execution reports are not project authority.
