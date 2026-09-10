# HP05 — Current-repository Git operations

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Provide full status, strategy review, exact repository/cwd/argv and distinct success/failure effects in the simulated Git world.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real git invocation, credentials, remotes and repository integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve current Git pull, push and full status. Preview pull is ff-only, status is a snapshot and push has no modeled effect.

**Source evidence and mandatory scope:** [matrix HP05](../holla-parity-matrix.md#hp05--current-repository-git-operations) — `OP01`, `OP02`, `OP03`, `OP04`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** A Git resource exposes full status and synchronization alternatives with effective repository/cwd. Retain fast-forward default while offering the ordinary configured pull behavior after explicit conflict/rebase/merge review; do not silently remove configurations that old git pull supports. Push must change modeled remote state and report rejection/auth/upstream failures. Status retains the detail users obtain from full git status.

**Architecture / reusable components:** Separate repository identity, operation argv and result/effect; remove success-only generic fallbacks for these actions. Reuse snapshot Props/TreeView and activity TextViewport; no Git-specific shared widget.

**Required deterministic fixture:** `parity-git-current` — .git file/dir, subdirectory context distinction, clean/dirty/behind/diverged/detached/rebase, no upstream, rejected push, merge-configured pull and command failure.

**Acceptance / automated verification:** Assert correct repository/cwd for discovery and invocation, preview/argv equality, no push effect on failure and observed state after success. Assert full status detail and truthful unavailable-state recovery. New scope expansion may extend local marker discovery without dropping the old current-folder route. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture full status, pull strategy review, push rejection and successful local/remote before-after state.

**Dependencies:** HP01/HP14/HP15/HP17; F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-git-current` (fixture fn `git_current` in src/bin/holla/domain/parity.rs) · `hp05_git_current_runs_at_the_repository_root_with_truthful_outcomes` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/parity.rs: every_parity_world_builds_a_catalogue_with_unique_ids`. · row proofs in src/bin/holla/app_tests_rows.rs: `op03_op11_op12_current_repository_status_fetch_prune_and_gc_are_exact`.

**What the journey asserts:**
- With cwd `/Users/alex/work/svc/src/api`, `.git` as a file, `pull_config = "merge"`, diverged (behind 2, ahead 1) and one modified file: the `git.pull` item's argv is `git -C /Users/alex/work/svc pull @/Users/alex/work/svc`, its effect is `Effect::GitPullMerge("/Users/alex/work/svc")` and its effects text contains "blocked · 1 modified".
- "Pull with merge" runs `git -C /Users/alex/work/svc pull --no-rebase @/Users/alex/work/svc`; on success `behind == 0`, `ahead == 2`, not diverged; on failure output contains "overwritten" or "error" and `behind` stays 2.
- "Push" runs `git -C /Users/alex/work/svc push @/Users/alex/work/svc`, settles `ActivityState::Failed`, output contains "fetch first" or "rejected".
- "Push dry run" runs `git -C /Users/alex/work/svc push --dry-run @/Users/alex/work/svc` and settles `ActivityState::Succeeded`.
- "Switch to main" runs `git -C /Users/alex/work/svc switch main @/Users/alex/work/svc`; on success `branch == Some("main")`, otherwise output has "overwritten"/"error" and `branch` stays `Some("feature/x")`.

**Captures:** `shots/h_hp05_pull_blocked` (Pull row "blocked · 1 modified" with strategy alternatives in the preview), `shots/h_hp05_merge` (pull with merge activity, git's refusal over the modified file), `shots/h_hp05_push_rejected` (push failed with "[rejected]" and "fetch first"); base matrix `shots/h_parity_git_current_{80x24,100x30,120x40,160x50,mono}`. State: provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP01 | hp05: `git.pull` argv `git -C /Users/alex/work/svc pull`, effect `GitPullMerge`, "blocked · 1 modified"; "Pull with merge" argv `pull --no-rebase` with world effect (`behind` 0, `ahead` 2) or unchanged `behind` 2 on refusal. | real git, credentials, remotes |
| OP02 | hp05: push argv `git -C /Users/alex/work/svc push`, state `Failed`, "fetch first"/"rejected"; dry run argv `push --dry-run` succeeds. A successful push changing remote state is not exercised (the fixture push is rejected). | real push, auth, upstream |
| OP03 | `op03_op11_op12_current_repository_status_fetch_prune_and_gc_are_exact`: `git.status` argv is `git -C /Users/alex/work/svc status @/Users/alex/work/svc` (full status, no `--short`). | real `git status` |
| OP04 | hp05: cwd is the subdirectory `src/api` and every argv targets the root `/Users/alex/work/svc`; fixture sets `git_file = true`. Git-on-PATH gating and hidden group are not asserted. | PATH probe |

**Deferred remainder:**
- Real git invocation, credentials, remotes and repository integration (Later).
- Full `git status` detail and truthful unavailable-state recovery are not asserted.
- Push success changing modeled remote state is not exercised; only the rejected path is.
- No-upstream, detached and in-progress rebase fixture states named in the contract are not in `git_current`.
- Discovery via a `.git` file is fixture input, not a separately asserted outcome.

**Limits:**
- Outcomes come from the simulated git world; the merge/switch branches accept either result and assert the matching world state.
- No process, network or credential path exists in the test.
