# Branch census vs `parity/termrock-visual-baseline` — 2026-10-21

Read-only census at parity tip `3196f4ef6` (143 remote branches, 137 with commits not on parity by `git rev-list --count origin/parity/termrock-visual-baseline..<branch>`). This records the triage; it merges nothing and deletes nothing. Landing or retiring any branch still requires its own review and gate evidence per the migration matrix.

## Excluded-by-design (recorded, never merged wholesale)

- `design/interaction-polish` (1064 ahead) — fixed reference; PR #26 at `f4867e9b`. Only individually ported contracts belong in parity (many already ported; ports tracked in the matrix).
- `ci/design-rust-timeout-20261020`, `ci/zizmor-design-20261020`, `parity/velnor-design-upgrade-20261020`, `docs/interaction-reference-cleanup`, `wip/reach-pages-a*`, `wip/rapps-3-7-14`, `wip/s14-followups`, `wip/plan-s16`, `wip/v1-showcase`, `wip/shared-design-working-tree-20261020`, `docs/branch-authority-cleanup`, `chore/rust-comment-cleanup`, `codex/*`, `preservation/*`, `audit/*` — design-side, preservation, or superseded rescue history; excluded by earlier recorded decisions in the matrix and handoff. Deletion only after final consolidation review.
- `ci/design-rust-timeout`/`zizmor` design-side CI changes were authorized onto `main` via CI-only PR #39, not parity (see matrix).

## Active WIP — owners still landing or holding for review

High-churn concurrent-session lanes (unmerged counts move daily): `wip/session-consolidation` (58), `wip/holla-home-migration` (51), `wip/showcase-pages-migration` (50), `parity/strict-accounting-fix` (34), `parity/runtime-fields-binding-query-current` (34), `parity/runtime-fields-integration` (28), `wip/owned-fresh-plan-lifecycle` (24), `wip/table-consumer-baseline` (15), `parity/binding-runtime` (15), `parity/acceptance-architecture-module-path` (15), plus the 1–13 commit families (`wip/*-20261021`, `parity/*-20261020`). The concurrent session retires these atomically with reviewed integrations (precedent: nine-ref atomic retirement recorded in the matrix).

Coordinator-owned rate-limit branches (protected state, already superseded where noted):

- `wip/jackin-rate-limit-20261021` (1) — superseded: Jackin batches landed through later tips; keep until usage overview/detail lane completes, then retire.
- `wip/holla-rate-limit-20261021` (7) — partially superseded by the concurrent session's holla landings; reconcile before retiring (commit `f01361f26` from this line is preserved on parity as `9fc87cf1c`).
- `wip/palette-rate-limit-20261021` (1), `wip/controls-adopt-rate-limit-20261021` (1) — superseded by landed palette (`4a0f8da7f` line) and controls/gutter matrix (`6fb70e629`, `423b0e5e4`); retire at consolidation.

## Merge-eligibility still owed (each needs fresh review + gate at current tip)

`wip/external-public-consumer-20261021` (2) and `wip/public-consumer-contracts-20261021` (1) — closest to the missing external public-API consumer gate; review before the §8 demonstration.
`wip/branch-consolidation-20261021` (7) — prior consolidation plan; supersede or execute it at the final checkpoint.
`wip/nextest-doc-inventory-guard-20261021`, `wip/ci-doc-inventory-20261021` (0 ahead — already merged or empty of unique commits).
`wip/pr38-completion-verify-20261021` (3) — completion-verification helper; evaluate with the final report.

## Rule applied

No branch is closed or deleted while its records are unresolved; superseded branches are closed only after their integration is recorded in the migration matrix, at the final consolidation checkpoint (task §"Branches"). Counts above are an observation, not acceptance evidence.
