# Production task container paths (TASK-002+)

**Scope:** Tasks **002–069** and **073** (and hybrid **071–072** for bootstrap checks). Complements [`task-001-verify-container.md`](task-001-verify-container.md) and [`path-contract.md`](path-contract.md).

---

## Why these paths exist

Production `verify.toml` files invoke the accepted harness worker:

```text
/proof/bin/tc-proof <operation> --context /run/tc-proof/contexts/CHK-NNN.json
```

The host (`tc-proof-host`) must install `/proof/bin/tc-proof` after TASK-001 receipt and materialize frozen contexts under `/run/tc-proof/` before `taskfmt verify` runs.

**taskfmt does not create these paths.** `prepare` and `freeze` do.

---

## Required container layout (TASK-002+)

| Container path | Host source | Required for |
| --- | --- | --- |
| `/task/` | Catalog package `completion/NNN` | taskfmt lint, package checks |
| `/work/` | Frozen candidate checkout | scope, subprocess CWD |
| `/progress/progress.md` | Host-frozen progress | progress gate |
| `/proof/bin/tc-proof` | Accepted harness install | CHK-001–007 (tc-proof operations) |
| `/run/tc-proof/context-index.json` | Host `prepare`/`freeze` | context binding |
| `/run/tc-proof/contexts/CHK-NNN.json` | One per check in verify.toml | each tc-proof invocation |

Bootstrap tasks **001** and **070** do not use `/proof/bin/` or `/run/tc-proof/` in verify.toml. Tasks **071** and **072** use both bootstrap and production paths (hybrid).

---

## Host sequence (not taskfmt run/promote)

```sh
tc-proof-host prepare --campaign "$CAMPAIGN" \
  --task terminal-components/completion/NNN \
  --parent "$INTEGRATION_PARENT" \
  --run "$RUN"

# executor completes /work edits and progress …

tc-proof-host freeze --run "$RUN" --candidate "$WORKTREE"
tc-proof-host verify --run "$RUN"
```

Inside `verify`, the host runs pinned `taskfmt verify` with container paths visible. Individual checks spawn `/proof/bin/tc-proof` per frozen context.

---

## Common mistakes

| Mistake | Result |
| --- | --- |
| Running `taskfmt verify` with only `--task-dir` and `--root` | CHK fail: `/proof/bin/tc-proof` not found |
| Using `/work/tools/refactor-proof/bin/tc-proof` in production tasks | Wrong — production uses `/proof/bin/tc-proof` after install receipt |
| Expecting `target/debug/tc-proof` to satisfy production verify.toml | Wrong — only bootstrap tasks use `/work/tools/.../bin/` |
| Confusing `$TC_RUN` with `/run/tc-proof/` | Host maps run dir → container `/run/tc-proof/` at verify time |

---

## Validation

- Planning: `validate-plan.py` `catalog_argv_smoke()` — 479 `/proof/bin/tc-proof` argv references across packages (does not invoke binary).
- Runtime: host must prove contexts exist and match `verify.toml` check IDs before verify.
