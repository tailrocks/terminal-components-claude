# Refactoring state — both branches, honest snapshot

Provenance: local branches `holla` (HEAD `b6b0d3c2`, working tree clean) and `main` (HEAD `7b27732a`), merge-base `cc14dd6`; main's ledgers `REFACTORING_STATE.md` (final checkpoints 2026-09-08/09), `docs/audit/consolidation/disposition-ledger.md`, `HANDOFF_SLICE4_WAVE1.md`; holla's `REFACTORING_COMPLETION_PLAN.md`, `docs/refactoring-plan/*`, `docs/holla-parity-*.md`; companion file `docs/refactoring/main-port-analysis.md`. Date 2026-09-12.

## Done on holla (this branch)

| Item | Evidence |
|---|---|
| Product line intact and evolved: root library `src/{core,ui,widgets}`, runtime, theme + four apps under `src/bin/{showcase,tablepro,jackin_preview,holla}` | working tree `src/` |
| Truthful scrolling at the model boundary, all containers refactored onto it | `02f5294b` (also the immutable UI oracle tag `holla-fable-2026-09-10`) |
| Scroll-edge fades for scrollable content | `92d91629` |
| One-click focus-and-edit for text fields; click-to-edit grid cells | `e4866ce4`, `53b8212f` |
| Holla app: parity journeys HP01–HP23 with the defects they surfaced; typed execution contract; world-owned cleanup; incremental output index | `11ae5078`, `4399e9dd`, `1aaa9b01`, `812bad84`, `b5f237d4`, `1615aa56` |
| Runtime owns job-control suspension, owned-PTY proof | `03df2681`, test `tests/terminal_suspend.rs`, `tests/holla_pty.rs` |
| Widget identity work: revisioned viewport identity, picker eligibility, tree focus identity | `c9710d81` |
| Capture tooling: one grapheme/cell reference for HTML+PNG with provenance | `8f7ed0e1`; `tools/ansi2html.py`, `ansi2png.py`, `cells.py`, `capture.sh`, `fidelity_check.py` |
| Legacy-holla capability audit: 226-row matrix (0 equivalent / 3 redesigned / 115 partial / 98 missing / 10 n/a), independently verified **as a plan** | `docs/holla-parity-matrix.md`, `docs/holla-parity-verification.md` |
| Exhaustive main-vs-holla diff analysis: branch inventories, BF01–BF14 library defect list, 620-clause obligation matrix, architecture adjudications ADJ-09–14, 73-package task catalog | `docs/refactoring-plan/branch-diff-*.md`, `historical-obligations-canonical.tsv`, `architecture-adjudication.md`, `refactoring-tasks/` |
| Refactoring completion plan with resolved source identities and topology | `REFACTORING_COMPLETION_PLAN.md`, `PLANNING_GOAL.md` |

## Done on main only (not on holla)

