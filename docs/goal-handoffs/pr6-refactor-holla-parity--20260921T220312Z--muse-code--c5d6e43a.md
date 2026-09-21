# GOAL: Finish the complete refactoring on PR #6's branch with exact visual-baseline parity, functional parity, and independently verified integration

## A. Identity and pause status

- Handoff ID: `pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a`
- Created (UTC): 2026-09-21T22:03:12Z | Last update: 2026-09-21T23:05Z (audit repair; doc finalized 2026-09-21T22:40Z — earlier drafts mislabeled +0700 local time as UTC, corrected in audit)
- Original goal status: `PAUSED_BY_USER`
- Handoff status: `READY` (handoff PR #10 published)
- Runtime/worker stop: sole goal worker (014-r3 verifier) sent the pause stop message, returned a STOP ACK status report, and reached terminal state (verified via runtime subagent_result delivery). No other goal workers were running. Pause-time note claimed no runtime pause API and an active-but-idle goal object; AUDIT CORRECTION: goals.db observed at audit shows status=`paused`, revisions `set` → `terminal_block` (01a0c55f…, 2026-09-21T19:09:08Z) → `terminal_pause` (01a0c622…, 2026-09-21T22:41:52Z). The pause-time claim is SUPERSEDED. The agent-attributable mechanism for those runtime transitions is UNKNOWN (issued via no agent-accessible tool; cause unlabeled in the log). Pause is effected by: stopped workers + zero queued continuations + runtime status=`paused` + this checkpoint. VERIFIED worker stop; runtime-control mechanism UNKNOWN, recorded honestly.
- Source agent: Muse Code (CLI) | Session 01a0bfe4-be83-7db0-bcd4-38f0357dda92 | Goal goal-b5184825
- Repository: `donbeave/terminal-components-claude` (github.com:donbeave/terminal-components-claude.git)
- Handoff path: `docs/goal-handoffs/pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a.md`
- Source branch / HEAD: `refactor/holla-parity` @ `227f21b6ba8a289ae585aa009b1a7d749a92aaf2` (pushed; in sync with origin at pause)
- Checkpoint code SHAs: branch tip `227f21b6` (= 079 squash-integrated); unintegrated worker checkpoint `80399a1ebf7020bcdb76ff3d48f89cd7e2e71c8b` (014 r3, preserved on remote ref, §E.3; parent == tip, CAS still valid)
- Preservation branch: `goal-handoff/pr6-parity-c5d6e43a` (this handoff + receipts bundle + appendix captures); PR base `refactor/holla-parity` @ `227f21b6`; PR URL: https://github.com/donbeave/terminal-components-claude/pull/10
- Recovery: REMOTE-PORTABLE for all goal code and acceptance evidence ("landed" = pushed to origin refs at READY; PR #10 intentionally stays DRAFT OPEN and is never merged — "landed" does NOT mean merged). Code pushed, worker checkpoints on remote refs, receipts + inventory captures bundled in the handoff PR. Explicitly NOT remote-portable (see §K): 3 local-only stashes, MAIN checkout dirt, review-f949a56 tree, prep/grok-era local-only branches, 071/072/074 full run/build trees (receipts bundled; ~392MB trees local-only), and the pgid-reap uncommitted fix (patch preserved in `appendices/at-risk/` at audit; worktree left untouched).
- Resume authorization: explicit later user request only. This handoff cannot authorize execution by itself (AGENTS.md source-of-truth hierarchy: handoff material is provenance, not authority). Resumption must re-read AGENTS.md + campaign contracts first (G §2) and reconcile the NO-GO headers (DOC-REBIND) before dispatching workers. See §J runbook.

`PAUSED_BY_USER` is the requested disposition, not proof of remote-job cessation. No remote jobs were running (all work is local subprocesses; PR #6 CI runs are automatic on push, not campaign dispatches; CI observed RED at an older head, §G).

## B. Original goal and success contract

### B.1 Original goal prompt — VERBATIM [Q]

Source S-001 (FULL): goal record `goal-b5184825-1beb-4b26-a11f-49a4d09e5c28`, revision `01a0bfe4-ced9-7850-9ccb-9f300a14b847`, set 2026-09-20T17:37:18Z. Byte-identical across session-log seq 478, `goal_control.applied` seq 46476/51381, and goals.db (26,416 chars, sha256 `01f2fd6254fc4fa542391bb029d9d54e640adaf793cee8051d4d6fe32c351717`). Pattern scan found no secrets/tokens/keys — no redactions. The goal object never changed afterward; all amendments arrived as chat. Byte-mirror: `appendices/audit/original-objective.txt`. (Reconstructed summaries are marked [S]; recovered quotations marked [Q].)

```text
Finish the complete refactoring on PR #6’s branch, restore exact visual and functional parity with the immutable visual-baseline, and independently verify the finished implementation.

Repository:
https://github.com/donbeave/terminal-components-claude

Working PR:
https://github.com/donbeave/terminal-components-claude/pull/6

Target integration branch:
refactor/holla-parity

Task catalog:
refactoring-tasks/terminal-components/
https://github.com/donbeave/terminal-components-claude/tree/refactor/holla-parity/refactoring-tasks/terminal-components

Immutable product reference:
https://github.com/donbeave/terminal-components-claude/releases/tag/visual-baseline

The required outcome is:

COMPLETE INTENDED REFACTORING
AND EXACT BASELINE VISUAL PARITY
AND EXACT BASELINE FUNCTIONAL PARITY
AND INDEPENDENTLY VERIFIED INTEGRATION.

These are separate requirements. Satisfying one does not compensate for failing another.

This is an implementation-and-completion goal, not a request for another plan, audit, preparation-only report, or suggested next goal.

Work autonomously. Do not ask questions or request another approval round. Use heavyweight subagents aggressively for investigation, implementation, testing, verification, review, and documentation. Continue through repair and verification iterations until the complete goal is genuinely satisfied.


## 1. Scope, authority, and protected boundaries

All integrated implementation work belongs on refactor/holla-parity, the branch associated with PR #6. Temporary isolated worker branches/worktrees are allowed only to support this campaign.

Never implement on main or visual-baseline. Never merge this work into main as part of this goal. Finish with the target branch verified and merge-ready.

The partially refactored main branch is an implementation/history reference, NOT the visual or behavioral oracle. Preserve valid architectural progress, but do not preserve broken visuals merely because they already exist on main.

Use the frozen visual-baseline as the authoritative reference for observable product appearance and behavior across:

- The reusable component library and its public examples.
- Showcase.
- Holla.
- Jackin Preview / jackin-preview.
- TablePro.

Use current, reconciled architecture and task contracts to determine the required internal design.

Never restore parity by resetting the branch to the old implementation, reverting the entire refactoring, or copying the old architecture over the new one.

Never move, delete, recreate, retarget, force-push, commit to, or otherwise modify the visual-baseline tag, branch, release, source checkout, snapshots, or expected artifacts. Treat all baseline inputs as read-only.

Do not run snapshot blessing, acceptance, regeneration, or update operations against the baseline. Do not substitute candidate-generated output for missing baseline output.

Preserve unrelated local changes, other agents’ worktrees, existing commits, and user work. Do not perform destructive cleanup, history rewriting, broad process termination, or permission changes outside this campaign.


## 2. This request authorizes prerequisite repair and subsequent implementation

Read AGENTS.md and the current campaign contracts first.

The repository currently contains preparation-only instructions, a NO-GO report, and a future implementation prompt that can exit immediately when startup checks fail. Do not blindly execute that historical stop condition.

This request explicitly authorizes the following sequence in one campaign:

A. Investigate and repair preparation, tooling, proof, and calibration blockers.
B. Independently verify that implementation prerequisites are satisfied.
C. Record the evidence-backed transition to implementation readiness.
D. Execute the complete reconciled refactoring.
E. Independently verify final product completion.

This authorization replaces the earlier preparation-only scope. It does NOT waive baseline protection, evidence requirements, independent verification, or safety boundaries.

While genuine implementation prerequisites remain unsatisfied, keep production dispatch disarmed. Use scoped repair subagents to fix those prerequisites; do not stop merely because the initial readiness report says NO-GO.

Distinguish three states:

- Prerequisite repair permitted.
- Ready for production refactoring.
- Product refactoring complete and accepted.

Do not create a circular gate requiring the finished refactoring or final candidate parity before allowing the refactoring to begin. Preserve those requirements as task, integration, and final acceptance obligations. Baseline calibration, trusted verification, valid dependencies, and genuine startup requirements remain prerequisites.

Independently review any correction to this distinction. Do not relabel a failed prerequisite as a final-only obligation simply to bypass it.

Once the real startup conditions pass, record the transition and continue automatically. Do not ask the user to issue another GO or paste another prompt.


## 3. Establish fresh branch and oracle identities

At startup, record actual local and remote refs, PR head, worktree state, merge-base with main, source commit/tree, and existing uncommitted work.

The PR head inspected while preparing this goal was:
a292cf860d87c93fc329d16d2310ca86ad370d3d

That is a historical inspection point, not an instruction to reset to it. Re-read the current branch and preserve newer valid work.

Verify the protected oracle identities:

Annotated visual-baseline tag object:
1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5

Peeled visual-baseline commit:
4a79c0a2d40fca46fc406b77157ce3b3f12ec16b

Baseline commit tree:
0b1f13431fdfd6060cf9f45a114afa5a99cc6c26

Baseline snapshots tree:
3f0261c32849e26feda24d87697de4a7ce6b8375

Fail closed on unexpected oracle identity changes. Never “repair” a mismatch by moving the protected ref or silently selecting another oracle.

Use independently verified, read-only baseline imports or disposable detached reference copies. Keep build outputs, logs, observed artifacts, and verification receipts in separate campaign-owned directories.

Record protected-ref and oracle-artifact hashes before work, at meaningful checkpoints, and at final acceptance. Verify that they remain unchanged.


## 4. Reconcile the complete task catalog and actual remaining work

Delegate a complete audit of:

- AGENTS.md and applicable directory policies.
- docs/refactoring-plan/ and docs/refactoring/.
- Every task package under refactoring-tasks/terminal-components/, including nested verification contracts.
- refactoring-tasks/visual-validation.md.
- COMPONENT_ARCHITECTURE.md, DESIGN.md, GOAL.md, and relevant architecture decisions.
- Source, public APIs, examples, application routes, tests, verification scripts, CI, and relevant Git history.
- PR #6’s current discussion, reviews, and review threads.

Distribute large documents and code areas among subagents and maintain a coverage ledger. Do not substitute a few document excerpts for an exhaustive obligation inventory.

Reconstruct the executable dependency graph from current contracts and source evidence. Record each task’s prerequisites, writable scope, shared interfaces, required behavior, required proof, and independent acceptance conditions.

The inspected catalog reports 73 direct task packages, with TASK-001 and TASK-070 retired, TASK-071 and TASK-072 serving qualification prerequisites, and the remaining valid implementation work including TASK-002 through TASK-069 and TASK-073.

Revalidate this against the current tree. Do not confuse catalog counts, lint success, graph generation, or old status fields with completed implementation.

Account for every task. A valid task is complete only with current implementation and evidence. A retired or superseded task requires a reviewed disposition explaining where its still-valid obligations are satisfied.

Do not resurrect retired container/lifecycle infrastructure just because a stale dependency or task title mentions it. Reconcile obsolete bootstrap edges with verified native prerequisites while preserving their legitimate requirements. Do not simply delete dependency edges to unlock dispatch.

Maintain traceability:

requirement → task → implementation → tests → visual/behavioral evidence → independent review → integrated commit.

Add narrowly scoped repair tasks when necessary to finish the promised outcome. Do not silently shrink scope, drop difficult cases, or replace implementation with documentation.


## 5. Repair and qualify verification before trusting its results

Delegate dedicated proof-tooling, visual-calibration, and platform-verification subagents.

Reproduce the current readiness blockers from fresh executions. The inspected report includes:

- Failure of the exact-tag known-good visual control.
- A missing observed TablePro artifact key:
  tablepro/query/results/160x50/nocolor
- HTML differences involving the compiled executable path.
- Failure of the unfiltered native proof suite.
- Missing accepted current verifier/reviewer receipts.
- Missing required native Linux evidence.

Treat these as reported starting points, not guaranteed current facts. Inspect raw evidence and reproduce before deciding what to fix.

First establish that the immutable known-good baseline can pass the complete qualified comparison against its own existing oracle. Otherwise, candidate comparison results are not trustworthy.

Repair the native launcher, capture adapters, suite integration, observer lifecycle, artifact collection, or other faulty verification infrastructure on the working branch.

Do not modify frozen baseline source or expected artifacts. Do not normalize away differences, spoof executable provenance, mask fields, loosen thresholds, skip cases, or manufacture passing receipts.

Resolve environmental and capture-provenance problems without weakening equality or concealing which binary actually executed.

Restore or correctly port the baseline visual suite and required nextest configuration into the candidate verification path. Preserve the frozen cases, inputs, expected outputs, and comparison semantics. An API adaptation must not change the acceptance contract.

Prove that the gate rejects, in disposable negative-test data:

- Missing or duplicated cases.
- Missing artifacts.
- Changed cells, styles, geometry, or cursor state.
- Incorrect source, binary, tool, oracle, or dependency identity.
- Stale or cross-run evidence.
- Nonzero exits, timeouts, hangs, truncated results, or failed observers.
- Missing required input replay or behavioral checkpoints.

A verifier must observe real command execution and numerical exit status. A success-shaped JSON file, self-reported worker result, or existing artifact directory is not proof.

Keep the evidence process finite: commit the payload and documentation, freeze that tree, then produce external verifier/reviewer receipts for it. Do not repeatedly invalidate final evidence by editing tracked reports after sealing.


## 6. Aggressive parallel execution with independent roles

Use one root Grok Build coordinator with a continuously replenished pool of native subagents.

The coordinator schedules, resolves ownership, tracks dependencies, and integrates independently reviewed changes. Task-owned code changes, test work, verification, and reviews belong to subagents.

Use full-capability subagents for implementation and executable verification. Read-only explore/plan agents may assist with investigation, but they cannot replace verifiers that must run commands.

Use the strongest suitable model and reasoning capability actually available in the configured Grok Build environment. Record actual model/tool usage; do not claim unavailable models or imitate another agent platform’s tools.

Parallelize every independently executable activity. Keep ready work moving while other agents build, test, review, or investigate failures.

Maintain concurrent workstreams for:

- Task reconciliation, architecture, and shared-interface decisions.
- Native proof tooling and baseline calibration.
- Runtime, layout, theme, text, input, focus, layers, and component families.
- Separate Showcase, Holla, Jackin Preview, and TablePro migration lanes.
- Independent visual/behavioral verification.
- Architecture/API/performance review and final acceptance.

Schedule by dependencies and actual file conflicts, not task number. Do not start every task at once without prerequisites.

Every worker assignment must specify:

- Task and requirement IDs.
- Exact starting commit and accepted dependency receipts.
- Owned writable paths and explicitly forbidden paths.
- Baseline surfaces and behavior to preserve.
- Expected implementation, tests, evidence, and completion criteria.
- Required handoff and integration conditions.

Give each implementer an isolated worktree and separate writable build/run directories. Give verifiers a clean, committed, read-only candidate view plus their own external run directories. Never share writable outputs between roles.

Reserve capacity for verification and review. Reclaim completed or failed agent sessions before spawning replacements. On capacity limits, queue work and keep available agents productive; do not abandon independent verification or spawn uncontrolled nested coordinators.

Maximize safe throughput, not agent count for its own sake. Bound simultaneous expensive builds and PTY captures to avoid resource-induced failures. Run performance measurements in an isolated, low-contention lane.

For every substantive change, require a different verifier and a separate independent reviewer. An implementer may run focused checks but cannot accept their own work.


## 7. Finish the intended architecture without disguising regressions

Implement the reconciled architecture completely, including:

- Caller-owned state.
- Borrowed props and appropriate ownership/lifetime boundaries.
- Read-only drawing.
- Runtime-owned routing, focus, layers, pointer handling, and cursor behavior.
- One reusable implementation per component family.
- Correct public APIs and fully migrated consumers.

Inspect the actual architecture contracts rather than treating this summary as their replacement.

Repair reusable components and shared primitives at their proper ownership boundaries. Do not solve each application independently with duplicate patches for a common library defect.

Remove transitional architecture when its consumers are migrated: duplicate renderers, obsolete adapters, conflicting state models, dead migration paths, and temporary compatibility code that violates the target design.

Specifically investigate the reported Showcase compatibility painting/fixed-grid workarounds, Jackin projections, TablePro legacy painters, and reduced Holla route/scenario coverage. Verify the current code rather than assuming these issues remain unchanged.

Forbidden shortcuts include:

- Repainting the old frame over incorrect new component output.
- Hardcoding captured baseline frames.
- Loading oracle snapshots in production code.
- Routing tests through the old renderer while real users receive the new one.
- App-local substitute components that bypass the refactored library.
- Preserving two competing architectures indefinitely.
- Removing features or interactions to make parity easier.

Do not redesign, simplify, or “improve” the frozen UI. This goal changes implementation architecture while preserving the product.


## 8. Exact visual parity is a complete-matrix requirement

The protected contract requires all 7,550 matrix keys and all 30,200 artifacts:

- ANSI.
- Plain text.
- PNG.
- HTML.

Cover all five frozen terminal sizes:

72x20, 80x24, 100x30, 120x40, 160x50.

Cover all five recorded color modes:

truecolor, 256-color, 16-color, none, nocolor.

Derive the complete key inventory independently from the pinned oracle. Compare key sets and per-key artifacts, not just total counts or the number of test functions.

For each baseline case, compare actual candidate output with the corresponding frozen expected output using the qualified exact comparator.

Check cells, graphemes, continuation cells, styles, foreground/background colors, dimensions, cursor, clipping, borders, spacing, alignment, wrapping, truncation, layering, selection, focus, hover, and captured motion states.

Require exact PNG/HTML comparison through the qualified deterministic artifact path as well as exact terminal-output comparison. Text-only or screenshot-only success is insufficient.

Replay affected cases immediately after relevant changes. Expand coverage at integration milestones. The final gate must execute the complete matrix, not a representative sample, reduced “supported” subset, or fast-mode approximation.

Retain expected/actual/diff artifacts for failures and have independent subagents inspect visual differences. Human-style image inspection supplements deterministic comparison; it does not replace it.

Never update expectations merely because the candidate differs. Fix the candidate or prove and repair a harness defect without redefining the oracle.


## 9. Functional parity must exercise real application paths

Snapshots alone do not prove functional correctness.

Build a baseline-derived interaction inventory covering all four applications, every public component family, every reachable route, and all frozen scenarios.

Verify applicable behavior including:

- Keyboard navigation, shortcuts, typing, editing, and completion.
- Focus entry, traversal, restoration, and modal isolation.
- Mouse hover, clicks, hit testing, drag/capture, scrolling, and resizing.
- Selection, filtering, sorting, tab changes, tree expansion, and navigation.
- Dialogs, menus, overlays, pickers, forms, tables, editors, and panels.
- Empty, loading, error, disabled, and populated states.
- State persistence and asynchronous updates.
- Terminal startup, raw mode, alternate screen, cursor management, exit, restoration, and process cleanup.

Use deterministic fixtures, controlled clocks/seeds, and the same input sequences where the contract requires them. Never change fixtures or timing solely to hide a regression.

Verify both state transitions and visible output after real interactions. Exercise startup, intermediate settled frames, resize, and shutdown through the real application/runtime path.

Include native PTY end-to-end verification. Headless render tests are useful but cannot replace actual terminal input/output and lifecycle evidence.

Use isolated test data and approved simulation facilities. Do not trigger real destructive operations, external account actions, or unintended network side effects while exercising demo flows.


## 10. Apply the same implementation–verification loop to every task

For each task:

1. Read its complete contract, current source, relevant baseline behavior, dependencies, and required history.
2. Identify concrete missing or incorrect implementation.
3. Implement in an isolated subagent worktree.
4. Add or update tests that detect the original defect and preserve required behavior.
5. Run targeted native tests and affected baseline comparisons.
6. Commit the candidate using repository commit conventions.
7. Have an independent verifier execute the task’s deterministic and native proof gates against that exact committed candidate.
8. Have a separate reviewer assess correctness, architecture, scope, visual/functional parity, and forbidden shortcuts.
9. Fix every substantiated finding and repeat verification.
10. Integrate the reviewed change serially.
11. Re-run affected integration gates on the resulting integrated commit.
12. Record acceptance only when evidence matches what was actually integrated.

Use the repository’s expected-parent/compare-and-swap integration protection. If the branch advances or conflicts occur, refresh the integration candidate and obtain the necessary fresh evidence.

Do not treat proof for an old parent or pre-conflict patch as proof for a new integrated tree. Preserve meaningful merge history; do not rewrite the shared branch.

Shared-interface changes require coordinated ownership and downstream revalidation. Integration conflict resolution that changes code must itself be reviewed.

Keep a durable ledger of task disposition, owner, dependencies, source identity, implementation commit, verifier/reviewer decisions, commands, exit codes, artifact paths, and remaining failures.

No substantive task may be accepted from an agent’s summary alone.


## 11. Native tooling and repository-wide quality gates

Execute locally on native macOS and use already-authorized native Linux execution where required by the platform contracts.

Do not use Docker, Podman, containers, images, mounts, root firmlinks, or retired container namespaces. Do not create new paid infrastructure or use unauthorized hosts.

All Rust tests and Rust test-validation commands must use cargo nextest. Never use cargo test. Use the repository’s qualified build, documentation, static-analysis, and feature-matrix commands where a test runner is not applicable.

Use the qualified standalone taskfmt from the current campaign contracts. Verify its source revision, executable version, and hash; requalify changed tooling before trusting it.

Use taskfmt only for deterministic per-task lint and verify. Do not use it as the agent orchestrator, workspace manager, ref manager, dispatcher, or promotion system.

Run the required formatting, compilation, lint, public API, documentation, link, shell, workflow, behavior, visual, performance, and allocation gates.

Do not assume an ordinary workspace test run includes ignored visual suites. Explicitly enumerate and execute every required suite and profile.

Treat candidate tests that encode the broken refactor’s behavior as potentially incorrect assertions. Reconcile them against the frozen product contract with documented test-identity mapping and replacement coverage. Do not delete tests merely because they fail.

Preserve CLAUDE.md as the required AGENTS.md symlink and follow repository commit/sign-off conventions.

Do not weaken CI, disable tests, broaden exclusions, suppress meaningful failures, inflate timeouts without diagnosis, or raise performance budgets to obtain green output.

Obtain genuine evidence for required platforms. An unavailable platform is not a pass, and CI status alone does not replace required native proof.


## 12. Final independent acceptance

After all valid tasks are integrated, freeze the complete candidate commit and run final acceptance from a clean verification view.

Commit final source-controlled documentation before the final evidence seal. Store the exact-tree final receipts externally so producing the report does not invalidate the tested commit.

Commission separate final verifier and adversarial reviewer subagents. They must inspect raw evidence, re-run decisive checks, and attempt to find regressions, omitted obligations, invalid provenance, and architecture shortcuts.

Every item below is mandatory:

[ ] Every original task is accounted for with an evidence-backed disposition.
[ ] Every valid refactoring obligation is implemented and independently accepted.
[ ] Retired tasks remain retired; their legitimate obligations are not lost.
[ ] All work is integrated into refactor/holla-parity.
[ ] The target architecture is complete and all consumers are migrated.
[ ] No duplicate renderer, snapshot painter, or forbidden compatibility workaround remains.
[ ] All four applications and required examples build and function.
[ ] The complete 7,550-key inventory matches the immutable oracle.
[ ] All 30,200 required artifacts pass exact comparison.
[ ] Every required behavioral, interaction, resize, and PTY lifecycle case passes.
[ ] Taskfmt, Rust/native tests, API/static/docs/workflow gates pass.
[ ] Performance and allocation requirements pass under valid measurement conditions.
[ ] Required native platform evidence exists and passes.
[ ] PR review feedback has been re-read and every applicable finding addressed or explicitly dispositioned with evidence.
[ ] The final verifier and separate reviewer accept the exact integrated commit.
[ ] Protected baseline refs, release, source, snapshots, and expected artifacts are unchanged.
[ ] The target branch is clean and merge-ready, with no required failure hidden as skipped or unsupported.

Missing, failed, skipped-required, stale, incomplete, or unexecuted evidence means the relevant gate is not satisfied.

A green checklist, successful lint, passing build, or visually similar screenshot does not establish completion.


## 13. Persistence and final handoff

Continue working through failures. Fix, test, compare, independently review, integrate, and repeat.

Do not finish because a plan was written, a task wave ended, the code compiles, one application looks correct, or the preparation report was updated.

Do not defer normal remaining work to another goal. Use durable checkpoints so the active campaign can survive context compaction and recover from failed workers without losing accepted evidence.

For a genuine external blocker that cannot be resolved with available permissions or tools, preserve the exact failure evidence, continue every independent permitted activity, and do not fabricate completion or bypass protections. Do not ask routine questions or repeatedly retry an unchanged failure without diagnosis.

The final handoff must state:

- Final branch and exact commit/tree.
- Reconciled task completion/disposition counts.
- Architecture changes and removed transitional code.
- Visual matrix results, including expected and observed artifact counts.
- Behavioral, PTY, platform, performance, and repository-wide gate results.
- Independent verifier/reviewer decisions and evidence locations.
- Proof that the baseline remained unchanged.
- Any genuine unresolved blocker, clearly separated from completed work.
- Whether the exact final commit is ready to merge.

Do not merge into main or modify the protected baseline.

Begin now by establishing fresh identities and dispatching parallel prerequisite-repair, task/architecture-audit, and application-inventory subagents. Once independently qualified, continue directly into dependency-ordered implementation and finish the full refactoring with exact visual and functional parity.
```

### B.2 Material amendments — VERBATIM [Q]

All 15 user chat messages recovered verbatim from the session log (S-002…S-006, FULL). No other user-authored material exists in the log.

**A1–A4 commit-trailer rule (binding; strongest phrasing governs).** A1 = A2 verbatim (intents `01a0c113` @ 2026-09-20T23:07:51Z; `01a0c1de` @ 2026-09-21T02:49:12Z):

> Always commit with Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>

A3 (intent `01a0c1e0` @ 2026-09-21T02:51:31Z):

> Only use Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com> for commits. Never commit with not Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>

A4 (intent `01a0c1e4` @ 2026-09-21T02:56:13Z, SEQ 23685):

> Commits must ALWAYS with Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>. Never anything else than Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>

Working interpretation [S]: every commit on every goal branch must carry exactly `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`; the `Co-authored-by: Codex <codex@openai.com>` trailer is a *repository* rule (AGENTS.md), not a user message — it coexists on the trailer block and does not violate "never anything else" (which governs the sign-off identity, not the trailer set). All 4 pause/audit-handoff commits and all campaign commits comply.

**A5 subagent-aggression rule (7 identical pastes, intents `01a0c349`…`9715919d`, 2026-09-21T09:25:49Z → 19:31Z), verbatim:**

> Use subagents aggressively for all work.
>
> Always delegate work to subagents whenever delegation is possible. Treat subagents as the default execution mechanism, not an optional optimization.
>
> Your execution strategy must:
>
> * decompose the goal into independent or partially independent workstreams;
> * spawn subagents for each workstream;
> * parallelize all work that can safely run concurrently;
> * use additional subagents for research, implementation, review, testing, verification, and cross-checking;
> * avoid doing work serially in the parent agent when it can be delegated;
> * keep spawning useful subagents as new independent tasks are discovered;
> * use independent subagents to verify important conclusions and completed changes;
> * coordinate and synthesize subagent results into the final implementation.
>
> Do not merely recommend parallelization—actually execute the goal through subagents.
>
> The parent agent should primarily orchestrate, resolve dependencies/conflicts, integrate results, run final deterministic checks, and ensure the complete goal is finished.
>
> Default rule: **delegate first, parallelize aggressively, verify independently, then integrate.**

**A6 never-ask rule (intent `01a0c5ce` @ 2026-09-21T21:10:38Z), verbatim:**

> Never ask the user questions or wait for clarification. Work fully autonomously.
>
> If anything is ambiguous, uncertain, conflicting, incomplete, or requires a decision:
>
> * Spawn subagents to investigate it independently.
> * Analyze the available context, repository, documentation, code, history, external references, and relevant best practices.
> * Research alternative approaches where necessary.
> * Compare multiple options and their tradeoffs.
> * Verify important assumptions and findings independently.
> * Re-verify critical decisions before acting.
> * Make the best reasonable decision yourself and continue execution.
>
> Do not stop because information is imperfect. Infer intent from the goal, existing architecture, conventions, documentation, and surrounding context. Prefer making a well-researched, reversible decision over asking the user.
>
> When uncertainty is significant, use multiple independent subagents to challenge the proposed solution and resolve disagreements through evidence.
>
> Your responsibility is to unblock yourself. Questions that would normally be sent to the user should instead become internal research, analysis, verification, or subagent tasks.
>
> Continue working until the goal is fully completed, verified, and no meaningful actionable work remains.

**A7 commit-often rule (intent `01a0c5f4` @ 2026-09-21T21:51:57Z), verbatim:**

> Always commit changes frequently while working.
>
> Prefer small, incremental, logically scoped commits instead of keeping a large dirty working tree for a long time and committing everything at the end. As soon as a meaningful unit of work is complete and verified, commit it.
>
> Push progress to the remote repository regularly so work is continuously propagated, recoverable, reviewable, and easy to bisect or revert.
>
> At the same time, avoid unnecessary branches. Prefer doing as much work as possible on a single working branch and keep committing to that branch throughout the task.
>
> Create additional branches only when there is a clear technical or workflow reason that makes working safely on the existing branch impractical or impossible.
>
> In short: **commit often, push regularly, and minimize branch proliferation.**

**A8 pause order (intent `01a0c5fa` @ 2026-09-21T21:59:07Z, SEQ 49402, 39,385 chars).** Full verbatim text: `appendices/audit/pause-order.txt` (too long to inline; byte-exact copy). Operative effect: freeze original-goal work, preserve everything, write §§A–K handoff, publish `GOAL:`-titled DRAFT PR, stop. It *supersedes* "continue autonomously until finished" **until explicit resumption**; it does not cancel any G requirement. Its H-contract is fully mapped in §M.

**A9 audit order (intent `d1effc6e` @ 2026-09-21T22:42:23Z, SEQ 51384, 19,455 chars).** Authorizes only this audit-and-repair; original goal stays paused. Requirements mapped in §M; outcome in §N.

### Consolidated statement [S]

Complete every valid task in `refactoring-tasks/terminal-components/` (002–069 + 073; 001/070 retired with dispositions; 071/072 qualification; repairs 074–079) on branch `refactor/holla-parity`, preserving the new component architecture while reproducing the frozen `visual-baseline` oracle 1:1 — all 7,550 matrix keys / 30,200 artifacts (ANSI+plain+PNG+HTML × sizes 72x20, 80x24, 100x30, 120x40, 160x50 × modes truecolor/256/16/none/nocolor) — plus full behavioral/PTY parity, via serial compare-and-swap integration of independently verified+reviewed subagent work, ending merge-ready (never merging to `main` in this goal).

### Non-goals / boundaries [Q+S]

Never implement on `main`/`visual-baseline`; never modify the baseline tag/branch/release/snapshots (read-only oracle); never bless snapshots or substitute candidate output; never use Docker/containers or `cargo test`; never rewrite shared history; never merge into `main`.

### Material constraints & preferences

- User: aggressive subagent delegation (all impl/verify/review via subagents); never ask questions — decide autonomously with evidence; commit often, push regularly, minimize branches; every commit `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` + `Co-authored-by: Codex <codex@openai.com>`.
- Repo: `cargo nextest` only; qualified taskfmt 0.2.0 / afd3b575 / f9781ef8… (lint+verify only); subagent roles + CAS integration; coordinator never edits task-owned files; `CLAUDE.md` stays an `AGENTS.md` symlink; workflows only via `velnor-workflow`; ledger `armed=false`.
- Post-resumption obligation (from this pause): integrate all required related goal work, resolve its PRs, and clean up verified-obsolete goal-owned local worktrees/branches — without blind-merging experiments or touching shared/unrelated resources.

## C. State at the exact interruption point

- Last completed action: TASK-079 integrated (`227f21b6`, squash, tree `9a1882f5`) + `refactor/holla-parity` pushed to origin (in sync, verified `e34314ce..227f21b6`).
- In-progress action: TASK-014 r3 verification. Candidate `80399a1e` (one commit on `227f21b6`, fade via 079 `fade_mix`, 3 files +1276/-2: `crates/tui/src/ui/paint.rs`, `crates/tui/src/components/scroll_region.rs`, `crates/tui/tests/completion_014.rs`) committed with implementer evidence (`appendices/014-r3-implementer-evidence.md` + `appendices/014-r3-logs/`). Independent verifier reached ~95% (integrity PASS, lint 0, build+prepare 0, 7/7 checks exit 0 with CHK-004 8/8, taskfmt verify 11/11, nextest completion_014 15/15, BOTH r1-failing arch gates PASS in-suite and in isolation, full suite exactly 5 knowns, zero production `Color::Rgb`, 079 dead_code warnings GONE) then STOPPED per pause before writing the formal verdict receipt. Verdict file `TASK-014-verify-r3.json` was NOT written (confirmed by filesystem hunt over `/tmp/tc-014-vfy3` + `/tmp/tc-014-r3`); NO verdict (positive or negative) may be claimed — resumption must finish/redo verification (§H.1).
- CI state: PR #6 CI is RED at `e10fd942` (14 fail / 11 pass), 8 commits behind the tip; CI status of tip `227f21b6` UNKNOWN at pause. Resumption must triage CI on the tip before drawing conclusions (§G, §H.0).
- Partially edited files: none in tracked worktrees (campaign checkout clean; only this handoff's untracked staging dir, which moves to the handoff branch). Uncommitted work: none goal-owned (only the pre-existing unrelated MAIN checkout dirt, §E.2).
- Open hypotheses: none blocking; 014 r3 expected-VERIFIED but unproven until verdict sealed.
- Workers: sole goal worker (014-r3 verifier) STOPPED via message, returned status, terminal. 4 pause-auditors (goal/worktree/branch-PR/verify-resume) ran read-only; coordinator additionally verified all core PR/branch/worktree facts directly (this doc supersedes any pending auditor deep-dive).
- No in-progress merge/rebase/cherry-pick/conflicts/detached HEAD in the campaign checkout. No remote jobs, deploys, or CI dispatches by this campaign.

## D. Requirement-by-requirement progress ledger

Task states: `VERIFIED_DONE` (integrated + receipts), `IMPLEMENTED_UNVERIFIED`, `IN_PROGRESS`, `NOT_STARTED`, `BLOCKED`, `RETIRED` (with disposition).

| ID | Requirement | Status | Evidence / files / commits | Remaining work | Dependencies |
|----|-------------|--------|----------------------------|----------------|--------------|
| TASK-071/072/074/075/076 | Qualification + harness repairs | VERIFIED_DONE | Receipts TASK-07{1,2,4,5,6}.json; commits bffd7224,b71c4d04,eacea930,77b30e8d,b905cb70 ancestors of tip | none | — |
| TASK-077 | Native oracle namespace gate | VERIFIED_DONE | Receipt; c8b37b54 | none | 071/072/074/075/076 |
| TASK-002/003/004/005/007 | Wave-3 completions | VERIFIED_DONE | Receipts; 57b411b6,2e48cbe3,731e688a,117a8121,dde02859 | none | chained |
| TASK-006 | Components adapter | VERIFIED_DONE | Receipt; d372b79c | none | 002–005/071/072 |
| TASK-008 | Test disposition | VERIFIED_DONE | Receipt; a0bf0f94 | none | 007/006/071/072 |
| TASK-078 | Production receipt-binding repair | VERIFIED_DONE | Receipt; 98af8ff9 (+contract 23180887); unblocked all production CHK-005 | none | 071/072/074/075 |
| TASK-073 | Resolution attribution | VERIFIED_DONE | Receipt; 1756c341 | none | 008/072 |
| TASK-011/012/009 | Theme ASCII / split minima / session contracts | VERIFIED_DONE | Receipts; 12f7f3bf,b4d2c0d0,735656ba | none | 006/008/073 |
| TASK-029/010/013 | Progress witnesses / focus capture / grapheme+grammar | VERIFIED_DONE | Receipts; e34314ce,100db11e,926264a1 | none | 009/010/011/012/008/073 chains |
| TASK-079 | Theme fade_mix helper repair | VERIFIED_DONE | Receipt; 227f21b6 (squash, tree 9a1882f5); dead_code transient cleared by 014 r3 (unverified claim until 014 verdict) | none (own); consumer proof via 014 | 011 |
| TASK-014 | Scroll-edge fade + drag follow | IN_PROGRESS (r3 pending verdict) | r1 01849854 REJECTED (verify-r1 receipt, rule-22+palette); r2 BLOCKED-STANDS (18-pt evidence); r3 80399a1e committed + implementer evidence; verifier 95% green, NO verdict file | finish verify verdict → review → CAS integrate | 013/010/011/012/008/073 + base-tree 079 |
| TASK-015/016/017 | Next DAG wave | NOT_STARTED | DAG: unlock on 014 acceptance | dispatch after 014 | 014 (+chains) |
| TASK-018–069 (minus retired/done) | Remaining catalog | NOT_STARTED | task-graph.json + task.toml DAG | dependency-ordered waves | per DAG |
| TASK-001/070 | Retired | RETIRED | Retired per catalog; obligations mapped (see readiness report) | keep retired; final reconfirmation | — |
| VISUAL-FULL | 7,550 keys / 30,200 artifacts final gate | NOT_STARTED | Per-task affected replays only (CHK-002/003/004 evidence in run dirs) | full-matrix run at final gate | all tasks |
| PLATFORM-LINUX | Native Linux evidence | NOT_STARTED | missing (reported) | authorized native run or documented exception | final |
| DOC-REBIND | Stale NO-GO headers | NOT_STARTED | readiness report/campaign-policy still say NO-GO | reconcile on resume | — |
| PR6-REVIEWS | PR #6 review-thread re-read | NOT_STARTED | user instruction; reviewDecision empty at pause | re-read + disposition before/during waves | — |
| WF-CMD | Workflow-syntax validation command | BLOCKED | AGENTS.md mandates, no runnable named (static probe) | coordinator decision on owning task | — |
| CI-RED | PR #6 CI red at e10fd942 (14 fail/11 pass) | NOT_STARTED | appendices/ci-checks.txt; tip-227f21b6 status unknown | triage on tip first (§H.0) | re-observe |

## E. Change and preservation inventory

Campaign branch `refactor/holla-parity` (tip `227f21b6`, pushed, 0/0 vs origin): all task/contract commits are single-commit, signed-off, CAS-integrated after VERIFIED verifier+reviewer evidence. Receipts bundle (`<id>/receipts/`, 68 files, 468K) carries the acceptance/evidence JSONs (manifest: `appendices/receipts-manifest.txt`). Unintegrated: ONLY 014 r3 `80399a1e` (remote checkpoint ref, §E.3). No other uncommitted/unpushed goal work exists in tracked worktrees.

### E.1 Discovery scope and ownership

- Inspected (read-only): main repo + campaign worktree (`git worktree list`, status, stash, log); all `/tmp/tc-*` worker dirs (176 entries); `/tmp/tc-goal-b518/receipts/` (68 files); `.muse/worktrees/` (empty, hygiene confirmed); sibling clones (`terminal-components-2026-09-20` not a repo; `review-f949a56` dirty staged tree; `.codex/worktrees/52aa/campaign` @9abd3ec3); `gh` PR/branch/check state; session transcript evidence refs. Timestamps: 2026-09-22 ~05:30–05:40Z. No filesystem-wide scan; no private dirs beyond campaign paths.
- Ownership: `GOAL_EXCLUSIVE` = campaign worktree, /tmp/tc-* worker dirs (118 worktrees), /tmp/tc-goal-b518, checkpoint refs, handoff branch/PR. `GOAL_SHARED` = main clone object store (hosts visual-baseline + campaign objects) — retain, read-only. `UNRELATED` = MAIN checkout dirt, review-f949a56 staged tree, prep-/proof-/grok-goal-era worktrees/branches, other users' branches, stashes — recorded for exclusion only, never touched.
- Coverage: COMPLETE. All 193 registered worktrees enumerated (`appendices/worktrees.txt`), all 34 local + 8 remote-only branches (`appendices/branches.txt`), all 176 /tmp/tc-* entries (`appendices/tmp-tc-dirs.txt`), all 9 PRs (`appendices/prs.txt`), PR #6 checks (`appendices/ci-checks.txt`). Re-enumerate live on resume before any cleanup (§E.6).

### E.2 Local worktree and clone ledger (grouped; full per-tree list in appendices/worktrees.txt)

193 registered worktrees: 118 GOAL_EXCLUSIVE, 75 unrelated/stale. 176 /tmp/tc-* disk entries (worktrees + log/run/file dirs).

| ID | Path group | Count / HEADs | State | Owner/purpose | Disposition |
|----|------|------|-------|---------------|-------------|
| W-MAIN | /Users/donbeave/Projects/terminal-components-claude (main checkout @ 4a79c0a2, `visual-baseline` branch) | 1 | DIRTY (M .gitignore; untracked .campaign/ .worktrees/ campaign-ledger.schema.json nprintf scripts/) | UNKNOWN/UNRELATED (pre-existing; NOT campaign work) | KEEP, never touch |
| W-CAMP | .worktrees/campaign | 1 @ 227f21b6, refactor/holla-parity, clean, pushed | GOAL_EXCLUSIVE integration checkout | KEEP (primary) |
| W-014R3 | /tmp/tc-014-r3/wt | 1 detached @ 80399a1e, clean | GOAL_EXCLUSIVE 014 r3 candidate | INTEGRATE_THEN_REMOVE (after 014 acceptance) |
| W-014V3 | /tmp/tc-014-vfy3/{candidate@80399a1e,parent@227f21b6} + run/run-verify/logs/nextest-target | 2 + evidence | GOAL_EXCLUSIVE stopped-verifier evidence | INTEGRATE_THEN_REMOVE (after verdict re-done) |
| W-079SERIES | /tmp/tc-079-{impl,r2,sq}/wt, vfy/vfy2 candidates | 8, detached, clean | GOAL_EXCLUSIVE superseded rounds (r1 5e625cff, r2 cd9c395f) + squash evidence | INTEGRATE_THEN_REMOVE (refs pushed; dirs after final gate) |
| W-PRIOR | /tmp/tc-{009,010,011,012,013,029,073,078}-* impl/r2/vfy/candidate | 36, detached, clean | GOAL_EXCLUSIVE accepted-round evidence | INTEGRATE_THEN_REMOVE (after final gate) |
| W-B518 | /tmp/tc-goal-b518/* (73 worktrees + receipts/ + observer providers) | 73, mixed detached | GOAL_EXCLUSIVE receipt store + round evidence | KEEP receipts (bundled with handoff); worktrees INTEGRATE_THEN_REMOVE later |
| W-TRIAGE | /tmp/tc-holla-triage/{base,post011,tip} + /tmp/tc-{health,static,proof-pgid-reap,009-adopt,worktree-archive} | 4 worktrees + file dirs | GOAL_EXCLUSIVE triage/health/static/adoption evidence | KEEP (small); worktrees INTEGRATE_THEN_REMOVE after final |
| W-PREP | .worktrees/prep-* | 25, detached old SHAs | UNRELATED prep-era | REVIEW_SHARED later; NEVER via goal runbook |
| W-GROK | /private/.../grok-goal-c228471198a8/implementer/* | 24, old SHAs | UNRELATED other-goal era | NEVER touch |
| W-TMPOTHER | /tmp/{grok-catalog-remap,prep-*,proof-*,terminal-components-review.*,terminal-components-task-072-*} | 9 | UNRELATED prep/proof era | NEVER touch |
| W-CODEX | .worktrees/codex-* + .codex/worktrees/52aa/campaign | 4 | UNRELATED other-agent work | NEVER touch |
| W-VIS | visual-trust-* (2) + .worktrees non-prep (authority-readiness-docs, catalog-remap-071, ledger-contract-agent, main, proof-namespace-fix, review-c7-×3, task-007-inventory, visual-baseline-suite) (10) | 12 | UNRELATED | NEVER touch |
| W-OTHER-CLONES | terminal-components-2026-09-20 (not a repo); review-f949a56 (dirty staged tree) | 2 | UNKNOWN/UNRELATED | REVIEW_SHARED (do not touch) |
| MUSE-WT | .worktrees/campaign/.muse/worktrees/ | EMPTY dir | hygiene confirmed | NOT_APPLICABLE |

Count check: 1+1+1+2+8+36+73+4+25+24+9+4+12 = 200? No — W-014R3/W-014V3/W-079SERIES/W-PRIOR/W-TRIAGE-worktrees are the 44 non-b518 /tmp/tc-* worktrees (1+2+8+36 → see below; triage 3 + pgid 1 = 4 included in the 44). Correct sum: 118 exclusive (campaign 1 + b518 73 + other-tc 44) + 75 unrelated (MAIN 1 + prep 25 + grok 24 + tmpother 9 + codex 4 + vis 12) = 193. ✓ (The 44: 009:2, 010:6, 011:1, 012:4, 013:7, 014:7, 029:3, 073:1, 078:1, 079:8, triage:3, pgid-reap:1.)

Missing/inaccessible: none observed. `git worktree prune`/GC: NOT run.

### E.3 Local and remote branch ledger (full list in appendices/branches.txt)

34 local branches, 8 remote-only refs.

| ID | Ref | Tip | Upstream / remote head | Ahead/behind | Disposition |
|----|-----|-----|------------------------|--------------|-------------|
| B-CAMP | refactor/holla-parity (local) | 227f21b6 | origin/refactor/holla-parity = 227f21b6 (fresh fetch) | 0/0 | KEEP (goal branch) |
| B-PR6BASE | main (local tracking) | 7b27732a (PR base per gh) | — | — | KEEP (never merge in-goal) |
| B-VIS | visual-baseline branch + tag | tag obj 1ee5ebdc, peel 4a79c0a2 | — | — | KEEP, read-only (verified unchanged) |
| B-CK14R3 | goal-checkpoint/014-r3 (origin branch ref) | 80399a1e (ls-remote verified) | pushed pre-pause | durable | INTEGRATE (via 014 acceptance) then remove remote ref post-final |
| B-CK14R1 | goal-checkpoint/014-r1 (origin) | 01849854 (REJECTED r1; ls-remote verified) | pushed pre-pause | history only | KEEP until final, then remove |
| B-CK79R1/R2 | goal-checkpoint/079-r1/r2 (origin) | 5e625cff / cd9c395f (ls-remote verified) | pushed pre-pause | superseded (squash integrated) | KEEP until final, then remove |
| B-HO | goal-handoff/pr6-parity-c5d6e43a | (handoff commit; PR URL §A) | pushed with this handoff | handoff PR head | KEEP (durable record) |
| B-HIST | prep-*/proof-*/task-*/catalog-remap-*/baseline-canonical-5x5 (~30 local) | various old | mostly no upstream | superseded prep-era | REVIEW_SHARED later; NOT this handoff's scope |
| B-REMOTE-OTHER | red-main/perf-fix-forward, rollout/agent-policy, rollout/velnor-wave, visual-baseline (remote-only) | — | origin | — | UNRELATED (own PRs #9/#7/#8/#3); never touch |
| STASH | 3 shared-repo stashes: 408466e2 (071 WIP), verifier-temp, review-temp | local-only | — | at-risk | KEEP; convert/preserve on resume if needed |

No local branch has unpushed goal commits besides the above (campaign branch in sync).

### E.4 Related PR ledger (full list in appendices/prs.txt)

| ID | PR | State | Head/base | Checks/reviews | Action |
|----|----|-------|-----------|----------------|--------|
| PR-6 | #6 `Refactor/holla parity` https://github.com/donbeave/terminal-components-claude/pull/6 | OPEN, not draft, MERGEABLE, updated 2026-09-21T21:56:15Z | head refactor/holla-parity @ 227f21b6 (= local tip); base main @ 7b27732a | CI RED at e10fd942 (14 fail / 11 pass, §G); tip-227f21b6 CI unknown; reviewDecision empty; review THREADS re-read still pending (§D) | KEEP OPEN; continue landing onto its branch; NO merge in-goal |
| PR-HO | handoff DRAFT PR (this checkpoint) | DRAFT (created with this handoff) | head goal-handoff/pr6-parity-c5d6e43a; base refactor/holla-parity @ 227f21b6 | n/a (docs-only) | Retain as pause record; close only after resume supersedes |
| PR-3 | #3 plan: Phase 0 closure… (visual-baseline) | OPEN | base/head on protected oracle line | — | UNRELATED; never touch (protected) |
| PR-7 | #7 docs: adopt shared agent policy (rollout/agent-policy) | OPEN | — | — | UNRELATED rollout; no goal dependency; no action |
| PR-8 | #8 chore(ci): adopt schema-2 generated CI (rollout/velnor-wave) | OPEN | — | — | UNRELATED rollout; no goal dependency; no action |
| PR-9/5/4/2/1 | perf-fix (MERGED), Wave-readiness (CLOSED), TASK-001 tools (CLOSED), Holla (CLOSED), main-holla (MERGED) | closed/merged | — | — | Historical only; no action |

No other PR carries unintegrated goal code (worktree+branch evidence). PRs #7/#8/#3 are mapped and excluded with reason (resolves the "unmapped PRs" question).

### E.5 Integration map and ordered landing plan — FUTURE EXECUTION ONLY

Map: `W-014R3 → (no local branch; detached 80399a1e) → B-CK14R3 → (no PR; lands directly onto PR-6 branch) → refactor/holla-parity`. All other goal work is ALREADY on the target (ancestors of 227f21b6). Superseded refs (B-CK14R1, B-CK79R1/R2) are history-only, never to merge.

Ordered plan (execute ONLY after explicit resumption):

0. Re-observe: fetch origin, confirm tip still 227f21b6 (or reconcile), confirm oracle peel, confirm checkpoint ref 80399a1e resolves remotely. Triage PR #6 CI on the tip (§H.0).
1. Finish 014 r3 verification: re-run/complete the verifier protocol on 80399a1e in a FRESH worktree+RUN_DIR (do not trust stale /tmp if rebooted; if /tmp intact, the stopped verifier's logs in appendices/014-r3-logs/ may guide but the verdict must come from a completed run), write TASK-014-verify-r3.json, dispatch independent review, CAS-integrate onto tip, push.
2. Dispatch wave 015/016/017 (DAG-ready on 014), then subsequent waves per task-graph.json + receipts.
3. Final gates (§D VISUAL-FULL + AGENTS.md list) before any merge claim.
4. Handoff PR (PR-HO) stays open as the pause record; close it only when resumed work supersedes it (or retain permanently).

Merge method: fast-forward-only CAS per task (campaign rule); PR-6 itself is NOT merged in-goal.

### E.6 Post-integration local cleanup runbook — FUTURE EXECUTION ONLY (documented, NOT executed)

Candidates (re-verify LIVE state before each action; any drift → stop that deletion):

| Resource | Path/ref | Expected tip | Target | Proof needed | Gates | Action |
|----------|----------|--------------|--------|--------------|-------|--------|
| W-014R3, W-014V3 | /tmp/tc-014-r3/wt, /tmp/tc-014-vfy3/* | 80399a1e | PR-6 branch | 014 acceptance receipt + ancestor check | §H gates 0–1 | worktree remove + rm -rf /tmp dirs |
| W-079SERIES, W-PRIOR, W-B518 worktrees | /tmp/tc-*/wt, candidates | per-table SHAs | PR-6 branch | task receipts + ancestor checks | same | same, per-task after final gate |
| W-TRIAGE/W-ADOPT file dirs | /tmp/tc-{holla-triage,health,static,009-adopt,worktree-archive} | n/a (files) | handoff bundle / PR-6 history | evidence superseded by final seal | same | rm after final |
| B-CK* remote refs | goal-checkpoint/* (origin branches) | 4 SHAs | PR-6 branch | final integration ledger | + remote-cleanup authorization | delete remote branches post-final |
| B-HIST, W-PREP/GROK/TMPOTHER/CODEX/VIS, W-OTHER-CLONES, W-MAIN dirt, STASH | various | — | — | — | ownership UNKNOWN or UNRELATED | NEVER via this runbook |

Historical prep branches, other clones/eras, MAIN dirt, and stashes are EXCLUDED (shared/unknown/unrelated).

## F. Decisions, findings, assumptions, rejected approaches

- 078/079 repairs: see §D + package READMEs (078 receipt-binding; 079 fade_mix + D-008 ApplyDim rename — variant name is package-internal, oracle mandates behavior only; renaming to avoid the R-20 ratatui token is compliance, not D-10 evasion — independently reviewed).
- Single-commit hygiene enforced: 079 r2's 2-commit stack was squashed to tree-identical 227f21b6 before integration (intermediate carried the banned token).
- Nested tool worktrees pollute the bless-guard WalkDir (git-blind): hygiene removal done; future implementers use self-made /tmp worktrees; structural `.muse`-skip fits TASK-065/068 scope later.
- Holla 3 reds triaged PRE-EXISTING (bisected red at 1756c341, byte-identical signatures), recorded future-failed owners 041/049 — not regressions.
- Wedged-worker protocol (established this session): backup uncommitted work + logs → cancel → successor adopts; applied to 009 implementer and 010 verifier (both preserved, zero loss).
- CI-RED finding (new at pause): PR #6 CI fails 14/25 at e10fd942 — 8 per-crate Rust jobs + Boundary×2 + Library-perf×2 + Control/Required + ci-required. The 8 wave/repair commits after e10fd942 may already fix some; triage must run against tip 227f21b6, not the stale head.
- Task-path question resolved: contracts live at `refactoring-tasks/terminal-components/completion/{014,079}/` with `task.toml` + `verify.toml` (verified on disk); earlier path references were correct.
- Rejected: weakening extension.py gate (079-shape-a rejected — binder fix chosen); embedding accepted_* in templates (breaks single-source); stealing 068's doc scope for the registry lag (recorded transient instead); narrowing rule-16 regex for Dimmed (renaming is lower-risk, rule stays intact).
- Assumptions needing validation: NONE blocking; 014 r3 expected-verdict must still be sealed by a completed run (not assumed).

## G. Verification evidence and known failures

- Per-task: `taskfmt lint` 0 + `taskfmt verify` pass/fail=0 + ALL check shells exit 0 + focused nextest + rule/architecture gates, per receipts in `<id>/receipts/` (bundled with this handoff; originals /tmp/tc-goal-b518/receipts/).
- 079 squash: 6/6 checks, taskfmt 10/10, fade 4/4, theme 80/80, rule-16 PASS, full suite 2778/5-knowns (base-reproduced).
- 014 r1: REJECTED (2 arch gates, evidence in TASK-014-verify-r1.json). 014 r3: implementer 7/7 + 11/11 + 15/15 + both arch gates PASS (appendices/014-r3-implementer-evidence.md); verifier 95% same-green, verdict INTERRUPTED (not failed).
- Branch-health @ b4d2c0d0: 3294 pass / 9 known-class fails (4 viewport 021, registry 068, 3 holla staged, baseline-moves env — latter hygiene-cleared). Static: Lychee 779 inputs 0 errors; CLAUDE.md symlink OK.
- PR #6 CI (observed 2026-09-22 ~05:36Z via `gh pr checks 6`, captured in appendices/ci-checks.txt): RED at `e10fd942` ("fix(preflight): import proof schema validator dependencies"), 8 commits behind tip. FAIL (14): Boundary ×2, Control / Required, Library Performance ×2, Rust ×8 (terminal-components, tuisnap, tui, tui-snap, velnor-workflow, refactor-proof, olive-terminal, ratatui-terminal), ci-required. PASS (11): Bless guard ×2, Control / Planning, Doc check ×2, Markdown links ×2, Rustdoc ×2, Workspace nextest ×2. CI status of tip 227f21b6 UNKNOWN at pause — triage on the tip first (§H.0). (This supersedes the earlier "COMPLETED-state" note, which recorded states without conclusions.)
- NOT RUN: full 7,550/30,200 matrix; Linux native; final AGENTS.md gate battery. No failures hidden: every red is dispositioned to an owning task or this plan.

## H. Ordered remaining-work plan

0. FIRST (minutes): re-observe (§E.5 step 0) + triage PR #6 CI on tip 227f21b6: check whether runs completed on the tip, classify each red as fixed-by-wave / pre-existing-known / new, and record dispositions before dispatching workers.
1. FIRST TASK: seal TASK-014 r3 — fresh verifier run on `goal-checkpoint/014-r3` (80399a1e) → TASK-014-verify-r3.json → independent review → CAS integrate → push. Validation: §E.5 steps 0–1. (Do not reuse the interrupted run's verdict — none exists; its logs are guidance only.)
2. Wave 015/016/017 in parallel (overlap-free), then DAG-ordered waves to cover 018–069 (each: implement → verify → review → CAS integrate → push).
3. PR #6 review-thread re-read + dispositions (overlaps waves; must complete before final).
4. Doc rebinding: update stale NO-GO headers (readiness report, campaign-policy) to reflect executed GO + this pause.
5. Workflow-syntax validation command decision (WF-CMD) + run it.
6. Final acceptance battery (§12 of the goal: full matrix, PTY/lifecycle, perf, ownership scans, platform evidence, final independent verifier+adversarial reviewer).
7. Execute §E.5 integration leftovers (none expected beyond step 1) then §E.6 cleanup with live re-verification.
8. Merge-readiness statement; do NOT merge into main (out of goal).

## I. Environment and operational recovery

- Platform: native macOS arm64; repo /Users/donbeave/Projects/terminal-components-claude; campaign worktree .worktrees/campaign; shell `rtk` prefix preferred; cargo 1.98.1 + cargo-nextest 0.9.14x (re-verify on resume); taskfmt 0.2.0 / afd3b575dbcc7044620bec4b9493a74eca3e5ef2 / f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de (re-verify hash before use); Lychee 0.24.2 for the docs gate.
- Key paths: campaign checkout `.worktrees/campaign` (branch refactor/holla-parity); receipts originals `/tmp/tc-goal-b518/receipts/`; 014-r3 candidate `/tmp/tc-014-r3/wt`, verifier views `/tmp/tc-014-vfy3/{candidate,parent}`; task contracts `refactoring-tasks/terminal-components/completion/<NNN>/{task.toml,verify.toml}`; visual contract `refactoring-tasks/visual-validation.md`; campaign contracts `docs/refactoring-plan/*.md`.
- Oracle pins (re-verify before importing any oracle input): tag obj `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, peel `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, commit tree `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26`, snapshots tree `3f0261c32849e26feda24d87697de4a7ce6b8375`.
- Commit discipline (every commit): `git commit -s` + `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` + `Co-authored-by: Codex <codex@openai.com>`; fast-forward-only CAS integration (confirm expected parent == branch tip before landing).
- Resume command (send verbatim as a user message): `Resume the paused goal goal-b5184825 (handoff pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a): re-observe branch/oracle/CI state, triage PR #6 CI on the tip, seal TASK-014 r3 (verify → review → CAS integrate), then continue dependency-ordered waves per §H.`
- First resumption task: §H.0 (re-observe + CI triage on tip), then §H.1 (seal 014 r3).

## J. Handoff bundle contents (this branch)

- `docs/goal-handoffs/pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a.md` — this file (complete; no truncation).
- `docs/goal-handoffs/<id>/receipts/` — 68 acceptance/evidence JSONs (+ COORDINATOR-NOTES.md), copied from /tmp/tc-goal-b518/receipts/.
- `docs/goal-handoffs/<id>/appendices/worktrees.txt` — full 193-worktree enumeration with HEADs.
- `docs/goal-handoffs/<id>/appendices/branches.txt` — full local + remote branch list.
- `docs/goal-handoffs/<id>/appendices/tmp-tc-dirs.txt` — all 176 /tmp/tc-* entries.
- `docs/goal-handoffs/<id>/appendices/prs.txt` — all 9 PRs with states.
- `docs/goal-handoffs/<id>/appendices/ci-checks.txt` — PR #6 check conclusions at pause.
- `docs/goal-handoffs/<id>/appendices/receipts-manifest.txt` — receipts file list.
- `docs/goal-handoffs/<id>/appendices/014-r3-implementer-evidence.md` + `014-r3-logs/` — stopped-run guidance (NOT a verdict).

## K. Pause receipt

- Original goal title: Finish the complete refactoring on PR #6's branch with exact visual-baseline parity, functional parity, and independently verified integration.
- Pause/worker status: PAUSED_BY_USER; sole goal worker stopped with STOP ACK and terminal state; zero queued continuations; goal object administratively active-but-idle (no runtime pause API).
- HANDOFF path: `docs/goal-handoffs/pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a.md` on branch `goal-handoff/pr6-parity-c5d6e43a` (PR https://github.com/donbeave/terminal-components-claude/pull/10).
- Branch: `refactor/holla-parity` @ `227f21b6ba8a289ae585aa009b1a7d749a92aaf2` (pushed, 0/0); unintegrated checkpoint `80399a1e` on `origin/goal-checkpoint/014-r3` (+ history refs 014-r1, 079-r1, 079-r2).
- Preservation completeness: code pushed; checkpoints on remote refs; 68 receipts + 7 inventory/evidence captures bundled in the handoff PR. FULLY REMOTE-PORTABLE after the handoff PR lands.
- Inventory coverage: 193/193 worktrees enumerated (118 goal-exclusive, 75 unrelated/stale); 34 local + 8 remote-only branches; 176 /tmp/tc-* entries; 9/9 PRs mapped; 3 stashes; MAIN dirt recorded.
- Unmapped or local-only work: NONE goal-owned beyond the above (014 r3 is remote-checkpointed). Local-only non-goal items (stashes, MAIN dirt, review-f949a56 tree) recorded for exclusion.
- First resumption task: §H.0 re-observe + CI triage on tip, then §H.1 seal TASK-014 r3.
- Resume command: see §I.
- Merging and local cleanup were DOCUMENTED (§E.5, §E.6) for later execution and were NOT performed during this handoff.
