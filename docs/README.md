# Documentation

The modular refactor remains incomplete; this index does not establish product readiness. `main` is the structural reference, while `design/interaction-polish` is the product reference for visuals, colors, interaction, behavior and planned work.

[Architecture](architecture.md) describes ownership and API direction; [product](product.md) identifies appearance and interaction authority. Product parity and architecture completion require separate evidence. Review design-branch API ideas individually before adopting them through the shared crate boundaries. Preserve design-branch work and acceptance criteria. Reusable behavior belongs in shared crates; application workflows and domain policy belong in application crates.

[Component contracts](components-contracts.md) maps public owners, named evidence and remaining acceptance. Its [authoring](contracts/authoring.md), [migration](contracts/migration.md), [overrides](contracts/overrides.md) and [theming](contracts/theming.md) pages distinguish available mechanisms from unproven behavior.

[Plan](plan.md) retains application acceptance work. The immutable [design work index](plans/design-work-index.md) records historical requirements and provenance; its checklist counts are not a current completion audit. Rendered comparisons must identify the actual source revision, state, terminal geometry and color level independently from an inventory revision. Compilation, tests, prose and candidate baselines alone do not establish parity.

[Development](development.md) describes workspace workflows and evidence limits. Follow the applicable repository and environment execution policies for commands. [Unsafe-operation rationales](../SAFETY.md) record source-qualified reasoning; their presence does not establish every unsafe precondition.

The [historical quickstart](https://github.com/tailrocks/terminal-components-claude/blob/b0edda802880473f5541dc71d18d5cfe5ecd991c/docs/guides/quickstart.md) retains the public-facade update/draw example. Its commands and snippet require current-source verification before reuse; a current quickstart remains pending. Historical branch audits and execution reports are evidence, not product authority.
