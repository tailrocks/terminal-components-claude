# Documentation

Use these pages for current project direction and work:

- [Architecture](architecture.md): implemented crate graph and reusable API direction. `main` is the structural authority; its refactor remains incomplete and not ready.
- [Product design](design.md): `design/interaction-polish` is the authority for UI, colors, interaction, behavior, and planned product work.
- [Component contracts](../COMPONENT_ARCHITECTURE.md): current public API locations and requirements awaiting consumer or behavior verification.
- [Plan](plan.md): open architecture, application, visual, and conformance work.
- [Development](development.md): toolchain, run commands, workspace checks, and generated visibility records.
- [Dependencies](dependencies.md): current tool pins, upgrade policy, upstream constraints, and frozen historical tool inputs.
- [Quick start](guides/quickstart.md): current in-workspace facade example; verify APIs against source before depending on them externally.
- [Safety rationales](../SAFETY.md): retained explanations linked to their original source context.
- [Engineering coordination](../COORDINATION.md): ownership and evidence handoff for parallel changes.

## Historical and frozen evidence

The `baseline/before/` path is an optional output of `tools/baseline_capture.sh`, not a checked-in current-design contract. The historical renderer and capture qualification workflows retain revision-bound inputs, outputs, and review evidence under `docs/historical-regeneration/` and `tools/qualified-capture/`. Treat those files as evidence only for their pinned source and scope; they do not establish present product readiness or authorize baseline updates. Root `STATUS.md` and `WORK_QUEUE.md` are generated publisher/test records, not product guidance. Their prose and structured inputs have separate roles, described in [development](development.md).

The [design work index](plans/design-work-index.md) preserves immutable acceptance detail from the reviewed design revision. Its item counts describe that source inventory and are not current completion measures. Retired audits, prompts, progress reports, and branch handoffs are not current project authority.
