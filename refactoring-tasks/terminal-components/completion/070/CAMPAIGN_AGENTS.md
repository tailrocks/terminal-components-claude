# Campaign execution protocol overlay

**Campaign worktree:** `.worktrees/campaign` on branch `refactor/holla-parity` (host maps to container `/work/`).

This file is the campaign-visible executor protocol for terminal-components. It preserves canonical `/task/AGENTS.md` provenance at taskfmt revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede` but **supersedes AGENTS.md steps 6–7**, the executor-authoritative ordered-check sequence, and the prohibition on claiming `DONE` without local `taskfmt verify` when those steps conflict with campaign host handoff.

Full operator/host contract: [campaign-executor-protocol.md](/work/docs/refactoring-plan/campaign-executor-protocol.md).

**Container paths:** `verify.toml` argv uses literal `/task`, `/work`, `/proof/bootstrap`, `/proof/bin`, `/run/tc-proof` — see [path-contract.md](/work/docs/refactoring-plan/path-contract.md). taskfmt `--task-dir` / `--root` do **not** rewrite check subprocess paths.

## Verify environment by task band

Host must expose container paths at filesystem root before `verify.toml` checks run. Do not substitute `--task-dir` / `--root` for mounts.

| Band | Tasks | Required read-only paths |
| --- | --- | --- |
| Bootstrap | 001, 070 | `/task/`, `/work/`, `/proof/bootstrap/bin/taskfmt`, `/proof/bootstrap/task-format/`, `/proof/bootstrap/experiment.toml`, `/work/tools/refactor-proof/bin/tc-proof` (+ `tc-proof-host` for 001) |
| Hybrid | 071, 072 | Above bootstrap paths **plus** `/proof/bin/tc-proof` and `/run/tc-proof/contexts/CHK-001.json` for CHK-001 preflight |
| Production | 002–069, 073 | `/task/`, `/work/`, `/proof/bootstrap/bin/taskfmt`, `/proof/bootstrap/experiment.toml`, `/proof/bin/tc-proof`, `/run/tc-proof/context-index.json`, `/run/tc-proof/contexts/CHK-NNN.json` per declared check |

Pinned taskfmt gate: `/proof/bootstrap/bin/taskfmt` with config `/proof/bootstrap/experiment.toml`. Run from `/work/`.

## Verification authority

- Local compiler/tests are **advisory only**. They must not use production expected bundles, impersonate `/proof/bin`, create accepted receipts, or authorize completion.
- **Prohibited:** `taskfmt verify --progress ""` and any progress-disabled invocation that claims completion or integration authority.
- **Required:** After implementation leaves complete, write the terminal progress event, submit candidate/progress to the operator, and await host `freeze` + `verify` with immutable per-check contexts, pinned configuration, explicit base, and **nonempty frozen progress**.
- Report the host-authentic verdict in the final `VERIFY:` line. A copied, forged, stale, or cross-run host report cannot authorize integration.

## TASK-001 bootstrap exception

TASK-001 cannot require its own future accepted host executable. For that task only, the trusted operator performs initial checkout/freeze/progress handling outside the executor, invokes pinned standalone taskfmt with explicit configuration/base/**nonempty** progress, and runs planner-frozen comparator and host bootstrap drivers. See campaign-executor-protocol.md § Initial bootstrap exception.

## Completion sequence (TASK-002+)

1. Finish scoped edits in `/work/` and append the terminal progress event to `/progress/progress.md`.
2. Place progress and a plain completion request in the operator-visible outbound directory.
3. Operator runs `tc-proof-host freeze` then `tc-proof-host verify` on the frozen tree.
4. Executor reports the returned host verdict faithfully; revise and resubmit on failure. Every revision requires a fresh full frozen gate.

## Final report

When reporting completion, the `VERIFY:` line must cite the **host-authentic** `tc-proof-host verify` result for this run, not a local `--progress ""` invocation.
