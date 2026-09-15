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

Do not arm `/goal` until `docs/refactoring-plan/campaign-pre-arm-checklist.md` is complete.
