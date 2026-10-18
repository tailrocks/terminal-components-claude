# F20 — Find in read-only output through existing composition

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Compose query input and match navigation in Holla output and file preview. Apply every retained search/follow/focus/Unicode acceptance case to those surfaces; keep implementation local until shared reuse is demonstrated.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Jackin panes/Inspect integration and extraction of a cross-application controller remain deferred. Holla completion does not close the full two-application contract.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · coverage gap: confirmed capability absence; proposed interaction/API design.**
TextViewport/Diff expose no query or next/previous match action. Holla service
filters and Jackin command filtering do not search displayed output. CodeEditor's
FindState and shared `ui::text::find_ranges` already provide a source-grapheme
mapping foundation. Ecosystem and API agents independently verified both absence
and reuse. This is not a defect in a promised current search feature.

After F19, compose a query input/match navigation into two present consumers
(Holla output and Jackin panes/Inspect), then extract only the demonstrated common
controller. Reuse TextInput/HintBar and source mapping, not a new search widget.
Risk: medium focus/follow/content invalidation. Acceptance: pasted query never
edits output, empty/no-match/next/previous/wraparound states, Unicode ranges,
content revision/eviction, match reveal, resize, Escape and explicit follow
restoration. Defer regex, replacement, file search and multiple cursors; current
evidence supports text finding, not those broader semantics.

Design evidence and peer dispositions:
[design verification](../plan-verification/plan-design-verification.md).

## Evidence

**Slice status:** current Holla/shared slice complete · Jackin integration and a shared controller stay Later.

**Composition:** activity output and the files preview compose a query input (`/`), `ui::text::find_ranges` (now public) and viewport marks; Enter and Down move to the next match, Up to the previous, wraparound is stated, Esc closes find and restores follow; a new query resets the match index; the pasted query never edits output; content revision and eviction keep marks valid. Tests: `app_tests_proofs.rs: output_scrollbar_press_and_drag_redraw_and_find_index_resets_on_a_new_query`; `app_tests_parity.rs: hp14_output_streams_are_exact_and_retention_drops_are_stated` (`find "line 44" · 1 of 100` with 500 dropped lines), `hp04_…` (find in the preview); `viewport.rs: marks_paint_and_survive_eviction`; `ui/text.rs: find_preserves_smart_case_and_whole_source_graphemes`.

**Captures inspected:** `shots/h_hp14_find`.

**Deferred remainder:** Jackin panes/Inspect and the extraction of a cross-application controller (Later checkbox); regex, replacement and multiple cursors are out of scope.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
