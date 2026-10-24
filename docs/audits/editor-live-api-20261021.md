# Shared CodeEditor Find API

Scope: `termrock-editors` public cursor and find inspection, plus public find
entry for the plain and completion-enabled editors. Fixed references remain
the `visual-baseline` source `4169783ae883788186d161af09ba9c481bd84e5c`,
tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, and design source
`f4867e9b5f8a08a3bb61ce80d644198a29f51984`.

`CodeEditorState::cursor_pos` delegates to `TextEditorCore::cursor_pos`, so
its zero-based column uses terminal-cell width. `find_info` borrows the
existing find state. It reports a zero-based current match, with `None` when
the query has no matches; it owns no second query or selection state.

`CodeEditor::open_find` and the keyboard Find binding share one transition:
preserve the existing query and selected match, switch to query typing, and
focus the editor. Disabled editors ignore the request; read-only editors allow
search. `CompletingCodeEditor::open_find` first dismisses the completion
controller and clears its items, then delegates to the same editor operation.

Real `Runtime` coverage lives in
`crates/termrock-editors/tests/code_editor_external_find.rs`. It compares
keyboard and sibling `Button` entry, checks the rendered find query, match
navigation, Escape cancellation, document preservation, completion dismissal,
disabled and read-only policies, wide and combining cell positions, reopening,
resize, and zero-sized draw bounds.

Focused validation on the writer revision:

```text
cargo nextest run -p termrock-editors
  62 passed, 0 failed, 0 skipped
cargo clippy -p termrock-editors --all-targets -- -D warnings
  passed
cargo fmt --check -p termrock-editors
  passed
```

This API and its crate tests do not prove Showcase adoption or product parity.
Showcase still has its `paint_baseline` path and must wire its live Find button
and cursor/find properties through shared APIs while removing that bypass in
the same product change. Unicode-safe find-query Backspace also remains a
separate baseline-backed defect; this work does not claim Unicode editing
completion.
