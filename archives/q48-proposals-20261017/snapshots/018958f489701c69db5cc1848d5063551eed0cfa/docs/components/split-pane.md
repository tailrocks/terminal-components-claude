# SplitPane

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W33 · Phase: P2 · Legacy family: C42.

## Purpose and exclusions

SplitPane is one axis-parameterized two-pane allocation and seam interaction mechanism. It allocates first and second caller-supplied panes along a horizontal or vertical axis, paints/registers the seam, and emits typed resize/maximize actions. The caller supplies pane content and decides what each pane means.

SplitPane must not own PTYs, processes, repositories, terminals, session routing, or child trees; use separate horizontal and vertical engines (axis is a parameter of one mechanism); own process or terminal lifecycle or application-specific pane routing; change a preferred ratio merely because the current terminal is too small; produce negative/wrapped pane geometry below combined minima; or promise a universal widget trait or current `WidgetId`/`Outcome`/`RenderCtx` API.

## Public API

The signatures below are the proposed target, not a claim about current Rust exports. No signature is source-checked.

```rust
SplitPane::new(id: Id, axis: Axis) -> SplitPane

update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut SplitPaneState,
) -> Response<SplitAction>

draw<R>(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &SplitPaneState,
    body: impl FnOnce(&mut Ui<'_>, SplitAreas) -> R,
) -> R

measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

Configuration:

```rust
minima(first: u16, second: u16)
seam_width(u16)
resizable(bool)
patch(StylePatch)
```

`SplitPaneState` owns desired ratio/fixed allocation and optional maximized child `ItemKey`. Runtime owns pointer-grab offset, capture, and hit geometry. The typed actions are:

```rust
SplitAction::Resized { ratio: SplitRatio }
SplitAction::Maximized { child: Option<ItemKey> }
```

The caller decides whether to persist or reconcile those preferences.

## Ordinary use

Consumer recipe: [EX-06 — Tree and detail layout](../api/consumer-recipes.md) (`proposed_target`).

Real preview consumers are the Jackin Preview capsule and accounts compositions. An independent synthetic fixture supplies two pane bodies with distinct minima and stable child keys.

The caller supplies both pane contents, applies the typed resize preference, and decides the meaning of each pane. The layout helper saturates under small constraints. If the API updates split preference directly, no redundant helper remains. Specify one accepted policy before implementation.

## Ownership

- The caller owns pane content, the meaning of each pane, and whether to persist resize/maximize preferences.
- SplitPane owns desired ratio/fixed allocation, optional maximized child key, seam painting/registration, and typed resize/maximize actions.
- `update` handles focus, seam pointer down/drag/release, keyboard increments, zoom/maximize commands, and typed actions. Pointer capture keeps the initial grab offset under the pointer and release outside ends the drag safely.
- `draw` derives clamped displayed allocations from current area, axis, seam, and minima, paints/registers the same seam rectangle, and invokes each nonempty body exactly once. It cannot overwrite the preferred ratio or mutate durable state. Below combined minima it uses the explicit responsive/drawer policy; there is no negative or wrapped dimension.
- `measure` computes pane rectangles and seam from constraints without changing ratio, maximized key, or body state. Narrowing clamps the display allocation; widening restores the preferred ratio. Nested splits reconcile removed or maximized children by stable key.
- Runtime owns pointer-grab offset, capture, and hit geometry. Focus/hover styling belongs to the seam runtime owner.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `first-body`, `seam`, and `second-body`.

Ordinary customization patches the declared seam tone. The baseline seam glyph is `│` or `─` when idle/hovered according to axis and `┃` or `━` while pressed/dragged. Hover and pressed semantic border tones must survive monochrome capabilities. The seam is quiet when idle and visibly strong while captured. Empty seam (gap zero or maximized pane) paints/registers nothing.

Advanced customization keeps exact pane body clipping, nonzero origins, minima, and allocation pixels. Colors resolve through shared theme capability policy; motion is only the shared press/resize timing if a recipe advertises it. Part patches cannot replace layout, focus/capture, or the whole surface.

## Behavior

Horizontal and vertical axes use identical rules. The seam is the only pointer resize target. A pointer press on the seam enters runtime capture; drag follows the pointer with its original offset, clamps both minima, and emits one resize action per effective ratio change. Pressing first/last seam cells and dragging outside are required cases.

Keyboard resize nudges the same preferred ratio used by pointer resize. Zoom or maximize selects a stable child key; toggling the same key restores both panes. Removing a maximized child clears the stale key and returns to the valid nonmaximized composition according to caller policy.

Keyboard suppression and pointer-motion restoration follow shared runtime rules. SplitPane has no generic activation animation; pressed/dragging is a seam interaction state.

Identity and reconciliation rules:

- The SplitPane identity and each child `ItemKey` are stable semantic IDs.
- Maximized state is keyed, not a display index.
- A child removal while maximized clears the stale key rather than granting the other child an accidental maximized identity.
- Preferred ratio is durable; displayed clamp is derived from current geometry.
- Nested splits use the same identity and ratio reconciliation rules.

## Visual matrix

| Axis | Required states | Applies when |
|---|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | Always |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover | Seam or child interactive |
| Activation | No generic whole-surface activation; resize and maximize are typed | Split recipe |
| Pointer | seam press, held drag, release inside, release outside, first/last cell | Resizable |
| Resize | keyboard nudge, ratio clamp, minima, below-minimum responsive policy, restore after widen | Resizable |
| Maximization | none, first, second, toggle restore, removed maximized child | Zoom policy |
| Motion | applicable pressed/drag phase, paused/reduced motion | Theme policy |

Generic content editing, selection, and scroll states belong to child panes.

## Verification

All parity is pinned to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Source evidence is `src/widgets/splitter.rs` (blob `80c113f7585827d340da45921011c2a1953a8b34`) and split allocation helpers in `src/ui/layout.rs`. Inline layout tests cover minima, maximize, drag clamping, nudge, and directional allocation. Splitter source covers seam registration, hover/pressed glyphs, and drag dispatch. Visual consumers are Jackin Preview capsule and accounts compositions, plus protected snapshots under `snapshots/jackin/capsule` and `snapshots/jackin/accounts`. Interaction traces remain under `tests/visual_baseline`. See [`visual-parity`](../verification/visual-parity.md), [`oracle-and-provenance`](../verification/oracle-and-provenance.md), and [`public-api`](../api/public-api.md).

Use [`../reference/capture-plans/split-pane.json`](../reference/capture-plans/split-pane.json). It is planned with no expected artifacts. Bind the frozen source as `ExistingOracle`/`ExtractedOracle`; API/safety extensions are explicit `Extension` cases.

Run applicable cases at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50`, with truecolor, 256-color, 16-color, `none`, and `nocolor` capabilities.

