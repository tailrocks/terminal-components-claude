# Independent project runner qualification, version 1

This package is planning-owned test preparation. It is not the implementation of `tc-proof`, an application baseline, or authority to bless one. It closes the independent fixture gap identified in BR-06 of [the bootstrap review](../bootstrap-review.md). The comparator and host suites remain separate prerequisites. The following bounded implementations own this contract:

- TASK-070 implements `preflight`, `required`, `oracle`, `capture`, and `close`.
- TASK-071 implements `account-tests`, including immutable stage accounting.
- TASK-072 implements `architecture` and its independently supplied source/runtime probes.

The planner freezes this protocol, all files listed by the [qualification extensions](runner-bootstrap-extensions-protocol.md), and the separately reviewed `host-bootstrap-observer.py` together outside all three implementation checkouts. Freeze their exact bytes, interpreter and Git identities, source review decision, and self-test evidence. The runner cannot edit the driver, fixtures, observer or acceptance inputs. Do not treat a checkout-local mutable file as the frozen package. The extensions add executable immutable per-check contexts, preparation accounting, assertion-span preservation, successful stage closure, measured performance, and native/resized source mapping qualification; the base cases alone are insufficient.

## Canonical acceptance commands

```sh
python3 /task/trusted/runner-bootstrap/runner-bootstrap-driver.py \
  --runner /work/tools/refactor-proof/bin/tc-proof --group 070
python3 /task/trusted/runner-bootstrap/runner-bootstrap-driver.py \
  --runner /work/tools/refactor-proof/bin/tc-proof --group 071
python3 /task/trusted/runner-bootstrap/runner-bootstrap-driver.py \
  --runner /work/tools/refactor-proof/bin/tc-proof --group 072
```

Omitting `--group` runs the union. Each negative case is followed by a fresh positive recovery for the same operation. Case order is randomized. Paths and run IDs are opaque random values; no submitted input contains the private mutation label or expected error category. The source seed and palette vary between invocations. There is no candidate command today, and no production runner has passed these commands.

The executable preparation check is:

```sh
python3 -B docs/refactoring-plan/evidence/runner-bootstrap-driver.py --self-test
```

It constructs all finite cases, rejects malformed results and a real zero-worker executable, and independently launches the fixture's real production code for the runtime positives and mutants. This tests the driver and fixture instrument, not an implemented runner. Qualification requires Darwin's available `sandbox-exec`, Python, and Git. Unsupported isolation is a preparation failure, never a successful negative. A different host platform needs an independently reviewed observer with equivalent authority separation before dispatch.

## Protected observer and execution authority

The submitted runner is sandboxed: only its empty output directory is writable, the independent observer's private directory is unreadable, process inspection/task ports/signals are denied, and network access is denied. The observer runs outside that sandbox. It owns a private fresh Git repository, executable fixture source selection, test configuration, PTY allocation, stdout/stderr collection, process status, and observations stored in its own memory. It permits no candidate-selected executable, source directory, worker argv, or test filter. The reusable sandbox function is imported from the planner-frozen host observer, not submitted code.

Two inherited pipes provide a capability-limited execution facility. The operator supplies `TC_PROOF_OBSERVER_REQUEST_FD`, `TC_PROOF_OBSERVER_RESPONSE_FD`, and an unpredictable `TC_PROOF_OBSERVER_NONCE`. The submitted runner sends one newline-terminated JSON object, at most 4096 bytes:

```json
{"schema":"tc-proof-runner-observe/v1","nonce":"operator supplied","operation":"capture","source_commit":"exact oracle commit","tree":"exact frozen candidate tree"}
```

All five fields are mandatory; extra fields, reordered operations, incorrect bindings, excess requests, or replay fail qualification. `oracle` requires exactly two requests. `capture`, `account-tests`, `architecture`, and `close` require exactly one. `preflight` and `required` require none. The observer independently binds the request to the fixture's actual source identity, not an identity read back from submitted output.

For execution operations, the observer returns `operation`, `run_id`, `tree`, `source_commit`, integer `exit`, parsed `payload` (or null on incomplete execution), `stdout_sha256`, and base64 `stderr`. The private copy, not the returned copy, is acceptance authority. The result must include the SHA-256 of each canonical observed object in exact launch order. Merely reproducing the object or its digest cannot create a private observation. A submitted program that runs zero workers fails even if it fabricates every expected field.

The fixture worker imports and compiles the selected `app.py`, creates its actual `App`, calls the actual `App.update` and `App.draw`, and records call events for real source methods. `Widget.draw` must paint the control; `Props.enabled` must govern activation. The worker instrumentation is planner-owned, not a candidate log. Its exact test source also exists in the original Git commit as `tests.py`, so historical test source identity names real committed bytes. The PTY lane allocates an actual PTY, sends one `+` byte in raw mode, and executes the same update/draw path. The worker checks `isatty` on input and output. The direct lane executes the same real methods without a PTY. Each capture records all four width/palette members for its selected lane, with complete before/after checkpoints; missing/extra members and missing checkpoints fail. The oracle lane extracts only the pinned original source blob from the independently created Git object database and performs two fresh launches of the direct namespace. The PTY capture cases separately qualify real PTY execution. The synthetic source is deliberately tiny; its frames are semantic JSON, not substitute schema-3 terminal frames. Schema-3 comparison remains the separate comparator suite.

