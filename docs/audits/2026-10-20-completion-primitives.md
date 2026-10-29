# Completion editor primitives

`CodeEditorState::splice_completion(replace_len, insertion)` replaces the requested byte length immediately before the editor cursor. `TextEditorCore::select_range` snaps that range to existing grapheme boundaries. The operation uses the normal edit lifecycle: changed text advances the highlight generation, refreshes find matches and updates vertical scroll; diagnostics and edit mode remain intact. Empty no-op insertion returns `false` and does not advance the generation.

`CompletionController::accept_with` passes the requested byte length and the selected item's insertion text to one caller-owned splice closure. It closes completion state and the popup after the closure returns, including when the splice reports no change. Existing `accept(TextBuffer, ...)` keeps its signature and delegates through this path, retaining its current replacement and dismissal behavior.

The adapter does not decide whether completion should open or whether Tab navigates. Callers retain that policy, so a shared editor-completion composition can accept during editing while preserving a consumer's existing navigation behavior in other modes. Candidate lists remain unchanged.

Focused coverage lives in `crates/termrock-editors/src/code.rs` and `crates/termrock-editors/tests/completion_integration.rs`: ASCII replacement before the cursor, Unicode byte lengths, grapheme-safe clamping, out-of-range replacement, editor lifecycle state, diagnostics retention, one splice invocation, and dismissal persistence across another frame.
