# FilterList

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W13 · Group: Collections · Phase: P4.

## Purpose and exclusions

Canonical Termrock contract for W13. `FilterList` is a keyed projection over caller-owned rows. It shares the list, text-input and scroll mechanisms; it is not a second data or query engine.

The component owns a keyed index projection and query draft, not cloned domain objects. It borrows source rows, applies the shared Unicode-safe matcher by default or a caller-supplied side-effect-free filter/ranker, and returns query/selection actions. The caller remains the committed query authority and owns data loading, ranking policy and activation effects.

## Public API

```rust
FilterList::new(id: Id, rows: &'a [T], revision: Revision) -> FilterList<'a, T>
where
    T: Keyed;

FilterList::query(&'a str)
    .filter(&'a dyn Fn(&T, &str) -> MatchResult)
    .row(&'a RowPainter<T>)
    .readiness(Readiness<'a>)
    .patch(StylePatch);

FilterList::update(&self, cx: &mut Cx<'_>, state: &mut FilterListState)
    -> Response<FilterListAction>;
FilterList::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &FilterListState) -> Rect;
FilterList::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`FilterListState` owns query draft, projected keyed cursor and the shared `ListState` scroll state. Source rows remain borrowed. Read-only access may expose `query()`, current stable result key and scroll observations; no mutable access to caller-owned results. Typed actions are `QueryChanged`, `Activate { key, origin }` and `SelectionRequested(SelectionRequest)`.

## Ordinary use

Showcase picker scenarios and the picker consumers in Jackin Preview and Holla exercise the filtering/query behavior.

The caller supplies borrowed rows, a source revision, and the committed query, then applies the typed query/selection actions.

## Ownership

- Query edits are emitted as actions. The caller may recompute rows immediately or asynchronously and returns a source revision; the component never commits domain data on its own.
- Default matching is Unicode-safe and reports match spans in source graphemes, not byte offsets treated as display columns. Custom matching must be deterministic and side-effect-free.
- Stable row keys survive source reorder, insert, removal and query changes. A current result that becomes ineligible is a stale target; activation is refused rather than rebound to a row now occupying its old index.
- `empty source`, `no matches`, `loading`, `partial` and `error` are distinct readiness presentations. Highlighted but unavailable rows never activate.
- `update` owns all semantic changes. `draw` is read-only and only registers current visible result hit regions. `measure` has no semantic or runtime side effects.
- Runtime owns focus, hover, pointer capture, layer placement and time. Query draft, projected cursor and scroll belong to `FilterListState`.

Shared dependencies: [`list`](./list.md), [`text`](../foundations/text.md), [`input-actions`](../foundations/input-actions.md), [`identity`](../foundations/identity.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `query`, `results`, `row`, `match`, `empty`, `scrollbar` and `fade`. Resolve styles through [`theme`](../foundations/theme.md); a patch may replace only declared parts. Preserve the baseline query field, highlighted match spans, row marker/gutter, text hierarchy, clipping, empty message and scroll fades. Match highlighting must follow grapheme boundaries and terminal cell width, including combining marks and CJK. Never let fades obscure a protected cursor/result row.

## Behavior

- Query editing has the shared text-editing semantics: printable input updates the draft, Backspace removes one grapheme, word deletion uses the shared word boundary, and paste is one query edit. Focus/caret belongs to the query field while editing.
- Up/Down, PageUp/PageDown, Home/End and the component's baseline navigation bindings move among eligible projected keys. Query changes reconcile the cursor deterministically.
- Enter/activation resolves the current result's stable key only when the source is ready and the row is enabled. Disabled, unavailable and stale results consume the gesture without an action.
- Pointer row activation requires a completed inside release. Release outside, source revision changes or target removal cancel the pending gesture. Hover is presentation only; keyboard activity suppresses stale hover until pointer movement.
- Wheel and scrollbar drag affect the result viewport under the pointer, preserve query/cursor identity and do not transfer focus. Boundary wheel events are consumed.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, long Unicode content, narrow-to-wide resize |
| Focus/hover | unfocused, focused query/results, hovered row, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | not a generic whole-surface action; child row down/held/release inside/outside, removed/disabled target and keyboard activation are covered |
| Source/readiness | empty source, ready matches, zero matches, selected differs from cursor, disabled entry, reorder/insert/remove, query filter, stale target, loading, partial, error, offered retry/refresh |
| Scroll | no overflow, start/middle/end, boundary wheel, thumb drag, resize while scrolled, edge fade and protected row |

Only applicable combinations are required. Use the shared fixture dimensions and color/capability matrix in [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). There is no separate filter widget in the frozen source (legacy family C23). Its oracle is the filtering/query behavior in [`src/widgets/picker.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/picker.rs). Current source names such as `Picker`, `WidgetId` and `Outcome` are legacy implementation evidence, not future API requirements.

Use [`../reference/capture-plans/filter-list.json`](../reference/capture-plans/filter-list.json) and [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md) to classify `ExistingOracle`, `ExtractedOracle` and `Extension` evidence before candidate testing. Record dimensions, cells, cursor, query draft/committed state, focus/capture owner and typed action count/target. A missing or invalid oracle is a blocked capture, never an expected result.

| Case | Required observation |
|---|---|
| W13-01 | Empty source differs from non-empty source with zero matches. |
| W13-02 | Paste Unicode query and Backspace exactly one grapheme. |
| W13-03 | Source revision while filtered retains the selected stable key when it survives. |
| W13-04 | Query change makes the current row ineligible; no stale activation occurs. |
| W13-05 | Match spans across combining characters and CJK use correct cells and modifiers. |

Negative tests must prove: no source-row cloning; no byte/column confusion; duplicate keys are rejected; stale or disabled results never activate; async/source revisions cannot retarget a gesture; query edits cannot mutate committed caller state; draw cannot change query, cursor or scroll; wheel cannot transfer focus; hidden rows cannot be hit; custom matchers cannot mutate source; and match/part styling reaches the expected cells. FilterList must reuse List and the shared text core rather than creating parallel implementations.

## Rejected use

Forbidden: filtering rows in preview code and painting match rows instead of using FilterList.

```rust
// Forbidden: preview owns filtering and paints its own rows.
let hits: Vec<_> = rows.iter().filter(|r| r.label.contains(query)).collect();
ui.paint_str(row, 0, hits[0].label);
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate selection. Use `FilterList::new` with stable keys and handle the typed `FilterListAction`.

## Known gaps

- Capture plan W13 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