| Case | Required observation |
|---|---|
| W33-01 | Horizontal and vertical allocations use identical rules. |
| W33-02 | Seam drag at first/last cell and resize during capture. |
| W33-03 | Narrow below minima then widen restores preferred ratio. |
| W33-04 | Keyboard increments and maximize/unmaximize share state. |
| W33-05 | Nested panes with one child removed while maximized. |

Record seam cells, pane dimensions, ratio/preferred ratio, min clamps, focus/capture/layer owners, action count and action target, capability, and release result. Compare symbols, continuations, colors/modifiers, and exact dimensions.

Required negative tests:

- The painted seam and registered hit rectangle must be identical.
- Pointer release outside cannot leave capture or pressed seam state.
- Drag cannot violate minima or overwrite the preferred ratio with a temporary narrow allocation.
- Below combined minima cannot create negative, wrapped, or overlapping panes.
- Keyboard resize and pointer resize cannot diverge in ratio semantics.
- Removing a maximized child cannot retarget maximization to another key.
- `draw`/`measure` cannot mutate ratio, maximized key, or child state.
- Part customization cannot replace layout, seam hit geometry, or capture.

Acceptance requires an external two-pane consumer, exact applicable snapshots, interaction traces, source-state tests, and independent review.

## Rejected use

Forbidden: painting a manual `│` divider column and splitting pointer columns by hand instead of using SplitPane.

```rust
// Forbidden: preview owns the seam and pane split.
for y in 0..height { ui.paint_str(y, mid, "│"); }
let left = area.x < mid;
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must not recreate component rendering or generic interaction. The following patterns are prohibited in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Use `SplitPane::new` with an axis parameter and stable child keys, and handle the typed `SplitAction`.

## Known gaps

- Capture plan W33 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
