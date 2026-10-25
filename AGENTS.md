# Termrock agent rules

## Work order

- Never use Astra in any task, review, fallback, or planning step.
- Use `gpt-6-luna` with max effort for implementation.
- Use independent `gpt-6.1-sol` reviewers. Use high effort for architecture and test design.
- Read [the goal](GOAL.md), [the runbook](AGENT-RUNBOOK.md), [the tasks](TASKS.md), and [the API rules](API-RULES.md).
- Complete T00, T37, T01, and T38 in order. Keep T02, T03, and T04 blocked until T38 passes review.
- Have one Sol high agent design T38. Have a different Sol high agent review it.
- Treat T37 as one-time test cleanup. Tests may return after T38 and each test plan pass review.
- Treat every zero-test result as `NOT VERIFIED`. It cannot pass a required gate.

## Test and code rules

- Follow [the testing rules](TESTING-RULES.md) and [the practice review](PRACTICE-REVIEW.md).
- These rules govern test and comment conflicts in older plan files. Keep unrelated contracts in force.
- Before every new or refactored test, require an independent Sol high review of its plan.
- Test required behavior through a real production path and public API. Assert observable results.
- Do not mock, fake, or stub code owned by this repository.
- Do not use test count, coverage, or speed as a quality KPI.
- Keep scenarios deterministic. Verify each test independently before you keep it.
- After an issue escapes, require a Sol high architect subagent to analyze the testability gap before test changes.
- Inspect source to establish actual behavior. Use approved contracts to establish required behavior.
- Fix the structure that enabled a verified defect when evidence supports that change.
- Keep T37 limited to test cleanup. Preserve unrelated APIs, safety rules, and production diagnostics.
- Leave no active test-only file, fixture, filter, inventory row, or citation after T37.
- Keep source free of nonfunctional comments. Preserve required tool text and generator-verified bytes.
- Use explicit Clap help attributes. Put essential safety rationale in maintained contracts.
- Do not weaken unsafe-code lints. Stop for a scoped Sol high decision if a required lint conflicts.

## Delegation and review

- Use subagents for parallel research, implementation, review, and independent verification.
- Give each implementation agent exclusive file ownership.
- Commit verified changes in small units. Push only with current user authorization.
- Re-fetch feedback at the exact candidate head. Disposition every current finding with evidence.
- Do not send public replies, merge, or release without current explicit authorization.
- Keep instructions short. Put plans, findings, and evidence in linked documents.
