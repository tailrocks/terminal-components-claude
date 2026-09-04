# Resume prompt — Complete Rust TUI Component-System Refactor

Paste this as the first message of the new session.

---

Continue the in-progress refactor in this repository. Do not restart from scratch and do not re-audit what is already recorded.

## Read first, in this order

1. `REFACTORING_STATE.md` — the durable ledger. Its "Next action (Resume)" section is your plan.
2. `GOAL.md` and `REFACTORING_GOAL.md` — the contract (definition of done is §29; final report is §30).
3. `COMPONENT_ARCHITECTURE.md` — the accepted architecture, §1–§24 plus Adjudications A–M. Change control: any change to a Decision, invariant, exact type or precedence rule needs fresh `opus-analyst` adjudication recorded in the document and the ledger.
4. `docs/reviews/slice3-foundations-review.md` — accepted; its §2 (eight adjudications), §3 (deviations D-1…D-13), §5 (fix list F1–F26) and §6 (Slice 3 gate commands) are binding.
5. `docs/reviews/adjudication-n-layer-measure.md` — accepted; its "Document amendments" table is binding.
6. `docs/audit/*.md` — the six Slice 1 audits plus the modern-API audit (dependency and API policy).

## Mandatory execution model (unchanged)

- Coordinator and every implementation worker: **`claude-opus-5`, effort `high`** — a user-authorized deviation from the goal's `claude-fable-5-1` mandate, because Fable 5.1 credits were exhausted. `.claude/agents/{refactor-coordinator,fable-builder}.md` and `.claude/settings.json` are already repointed; agent definitions own routing, so never pass a per-invocation model override. Revert all three to `claude-fable-5-1` if Fable capacity returns.
- Every audit, research question, architecture decision, alternative comparison, root-cause diagnosis, public-API or test-design critique, domain-boundary decision, security analysis, performance interpretation, visual judgment and independent verification: a fresh, read-only `opus-analyst` (`claude-opus-5`, high).
- All implementation through `fable-builder` subagents. The coordinator only spawns agents, records results into `REFACTORING_STATE.md` and the architecture document, commits, and reports. User directive: **all audits and implementations run in subagents.**
- Never use generic, inheriting, built-in Explore or built-in Plan agents. Never override configured agent models or effort.
- Parallel builders must have explicit, disjoint file ownership; keep shared foundations under one owner.
- User directives to keep: use the latest dependency versions and their latest APIs and modern practices; commit and push `origin/main` after every recorded result.

## Recorded blocker and its user-authorized resolution

The previous session ended because the Fable 5.1 monthly spend limit was reached — three `fable-builder` agents were terminated mid-task (HTTP 429). The goal treats an unavailable required model as a blocker. The user resolved it by directing that the work continue on Opus 5 only. The routing files are already updated; surface this deviation in the final report rather than treating it as unresolved.

## Immediate state (session 2 ended at a token limit)

The tree does **not** compile at HEAD. Three Slice-4 builders were killed mid-work and their partial output is committed as WIP; `cargo build -p junie-tui --all-targets` fails with one `E0502`. The last fully green commit is `0f66160` (797 tests, exit 0). Start by reading `REFACTORING_STATE.md`'s "SESSION 2 INTERRUPTION" block, then `git diff 0f66160 -- crates/` to see what the three builders produced, and decide per file whether to finish or revert. Reverting an unfinished component and re-running its package is often faster than repairing a half-written file.

## Earlier state

- HEAD `69fcdca`, pushed to `origin/main`, worktree clean.
- Slice 1 (baseline + six audits + 499 before-captures + TablePro/Jackin digests + perf baseline, tag `perf/baseline`) is complete.
- Slice 2 (architecture, independent review, corrections §21–§24) is complete.
- Slice 3 foundations exist at `crates/tui` (package `junie-tui`, lib `junie_tui` — a deliberate temporary name renamed to `junie-tui`/`junie_tui` at Slice 5), `crates/tui-testing`, `xtask`. They were green at commit `18afddd` and were then independently reviewed; **the F1–F26 corrections and Adjudication N code changes are not applied**.
- Two tasks were interrupted and their partial output is committed as WIP: the prototype components (`crates/tui/src/components/**`, examples 01/05–11) may not compile or pass gates, and the §25/§26 architecture amendments were not appended.
- The legacy package at the repository root still builds and its 247 tests pass; it must stay green until Slices 5–7 migrate the applications.

## First actions

1. Establish ground truth: `git status`, `cargo test -p junie-tui -p junie-tui-testing --all-targets --all-features`, `cargo run -p xtask -- boundary`, `cargo test --all-targets`.
2. Then follow `REFACTORING_STATE.md` → "Next action (Resume)" steps 2–6, updating the ledger and pushing after each result.

At every turn end, report completed evidence, unresolved criteria and active blockers, and do not claim completion without surfaced command results and review findings.
