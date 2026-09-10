# F19 — Keyboard selection in the existing TextViewport

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete shared TextViewport keyboard selection and Holla Activity/Plan/preview integration, preserving Diff delegation and existing mouse behavior.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No separate viewer or full terminal emulator.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed coverage gap.** `viewport.rs:563` scrolls/follows/copies/clears;
Shift+Right does nothing and Shift+Down scrolls without selection. Range creation
is mouse-only. Diff delegates to the same handler. Design, interaction, API and
research agents independently checked the absence and present consumers: Holla
output, Jackin panes/Inspect, Showcase Terminal/Diff.

Add an explicit keyboard browse/selection caret, distinct from producer-owned
terminal caret. Reuse logical CellPos, selection/copy events and visual mapping.
Specify entry/exit, anchor extension, word/line/document movement, follow pause,
offscreen autoscroll and hints before implementation. Follow F10–F14 ownership
work; do not introduce another viewer or duplicate Diff selection state.

Risk: medium/high interaction semantics. Acceptance: keyboard and mouse copy
identical partial/word/multiline/wrapped Unicode ranges; preserve source identity
under resize, append and retention. Escape clears selection before exiting the
owner; no browse keys leak into attached terminal input. Cover empty/single-line,
no-selection copy, monochrome reversal and hardware-cursor policy. Plain scrolling
keeps existing behavior outside explicit selection mode.

## Evidence

**Slice status:** current shared slice complete.

**Shared widget:** Shift+arrows extend a keyboard selection from an explicit caret (Ctrl/Alt by word, Ctrl+Shift+Home/End to the document ends), Esc clears it before leaving the owner, selection pauses follow and survives append and resize, and keyboard and mouse copy identical ranges (partial, word, multiline, wrapped Unicode). Tests: `viewport.rs: keyboard_selection_matches_mouse_selection`, `keyboard_selection_pauses_follow_and_survives_append_and_resize`, `drag_selects_and_copies_text`; `tests/focus_gutter.rs: text_selection_uses_reverse_video_only_in_monochrome`.

**Holla integration:** activity output (`activity.rs: view.on_key`), plan output (`plan.rs: view.on_key`) and the files preview (`files.rs: preview.on_key`) forward keys to the viewport when it owns focus; `y` copies the selection or the whole retained output through the simulated clipboard (`Go::Osc52`), and Linux without OSC 52 refuses with a reason. Tests: `app_tests_parity.rs: hp14_…` (copy), `hp23_…` ("does not accept OSC 52"); activity input mode never forwards browse keys to stdin (`hp15_…`).

**Captures inspected:** `shots/h_hp14_find`, `shots/h_hp15_input_mode`.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
