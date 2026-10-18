# List

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W12 · Group: Collections · Phase: P2.

## Purpose and exclusions

Canonical Termrock contract for the W12 collection component. The contract describes the future API on `termrock-implementation`; it does not rename the current implementation yet.

`List` renders caller-owned keyed rows and provides one shared navigation, selection and scroll mechanism. It borrows the row slice and a constrained row presenter. It does not clone domain rows, own application labels, route pages, load data, or execute a selected action. Reconciliation runs only when the caller declares a new source revision.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
List::new(id: Id, rows: &'a [T], revision: Revision) -> List<'a, T>
where
    T: Keyed;

List::selected(&'a [ItemKey])
    .selection_mode(SelectionMode)
    .row(&'a RowPainter<T>)
    .readiness(Readiness<'a>)
    .disabled(bool)
    .patch(StylePatch);

List::update(&self, cx: &mut Cx<'_>, state: &mut ListState)
    -> Response<ListAction>;
List::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ListState) -> Rect;
List::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`ListState` owns only durable view state: cursor `ItemKey`, range-selection anchor and [`ScrollState`](./scroll-region.md#scroll-and-capture-behavior). Selected keys remain caller-controlled borrowed props. `ListAction` is typed: `Activate { key, origin }` and `SelectionRequested(SelectionRequest)`. Expose read-only cursor, selected-key and scroll observations; keep rows, geometry and runtime handles private.

## Ordinary use

Consumer recipe: [EX-05 — List content without a custom row painter](../api/consumer-recipes.md) (`proposed_target`).

Existing consumers include Showcase list, settings, scrolling and panel pages, TablePro query lists, and Jackin Preview modal lists; their baseline output remains protected.

The caller supplies borrowed rows, a source revision, controlled selected keys, and a constrained row presenter, then applies the typed `ListAction` requests.

## Ownership

- The caller owns row data, readiness, controlled selected keys and the meaning of actions.
- `update` consumes events through the shared runtime and changes durable state or returns typed requests. Keyboard and pointer actions resolve the current stable key at completion.
- `draw` is read-only with respect to semantic state. It may register hit regions and paint from the supplied state, but never advances selection, scroll, animation or reconciliation.
- `measure` reports the required size without registering focus, capture or hit regions.
- Runtime owns focus, hover, pointer capture, geometry, layers and time. The component owns only list-local cursor/selection/scroll state.
- A source revision is the reconciliation boundary. Surviving keys retain cursor and selected meaning through insert, remove, reorder and filter. A removed key becomes an explicit stale target and is not silently rebound to the new occupant of its old index.

Single selection renders one controlled selected key. Multi selection renders a set of controlled keys and supports range extension and select-all through typed requests. Cursor and selection are distinct and may differ.

Shared dependencies: [`scroll-region`](./scroll-region.md#scroll-and-capture-behavior), [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`authoring`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Supported styled parts are `container`, `row`, `gutter`, `marker`, `label`, `meta`, `status`, `scrollbar`, `fade` and `empty`. Theme resolution and [`StylePatch`](../foundations/theme.md) precedence are shared. A row painter can change only its declared row parts; it cannot replace list geometry, focus/capture ownership or the whole surface.

The baseline uses a focus gutter, selected markers (`›` for single selection and `✓` for multi selection), row and metadata hierarchy, disabled styling and a centered empty message. Metadata is shown for the entire view or omitted for the entire view: suppress it when keeping it would leave fewer than 12 cells for the label. Long content is clipped by display-cell width, including Unicode width. Scroll edge fades hint hidden content while protecting the cursor/selected row from being faded. Preserve the exact baseline glyphs, spacing, colors, clipping and surface at every supported capability; do not infer a redesign from the component name.

## Behavior

- Up/Down and `j`/`k` move the cursor; PageUp/PageDown, Home/`g` and End/`G` use the shared scroll/navigation rules. The exact baseline key map is an oracle input, not permission to expose legacy event types.
- Enter and Space request activation or selection for the current enabled key. Disabled rows consume the gesture without an action. In multi mode, range extension and select-all preserve disabled rows and key identity.
- Pointer press/release is a completed gesture. A down event alone never activates. Release outside cancels the pending row activation; source removal or disabling during a held press cancels it rather than retargeting another row.
- Wheel scrolls the list under the pointer and does not transfer focus. Boundary wheels are consumed with no state change. Scrollbar thumb press, track press and captured drag use the shared [`ScrollRegion`](./scroll-region.md#scroll-and-capture-behavior) mechanism.
- Hover may lift a row only while pointer hover is current. Keyboard navigation suppresses stale hover until pointer motion restores it. Disabled rows do not hover or activate.
- Where the baseline supplies press/activation feedback, sample the 140 ms feedback window and its expiry explicitly; redraw frequency never advances it.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, non-zero origin, exact fit, one cell short, long Unicode content, narrow-to-wide resize |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration |
| Activation | pointer down, held press, release inside/outside, removed target, disabled target, keyboard activation, feedback active/expired |
| Source/readiness | empty, ready, cursor separate from selected, disabled row, reorder, insert, remove, filter, stale target, loading, partial, error and offered retry/refresh |
| Scroll | no overflow, start, middle, end, wheel boundaries, thumb drag, resize while scrolled, edge fade, protected cursor/selected row |

Only states meaningful for the supplied readiness and row painter are instantiated. Exact dimensions (`72x20`, `80x24`, `100x30`, `120x40`, `160x50`) and terminal capabilities (`truecolor`, `256`, `16`, no-color and `nocolor`) come from the shared verification contract.

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Starting implementation is `ListBox` in [`src/widgets/list.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/list.rs) (legacy family C22). The legacy `WidgetId`, `Outcome` and `RenderCtx` names are source evidence only. They are not Termrock public API requirements.

The approved capture lane is the frozen baseline, classified before candidate comparison. Use [`../reference/capture-plans/list.json`](../reference/capture-plans/list.json), [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md) and [`../verification/visual-parity.md`](../verification/visual-parity.md); candidate code cannot create expected output.

| Case | Required observation |
|---|---|
| W12-01 | Cursor, controlled selected row and hovered row all differ. |
| W12-02 | Empty, loading, partial and error readiness render distinct caller-supplied states. |
| W12-03 | Sort/insert/remove between pointer down and up, with duplicate labels and distinct keys, never retargets activation. |
| W12-04 | Wheel at both boundaries leaves focus and cursor unchanged; thumb drag maps to the shared scroll region. |
| W12-05 | 100,000 rows draw only visible rows plus documented overscan. |
| W12-06 | A supported row-part patch changes the expected cells and is not overwritten by later custom paint. |

Each trace records dimensions, symbols, wide-cell continuation, foreground/background/modifiers supported by the tool, cursor, focus owner, capture owner, selected stable key, navigation key, readiness and typed action target/count. Invalid oracle setup is blocked, never a pass.

Required negative tests include: draw must not mutate semantic state; duplicate keys are rejected; disabled or stale keys produce no activation; release-outside and removed-target gestures produce no action; selection cannot silently follow an index; wheel cannot move focus; hidden rows cannot register hit regions; Unicode matching and width cannot split a grapheme or continuation cell; and a part override must affect the cells it claims. The component uses the shared engine rather than inventing a second list or scroll implementation.

## Rejected use

Forbidden: painting row strings with a `›` marker and tracking a selected index instead of using List.

```rust
// Forbidden: preview paints its own rows and owns selection.
ui.paint_str(row, 0, "› first item");
if clicked { picked = 0; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate selection. Use `List::new` with stable keys and handle the typed `ListAction`.

## Known gaps

- Capture plan W12 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
