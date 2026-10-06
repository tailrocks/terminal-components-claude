# W09 · TextArea

**Group:** Forms · **Phase:** P3 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow committed text; caller-owned state owns draft/editor positions. Field chrome and text operations are shared with TextInput, not copied.

Legacy family mapping: `C18`, `C19`  
Proposed implementation home: `crates/termrock/src/components/textarea.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/textarea.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/textarea.rs) — observed Git blob `5bfa5148a8b335c28ed52a5af305fa9df39917e5`
- [src/widgets/field_common.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/field_common.rs) — observed Git blob `bd41cdd85e9c2605c356f7468dc0de3e3c50e54b`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: textarea.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/textarea.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/audit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/audit)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
TextArea::new(id: Id, value: &'a str, revision: Revision) -> TextArea<'a, Plain>
TextArea::secret(id: Id, value: &'a Secret, revision: Revision) -> TextArea<'a, SecretText>

update(&self, cx: &mut Cx<'_>, state: &mut TextAreaState<M>) -> Response<TextAction<M::Value>>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TextAreaState<M>) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
label(&'a str)
help(&'a str)
rows(u16)
disabled(bool)
read_only(bool)
validation(Option<&'a ValidationMessage>)
wrap(WrapMode)
patch(StylePatch)
conflict_policy(ConflictPolicy) // default PreserveDraftAndReport
```

**Durable state:** TextAreaState<Plain>: shared text editor state, multiline caret/selection, vertical and horizontal scroll, edit phase and follow-caret. Secret specialization follows the same non-Clone policy as TextInput.

Safe access: `phase()`, `caret()`, `selection()`, `resolve_conflict(ConflictResolution)`. Plain specialization also has `draft() -> &str`; SecretText does not. Commit transfers value through the typed action.

**Typed actions:** TextAction<V>::{Edited, Commit { value: V }, Cancelled, Conflict}; explicit cancel command is distinct from the baseline Escape binding.

## Visual parts and composition

Advertised styled parts: `container`, `label`, `gutter`, `text`, `selection`, `cursor`, `help`, `error`, `scrollbar`, `fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [field](./field.md), [scroll-region](./scroll-region.md)

## Behavioral contract

1. Enter/F2 or completed click enters edit mode. Enter within edit inserts a newline through multiline edit grammar.
2. Important baseline exception: Escape finishes and commits a TextArea edit; it does not perform the single-line rollback behavior. Tab/Shift+Tab commits and traverses.
3. The tagged textarea on_paste requires editing already active; navigation-mode paste does not insert. Preserve the visible caller route when integrating other input policies.
4. Navigation keys scroll while not editing. During editing they move the caret and keep it visible. Wheel scroll changes reading position without changing focus.
5. Clip caret/selection and preserve source offsets across wrapped lines, empty final lines and resize.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| editing | `navigation`, `editing`, `selected_text`, `invalid`, `read_only`, `commit`, `cancel_or_finish_per_component`, `blur`, `tab_traversal`, `source_revision_conflict` | editable specialization |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W09-01 | edit Enter inserts newline; Escape commits and leaves edit mode |
| W09-02 | navigation-mode paste ignored; edit-mode bracketed paste preserves newlines |
| W09-03 | PageUp/PageDown and Home/End in navigation versus editing |
| W09-04 | selection spans newline, tab and grapheme boundaries |
| W09-05 | wheel while editing followed by caret movement; follow-caret resumes correctly |
| W09-06 | height 0/1 and rows+2 Field layout |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/text-area.json](../capture-plans/text-area.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
