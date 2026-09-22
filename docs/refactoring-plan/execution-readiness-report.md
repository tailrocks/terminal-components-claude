# Execution readiness report

**Verdict: NO-GO.**

This is a fail-closed current-target readiness result. It is not product-task
acceptance, implementation acceptance, final parity acceptance, permission to
bless or change snapshots, or permission to arm the campaign ledger. Keep the
ledger `armed = false`.

The report records one qualification receipt (`TASK-071`) below, but the
independent review found its validate context accepts extra inputs. The current
source contains a candidate TASK-072 generated-bundle and broker
field-allowlist repair, but no independent verifier/reviewer receipt accepts
it. The current proof-preparation wrapper has stale bindings. These are open
prerequisites, not accepted production work. Before any future dispatch, the
coordinator must repair and independently verify them, rerun live preflight
against the exact post-repair tree, recheck every dependency and receipt, and
preserve the frozen oracle.

## 1. Source and branch binding

The current documentation-repair target is:

```text
branch: refactor/holla-parity
commit: 72db3941b49a1a2c6258eed6b38fc4b830a217f6
tree:   62eac45530e264ebf2b1290030abae51c35a745d
parent: dcb0fa4e2d7d840b00f05f312bb4e25d72118e1d
```

The external PASS preflight and native build receipt below bind only the
predecessor candidate `01532f6b` / `d9b0fd78`. The current-target preflight
fails closed at the NO-GO readiness gate; its underlying taskfmt/proof
preparation bindings remain stale. The current source includes the TASK-072
repair as a candidate only, with no independent verifier/reviewer acceptance.
This edit changes documentation only, so its eventual commit will have a new
identity; rerun preflight and rebind the exact resulting HEAD before any
dispatch. No ref, workflow, ledger, oracle, task worktree, or product source
was changed.

## 2. Frozen visual oracle

The protected product oracle remains:

```text
tag:             refs/tags/visual-baseline
peeled commit:   4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
tag tree:        0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
snapshots tree:  3f0261c32849e26feda24d87697de4a7ce6b8375
matrix keys:     7,550
artifacts:       30,200 (7,550 each: ANSI, plain text, PNG, HTML)
```

The baseline branch, tag, release, snapshot store, and expected artifacts are
read-only inputs. The complete gate must still compare all 7,550 keys and all
30,200 artifacts across every recorded size, color mode, application, state,
interaction checkpoint, and settled PTY transition. Candidate output must
never be blessed as a replacement oracle.

The candidate contains the tag-derived visual suite and nextest configuration,
but not the protected `snapshots/` store or `parity/evidence.tsv`. No final
visual acceptance run is claimed here. Historical census mismatches in
`tablepro/connections/form_advanced` remain product obligations, not reasons to
change the oracle or weaken comparison.

## 3. Catalog and executable graph

Current structural identities at the bound source tree:

```text
catalog/task index:
  commit: 72db3941b49a1a2c6258eed6b38fc4b830a217f6
  tree:   62eac45530e264ebf2b1290030abae51c35a745d
  path:   docs/refactoring-plan/task-index.tsv
  SHA-256: fb6d553f436a29c16f33e48f0dc862577cb9be0e733ab9a18e54318dee117b72

task graph:
  commit: 72db3941b49a1a2c6258eed6b38fc4b830a217f6
  tree:   62eac45530e264ebf2b1290030abae51c35a745d
  path:   docs/refactoring-plan/task-graph.json
  SHA-256: 34bce5b71e82d6d3eeda5ac390fd92a8cc901431a69f6919ee5e0fc391b41916
```

The live validation summary is `validate-plan-summary.json`, exit 0, SHA-256
`1320a4213e97d0eb0759ebcce178492d9a1b939180256bfeca0d40845e0a0743`.
Its graph is acyclic and structural; it contains dependencies, conflicts,
locks, interfaces, and verification waves, but no status, acceptance, or
dispatch authority.

| measure | current live value |
| --- | ---: |
| direct task packages | 79 |
| recursive `verify.toml` contracts | 83 |
| direct checks | 533 |
| recursive checks | 553 |
| dependency edges | 282 |
| maximum dependency depth | 33 |
| longest-path count | 24 |
| file-conflict pairs | 205 |
| serialization locks | 6 |
| shared interfaces | 584 |
| shared source groups | 859 |
| source groups | 1,174 |
| traceability rows | 3,256 |

`TASK-001` and `TASK-070` are retired and fail closed. `TASK-071` has
historical qualification evidence recorded below but is not currently accepted.
`TASK-072` has a candidate generated-bundle and broker field-allowlist repair,
but remains a pending, unaccepted qualification prerequisite. `TASK-002`–
`TASK-069` and `TASK-073`–`TASK-079` remain valid implementation obligations.
Accepted production tasks: **0**.

The catalog/policy count drift is repaired in this documentation change. The
counts remain structural only and do not authorize dispatch.

## 4. Frozen tools and current external evidence

Qualified standalone taskfmt:

