# ChipBar

Status: implemented on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W06 · Group: Collections · Phase: P3.

## Purpose and exclusions

`ChipBar` renders a horizontal keyed strip of chips. The caller supplies labels, checked/disabled/closable flags, and action metadata. Durable state stores only cursor and horizontal scroll. Termrock reports body activation, checked changes, close, and add actions; it does not parse product filters or own tag persistence.

Exclusions:

- No filter-query parser, product tag model, persistence, or application-specific action semantics.
- No conflation of chip body, close, leading, and Add hit regions.
- No independent horizontal scrolling or focus engine.

## Public API

Source-checked against `crates/termrock-navigation/src/chip.rs`.

```rust
ChipBar::new(id: Id) -> ChipBar<'_, T, ByIndex, DefaultRow>

bar.key(K2: Fn(&T) -> ItemKey) -> ChipBar<'_, T, K2, R>
bar.row(R2: Fn(&T, &mut RowUi)) -> ChipBar<'_, T, K, R2>
bar.select_mode(SelectMode) -> Self   // default Multi
bar.closable(bool) -> Self           // default false
bar.lead(&str) -> Self               // default none
bar.add(&str) -> Self                // default none
bar.plus_add(bool) -> Self           // default false
bar.clear_all(bool) -> Self          // default false
bar.read_only(bool) -> Self
bar.disabled(bool) -> Self
bar.patch(&StylePatch) -> Self
bar.patch_part(&[(Part, StylePatch)]) -> Self
bar.slot(Part, SlotFn) -> Self       // CLOSE and OVERFLOW only

bar.update(&self, cx: &mut Cx, st: &mut ChipBarState, items: &[T]) -> Response<ChipBarAction>
bar.draw(&self, ui: &mut Ui, area: Rect, st: &ChipBarState, items: &[T]) -> Rect
bar.measure(&self, ui: &Ui, c: Constraints) -> Size
```

`ChipBarState` carries the cursor key, the checked set, the add-stop flag, and the keyed window head (`Clone + Default`). Typed actions are `ChipBarAction::{ Toggled(ItemKey), Closed(ItemKey), Activated(ItemKey), AddRequested, Lead, Cleared }`; the lead, add, and clear-all affordances name no item and report payloadless actions, so none can collide with an item key.

Update, draw, and measure:

- `update` reconciles cursor/checked/window over stable keys, then moves the cursor, activates, toggles, closes, or fires the gated `+`/`X` chords. Close never also activates the body.
- `draw` paints lead, chips, close affordances, overflow, and Add as actual subparts with clipped hitboxes. It never mutates chip data or starts scrolling.
- `measure` reports one row: minimum 8 columns, preferred strip width.

## Ordinary use

Showcase chips page is the primary baseline consumer.

Ordinary use passes borrowed `items` to each phase (chips are never held) plus a caller-kept `ChipBarState` across frames, and handles the typed `ChipBarAction` for body activation, checked changes, close, add, lead, and clear-all. The component owns the checked set in its state; the caller owns the items. Update and draw must build the bar identically (same id, key, and row), or pointer actions misroute.

## Ownership

The bar `Id`, chip key, and subpart role (`body`, `close`, `add`, `lead`) form stable semantic identities. Reorder/removal preserves a surviving cursor or resolves to a deterministic neighbor. A pointer press stores the subpart key and release validates current membership/eligibility. Horizontal scroll is reconciled from keys, never stale indices.

Shared dependencies:

- [`identity`](../foundations/identity.md), [`collections`](../foundations/collections.md) — keyed chip/source reconciliation.
- [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md) — typed subpart actions, focus, hover, and capture.
- [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md) — cell geometry and semantic style.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — constrained row parts and oracle proof.
- [`ScrollRegion`](scroll-region.md) — shared horizontal scrolling and fade policy.

## Customization

Parts are `CONTAINER`, `MARKER`, `LABEL`, `CLOSE`, `OVERFLOW`, `NEW`, and `LEAD`. Preserve baseline strip density, toggle/check marker, hover lifting, close glyph, Add affordance, lead label, gaps, overflow ellipsis, disabled/error tones, and clipping. [`ScrollRegion`](scroll-region.md) and [`theme`](../foundations/theme.md) own shared scroll/style policy. A row painter may style supported parts but cannot replace subpart hit ownership.

