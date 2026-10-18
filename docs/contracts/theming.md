# Theme and visual customization contract

## Status

Current theme source index and candidate visual/API contract. The former guide's twelve scenarios and code samples target obsolete `junie_tui` paths; they are not claimed to compile against this workspace. Exact visual output is controlled by `design/interaction-polish`.

## Authority

Current architecture: [architecture](../architecture.md). Product appearance, colors, and interaction: `design/interaction-polish`. Theme implementation: `crates/termrock-theme`; runtime theme application: `crates/termrock-runtime`.

## Current evidence

Current source defines `Theme`, `ColorLevel`, color/design tokens, recipes, `Slot`, `StylePatch`, family/variant definition and override methods, and capability downgrade. Runtime `Ui` defines scoped overlays and theme replacement. These symbols establish current source locations only; historical color values, exact precedence, downgrade algorithms, mono-state guarantees, and allocation claims require source and rendered verification.

The complete former theming guide remains at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/theming.md). Its twelve scenarios and detailed rule tables are retained as historical candidate requirements.

## Remaining work

Verify the following contracts against current implementation and compare every visual result with `design/interaction-polish`:

| Historical section | Requirement to verify | Disposition |
|---|---|---|
| Theme shape | Keep color tokens, layout/design values, recipes, and capability handling in explicit shared theme structures. | Candidate architecture/API contract; verify actual fields and ownership. |
| Scenario 1: default theme | Default construction yields the accepted product appearance. | Design acceptance; current output must be captured and compared. |
| Scenario 2: complete custom theme | An app can provide a complete coherent theme without editing component internals. | Candidate consumer contract; prove with an independent app. |
| Scenario 3: change selected roles | Partial token changes preserve derivation and supported color-level behavior. | Candidate semantic-color contract; verify exact token mapping and rendered states. |
| Scenario 4: family override | Change one component family app-wide without affecting other families. | Candidate customization contract; verify current recipe APIs and isolation. |
| Scenario 5: variant override | Change one variant without changing siblings or other variants. | Candidate customization contract; verify current recipe APIs and isolation. |
| Scenario 6: scoped override | Apply rules to one draw subtree and restore outer rules afterward. | Candidate runtime contract; verify nested overlays and state matching. |
| Scenario 7: instance override | Change one instance while same-family siblings remain unchanged. | Candidate API contract; verify per-component patch support and precedence. |
| Scenario 8: logical parts | Expose stable named parts for styling, identity, and composition. | Candidate component contract; verify each current component's declared parts. |
| Scenario 9: state rules | Match combined runtime/semantic states predictably, including rule precedence. | Candidate rendering contract; verify actual resolution order and state combinations. |
| Scenario 10: app-owned family | Permit custom components to use shared semantic roles and recipes. | Candidate authoring contract; independent downstream consumer acceptance remains open. |
| Scenario 11: color capability | Downgrade built-in and custom theme tokens consistently for each supported terminal color level. | Candidate capability contract; compare actual output at supported levels. |
| Scenario 12: mono state meaning | Preserve distinguishable focus, selection, disabled, error, warning, and editing states without color alone. | Candidate accessibility/visual contract; verify current rules and rendered component matrices. |
| Precedence | Define ordering among base recipes, variants, state rules, global overrides, scoped overlays, and instance patches. | Candidate API contract. Verify actual implementation and docs together. |
| `Slot` merge law | Distinguish inherit, explicit set, and explicit clear across style and glyph values. | Candidate merge contract; verify source and tests. |
| Concern ownership | Keep color, spacing, sizing, glyphs, borders, motion, density, and component state styling in their intended shared or app-owned layers. | Candidate architecture contract; verify current ownership and dependency direction. |

Historical numeric color thresholds, token names, recipe counts, and precedence details remain in the immutable source link until remeasured and reconciled. Do not copy those numbers into current product guidance without verification.

## Verification

Inspect current public exports and theme resolution source. Render built-in and custom themes at supported sizes and color levels, then compare with `design/interaction-polish`. A source symbol or passing unit test alone does not prove visual parity.