| Item | Evidence | Caveat |
|---|---|---|
| Workspace cut: `crates/tui` (`junie-tui`), `crates/tui-testing`, `xtask`, four `apps/*` packages; strict lint table; MSRV 1.88 gate | `main:Cargo.toml`, `main:.github/workflows/ci.yml` | structure proven; member layout differs from what holla's plan assumes |
| Complete component architecture (Slices 1–4): ~40 components, conformance driver, render digest matrix, perf harness; lib 762 + conformance 934 green at last measurement | `HANDOFF_SLICE4_WAVE1.md` stop update; `crates/tui/` tree | green against **self-generated** baselines; BF01–BF14 source-proven deviations from oracle behavior |
| Thread/process-safe baseline blessing (race structurally closed, two-way byte-identical proof) | `2a9316c0`; `crates/tui-testing/src/digest.rs` | none — clean port candidate |
| Runtime correctness cluster (publish-after-output, focus/hover/pointer fixes) | `2caff455`, `493eacc7`, `5eb277c6`, `577bf533`, `715ee077`, `f5c37635`, `cdbcb053` | each with regression tests |
| Component features: grid gutters/header prefixes, picker semantic columns, logical split seam, scrollbar visibility, badge recipe, tree disclosure | `aacb7cb3`…`30924105`, `e9364b14`/`21811265`, `c562ee53`/`ec2ef742`/`03e7cf93`, `da4b1364`, `42d3a686`, `0d676b1b` | deliberately visible; digest-classified on main, not oracle-approved |
| Gate toolchain: xtask boundary/bless-guard/doc-check/capture-contract, parallel parity replay, literal masking | `c3fe5049`, `f2860496`, `da92e485`, `7bd88314` | bless-guard fails closed without a base ref |
| Historical evidence frozen: 499 pre-refactor captures + recipes + manifest | `main:baseline/before/`, `main:parity/recipes.tsv` | parity replay against it is red (below) |
| One full main+holla integration round (PR #1) incl. holla app migration to 11 worlds/journeys | merge `7b27732a`; `docs/plans/main-holla-integration.md` | integration reference was holla `794b095c`; holla's 41 later commits are not represented |
| Workspace/branch cleanup: 233 worktrees dispositioned, 91 stale remote branches deleted | `docs/audit/consolidation/disposition-ledger.md` | 6 worktrees REJECTED with root causes (manager rewrite, item-row part-patch threading, showcase WIP fixtures) |

## Open / unfinished (either branch)

| Item | State | Evidence source |
|---|---|---|
| `parity_contract` (499 historical captures vs current rendering) | **Red at main HEAD, honestly registered**: 60 exact / 379 mismatch / 60 run failures; all five repair paths refused; reconciliation deferred to "a successor effort" — holla's campaign is that successor | main `REFACTORING_STATE.md` consolidation checkpoint 2026-09-09 |
| Oracle parity of apps: Showcase 22/23 pages (Diff missing), Holla 11/34 worlds, Jackin simplified Settings/Usage + mixed historical painters, TablePro reduced query editor and unmounted overlays | gaps measured, not repaired | holla `REFACTORING_COMPLETION_PLAN.md` §7 |
| BF01–BF14 behavioral defects inside main's library (text buffer, graphemes, fuzzy, wrap, panic cursor restore, split axis, paint clusters, keymap shift, input re-delivery, style-resolution authority, ascii glyphs, disabled-hover) | source-proven counterexamples; no fixes landed anywhere | holla `docs/refactoring-plan/branch-diff-foundation.md` |
| Holla-side features absent from main's component line: truthful scroll model, scroll-edge fades, one-click editing, HP01–HP23 journeys | done on holla only; conflict zone for any port of main's ScrollRegion/input/list code | holla log `main..holla` |
| Production refactor execution on holla: re-audit reopened; task catalog status `pending`; "production refactor and production verification harness have not been implemented or executed" | not started (planning gate not accepted) | `REFACTORING_COMPLETION_PLAN.md` header, `refactoring-tasks/README.md` |
| tui-snap verification tool: PR #1 (`883d03f1`) reviewed but unmerged upstream | external blocker for the visual-verification contract | `REFACTORING_COMPLETION_PLAN.md` §3 |
| Main's leftover obligations: Slice-5 showcase Buttons merge (P1), `FrameServices` undeclared, `Anchor::Point` visual-changes entry, Slice 8 reviews, remaining mono-narrowing reasons (Q) | open on main; would transfer to holla's campaign | main `REFACTORING_STATE.md` unresolved findings |
| Main CI self-maintenance: 316/316 recorded runs red at MSRV compile before the consolidation repair | repaired at `aebe6e5e`/merge; evidence of how drift accumulated | main `REFACTORING_STATE.md` consolidation checkpoint |

## Reading guide

- "Done on main" means *present and gate-measured on main*, not *correct against the pre-refactor UI* — main's own GOAL.md disclaims that proof, and its app baselines were re-blessed to current behavior (`0191acbf`).
- "Done on holla" product behavior is the acceptance oracle (tag `holla-fable-2026-09-10` = `02f5294b`); any ported code must reproduce it exactly.
- Next step per holla's plan: accept the re-audit, then execute the task catalog; `docs/refactoring/main-port-analysis.md` names the concrete port candidates and their risk classes.
