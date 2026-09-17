# Verification path contract

**Authority:** This document is the single canonical reference for filesystem paths used by `taskfmt verify` and `tc-proof-host`. Task packages (`verify.toml`) use **container paths** literally; taskfmt does **not** rewrite subprocess argv.

Related: [`task-production-verify-container.md`](task-production-verify-container.md), [`proof-contract.md`](proof-contract.md).

---

## Three namespaces

| Namespace | Examples | Who owns it |
| --- | --- | --- |
| **Container verification** | `/task`, `/work`, `/proof/bootstrap`, `/proof/bin`, `/run/tc-proof` | Host must expose at filesystem root before checks run |
| **Host operator** | `$REPO`, `.worktrees/campaign`, `/tmp/taskfmt-latest-install`, `$TC_RUN` | Operator maps into container namespace |
| **Campaign store** | `campaign/` or `.campaign/host/` (bootstrap, catalog, runs, ledger) | Host only; never candidate-writable |

---

## taskfmt behavior (no argv rewriting)

`taskfmt verify --root`, `--task-dir`, and `TASKFMT_*` env vars affect **taskfmt's own** gate (lint, scope, forbidden paths, progress). They do **not** change `verify.toml` check argv.

Every subprocess in `verify.toml` runs with:

- `argv` exactly as written in the package
- `current_dir` = `--root` (typically `/work`)

**Passing `--task-dir /path/to/catalog/001` does not make `/task/...` resolve.** Root-level mounts (or a sandbox script) are mandatory.

---

## TASK-001 bootstrap layout (001, 070)

Required at filesystem root when CHK checks execute:

```text
/task/                                    → read-only task package (completion/NNN)
/work/                                    → candidate git checkout (taskfmt CWD)
/proof/bootstrap/bin/taskfmt              → pinned standalone taskfmt
/proof/bootstrap/task-format/              → source @ afd3b575…
/work/tools/refactor-proof/bin/tc-proof     → synced Mach-O (after sync-binaries.sh)
/work/tools/refactor-proof/bin/tc-proof-host → synced Mach-O (after sync-binaries.sh)
```

Do not provision root firmlinks with a repository script. The old sandbox
staging scripts were retired because they required interactive root changes
and could not qualify the macOS host path. The current host adapter must map
these namespaces in an isolated per-run directory before invoking taskfmt.

### Harness binary roles

| Path | Role |
| --- | --- |
| `target/debug/tc-proof-host` | Built Mach-O — **advisory** driver runs on bare macOS only |
| `tools/refactor-proof/bin/tc-proof-host` | Synced Mach-O — **required** for `verify.toml` CHK-005/006/007 (container path `/work/tools/.../bin/tc-proof-host`) |
| `scripts/dev-tc-proof-host.sh` | Shell wrapper — dev only; never sync to `bin/` |

---

## TASK-002+ production layout (002–069, 073; hybrid 071–072)

Required at filesystem root:

```text
/task/                                    → read-only task package
/work/                                    → frozen candidate checkout
/progress/progress.md                     → frozen nonempty progress (or explicit --progress path)
/proof/bin/tc-proof                       → host-installed accepted harness worker
/run/tc-proof/context-index.json          → frozen by host prepare/freeze
/run/tc-proof/contexts/CHK-NNN.json       → one per declared check
```

Host `tc-proof-host prepare` materializes contexts; `freeze` binds the candidate tree. See [`task-production-verify-container.md`](task-production-verify-container.md).

### Task bands in verify.toml

| Tasks | Check argv pattern |
| --- | --- |
| **001, 070** | Bootstrap Python under `/task/trusted/…`; `/proof/bootstrap/` for taskfmt |
| **071, 072** | Hybrid: CHK-001 → `/proof/bin/tc-proof preflight`; others → bootstrap drivers |
| **002–069, 073** | All checks → `/proof/bin/tc-proof <op> --context /run/tc-proof/contexts/CHK-NNN.json` |

---

## Worktree contract (campaign)

| Phase | Worktree | Branch |
| --- | --- | --- |
| Campaign execution | `.worktrees/campaign` | `refactor/holla-parity` |
| Planning catalog (read-only) | repo checkout | `prep-wave1-verify` (archive after SHA recorded) |

Historical prep-wave1 docs may reference `.worktrees/main` @ `task-001-bootstrap`; current production work, if later authorized, uses `.worktrees/campaign`.

---

## taskfmt verify invocation (canonical)

After container mounts exist:

```sh
cd /work
taskfmt verify --root /work --task-dir /task \
  --base "$SCOPE_BASE" \
  --progress "$PROGRESS" \
  --log-dir "$LOG_DIR"
```

Optional `--root /work --task-dir /task` for taskfmt internal resolution only.

---

## Host run directory (operator)

```text
$TC_RUN/                    # e.g. /tmp/tc-task-001-run-* or campaign/runs/<id>/
  progress.md
  taskfmt-logs/CHK-*.log
  contexts/                 # host-side; mount as /run/tc-proof/contexts/
  freeze.json
  verdict.json
```

`$TC_RUN` is **not** the same as container `/run/tc-proof/` — the host maps between them.
