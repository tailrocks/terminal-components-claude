# Terminal Components Refactor — Execution-Readiness Report

**Current authority:** This is the sole current readiness report for
2026-09-19. It is preparation documentation, not execution authorization.

## A. Verdict

**NO-GO.**

The branch is campaign/proof scaffolding. No production refactoring task has
been dispatched, no task is accepted, and product parity is unproven.

### Two-layer sealing protocol

Layer 1 is the verified preparation payload, retained as historical/tested
evidence only and never as the current final-tree binding:

- commit `14aa8ed0469219ff8f6570be7824e5ade39240cc`;
- tree `f3ef0f6badd01161bc24cdfef5161db55ba5d579`;
- parent `96c6c475b5d22193b7539565fa5b4612a88ea007`;
- run root `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-14aa8ed-lagrange`;
- verdict `VERDICT.md`: **REJECTED**.

Its proof qualification subchecks pass, but dispatcher verification and
readiness preflight fail closed, and calibration is incomplete. These are
historical/tested payload facts, not current final-tree evidence.

Layer 2 is this evidence-only docs package. Its package parent is commit
`3b79d3403d52ededca087d62dccb9ad4474c105b` with tree
`d6b85e1feb8e89f25cae6db360b92f071a4c4f43`. Final sealing is external at the
predeclared run root
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`.
Only the final verifier’s manifest from that root may bind actual HEAD/tree,
task contracts, qualified tools, oracle identities, and receipts. This report
does not self-attest final tree identity. Any subsequent relevant edit
invalidates the run and requires fresh external sealing.

Therefore no accepted verifier/reviewer receipt, ledger arm, dispatch, push,
merge, or GO decision exists.

## B. Fixed identities and trust roots

| Input | Bound identity |
|---|---|
| Layer-1 tested payload | commit `14aa8ed0469219ff8f6570be7824e5ade39240cc`; tree `f3ef0f6badd01161bc24cdfef5161db55ba5d579`; parent `96c6c475b5d22193b7539565fa5b4612a88ea007`; historical/tested only |
| Layer-2 docs package parent | commit `3b79d3403d52ededca087d62dccb9ad4474c105b`; tree `d6b85e1feb8e89f25cae6db360b92f071a4c4f43`; evidence-only |
| Final seal authority | external verifier manifest under `.../verifier-final-sealed`; actual HEAD/tree determined there |
| Protected oracle tag object | `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` |
| Peeled `visual-baseline` commit | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| Baseline tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| Snapshot tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| Oracle inventory | 7,550 keys; 30,200 artifacts; 7,550 each ANSI/plain/PNG/HTML; no other files |
| Taskfmt source | `/Users/donbeave/Projects/taskfmt/task-format` |
| Taskfmt | revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`; version `0.2.0`; binary `/tmp/taskfmt-latest-install/bin/taskfmt`; SHA-256 `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de` |
| Native proof | schema `tc-proof-native-build/v1`; verifier binary SHA-256 `a3b7712ab7c3ea22940ffe915a2328d25767d35e5253e9df82776db0d3b70fcc`; receipt binds the candidate commit/tree |
| Ledger | `.campaign/ledger.json`, `campaign-ledger/v1`, 4 rows, `armed=false` |

The oracle tag, branch, release, grouped store, fixtures, snapshots, and
expected artifacts are read-only policy inputs. The observed tag-derived
inventory is unchanged. Provider-enforced immutability is not assumed;
repository policy is the guard.

## C. Layer-1 evidence — historical/tested payload

The verifier setting for the Layer-1 test was inherited `gpt-5.6-luna/max`.
The following qualification evidence passes for that historical/tested
payload:

- locked offline `cargo nextest run --locked --offline -p refactor-proof
  --no-fail-fast`: 26 passed, 0 failed, 0 skipped;
