# Worktree sync operation note

This is a derivative note, **not a raw execution receipt**. It records Root's handoff of the operation and tool result; no original raw log file was supplied or created here.

## Root-reported operation

- Working directory: `/Users/donbeave/Projects/tailrocks/terminal-components-claude`
- Command:

```sh
git restore --source=1d66f444f5962414b7c33734fa37376471e5f853 --worktree -- \
  STATUS.md \
  crates/termrock-visibility-tests/tests/status.rs \
  tools/visibility/source-facts.json \
  tools/visibility/status.py \
  docs/implementation/visibility/evidence/reports/status-source-archive-20261009
```

- Root-reported execution chunk `4accef`: exit 0, empty output.
- Root-reported before-guard chunk `764875`: exit 0, `mode=before`, `PASS`, 64 paths.
- Root-reported after-guard chunk `ec3128`: exit 0, `mode=after`, `PASS`, 64 paths.
- The two Root-reported guards preserve HEAD `766ae1e925e32b4b28aa9100bb3fde5279aa223b`, index SHA-256 `bdb1c1ea3935936eeaaaa4dadd3728f4490c4e2b20bbdc8f7decf8e7be633ed8`, excluded fixture SHA-256 `771b3dff1fbf53a60d9b90ae6cc3934bc411bb3ef953bfbda31b5fb2dfca360c`, and publication commit/tree `1d66f444f5962414b7c33734fa37376471e5f853` / `0ae9e285e8f771b8e250c63bf80f902d112dcd95`.

## Independent post-state check

I separately ran the R4 guard in `after` mode. The saved output is `independent-after-guard.json`, SHA-256 `f57bfa40095b963bb464715a134ebe270841324f10a1d72f3f9af68ec129853c`. It reports `PASS` for 64 published files with the same HEAD, index, fixture, commit, and tree values. This verifies present state; it is not the original restore command's raw output.

No Cargo or Rust tests were run as part of this sync.
