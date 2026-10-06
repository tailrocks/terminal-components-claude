# ChipBar

Status: proposed target; implementation is future work on `termrock-implementation`.
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

Proposed target. No part of this API is source-checked against current code.

```rust
ChipBar::new(id: Id, chips: &'a [ChipItem<'a>], revision: Revision) -> ChipBar<'a>

bar.update(&mut cx, &mut state) -> Response<ChipAction>
bar.draw(&mut ui, area, &state) -> Rect
bar.measure(&measure_cx, constraints) -> Size
```

Builders: `add_action(Option<ActionMeta<'a>>)`, `disabled(bool)`, `patch(StylePatch)`, and `row(&ChipRowPainter)`.

`ChipBarState` contains cursor key and horizontal `ScrollState`; checked state remains controlled in chip rows. Typed actions are `ChipAction::{ Activate { key, origin }, SetChecked { key, checked, origin }, Close { key, origin }, Add { origin } }`. Optional lead/clear affordances are caller action metadata routed through these stable action identities; they do not create another event engine or an untyped product callback.

Update, draw, and measure:

- `update` navigates stable chip keys, routes each subpart to its distinct action, and reconciles cursor/scroll after source changes. Close never also activates the body.
- `draw` paints lead, chips, close affordances, overflow, and Add as actual subparts with clipped hitboxes. It never mutates chip data or starts scrolling.
- `measure` computes display-cell widths for each chip, close glyph, gaps, lead/Add affordance, and overflow marker.

## Ordinary use

No consumer recipe covers ChipBar yet.

Showcase chips page is the primary baseline consumer.

Ordinary use passes borrowed `chips`, `revision`, and optional `add_action`. The caller keeps `ChipBarState` (cursor key and horizontal scroll) across frames and handles the typed `ChipAction` for body activation, checked changes, close, and add. Checked state stays controlled in chip rows; the bar never mutates it.

## Ownership

The bar `Id`, chip key, and subpart role (`body`, `close`, `add`, `lead`) form stable semantic identities. Reorder/removal preserves a surviving cursor or resolves to a deterministic neighbor. A pointer press stores the subpart key and release validates current membership/eligibility. Horizontal scroll is reconciled from keys, never stale indices.

Shared dependencies:

- [`identity`](../foundations/identity.md), [`collections`](../foundations/collections.md) — keyed chip/source reconciliation.
- [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md) — typed subpart actions, focus, hover, and capture.
- [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md) — cell geometry and semantic style.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — constrained row parts and oracle proof.
- [`ScrollRegion`](scroll-region.md) — shared horizontal scrolling and fade policy.

## Customization

Parts are `container`, `gutter`, `marker`, `label`, `close`, `add`, and `overflow`. Preserve baseline strip density, toggle/check marker, hover lifting, close glyph, Add affordance, lead label, gaps, overflow ellipsis, disabled/error tones, and clipping. [`ScrollRegion`](scroll-region.md) and [`theme`](../foundations/theme.md) own shared scroll/style policy. A row painter may style supported parts but cannot replace subpart hit ownership.

The baseline anatomy is an optional lead such as `match all ▾`, then chips
rendered as `▎label ×`, a subtle `+ Add filter` stop, and one `…` overflow
marker when the strip cannot show every chip. Disabled chips are faint and
invalid chips use the error tone. The whole bar is one focus stop with a
logical cursor among lead, chip body, chip close, and Add subparts.

Ordinary example: `ChipBar::new(id, &chips, rev).add_action(Some(add_meta))`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Left/Right or `h`/`l` move the logical cursor. Enter edits the chip or activates Add. Space toggles. `x`/Delete/Backspace removes the current closable chip; `+` adds and uppercase `X` clears all only when configured. Modified unassigned chords do nothing. |
| Pointer | Body, close, lead, and Add have disjoint clipped targets. Close emits only Close; body emits only Activate; outside/removal cancels. |
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

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate selection. Use `ChipBar::new` with stable chip keys and handle the typed `ChipAction::Close`.

## Known gaps

- Capture plan W06 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