- native proof build and source/tree receipt binding: pass;
- Python rebundle/source/AST/compile checks: pass, with 46 Python files swept;
- native proof path contract: pass;
- seven frozen bootstrap asset freeze checks: all pass, changed 0;
- plan validator: pass, 73 task entries and 211 bootstrap assets;
- DAG derivation: pass, 73 tasks, 276 dependency edges, maximum depth 35,
  no serialization pairs;
- standalone taskfmt lint: 73/73 passed;
- external comparator: 72/72 cases, 141 invocations, 0 failures;
- all 7 comparator external-binding negative controls rejected;
- native observer-backed launch/validate regression: pass;
- canonical runner, host, and broker bootstrap self-tests: pass.

These are qualification subchecks, not acceptance. The decisive result remains
**REJECTED**:

1. Dispatcher TASK-001 lint exits `0` and native dispatch validation exits `0`,
   but dispatcher verify exits `1` with `RESULT FAIL` (`pass=3`, `fail=6`).
   Scope/forbidden-path checks reject preparation/documentation changes outside
   TASK-001, and the intentional NO-GO checks retain their reserved failure
   status.
2. `scripts/campaign-preflight.sh` exits `1` at the explicit report NO-GO gate.
   It fails closed and does not arm or dispatch the campaign.
3. The existing independent full calibration remains 301/302 passed, with one
   failure and two skips.

The auxiliary copied-runner style-timing self-test exits `1` with
`FileNotFoundError` for sibling `runner-bootstrap-index.py`. It is explicitly
**unqualified** and remains visible in raw evidence; it is not hidden and is
not counted as a pass. The canonical runner checks passed.

No independent reviewer returned `VERIFIED`. No receipt was issued. No
product full matrix was run by the Layer-1 verifier. These results do not bind
the Layer-2 docs package or the final sealing tree.

## D. Calibration blocker and history

