# Picker

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W18 · Group: Overlays · Phase: P4.

## Purpose and exclusions

Canonical Termrock contract for W18. `Picker` is the shared searchable/fixed-choice overlay mechanism used by command and item selection flows.

The caller supplies eligible keyed results, title/query presentation, readiness and filtering policy. `Picker` owns query draft, highlighted result and viewport state while open, then returns typed actions. It does not access a filesystem, credentials, network search, executor, provider, shell or product domain.

## Public API

```rust
Picker::new(id: Id, items: &'a [PickerItem<'a>], revision: Revision) -> Picker<'a>

Picker::title(&'a str)
    .query(&'a str)
    .searchable(bool)
    .readiness(Readiness<'a>)
    .row(&PickerRowPainter)
    .patch(StylePatch);

Picker::update(&self, cx: &mut Cx<'_>, state: &mut PickerState)
    -> Response<PickerAction>;
Picker::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &PickerState) -> Rect;
Picker::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`PickerState` owns query/editor state, highlighted stable result key and scroll. Runtime owns modal/popover layer lifetime, focus trap, backdrop, placement and restoration. Typed actions are `Accept { key, alternate: bool, origin }`, `ScopeNext` and `Dismissed`. Read-only query/current-key/scroll observations are allowed; caller-owned results remain immutable borrowed props.

## Ordinary use

Showcase picker pages, Holla activities/files pickers and Jackin Preview manager, capsule, editor, settings and modal pickers are preserved consumers.

The caller supplies borrowed items, a source revision, and readiness, then applies the typed `PickerAction` results.

## Ownership

- Query edits are actions; the caller re-supplies results with a revision. The component never commits a query to product state or mutates result data.
- Stable keys survive refresh, insertion, removal and reorder. A vanished/disabled result is ineligible and cannot be activated by Enter, Delete, pointer or a stale held gesture.
- `Readiness` distinguishes ready, loading, partial, error and retry/refresh presentations. Loading and error rows are status content, not selectable results.
- Search-disabled mode still navigates, scrolls and dismisses; it removes the query editor without creating another engine.
- `update` owns semantic state and typed requests. `draw` is read-only, registers the modal surface and visible row hits, and never pulls the viewport because of a repaint. `measure` is pure.

Shared dependencies: [`filter-list`](./filter-list.md), [`text`](../foundations/text.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`identity`](../foundations/identity.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `container`, `title`, `query`, `breadcrumb`, `row`, `gutter`, `marker`, `label`, `detail`, `status`, `footer`, `scrollbar`, `fade` and `empty`. Theme patches follow [`theme`](../foundations/theme.md); part customization cannot replace modal geometry, focus/capture, backdrop or layer ownership.

Preserve the baseline centered rounded modal/popover geometry, modal backdrop, title/scope hierarchy, query field/caret, row gutter/marker, match/detail/tag alignment, disabled styling, status rows, footer hints, clipping, scrollbars and fades. Query caret placement and Unicode display width are exact parity data. Narrow screens clamp the modal and retain at least the meaningful result row under the baseline rules.

The frozen modal sits in the upper third of the page. Its title has the scope
readout at right; searchable modes show an accent-marked query field. Result
rows use fixed columns for gutter, glyph, label, detail, tag, and group, with
widths computed from all items so scrolling never shifts alignment. The hint
row stays faint at the bottom. Search-disabled mode omits only the query editor
and retains the same single picker/list mechanism.

## Behavior

- Open creates a runtime-owned modal layer and traps focus. Query input has priority over background shortcuts. Close restores the previous focus owner and does not change caller product state.
- Enter accepts the current eligible stable key. Alt+Enter returns alternate intent. Tab returns `ScopeNext`. Search-disabled pickers use the baseline navigation keys without text editing.
- Escape first clears a non-empty searchable query according to the frozen behavior; a subsequent Escape dismisses the picker. Dismissal returns `Dismissed` and lets the caller decide what happens next.
- Backspace removes one grapheme; word deletion and paste use the shared text core. Empty-query Backspace returns `Back`. Paste normalizes line breaks/tabs to spaces and emits one query-change action when searchable.
- Delete/secondary is available only for an eligible current result where configured. Disabled, loading, error and empty-result rows consume activation without an action.
- Pointer click requires a completed inside release and resolves the captured stable key. Outside click dismisses only when the runtime layer policy permits it. Wheel scroll and thumb drag stay in the picker, preserve query/cursor identity and do not transfer focus.
- Hover may lift eligible rows; keyboard activity suppresses stale hover until pointer motion. Reanchor/clamp the modal on resize without changing semantic selection.
- Where a result action has baseline feedback, verify the 140 ms sample and expiry explicitly; motion is driven by supplied time, not repaint count.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, long Unicode query/results, narrow geometry and caret placement |
| Focus/hover | unfocused background, focused query, focused row, hovered, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | pointer down/held/release inside/outside, removed/disabled target, keyboard accept/alternate, activation feedback where baseline provides it |
| Source/readiness | empty, ready, selected differs from cursor, disabled/unavailable, insert/remove/reorder/filter, stale target, loading, partial, error, retry/refresh |
| Scroll/layer | no overflow/start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade/protected row; closed/open/nested/Escape/outside/owner removed/reanchor/focus restore/modal-first-paste |

Use only applicable combinations. Shared dimensions/capabilities and motion policy are in [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen implementation is [`src/widgets/picker.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/picker.rs) (legacy family C28), currently using `Picker`, `PickerItem`, `PickerStatus`, `PickerEvent`, `WidgetId` and `Outcome`. These current names and source layouts are evidence, not future Termrock API promises.

Use [`../reference/capture-plans/picker.json`](../reference/capture-plans/picker.json). Bind expected output before candidate comparison through [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md). Record exact cells and wide-cell continuation, styles/modifiers supported by the tool, cursor visibility/position, focus/capture/layer owner, query draft/committed state, selected key and typed action count/target. Missing oracle setup is blocked, never passing.

| Case | Required observation |
|---|---|
| W18-01 | Searchable and non-searchable modes preserve distinct geometry and key handling. |
| W18-02 | Loading then partial results; empty versus error; retry action when supplied. |
| W18-03 | Nested picker above a dirty form: paste, Escape and outside click preserve form draft/focus policy. |
| W18-04 | Unavailable/destructive item action retains its exact stable key after reorder. |
| W18-05 | Narrow geometry preserves baseline modal placement and query caret. |

Negative tests must prove: no filesystem/network/credential/executor behavior; duplicate or empty identity handling is explicit; disabled/loading/error rows cannot activate; stale keys cannot retarget; query edits are grapheme-safe and one paste is one action; Escape ordering and focus restoration are stable; outside click respects the modal layer; draw cannot mutate query/cursor/scroll; candidate code cannot create expected snapshots; and the component shares one picker/filter/list mechanism.

## Rejected use

Forbidden: painting a picker modal with query text and choosing an item in preview instead of using Picker.

```rust
// Forbidden: preview paints its own picker modal and owns choice.
ui.paint_str(row, 0, "Find: app");
if clicked { chosen = items[0]; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate menus. Use `Picker::new` with stable keys and handle the typed `PickerAction`.

## Known gaps

- Capture plan W18 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-002 historical Manager/modal screens](../implementation/code-remediation-backlog.md): pending; the Jackin Preview manager and modal picker consumers still use historical paths.
- [FIX-004 historical Capsule screens](../implementation/code-remediation-backlog.md): pending; the Jackin Preview capsule, editor, and settings picker consumers still use historical paths.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
