@/Users/donbeave/.codex-chainargos2/RTK.md
@/Users/donbeave/.codex-chainargos2/CODEX_POLICY.md

# Termrock launch rules

## Start and scope

- `PLAN_ROOT` is this launch directory. It contains plans, not the Termrock Rust workspace.
- Read all package documents before dispatch. Start with [README.md](README.md) and the complete [GOAL.md](GOAL.md).
- Follow [PLAN.md](PLAN.md), [TASKS.md](TASKS.md), [task-index.json](task-index.json), and [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md).
- Preserve [API-RULES.md](API-RULES.md), [ARCHITECTURE.md](ARCHITECTURE.md), [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md), and [VALIDATION.md](VALIDATION.md).
- Follow the separate checkout and instruction import steps in README before T00.
- Never run Cargo or T37 cleanup in `PLAN_ROOT`. Limit source, test, and comment changes to the candidate checkout.
- Preserve user changes, target instructions, immutable references, production diagnostics, safety contracts, and unrelated APIs.

## Models and delegation

- Never use Astra, including fallbacks. Coordinate with `gpt-6.1-sol`, low effort.
- Use `gpt-6-luna`, max effort, for every implementation, including tests, configuration, scripts, and documentation.
- Use independent `gpt-6.1-sol`, high effort, for product architecture and API, runtime, safety, dependency, performance, and acceptance reviews.
- Use independent `gpt-6.1-sol`, medium effort, for all test architecture, test-plan, test-diff, and escaped-test-gap work.
- Set model and effort in agent tools. Use `fork_turns="none"` when required. Record agent IDs and selection limits.
- Block a task if its required model or effort is unavailable. Never silently substitute a role, model, or effort.
- Use bounded parallel tasks with exclusive file ownership. Reserve review capacity. Do not spawn nested agents by default.
- Give each worker complete task context, fixed input SHA, owned files, contracts, approved cases, and output requirements.

## Gates and evidence

- Complete T00, T37, T01, and T38 in order. Block T02, T03, and T04 until T38 passes independent review.
- Have one Sol medium agent design T38. Have a different Sol medium agent review it.
- Follow [TESTING-RULES.md](TESTING-RULES.md) and [PRACTICE-REVIEW.md](PRACTICE-REVIEW.md). Approve each later test plan before test edits.
- T37 is one-time cleanup. Reintroduce tests only through reviewed plans. Zero-case behavior lanes are `NOT VERIFIED`.
- Preflight tasks can pass their scoped document, structural, and build gates. They cannot prove product behavior.
- Use real public production paths, observable outcomes, deterministic cases, and independent red-green proof. Never mock owned code.
- Preserve required directives, legal text, explicit Clap help, and generator-verified bytes. Keep safety rationale in maintained contracts.
- Do not weaken unsafe-code lints. Require a scoped Sol high product decision for a conflict.
- Follow the Rust toolchain skill and `/Users/donbeave/.agents/DISK_POLICY.md` before target Rust builds.
- Complete all 39 tasks. Verify final acceptance gates at one source revision, including all 54 routes, W01–W45, additional public components, and four applications.
- A build alone cannot finish the goal. Record actual commands, cases, source SHAs, limits, and independent decisions.
- Commit verified units. Re-fetch feedback at the exact candidate head. Follow current user authorization for every external action.
