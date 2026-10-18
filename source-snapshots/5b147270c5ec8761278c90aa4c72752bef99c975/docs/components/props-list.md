# PropsList

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Inventory: W35 · data and text · P3 · baseline composition.

## Purpose and exclusions

PropsList is the interactive, scrollable property sheet. It reuses the Props row painter and the shared List and ScrollRegion mechanisms. The caller owns row data, safe copy payloads and protected-value policy.

PropsList must not own a second list, scrollbar or row-rendering engine; copy clipped display text or reveal protected data; expose mutable domain rows, runtime geometry or a raw index as identity; or poll services or interpret property values.

## Public API

The signatures below are the proposed target, not a claim about current Rust exports. No signature is source-checked. The frozen implementation currently uses index-backed rows internally; the target API replaces that accidental identity with stable semantic row keys.

```rust
PropsList::new(
    id: Id,
    rows: &'a [PropsRow<'a>],
    revision: Revision,
) -> PropsList<'a>

update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut PropsState,
) -> Response<PropsAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &PropsState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Builders:

```rust
label_width(LabelWidth)
copy_policy(CopyPolicy)
patch(StylePatch)
```

PropsState is caller-owned durable state containing a keyed cursor, selection where applicable and shared ScrollState. Fields remain private; read-only observations expose the selected key and scroll position without exposing geometry.

```rust
PropsAction::CopyRequested { key: ItemKey }
```

CopyRequested identifies source data. It never contains rendered or truncated text. Activation of a row, if a future consumer needs it, is a separate typed action and must not be inferred from copying.

## Ordinary use

PropsList appears in [EX-06 — Tree and detail layout](../api/consumer-recipes.md) (`proposed_target`) as the detail pane inside Panel.

Real preview consumers are the frozen Jackin accounts and manager views. An independent synthetic fixture supplies keyed rows with safe copy payloads, one protected row, and one wrapped value.

The caller supplies borrowed row data with a source revision; `selected_properties` returns borrowed property data and cannot draw. PropsList handles navigation, scroll, and copy eligibility; the caller applies the typed copy request to source data.

## Ownership

| Concern | Owner |
| --- | --- |
| Rows and safe copy data | Caller, borrowed for the frame |
| Cursor, selected key and scroll offset | PropsState |
| Focus, hit testing, hover and pointer capture | Runtime |
| Reconciliation across revision | Collections foundation |
| Copy policy and protected-value rejection | Caller policy plus component gate |
| Painting and measured row geometry | Shared Props painter and ScrollRegion |

update handles keyboard navigation, pointer row targeting, wheel scrolling, thumb presses/drags and copy eligibility. It returns a typed response and never mutates the source slice. draw is read-only with respect to PropsState; measure and draw share wrapped-row geometry so hitboxes and scroll ranges agree.

Reordering, insertion, removal and filtering reconcile by stable key; a stale pointer target cannot retarget a new row.

Shared dependencies: [`identity`](../foundations/identity.md), [`input/actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`text`](../foundations/text.md), [`collections`](../foundations/collections.md), [`theme`](../foundations/theme.md), [`authoring`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Parts are container, row, gutter, marker, label, value, status, scrollbar and fade. Props supplies label/value alignment and wrapping. List and ScrollRegion supply focus gutter, row marker, scrollbar thumb, edge fade and pointer capture. Style patches cannot replace hit geometry or runtime ownership.

Ordinary customization patches declared row, label, or value tones. Unicode grapheme widths, combining marks and wrapped values must match between measurement, painting and hit testing. Preserve semantic tones and capability fallbacks through [theme](../foundations/theme.md).

Advanced customization keeps protected-row presentation: protected rows may show a locked or masked state. Read-only means no editing; it does not make every value safe to export. A protected copy request is rejected with no action. Scroll thumb drag, resize-while-scrolled, top/middle/end offsets and protected-row fades follow [runtime](../foundations/runtime.md), [layout](../foundations/layout.md) and [text](../foundations/text.md).

## Behavior

The baseline key behavior includes Up/Down, j/k, PageUp/PageDown, Home, End/g/G, Enter and y for an eligible copyable row. The final keymap is owned by [input/actions](../foundations/input-actions.md); preserve observable baseline behavior while allowing caller remapping through typed bindings.

There is no text editing and no component-owned animation. Protected values never enter CopyRequested; rendered truncation cannot become an action payload. Row removal during a pointer press cannot activate a different row. Pointer capture ends on release outside the list.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | pointer down, held press, release inside/outside, removed target, disabled target, keyboard copy/activate, feedback timing |
| Source | empty, ready, selected differs from cursor, disabled row, reorder, insert, remove, filter, stale target |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row |
| Editing/motion | no text editing; no component-owned animation |

## Verification

Oracle evidence is the [frozen widget source](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/props.rs) at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Snapshot discovery is frozen Jackin accounts and manager views. See the [PropsList capture plan](../reference/capture-plans/props-list.json), the [List component contract](./list.md), the [Props contract](./props.md), and the [visual parity proof](../verification/visual-parity.md). The capture plan is planned with no approved expected artifacts.

| Case | Required observation |
| --- | --- |
| W35-01 | Normal copy returns source identity, never clipped display text |
| W35-02 | Protected row copy is rejected |
| W35-03 | Reorder/delete preserves or explicitly drops the keyed hovered/selected property |
| W35-04 | Wrapped rows keep accurate hit and scroll geometry with protected-row fade |

Record exact frame cells/styles, dimensions and capabilities, cursor, focus owner, capture owner, selected stable key, navigation key, source revision, draft/selection state and typed action target/count.

Required negative tests:

- protected values never enter CopyRequested;
- rendered truncation cannot become an action payload;
- row removal during a pointer press cannot activate a different row;
- measure/draw/hit geometry cannot disagree for wrapped Unicode rows;
- draw cannot mutate cursor, scroll or source rows;
- duplicate or missing stable keys fail reconciliation;
- pointer capture ends on release outside the list.

## Rejected use

Forbidden: painting padded property rows and copying the painted text instead of using PropsList.

```rust
// Forbidden: preview owns property rendering and copies display text.
ui.paint_str(row, 0, "› Token:  sk-****         ");
if copy_key { clipboard.write(painted_text); }
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

Use `PropsList::new` with stable row keys and handle the typed `PropsAction::CopyRequested` with source data.

## Known gaps

- Capture plan W35 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
