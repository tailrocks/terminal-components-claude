# Independent final verification review

Status: **changes required; not accepted for dispatch**. Review date: 2026-09-11. This is an independent review of planning and qualification preparation, not an execution of terminal-components refactoring or acceptance of the future proof harness.

## Scope and evidence identity

Read the complete `PLANNING_GOAL.md`, root execution plan, proof contract, verification contract, comparator and runner protocols, runner implementation and worker, comparator fixtures/report validator, bootstrap review, host vectors and selected host implementation, app/component inventories, and protected baseline/accounting/architecture/closure obligations. Parsed all 73 canonical `verify.toml` files: 503 checks, including 479 context-bearing invocations. These are catalog counts, not executed production checks. The catalog and evidence were changing concurrently; findings apply to the exact reviewed bytes below and require re-review after correction.

The reviewed UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`. The existing tui-snap evidence names tested, unmerged PR #1 head `883d03f19d890bbbf27468798db78b04e85297ac`. This review does not repeat that tool's production qualification or infer terminal-components parity from it. TASK-001 and TASK-070–072 remain future implementation tasks. Prepared fixture success does not mean those implementations exist or passed.

Review SHA-256 identities:

| Input | SHA-256 |
| --- | --- |
| `PLANNING_GOAL.md` | `728c65a7a771ed2cd8ba15c78e685e889244a9fa339a18dc2dc2cde0f3d4cbe5` |
| `REFACTORING_COMPLETION_PLAN.md` | `91e0aa45b6ca74fd6fdaf9776776709dd5c9dde263894956c509b731cddc17ef` |
| `proof-contract.md` | `451e6fb017f868198552a2abe49839272fb7cf4a5edd1ab8eea2ede63018ae74` |
| `verification.md` | `fe103ab04f57e78a9f416597fac0d55abead65c5f9bda409f7a3d115e8a61059` |
| `proof-comparator-bootstrap.py` | `c3a646b1759024521ab650051bc76af37bdf54bc8067d21c157c5465053bfda4` |
| `proof-comparator-vectors.json` | `cd2a9689d47bbad3b0c00ddcff70d47f581cbee4a5103ed6f762bdb4a3a49c67` |
| `runner-bootstrap-driver.py` | `5fc31127c563a6c6f98cbb91ab60fe2676f1bb8fdf3b4859fb46d84075f59c68` |
| `runner-bootstrap-worker.py` | `a1ff1fa52f212a53b6c6176a4d76b67a1ad61388a52c5f6d44ad67ad7425f09e` |
| `runner-bootstrap-protocol.md` | `17485f6be386165aae3943a2d94a4a565454de14fdbffb987dcbc85445e10947` |
| `070/verify.toml` | `8f24f8eeaad15229973fa0c7e7f07a4e348915880e93375da77cc512167e76d3` |
| `071/verify.toml` | `18f33a32b86231737d3b2b6fcb5f7867ac5a03f2b741132c02e127746cef857d` |
| `072/verify.toml` | `224cf7f14f1dbb2f82bebcf9a7db39b8555e321f45a32fab575b39d0a7d57c62` |

The temporary evidence file `/tmp/tc-verification-final.opiCMQ/evidence.json` records all 73 individual package hashes and the finite-case inventory. Its SHA-256 is `02d298872f2175c00035a3994b999746d1d25e9eb880b34689ba39dac5d0a4b8`. Reproducer scripts remain in that temporary directory; the decisive observations and reconstruction instructions are retained below so this report does not rely on temporary retention.

## Executed preparation checks

- `rtk proxy python3 -B -O docs/refactoring-plan/evidence/proof-comparator-bootstrap.py --self-test`: passed 62 fixture constructions, 744 malformed-report rejections and seven deliberately incorrect runners. `qualified_production_harness` was explicitly false; tool validation was not requested.
- `rtk proxy python3 -B -O docs/refactoring-plan/evidence/runner-bootstrap-driver.py --self-test`: passed 51 cases, 27 actual observer cases, 204 malformed-result rejections and the zero-worker liar rejection.
- Constructed all 62 comparator fixtures independently and inspected protected contexts: every `required_count` was exactly one.
- Parsed every canonical check against the runner's actual group choices: 15 invalid group arguments, across all three runner producer tasks.
- Called the comparator report validator with two protected required IDs and an incomplete one-result success report: it returned no problems. VF-02 preserves the exact input below.

No production source, real oracle output, integration branch, remote ref or accepted receipt was changed. Temporary synthetic fixtures were disposable.

## Findings

### VF-01 — P1: All runner producer tasks invoke nonexistent qualification groups

Sources: `refactoring-tasks/terminal-components/completion/070/verify.toml:17`, `071/verify.toml:17`, `072/verify.toml:17`; `evidence/runner-bootstrap-driver.py:29` and `:540`.

The task packages pass `--group 70`, `71` and `72`. The actual parser accepts only `070`, `071` and `072`. All 15 such checks fail during argument parsing, before a submitted implementation runs. This prevents the runner, accounting and architecture prerequisites from qualifying and therefore blocks the entire production graph. Repeating the same command under focused/regression/lint/gate phases does not add proof.

Reproducer:

```sh
rtk proxy python3 -B docs/refactoring-plan/evidence/runner-bootstrap-driver.py --runner /usr/bin/true --group 71
```

Observed exit 2: `argument --group: invalid choice: '71' (choose from '070', '071', '072')`.

Required repair: align the generated canonical argv with the frozen driver's ABI, including prerequisite regression groups. Add a catalog-to-parser smoke check for every existing qualification command, so task-format schema validity cannot conceal an invalid tool invocation. Re-review exact generated package bytes.

### VF-02 — P1: Comparator preparation accepts incomplete success when membership exceeds one

Sources: `evidence/proof-comparator-bootstrap.py:204`, `:225`, `:246`; `evidence/proof-comparator-protocol.md`, “Manifests and observations” and “Results and failure classes”; `proof-contract.md:154`.

The report validator compares a successful report only with `required_ids[0]` and hardcodes checked/passed counts to one. It validates the reported required count against context but never requires checked/passed counts or result membership to exhaust that context. All 62 fixtures use one checkpoint, so no current case exercises the condition. A normal first-item-only aggregation defect can satisfy the finite suite while leaving later scenarios unchecked. This is a demonstrated defect in qualification preparation, not a claim that an unimplemented production comparator passed.

Independent reconstruction: import `proof-comparator-bootstrap.py`; set context `run_id="independent-run"`, `task_id="independent-task"`, `required_ids=["first","second"]`, `required_count=2`; calculate context SHA using its `canonical` and `sha` helpers. Pass this otherwise complete report to `validate_report(report, context, canonical(context), {"code": None})`:

```json
{"schema":"tc-proof-comparison/v1","run_id":"independent-run","task_id":"independent-task","context_sha256":"f7485fc579552e43b3db4a97c4f419c14f37cab8d8a37bf7ababec55a435fa80","required_count":2,"checked_count":1,"passed_count":1,"results":[{"id":"first","status":"passed"}],"failures":[]}
```

Observed result: `[]`; the incomplete report is accepted. The exact probe is `/tmp/tc-verification-final.opiCMQ/probe-report.py`, SHA-256 `0cf5e4c42b102d4c721382f8701484819befec79b6dfe26c9712b497ef6d6d33`.

Required repair: derive complete result membership and counts from protected requirements. Independently freeze positives with multiple scenarios and multiple checkpoints per scenario, plus failures in only the first, middle or final member, omission of a later member, duplicate result identities and cross-scenario checkpoint substitution. Define mixed pass/fail counts precisely. Reject a first-item-only comparator and rerun recovery positives. Increasing unrelated mutation counts does not cover aggregation.

### VF-03 — P1: Per-check context-index authority has no executable qualification contract

Sources: `proof-contract.md:115` and `:117`; `evidence/runner-bootstrap-driver.py:207` and `:245`; `evidence/host-bootstrap-vectors.json`; all context-bearing canonical invocations.

The repaired plan now requires an immutable index binding every child check's bytes, schema, operation, lane/namespace, required set and output identity. This is a necessary trust boundary: individually valid contexts must not let the direct lane borrow the PTY result or let `close` join another check's evidence. The actual runner fixture creates one `inputs/context.json`, without a check ID or context index. The host vectors contain no context-index/member mutations. Neither suite currently exercises the new boundary, although the prose assigns its independent qualification to TASK-001/TASK-070 before consumers dispatch.

Reproducer: inspect the complete case manifests and run `rg -n 'context-index|context_index|contexts/'` over both drivers, the host vectors and runner protocol. At the reviewed bytes there is no index fixture. The fixture constructor independently confirms one context per invocation; testing that single file's hash does not test a sealed multi-check index.

Required repair: provide a strict index/member schema and an independently authored multi-operation campaign fixture before freezing bootstrap inputs. It must reject missing/extra child contexts, swapping direct and PTY children, a correct child from another run/tree/task, output-identity aliasing, mutation between operations and replay of a completed check under a second ID. Include a passing immutable sequence and failed-member closure rejection. The implementation owner cannot author the sole qualification of this authority boundary.

### VF-04 — P2: Accounting qualification omits assertion preservation and successful stage closure

Sources: `071/trusted/obligations.md:5` and `:9`; `066/trusted/obligations.md:15` and `:19`; `evidence/runner-bootstrap-driver.py:103`, `:157`, `:194`, `:416`, `:430`; `evidence/runner-bootstrap-worker.py:54`.

The protected contracts require exact source assertions, failure classification, complete identities and empty unresolved sets at closure. The actual 13 accounting cases use one package/target/profile, never mutate the committed test body, and always retain the same intentionally failing `test_future` and unresolved `tiny-shell`. Renaming or dropping a test is exercised; replacing its assertion with a no-op while retaining its name is not. Positive validation even requires `test_future` to remain failed, so the suite cannot currently qualify the transition from a legitimate future failure to a closed passing obligation.

This leaves two distinct implementations indistinguishable under the supplied corpus: one preserving assertion identity and advancing stage closure, and one merely matching test names/statuses with a fixed unresolved set. The latter violates TASK-066 and final integration requirements. No future production false pass is asserted here; the finite qualification gap is source-proven.

Required repair: add a committed original test and candidate variant with the same identity but a weakened assertion, plus an independently approved replacement/relocation positive. Add a stage sequence with a future failure, its real correction, accepted closure, and subsequent regression rejected despite its former allowance. Exercise an all-green empty unresolved set, changed failure classification, duplicate results, and multiple package/target/profile identities with colliding test names. Freeze these cases before TASK-071 dispatch; candidate accounting unit tests cannot supply their own authority.

### VF-05 — P2: Performance closure requires an independently qualified measurement path that no producer fixture currently covers

Sources: `067/trusted/obligations.md:13`–`:35`; `067/verify.toml:49`; `072/trusted/obligations.md:5`; `evidence/architecture-bootstrap-vectors.json`; `evidence/runner-bootstrap-protocol.md`, “Architectural path cases”.

TASK-067 gives CHK-006 responsibility for actual style-resolution/frame-time share, allocation counts, cache/work counters and strict ratios. Its third clause explicitly requires a negative allocator-count perturbation. CHK-006 invokes the architecture operation produced by TASK-072. The current independent Python architecture cases cover widget/Props ownership; the new standalone Rust corpus covers receiver/source/registry/paint/slot behavior. Neither corpus qualifies measurement collection, denominator selection, release/serial profile enforcement, strict-mode propagation, or allocator perturbation. Performance test source is writable by TASK-067, so its newly authored tests cannot be the sole proof of their own measurement validity.

Required repair: assign and freeze a bounded independent performance qualification pack for the existing architecture/accounting producer before TASK-067 consumes it. Use real allocation/work counters and known perturbations, a production-path witness, wrong denominator/no-op workload negatives, and explicit strict-profile/environment assertions. Keep machine-sensitive timing thresholds tied to the accepted controlled profile. No new terminal-components production implementation is required during planning; exact independent test preparation and producer ownership are required.

## Existing tracked blockers and re-review

The previously recorded host probe-substitution and receipt-append authority findings remain separately tracked in `bootstrap-review.md`; this review does not close them. Likewise, the actual Rust qualification corpus declares `ready_for_072_dispatch: false` with production Grid/Brand/StatusBar, nested provenance and related evidence still uncovered. Prepared standalone Rust probes must not be relabeled as actual production qualification. The parallel architecture and parity reviews own their detailed registry, cross-route and frame-contribution findings.

The capture strategy itself keeps useful boundaries: exact schema-3 cells/cursor plus semantic state, separate direct and PTY oracle lanes, original-source clock adapters, numeric oracle-only action expansion, and immutable artifact/receipt ownership. These are appropriate requirements, but their descriptions do not repair the five concrete qualification gaps above.

Re-review is mandatory after repairs: execute canonical argv against the frozen preparation interfaces, run multi-member comparator adversaries, exercise the new context-index and accounting stage fixtures, inspect performance producer coverage, and verify the updated corpus/package hashes. Then independently review the actual TASK-001/070–072 implementations and host-owned rebuild evidence during later execution. This report grants no oracle seal, production-harness acceptance or merge authority.
