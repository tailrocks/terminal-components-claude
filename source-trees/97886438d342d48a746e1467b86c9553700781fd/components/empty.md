# Empty

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W36 · Group: Feedback · Phase: P2.

## Purpose and exclusions

Empty presents a caller-controlled readiness state inside a component surface. Empty, loading, partial and error are distinct states. An optional retry or action affordance is a real Button child with a typed action.

Exclusions:

- Fetching, retrying, polling or interpreting domain errors.
- Replacing partial data with a blank screen.
- Treating an error, loading state or empty result as the same boolean.
- Painting action-looking text without a focusable Button contract.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
Empty::new(id: Id, state: Readiness<'a>) -> Empty<'a>

Empty::update(&self, cx: &mut Cx<'_>) -> Response<EmptyAction>
Empty::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
Empty::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

Empty::title(&'a str)
    .detail(&'a str)
    .action(Option<ActionMeta<'a>>)
    .patch(StylePatch);

EmptyAction::Invoke {
    action: ActionKey,
    origin: ActivationOrigin,
}
```

Readiness is borrowed and controlled. Empty stores no durable readiness state. A message-only state has no focus or hit region. With action, the child Button owns focus, hover, pointer capture and activation feedback; Empty forwards its typed action without performing the domain operation.

The frozen source currently exposes EmptyState with empty/error title and hint variants. The target Readiness model makes loading, partial and retry eligibility explicit; each new state is verified as an extension where the baseline has no equivalent.

## Ordinary use

Existing consumers include the Showcase overview and frozen Jackin compositions; their baseline output remains protected.

The same readiness presentation may be composed by List, Tree, Grid, Picker and text viewport consumers while inheriting the owner surface and part patches. The caller supplies borrowed readiness, title, detail and retry metadata. Empty forwards the child Button action and runs no domain operation.

## Ownership

| Concern | Owner |
| --- | --- |
| Readiness, title, detail and retry metadata | Caller, borrowed |
| Durable focus and Button interaction | Runtime plus child Button state |
| Layout and wrapping | measure, shared layout and text foundations |
| Status styling | Theme and explicit part patches |
| Retry/network/domain work | Application |

update delegates eligible events to the optional Button. draw paints the status and child without semantic mutation. measure includes detail wrapping and the Button only when present. Partial readiness leaves existing rows visible and adds a status/sentinel through the owning collection.

Shared dependencies: [`identity`](../foundations/identity.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`layers`](../foundations/layers.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Parts are container, icon, title, detail and action. Preserve the frozen quiet centered title/detail treatment, error marker and baseline spacing. Empty does not infer a new illustration or product-specific copy. Part patches inherit the owner surface and affect actual cells.

Use [theme](../foundations/theme.md) semantic tones for empty, loading, partial and error. Text measurement respects grapheme boundaries and narrow wrapping.

Ordinary example: `Empty::new(id, Readiness::Empty).title("No results")`.

## Behavior

- update delegates eligible events to the optional Button child.
- draw paints the status and child without semantic mutation. Zero-area drawing is a no-op; no unsigned subtraction may overflow.
- measure includes detail wrapping and the Button only when present.
- Message-only Empty has no focus or hit region. With action, the child Button owns focus, hover, pointer capture and activation feedback.
- Partial readiness leaves existing rows visible and adds a status/sentinel through the owning collection.
- There is no component-owned motion. A loading spinner, if composed, receives its explicit animation sample from the caller/runtime.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Readiness | empty, loading, partial with real rows, error; retry absent/present |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, narrow detail, narrow then wide |
| Focus/hover | none for message-only; child Button focused/hovered/focus+hover when action exists |
| Activation | pointer down, held, release inside/outside, removed/disabled action, keyboard activation and feedback for child Button |
| Resize | reflow detail and preserve caller/runtime state |
| Color/capability | semantic fallback across all capture capabilities |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen widget source is frozen-at-4a79c0a2 ([empty.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/empty.rs)). Use [`../reference/capture-plans/empty.json`](../reference/capture-plans/empty.json), the [Button contract](./button.md), the [visual contract](../design/visual-contract.md) and [interaction parity](../verification/interaction-parity.md); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W36-01 | Empty/loading/error with and without retry |
| W36-02 | Partial-data sentinel does not erase real rows |
| W36-03 | Owner theme/part override reaches empty-state cells |
| W36-04 | Narrow detail wrapping and defined zero-area behavior |

Record exact cells/styles, cursor visibility, focus/capture owner, typed action count/target, readiness value and child state. Extension cases must be marked as such when the frozen source has no corresponding state.

Required negative tests:

- partial state cannot discard or hide real rows;
- message-only Empty cannot register focus/hit/capture;
- an action-looking label without a Button child cannot emit an action;
- retry action cannot run during draw;
- zero/tiny rectangles cannot panic or write outside area;
- theme/part patches cannot silently be ignored or recolor unrelated cells.

## Rejected use

Forbidden: painting status text with a fake retry label instead of composing Empty with a Button child.

```rust
// Forbidden: preview paints its own status and fake action.
ui.paint_str(row, 0, "No results found   [ Retry ]");
if clicked { retry_now(); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls" and "padded strings that simulate columns, selection, fields, menus, or buttons". Use `Empty::new` with a real Button child and handle the typed `EmptyAction`.

## Known gaps

- Capture plan W36 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
