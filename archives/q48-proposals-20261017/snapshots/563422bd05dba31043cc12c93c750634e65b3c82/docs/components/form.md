# Form

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-forms.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W11 · Group: Forms · Phase: P3.

## Purpose and exclusions

`Form` composes borrowed field declarations, child controls, validation, and action metadata. The caller binds live values and child state. Form owns only submission-attempt and invalid-field focus bookkeeping. It reports intent through typed actions; it does not save, persist, authorize, or perform product work.

Exclusions:

- No untyped value map, schema engine, persistence, or domain model.
- No Jackin service, account, provider, Docker, Git, Holla, or TablePro implementation.
- No decorative child painting in place of real controls.
- No hidden field focus or automatic reset of sibling drafts.

## Public API

Proposed target. No part of this API is source-checked against current code.

```rust
Form::new(
    id: Id,
    fields: &'a [FieldSpec<'a>],
    actions: &'a [ActionMeta<'a>],
) -> Form<'a>

form.update(
    &mut cx,
    &mut state,
    &mut controls,
) -> Response<FormAction>
form.draw(
    &mut ui,
    area,
    &state,
    &controls,
) -> Rect
form.measure(&measure_cx, constraints) -> Size
```

Builders: `validation(&'a [FieldError])`, `dirty(bool)`, `busy(bool)`, and `patch(StylePatch)`.

`FormState` stores submission attempt and invalid-field focus bookkeeping only. Child control state and caller values live in `FormControls`. Typed actions are `FormAction::{ Submit, Cancel, Auxiliary { action: ActionKey } }`. Expose read-only keyed cursor/navigation/scroll observations with invariant-preserving commands; fields remain private, and runtime geometry/domain rows are not public mutable state.

Update, draw, and measure:

- `update` traverses visible enabled declarations in paint order, forwards child responses, commits the focused draft before submit, validates current values, and emits Submit only if valid. Invalid submission focuses the first invalid visible field. Busy submission is blocked; Cancel follows explicit caller eligibility.
- `draw` paints real child controls through Field/child contracts, validation summary, and action controls. It must not mutate values, commit drafts, save, or reset sibling state.
- `measure` accounts for field groups, validation summary, action row, and constrained child sizes. Hidden fields consume no geometry or hit region; resize reconciles focus deterministically.

## Ordinary use

Consumer recipe: [EX-04 — A form with validation and protected input](../api/consumer-recipes.md) (`proposed_target`).

Current application evidence is in [`src/bin/jackin_preview/screens/modals.rs`](../../src/bin/jackin_preview/screens/modals.rs) and [`src/bin/jackin_preview/screens/editor.rs`](../../src/bin/jackin_preview/screens/editor.rs), with Showcase form snapshots as a second protected consumer. These files remain product fixtures and are not implementation targets.

Ordinary use passes borrowed `fields` declarations and `actions` metadata. The caller binds live values and child state in `FormControls` and routes `FormAction` (Submit, Cancel, Auxiliary). Submit commits the focused draft in update, validates, and emits only when valid; invalid submission focuses the first invalid visible field.

## Ownership

Each declaration has a stable `FieldKey` and child `Id`. Visibility/disabled changes reconcile by key: hidden fields lose focus/hit ownership; disabled fields remain visible as applicable but cannot edit. A nested picker layer returns or cancels to the exact parent draft and focus; sibling drafts survive. Reordering declarations does not transfer a draft or pending action by index.

Shared dependencies:

- [`identity`](../foundations/identity.md), [`collections`](../foundations/collections.md) — stable FieldKey/child identity and declaration reconciliation.
- [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md) — traversal, typed actions, focus, hit, capture, and time.
- [`layers`](../foundations/layers.md) — nested picker ownership and restoration.
- [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md) — composition geometry and semantic styles.
- [`secret-validation`](../foundations/secret-validation.md) — validation and redaction rules for child values.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — controlled composition and oracle proof.
- [`Field`](field.md), [`TextInput`](text-input.md), [`Button`](button.md), and [`Select`](select.md) — child contracts and intentional behavior.

