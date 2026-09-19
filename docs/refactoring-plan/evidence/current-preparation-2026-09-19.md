# Current preparation evidence — 2026-09-19

This is a bounded preparation index, not a verifier receipt, task acceptance,
execution authorization, or GO decision. It records two distinct layers. Raw
evidence remains under the external run roots named below.

## 1. Two-layer sealing identities

### Layer 1 — historical/tested preparation payload

The following identities belong only to the verified preparation payload. They
are historical/tested evidence and are **never** the current final-tree
binding:

| Item | Exact value |
|---|---|
| Branch observed during payload test | `refactor/holla-parity` |
| Tested payload commit | `14aa8ed0469219ff8f6570be7824e5ade39240cc` |
| Tested payload tree | `f3ef0f6badd01161bc24cdfef5161db55ba5d579` |
| Tested payload parent | `96c6c475b5d22193b7539565fa5b4612a88ea007` |
| Verifier setting | `gpt-5.6-luna/max` |
| Historical/tested run | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-14aa8ed-lagrange` |
| Historical/tested verdict | `.../verifier-14aa8ed-lagrange/VERDICT.md`: **REJECTED** |

### Layer 2 — evidence-only docs package and external final seal

The evidence-only docs package parent is commit
`3b79d3403d52ededca087d62dccb9ad4474c105b` with tree
`d6b85e1feb8e89f25cae6db360b92f071a4c4f43`. The final sealing run is
predeclared at:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`

The final verifier’s manifest from that external root is the sole authority
for actual final HEAD/tree, task contracts, tool/oracle identities, and
receipts. This docs package does not self-attest final tree identity. Any
subsequent relevant edit invalidates that run and requires fresh external
sealing. No final sealing receipt is claimed here.

The ledger remains `.campaign/ledger.json`, schema `campaign-ledger/v1`, with
4 rows and `armed=false`; dispatch and accepted receipt remain none.

## 2. Protected oracle identity

| Item | Exact value |
|---|---|
| Tag ref object | `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` |
| Peeled tag commit | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Baseline tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| Snapshot tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| Inventory | 7,550 keys; 30,200 artifacts; 7,550 each ANSI/plain/PNG/HTML; no other files |
| Matrix dimensions | `72x20`, `80x24`, `100x30`, `120x40`, `160x50`; `truecolor`, `256`, `16`, `none`, `nocolor` |

The tag, branch, release, grouped store, fixtures, snapshots, and expected
artifacts were observed unchanged. They are read-only policy inputs. No
candidate output was blessed or substituted.

## 3. Layer-1 proof and taskfmt identities — historical/tested only

| Item | Exact value |
|---|---|
| Native build schema | `tc-proof-native-build/v1` |
| Native build command | `cargo build --locked --offline -p refactor-proof --bin tc-proof` |
| Historical/tested native proof binary | `.../verifier-14aa8ed-lagrange/target/debug/tc-proof` |
| Native proof binary SHA-256 | `a3b7712ab7c3ea22940ffe915a2328d25767d35e5253e9df82776db0d3b70fcc` |
| Native build receipt | `.../target/debug/tc-proof.build.json` |
| Taskfmt source | `/Users/donbeave/Projects/taskfmt/task-format` |
| Taskfmt revision | `afd3b575dbcc7044620bec4b9493a74eca3e5ef2` |
| Taskfmt version | `0.2.0` |
| Taskfmt binary | `/tmp/taskfmt-latest-install/bin/taskfmt` |
| Taskfmt SHA-256 | `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de` |

The native proof receipt binds the proof binary to the historical/tested
Layer-1 payload commit/tree. The external comparator binding passes with
`TC_PROOF_NATIVE_BINARY` forwarded; its comparator hash and result are
historical/tested payload evidence, never a binding for the final docs tree
and never campaign acceptance.

## 4. Layer-1 verifier result — historical/tested payload

The Layer-1 verifier’s exact conclusion is **REJECTED**. It is not a current
final-tree receipt or authorization. The following qualification subchecks
passed for the historical/tested payload:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| Native proof build and receipt | exit 0; candidate commit/tree bound |
| Locked offline `refactor-proof` nextest | 26 passed, 0 failed, 0 skipped |
| Python rebundle/source/AST/compile checks | pass; AST sweep covered 46 files |
| Native proof path contract | exit 0 |
| Seven bootstrap asset freeze checks | all exit 0; changed 0 |
| Plan validator | exit 0; 73 task entries; 211 bootstrap assets |
| DAG derivation | exit 0; 73 tasks; maximum depth 35; no serialization pairs |
| Standalone taskfmt lint | 73/73 passed, 0 failed |
| External comparator qualification | 72/72 cases; 141 invocations; 0 failures |
| Comparator negative controls | 7/7 rejected: relative, missing, symlink, unbound, ambiguous, candidate-worktree, shared-cache-default |
| Native observer-backed launch/validate | exit 0 |
| Runner/host/broker bootstrap self-tests | canonical checks exit 0 |

