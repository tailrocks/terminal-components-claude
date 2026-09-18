# Current preparation evidence — 2026-09-18

This index records bounded preparation observations. It is linked from the
[current readiness report](../execution-readiness-report.md) and the
[refactoring-plan README](../README.md). It is not a verifier receipt, task
acceptance, execution authorization, or a GO decision.

## Binding identities

The source tree under review was observed before this documentation-only
reconciliation commit:

| Item | Observed value |
| --- | --- |
| Branch | `refactor/holla-parity` |
| HEAD | `f5013f609aed1ba32ce60352b38fd0b1b11b063c` |
| HEAD tree | `e08a965702a4674e9e61970fb1c3923e05e1dec7` |
| Peeled local `visual-baseline` tag | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Peeled remote `visual-baseline` ref | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Campaign ledger | `armed: false`; ignored working-tree file; not changed |
| Ledger `integration_head` | `a34a1cffc17b2572fc6b42d31e433166d04658ff` (not current HEAD) |
| Ledger candidate tree | `4d3501a6abc32ebd5999ddcdad55acfa6434bd17` (not current HEAD) |

The worktree was not clean during evidence collection. Unscoped changes were
preserved in `baseline/before/MANIFEST.md`,
`tools/refactor-proof/Cargo.toml`, and `tools/refactor-proof/src/lib.rs`.
This reconciliation did not stage or modify them. No clean-candidate or
proof-tree receipt is claimed.

## Frozen-oracle inventory

The peeled tag contains:

- `7,550` matrix keys represented by `30,200` snapshot artifacts;
- exactly four artifacts per key: `.ansi`, `.txt`, `.png`, and `.html`;
- five terminal sizes: `72x20`, `80x24`, `100x30`, `120x40`, `160x50`;
- five color modes: truecolor, `256`, `16`, `none`, and `nocolor`;
- eight files under `tests/visual_baseline/`; and
- the tag `.config/nextest.toml`.

The current HEAD contains none of `snapshots/`, `tests/visual_baseline/`,
`.config/nextest.toml`, or `docs/baseline/`. The tag-derived corpus is
read-only input. No baseline, tag, snapshot, or expected artifact was changed.

The preparation catalog contains `73` numbered task packages and `211` frozen
bootstrap asset bindings. The plan audit reports a maximum dependency depth of
`35`.

## Tool identity

The qualified standalone taskfmt is:

| Item | Observed value |
| --- | --- |
| Source | `/Users/donbeave/Projects/taskfmt/task-format` |
| Revision | `afd3b575dbcc7044620bec4b9493a74eca3e5ef2` |
| Version | `0.2.0` |
| Executable | `/tmp/taskfmt-latest-install/bin/taskfmt` |
| SHA-256 | `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de` |

The source checkout was clean and at the recorded revision when inspected.

## Command outcomes

All outcomes below are preparation observations only.

| Check | Command/result |
| --- | --- |
| Preflight | `scripts/campaign-preflight.sh` exited `1`. Tag, branch, and worktree checks passed; ledger validation then failed with `no current accepted verifier receipt exists`. It did not authorize arming or dispatch. |
| Plan audit | `python3 docs/refactoring-plan/evidence/validate-plan.py --summary` exited `0`; `passed: true`, `error_count: 0`, `task-index.tsv: 73`, `bootstrap-assets.tsv: 211`, maximum dependency depth `35`. |
| Taskfmt lint | The qualified binary linted all numbered packages: `73/73 passed`. |
| Focused nextest, current dirty tree | `NEXTEST_USER_CONFIG_FILE=none cargo nextest run --locked -p refactor-proof` exited `102` before tests because Cargo would need to update `Cargo.lock` for the unscoped dirty `toml = 1.1.5` dependency change. This is not a passing gate. |
| Earlier focused diagnostic | Before those dirty proof changes appeared, the same focused command reported `3 passed (2 binaries, 0.021s)`. It has no clean-tree receipt and is retained only as a superseded diagnostic, not accepted evidence. |

## Baseline calibration status

The calibration has only a control result so far: **control passed**. The full
`7,550`-key / `30,200`-artifact replay is **pending**. The tag's checked-in
nextest configuration contains a `binary(visual_baseline)` profile override;
the calibration attempt hit a parser rejection for that binary override. The
next run therefore requires a corrected **external** nextest configuration.
The tag configuration must remain read-only. No full-run result, comparison
receipt, blessing, or visual acceptance is claimed.

## Remaining blockers

- The ledger remains disarmed and has no current accepted verifier receipt;
  its recorded integration head/tree do not bind the current HEAD/tree.
- The frozen suite/config/grouped store is absent from HEAD and still needs a
  read-only tag-derived import and complete replay.
- The full baseline calibration is pending and needs the external corrected
  nextest configuration described above.
- No trusted native per-check context/index/result/observer materializer has
  produced accepted current subagent evidence.
- The worktree contains preserved out-of-scope changes, so clean-tree and
  locked nextest gates are not currently proven.
- Consumer migration, ownership cleanup, behavioral parity, performance
  parity, and independent verifier/reviewer evidence remain incomplete.

The readiness verdict stays **NO-GO** and the ledger stays `armed: false`.
