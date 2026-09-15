# Campaign policy — single branch, pre-arm

**Status:** Preparation only. **Do not arm `/goal`** until every item in [`campaign-pre-arm-checklist.md`](campaign-pre-arm-checklist.md) is green and the operator explicitly authorizes arming.

## Integration branch (sole production line)

| Ref | Role |
| --- | --- |
| **`refs/heads/refactor/holla-parity`** | Only advancing branch for all 73 tasks |
| **`main` @ `7b27732a8c3c131760ec3438f641cb3c11343a42`** | Architectural parent at campaign start; untouched until final authorized merge |
| **Tag `visual-baseline` (peeled `4a79c0a2`)** | Frozen oracle pin — never move, retarget, or recreate |
| **Branch `visual-baseline` / `prep-wave1-verify`** | Planning archive — record catalog SHA, then stop advancing |

Retired as long-lived targets: `task-001-bootstrap`, open PR #3/#4/#5 merge strategy.

## Repository scope

- **One repo:** `terminal-components-claude` — all production commits land on `refactor/holla-parity`.
- **Tools (not work product):** [task-format](https://github.com/donbeave/task-format) @ `52d9f1eb…` and tui-snap @ campaign pin — installed binaries only.

## Worktrees

Ephemeral worktrees are allowed. They must checkout **`refactor/holla-parity`** at an explicit parent SHA — not separate long-lived branch names.

Default campaign worktree: `.worktrees/campaign` (created by [`scripts/campaign-init.sh`](../../scripts/campaign-init.sh)).

## Host integration

Every accepted task integrates with compare-and-swap:

```sh
tc-proof-host integrate --run "$RUN" \
  --ref refs/heads/refactor/holla-parity \
  --expected-parent "$PARENT_SHA"
```

## What this policy does not authorize

- Arming `/goal` or starting TASK-002+ production dispatch
- Merging or pushing to `main`
- Moving the `visual-baseline` tag or writing `snapshots/`
- Using `taskfmt run`, `monitor`, or `promote`

## Verification paths

Container paths in `verify.toml` are literal; see [`path-contract.md`](path-contract.md). Campaign worktree: `.worktrees/campaign` (not `.worktrees/main` / `task-001-bootstrap`).

## Related docs

- Path contract: [`path-contract.md`](path-contract.md)
- Pre-arm checklist: [`campaign-pre-arm-checklist.md`](campaign-pre-arm-checklist.md)
- Ledger schema: [`campaign-ledger.schema.json`](campaign-ledger.schema.json)
- Arm prompt (use only after checklist): [`campaign-execution-prompt.md`](campaign-execution-prompt.md)
- Executor protocol: [`campaign-executor-protocol.md`](campaign-executor-protocol.md)
