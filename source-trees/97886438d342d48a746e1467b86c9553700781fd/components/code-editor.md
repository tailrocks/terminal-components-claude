# CodeEditor

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-editors.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W29 · Phase: P5 · Legacy family: C37.

## Purpose and exclusions

CodeEditor presents a borrowed document with a shared text editing core, gutter, line numbers, syntax spans, diagnostics, find state, completion, and a scrolling viewport. The caller supplies the document, source revision, highlighter/segmenter callbacks, diagnostics, and completion provider.

The component must not own syntax parsing, language-server transport, file watching, or save policy; change a caller document from `draw` or from read-only input; keep a hidden global text buffer or source-file store; promise a universal widget trait or legacy `WidgetId`/`Outcome`/`RenderCtx` API; treat diagnostics or completion as decoration that can alter source coordinates; or embed a second grapheme editor, scroll model, or display mapping. It shares the text core with TextInput and TextArea and shares the projection/scroll machinery with TextViewport.

The related canonical contracts are [`TextInput`](./text-input.md), [`TextArea`](./text-area.md), [`TextViewport`](./text-viewport.md), and [`Completion`](./completion.md).

## Public API

The signatures below are the proposed target for the in-place refactor, not a claim about current Rust exports. No signature is source-checked.

```rust
CodeEditor::new(id: Id, document: &'a dyn TextSource) -> CodeEditor<'a>

update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut CodeEditorState,
) -> Response<CodeAction>

draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &CodeEditorState,
) -> Rect

measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

Builders:

```rust
read_only(bool)
highlighter(&'a Highlighter)
segmenter(&'a Segmenter)
diagnostics(&'a [Diagnostic])
tab_behavior(TabBehavior)
line_numbers(bool)
patch(StylePatch)
```

The target action family is:

```rust
CodeAction::Edited
CodeAction::Commit { text: String }
CodeAction::Cancelled
CodeAction::SelectionChanged
CodeAction::CompletionRequested { position: TextPosition }
CodeAction::SegmentActivated { key: ItemKey }
```

The caller receives the proposed committed text or typed request and decides how to apply it. No draw path mutates the source.

`CodeEditorState` contains shared text/editor state, source revision, selection and caret, vertical/horizontal viewport, find query/matches/current match, completion controller, edit/read-only mode, and any diagnostics display state. It does not own the source document, language service, or file operations. Source-keyed selection/caret ranges are reconciled on revision changes. Stale completion ranges and highlighting/projection caches are discarded rather than applied against new offsets.

## Ordinary use

Consumer recipe: [EX-12 — Code and diff views](../api/consumer-recipes.md) (`proposed_target`).

The real preview consumer is the Showcase CodeEditor page. An independent synthetic fixture supplies a short document with stable line keys, caller diagnostics, and a completion provider.

The caller supplies text, diagnostics, and completion data; the editor owns caret geometry, selection, scrolling, and edit policy. The caller applies the typed request to application data and never draws diagnostics, line numbers, or a caret. Read-only usage still navigates, selects, finds, and scrolls.

## Ownership

- The caller owns languages, parsing, language servers, file I/O, persistence, diagnostics computation, and completion data.
- The component owns caret/selection, draft editing state, find query and current match, viewport positions, and completion controller state.
- `update` is the only mutating phase. It routes overlay/completion priority, find editing, keyboard editing/navigation, pointer caret placement and selection, scroll, and typed actions. It validates read-only before dispatch on every event so a mode change cannot leave an edit path open.
- `draw` reads the document and state, computes one source-to-display projection, then paints gutter, line numbers, current-line indication, syntax, diagnostics, selection, caret, find marks, completion, scrollbar, and fade. Styles never change logical text coordinates. The candidate must not mutate durable state while painting.
- `measure` computes line count, gutter width, text area, completion anchor, and scroll geometry from current constraints. Resize invalidates only the relevant projection/cache facts and keeps semantic caret/selection where valid.
- Runtime owns focus, hit, and capture. Completion and find overlays receive input before the editor body when they are active.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`collections`](../foundations/collections.md), and [`conformance`](../foundations/conformance.md).

## Customization

Parts are `container`, `gutter`, `line-number`, `current-line`, `text`, `syntax`, `selection`, `cursor`, `diagnostic`, `find`, `completion`, `scrollbar`, and `fade`.

Ordinary customization patches declared syntax, selection, or diagnostic tones. The baseline appearance, spacing, focus gutter, current-line treatment, diagnostic marks, cursor visibility, completion placement, and scrollbar/fade policy remain unchanged. Read-only is visibly distinct from disabled while remaining navigable. Focused monochrome output keeps the baseline navigation gutter. Truecolor/256/16/no-color style resolution and motion use shared foundations. No generic whole-editor activation feedback is added. Cursor, find, completion, and diagnostic emphasis use only applicable shared timing and reduced-motion rules.

Advanced customization keeps the frozen gutter recipe `▎ marker nn`: `▎` marks the cursor line, `›` the current block, the spinner a running block, and `!` a diagnostic. The current line number is bold; other line numbers in the current block are secondary. The footer shows the find bar or nearest diagnostic on the left and a position such as `ln 1/26 · col 18` on the right. Syntax tones are supplied by the caller; selection uses the popover plane, find matches and bracket matches use their documented underline/bold roles, and diagnostics underline the affected range.

One display mapping handles tabs, line numbers, styled runs, diagnostics, selection, completion anchor, and cursor. Grapheme clusters, combining marks, wide characters, and nonzero-origin/narrow regions never split cells or write outside the allocation.

## Behavior

Read-only mode still supports focus, caret positioning, navigation, selection, copy, find, and scrolling. It rejects insert, delete, indent, dedent, paste, and commit. A click in read-only moves the caret without creating a document edit.

Editable mode uses the shared text core. Enter or `i` begins editing; `a` enters and moves right. Escape cancels/commits according to the shared editor policy captured from the baseline; the committed target API exposes the result as `Commit` or `Cancelled`, never an untyped side effect. Ctrl+Enter/other configured commit keys commit without changing document text. Multiline paste is one edit operation with grapheme-safe cursor and selection behavior.

Tab and BackTab either indent/dedent a selected range or insert configured indentation. `TabBehavior` can instead commit and emit a leave action for a form-like composition. This policy is explicit; it cannot be inferred from the component caller.

Navigation uses arrows or `h`/`j`/`k`/`l`, PageUp/PageDown, Home/End and `g`/`G`. `{`/`}` jump between caller-supplied blocks where the document provides them. `/` opens find; find query editing supports insertion, grapheme-safe backspace, paste, Enter/Shift+Enter next/previous, and Escape cancellation. `n`/`N` navigate matches. Query edits rebase ranges from original source offsets after case expansion and Unicode segmentation.

Pointer click enters editing once in writable mode and places the caret at the display position; drag extends source selection. Release outside must end drag safely.

A source replacement invalidates stale find, diagnostic, highlight, and completion ranges. Selection/cursor mapping cannot split graphemes, tabs, controls, or wide cells; copy returns source text.

## Visual matrix

| Axis | Required states | Applies when |
|---|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | Always |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover | Interactive editor/popup |
| Activation | No generic whole-editor activation; edit, commit, find, completion, and selection actions are typed | Editor recipe |
| Editing | navigation, editing, selected text, invalid, read-only, commit, cancel, blur, tab traversal, source revision conflict | Editor mode |
| Scroll | no overflow, start, middle, end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected cursor row | Overflow exists |
| Output | source replaced, source appended, selection, source copy, cursor clipped | Source changes/selection |
| Readiness | loading/error diagnostics or completion state only when caller supplies it | Provider state |
| Motion | every applicable cursor/completion/feedback phase, paused/reduced motion | Theme policy |

## Verification

Evidence is pinned to the baseline commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Source evidence is `src/widgets/code.rs` (blob `3c653fc1e7f76563b92ff702abeec00fd3574bbb`) and completion behavior in `src/widgets/completion.rs` (blob `89c9e468c89dfc91c1afa69e11fefe951bcddade`). Inline tests cover scrollbar endpoints, read-only navigation, edit commit/cancel, find grapheme offsets, multiline paste, document replacement, manual scroll, horizontal cursor visibility, tab behavior, resize, and narrow/nonzero-origin clipping. The visual consumer is the Showcase CodeEditor page and its protected snapshots under `snapshots/showcase/pages/codeeditor`; interaction evidence remains in `tests/visual_baseline`. See [`visual-parity`](../verification/visual-parity.md), [`oracle-and-provenance`](../verification/oracle-and-provenance.md), and [`public-api`](../api/public-api.md).

Use [`../reference/capture-plans/code-editor.json`](../reference/capture-plans/code-editor.json). It is planned, with no expected artifacts. Bind baseline captures as `ExistingOracle`/`ExtractedOracle`; new robustness or API guarantees are `Extension`.

Run applicable cases at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50` with truecolor, 256-color, 16-color, `none`, and `nocolor` capabilities.

| Case | Required observation |
|---|---|
| W29-01 | Read-only click/caret/selection never mutates the document. |
| W29-02 | Edit mode, commit/cancel, and multiline paste preserve source ranges. |
| W29-03 | Find next/previous, grapheme query edits, and query cancellation. |
| W29-04 | Diagnostic, selection, current line, and syntax overlap correctly. |
| W29-05 | Completion popup remains anchored to the caret after resize. |
| W29-06 | Tab expansion and styled-run grapheme boundaries stay aligned. |
| W29-07 | Long document performs visible-only highlight/draw work under policy. |

Record dimensions, exact cells, symbols/continuations, colors/modifiers, cursor position/visibility, focus/capture/layer owners, source revision, selection/caret, draft/committed text, completion/find state, typed action count and target. Draw cannot create expected output.

Required negative tests:

- Read-only consumers cannot mutate the document through any action path, including Tab, BackTab, paste, completion, or a mode switch during editing.
- `draw` cannot commit, alter caret/selection/scroll, change source revision, or consume completion/find actions.
- A source replacement invalidates stale find, diagnostic, highlight, and completion ranges.
- Selection/cursor mapping cannot split graphemes, tabs, controls, or wide cells; copy returns source text.
- Narrow, zero, tiny, and nonzero-origin editor/find allocations cannot panic or paint outside their area.
- Scrollbar drag reaches exact top/bottom and leaves no capture on release.
- Completion/diagnostic customization cannot replace editor layout or focus ownership.

Acceptance requires an external consumer example, source-state tests, exact applicable snapshots/traces, and independent review of ownership and evidence.

## Rejected use

Forbidden: painting line numbers and text with padded strings and inserting typed characters instead of using CodeEditor.

```rust
// Forbidden: preview owns editor rendering and key handling.
ui.paint_str(row, 0, "  12  fn main() {");
if let Key::Char(c) = key { doc.insert(caret, c); }
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

Use `CodeEditor::new` with a keyed document source and handle the typed `CodeAction`.

## Known gaps

- Capture plan W29 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-003 historical editor and cockpit paths](../implementation/code-remediation-backlog.md): pending.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
