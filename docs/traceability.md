# Documentation traceability

Date: 2026-09-12 (docs consolidation on branch `holla`).

This table records where every retired or moved documentation source landed
when the repository documentation was consolidated under `docs/`. Content was
moved with `git mv` (history preserved) unless noted otherwise.

## Mapping

| Retired / moved source | Landed at | Notes |
| --- | --- | --- |
| `DESIGN.md` (root) | `docs/design/DESIGN.md` | Verbatim move. |
| `IMPROVEMENTS_PLAN.md` (root) | `docs/improvements/PLAN.md` | Verbatim move. This is the ACTIVE tracker. |
| `PLANNING_GOAL.md` (root) | `docs/sources/PLANNING_GOAL.md` | Verbatim archive of the superseded planning-phase goal; synthesized into `docs/refactoring/goal.md`. |
| `REFACTORING_COMPLETION_PLAN.md` (root) | `docs/sources/REFACTORING_COMPLETION_PLAN.md` | Verbatim archive of the top-level plan; its living summary is `docs/refactoring/plan.md`. |
| `docs/holla-parity-evidence.md` | `docs/parity/holla-parity-evidence.md` | Verbatim move. |
| `docs/holla-parity-matrix.md` | `docs/parity/holla-parity-matrix.md` | Verbatim move. |
| `docs/holla-parity-verification.md` | `docs/parity/holla-parity-verification.md` | Verbatim move. |
| `docs/holla-preview-inventory.md` | `docs/parity/holla-preview-inventory.md` | Verbatim move. |
| `docs/tui-audit.md` | `docs/audits/tui-audit.md` | Verbatim move. |
| `docs/tui-audit-editor.md` | `docs/audits/tui-audit-editor.md` | Verbatim move. |
| `docs/tui-audit-interactions.md` | `docs/audits/tui-audit-interactions.md` | Verbatim move. |
| `docs/tui-audit-inventory.md` | `docs/audits/tui-audit-inventory.md` | Verbatim move. |
| `docs/tui-audit-research.md` | `docs/audits/tui-audit-research.md` | Verbatim move. |
| `docs/tui-audit-verification.md` | `docs/audits/tui-audit-verification.md` | Verbatim move. |
| `docs/plan-api-verification.md` | `docs/plan-verification/plan-api-verification.md` | Verbatim move. |
| `docs/plan-design-verification.md` | `docs/plan-verification/plan-design-verification.md` | Verbatim move. |
| `docs/plan-ecosystem-research.md` | `docs/plan-verification/plan-ecosystem-research.md` | Verbatim move. |
| `docs/plan-final-review.md` | `docs/plan-verification/plan-final-review.md` | Verbatim move. |
| `docs/plan-interaction-verification.md` | `docs/plan-verification/plan-interaction-verification.md` | Verbatim move. |
| `docs/plan-prior-fixes-verification.md` | `docs/plan-verification/plan-prior-fixes-verification.md` | Verbatim move. |
| `docs/plan-terminal-verification.md` | `docs/plan-verification/plan-terminal-verification.md` | Verbatim move. |
| `docs/plan-text-verification.md` | `docs/plan-verification/plan-text-verification.md` | Verbatim move. |
| `docs/improvements-plan-reference.md` | `docs/improvements/plan-reference-audit.md` | Verbatim move + rename; historical annex of `docs/improvements/PLAN.md`. |
| `holla-project/CONCEPT.md` | `docs/product/CONCEPT.md` | Verbatim move; `holla-project/` directory removed. |
| `holla-project/GOAL.md` | `docs/product/GOAL.md` | Verbatim move. |
| `holla-project/PROMPT.md` | `docs/product/PROMPT.md` | Verbatim move. |
| `holla-project/notes/` (6 files: 00-decisions, 01-concept-constraints, 02-interaction-models, 03-recipe-audit, 04-design-note, 05-visual-critique) | `docs/product/notes/` | Verbatim move. |
| `holla-project/references/` (6 files incl. its README.md) | `docs/product/references/` | Verbatim move. |
| `main:COMPONENT_ARCHITECTURE.md` | `docs/sources/main/COMPONENT_ARCHITECTURE.md` | Verbatim main-branch snapshot (new file, via `git show main:...`). |
| `main:CONTINUE_PROMPT.md` | `docs/sources/main/CONTINUE_PROMPT.md` | Verbatim main-branch snapshot. |
| `main:COORDINATION.md` | `docs/sources/main/COORDINATION.md` | Verbatim main-branch snapshot. |
| `main:GOAL.md` | `docs/sources/main/GOAL.md` | Verbatim main-branch snapshot. |
| `main:GOAL2.md` | `docs/sources/main/GOAL2.md` | Verbatim main-branch snapshot. |
| `main:HANDOFF_SLICE4_WAVE1.md` | `docs/sources/main/HANDOFF_SLICE4_WAVE1.md` | Verbatim main-branch snapshot. |
| `main:REFACTORING_GOAL.md` | `docs/sources/main/REFACTORING_GOAL.md` | Verbatim main-branch snapshot (byte-identical to v6 `d4715f8e`). |
| `main:REFACTORING_STATE.md` | `docs/sources/main/REFACTORING_STATE.md` | Verbatim main-branch snapshot. |
| `main:RESUME_PROMPT.md` | `docs/sources/main/RESUME_PROMPT.md` | Verbatim main-branch snapshot. |
| `REFACTORING_GOAL.md` — the 6 historical versions `e48137f1`, `dce91d51`, `2d81eec4`, `25ea92b0`, `efada044`, `d4715f8e` | Git history, commits `e48137f1..d4715f8e`; analyzed in `docs/refactoring/goal-evolution.md` (not duplicated) | Only the final main version is snapshotted at `docs/sources/main/REFACTORING_GOAL.md`. |
| `REFACTORING_STATE.md` — 87-commit history on `main` (`git log --follow main -- REFACTORING_STATE.md`) | Analyzed in `docs/refactoring/goal-evolution.md` (§2) + final main snapshot at `docs/sources/main/REFACTORING_STATE.md` | Intermediate states are not duplicated. |

