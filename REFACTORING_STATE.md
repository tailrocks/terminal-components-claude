# Refactoring State

## Status

- Overall: Slice 1 in progress — baseline and audit
- Slice: 1 — baseline and audit

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
- Slice 1 Opus audits (running): api-audit, app-audit, domain-boundary-audit, interaction-audit, architecture-research. Results to be recorded under docs/audit/.

## Accepted decisions

- None recorded yet (awaiting Slice 1 research).

## File ownership

- fable-builder "baseline-capture": baseline/before/**, tools/** (capture additions only). No src/ edits.

## Completed gates

- Slice 1 baseline commands: done (see Baseline).

## Unresolved findings

- None known.

## Next action

- Collect Opus audit results, save to docs/audit/, synthesize into COMPONENT_ARCHITECTURE.md draft; complete baseline captures.
