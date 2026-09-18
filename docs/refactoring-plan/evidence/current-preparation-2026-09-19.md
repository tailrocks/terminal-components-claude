# Current preparation evidence — 2026-09-19

This index records the current bounded preparation observations. It is linked
from the [current readiness report](../execution-readiness-report.md) and the
[refactoring-plan README](../README.md). It is not a verifier receipt, task
acceptance, execution authorization, or GO decision.

## Binding identities

The reviewed source payload was a clean worktree at the following identity,
before this bounded documentation-only correction. This file and the other
owned documents change the tree; every source-bound receipt must be
regenerated against the final clean commit/tree. No receipt may attest to
this future documentation commit from inside that same commit.

| Item | Observed value |
|---|---|
| Branch | `refactor/holla-parity` |
| HEAD | `f9801a89cbe0d154f38d184b007fca07d058f39b` |
| HEAD tree | `550d22547254eb9eee2bd11a74d6e77505c5c913` |
| `main` / `origin/main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| Peeled `visual-baseline` tag | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Snapshot tree at the tag | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| Campaign ledger | `armed: false`; ignored; no accepted current task rows |
| Product dispatch | none |

Protected baseline refs, release, grouped store, fixtures, snapshots, and
expected artifacts were not changed. No product refactoring was dispatched.

## Frozen oracle

The tag-derived oracle inventory is `7,550` matrix keys and `30,200` artifacts:
four artifacts per key (`ANSI`, plain text, `PNG`, `HTML`), five terminal sizes
(`72x20`, `80x24`, `100x30`, `120x40`, `160x50`), and five color modes
(`truecolor`, `256`, `16`, `none`, `nocolor`). The tag-derived suite,
configuration, grouped store, and fixtures remain read-only inputs.

The latest calibration membership evidence reports `30,200` expected and
`30,200` actual artifacts, with zero missing and zero extra. Membership is not
acceptance: one selected test failed and two were skipped.

## Qualified taskfmt

| Item | Observed value |
|---|---|
| Source | `/Users/donbeave/Projects/taskfmt/task-format` |
| Revision | `afd3b575dbcc7044620bec4b9493a74eca3e5ef2` |
| Version | `0.2.0` |
| Executable | `/tmp/taskfmt-latest-install/bin/taskfmt` |
| SHA-256 | `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de` |

The catalog contains `73` numbered packages, `506` checks, `276` dependency
edges, and maximum DAG depth `35`. Latest standalone taskfmt lint evidence is
`73/73`; lint does not accept implementation or verifier evidence.

## Native proof status

Proof preparation changes landed in these commits:

1. `8c9e050c` — native worker and observer ABI binding.
2. `f4ce758e` — native target paths, build receipt, and dispatcher binding.
3. `4737da3c` — native proof ABI repair and bundled proof regeneration.
4. `7639e7ae` — context-index ABI acceptance repair.
5. `19f6d2eb` — context-index ABI alignment repair.
6. `12ac27ff` — observer-provider descriptor closure repair; proof parent tree
   `8d66c95408d7e9e404eaf9019857b33659906e84`. The current proof payload,
   carried by docs-only HEAD `f9801a89`, has a positive native-launch check
   that is **UNVERIFIED/PENDING**; no independent verifier has run or
   accepted it.

Fresh verifier evidence is at:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-4737da3c`

Its explicit `VERDICT.md` result for historical candidate `4737da3c` is
**REJECTED**, not an acceptance for current HEAD:

- `cargo fmt --all -- --check` failed.
- The positive bundled-worker execution emitted a
  `tc-proof-runner-result/v1` with `status=rejected`, category
  `CONTEXT_INDEX`, and no observer event.
- The original positive-launch recorder was interrupted before its command,
  output, and exit tuple was emitted.
- Identity, external native build/receipt, locked offline proof nextest
  (`26 passed, 0 failed, 0 skipped`), rebundle/static checks, `73/73` lints,
  native prepare, and pre-execution validate passed as focused evidence.
- Direct-child bypass, full adversarial matrix, dispatcher lint/verify, ledger
  tests, comparator replay, and final protected-ref recheck were not executed.

No independent verifier or reviewer has returned an acceptance for the current
`12ac27ff` proof payload or docs-only `f9801a89` tree. Therefore there is no
current accepted verifier/reviewer receipt and the proof path is not
dispatch-ready.

## Calibration