Production uses the same authority split with pinned Rust build/test tools and accepted application adapters. The operator-owned supervisor must construct argv and source paths from accepted inputs, build the real frozen production tree, and collect actual process evidence. It must not expose this synthetic source as an alternate production execution mode or return fixture data for real application identities. Independent source review and rebuild must verify that the qualified command dispatch reaches the production worker path. A finite public suite cannot prove absence of a deliberately fixture-specific implementation by black-box testing alone.

## Context and result ABI

The exact CLI remains the project interface in [proof-contract.md](../proof-contract.md): `tc-proof OPERATION --context PATH`; oracle adds `--namespace synthetic`, and capture adds `--lane direct` or `--lane pty`. Unknown operations and flags fail. The synthetic namespace belongs only to this operator-owned qualification context; normal campaign membership cannot be expanded by passing it as a CLI option.

`Fixture.__init__` in the independent driver defines the normative `tc-proof-runner-context/v1` object. It binds the opaque run, operation, frozen tree, oracle commit/tree and Git bundle/hash, exact interpreter path/hash, accepted/integrated prerequisite records, adapter delta list, lane, finite axes, flat membership, test inventory, typed prerequisite evidence, and execution configuration. The operator additionally supplies `TC_PROOF_CONTEXT_SHA256`, `TC_PROOF_RUN_ID`, `TC_PROOF_SOURCE_TREE`, and `TC_PROOF_ORACLE_COMMIT`. These are protected host entrypoint inputs; never inherit them into arbitrary candidate workers. They allow fail-closed reporting even if the context is damaged. No hash is expected to reveal the original run identity.

The runner writes exactly one regular, single-link JSON file to the operator-assigned `TC_PROOF_RESULT` path. Duplicate JSON keys, symlinks, extra/missing fields, unsupported versions, invalid field types, and wrong bindings reject. The exact result fields are:

```json
{
  "schema":"tc-proof-runner-result/v1",
  "run_id":"operator supplied",
  "operation":"capture",
  "context_sha256":"operator supplied",
  "status":"passed",
  "category":null,
  "observation_digests":[],
  "outputs":{}
}
```

Success requires process exit zero, `status: passed`, and null category. Rejection requires nonzero exit, `status: rejected`, the exact category below, and empty outputs. Observation digests must always equal all actual private observations, including negative executions. A runtime rejection without a real launch fails. Diagnostics and arbitrary stdout are not acceptance authority. Inputs and existing private source files must retain their original hashes.

Successful outputs are exact:

- `preflight`: `{"validated":true}` after validating context integrity, tool bytes, and accepted/integrated prerequisites. Host receipt authenticity and Git ancestry are independently exercised by the host suite; these synthetic booleans represent protected resolution results, not candidate authorization.
- `required`: `{"members":[...]}` containing exactly the ordered Cartesian product of lanes `[direct,pty]`, widths `[8,12]`, and palettes `[blue,yellow]`, formatted `tiny/LANE/WIDTH/PALETTE`. There are exactly eight unique members. Unknown axes, extra members, duplicate members or omissions fail. Source-level axes are independently pinned before expansion.
- `oracle`, `capture`, `architecture`, and `close`: `{"observations":[...]}` containing exact private observed objects. Oracle payloads must be identical across its two fresh executions. Capture must reach update and both production draws, and the actual lane must match the required lane.
- `account-tests`: the exact observations plus `unresolved_tests:["test_future"]`, `closed_contributions:["tiny-shell.route"]`, and `unresolved_scenarios:["tiny-shell"]`. These remain separate identities. A permitted diagnostic failure stays visibly failed inside the observation; it is not relabeled as a passing test or scenario.

## Source and expansion cases for TASK-070

The finite `cases()` table in the driver is the authoritative case manifest. These mutations are independent inputs, not permission to mutate real oracle authority:

- Changed context bytes, duplicate context keys, edited finite axes after pinning: `INTEGRITY`. The unknown `--approve` flag must fail `PROTOCOL`.
- Wrong interpreter hash, unaccepted prerequisite, prerequisite not integrated: `PREFLIGHT`.
- Missing, extra or duplicate flat member; an undocumented axis: `REQUIRED_SET`.
- Wrong oracle SHA, caller-selected source directory, a proposed render-replacement adapter, or stale candidate tree: `SOURCE`.
- Unequal real oracle repeat observations: `REPEAT`.
- Actual update omitted, Widget drawing bypassed, requested PTY executed without a PTY, missing/duplicate capture membership, or an omitted checkpoint: `EXECUTION`.
- Missing, failed, wrong-tree or candidate-authored closure evidence: `CLOSURE`; changed evidence bytes after context pinning: `INTEGRITY`.

