# GOAL: Finish the complete refactoring on PR #6's branch with exact visual-baseline parity, functional parity, and independently verified integration

## A. Identity and pause status

- Handoff ID: `pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a`
- Created (UTC): 2026-09-21T22:03:12Z | Last update: see handoff-branch commit history (doc finalized 2026-09-22 ~05:40Z)
- Original goal status: `PAUSED_BY_USER`
- Handoff status: `READY` (handoff PR #10 published)
- Runtime/worker stop: sole goal worker (014-r3 verifier) sent the pause stop message, returned a STOP ACK status report, and reached terminal state (verified via runtime subagent_result delivery). No other goal workers were running. No runtime goal-pause API is exposed to this agent (only completion/blocked transitions exist); the goal object therefore remains administratively active-but-idle. Pause is effected by: stopped workers + zero queued continuations + this checkpoint. This is a VERIFIED worker stop with an UNSUPPORTED runtime-control limitation, recorded honestly.
- Source agent: Muse Code (CLI) | Session 01a0bfe4-be83-7db0-bcd4-38f0357dda92 | Goal goal-b5184825
- Repository: `donbeave/terminal-components-claude` (github.com:donbeave/terminal-components-claude.git)
- Handoff path: `docs/goal-handoffs/pr6-refactor-holla-parity--20260921T220312Z--muse-code--c5d6e43a.md`
- Source branch / HEAD: `refactor/holla-parity` @ `227f21b6ba8a289ae585aa009b1a7d749a92aaf2` (pushed; in sync with origin at pause)
- Checkpoint code SHAs: branch tip `227f21b6` (= 079 squash-integrated); unintegrated worker checkpoint `80399a1ebf7020bcdb76ff3d48f89cd7e2e71c8b` (014 r3, preserved on remote ref, §E.3; parent == tip, CAS still valid)
- Preservation branch: `goal-handoff/pr6-parity-c5d6e43a` (this handoff + receipts bundle + appendix captures); PR base `refactor/holla-parity` @ `227f21b6`; PR URL: https://github.com/donbeave/terminal-components-claude/pull/10
- Recovery: FULLY REMOTE-PORTABLE after this handoff lands (code pushed, worker checkpoints on remote refs, receipts + inventory captures bundled in the handoff PR). Pre-handoff state depended on `/tmp` (see §E).
- Resume authorization: explicit later user request only.

`PAUSED_BY_USER` is the requested disposition, not proof of remote-job cessation. No remote jobs were running (all work is local subprocesses; PR #6 CI runs are automatic on push, not campaign dispatches; CI observed RED at an older head, §G).

## B. Original goal and success contract

### Recovered original objective (verbatim gist, secrets redacted)

> Finish the complete refactoring on PR #6's branch, restore exact visual and functional parity with the immutable visual-baseline, and independently verify the finished implementation. Required outcome: COMPLETE INTENDED REFACTORING AND EXACT BASELINE VISUAL PARITY AND EXACT BASELINE FUNCTIONAL PARITY AND INDEPENDENTLY VERIFIED INTEGRATION. Implementation-and-completion goal; work autonomously through repair/verification iterations until genuinely satisfied.

Full 13-section objective (scope, authorization, identities, reconciliation, verification repair, parallel execution, architecture, visual matrix, functional parity, per-task loop, native tooling, final acceptance, persistence/handoff) is preserved in the goal record; the consolidated statement below incorporates amendments. (Reconstructed summaries are marked [S]; recovered quotations marked [Q].)

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
