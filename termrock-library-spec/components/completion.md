# W21 · Completion

**Group:** Overlays · **Phase:** P4 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller supplies candidates and valid edit ranges. Completion does not invoke a language server or guess application syntax.

Legacy family mapping: `C30`  
Proposed implementation home: `crates/termrock/src/components/completion.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/completion.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/completion.rs) — observed Git blob `89c9e468c89dfc91c1afa69e11fefe951bcddade`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: completion.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/completion.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/codeeditor](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/codeeditor)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Completion::new(id: Id, owner: Id, items: &'a [CompletionItem<'a>], revision: Revision) -> Completion<'a>

update(&self, cx: &mut Cx<'_>, state: &mut CompletionState) -> Response<CompletionAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CompletionState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
anchor(Anchor)
replacement(TextRange)
patch(StylePatch)
```

**Durable state:** CompletionState: highlighted candidate key and scroll; CompletionController binds the popup to an editor Id and query revision.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** CompletionAction::{Apply { key, edit: TextEdit }, Dismissed}

## Visual parts and composition

Advertised styled parts: `container`, `row`, `kind`, `label`, `detail`, `match`, `scrollbar`, `fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [list](./list.md)

## Behavioral contract

1. Editor keeps typing/caret ownership while the popup handles its navigation keys. Tab/Enter apply once; Escape closes the popup before canceling editing.
2. A replacement range is a source-text range tagged with document revision, never display-cell coordinates. Reject stale/out-of-bounds edits.
3. The popup tracks the owner caret, flips/clamps and clips without stealing a second hardware cursor.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| layer | `closed`, `open`, `nested`, `escape`, `outside`, `owner_removed`, `resize_reanchor`, `focus_restore`, `modal_first_paste` | runtime-owned overlay |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W21-01 | editor typing while completion list remains open |
| W21-02 | Tab/Enter apply one revision-valid splice |
| W21-03 | Escape closes popup, second Escape follows editor policy |
| W21-04 | paste invalidates old candidate range |
| W21-05 | narrow bottom/right corner popup placement |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/completion.json](../capture-plans/completion.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
