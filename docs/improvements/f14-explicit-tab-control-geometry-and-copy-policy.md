# F14 — Explicit tab/control geometry and copy policy

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Define source-preserving tab/control geometry and apply it at the shared text boundary used by Holla input, output and preview. Complete the retained cross-renderer fixture and required compatibility migrations so shared policy stays consistent.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No unrelated editor features; the shared policy and its cross-renderer correctness checks remain current work.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · design inconsistency / architecture weakness.** Literal `A\tB` gives
different geometry in TextInput/TextArea/CodeEditor versus the viewport's existing
four-space expansion. `TextBuffer` normalizes CR/LF but stores other controls;
separate renderers have no shared declared display/copy rule. Text probes and
interaction peer verification agree. Ratatui filters control graphemes here;
these observations do **not** prove terminal escape injection.

Define source storage separately from display: tab expansion, visible treatment
of controls, source-to-cell mapping and copy-as-source versus copy-as-displayed.
Reuse the logical grapheme mapping from F10 and existing four-space viewport
policy as the default candidate. Literal document tabs are not the same as the
Tab key's configurable code indentation. Preserve data; no silent control-byte
deletion. Risk: medium/high public cursor-column semantics. Acceptance: the same
tab/ESC/BEL/CR/LF/CRLF/combining fixture across input, textarea, code, cell editors,
viewport and both Diff modes proves declared storage, cursor, click, clipping
and copy behavior. Use additive policy/mapping APIs during compatibility migration.

## Evidence

**Slice status:** current shared slice complete.

**Policy:** source is stored as typed; display expands a tab to the next four-column stop and shows other controls as visible glyphs; copy returns the source. The same mapping serves TextInput, TextArea, CodeEditor, the viewport and both Diff modes. Tests: `viewport.rs: tabs_and_controls_display_visibly_and_copy_as_source`; `diff.rs: tab_indented_review_aligns_context_changes_and_copied_text`; `src/core/text.rs: every_text_entry_preserves_line_mode` (the shared buffer keeps control bytes; nothing is silently deleted); `src/widgets/input.rs: wide_graphemes_keep_cursor_and_clicks_aligned_after_scroll`. Tab controls register explicit geometry (`Tabs` registers `ctx.control(id, area, false)`), so keyboard focus and hit regions agree: `tabs.rs: active_tab_has_a_plane_and_the_only_accent_underline_and_no_gutter`, `hover_and_cursor_differ_from_active`.

**Holla proof:** `app_tests_parity.rs: hp04_browser_lists_previews_and_jumps_safely` (a preview with control bytes is shown visibly and never copied as escapes), `hp14_…` (`y` copies the exact retained lines); `sim/fs.rs: preview_is_bounded_sanitised_and_refuses_special_files`.

**Captures inspected:** `shots/h_hp04_preview_control`.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