The baseline anatomy is an optional lead such as `match all ▾`, then chips
rendered as `▎ label ␣ × ␣␣` when closable (gutter | label | pad | × |
pad | pad — the close zone is `×` plus two pads, `×` at right-3, so an
18-cell label spans exactly 23 cells per the oracle `w = 1 + label_w + 1
+ 2 + 1`, `src/widgets/chips.rs:194-195`) and `▎ label ␣` when not
(non-closable width is unchanged; whether it also gains a trailing cell
is deferred to the holla-activity adoption slice, whose chips are
removable=false). A subtle `+ Add filter` stop follows, and one `…`
overflow marker when the strip cannot show every chip. Disabled chips are
faint and invalid chips use the error tone. The whole bar is one focus
stop; the logical cursor moves among chips and the Add stop, while the
lead is click-only and never a cursor stop (the oracle wins over the
earlier cursor-among-lead clause).

Ordinary example: `ChipBar::new(id).closable(true).lead("match all ▾").add("+ Add filter")`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Left/Right or `h`/`l` move the logical cursor. Enter edits the chip or activates Add. Space toggles. `x`/Delete/Backspace removes the current closable chip; `+` adds and uppercase `X` clears all only when configured. Modified unassigned chords do nothing. |
| Pointer | Body, close, lead, and Add have disjoint clipped targets. Close emits only Close (the label registers before the close cell; hit-testing is last-registration-wins, `crates/termrock-runtime/src/hit.rs:288`); body emits only Activate; outside/removal cancels. |
| Focus/hover | One bar focus stop; runtime owns hover/capture. Focused cursor and hovered close are distinct visual states. Keyboard suppresses stale hover. |
| Disabled/read-only | Disabled chip body and close eligibility follow the caller's flags; disabled targets cannot emit action or retain press. |
| Editing | Not applicable; labels are borrowed text. |
| Scroll/resize | Shared horizontal scroll, exact-fit/one-cell-short overflow, and deterministic cursor after source changes. |
| Unicode | Long, combining, and wide labels are measured/truncated by display cells; no write crosses the strip. |
| Capability/color | Semantic toggle, muted, error, disabled, hover, and surface styles map across capabilities. |
| Motion | Shared action feedback only; no separate animation loop. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | pointer down, held press, release inside/outside, removed target, disabled target, keyboard activation, feedback active/expired when applicable |
| Source | empty, ready, checked differs from cursor, disabled entry, reorder, insert, remove, filter, stale target |
| Scroll | no overflow, start, middle, end, wheel boundary, thumb drag, resize while scrolled, edge fade/protected row where applicable |

Run at 72×20, 80×24, 100×30, 120×40, and 160×50 under truecolor, 256, 16, none, and nocolor. Record exact subpart cells, focus/capture owner, cursor/scroll keys, action count/target, and source revision.

## Verification

The frozen oracle is [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Current source is [`src/widgets/chips.rs`](../../src/widgets/chips.rs), legacy `ChipBar`/`Chip`, source blob `36cb21bf1be548c39544cef2916669a0a4f55b30`.

The W06 capture plan is [`capture-plans/chip-bar.json`](../reference/capture-plans/chip-bar.json), planned and uncaptured; bind source output before candidates.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W06-01 | Checked/unselected and checked/focused chips. |
| W06-02 | Close-button press followed by reorder/delete. |
| W06-03 | Overflow at exact fit, one cell short, and long Unicode label widths. |
| W06-04 | Empty strip with and without Add. |
| W06-05 | Disabled chip body and close eligibility. |

Negative tests:

- Close click cannot also activate or toggle the body.
- Removed/reordered keys cannot receive a stale press or close action.
- Disabled chip body/close cannot capture or emit.
- Exact-fit and overflow layouts stay inside the allocation.
- Empty bars cannot index a chip; Add only exists when configured.
- Draw cannot mutate rows, checked state, cursor, or scroll semantics.

Accept after external API, source/reconciliation, exact snapshots, subpart hit tests, Unicode/clipping, and independent review pass.

## Rejected use

Forbidden: painting chip labels with close glyphs and removing tags on click instead of using ChipBar.

```rust
// Forbidden: preview owns chip rendering and removal.
ui.paint_str(row, 0, "▎rust × ▎tui ×");
if clicked { tags.remove(i); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate selection. Use `ChipBar::new` with stable chip keys and handle the typed `ChipBarAction::Closed`.

## Known gaps

- Capture plan W06 is planned and uncaptured; no expected artifacts are bound.
