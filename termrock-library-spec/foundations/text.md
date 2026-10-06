# F07 · Shared text editing and source/display projection

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C07  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/core/text.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/text.rs)
- [src/ui/text.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/text.rs)
- [src/widgets/field_common.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/field_common.rs)
- [src/widgets/viewport.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/viewport.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
trait TextSource {
    fn revision(&self) -> Revision;
    fn line_count(&self) -> usize;
    fn line(&self, index: usize) -> Option<TextLine<'_>>;
}
struct TextLine<'a> { key: ItemKey, text: &'a str, spans: &'a [StyleSpan] }
struct TextPosition { line: ItemKey, byte: usize }
struct TextRange { revision: Revision, start: TextPosition, end: TextPosition }
TextEditorCore::new() -> TextEditorCore
TextEditorCore::apply(&mut self, edit: TextCommand) -> EditOutcome
text::width(text: &str) -> usize
text::truncate(text: &str, columns: usize) -> Cow<'_, str>
text::fuzzy(query: &str, text: &str) -> MatchResult
```

## Contract

1. One engine handles grapheme motion, word motion, deletion, selection, undo/edit history, paste and source/display coordinates. Higher-level widgets choose commit/cancel/tab/newline policy rather than forking text operations.
2. Segment a logical grapheme before splitting style spans. Tabs expand for display only; copy uses source text. Controls displayed safely must not rewrite the original copied content without an explicit copy policy.
3. Every editable range belongs to a source revision and valid grapheme boundary; reject stale or invalid completion/edit ranges. Selection crossing a newline preserves actual newline semantics.
4. Account for double-width glyph continuation cells and zero-width combining marks. Document the supported Unicode width policy; no unverified blanket BiDi or terminal-font fidelity claim.
5. Cache keys include source revision, available width, tab/wrap policy and metric/theme changes relevant to layout. Append/replace/eviction have distinct reconciliation tests.
6. Fields may keep a draft String during editing; borrowed output views must not clone a whole log/document per frame. Bound undo/history allocations explicitly.

## Required proof

- styled grapheme split across spans
- combining and CJK cursor movement, selection and deletion
- tabs/control display versus source copy
- revision-tagged stale completion rejected
- eviction preserves surviving marks and invalidates removed selection
- input Escape cancels while textarea Escape commits

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
