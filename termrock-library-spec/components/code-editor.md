# W29 · CodeEditor

**Group:** Data and text · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow source text and pure highlight/segment callbacks. State owns editing drafts and view positions. Languages, parsing services and file IO remain outside the library.

Legacy family mapping: `C37`  
Proposed implementation home: `crates/termrock/src/components/code.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/code.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/code.rs) — observed Git blob `3c653fc1e7f76563b92ff702abeec00fd3574bbb`
- [src/widgets/completion.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/completion.rs) — observed Git blob `89c9e468c89dfc91c1afa69e11fefe951bcddade`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: code.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/code.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/codeeditor](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/codeeditor)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
CodeEditor::new(id: Id, document: &'a dyn TextSource) -> CodeEditor<'a>

update(&self, cx: &mut Cx<'_>, state: &mut CodeEditorState) -> Response<CodeAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CodeEditorState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
read_only(bool)
highlighter(&'a Highlighter)
segmenter(&'a Segmenter)
diagnostics(&'a [Diagnostic])
tab_behavior(TabBehavior)
line_numbers(bool)
patch(StylePatch)
```

**Durable state:** CodeEditorState: shared text/editor state, document revision, find state, viewport and completion controller; no language server or source-file owner.

Safe access: source-keyed selection/caret and reading offset. Mutations are explicit commands or dispatched input; no global Buffer or source collection is exposed.

**Typed actions:** CodeAction::{Edited, Commit { text: String }, Cancelled, SelectionChanged, CompletionRequested { position: TextPosition }, SegmentActivated { key: ItemKey }}

## Visual parts and composition

Advertised styled parts: `container`, `gutter`, `line-number`, `current-line`, `text`, `syntax`, `selection`, `cursor`, `diagnostic`, `find`, `completion`, `scrollbar`, `fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [text-area](./text-area.md), [completion](./completion.md), [text-viewport](./text-viewport.md)

## Behavioral contract

1. Reuse shared grapheme, editor and viewport machinery. Read-only must still support caret positioning and source-preserving selection/copy where the oracle does.
2. Find query editing and completion respect popup/editor priority; styling a diagnostic cannot change logical text coordinates.
3. Tabs, line numbers, current-line indication, inline diagnostics and selection must share one source-to-display projection.
4. Source revision changes invalidate highlighting/projection caches and stale completion ranges, not just replace a string behind cached offsets.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| editing | `navigation`, `editing`, `selected_text`, `invalid`, `read_only`, `commit`, `cancel_or_finish_per_component`, `blur`, `tab_traversal`, `source_revision_conflict` | editable specialization |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| output | `source_replaced`, `source_appended`, `selection`, `source_copy`, `cursor_clipped` | borrowed content projection; eviction only for retained-history sources, cursor only when owned |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W29-01 | read-only click/caret/selection without mutation |
| W29-02 | edit mode, commit/cancel and multiline paste |
| W29-03 | find next/previous and query cancellation |
| W29-04 | diagnostic plus selection plus current line overlapping |
| W29-05 | completion popup anchored to caret after resize |
| W29-06 | tab expansion and styled-run grapheme boundaries |
| W29-07 | long document: visible-only highlight/draw work under declared cache policy |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/code-editor.json](../capture-plans/code-editor.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
