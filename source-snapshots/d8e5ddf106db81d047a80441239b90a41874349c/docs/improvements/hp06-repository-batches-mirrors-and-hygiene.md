# HP06 — Repository batches, mirrors and hygiene

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build sibling/mirror/hygiene plans with exact targets, bounded branch review, revalidation and mixed outcomes on fixture repositories/worktrees.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real repository discovery, git operations, branch deletion and worktree/remote checks.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve sibling-repository pull/push/status, origin+GitLab mirroring and Git hygiene. Existing child plans cover only part of these outcomes.

**Source evidence and mandatory scope:** [matrix HP06](../parity/holla-parity-matrix.md#hp06--repository-batches-mirrors-and-hygiene) — `OP05`, `OP06`, `OP08`, `OP10`, `OP11`, `OP07`, `OP09`, `OP12`, `OP13`, `OP14`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Show immediate discovered repositories as resources, including legacy multi-repo threshold behavior as a fixture; new scope controls may expose one repository too. Build plans for parallel pull/push, sequential status --short and origin plus optional GitLab pushes. Label mirror scope exactly; do not call arbitrary remotes included. Offer fetch --prune and gc independently of merged-branch availability. Resolve origin/HEAD then main/master fallback; explain unavailable default.

**Architecture / reusable components:** Use canonical repository/worktree identity, per-repo remote identities and explicit concurrency/failure policies. Merged cleanup reviews sorted unique candidates, current/default exclusions and visible 30-of-N bound; execute git branch -d --, never implicit force. Revalidate branch/worktree/merge state before execution. Reuse Picker, Plan/StepRail and existing facts dialogs.

**Required deterministic fixture:** `parity-git-batch` — zero/one/many immediate repositories, colliding leaf names, failed repo, absent/present GitLab, custom default, no default, no merged branches, >30 branches, branch occupied by another worktree, stale merge eligibility.

**Acceptance / automated verification:** Assert every intended repo/remote operation and cwd, independence versus ordered status, retained per-repo failure output, no hard-coded default, exact delete argv and protected current/default branches. A failed prerequisite blocks only dependents; independent batch peers still run. Test stale confirmation revocation and Git refusal without -D fallback. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture batch selection, mirror preview, mixed results, capped branch review and stale/worktree refusal.

**Dependencies:** HP01/HP05/HP14/HP15/HP21; F02/F05/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-git-batch` (fixture fn `git_batch` in src/bin/holla/domain/parity.rs) · `hp06_sibling_batches_carry_exact_members_and_report_each_failure` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/app_tests_parity.rs: hp14_output_streams_are_exact_and_retention_drops_are_stated` (its batch-mode block runs "Push 4 repositories to origin and GitLab"), `src/bin/holla/domain/parity.rs: every_parity_world_builds_a_catalogue_with_unique_ids`. · row proofs in src/bin/holla/app_tests_rows.rs: `op07_op08_sibling_push_and_status_batches_name_every_repository`, `op03_op11_op12_current_repository_status_fetch_prune_and_gc_are_exact`.

**What the journey asserts:**
- `git.pull-all` has exactly 4 batch members and none contains "nested" (the nested `alpha` leaf-name collision is not a sibling).
- `git.push-all-remotes` argv contains `git -C /Users/alex/work/repos/alpha push gitlab @/Users/alex/work/repos/alpha` and no command containing "beta push gitlab" (only alpha has the mirror remote).
- With cwd `/Users/alex/work/repos/alpha`, `git.delete_merged` commands contain "feature/done" and "hotfix/1" and never "feature/wt" (occupied by another worktree) or " main ".
- Running "Pull 4 sibling repositories" creates a batch with `members.len() == 4`; every member finishes; the member whose command contains "/delta " is `ActivityState::Failed`; at least 2 members `Succeeded`; alpha's `behind` becomes 0.
- The frame contains "1 failed" or "failed".
- hp14: `git.push-all-remotes` batch `mode` equals the item's `batch_mode`; first member `Running`; second `Queued` with "queued" shown when sequential, `Running` when parallel.

**Captures:** `shots/h_hp06_batch` (pull batch after completion, summary with ok/failed counts), `shots/h_hp06_batch_picker` (activities picker listing each member with its state); base matrix `shots/h_parity_git_batch_{80x24,100x30,120x40,160x50,mono}`. State: provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP05 | hp06: 4 members from `location.children`, nested repo excluded. The more-than-one threshold and the zero/one-repo cases are not asserted. | real immediate `.git` discovery |
| OP06 | hp06: 4 member activities, each `git -C <repo> pull`; delta fails, alpha `behind == 0`; summary names the failure. Parallel versus sequential mode for pull is not asserted (hp14 asserts it for the mirror push). | real parallel git |
| OP08 | `op07_op08_sibling_push_and_status_batches_name_every_repository`: `git.status-all` carries exactly `git -C /Users/alex/work/repos/<repo> status --short` for alpha, beta, delta and gamma. | real sequential status |
| OP10 | Partly: hp06 excludes " main " from deletion in alpha (`origin/HEAD` main). Custom default (beta), no default (gamma) and group disappearance are not asserted. | `rev-parse`, `origin/HEAD` resolution |
| OP11 | `op03_op11_op12_current_repository_status_fetch_prune_and_gc_are_exact`: `git.fetch` argv is `git -C /Users/alex/work/svc fetch --prune`. | real fetch |
| OP07 | `op07_op08_sibling_push_and_status_batches_name_every_repository`: `git.push-all` carries exactly `git -C /Users/alex/work/repos/<repo> push` for the four repositories (parallel batch). | real push |
| OP09 | hp06: alpha carries `push gitlab`, beta does not; hp14 runs the batch and asserts member states per `batch_mode`. The "(no gitlab remote)" label is not asserted. | `remote get-url gitlab` probe |
| OP12 | `op03_op11_op12_current_repository_status_fetch_prune_and_gc_are_exact`: `git.gc` argv is `git -C /Users/alex/work/svc gc`. | real gc |
| OP13 | hp06: candidates include "feature/done" and "hotfix/1", exclude "feature/wt" and "main". The 30-of-N cap (beta has 35 merged) is not asserted for this slice. | `git branch --merged` query |
| OP14 | hp06: delete commands name the candidates; the exact argv `git branch -d -- <names>` is not asserted verbatim, nor Git refusal without `-D`. | real deletion |

**Deferred remainder:**
- Real repository discovery, git operations, branch deletion and worktree/remote checks (Later).
- Status-all ordering, fetch-prune, gc and push-all have no assertion.
- Default-branch fallback (main/master) and unavailable-default explanation are not asserted.
- Exact `git branch -d --` argv, 30-of-N branch cap, stale confirmation revocation and Git refusal without `-D` fallback are not asserted.
- Dependent-versus-independent prerequisite blocking is not asserted.

**Limits:**
- Member outcomes come from fixture `command_failure` values; no git process or remote exists.
- Worktree occupancy is a fixture list, not `git worktree list`.
