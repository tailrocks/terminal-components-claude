Consolidate every existing worktree and its meaningful changes into the open PR branch:

https://github.com/donbeave/terminal-components-claude/pull/1

Target integration branch:

`codex/main-holla-integration`

The final repository state must contain only:

* `main`
* `codex/main-holla-integration`

and only the worktrees required for those branches. Every other worktree must be fully analyzed, dispositioned, and removed.

## Core objective

We have a very large number of worktrees containing changes produced during previous implementation, repair, review, verification, and experimentation passes.

Do NOT blindly merge them.

For every worktree, determine:

1. What changes it contains.
2. Why those changes were introduced.
3. Whether equivalent or superseding changes already exist in `codex/main-holla-integration`.
4. Whether the change is still correct given the repository's current architecture and goals.
5. Whether integrating it improves correctness, architecture, TUI/TUX fidelity, performance, tests, maintainability, or completeness.
6. Whether it conflicts semantically with newer/better changes.
7. Whether it exposes a real defect in the integration branch even if the worktree's exact implementation should not be reused.

Only integrate changes that are demonstrably beneficial.

Reject changes that are stale, redundant, superseded, incorrect, incomplete, architecture-regressing, or otherwise worse than the target branch.

## Use subagents aggressively

Use subagents for essentially the entire task: inventory, Git archaeology, comparison, architectural review, implementation, conflict resolution, testing, and independent verification.

Maximize safe parallelism.

Do not perform the worktree analysis sequentially when independent work can happen concurrently.

For EACH worktree, ensure it receives multiple independent reviews before its disposition is finalized.

At minimum, cover these perspectives:

### Agent A — provenance/diff analysis

Determine:

* HEAD commit
* branch/detached state
* merge base with `codex/main-holla-integration`
* commits unique to the worktree
* files and components changed
* semantic purpose of those changes
* relationship to other worktrees
* whether the commits are already reachable, cherry-picked, rewritten, or otherwise represented in the integration branch

### Agent B — correctness/relevance review

Independently determine:

* whether the change is still desirable
* whether it satisfies the current architecture
* whether it improves the target branch
* whether a better implementation already exists
* whether the original change was only an experiment/review/proof artifact
* whether accepting it would regress another area

### Agent C — integration/verification review when warranted

For non-trivial candidates, independently inspect:

* conflict resolution
* affected tests
* runtime behavior
* TUI/TUX behavior
* architecture boundaries
* API compatibility
* performance implications
* interaction with changes already accepted from other worktrees

Agents should challenge each other's conclusions. Do not accept a change merely because the agent that originally examines it recommends it.

Identical worktrees/commits may be grouped for expensive semantic analysis only after provenance agents establish that they are genuinely equivalent. Every worktree must still receive an explicit disposition.

## Establish ground truth first

Before modifying anything:

1. Read repository instructions and relevant architecture/design documentation.
2. Inspect PR #1 and `codex/main-holla-integration`.
3. Fetch current refs.
4. Build the complete worktree inventory using Git itself, not only the provided list.
5. Build the branch/commit graph.
6. Identify:

   * duplicate HEADs
   * ancestor/descendant relationships
   * detached review worktrees
   * prunable worktrees
   * implementation branches
   * review/proof worktrees
   * branches already incorporated into the PR
   * independent changes not represented in the PR
7. Establish the correct build/test/verification commands for this repository.

Treat `codex/main-holla-integration` as the canonical integration target.

Do not use `main` as an intermediate integration branch.

## Analyze semantic deltas, not branch names

Names such as:

* `*-review`
* `*-parent-review`
* `*-fixed-review`
* `*-final-review`
* `*-independent`
* `*-baseline`
* `*-reference`
* `*-production`
* `*-repair`
* `*-proof`

are hints only.

Do not assume from the name that a worktree should be accepted or discarded.

Inspect its actual Git history and code.

A detached HEAD may contain important unique work.
A named branch may contain nothing useful.
A review worktree may contain a corrective commit absent from its parent.
A later branch may supersede several earlier branches.

Prove the relationship.

## Maintain a disposition ledger

Create/update a temporary consolidation ledger while executing the task.

Every worktree must end in exactly one explicit state:

* `ALREADY_PRESENT`
* `ACCEPTED_AS_IS`
* `ACCEPTED_PARTIALLY`
* `REIMPLEMENTED`
* `SUPERSEDED`
* `REJECTED`
* `NO_UNIQUE_CHANGE`

For each worktree record:

* path
* branch or detached HEAD
* HEAD SHA
* unique commits
* relevant files/components
* concise purpose
* reviewers
* disposition
* evidence/reasoning
* resulting integration commit(s), if any
* verification performed
* safe-to-delete confirmation

This is an execution ledger, not a substitute for actually doing the work.

## Integration rules

For accepted changes:

* integrate them into `codex/main-holla-integration`
* prefer the cleanest implementation, not necessarily the exact historical patch
* cherry-pick coherent commits when they remain correct
* manually port/reimplement the useful part when a commit mixes good and obsolete changes
* resolve conflicts semantically, never mechanically
* preserve newer/better target-branch behavior
* combine overlapping solutions instead of letting later cherry-picks accidentally overwrite earlier improvements

