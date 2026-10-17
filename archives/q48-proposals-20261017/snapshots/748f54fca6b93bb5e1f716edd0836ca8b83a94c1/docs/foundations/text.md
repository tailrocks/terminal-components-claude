# Shared text, editing, and display projection

**Canonical owner:** Unicode-safe text operations, source/display coordinate
mapping, styled segmentation, wrapping, fuzzy matching, and bounded text
projection caches. This is the target contract for the in-place Termrock
refactor; the current `junie_tui` text types remain legacy implementation
evidence until the implementation phase.

**Specification record:** F07, legacy family C07. The visual authority is the
immutable [`visual-baseline` commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b).

Read this with the [public API](../api/public-api.md), [shared
types](../api/types.md), [layout and measurement](layout.md), [semantic
theme](theme.md), [identity and revisions](identity.md), and [secret-aware
validation](secret-validation.md) contracts. Component pages own their
editing policy and visual parts; this page owns the shared engine and source
projection.

## Baseline evidence and future names

The frozen implementation supplies behavior evidence in
[`src/core/text.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/text.rs),
[`src/ui/text.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/text.rs),
[`src/widgets/field_common.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/field_common.rs),
and [`src/widgets/viewport.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/viewport.rs).
Current `TextBuffer`, `TextInput`, `TextArea`, and `WidgetId` spellings are
legacy source names. The future contract is the shared Termrock text core;
it does not preserve those types as public compatibility requirements.

## Responsibility

This foundation owns:

- grapheme-safe caret movement, word movement, selection, deletion, insertion,
  newline policy primitives, paste normalization, and edit history;
- revisioned borrowed text sources and stable line identities;
- source positions and ranges that remain valid only at their source revision;
- segmentation of logical graphemes into display cells before style spans are
  painted;
- terminal-cell width, truncation, horizontal slicing, wrapping, tab display,
  control display, matching, and fuzzy ranking;
- projection caches keyed by source revision, width, tab/wrap policy, and
  theme metrics;
- bounded retained-line indexes and deterministic anchor/selection behavior
  when data is appended, replaced, or evicted.

Runtime owns input normalization, focus, capture, and time. Layout owns
rectangles and clipping. Theme owns semantic roles and styles. Collections own
stable item reconciliation. Secret storage and validation restrictions belong
to [the secret foundation](secret-validation.md). Application code owns
documents, logs, providers, persistence, and external completion services.

## Non-goals

- no terminal escape parser, PTY, shell, language server, file/database IO, or
  provider service;
- no universal widget or rendering trait and no `show()` method that combines
  editing semantics with paint;
- no source cloning per frame and no library-owned unbounded document/log;
- no blanket promise of BiDi, font shaping, or terminal-font fidelity beyond
  the documented Unicode width policy;
- no component-specific Enter/Escape/Tab behavior hidden in the core. The
  core exposes operations; TextInput, TextArea, and CodeEditor choose their
  documented policy;
- no semantic color or focus decisions. Styled spans carry semantic tones and
  modifiers; [the theme contract](theme.md) resolves them.

## Proposed source and editing API

These Rustdoc-style declarations define the target data boundary. They are
design notation, not compiled exports.

```rust
pub trait TextSource {
    fn revision(&self) -> Revision;
    fn line_count(&self) -> usize;
    fn line(&self, index: usize) -> Option<TextLine<'_>>;
}

pub struct TextLine<'a> {
    pub key: ItemKey,
    pub text: &'a str,
    pub spans: &'a [StyleSpan],
}

pub struct TextPosition {
    pub line: ItemKey,
    pub byte: usize,
}

pub struct TextRange {
    pub revision: Revision,
    pub start: TextPosition,
    pub end: TextPosition,
}

pub struct TextEditorCore { /* fields remain private */ }

impl TextEditorCore {
    pub fn new() -> TextEditorCore;
    pub fn apply(&mut self, edit: TextCommand) -> EditOutcome;
}

pub fn width(text: &str) -> usize;
pub fn truncate(text: &str, columns: usize) -> Cow<'_, str>;
pub fn fuzzy(query: &str, text: &str) -> MatchResult;
```

`TextEditorCore` is one mechanism shared by `TextInput`, `TextArea`, and
`CodeEditor`. A component supplies its mode, committed value/source, and
policy; it must not fork cursor, deletion, selection, or Unicode algorithms.
Plain editable components may expose a draft string through their narrow safe
state view. Secret specialization follows the [secret contract](secret-validation.md)
and never exposes plaintext through ordinary debug, copy, serialization, or a
generic draft accessor.

### Revisions and ranges

1. `Revision` identifies accepted source content and order. It advances before
   update/draw when a caller changes a borrowed source.
2. Every `TextRange`, completion replacement, mark, and selection records the
   source revision and uses valid grapheme boundaries. A stale or invalid
   range is rejected; it cannot mutate a newer source by byte coincidence.
3. A text conflict during editing preserves the draft and reports a typed
   conflict. The caller explicitly chooses `KeepDraft` or `Reload` through the
   conflict policy; the library never silently discards either value.
4. Selection crossing a newline retains newline semantics in source copy and
   edit commands. Display wrapping and clipping do not change source text.

### Graphemes, width, and display

Segment each logical line into complete grapheme clusters before splitting or
combining style spans. Use the documented `unicode-width` policy for terminal
cells:

- combining and zero-width marks attach to the preceding logical cell;
- double-width CJK and emoji clusters occupy a lead and continuation cell;
- a wide cluster clipped at an edge is represented by the documented ellipsis
  or padding policy, never a partial UTF-8 sequence or orphan continuation;
