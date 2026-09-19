# SUPERSEDED / HISTORICAL

> This file is retained unchanged as historical provenance only. Its old
> commit, tree, and evidence claims cannot authorize current execution, task
> acceptance, ledger arming, or a GO decision. Use the current preparation
> index: [`current-preparation-2026-09-19.md`](current-preparation-2026-09-19.md).

# Current preparation evidence — 2026-09-18

This index records bounded preparation observations. It is linked from the
[current readiness report](../execution-readiness-report.md) and the
[refactoring-plan README](../README.md). It is not a verifier receipt, task
acceptance, execution authorization, or a GO decision.

## Binding identities

The preparation source tree was observed from a clean worktree before this
documentation-only reconciliation commit. This evidence is not automatically
valid for the post-commit tree; source-bound receipts must be rebound.

| Item | Observed value |
| --- | --- |
| Branch | `refactor/holla-parity` |
| HEAD | `6ec1c83b9123c5fc531468449ef259e2927b6604` |
| HEAD tree | `698cf69e073ca85826617cc641d7149a376e0996` |
| Peeled local `visual-baseline` tag | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Peeled remote `visual-baseline` ref | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Campaign ledger | `armed: false`; ignored working-tree file; no accepted task rows; not changed |

The worktree was clean during evidence collection. This reconciliation is
scoped to canonical documentation only; no product, proof, task, ledger,
baseline, tag, snapshot, or expected artifact was changed. The final committed
worktree must remain clean.

## Frozen-oracle inventory

The peeled tag contains:

- `7,550` matrix keys represented by `30,200` snapshot artifacts;
- exactly four artifacts per key: `.ansi`, `.txt`, `.png`, and `.html`;
- five terminal sizes: `72x20`, `80x24`, `100x30`, `120x40`, `160x50`;
- five color modes: truecolor, `256`, `16`, `none`, and `nocolor`;
- eight files under `tests/visual_baseline/`; and
- the tag `.config/nextest.toml`.

The preparation HEAD contains none of `snapshots/`, `tests/visual_baseline/`,
`.config/nextest.toml`, or `docs/baseline/`. The tag-derived corpus is
read-only input. The baseline refs and grouped store remained unchanged; no
baseline, tag, snapshot, or expected artifact was changed.

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
| Readiness preflight | Exited `1` at the readiness-report **NO-GO** gate. The ledger remains `armed: false` with no accepted task rows; no arming or dispatch was authorized. |
| Proof preparation | Exited `1` because the external run directory/receipt was absent. |
| Native Darwin verifier | **REJECTED** for verifier commit `6ec1c83b9123c5fc531468449ef259e2927b6604`: `cargo nextest` ran 11 tests, 11 passed with 1 leaky test; positive prepare/launch passed, but `validate` was absent and observer binding/socket/result side-effect checks failed. No accepted verifier receipt exists. |
| Independent reviewer | **REJECTED**: compare ABI mismatch, dispatcher bypass, no observer supervisor, and weak result/nonce/oracle/report binding. |
| Baseline replay | Using the external corrected config SHA-256 `bdbe0a8958a696e4b5108ae190a0c07089f2d7aea91c64a20d52e3346e7092da` and the logical target path: `302` tests, `300` passed, `2` failed, `2` skipped. Holla timing passed isolated rerun; TablePro `form_advanced` failed isolated rerun with `23/25` cells. |

## Baseline calibration status

The full available baseline replay used the read-only tag-derived suite with
the external corrected nextest configuration above and the logical target path.
It ran `302` tests: `300` passed, `2` failed, and `2` skipped. The Holla timing
failure passed an isolated rerun. TablePro `connections/form_advanced` still
failed its isolated rerun: `23/25` matrix cells differed. This is failed
evidence, not calibration acceptance; the complete final parity gate remains
unproven. Never bless or modify snapshots. The tag config, baseline refs, and
grouped store remain read-only and unchanged.

Raw baseline logs/config and copied native-verifier logs are retained under
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-18/`.
That external directory is evidence only, not an accepted campaign receipt.

Durable raw evidence index:

| File | SHA-256 |
| --- | --- |
| `baseline/campaign-baseline-nextest.toml` | `bdbe0a8958a696e4b5108ae190a0c07089f2d7aea91c64a20d52e3346e7092da` |
| `baseline/campaign-baseline-full.log` | `906ff8f73b73998d89f8332ffed02bc2b0de3e6cd9a96189a163a02c9ff8c0ca` |
| `baseline/campaign-baseline-rerun-holla.log` | `efc13afd0293c6eb6f4b1b337485a295ea4eafb90f5879746eb266d90c8c3db6` |
| `baseline/campaign-baseline-rerun-tablepro.log` | `0e61be040af647e857c33c3340561d23fbfd02c568b1cc241f97446013f024a8` |
| `native-verifier/logs/final-verifier-summary.log` | `56b5fd6f220af49c399621792824cc41201cc3a63e8c0ed4b3d75e2d459e4fad` |
| `native-verifier/logs/cargo-nextest-refactor-proof.log` | `96ec3d26cac06a07dd410ea6a0b13fe5d0479e1d03d4b620ec7d4c8f6c9c7854` |

## Remaining blockers

- The ledger remains disarmed and has no current accepted verifier receipt;
  it has no accepted task rows.
- The frozen suite/config/grouped store is absent from HEAD and still needs a
  read-only tag-derived import and complete replay.
- The baseline replay failed two cases; Holla needs root-cause disposition and
  rerun evidence, while TablePro `form_advanced` still fails `23/25` cells.
- No trusted native per-check context/index/result/observer materializer has
  produced accepted current subagent evidence; proof preparation lacks its
  external run directory/receipt.
- The native Darwin verifier and independent reviewer both returned
  **REJECTED** for the failures recorded above.
- Consumer migration, ownership cleanup, behavioral parity, performance
  parity, and independent verifier/reviewer evidence remain incomplete.

The readiness verdict stays **NO-GO** and the ledger stays `armed: false`.
