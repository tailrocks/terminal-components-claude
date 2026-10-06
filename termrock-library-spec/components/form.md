# W11 · Form

**Group:** Forms · **Phase:** P3 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Use borrowed declarations with stable FieldKey/child Id; caller binds live values, state and validation. No untyped value map, schema engine or persistence.

Legacy family mapping: `C21`  
Proposed implementation home: `crates/termrock/src/components/form.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/bin/jackin_preview/screens/modals.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs)
- [src/bin/jackin_preview/screens/editor.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/editor.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: form.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/form.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/audit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/audit)
- [snapshots/jackin/editor](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/editor)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Form::new(id: Id, fields: &'a [FieldSpec<'a>], actions: &'a [ActionMeta<'a>]) -> Form<'a>

update(&self, cx: &mut Cx<'_>, state: &mut FormState, controls: &mut dyn FormControls) -> Response<FormAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &FormState, controls: &dyn FormControls) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
validation(&'a [FieldError])
dirty(bool)
busy(bool)
patch(StylePatch)
```

**Durable state:** FormState: submission attempt and invalid-field focus bookkeeping only. Child control states and caller values remain outside FormState.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** FormAction::{Submit, Cancel, Auxiliary { action: ActionKey }}

## Visual parts and composition

Advertised styled parts: `container`, `field-group`, `validation-summary`, `actions`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [field](./field.md), [text-input](./text-input.md), [button](./button.md)

## Behavioral contract

1. Traversal follows visible enabled field declarations in paint order. Hidden fields have no hit/focus region; disabled fields cannot receive editing input.
2. Submit first commits the focused draft in update, validates current values, and emits Submit only when valid; otherwise focus the first invalid field.
3. Nested picker close restores the exact parent draft and focus. Layout slots paint real child widgets; Form never draws a decorative replacement of them.
4. Save/Discard/Cancel semantics and dirty computation remain caller decisions. Form reports intent only.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W11-01 | invalid submit focuses first invalid visible field |
| W11-02 | hide/disable a field after focus; reconcile deterministically |
| W11-03 | nested picker returns or cancels without resetting sibling drafts |
| W11-04 | submit while busy is blocked; cancel follows explicit eligibility |
| W11-05 | auxiliary Validate action does not submit or save |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/form.json](../capture-plans/form.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
