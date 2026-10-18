# StatusBar

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-feedback.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W40 · Group: Chrome · Phase: P5.

## Purpose and exclusions

StatusBar paints one full-width status row from caller-owned left, center and right groups. It carries labels, values, priorities, tones and optional action metadata. It is the single future engine for the old StatusBar and Segments surfaces.

Exclusions:

- Service polling, account knowledge or provider interpretation.
- A second Segments painter.
- Focus stops for noninteractive metadata.
- Synthesizing success from missing or stale values.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
StatusBar::new(
    id: Id,
    groups: &'a [StatusGroup<'a>],
    revision: Revision,
) -> StatusBar<'a>

StatusBar::update(&self, cx: &mut Cx<'_>) -> Response<StatusAction>
StatusBar::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
StatusBar::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

StatusBar::animation(AnimationSample)
    .patch(StylePatch)
    .patch_part(Part, StylePatch);

StatusAction::Invoke {
    key: ItemKey,
    action: ActionKey,
    origin: ActivationOrigin,
}
```

Groups and items are borrowed. Every interactive item has a stable ItemKey; noninteractive items are not focus stops. Runtime owns keyed hover/press and feedback state; StatusBar stores no value state.

## Ordinary use

Existing consumers include Showcase chrome and the Jackin capsule; their baseline output remains protected.

The caller supplies borrowed groups, a source revision and the animation sample, then handles the typed `StatusAction`. update resolves runtime events to stable item keys and returns typed actions. draw is pure with respect to the borrowed groups and runtime state.

## Ownership

| Concern | Owner |
| --- | --- |
| Group contents, labels, values, priorities and action metadata | Caller |
| Stable item identity and reconciliation | Identity/collections foundations |
| Focus, hit testing, hover, press and pointer capture | Runtime |
| Priority fit and left/center/right geometry | StatusBar |
| Spinner phase | Runtime/caller AnimationSample |
| Service/account state | Application |

measure and draw share truncation, priority-drop and gap calculations.

Shared dependencies: [`identity`](../foundations/identity.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`input/actions`](../foundations/input-actions.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Parts are container, group, item, label, value, icon and separator. Preserve the frozen row's group spacing and tones. There are no separator glyphs between groups; spacing carries the separation. The baseline uses a three-cell gap and one-cell edge inset.

At narrow widths, drop the lowest-priority surviving item in the baseline order: center first, then right, then left; the strongest left item survives and truncates if necessary. A spinner changes only its declared cells and cannot change item width unpredictably.

Unicode labels/readouts and capability fallback use [text](../foundations/text.md) and [theme](../foundations/theme.md).

Ordinary example: `StatusBar::new(id, &groups, revision)`.

## Behavior

- Hover lifting, press feedback, focus gutter and pointer capture apply only to interactive items. Noninteractive items are not focus stops.
- Keyboard-suppressed hover and release outside follow the [runtime](../foundations/runtime.md) contract.
- Removal during capture cannot activate a replacement item; duplicate/missing item keys fail reconciliation.
- Priority dropping cannot change a surviving item's stable geometry unexpectedly.
- draw cannot poll services, mutate groups or advance animation.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, Unicode content, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | pointer down, held, release inside/outside, removed target, disabled item, keyboard activation, feedback timing |
| Source | empty group, dynamic update, reorder/removal, stale item key |
| Motion | all unique spinner phases, phase boundary, paused/reduced, stable width |
| Color/capability | semantic item tones across all capture capabilities |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen sources are frozen-at-4a79c0a2 ([statusbar.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/statusbar.rs) and [segments.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/segments.rs)). The second source is a legacy family disposition: Segments is absorbed into StatusBar. It must not remain a competing rendering engine. Use [`../reference/capture-plans/status-bar.json`](../reference/capture-plans/status-bar.json); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W40-01 | Left/center/right alignment at exact fit |
| W40-02 | Narrow width drops items in declared priority order |
| W40-03 | Interactive item hover/click versus noninteractive item |
| W40-04 | Spinner frames preserve gaps and stable width |
| W40-05 | Dynamic removal during pointer press cannot retarget |

Record exact cells/styles, selected stable item key, navigation key, cursor, focus owner, capture owner, action target/count, revision and animation sample.

Required negative tests:

- Segments cannot create a parallel painter or conflicting fit order;
- a noninteractive item cannot receive focus or emit an action;
- removal during capture cannot activate a replacement item;
- duplicate/missing item keys fail reconciliation;
- priority dropping cannot change a surviving item's stable geometry unexpectedly;
- draw cannot poll services, mutate groups or advance animation.

## Rejected use

Forbidden: painting a status row with a parallel segments painter instead of using the single StatusBar engine.

```rust
// Forbidden: preview keeps its own segments painter.
ui.paint_str(row, 0, "main   ✓ synced   38%");
segments_paint(row, &extra);
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls". Use `StatusBar::new` with borrowed groups and stable item keys; Segments is absorbed, not a second painter.

## Known gaps

- Capture plan W40 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
