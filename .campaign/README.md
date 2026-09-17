# Campaign runtime state (local, gitignored)

This directory holds operator-maintained campaign state. It is **not** candidate authority and is **not** committed.

| File | Purpose |
| --- | --- |
| `ledger.json` | Resumable campaign ledger — see `docs/refactoring-plan/campaign-ledger.schema.json` |
| `runs/` | Per-task host run directories (optional mirror of host `campaign/runs/`) |
| `evidence/` | IW-03 and operator evidence exports |

Initialize with:

```sh
./scripts/campaign-init.sh
```

This directory is local operator state, not authority. The old pre-arm checklist
is superseded. Read
[`docs/refactoring-plan/execution-readiness-report.md`](../docs/refactoring-plan/execution-readiness-report.md)
and follow its current NO-GO/GO conditions; no container or taskfmt lifecycle
arming path exists.