The decisive failures are:

- Dispatcher lint for TASK-001 exits `0`, native dispatch validation exits
  `0`, but dispatcher verify exits `1` with `RESULT FAIL` (`pass=3`,
  `fail=6`). Scope/forbidden-path checks reject preparation/documentation
  changes outside TASK-001, and the intentional NO-GO checks retain their
  reserved failure status.
- `scripts/campaign-preflight.sh` exits `1` and fails closed at the explicit
  readiness-report **NO-GO** gate. It does not arm or dispatch the ledger.

Raw decisive files are `dispatch-verify.{stdout,stderr,exit}` and
`preflight.{stdout,stderr,exit}` under the historical/tested Layer-1 run
root. No independent reviewer returned `VERIFIED`; no acceptance receipt was
issued.

### Auxiliary unqualified diagnostic

The copied-runner style-timing self-test was run as an auxiliary diagnostic:

```text
python3 .../style-timing-bootstrap/runner-bootstrap-driver.py --self-test
```

It exited `1` with `FileNotFoundError` for the absent sibling
`runner-bootstrap-index.py`. Raw files are
`style-runner-bootstrap-072-selftest.{stdout,stderr,exit}`. This is recorded as
**unqualified**, not hidden and not counted as a pass. The canonical runner
self-tests passed; this copied-runner diagnostic is not campaign acceptance.

## 5. Calibration and mixed-oracle finding

No product full matrix was run by the Layer-1 verifier. Existing independent
calibration evidence is cited, not reused as a fresh pass:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-frozen-connections/full-run-1/`

| Result | Count |
|---|---:|
| Selected | 302 |
| Passed | 301 |
| Failed | 1 |
| Skipped | 2 |
| Exit | 100 |

The sole failed case is
`tablepro_connections_form_advanced_120x40_truecolor`: 24/100 artifacts
match and 76/100 mismatch, broken down as 23 ANSI, 15 plain-text, 15 PNG, and
23 HTML artifacts. No snapshot was blessed, altered, or substituted.

Independent history establishes a mixed frozen-oracle state: commit
`3570a2ed23444dddf1eddcdcc49b654b169038fe` supplies the responsive
form-ownership overlay, while source commit
`89218626011f2f82c4e87c4dfd5868a4c5f3e284` supplies the later responsive
rendering source. Frozen expected output requires list+form for
`connections/form_new` and full-pane form ownership for
`connections/form_advanced`. This explains the historical split; it does not
permit changing the oracle or accepting the mismatch.

## 6. Catalog and remaining product obligations

The current generated catalog contains 73 task packages, 506 checks, 276
dependency edges, and maximum DAG depth 35. All task manifests remain
unaccepted. `TASK-001` and `TASK-070` describe retired/non-qualifying
host-lifecycle paths; replacement proof work still needs accepted subagent
evidence. `TASK-071`/`072` remain blocked by accepted proof and dependency
receipts. Tasks `002–069` and `073` remain pending or blocked by their DAG
dependencies and missing acceptance evidence.

Remaining product obligations include consumer migration, removal of duplicate
renderers and compatibility painters, component ownership/state/hit-testing
correctness, all application flows, focus/hover/scroll/selection/layout,
resize and lifecycle behavior, real PTY setup/cleanup/input/settled-frame
transitions, strict performance budgets, public API/static/documentation
checks, native platform checks, and complete oracle parity.

## 7. Required final sealing evidence

The final verifier must use the predeclared external run root
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`.
It must bind the actual clean HEAD/tree at run time, plus task contracts,
tool/oracle identities, contexts, results, and receipts. Do not copy the
Layer-1 payload identity into the final manifest.

From that clean native macOS worktree, with verifier-owned external target and
run directories:

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

Then obtain accepted independent verifier and reviewer evidence for the native
positive/adversarial proof matrix, resolve the calibration failure and skips
without changing the oracle, and run the complete 7,550-key/30,200-artifact
PTY visual gate. Any stale, missing, rejected, unqualified, unknown, or
source-mismatched result invalidates the final sealing run and keeps the
verdict **NO-GO**. Any subsequent relevant edit invalidates the run.