Raw evidence root:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-frozen-connections`

Inputs include source base commit `89218626011f2f82c4e87c4dfd5868a4c5f3e284`,
frozen `pointer.rs`, corrected frozen `connections.rs` blob
`d4be31bc3cda2ffa205de62a0c43c59ff983ed81`, its SHA-256
`f5095c284c051ad175d44216ed5e8a7be40af354ee1a59a4af85e6275a677ab2`, and
external nextest config SHA-256
`bdbe0a8958a696e4b5108ae190a0c07089f2d7aea91c64a20d52e3346e7092da`.

Targeted calibration passed `3/3`; each targeted root produced `100/100`
matching artifacts. The full selected run produced:

| Result | Count |
|---|---:|
| Selected | 302 |
| Passed | 301 |
| Failed | 1 |
| Skipped | 2 |
| Exit | 100 |

The failure is
`tablepro_connections_form_advanced_120x40_truecolor`. Its root had `24/100`
matching and `76/100` mismatching artifacts: 23 ANSI, 15 plain-text, 15 PNG,
and 23 HTML artifacts. The raw mismatch list is in
`calibration-frozen-connections/full-run-1/mismatch-paths.tsv`.

The mismatch is structural, not timing. Historical source comparison records
the split: commit
`3570a2ed23444dddf1eddcdcc49b654b169038fe` supplies the responsive
form-ownership overlay, while source commit
`89218626011f2f82c4e87c4dfd5868a4c5f3e284` supplies the later responsive
rendering source. Frozen expected output requires list+form for
`connections/form_new` and full-pane form ownership for
`connections/form_advanced`. No source, snapshot, oracle, or expected artifact
was mutated to reconcile the split.

The divergent output was rejected by the frozen oracle with `cells-differ`.
That is useful negative evidence, not calibration acceptance. The complete
final parity gate remains unproven.

## Current command outcomes

These are preparation observations only:

| Check | Result |
|---|---|
| `scripts/campaign-preflight.sh` | Exit `1`; explicit report **NO-GO** gate / no current accepted verifier receipt. No arming or dispatch. |
| `python3 docs/refactoring-plan/evidence/validate-plan.py --summary` | Exit `0`; `passed: true`, `error_count: 0`, `73` task packages, `211` bootstrap assets, maximum depth `35`. |
| Qualified taskfmt lint | `73/73` packages passed. |
| Historical candidate verifier (`4737da3c`) | **REJECTED**; see the native proof section. |
| Current calibration | Targeted `3/3` passed; full selected `301/302` passed, `1` failed, `2` skipped. |
| Ledger | `armed: false`; no accepted task rows. |

The current preflight failure is stale-ledger state, not permission to repair
the ledger by fabrication. Preparation evidence must be sealed after the final
documentation tree is committed, then independently verified and reviewed.

## Required next checks

From the final clean campaign worktree, with external verifier-owned run and
target paths, rerun:

```sh
scripts/campaign-preflight.sh
python3 docs/refactoring-plan/evidence/validate-plan.py --summary
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" --log-dir "$RUN_DIR/taskfmt-logs"
scripts/campaign-build-proof.sh
cargo fmt --all -- --check
cargo nextest run --locked --workspace --no-fail-fast
```

Then rerun the complete native positive/adversarial proof matrix, obtain an
independent verifier and reviewer decision, and rerun the tag-derived visual
calibration/final matrix. Do not use taskfmt lifecycle commands, `cargo test`,
baseline acceptance/blessing, candidate-generated expected output, or a
reduced matrix.

## Remaining obligations and blockers

Preparation blockers:

- native proof verifier is explicitly rejected and lacks independent reviewer
  acceptance;
- proof positive/adversarial evidence is incomplete;
- full calibration has one structural failure and two skips;
- the ledger is disarmed with no current accepted receipts;
- the frozen suite/store is external read-only input, not active branch gate;
- source-bound receipts must be regenerated after this documentation commit;
- full native, behavioral, PTY, performance, API, static, and documentation
  gates remain to be executed and reviewed.

Remaining product obligations are not preparation acceptance: complete the 73
task DAG, migrate all consumers, remove duplicate renderers and compatibility
painters, preserve component ownership and APIs, prove application and PTY
behavior, satisfy performance budgets, and compare all `7,550` keys and
`30,200` artifacts exactly.

Verdict: **NO-GO**. The future implementation goal remains **NOT AUTHORIZED
FOR EXECUTION**. No campaign task was dispatched, no ledger arm occurred, no
protected baseline input changed, and no push or merge was performed.
