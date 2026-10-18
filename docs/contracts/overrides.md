# Component override contract

## Status

Current source index and candidate customization contract. The detailed old guide used `junie_tui` imports and contained code examples not verified against current APIs. No example here is claimed to compile as written.

## Authority

Current architecture: [architecture](../architecture.md). Product colors and appearance: `design/interaction-polish`. Public theme sources: `crates/termrock-theme`; draw-scoped overlay source: `crates/termrock-runtime`.

## Current evidence

Current theme source defines `Theme::define_family`, `define_variant`, `override_family`, `override_variant`, and `downgrade`; the theme crate defines `StylePatch`, `Slot`, and `Overlay`. Current runtime `Ui` defines `with_overlay`. These symbol locations prove implementation exists, not every old precedence, allocation, propagation, or consumer guarantee. Per-instance `.patch`, `.patch_part`, and `.slot` behavior must be verified on each current component that exposes it.

The full historical customization guide remains at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/overrides.md). Its scenarios and exact code are archived for traceability, not current instructions.

## Remaining work

Verify these customization scopes and contracts against current public exports, current component builders, and the accepted design branch:

| Historical section | Requirement to verify | Disposition |
|---|---|---|
| `.patch_part` | Apply style changes to named parts of one component instance; siblings remain unchanged. | Candidate API contract. Confirm builder availability, precedence, and part isolation by component. |
| `.patch` | Apply a uniform instance-level patch without affecting siblings. | Candidate API contract. Verify actual component API and merge behavior. |
| Part slots | Replace painting for a named part while keeping ownership, geometry, and other parts intact. | Candidate composition contract. Verify callback lifetime, interaction registration, and inert/reference behavior. |
| Custom variants | Define or select a variant without changing unrelated variants. | Candidate theme contract. Verify identifiers, defaults, and override precedence. |
| Custom families and recipes | Define all styled parts for an app-owned family; distinguish recipe definition from restyling an existing family. | Candidate theme/API contract. Verify neutral fallback and mono behavior. |
| `define_*` versus `override_*` | Keep recipe ownership distinct from application-wide changes to existing families/variants. | Current methods exist; semantics and precedence need current tests and consumer verification. |
| Scoped overlays | Apply borrowed rules only to a draw subtree, then restore prior scope. | Current `Ui::with_overlay` exists; verify nesting, precedence, allocation, and no theme mutation. |
| Scenario C: local sibling override | Demonstrate one instance changes while same-family siblings remain unchanged. | Historical example only; rebuild as an independent consumer after API verification. |
| Choosing | Select the narrowest customization scope that meets the product requirement. | Retain as guidance after the supported mechanisms and boundaries are verified. |

Do not change component source merely to restyle a family. Keep shared mechanisms in reusable crates and app-specific appearance choices in application theme setup where supported.

## Verification

Build an independent consumer for each supported scope. Verify rendered cells and sibling isolation across relevant states, themes, and color levels. Compare results with `design/interaction-polish`; old code samples and test names are not current proof.
