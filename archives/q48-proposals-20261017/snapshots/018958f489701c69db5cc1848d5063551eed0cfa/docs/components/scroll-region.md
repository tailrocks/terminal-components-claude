# ScrollRegion

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-viewport.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W45 · Phase: P2 · Legacy family: C08.

## Purpose and exclusions

ScrollRegion is the one shared scroll model, scrollbar painter, thumb capture policy, and edge-fade policy used by Termrock list, tree, grid, output, picker, props, viewport, editor, and diff consumers.

It maps content extent and viewport extent to a bounded offset, visible range, scrollbar track/thumb, wheel routing, thumb drag, follow/anchor policy, and semantic edge fade. The parent component remains the focus owner unless its own contract explicitly says otherwise.

ScrollRegion must not store list/grid/output data; claim focus for a decorative scrollbar; change domain selection or navigation at a wheel boundary; mutate durable state from draw geometry; or allow a second scroll engine for ScrollPanel, TextViewport, or Grid. All consumers use this mechanism; component-local scrollbars, thumb math, or fade engines are prohibited.

## Public API

The signatures below are the proposed target, not a claim about current Rust exports. No signature is source-checked.

```rust
ScrollRegion::new(owner: Id, axis: Axis, content: Extent) -> ScrollRegion<'a>

update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut ScrollState,
) -> Response<ScrollAction>

draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &ScrollState,
) -> Rect

measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

Configuration:

```rust
bars(ScrollbarPolicy)
fade(FadePolicy)
protected(&'a [ProtectedRange])
```

`ScrollState` owns only bounded offset and follow/anchor policy. Runtime-owned layout facts contain track, thumb, viewport, and hit geometry. Thumb drag capture is never duplicated in parent components. Fields remain private; expose read-only keyed cursor/navigation/scroll observations and invariant-preserving commands only.

Typed actions:

```rust
ScrollAction::OffsetChanged
ScrollAction::FollowChanged(bool)
```

## Ordinary use

No consumer recipe owns ScrollRegion directly; it is the shared mechanism behind scrolled components such as [EX-11 — A log viewport, not a repainted text block](../api/consumer-recipes.md) (`proposed_target`).

Real preview consumers include the Showcase scrolling pages; all four preserved applications exercise shared scroll composition. An independent synthetic fixture supplies a content extent longer than the viewport, one protected row, and a semantic cursor fact.

The caller supplies content extent, protected rows, and any semantic cursor or selection facts. The parent component owns domain rows and focus; ScrollRegion owns only bounded offset and follow/anchor policy. At least two parent surfaces must consume this one mechanism.

## Ownership

- The caller supplies content extent, protected rows, and any semantic cursor or selection facts. ScrollRegion does not own domain rows, text, focus rings, or child state.
- `update` routes wheel, key paging, track press, thumb drag, and release through the topmost eligible scroll owner under the pointer. It preserves parent focus, uses runtime capture, clamps offsets, and emits typed changes only when the offset/follow state changes. At a boundary it consumes according to the frozen routing policy instead of chaining into an unrelated pane.
- `draw` paints the track and thumb only when content overflows, registers the same rectangle for hit testing, and applies semantic edge fade after content rows but before the scrollbar. It may publish fresh track/visible-range facts; it cannot silently alter durable offset, follow, anchor, or parent focus.
- `measure` calculates viewport/content extent and visible-range facts before labels or metadata consume them. Resize clamps offset to the valid range while preserving the semantic anchor where possible.
- Runtime owns focus, hit, capture, hover suppression, cursor, and time. The parent/child focus owner is unchanged by ordinary wheel or scrollbar use.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), and [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `viewport`, `track`, `thumb`, `start-fade`, and `end-fade`.

Ordinary customization selects scrollbar and fade policy through `bars` and `fade` configuration. Apply the baseline edge fade only where hidden content exists. At the top only the bottom edge fades; at the bottom only the top edge fades; in the middle both edges fade. Short viewports below the minimum row threshold do not fade. Tall viewports use the two-row depth threshold. The source baseline uses the height boundary cases 3, 4, 11, and 12 and these remain required captures.

Advanced customization declares protected ranges that fade must preserve. Fade mixes effective foreground toward the majority container background in truecolor. 256-color, 16-color, and no-color modes use the outer-row DIM cue without inventing RGB. Never fade a selected/hovered/marked/reversed cell, the hardware cursor row, or a caller-declared protected row. Fading must preserve semantic emphasis rather than flattening all content to gray.

Track and hit geometry use the same calculation. Part patches cannot replace the content viewport, runtime capture, or parent focus ownership. Motion and fade timing use the shared time/reduced-motion policy; a static scroll region has no generic activation animation.

## Behavior

The model provides bounded `scroll_by`, `scroll_to`, page, start/end, and ensure-visible operations. No-overflow content has no scrollbar, no fade, and no scroll action. Overflow thumb length is proportional and at least one cell.

A press inside the thumb records the exact grabbed row and does not jump. A press on bare track moves the thumb under the pointer. Drag keeps the grabbed row under the pointer, reaches exact first/last offsets, and a release clears capture. A drag without a prior press follows the documented safe press path.

Wheel routes to the innermost eligible region under the pointer, including nested modal/overlay composition. Disabled regions cannot consume or capture. Editing, activation, and domain readiness belong to the parent component.

## Visual matrix

| Axis | Required states | Applies when |
|---|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | Always |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover | Parent/scrollbar interaction |
| Activation | No generic whole-surface activation; offset/follow changes are typed | Not applicable at scroll surface |
| Scroll | no overflow, start, middle, end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row | Overflow/edge policy |
| Pointer | press thumb, press track, held drag, release inside, release outside, nested eligible owner | Pointer available |
| Capability | truecolor, 256, 16, none/nocolor | Terminal advertises mode |
| Motion | active fade/press phases, paused/reduced motion | Theme policy |

## Verification

Parity is pinned to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Source evidence is the model in `src/core/scroll.rs`, the baseline scrollbar painter/hit mapping in `src/widgets/scrollbar.rs` (blob `fa0132637370da4afe2e40a07fb558c135e188df`), and the baseline edge fade in `src/ui/fade.rs`, including RGB ramp, non-RGB DIM fallback, protected/emphasised rows, short-view rules, and cursor protection. Inline tests cover bounded scrolling, visible ranges, thumb proportion, nonzero grab offset, exact end/start drag, track clicks, boundary no-ops, fade direction, height thresholds, protected rows, emphasised cells, cursor row, and capability fallback. The visual consumer is Showcase scrolling pages, with discovery root `snapshots/showcase/pages/scrolling`; all four preserved applications exercise shared scroll composition through `tests/visual_baseline`. See [`visual-parity`](../verification/visual-parity.md), [`oracle-and-provenance`](../verification/oracle-and-provenance.md), and [`public-api`](../api/public-api.md).

Use [`../reference/capture-plans/scroll-region.json`](../reference/capture-plans/scroll-region.json). It is planned with no approved expected artifacts. Bind baseline captures as `ExistingOracle`/`ExtractedOracle`; new robustness/API rules are `Extension`.

Run applicable cases at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50`, with truecolor, 256-color, 16-color, `none`, and `nocolor` capabilities.

| Case | Required observation |
|---|---|
| W45-01 | No overflow versus first overflowing row; scrollbar/fade appear exactly. |
| W45-02 | Top/middle/bottom wheel and boundary consumption. |
| W45-03 | Thumb drag with nonzero grab offset reaches exact ends. |
| W45-04 | Nested scrollables under modal and disabled regions route safely. |
| W45-05 | Fade at heights 3, 4, 11, and 12 with protected rows. |
| W45-06 | Resize/reflow retains the semantic content anchor. |

Record content/viewport lengths, offset, visible range, track/thumb cells, grab row, focus/capture/layer owners, protected rows, capabilities, modifiers, fade colors/DIM, action count and action target. Invalid oracle setup is a blocked capture.

Required negative tests:

- No-overflow content cannot paint or hit-test a scrollbar/thumb/fade.
- Track paint and hit regions cannot diverge.
- Thumb drag preserves grab offset and reaches exact first/last offsets.
- Wheel at a boundary cannot mutate an unrelated pane or focus owner.
- Nested modal/disabled routing cannot leave stale capture.
- Resize cannot produce out-of-range offsets or lose a valid semantic anchor.
- Fade cannot alter selected, cursor, marked, reversed, or protected rows.
- RGB blending cannot appear in non-RGB capability modes.
- `draw` cannot change durable offset/follow/anchor or domain state.
- Component consumers cannot reimplement thumb, fade, or capture math.

Acceptance requires an external consumer for at least two parent surfaces, exact snapshots/traces, source-state tests, and independent review confirming that all consumers use this one mechanism.

## Rejected use

Forbidden: painting a manual scrollbar track and computing a thumb offset instead of using ScrollRegion.

```rust
// Forbidden: preview owns scrollbar rendering and thumb math.
for y in 0..height { ui.paint_str(y, w - 1, "│"); }
let thumb = offset * height / content_len;
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

Use the shared `ScrollRegion` mechanism and handle the typed `ScrollAction`.

## Known gaps

- Capture plan W45 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-005 audit and repair TablePro, Holla, and Showcase](../implementation/code-remediation-backlog.md): pending.
- [FIX-006 missing reusable content/layout API](../implementation/code-remediation-backlog.md): pending.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
