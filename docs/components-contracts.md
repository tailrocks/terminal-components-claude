# Component contracts

## Current API map

The public facade is `termrock` at [`crates/termrock/src/lib.rs`](../crates/termrock/src/lib.rs). It re-exports runtime, session, core identity/input/response types, and component modules from the split crates. `termrock::author` re-exports the current lower-level authoring surface from `termrock-runtime`; see [`termrock-runtime/src/author.rs`](../crates/termrock-runtime/src/author.rs). These exports prove symbols exist, not that the public API is complete, stable, or sufficient for downstream authors.

The workspace separates foundations, layout, theme, text, rendering, collections, runtime, controls, fields, feedback, navigation, viewport, overlays, grid, editors, forms, terminal, and session. Applications are split into showcase, Jackin Preview, Holla, and TablePro crate families. See [architecture](architecture.md) and workspace manifests for exact dependency boundaries.

## Acceptance contracts

- **Independent authoring:** build a downstream consumer using only public `termrock` APIs. Verify construction, caller-owned interaction state, update/draw separation, stable item identity, focus and hit registration, theme parts/overrides, layout measurement, deterministic input, conformance, styled-part accounting, and tiny or empty geometry. No old example is claimed to compile.
- **Identity and event flow:** resolve pointer targets before app code; expose stable item/part identity; distinguish component IDs from subregions; preserve typed responses, consumed-without-repaint behavior, invalidation, and explicit event phases. Keep positional keys only for positional items.
- **State and rendering phases:** keep semantic mutation in update and painting observational except for registration/reporting. Keep runtime interaction state separate from semantic/reference fixture state. Reference rendering must suppress side effects and use exact target state.
- **Theme and customization:** preserve semantic roles and color-capability downgrade; verify family/variant definitions, global overrides, scoped overlays, per-instance patches, part composition, precedence, sibling isolation, `Slot` inherit/set/clear semantics, mono distinguishability, and output against the design reference. Current `Theme`/`Ui` methods do not prove these full guarantees.
- **Components and data:** preserve composable dialog bodies, focus/dismissal/modal ownership, capturing validators/callbacks and typed errors, current-frame geometry facts, app-owned data with visible-row rendering, keyed Grid sorting, edit policies, stable selection, navigation and collection identity, and semantic Grid/viewport behavior.
- **Safety and determinism:** no unchecked string slicing for generated IDs/tokens; masked secrets stay out of rendered/logged/copied output; timers and update causes remain deterministic; captured UI state is not stale for event routing. Preserve safety rationales in [SAFETY.md](../SAFETY.md).
- **Dependency ownership:** reusable generic behavior belongs in shared crates; app-specific domain vocabulary, workflow policy, SQL semantics, and composition belong in app crates. Applications consume shared components rather than painting their screens directly.

## Historical contract sources

Detailed contract maps remain in [authoring](contracts/authoring.md), [migration](contracts/migration.md), [theme](contracts/theming.md), and [customization](contracts/overrides.md). They enumerate independent-consumer expectations, identity and event flow, rendering phases, theme scenarios and precedence, component composition, and app/library ownership. Their former examples and old API paths are not current instructions. The exact original guides remain at immutable [authoring](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/authoring.md), [migration](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/migration.md), [theme](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/theming.md), and [customization](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/overrides.md) snapshots. Treat those snapshots as historical candidate requirements, not verified behavior. Other late architecture decisions are dispositioned section-by-section in [architecture](architecture.md).

## Verification

Use current exports and manifests, focused tests, and independently compiled consumers. Compare visible behavior against `design/interaction-polish`; source symbols or passing unit tests alone do not establish API usability or product parity.