```text
source:   /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
tree:     b7d90bd8adbe6c341a08fc485099ee8cf1584431
version:  0.2.0
binary:   /Users/donbeave/.cargo/bin/taskfmt
SHA-256:  f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

The fresh external taskfmt lint run passed all 79 numbered packages, exit 0:

```text
run:      /private/tmp/campaign-readiness-final-20260923/taskfmt-lint-final.log
log SHA-256: 463e6ae9f02d1eea67174adf41fc119234b120d458add6a0bbb544077d9571f2
```

The current native comparator build is externally materialized and binds the
source candidate above:

```text
receipt:  /private/tmp/campaign-readiness-final-20260923/target/debug/tc-proof.build.json
receipt SHA-256: 2181b566f7a1c7d0effde51ee9aee9121e3d12e028a74746c247bb018fef43a7
binary:   /private/tmp/campaign-readiness-final-20260923/target/debug/tc-proof
binary SHA-256: 524928c12cbe8e639f129215c2e000c41f28a2e7de7e9740c26cd285b15ddacf
command:  cargo build --locked --offline -p refactor-proof --bin tc-proof
build exit: 0
```

The named fresh pre-arm preflight passed without dispatch or integration:

```text
report:   /private/tmp/campaign-readiness-final-20260923/preflight-after-proof-20260923.json
report SHA-256: 02afb5aee46cc7ac1a9ab159425060263106eba4aa3aef8256390d753ab7e05f
log:      /private/tmp/campaign-readiness-final-20260923/preflight-after-proof-20260923.log
log SHA-256: 54f8df35caa07b2a2d47a0afa7f4b8bfc10b530e78e5bffb010f3c5fdadd2b19
exit:     0
verdict:  PASS
```

That preflight reports the tag/tree, branch, worktree, ledger, native proof,
taskfmt, host-local task paths, graph validation, and readiness bindings as
passing. It is predecessor-candidate evidence only, not a seal for the current
target or the new documentation commit.

The current-target `scripts/campaign-preflight.sh preflight` exits 1 at the
canonical NO-GO readiness check. Its underlying ledger validation remains
stale: the local disarmed ledger binds
`/Users/donbeave/.cargo/bin/taskfmt`, while the qualified expected path is
`/private/tmp/taskfmt-latest-install/bin/taskfmt`. Independent review also
found the proof-preparation wrapper stale and `TASK-071` validate context
accepting extra inputs. The TASK-072 generated-bundle and broker
field-allowlist repair is a candidate source change only; no current-target
readiness PASS or independent TASK-072 verifier/reviewer receipt exists.

## 5. TASK-071 qualification evidence (not accepted current)

The disarmed `.campaign/ledger.json` records historical qualification evidence
under `task-071`; independent review found that its validate context accepts
extra inputs, so it is not an accepted current prerequisite. The supplied
predecessor snapshot's ledger SHA-256 is
`def3f9e898ea2b0eacd247763a9aaf6120de6a3edf652f063a957b697fc3dd77`.
The ledger and the supplied preflight snapshot agree on `armed=false` and
integration head `01532f6b9442c573efa6224d206788a6c9404f93`.

```text
task:             TASK-071
status:           integrated qualification receipt
base:             eff3815aae19a361fdc1016bf1ac1558dbb86047
candidate commit: 0e6d706d1a59f2ea1edfba6f16501eb798182af4
candidate tree:   92963fc5e6c2e72bdc1942128e60918834fd853a
integration:      0e6d706d1a59f2ea1edfba6f16501eb798182af4
run:              /private/tmp/task071-proof-final-ampere-retry-20260922/run
verifier result:  exit 0, final line DONE, status passed
result SHA-256:   a8b3f7c2d54458a0a4a49ef178c6821aa580338561d997d2b1ff73ae030883c3
reviewer:         VERIFIED
```

This evidence does not accept a product change, authorize a task, prove final
architecture, or prove visual parity. The context-input defect must be repaired
and independently verified before TASK-071 can qualify.
The receipt's candidate is not the current coordinator candidate; its ancestry
and dependency binding must be rechecked by every future preflight.

## 6. Remaining work and open blockers

| item | status |
| --- | --- |
| proof-preparation wrapper | **blocking**: stale bindings; repair and independently verify. |
| TASK-071 validate context | **blocking**: extra inputs accepted; repair and independently verify. |
| TASK-072 qualification | **blocking**: candidate generated-bundle and broker field-allowlist repair has no independent verifier/reviewer receipt. |
| current-target preflight | **blocking**: stale taskfmt/proof preparation binding; rerun after repairs. |
| docs-only commit rebind | **open**: this repair creates a new source identity; bind it with a fresh preflight before dispatch. |
| product migration and ownership | **open**: migrate consumers, remove duplicate/compatibility renderers, and preserve component ownership and APIs. |
| application behavior | **open**: Holla, Showcase, Jackin, and TablePro integration; focus, hover, input, selection, scrolling, resize, PTY, lifecycle, and settled transitions. |
| visual parity | **open**: compare the complete 7,550-key / 30,200-artifact candidate output to the frozen oracle; historical `form_advanced` Class A/B differences remain product work. |
| performance and static gates | **open**: allocation/performance budgets, workspace tests, API/static checks, documentation/link gates, and required platform evidence. |
| Linux native evidence | **open/unavailable** in this environment; supplementary CI cannot replace the required native evidence. |
| protected oracle import | **open**: no candidate output may replace missing external oracle data; a final verifier must import and bind the exact read-only store. |
| ledger arm | **binding**: remain `armed=false`; no ledger arm, task dispatch, snapshot update, ref update, or merge to `main`. |

The final campaign gate still requires independently reviewed task receipts,
full locked/unfiltered `cargo nextest`, required static/API/documentation
checks, real behavioral verification, performance evidence, complete settled
PTY replay, all 7,550 keys, all 30,200 artifacts, clean ancestry, and an
independent final verifier/reviewer decision. No such final acceptance is
claimed here.

This NO-GO remains binding until the listed prerequisite repairs and exact
post-repair preflight pass. The product and final-parity gates remain open, and
the ledger remains disarmed.
