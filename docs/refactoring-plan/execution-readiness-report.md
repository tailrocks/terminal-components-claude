# Execution readiness report

**Verdict: GO.**

This is a preparation-qualified GO for **future graph-ordered dispatch only**.
It is not product-task acceptance, implementation acceptance, final parity
acceptance, permission to bless or change snapshots, or permission to arm the
campaign ledger. Keep the ledger `armed = false`.

The report records one accepted qualification receipt (`TASK-071`) below. It
does not accept any production task, final campaign state, visual result, or
product behavior. Before any future dispatch, the coordinator must rerun the
live preflight against the exact post-documentation tree, recheck every
dependency and receipt, and preserve the frozen oracle.

## 1. Source and branch binding

The supplied fresh evidence was captured from this clean coordinator
candidate:

```text
branch: refactor/holla-parity
commit: 01532f6b9442c573efa6224d206788a6c9404f93
tree:   d9b0fd78a9ee3024039073805e800c7a57b0bf0f
parent: 27aa11debc9c711223512aca9aebc8f89adb972c
```

The external preflight and native build receipt both bind that exact
commit/tree. This edit changes only this report, so its eventual documentation
commit will have a new identity. The supplied evidence is not a seal for that
future commit; rerun preflight and rebind the exact resulting HEAD before
dispatch. No ref, workflow, ledger, oracle, task worktree, or product source
was changed by this rebind.

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
  commit: 01532f6b9442c573efa6224d206788a6c9404f93
  tree:   d9b0fd78a9ee3024039073805e800c7a57b0bf0f
  path:   docs/refactoring-plan/task-index.tsv
  SHA-256: fb6d553f436a29c16f33e48f0dc862577cb9be0e733ab9a18e54318dee117b72

task graph:
  commit: 01532f6b9442c573efa6224d206788a6c9404f93
  tree:   d9b0fd78a9ee3024039073805e800c7a57b0bf0f
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

`TASK-001` and `TASK-070` are retired and fail closed. `TASK-071` is the
accepted qualification receipt recorded below. `TASK-072` remains a pending
qualification prerequisite. `TASK-002`–`TASK-069` and `TASK-073`–`TASK-079`
remain valid implementation obligations. Accepted production tasks: **0**.

The repository policy text in `AGENTS.md` still declares the older
73/77-package, 506/526-check, 263-edge, 188-conflict summary. The live
catalog and graph above are the measured current files; this documentation
drift is an open rebind blocker and must be reconciled before relying on any
future catalog claim. It does not authorize dispatch by itself.

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
passing. It is bound to the clean source candidate in §1, not to the new
documentation commit produced by this edit.

## 5. Accepted TASK-071 qualification receipt

The disarmed `.campaign/ledger.json` records one accepted qualification
receipt under `task-071`; its current ledger SHA-256 is
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

This receipt is qualification evidence only. It does not accept a product
change, authorize a task, prove final architecture, or prove visual parity.
The receipt's candidate is not the current coordinator candidate; its ancestry
and dependency binding must be rechecked by every future preflight.

## 6. Remaining work and open blockers

| item | status |
| --- | --- |
| docs-only commit rebind | **open**: supplied preflight binds the pre-edit source tree; rerun against the resulting HEAD before dispatch. |
| catalog/policy count drift | **open**: reconcile the `AGENTS.md` 73/77 summary with the live 79/83 catalog and graph counts. |
| TASK-072 qualification | **open**: no accepted verifier/reviewer receipt. |
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

This GO remains limited to preparation-qualified permission for future
graph-ordered dispatch after the exact live post-edit preflight and all
required dependency checks pass. The product and final-parity gates remain
open, and the ledger remains disarmed.
