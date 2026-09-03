# Refactoring State

## Status

- Overall: Slice 2 — COMPONENT_ARCHITECTURE.md complete (20 sections + slice plan + package layout), under fresh Opus critique
- Slice: 2 — architecture review and representative prototype

## Baseline

- Revision: d5e7075 (main). Worktree clean except untracked-noise `.DS_Store`.
- Toolchain: cargo/rustc 1.98.0; package `junie-tui` edition 2024, MSRV 1.88; ratatui 0.30 (crossterm_0_29), unicode-width 0.2, unicode-segmentation 1.
- Gates before refactor (logs in target/baseline/*.log):
  - `cargo fmt --check` exit 0
  - `cargo clippy --all-targets -- -D warnings` exit 0, 0 warnings
  - `cargo test --all-targets` exit 0 — 76 (lib) + 63 (jackin) + 26 (tablepro) + 33 (showcase) = 198 passed, 0 failed
  - `cargo build --bins` exit 0
- Pre-existing failures (from captures, see baseline/before/NOTES.md):
  1. PANIC jackin-preview View→Container info: src/bin/jackin_preview/screens/capsule.rs:1183 `&i.run_id.replace('-', "")[..8]` on 7-byte id (unguarded byte slice). Fix in Slice 7 (structural: width-safe truncation helper).
  2. jackin `Ctrl+B i` listed in View menu but prefix handler rejects it (menu/binding drift — the B9 binding-table design removes the class).
  3. `Ctrl+\` palette chord undeliverable via tmux legacy encoding (harness limitation, not app bug).
  4. jackin F10 reopens last-used menu, not File (semantics decision deferred to Slice 7 Opus review).
  5. TablePro has no menu bar (by design today).
- Source size: ~72K lines; lib ~16K (core/ui/theme/runtime/widgets), showcase ~8K, tablepro ~12K, jackin ~35K.
- Before-refactor captures: baseline/before/ — 499 captures (208 showcase, 108 tablepro, 183 jackin) as .ansi/.txt/.cursor (committed) + .html/.png (gitignored, regenerable via `tools/baseline_capture.sh all`); MANIFEST.md has exact key/mouse recipes; NOTES.md findings.

## Assignments

- Coordinator: `refactor-coordinator` (`claude-fable-5-1`, high) — this session.
- Implementation: `fable-builder` (`claude-fable-5-1`, high).
- Research/review: `opus-analyst` (`claude-opus-5`, high, read-only).
- Slice 1 Opus audits: docs/audit/app-audit.md, domain-boundary-audit.md, interaction-audit.md, architecture-research.md (done, committed); api-audit.md, performance-audit.md (all six done, committed).

## Accepted decisions

- COMPONENT_ARCHITECTURE.md (commit after cefc4b8) is the accepted architecture, pending Slice 2 critique edits. Adjudications recorded there: A retained state + props + explicit update/draw phases with pre-resolved intents (immediate-mode `show` rejected); B unified Id (FNV with separator+kind, ItemKey, typed Part, PartRef in registry); C `Response<A>{flow,invalidate,state,action}`; D tokens+recipes+StylePatch+overlay scopes, precedence 1–6, mono fallback rule; E layer stack/focus scopes/capture/wheel/cursor/invalidation; F workspace, crate name stays `junie-tui`, `author` module; G keyed collection vocabulary, DataTable removed, Grid split via GridModel/GridEditor/GridCellActions; H Field/TextEditorCore/BlurPolicy/Validate/Secret; I dispositions + J1–J13.
- §20.9 folds performance-audit R1–R7 and §6.3 as binding amendments (16 items) with named acceptance tests.
- §20.10 lists 14 intentional visual changes; each requires a docs/visual-changes.md entry before any baseline regeneration.
- Appendix A amends goal §27 Slice 4: family builders do not touch apps; showcase migration is entirely Slice 5 (disjoint ownership).
- Coordinator staging proposal (awaiting Opus confirmation in Slice 2 review): keep old root package compiling as `junie-tui-legacy` during Slices 3–4 so apps + 198 tests stay green until migrated.

## File ownership

- fable-builder "baseline-capture": baseline/before/**, tools/** (capture additions only). No src/ edits.
- fable-builder "perf-baseline": tests/perf.rs, tests/perf_common.rs, tests/perf_baseline.txt, src/bin/*/perf_tests.rs (+ one `mod perf_tests;` line per main.rs). No library edits.

## Completed gates

- Slice 1 baseline commands: done (see Baseline).

## Unresolved findings

- Library domain leaks (grep sweep, coordinator): src/widgets/grid.rs (105 SQL/NULL/PK/FK hits), src/widgets/dialog.rs:21 ("(SQL)" doc), src/widgets/statusbar.rs:335 (jackin label in test). Apps: 82 `.owns(`/`.locate` call sites.
- Research conflict to adjudicate (Opus synthesis): interaction-audit B2/B3 proposes retained handle/render split with runtime-delivered `Event` + `Response{flow,invalidate,action}`; architecture-research §2 proposes immediate-mode `show(ui, area)` with intent queue drained during show. Must weigh Scenario A ergonomics vs migration of ~55K lines of app code and deterministic test harnesses.
- app-audit agent reported no grep/glob; its counts are lower bounds over files read in full.

## Next action

- Apply Slice 2 critique edits to COMPONENT_ARCHITECTURE.md; commit perf baseline (WP-0); then start Slice 3 foundations builder per Appendix A.
