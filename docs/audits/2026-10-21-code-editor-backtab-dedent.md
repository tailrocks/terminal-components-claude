# Code editor BackTab dedent

Frozen source `visual-baseline` commit `4169783ae883788186d161af09ba9c481bd84e5c` implements `CodeEditor::dedent_selection` in `src/widgets/code.rs`. It removes up to the configured number of leading ASCII spaces on each selected line, or on the cursor line when there is no selection. Its `selection_lines` contract excludes the following line when a selection ends at that line's start. Tabs are not indentation characters.

At PR #38 source `6825b07e86bab75872866c6ed49e6877084db9bb`, `crates/termrock-editors/src/code.rs` consumed `CodeCmd::Tab(true)` without editing. Added real-Runtime tests first and ran them before the implementation:

```text
CARGO_TARGET_DIR=/private/tmp/termrock-backtab-before-6825 cargo nextest run -p termrock-editors --lib code::tests::backtab -- --nocapture
Nextest run ID: 926f7749-6ca2-464f-ab03-76ffee5e2968
Exit: 100. Four behavior probes failed; the no-op probe passed.
```

The shared editor now removes leading spaces through `TextEditorCore::apply(Paste(""))` in descending ranges, restores cursor and selection direction through byte-offset mapping, and calls `after_edit` once. It preserves the frozen `Changed` result for a BackTab even when no leading spaces are removed. Read-only BackTab is blocked; disabled handling and `TabBehavior::Leave` retain their existing paths. No screen-specific behavior or public API changed.

Runtime probes cover selected and reversed multiline ranges, next-line-start boundary, current-line dedent with wide and combining text, tabs and empty/no-op lines, zero indent, read-only and disabled behavior, and `TabBehavior::Leave`.

```text
CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after-6825 cargo nextest run -p termrock-editors --lib code::tests::backtab -- --nocapture
Nextest run ID: 95abbdbd-8a42-4a88-9b65-119ea665f652
Exit: 0. Five tests passed, zero failed.

CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after-6825 cargo nextest run -p termrock-editors --all-targets
Nextest run ID: 8ef5a023-f2f1-4d07-a50a-c14a5f602088
Exit: 0. 67 tests passed, zero skipped after the final semantic adjustment.

CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after-6825 cargo clippy -p termrock-editors --all-targets -- -D warnings
Exit: 0.

cargo fmt --all --check
Exit: 0 after formatting.
```

The real-Runtime regression test fails on the pre-change editor code and passes with this change. Independent review and complete application parity remain outstanding; this increment does not establish either acceptance gate.