## Customization

Parts are `container`, `field-group`, `validation-summary`, and `actions`. Preserve existing form row spacing, Field chrome, error/required treatment, action layout, busy/disabled styling, focus gutter, modal surfaces, and clipping. [`Field`](field.md), [`TextInput`](text-input.md), and [`Button`](button.md) own child visual mechanisms; Form composes them without duplicate renderers. [`theme`](../foundations/theme.md) owns semantic style precedence.

Ordinary example: `Form::new(id, &fields, &actions).validation(&errors).busy(false)`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard/focus | Traversal visits visible enabled fields in paint order, exactly once each. Hidden fields have no focus/hit region; disabled fields cannot receive editing input. |
| Submit | Commit focused draft in update, validate values, emit Submit only when valid. On invalid result, focus first invalid visible field deterministically. |
| Pointer | Real child controls receive pointer events. Action buttons have disjoint targets and typed action metadata. |
| Nested layers | Picker open/close restores exact parent focus and draft; Escape/outside obeys layer owner. |
| Disabled/read-only | Busy/disabled controls reject ineligible actions. Read-only/dirty/save semantics remain caller decisions. |
| Editing | Child TextInput/TextArea/Select semantics apply; Form does not flatten their Escape/paste differences. |
| Resize/Unicode | Re-measure child fields and validation/actions; preserve keyed focus and display-cell clipping. |
| Capability/color | Field, error, summary, action, modal, disabled, and busy styles resolve semantically across capabilities. |
| Motion | Child feedback/fade timing follows their contracts; Form adds no animation loop. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover on applicable child/action |
| Activation | not applicable as a generic Form surface; child/action gestures apply |
| Composition | valid/invalid submit, hidden/disabled reconciliation, nested picker return/cancel, busy submit, auxiliary action |

Run applicable cases at all five plan dimensions and truecolor, 256, 16, none, and nocolor. Do not invent interaction states for display-only summaries.

## Verification

The immutable oracle is [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b).

Use [`capture-plans/form.json`](../reference/capture-plans/form.json), W11, planned and without expected artifacts. Bind ExistingOracle/ExtractedOracle before candidate comparison.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W11-01 | Invalid submit focuses first invalid visible field. |
| W11-02 | Hide/disable a field after focus; reconcile deterministically. |
| W11-03 | Nested picker returns or cancels without resetting sibling drafts. |
| W11-04 | Submit while busy is blocked; cancel follows explicit eligibility. |
| W11-05 | Auxiliary Validate action does not submit or save. |

Record exact cells, cursor, child focus/capture/layer owners, field keys, action count/target, draft/committed values according to child redaction policy, and validation/dirty/busy state.

Negative tests:

- Hidden/disabled field changes cannot leave stale focus, hit, or capture ownership.
- Invalid submit cannot emit Submit; it focuses exactly the first invalid visible field.
- Busy submit cannot emit; Validate cannot submit/save; Cancel follows explicit eligibility.
- Nested picker close/cancel cannot reset sibling drafts or parent focus.
- Form cannot mutate caller values, save, persist, or perform product operations during draw/update beyond typed child/form responses.
- Child controls are painted and hit-tested as real controls exactly once.
- Candidate output cannot become expected output.

Accept after external API, traversal/reconciliation, validation/action, exact snapshots, nested layer, and independent review gates pass.

## Rejected use

Forbidden: painting field rows and a submit label with `ui.paint_str` and saving on click instead of composing real child controls.

```rust
// Forbidden: preview owns form rendering and the save.
ui.paint_str(0, 0, "Name: Ada");
ui.paint_str(4, 0, "[ Submit ]");
if clicked { save(values); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate fields or buttons. Compose `Form::new` with real child controls and route the typed `FormAction`.

## Known gaps

- Capture plan W11 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-001 prelude/forms](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
