# Code editor BackTab dedent

Frozen source `visual-baseline` commit `4169783ae883788186d161af09ba9c481bd84e5c` implements `CodeEditor::dedent_selection` in `src/widgets/code.rs`. It removes up to the configured number of leading ASCII spaces on each selected line, or on the cursor line when there is no selection. Its `selection_lines` contract excludes the following line when a selection ends at that line's start. Tabs are not indentation characters.

At PR #38 source `6825b07e86bab75872866c6ed49e6877084db9bb`, `crates/termrock-editors/src/code.rs` consumed `CodeCmd::Tab(true)` without editing. Added real-Runtime tests first and ran them before the implementation:

```text
CARGO_TARGET_DIR=/private/tmp/termrock-backtab-before-6825 cargo nextest run -p termrock-editors --lib code::tests::backtab -- --nocapture
Nextest run ID: 926f7749-6ca2-464f-ab03-76ffee5e2968
Exit: 100. Four behavior probes failed; the no-op probe passed.
```

The shared editor now removes leading spaces through `TextEditorCore::apply(Paste(""))` in descending grapheme-safe ranges, restores cursor and selection direction through byte-offset mapping using those expanded ranges, and calls `after_edit` once. It preserves the frozen `Changed` result for a BackTab even when no leading spaces are removed. Read-only BackTab is blocked; disabled handling and `TabBehavior::Leave` retain their existing paths. No screen-specific behavior or public API changed.

Runtime probes cover selected and reversed multiline ranges, next-line-start boundary, current-line dedent with wide and combining text, tabs and empty/no-op lines, zero indent, read-only and disabled behavior, and `TabBehavior::Leave`.

Independent review of commit `2505fec7a4041154e8d7ff1c1ccd26c9efcd51dd` found that byte ranges could split an ASCII space from an attached combining mark; `TextEditorCore::select_range` then deleted nothing while the cursor mapping assumed deletion. Review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/editor-backtab-independent-review-2505fec-20261021/review.json`, SHA256 `bcb094c2e78610886b87088057b0b25bd614030a8c1e5937c8c9a4b557daec92`. Added current-line and reversed multiline regressions. Both failed on that commit:

```text
CARGO_TARGET_DIR=/private/tmp/termrock-backtab-g2-before-2505 cargo nextest run -p termrock-editors --lib code::tests::backtab_dedents_attached_combining -- --nocapture
Nextest run ID: e9025f46-2d44-4460-8e82-177ad7812d8c
Exit: 100. Both combining-mark probes failed.
```

Removal ranges now expand using shared grapheme iteration to the same floor/ceiling boundaries as frozen `TextBuffer::remove_range`; offset mapping uses expanded ranges.

```text
CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after-6825 cargo nextest run -p termrock-editors --lib code::tests::backtab -- --nocapture
Nextest run ID: 95abbdbd-8a42-4a88-9b65-119ea665f652
Exit: 0. Five tests passed, zero failed before the independent grapheme finding.

CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after2-2505 cargo nextest run -p termrock-editors --lib code::tests::backtab_dedents_attached_combining -- --nocapture
Nextest run ID: 19c12fa4-a8cc-4905-b02a-d4b1a77d7cd4
Exit: 0. Both combining-mark probes passed.

CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after2-2505 cargo nextest run -p termrock-editors --all-targets
Nextest run ID: e5526b68-a634-4a36-a004-a8577bcdfd14
Exit: 0. 69 tests passed, zero skipped after grapheme-boundary correction.

CARGO_TARGET_DIR=/private/tmp/termrock-backtab-after2-2505 cargo clippy -p termrock-editors --all-targets -- -D warnings
Exit: 0.

cargo fmt --all --check
Exit: 0 after formatting.
```

The real-Runtime regression test fails on the pre-change editor code and passes with this change. Independent review and complete application parity remain outstanding; this increment does not establish either acceptance gate.
