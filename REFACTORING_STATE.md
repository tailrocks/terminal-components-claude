# Refactoring State

## Status

- Overall: Slice 1 audits complete; Slice 2 architecture synthesis running (Opus)
- Slice: 1→2

## Baseline

- Revision: d5e7075 (main). Worktree clean except untracked-noise `.DS_Store`.
- Toolchain: cargo/rustc 1.98.0; package `junie-tui` edition 2024, MSRV 1.88; ratatui 0.30 (crossterm_0_29), unicode-width 0.2, unicode-segmentation 1.
- Gates before refactor (logs in target/baseline/*.log):
  - `cargo fmt --check` exit 0
  - `cargo clippy --all-targets -- -D warnings` exit 0, 0 warnings
  - `cargo test --all-targets` exit 0 — 76 (lib) + 63 (jackin) + 26 (tablepro) + 33 (showcase) = 198 passed, 0 failed
  - `cargo build --bins` exit 0
- Pre-existing failures: none.
- Source size: ~72K lines; lib ~16K (core/ui/theme/runtime/widgets), showcase ~8K, tablepro ~12K, jackin ~35K.
- Before-refactor captures: in progress (baseline/before/), existing shots/ f_* and j_* retained.

## Assignments

- Coordinator: `refactor-coordinator` (`claude-fable-5-1`, high) — this session.
- Implementation: `fable-builder` (`claude-fable-5-1`, high).
- Research/review: `opus-analyst` (`claude-opus-5`, high, read-only).
- Slice 1 Opus audits: docs/audit/app-audit.md, domain-boundary-audit.md, interaction-audit.md, architecture-research.md (done, committed); api-audit.md (done, committed); performance-audit.md (running).

## Accepted decisions

- None recorded yet (awaiting Slice 1 research).

## File ownership

- fable-builder "baseline-capture": baseline/before/**, tools/** (capture additions only). No src/ edits.

## Completed gates

- Slice 1 baseline commands: done (see Baseline).

## Unresolved findings

- Library domain leaks (grep sweep, coordinator): src/widgets/grid.rs (105 SQL/NULL/PK/FK hits), src/widgets/dialog.rs:21 ("(SQL)" doc), src/widgets/statusbar.rs:335 (jackin label in test). Apps: 82 `.owns(`/`.locate` call sites.
- Research conflict to adjudicate (Opus synthesis): interaction-audit B2/B3 proposes retained handle/render split with runtime-delivered `Event` + `Response{flow,invalidate,action}`; architecture-research §2 proposes immediate-mode `show(ui, area)` with intent queue drained during show. Must weigh Scenario A ergonomics vs migration of ~55K lines of app code and deterministic test harnesses.
- app-audit agent reported no grep/glob; its counts are lower bounds over files read in full.

## Next action

- Record Opus synthesis as COMPONENT_ARCHITECTURE.md; record baseline captures manifest and performance audit; then fresh Opus API-review of the architecture (Slice 2 critique) before prototype.
