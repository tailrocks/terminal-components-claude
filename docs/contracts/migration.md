# Migration contract for the split workspace

## Status

Candidate migration contract, not a current step-by-step recipe. The codebase has moved from the former root monolith into split Rust crates and application crates. The refactor remains incomplete and not ready. The old `crates/tui`, `junie-tui`, `junie_tui`, and `apps/*` references below the archived source are not current package instructions.

## Authority

Current reusable structure: [architecture](../architecture.md). Current UI, interaction, and colors: `design/interaction-polish`. Current work and readiness: [plan](../plan.md). Never infer readiness or product behavior from the historical migration guide.

## Current evidence

Current workspace manifests declare `termrock-*` shared crates and app-specific crates. Current APIs live at their present crate roots and the `termrock` facade. Each historical API mapping below needs source and consumer verification before it becomes migration guidance.

The complete former guide, including its code samples and module maps, remains at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/migration.md). The map below preserves its unique requirements and dispositions. Its “after” code is not claimed to compile now.

## Remaining work

Use this map to verify candidate contracts against current crate exports, application consumers, and the accepted design branch. Reconcile confirmed gaps in the canonical plan. See [component authoring acceptance](authoring.md#remaining-work) for unresolved independent-consumer proof. Do not run the former monolithic package instructions or treat historical “implemented” labels as current proof.

| Historical guide section | Requirement to verify in current workspace | Disposition |
|---|---|---|
| `Outcome` to `Response` | Separate event flow, invalidation, and typed actions; preserve consumed-without-repaint behavior and require handling of `Response`. | Candidate contract. Verify current response methods, event routing, and boundary scroll behavior. |
| `owns`/`locate`/ID helpers to intents | Resolve pointer targets before app code; return stable item and part identity; avoid duplicated hit/geometry lookup. | Candidate contract. Verify runtime regions, actions, and current collection APIs. |
| `WidgetId` to `Id`/`ItemKey` | Use stable domain identity for reorderable collections; reserve positional identity for truly positional items; distinguish component IDs from subregions. | Candidate contract. Verify key reconciliation and public identity APIs. |
| Raw background parameters to `Surface` | Inherit visual planes through draw context; retain semantic roles and capability handling. | Candidate visual/API contract. Match actual colors and plane behavior to `design/interaction-polish`. |
| Forced component state to reference scopes | Keep runtime-owned interaction state separate from semantic fixture state; scope injected state and suppress side effects in reference rendering. | Candidate testing/paint contract. Verify current reference API and capture path. |
| `DialogBody` to composed bodies | Allow app-owned body content while keeping named actions, focus, dismissal, and modal ownership coherent. | Candidate API/behavior contract. Verify current overlays and dialog actions. |
| Function pointers to validators/callbacks | Permit application-capturing validation and extension behavior while preserving typed errors and borrow lifetimes. | Candidate API contract. Verify actual traits and closures in current field/form crates. |
| Public geometry fields to frame reads | Prevent stale event routing from render-cached rectangles; expose only current/last-frame facts with explicit absence. | Candidate safety contract. Verify current `Cx`/`FrameRead` behavior and consumers. |
| Render-time commits to update phase | Keep semantic mutation in update and painting observational; permit only runtime registration/reporting during draw. | Candidate phase contract. Verify actual signatures and state transitions. |
| Inline keys to bindings | Keep displayed hints tied to handled chords; allow app remapping and explicit capture/bubble policy. | Candidate interaction contract. Verify current binding APIs and hint behavior. |
| Owned rows to borrowed data | Keep domain data caller-owned, render only visible rows, and avoid per-frame string or row copies. | Candidate API/performance contract. Measure current consumers before retaining old allocation claims. |
| “Other deletions” inventory | Preserve nested focus scopes, layer identity, shared scroll behavior, semantic grids, text viewports, and app-owned domain vocabulary. | Candidate boundary checklist. Each old deletion and replacement needs current source proof. |
| Part 2 foundation module map | Separate identity, input, response, intent, runtime/session, theme, UI, layout, and layer responsibilities by crate. | Historical path map. Verify current crate ownership and dependency direction from manifests. |
| Part 3 widget module map | Preserve component-specific behavior for controls, fields, navigation, overlays, grid, editors, and viewport. | Historical inventory. Old checkmarks and APIs do not establish current parity. Verify against current source and the design reference. |
| Part 4 app-level machinery | Keep generic focus, hit testing, layers, capture, repaint, and intents in shared runtime; keep navigation/domain actions in each app. | Candidate architecture contract. Current code must prove the boundary without changing product behavior. |
| Porting order | Identity, phase split, event routing, response handling, surfaces, semantic roles, bindings, and conformance were proposed as a migration sequence. | Retired workflow. The package split already exists; use current plan/status for remaining work order. |

## Verification

For every retained requirement, cite current source and a consumer or focused test. Verify behavior and rendered output against `design/interaction-polish`. Compilation alone does not prove compatibility, completion, or visual parity.