- tabs expand only for display according to the active tab policy;
- controls use safe display glyphs without rewriting the source copied back to
  the caller;
- cursor, selection, click-to-position, wrapping, and copy all use the same
  source/display mapping.

The contract makes no unverified claim about right-to-left shaping or font
rendering. New Unicode policy beyond this list is an extension requiring its
own evidence lane.

## Editing policy integration

The shared engine gives components the operations. Their distinct behavior is
intentional and must remain visible in component contracts:

| Component | Navigation mode | Editing mode | Escape | Paste |
| --- | --- | --- | --- | --- |
| `TextInput` | focus/cursor navigation | single-line draft | restores the start-of-edit value and cancels | tagged paste starts editing, then inserts |
| `TextArea` | scroll/navigation | multiline draft; Enter inserts newline | finishes and commits the edit | inserts only when editing is already active |
| `CodeEditor` | source/document navigation | shared editor commands; caller controls commit policy | follows its documented `CodeAction` policy | uses source-aware edit/paste mapping |

The table is a policy boundary, not permission to duplicate the underlying
engine. TextInput and TextArea also preserve their different newline
normalization and focus/scroll behavior. `Tab` and `Shift+Tab` commit/traverse
according to each component's API and the runtime key scope; the text core
does not globally consume them.

`TextViewport` and `DiffView` use the read-only projection and selection model.
Viewport log mode follows only at the real tail; prose mode never turns into a
tailing log because `End` was pressed. Panel plus TextViewport is the canonical
replacement for the old `ScrollPanel` mechanism.

## Styled projection and caching

`StyledText<'a>` and `StyleSpan` borrow logical source text. Segment first,
then resolve semantic tones/modifiers through [the theme](theme.md), then
write cells inside the rectangle supplied by [layout](layout.md). Copy
operations use source text and source ranges, never rendered truncation,
padding, tabs, control stand-ins, or masked glyphs.

Projection caches include every input that can alter cells or positions:

- source `Revision`;
- available width/height and non-zero-origin geometry where relevant;
- tab size and wrap mode;
- theme metrics and capability-dependent display policy;
- highlighter/segmenter revision or equivalent pure callback identity;
- retention and fade policy when visual rows are indexed.

Append and tail replacement extend or invalidate only the affected projection
when possible. Width, wrap, theme, or source replacement rebuilds the relevant
index. Cache bounds and undo/history limits are explicit. A component never
clones an entire caller-owned document or log to make a frame.

When a retained source evicts lines, reconcile anchors by stable line key:
selections clip to surviving boundaries, a fully evicted selection disappears,
marks for removed lines are removed, and reading/caret anchors move according
to the documented fallback. An evicted line must never silently become a
selection on an unrelated newly appended line.

## Integration rules

- `measure` reads source/model and theme metrics without changing the editor,
  scroll, selection, focus, or cache-visible semantic state. `draw` consumes
  immutable state and publishes cursor/geometry facts; updates apply edits and
  reconcile revisions before the next draw.
- `TextInput`, `TextArea`, `CodeEditor`, `TextViewport`, `DiffView`,
  `FilterList`, `Completion`, `Grid` editors, and terminal-cell adapters use
  this mapping. Their pages own component-specific actions, fields, and
  advertised parts.
- Completion and syntax/highlight callbacks are borrowed, pure, revision
  aware, and nonblocking. They cannot query an external service synchronously
  during draw.
- Stable `ItemKey` line identities come from [identity](identity.md) and
  collection reconciliation; byte offsets are local coordinates only.
- Scroll offsets, scrollbar geometry, fades, focus, and pointer capture come
  from their owning foundations. This page supplies content extents and
  source/display positions to them.

## Conformance and negative tests

Text cases use exact cell comparisons plus source/state observations from the
[visual parity](../verification/visual-parity.md) and [interaction
parity](../verification/interaction-parity.md) contracts. Record source
revision, draft/committed values, caret/selection, visible line keys, cursor,
focus/capture, and typed action target. The shared registry classifies each
case as `ExistingOracle`, `ExtractedOracle`, or `Extension`.

Required proof:

- styled grapheme split across spans;
- combining marks, CJK, emoji, tabs, controls, cursor movement, selection,
  deletion, and copy;
- clipped wide glyph at both viewport edges without continuation corruption;
- revision-tagged stale completion/edit range rejected;
- append, replace, and eviction preserve surviving marks/anchors and
  invalidate removed selections;
- TextInput Escape cancels while TextArea Escape commits;
- navigation-mode versus editing-mode paste behavior;
- resize/wrap/cache-key changes retain source identity and valid cursor;
- bounded visible-row segmentation and no whole-document clone per frame.

Negative tests must fail when a byte offset splits a grapheme, copied text
contains display decorations, a stale range edits current data, a hidden
control leaks raw escape content, a wide glyph overflows its clip, a source
eviction selects an unrelated row, draw mutates the editor, or an unbounded
projection/history cache grows with every frame.

## Source-pack coverage

This page migrates F07's complete shared text contract: `TextSource`,
revisioned line/range types, one editor core, grapheme/word/edit operations,
source/display segmentation, tab/control copy policy, Unicode width and
continuation cells, revision-safe completion, bounded projections, append/
replace/eviction reconciliation, plain draft boundaries, and the required
proof cases. Component-specific Escape/paste differences remain explicit in
the component inventory. F01 owns identity/revision construction; F09 owns
secret handling; F06 owns semantic style resolution; those rules are linked
and not duplicated here.
