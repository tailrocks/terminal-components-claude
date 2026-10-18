# F10 — Segment logical text before applying styles

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the retained shared TextViewport contract. Prove split-style grapheme geometry and exact copies in Holla Activity, Plan and file preview, with relevant Diff primitive regressions.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No deferred implementation slice; unrelated application redesign is excluded.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed text-loss defect.** `TextViewport::ensure_layout`,
`src/widgets/viewport.rs:305`, segments each span separately and drops zero-width
pieces. One span `👩‍💻X` puts X at column 2; split styled spans put X at 4.
Separate spans `a`, combining acute, `X` copy `aX`, losing the stored accent.
Text and interaction agents independently rendered and copied both cases.

Root: style boundaries incorrectly define grapheme boundaries. Segment a whole
logical line, map source byte ranges to complete graphemes, then resolve styles
with a documented deterministic precedence for a cluster crossing runs. Use the
same map for rendering, width, selection, caret and copy. Reuse current Line/Span
concepts; the previous Diff-specific span fix is not the generic root fix.

Risk: medium style/API compatibility. Acceptance: split/unsplit combining,
ZWJ, variation-selector and flag sequences retain identical text, widths and
copied ranges; only declared style precedence may differ. Test narrow clipping,
nonzero origin, mixed tones and existing Diff emphasis/geometry. Cache logical
graphemes with document revisions so correctness does not add idle reparsing.

## Evidence

**Slice status:** current shared slice complete.

**Shared boundary:** `TextViewport` segments a whole logical line into graphemes, maps source byte ranges to complete clusters and resolves split styles per cluster; render, width, selection, caret and copy share the map. Tests: `viewport.rs: split_styles_never_split_a_grapheme_cluster`, `narrow_clipping_and_nonzero_origin_keep_cells_whole`, `tabs_and_controls_display_visibly_and_copy_as_source`; `src/core/text.rs` and `src/ui/text.rs` grapheme tests (`word_motion_and_deletion_preserve_combining_clusters`, `wrapping_never_splits_or_overflows_a_wide_grapheme`, `find_preserves_smart_case_and_whole_source_graphemes`); `tests/focus_gutter.rs: text_selection_uses_reverse_video_only_in_monochrome`; Diff regressions in `src/widgets/diff.rs` still pass.

**Holla consumers:** activity output, plan output and the files preview render through the same viewport; `app_tests_parity.rs: hp14_output_streams_are_exact_and_retention_drops_are_stated` (ANSI-carrying lines sanitised and copied whole), `hp04_…` (control and Unicode previews), `hp03_…` (`café` file).

**Captures inspected:** `shots/h_hp04_preview_control`, `shots/h_hp03_find_unicode`, `shots/h_hp14_stream`; the F22 fidelity fixtures (`tools/fidelity_check.py`, 8 fixtures) confirm the capture tools agree with the reference cells for split styles, combining marks, ZWJ and CJK.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
