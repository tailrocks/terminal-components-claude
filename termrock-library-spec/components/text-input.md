# W08 · TextInput

**Group:** Forms · **Phase:** P3 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller owns committed value. State owns the active draft and rollback snapshot. No geometry or palette in state; no editing String is ever copied per paint.

Legacy family mapping: `C17`, `C19`  
Proposed implementation home: `crates/termrock/src/components/input.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/input.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/input.rs) — observed Git blob `f440b74c7d305e987ec31bf04d34afd993844bb0`
- [src/widgets/field_common.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/field_common.rs) — observed Git blob `bd41cdd85e9c2605c356f7468dc0de3e3c50e54b`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: input.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/input.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/audit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/audit)
- [snapshots/jackin/editor](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/editor)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
TextInput::new(id: Id, value: &'a str, revision: Revision) -> TextInput<'a, Plain>
TextInput::secret(id: Id, value: &'a Secret, revision: Revision) -> TextInput<'a, SecretText>

update(&self, cx: &mut Cx<'_>, state: &mut TextInputState<M>) -> Response<TextAction<M::Value>>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TextInputState<M>) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
label(&'a str)
placeholder(&'a str)
help(&'a str)
required(bool)
disabled(bool)
read_only(bool)
validation(Option<&'a ValidationMessage>)
validator(&Validator)
blur(BlurPolicy) // Commit baseline
secret_policy(SecretPolicy) // secret specialization only
patch(StylePatch)
conflict_policy(ConflictPolicy) // default PreserveDraftAndReport
```

**Durable state:** TextInputState<Plain>: EditPhase, draft, rollback value, caret, selection and horizontal offset. Secret mode uses TextInputState<SecretText> with redacted Debug and no Clone.

Safe access: `phase()`, `caret()`, `selection()`, `resolve_conflict(ConflictResolution)`. Plain specialization also has `draft() -> &str`; SecretText does not. Commit transfers value through the typed action.

**Typed actions:** TextAction<V>::{Edited, Commit { value: V }, Cancelled, Conflict}; V is String for Plain or Secret for SecretText. Conflict is a payload-free notification; resolve through explicit state conflict policy.

## Visual parts and composition

Advertised styled parts: `container`, `label`, `gutter`, `text`, `placeholder`, `selection`, `cursor`, `help`, `error`, `marker`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [field](./field.md)

## Behavioral contract

1. Keyboard focus is navigation, not edit mode. Enter/F2 or a completed click starts editing; click positions the caret using displayed grapheme widths.
2. Enter commits and validates. Escape restores the start-of-edit value. Tab/Shift+Tab commits before focus traversal. Baseline blur-commit is moved from render to update.
3. The tagged on_paste starts editing before insertion, even from navigation mode. Route paste to this owner only after modal priority. Do not generalize this behavior to TextArea.
4. Masked mode is one bullet per grapheme, so a masked wide character occupies one cell. Tail reveal is explicit and only outside editing. Tests use synthetic secrets.
5. Invalid text may be committed as a draft value with an error indication; form submission remains separately validated. A conflicting external value change during editing follows the explicit revision/conflict contract.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| editing | `navigation`, `editing`, `selected_text`, `invalid`, `read_only`, `commit`, `cancel_or_finish_per_component`, `blur`, `tab_traversal`, `source_revision_conflict` | editable specialization |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W08-01 | focus then Enter; edit then Escape restores original |
| W08-02 | completed click on a wide/combining/masked grapheme; Down-only does not edit |
| W08-03 | paste while navigation starts editing; disabled paste is ignored |
| W08-04 | edit then Tab/Shift+Tab/blur emits one commit and correct traversal |
| W08-05 | required empty and custom validation error; correction clears prior error |
| W08-06 | secret mode: no plaintext in Debug, clipboard, logs, snapshots or rollback after close |
| W08-07 | external value changes while draft is active; no silent overwrite |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/text-input.json](../capture-plans/text-input.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