`README.md` remains the only Markdown file at the repository root; only its
links were updated.

## `docs/sources/main/*` are verbatim snapshots

Every file under `docs/sources/main/` is a byte-verbatim snapshot of the
named file on branch `main`, materialized with `git show main:<FILE>`. Their
content is historical: intra-main cross-links and paths they cite (e.g.
`baseline/`, `parity/`, `crates/`, `docs/historical-regeneration-approval.md`,
sibling `GOAL.md`/`REFACTORING_STATE.md` references) refer to **main's tree**,
not this branch, and are intentionally left unresolved here.

## References not updated (sealed task packages)

The following files under `refactoring-tasks/` still cite retired paths
(`IMPROVEMENTS_PLAN.md`, `docs/improvements-plan-reference.md`,
`REFACTORING_COMPLETION_PLAN.md`, `PLANNING_GOAL.md`, `DESIGN.md`,
`holla-project/...`). They are sealed task packages and were NOT modified;
the citations refer to the paths as they existed at package-creation time:

- `refactoring-tasks/README.md`
- `refactoring-tasks/terminal-components/README.md`
- `refactoring-tasks/terminal-components/completion/069/README.md`
- `refactoring-tasks/terminal-components/completion/{009,010,012,013,016,017,020,021,022,023,024,025,026,028,030,031,069}/trusted/obligations.md`
- `refactoring-tasks/terminal-components/completion/{001,009,010,012,013,016,017,020,021,022,023,024,025,026,028,030,031,032,033,039,040,041,042,043,044,045,046,047,048,049,050,051,055,056,057,058,059,060,064,065,066,067,068,069,070}/trusted/source-obligations.tsv`

Equivalents under the new layout: `IMPROVEMENTS_PLAN.md` →
`docs/improvements/PLAN.md`; `docs/improvements-plan-reference.md` →
`docs/improvements/plan-reference-audit.md`; `REFACTORING_COMPLETION_PLAN.md` →
`docs/sources/REFACTORING_COMPLETION_PLAN.md`; `PLANNING_GOAL.md` →
`docs/sources/PLANNING_GOAL.md`; `DESIGN.md` → `docs/design/DESIGN.md`;
`holla-project/...` → `docs/product/...`.

## Historical evidence left verbatim by design

`docs/refactoring-plan/**` (ledgers, re-audit registers, branch-diff history)
cites old doc paths — including `docs/tui-audit-*.md`,
`docs/holla-parity-*.md`, `docs/improvements-plan-reference.md` and
`holla-project/...` — as records of paths and blobs at specific commits.
Those citations are historical evidence, not navigation links, and were left
untouched. Live navigation links in that tree that pointed at moved files
were retargeted (`docs/refactoring-plan/planning-acceptance.md` →
`docs/sources/...`).
