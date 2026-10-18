# Tree

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W15 · Group: Collections · Phase: P2.

## Purpose and exclusions

Canonical Termrock contract for W15. `Tree` presents a caller-owned keyed hierarchy with shared navigation, expansion and scroll behavior.

`TreeSource` supplies stable node keys, parent/child relationships, source revision and readiness. `Tree` flattens only the visible projection, maintains keyed expansion/cursor state and returns typed requests. It does not traverse a filesystem, run Docker, fetch asynchronously, own a loader, or perform domain activation.

## Public API

```rust
Tree::new(id: Id, source: &'a dyn TreeSource) -> Tree<'a>

Tree::selected(&'a [ItemKey])
    .branch_activation(BranchActivation)
    .row(&TreeRowPainter)
    .readiness(Readiness<'a>)
    .patch(StylePatch);

Tree::update(&self, cx: &mut Cx<'_>, state: &mut TreeState)
    -> Response<TreeAction>;
Tree::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TreeState) -> Rect;
Tree::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`TreeState` owns expanded `ItemKey`s, cursor key, range anchor and shared scroll. It never owns node data. Typed actions are `Activate { key, origin }`, `ExpansionChanged { key, expanded }`, `LoadChildren { key }` and `SelectionRequested(SelectionRequest)`. Stable node keys are mandatory; duplicate labels are valid, duplicate keys are invalid.

## Ordinary use

Consumer recipe: [EX-06 — Tree and detail layout](../api/consumer-recipes.md) (`proposed_target`).

Showcase trees, TablePro connection/workbench trees, Jackin Preview inspect trees and Holla disk trees remain unchanged reference consumers.

The caller supplies the borrowed hierarchy source and controlled selected keys, then applies the typed `TreeAction` requests.

## Ownership

- The caller owns the source hierarchy, node readiness, labels, metadata, glyphs, disabled/note state and child delivery.
- Expansion of an unloaded node emits `LoadChildren { key }`; later children are accepted only for the same source revision/generation. A collapse or removal while loading cannot retarget a completion.
- Source replacement/filtering retains surviving node keys. If the focused node disappears, relocate to its visible ancestor, then the documented surviving neighbor; never bind an old path/index to an unrelated node.
- Filtering keeps matching ancestors visible and applies the source's stable-key policy. Expansion/collapse is view state, not a domain mutation.
- `update` performs reconciliation and typed requests; `draw` only paints/registers visible row and disclosure hit regions; `measure` is pure. Runtime owns focus, hover, capture, geometry, layers and time.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `container`, `row`, `indent`, `disclosure`, `gutter`, `marker`, `label`, `badge`, `status`, `scrollbar`, `fade` and `empty`. Theme and [`StylePatch`](../foundations/theme.md) precedence are shared; a row painter cannot replace tree geometry or runtime ownership.

Preserve the baseline indentation at two cells per depth, disclosure glyph/spinner, object glyph, focus gutter, selected marker, label/meta alignment, note styling, busy/loading/error status, clipping and edge fades. The disclosure hit target is two cells wide and remains separate from the row body. In the TablePro fixture, object-kind glyphs are `D S T V ƒ #`; they belong to its supplied row data. A custom row inherits indentation, clipping, focus marker and style parts. Unicode labels and metadata use display-cell measurement.

## Behavior

- Up/Down, `j`/`k`, PageUp/PageDown, Home/`g` and End/`G` move the visible cursor by stable node key. Right/`l` expands a collapsed branch or enters its first child; Left/`h` collapses an expanded branch or moves to its parent. `*` and `-` preserve the baseline expand-all/collapse-all behavior.
- Enter/Space on a branch toggles expansion; on an enabled leaf it emits `Activate` for the node key. Note rows are nonselectable and consume the action. Disclosure click toggles without body activation; row click follows the baseline branch/leaf policy.
- Lazy expansion emits a keyed load request. While busy, the node presents its loading status and does not accept a second unqualified completion. Error/retry is caller-provided readiness.
- Pointer row activation requires a completed inside release. A removed, disabled or stale node cancels the gesture. Hover may lift the row but keyboard input suppresses stale hover until pointer movement.
- Where branch/leaf activation has baseline feedback, verify the 140 ms sample and expiry explicitly; redraw count never advances motion.
- Wheel and scrollbar drag affect the tree under the pointer, preserve cursor identity and do not transfer focus. Boundary wheel events are consumed. Edge fades never obscure the protected cursor/selected row.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, long Unicode content, narrow-to-wide resize |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | disclosure/body pointer down/held/release inside/outside, removed target, disabled target, keyboard activation and applicable feedback |
| Source/readiness | empty, ready, selected differs from cursor, disabled entry, reorder/insert/remove/filter, stale target, loading, partial, error and offered retry/refresh |
| Scroll | no overflow, start/middle/end, boundary wheel, thumb drag, resize while scrolled, edge fade, protected row |

Only applicable states are captured. Dimensions and terminal capabilities come from [`../verification/visual-parity.md`](../verification/visual-parity.md); do not fabricate states for absent branches or decorative notes.

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen implementation is [`src/widgets/tree.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/tree.rs) (legacy family C25), currently exposing `TreeNode`, `TreeView`, path-indexed events, `WidgetId` and `Outcome`. This source is the visual/interaction oracle only.

Use [`../reference/capture-plans/tree.json`](../reference/capture-plans/tree.json). Bind expected output before candidate comparison through [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md). Record exact cells, continuation cells, colors/modifiers supported by the tool, focus/capture/layer owners, selected/cursor key, expansion/readiness and typed action target/count. Missing oracle setup is blocked, never passing.

| Case | Required observation |
|---|---|
| W15-01 | Collapsed/expanded leaf and branch, including loading/error children. |
| W15-02 | Disclosure click differs from body click and preserves branch activation policy. |
| W15-03 | Remove parent/cursor during a press or pending lazy load without retargeting. |
| W15-04 | Expand/collapse all on a large tree draws a bounded visible projection. |
| W15-05 | Filtered tree preserves matching ancestors and stable selection. |

Negative tests must prove: duplicate node keys are rejected; duplicate labels remain distinct; stale lazy completions cannot mutate a replacement node; path/index changes cannot retarget press activation; notes and disabled rows cannot activate; disclosure and body hit regions remain distinct; draw cannot expand/collapse or alter selection; hidden rows cannot be hit; cursor relocation follows documented ancestor/neighbor policy; and a custom row cannot erase required indentation, focus or clipping. Use the shared scroll engine, not a tree-local scrollbar.

## Rejected use

Forbidden: painting indented rows with disclosure glyphs and tracking expansion in preview instead of using Tree.

```rust
// Forbidden: preview paints its own tree and owns expansion.
ui.paint_str(row, 0, "  ▸ src");
if clicked { expanded.insert("src"); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate selection. Use `Tree::new` with stable node keys and handle the typed `TreeAction`.

## Known gaps

- Capture plan W15 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