Raw calibration root:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-frozen-connections/full-run-1/`

| Selected | Passed | Failed | Skipped | Exit |
|---:|---:|---:|---:|---:|
| 302 | 301 | 1 | 2 | 100 |

The sole failure is
`tablepro_connections_form_advanced_120x40_truecolor`. It has 24/100
matching artifacts and 76/100 mismatches: 23 ANSI, 15 plain-text, 15 PNG, and
23 HTML. Independent history establishes a mixed frozen-oracle state:

- `3570a2ed23444dddf1eddcdcc49b654b169038fe` supplies responsive
  form-ownership overlay behavior;
- `89218626011f2f82c4e87c4dfd5868a4c5f3e284` supplies later responsive
  rendering source;
- frozen expected output requires list+form for `connections/form_new` and
  full-pane form ownership for `connections/form_advanced`.

This is unresolved negative evidence. It is not permission to modify source,
fixtures, snapshots, expected artifacts, thresholds, timing, or the oracle.

## E. Task catalog and dependency DAG

The current catalog has 73 task packages, 506 checks, 276 dependency edges,
and maximum dependency depth 35. All task manifests remain unaccepted.
`TASK-001` and `TASK-070` are retired/non-qualifying host-lifecycle paths;
their replacement proof path still lacks accepted verifier/reviewer evidence.
`TASK-071` and `TASK-072` remain blocked. Tasks `002–069` and `073` remain
pending or blocked by dependencies and missing accepted evidence.

Exact generated dependency layers are:

~~~text
L01  001
L02  070
L03  071
L04  072
L05  002 003 004 005 007
L06  006
L07  008
L08  073
L09  009 011 012
L10  010 013 029
L11  014
L12  015 016 017 021
L13  018 022 026
L14  019 020
L15  023
L16  024
L17  025 027
L18  028
L19  030
L20  031
L21  032 040 051 058
L22  033 041 052 059
L23  034 042 053 060
L24  035 043 054 061
L25  036 044 055 062
L26  037 045 056 063
L27  038 046 057 064
L28  039 047
L29  048
L30  049
L31  050
L32  065 067
L33  066
L34  068
L35  069
~~~

Only independent tasks in disjoint worktrees may run concurrently. A single
coordinator integrates accepted commits serially with compare-and-swap parent
checks. The generated graph is structural data only; it carries no status or
authorization.

## F. Remaining product obligations

Preparation evidence does not establish product completion. The campaign must
still:

- migrate every consumer to the intended reusable component ownership and
  remove compatibility painters, duplicate state, hidden renderers, and
  app-local repaint overlays;
- preserve public APIs, component boundaries, canonical cells/graphemes,
  continuation cells, styles, cursor, dimensions, focus, hover, hit testing,
  scrolling, selection, layout, resize, editing, cancellation, stale-result
  handling, and lifecycle semantics;
- prove every Showcase, Holla, Jackin, and TablePro route, modal, menu,
  dialog, editor, overlay, drawer, reconnect, stale-result, and completion
  path through real application code;
- prove PTY startup/setup/cleanup, alternate screen, input delivery, resize,
  color modes, cursor/focus state, and settled-frame transitions;
- meet strict existing component/application performance budgets and record
  the required native macOS results;
- pass locked workspace `cargo nextest`, formatting/build/doc/API/static and
  documentation link checks, platform checks, ownership scans, and generated
  artifact cleanliness; and
- compare every frozen key 1:1 across all 30,200 ANSI, plain-text, PNG, and
  HTML artifacts, five sizes, five color modes, all routes/states, and all
  captured interaction checkpoints.

## G. Required execution controls after a future GO

The following controls remain mandatory even after the report can become GO:

- native macOS only; no Docker, Podman, containers, images, mounts,
  firmlinks, or old `/task`, `/work`, `/proof`, or `/run` namespaces;
- coordinator, implementer, verifier, and independent reviewer as isolated
  host-local subagents using `gpt-5.6-luna/max`;
- no shared writable worktree, build directory, run directory, snapshot store,
  or expected artifact store;
- verifier-owned native proof build, exact immutable per-check contexts and
  context index, observer capability, nonce/request binding, result paths,
  and post-run validation before taskfmt;
- only standalone taskfmt `lint` and `verify`; no lifecycle, dispatch,
  runtime, host, monitor, or promotion command;
- independent reviewer status `VERIFIED` bound to the exact commit, tree,
  task, scope base, run directory, tool identities, and raw outputs before
  integration; and
- serial compare-and-swap integration, clean ancestry, and re-run of affected
  frozen parity before proceeding.

Required native preparation commands are the qualified commands recorded in
the current contracts:

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

## H. Hard blockers and disposition

The readiness report stays NO-GO until all of these are closed with fresh,
independent, source-bound evidence:

- Layer-1 historical/tested verdict **REJECTED** and no reviewer acceptance;
- no external final-sealing manifest yet binds the actual Layer-2/final
  HEAD/tree;
- dispatcher TASK-001 verify failure and preflight NO-GO exit;
- auxiliary style-timing diagnostic remains unqualified;
- calibration is 301/302 with one failure and two skips, including the mixed
  frozen-oracle history;
- ledger is disarmed and has no accepted current task rows;
- proof, taskfmt verify, task, behavior, performance, API, static, docs,
  platform, PTY, ancestry, and full visual gates are not accepted;
- frozen oracle is external read-only input and is not yet the active branch
  gate; and
- source-bound evidence must be regenerated and sealed by the external final
  verifier against the exact clean tree used for any future decision; any
  subsequent relevant edit invalidates that run.

No blocker may be hidden by reducing the matrix, blessing candidate output,
changing thresholds, weakening provenance, manufacturing a receipt, or
reclassifying a rejected/unknown result.

## I. Decision

**NO-GO remains binding.** No task worktree, implementer, verifier, reviewer,
ledger arm, push, or merge is authorized by this report. The future goal in
[`next-implementation-goal.md`](next-implementation-goal.md) is a complete
conditional plan, explicitly **NOT AUTHORIZED FOR EXECUTION**.