If a worktree reveals a valid missing fix but its implementation is poor or stale, implement the correct version directly on the integration branch and mark the source worktree `REIMPLEMENTED`.

Never accept a change solely to preserve history.

## Handle dependencies intelligently

Some worktrees are likely stacked or derived from others.

Build dependency/ancestry clusters first.

Within a cluster:

1. understand the progression of changes
2. determine which commits represent intermediate attempts
3. identify the final correct semantic state
4. integrate only what is necessary to reproduce that state cleanly

Do not cherry-pick five historical attempts when one final implementation supersedes all five.

At the same time, do not discard an intermediate commit if a later branch accidentally removed an important behavior.

## Continuous verification

Do not postpone verification until the end.

After each logical integration batch:

* format
* compile/check
* run relevant tests
* run relevant clippy/lint checks
* run targeted tests for affected components/apps
* verify behavioral/TUI changes where applicable

Use subagents to investigate failures in parallel.

A test failure is not permission to remove a valid test or weaken an assertion unless independent analysis proves that the test itself is obsolete.

Never "fix" integration by reducing correctness.

## TUI/TUX verification

This repository is especially sensitive to behavior and visual fidelity.

For worktrees touching rendering, interaction, focus, hover, keyboard/mouse handling, grid/list/tree behavior, dialogs, navigation, showcase apps, Holla, Jackin, TablePro, runtime/input systems, or other interactive components:

verify actual behavior rather than relying only on compilation.

Pay attention to:

* focus state
* hover state
* pointer behavior
* keyboard traversal
* activation
* scrolling
* geometry
* clipping
* alignment
* colors/styles
* editor behavior
* modal/dialog behavior
* state ownership
* model/view separation
* terminal cleanup
* runtime lifecycle
* action routing
* fidelity to intended product behavior

Preserve architecture while restoring or improving correct UX.

## Parallel integration discipline

Analysis can be massively parallel.

Writes to the canonical integration branch must be coordinated.

Do not allow agents to race modifications into the same working tree.

Agents may use temporary branches/worktrees for candidate integration or validation, but accepted results must ultimately be reconciled through the canonical integration branch.

After each integration batch, agents analyzing remaining worktrees must compare against the UPDATED target, because previously missing changes may now have become redundant or superseded.

## Cleanup is part of the task

Once a worktree has:

1. been analyzed by independent agents,
2. received a final disposition,
3. had all useful changes incorporated or consciously rejected,
4. been verified as containing no remaining valuable unintegrated changes,

remove that worktree.

Do not preserve worktrees merely because they might be useful later.

Handle prunable and detached worktrees too.

Before deleting any branch, prove that every valuable commit reachable only from it has been dispositioned.

Delete local branches associated with completed temporary worktrees once safe.

Do NOT delete `main` or `codex/main-holla-integration`.

Do not delete unrelated remote branches merely because they exist remotely. The cleanup objective concerns this consolidation's local worktrees/local branches unless a branch is explicitly proven to be one of these temporary consolidation branches.

Run worktree prune/cleanup as appropriate after safely removing worktrees.

## Final independent audit

After the apparent consolidation is complete, spawn fresh verifier subagents that did NOT perform the integrations.

They must independently audit:

### Git completeness

Verify that:

* every original worktree appears in the disposition ledger
* no unique valuable commit was silently lost
* rejected commits have defensible reasons
* accepted changes actually exist in the target branch
* stacked histories were handled correctly
* no unreviewed worktree remains

### Code correctness

Verify:

* repository builds
* tests pass
* clippy/lints pass at the appropriate repository strictness
* formatting passes
* no accidental dead code or temporary compatibility hacks remain
* no merge-conflict artifacts remain
* APIs and architecture remain coherent

### Behavioral correctness

Verify important interactive applications/components still work and that merged fixes did not introduce regressions.

### Repository cleanup

The final `git worktree list` should contain only the intended surviving worktrees.

The final relevant local branch state should be only:

* `main`
* `codex/main-holla-integration`

If additional local branches remain because Git safety prevents their deletion, prove why, resolve the underlying condition, and continue cleanup.

## Final PR state

Everything accepted must end up committed and pushed to:

`codex/main-holla-integration`

Update PR #1 through that branch.

Do not merge PR #1 into `main`. `main` must remain waiting for the PR.

Do not stop after merely analyzing or producing recommendations.

Execute the entire consolidation.

## Completion criteria

Do not declare success until ALL of the following are true:

* every worktree has been independently analyzed
* every worktree has an explicit disposition
* every useful change has been integrated or correctly reimplemented
* stale/worse/superseded changes have been rejected with evidence
* conflicts have been resolved semantically
* the integrated repository passes its required verification
* fresh independent verifier agents have audited the result
* all obsolete worktrees have been removed
* all obsolete local branches associated with this consolidation have been removed
* only `main` and `codex/main-holla-integration` remain as the relevant local branches
* `main` has not been modified by merging the PR
* the final integration branch is pushed
* PR #1 contains the complete consolidated result

Do not ask me to manually choose between implementations when repository evidence can determine the correct answer.

Do not stop because the task is large.

Do not leave TODOs describing work that can be completed now.

Use subagents aggressively and continue autonomously until the repository satisfies the completion criteria.