Oracle source selection comes only from the pinned Git bundle/commit and independently selected original blob. A writable source directory and an adapter which changes drawing cannot become oracle authority. Valid adapter changes in this tiny fixture are empty; the real accepted observation/time adapter is separately qualified by the source-reversal and timer-boundary evidence in the main proof contract. A later adapter extension requires independent additional fixtures, not candidate-defined allowlisting.

Closure obtains its prerequisite evidence from the observer capability and joins exact operation/run/tree bindings. The synthetic evidence is operator-authored fixture authority, as with the host suite's synthetic receipts; it does not claim that a real application campaign has executed. A candidate `candidate_success` field has no authority. Production closure must resolve actual independently accepted operation records and recheck their integrity, required membership and source/tree binding. It cannot create a missing result, reinterpret a failed result, or accept a candidate-written success file.

## Complete execution and monotonic stage cases for TASK-071

The tiny inventory uses source commit, package `tiny`, target `unit`, primary profile and four exact historical test names. The observer executes a real `unittest` suite without fail-fast, collecting each actual test result and discovery list. `test_future` intentionally fails and has one fixed future correction owner. `test_closed` is already closed. No missing result is an allowed failure.

The independent cases reject partial execution, a dropped test, an unapproved rename, a newly discovered failing test, failure of a closed test, a modified allowed-failure map, profile substitution and absent subordinate output. All execution/accounting failures use `TEST_ACCOUNTING`; modified pinned authority uses `INTEGRITY`. A positive approved relocation maps the original `test_draw` identity to `test_renamed` and requires actual execution of the replacement. This demonstrates that approved identity preservation works without blanket acceptance of renames.

The fixture also records a real shell update/draw observation. The frozen contribution `tiny-shell.route` checks one exact state key and is already closed. The full frame intentionally differs in custom art from its protected oracle expectation; that scenario remains future-owned and unresolved even while the contribution passes. Negative cases require failure when the contribution observation is missing, when its already-closed state regresses, or when a contribution pass is used to claim the entire mismatching parent scenario closed.

Production contribution records follow [shell-contributions.tsv](../shell-contributions.tsv): unique contribution ID, parent scenario, fixed owner, checkpoint, exact state keys, source clause, exact assertions and `required_for_closure`. State/geometry contributions are checked by the independent architecture operation, not disguised as comparator passes. Whole-frame subsets use the separate [shell-frame-contributions.tsv](../shell-frame-contributions.tsv), with exact source-qualified checkpoints and all cells/cursor compared. The baseline owner materializes both tables from frozen source-level definitions before production dispatch. Neither candidates nor accounting may select a subset. A task can close its complete contribution set without claiming a full parent frame passes. Full frames are still captured and compared without masks; failures retain their immutable future owner. A scenario closes only after all required contributions, checkpoints and complete frame/semantic comparisons pass. Previously closed contribution and scenario sets are monotonic.

## Architectural path cases for TASK-072

These are real source mutations, not altered success JSON. The inline-control painter produces the same visible control JSON while bypassing `Widget.draw`; it must fail `ARCHITECTURE`. The Props bypass reads no authoritative enabled policy and allows a disabled update; both call-path and disabled-state checks fail. The valid custom-art variant changes application-owned decorative art while retaining Widget painting and Props-controlled activation; it must pass. This prevents a blanket ban on legitimate application art.

These tiny probes qualify execution and result enforcement. They do not alone prove Rust ownership, all widget families, registry completeness, PARTS equality, historical API obligations, or every application migration. The planning-owned producer supplies `architecture-bootstrap-driver.py`, `architecture-bootstrap-protocol.md`, `architecture-bootstrap-vectors.json`, and `architecture-bootstrap-fixture/` in this evidence directory. That independent preparation is a prerequisite to TASK-072 dispatch, not work delegated to TASK-072 or the downstream TASK-073. TASK-072 must additionally pass this actual-Rust bootstrap and the independently frozen actual-source checks and architecture mutants enumerated in the architecture matrix, including source-level checks where runtime observation alone cannot establish absence. Existing canonical commands remain mandatory. Test fixtures and mutant definitions must be accepted outside TASK-072's writable implementation before dispatch; candidate-authored checker tests cannot become their sole authority.

## Readiness and limits

The preparation self-test and independent review must pass against exact frozen bytes before the three runner tasks dispatch. Then the submitted implementations must pass their own group and all already accepted groups. The complete application inventories, numerical action expansions, schema-3 oracle captures and actual Rust source probes remain explicit prerequisite deliverables; this fixture never claims they already exist. Independent final review must reject any catalog which treats this synthetic positive as completed production parity, or which allows new probe definitions to be chosen by the implementation being tested.
