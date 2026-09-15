# Pre-arm checklist — prepare without starting `/goal`

Complete every section before arming [`campaign-execution-prompt.md`](campaign-execution-prompt.md). Preparation scripts live under `scripts/campaign-*.sh`.

**Policy:** [`campaign-policy.md`](campaign-policy.md) — single branch `refactor/holla-parity`, one repo.

---

## Phase P0 — Branch and workspace

| # | Item | Command / evidence | Done |
| --- | --- | --- | --- |
| P0-1 | Campaign init run | `scripts/campaign-init.sh` | [ ] |
| P0-1b | Planning catalog on campaign branch | `scripts/campaign-absorb-planning.sh` then commit on `refactor/holla-parity` | [ ] |
| P0-2 | Integration branch exists | `git rev-parse refactor/holla-parity` | [ ] |
| P0-3 | Worktree at `.worktrees/campaign` | `git -C .worktrees/campaign branch --show-current` → `refactor/holla-parity` | [ ] |
| P0-4 | Tag unmoved | `git rev-parse refs/tags/visual-baseline^{commit}` → `4a79c0a2…` | [ ] |
| P0-5 | Catalog SHA recorded in ledger | `.campaign/ledger.json` → `catalog.commit` | [ ] |
| P0-6 | Retire multi-branch PR targets | Close or supersede PR #3/#4/#5; push only `refactor/holla-parity` | [ ] |

---

## Phase P1 — Toolchain pins

| # | Item | Command / evidence | Done |
| --- | --- | --- | --- |
| P1-1 | taskfmt installed @ `52d9f1eb` | `scripts/campaign-install-taskfmt.sh` then `taskfmt fingerprint` | [ ] |
| P1-2 | tui-snap release binary built | path recorded in ledger `toolchain.tuisnap_path` | [ ] |
| P1-3 | Catalog lint green | `taskfmt --config … project lint terminal-components --projects-root refactoring-tasks/terminal-components` → 73/73 | [ ] |
| P1-4 | validate-plan green | `python3 docs/refactoring-plan/evidence/validate-plan.py --summary` → `error_count: 0` | [ ] |

---

## Phase P2 — Harness bootstrap (TASK-001)

| # | Item | Command / evidence | Done |
| --- | --- | --- | --- |
| P2-1 | `refactor-proof` builds on campaign branch | `cargo build -p refactor-proof` in worktree | [ ] |
| P2-2 | Mach-O harness synced | `tools/refactor-proof/scripts/sync-binaries.sh` | [ ] |
| P2-3 | Comparator driver 141/141 | `proof-comparator-bootstrap.py` log in evidence dir | [ ] |
| P2-4 | Host matrix 63/63 (Darwin) | Advisory: `--host target/debug/tc-proof-host`. Gate (verify.toml): `/work/tools/refactor-proof/bin/tc-proof-host` after `sync-binaries.sh` — see [`path-contract.md`](path-contract.md) | [ ] |
| P2-5 | IW-03 checklist complete | [`task-001-operator-evidence-template.md`](task-001-operator-evidence-template.md) EV-001–EV-027, SO-001–SO-007 | [ ] |
| P2-6 | Standalone `taskfmt verify` DONE | Runbook §6 after `task-001-verify-sandbox.sh mount` — container paths `/task`, `/work`, `/proof/bootstrap` | [ ] |
| P2-7 | First `qualified-harness` receipt | EV-027 populated; ledger `receipts.task-001` | [ ] |
| P2-8 | First integrate on campaign branch | `ledger.integration_head` advanced; parent was architectural main or prior tip | [ ] |

---

## Phase P3 — Orchestration (autonomous prep)

| # | Item | Command / evidence | Done |
| --- | --- | --- | --- |
| P3-1 | Preflight script green | `scripts/campaign-preflight.sh` exit 0 | [ ] |
| P3-2 | Cursor rule loaded | `.cursor/rules/campaign-execution.mdc` | [ ] |
| P3-3 | Campaign ledger initialized | `.campaign/ledger.json` matches [`campaign-ledger.schema.json`](campaign-ledger.schema.json) | [ ] |
| P3-4 | Dispatch script smoke | `scripts/campaign-dispatch.sh status` | [ ] |
| P3-5 | CI targets campaign branch | workflow triggers on `refactor/holla-parity` (optional until push) | [ ] |

---

## Phase P4 — Explicit arm authorization (do last)

| # | Item | Done |
| --- | --- | --- |
| P4-1 | `READY FOR REFACTORING EXECUTION` issued on recorded catalog SHA | [ ] |
| P4-2 | Operator explicitly authorizes arming `/goal` (issuance alone is insufficient) | [ ] |
| P4-3 | Copy `/goal` body from [`campaign-execution-prompt.md`](campaign-execution-prompt.md) only | [ ] |
| P4-4 | Append single-branch policy paragraph from [`campaign-policy.md`](campaign-policy.md) to goal | [ ] |

**Do not proceed to P4 until P0–P3 are complete.**

---

## Quick commands

```sh
# One-time setup
./scripts/campaign-init.sh
./scripts/campaign-install-taskfmt.sh
./scripts/campaign-preflight.sh

# After TASK-001 receipt exists
./scripts/campaign-dispatch.sh status
```
