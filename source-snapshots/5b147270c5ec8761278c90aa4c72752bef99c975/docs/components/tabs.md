# Tabs

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W17 · Group: Collections · Phase: P2.

## Purpose and exclusions

Canonical Termrock contract for W17. `Tabs` is one keyed tab-strip engine with optional close, new, reorder and context requests. It does not own tab sessions or the content hosted by a tab.

The caller supplies stable tab keys, labels, status metadata, close eligibility and the controlled active key. `Tabs` owns only strip navigation/cursor/scroll view state and emits typed activation, close, reorder and context requests. Runtime remembers nested content focus scopes; `Tabs` never owns a session, document, pane or product workflow.

## Public API

```rust
Tabs::new(id: Id, tabs: &'a [TabItem<'a>], revision: Revision) -> Tabs<'a>

Tabs::active(Option<ItemKey>)
    .closable(bool)
    .reorderable(bool)
    .part_painter(&TabPartPainter)
    .patch(StylePatch);

Tabs::update(&self, cx: &mut Cx<'_>, state: &mut TabsState)
    -> Response<TabsAction>;
Tabs::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TabsState) -> Rect;
Tabs::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`TabsState` owns the navigation cursor key and strip window/scroll. Active key remains caller-controlled. Typed actions are `Activate { key, origin }`, `Close { key, origin }`, `Reorder { key, before: Option<ItemKey> }` and `Context { key, anchor: Anchor }`. Stable keys, not display indexes, identify every tab and close/context affordance.

## Ordinary use

Consumer recipe: [EX-07 — Tabs and screen selection](../api/consumer-recipes.md) (`proposed_target`).

Existing consumers include Showcase picker/chrome pages, Holla's tab strip and Jackin Preview capsule, editor and settings screens. These applications and their content remain parity fixtures.

The caller supplies borrowed tabs, a source revision, and the controlled active key, then applies the typed `TabsAction` requests.

## Ownership

- Caller owns tab content identity, labels, dirty/busy/error status, close policy and the active value.
- Cursor, active tab, hovered tab, close target and activation feedback are independent. Switching tabs requests a new active key; it does not transfer or destroy child content state.
- Source revision reconciliation retains surviving keys through insert, removal and reorder. A held close/context gesture resolves only its captured key; removal cannot close a new occupant of the old index. Closing the active tab asks the caller for the next active key under its policy.
- Inactive tab content has no active hit regions or hardware cursor ownership. Returning to a tab restores the runtime's saved child focus scope where supplied.
- `update` is the only semantic mutation path. `draw` paints/registers current tab, close, overflow and new-button hit areas without changing active/cursor/source state. `measure` is pure.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `container`, `tab`, `gutter`, `marker`, `label`, `meta`, `status`, `close`, `underline` and `overflow`. Theme/style patches follow [`theme`](../foundations/theme.md); part painters cannot replace strip geometry or focus/capture ownership.

The frozen strip has one row of labels and an underline row. Each tab may carry a prefix and suffix; the status slot after its label carries dirty `•`, error `!`, or busy spinner, while the suffix slot may carry a muted state glyph such as `▶`, `●`, `○`, or `◆` that becomes secondary text on the active tab. Preserve the baseline blank rows, width policy, close affordance, overflow indicators and new-tab affordance.

The active tab is one plane above the strip, bold, primary text, with an accent
`━` underline spanning exactly its plane edge to edge. A nested quiet strip
keeps the plane and bold but uses a `border-strong` underline. Hover on an
inactive tab lifts it two planes, uses primary text, and has no underline or
bold. A keyboard cursor on an inactive tab uses that lifted plane and bold,
with no underline. Thus the underline means active, plane lift means hover or
cursor, and bold marks keyboard focus. Selected, hovered and cursor states
remain visually distinct. Labels never shrink by silently changing the
baseline recipe; Unicode display width and clipping are measured in cells.

## Behavior

- Plain Left/Right and `h`/`l` move the cursor/active key according to the baseline. Plain digits select the corresponding tab where offered. Enter/Space activates the cursor; `x` or Delete requests close only for a closable tab. `n` or the configured new affordance requests a new tab.
- Pointer click on a tab activates its stable key. Close, new and overflow controls are separate hit parts. Context requests target the captured key and anchor; their product behavior remains caller-owned.
- Reorder uses stable-key `before` placement and pointer capture. Source changes during a held drag cannot retarget another tab. If reorder is not enabled, no hidden reorder path exists.
- Hover styling is suppressed after keyboard navigation until pointer motion. Release outside cancels pending activation/close/reorder. Disabled or non-closable tabs retain visible status but emit no prohibited action.
- Strip scroll and overflow controls use shared runtime capture/scroll rules. Resizing or label changes remeasure, preserve active/cursor key and keep the active tab visible.
- Activation feedback, where present in the baseline recipe, is sampled at 140 ms and at expiry; redraw frequency never advances it.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, Unicode labels, narrow-to-wide resize |
| Focus/hover | unfocused, focused cursor, hovered, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | tab/close/context pointer down/held/release inside/outside, removed target, disabled/non-closable target, keyboard activation, 140 ms feedback where baseline supplies it and expiry |
| Source | empty, ready, active differs from cursor, disabled/status variant, insert/remove/reorder, stale target |
| Scroll | no overflow, start/middle/end, overflow-button and wheel boundary, thumb/captured drag, resize while scrolled and protected active tab |

Layer/focus restoration applies when tabs are nested in overlays or workspaces. Exact dimensions and capabilities are defined by [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The visual oracle is [`src/widgets/tabs.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/tabs.rs) (legacy family C27), currently using `TabItem`, `Tabs`, `TabEvent`, `WidgetId` and `Outcome`.

Use [`../reference/capture-plans/tabs.json`](../reference/capture-plans/tabs.json). Compare symbols, wide-cell continuation, foreground/background/modifiers supported by the tool, cursor visibility/position, focus/capture/layer owner, active/cursor/close keys and typed action count/target. Bind expected output first with [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md); candidate code never blesses its own baseline.

| Case | Required observation |
|---|---|
| W17-01 | Active/inactive plus hovered/focused/pressed/close-hover variants remain distinct. |
| W17-02 | Overflow scroll followed by a longer label preserves active key and baseline width/indicators. |
| W17-03 | Deleting the active tab restores successor focus deterministically through caller policy. |
| W17-04 | Context/close while the source reorders targets the original stable key. |
| W17-05 | Switching back restores per-tab child focus where a nested scope exists. |

Negative tests must prove: duplicate keys are rejected; active/cursor/hover/close targets cannot collapse into one state; removed or reordered tabs cannot retarget a gesture; close on a non-closable tab emits nothing; inactive content cannot own focus/hit/cursor; draw cannot change active/cursor/source; overflow controls cannot lose the active tab; mode/label changes preserve stable identity; and a tab part patch changes only declared cells. There is one tab engine; entity-strip recipes are compositions of its parts.

## Rejected use

Forbidden: painting a tab strip and switching content on click instead of using Tabs.

```rust
// Forbidden: preview paints its own tab strip and owns switching.
ui.paint_str(row, 0, "[ General ]  Mounts");
if clicked { active = 1; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate selection. Use `Tabs::new` with stable keys and handle the typed `TabsAction`.

## Known gaps

- Capture plan W17 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-004 historical Capsule screens](../implementation/code-remediation-backlog.md): pending; the Jackin Preview capsule, editor, and settings consumers still use historical tab paths.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
