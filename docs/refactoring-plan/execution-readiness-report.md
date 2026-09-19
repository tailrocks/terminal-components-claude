# Execution readiness report

**Verdict: NO-GO.**

This is the sole current readiness authority for `refactor/holla-parity`. It
is a preparation gate, not an implementation prompt. Keep the campaign ledger
`armed = false`; do not dispatch production refactoring tasks.

## 1. Exact source and branch truth

The tested preparation payload before this final documentation freeze is:

```text
branch: refactor/holla-parity
commit: bc4e5980256f1fa2c66d673790c99610b610fd16
tree:   1a388126806c701ff4420f17023a8b792bfd4b98
parent: 1da58a09f420b653195b5d8015ed5ca22deb8a6
```

Git facts independently read from refs:

```text
refs/heads/main:                  7b27732a8c3c131760ec3438f641cb3c11343a42
refs/remotes/origin/main:         7b27732a8c3c131760ec3438f641cb3c11343a42
refs/remotes/origin/refactor/...  1da58a09f420b653195b5d8015ed5ca22deb8a6
merge-base campaign/main:         7b27732a8c3c131760ec3438f641cb3c11343a42
local commits ahead of remote campaign: 1
```

The final verifier must recompute these values after this documentation
commit. Any relevant edit invalidates affected evidence. Final exact-tree
reports must be external and must bind the post-documentation commit/tree;
their paths are:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
```

No accepted preparation receipt exists. Historical report hashes and prior
NO-GO identities are not current bindings.

## 2. Immutable baseline and exact corpus

```text
tag peel:       4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
tag tree:       0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
snapshots tree: 3f0261c32849e26feda24d87697de4a7ce6b8375
keys:           7,550
artifacts:      30,200 (ANSI/plain/PNG/HTML: 7,550 each)
```

Read-only import:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`

Manifest:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19/sha256.manifest`

Manifest SHA-256:
`95e1f38220bd2fd09da44d3b98590543d1f03837b50e0069bf53bd1f73893637`.
The import was derived from the peeled tag, has zero symlinks, and was not
used as a candidate-generated baseline. Protected refs and artifacts are
unchanged.

## 3. Catalog and executable DAG

The validated structural graph contains 73 direct packages, 77 recursive
contracts, 506 direct checks, 526 recursive checks, 276 dependency edges,
depth 35, 193 file-conflict pairs, 0 serialization pairs, 1,174 source
obligations, and 3,256 traceability rows. Plan validation and graph derivation
both exit 0; graph output contains no status or acceptance field.

Disposition:

- `TASK-001`, `TASK-070`: retired lifecycle/bootstrap contracts; fail closed;
  not dispatchable.
- `TASK-071`, `TASK-072`: qualification prerequisites; blocked until fresh
  accepted evidence exists.
- `TASK-002`–`TASK-069`, `TASK-073`: remaining valid implementation work.
- accepted production tasks: 0.

The graph is acyclic and machine-checkable. It encodes dependencies, shared
interfaces, file conflicts, migration boundaries, and verification waves; it
does not encode completion or authorization. Every task must receive
independent verifier and reviewer evidence after safe serial integration.

## 4. Preparation qualification

Qualified taskfmt:

```text
source:   /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
tree:     b7d90bd8adbe6c341a08fc485099ee8cf1584431
version:  0.2.0
binary:   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-qualified-2026-09-19/install/bin/taskfmt
SHA-256:  f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Current package-lint run:
`taskfmt-lints-bc4e5980/summary.tsv`: 73/73 passed, 0 failed. Lint validates
contract format only; it does not accept implementation.

Native proof runs:

| run | command/result |
| --- | --- |
| `proof-full-bc4e5980` | unfiltered `cargo nextest` exit 100; 37 passed, 1 bounded observer-hang test failed |
| `native-preparation-bc4e5980` | build, prepare, validate, and explicit proof-preparation preflight exit 0; exact `bc4e5980`/`1a388126` bindings |
| `adversarial-preparation-bc4e5980` | preparation guards, proof paths, dispatch authorization, and ledger contracts exit 0 |
| `taskfmt-lints-bc4e5980` | 73/73 package lints passed, 0 failed |
| `final-checks-bc4e5980` | plan and graph validators exit 0; pre-arm preflight exit 1 on canonical NO-GO |

The native protocol is non-circular: pin source/tools/contracts/oracle and
prerequisite receipts, materialize contexts, start the observer, execute the
declared operation, validate observed results, then review and seal evidence.
It rejects missing/extra/duplicate contexts, wrong IDs/scope/tree/oracle/tool,
mutations, links, stale/cross-run results, wrong hashes/nonces, replay,
truncation, incomplete close, missing outputs, nonzero/signal/timeout,
observer/taskfmt failure, invalid dependencies, and trust-input mutation.

The native threat model is honest: same-user hostile processes are outside the
isolation claim. Hashes, regular-file checks, path checks, read-only inputs,
observer evidence, and independent exit observation provide integrity and
detection controls.

## 5. Frozen calibration and parity blocker

The exact peeled-tag source was executed against the pre-existing read-only
oracle in external run
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-tag-control-4a79-2026-09-19`.
The log records all 302 matrix tests failing; all 7,550 keys were exercised,
with 7,549 observed ANSI/TXT/PNG/HTML artifacts each. The missing observed key
is `tablepro/query/results/160x50/nocolor`. HTML also differs in the embedded
absolute compiled executable path: expected path uses the protected checkout
target; the external verifier target path appears in actual output. The raw
log SHA-256 is
`c4128ac22c6b99da3e2dee865c6fae2cafb8c3d1f2f9bfc60657a423460e5168`.
The wrapper did not persist a numeric exit file; the log ends with
`error: test run failed`. No normalization, mask, threshold, snapshot bless,
fixture change, or expected-artifact replacement was used.

A separate complete control from source `89218626011f2f82c4e87c4dfd5868a4c5f3e284`
ran 302 cases: 298 pass, 4 fail, 2 skipped, 1 leaky, exit 100. It is not a
clean calibration. These failures are preparation blockers because the gate
cannot yet prove a known-good source against the complete immutable oracle. A
fresh repeat was started in `baseline-tag-control-current-1da58a09`; it was
terminated after 1:03:39 at its exact process group (`pgid 26873`) because no
durable result appeared and it was consuming the host. No exit file was
written, so it is incomplete evidence and not a pass.

## 6. Product obligations, scripts, and platform state

The campaign product remains incomplete: consumer migration and ownership
cleanup, duplicate/compatibility renderer removal, Holla/Showcase/Jackin/
TablePro application integration, focus/hover/input/scroll/resize/PTY and
lifecycle behavior, exact visual parity, performance/allocation budgets, API,
workspace/static/documentation, and platform gates remain implementation
obligations. They are not accepted by the preparation suite.

Current macOS-native preparation checks are recorded under the external run
root. Current shell syntax/ShellCheck/actionlint evidence is pass. Required
Linux execution is unavailable in this environment and therefore is not
claimed. CI is supplementary and cannot substitute for native macOS/Linux
evidence. No Docker, Podman, container, image, mount, firmlink, namespace, or
retired lifecycle path was used by this campaign.

## 7. Blocker register and decision

| blocker | owner | closure condition |
| --- | --- | --- |
| exact-tag known-good matrix fails | next implementation / proof owner | independently execute all 7,550 keys and 30,200 artifacts with exact-safe executable provenance and zero unexplained mismatch |
| observed corpus misses one key and HTML path differs | visual harness owner | repair source/harness provenance without normalization or oracle mutation; rerun complete tag control |
| no accepted current preparation receipt | verifier/reviewer | final clean tree gets independent `VERIFIED` verifier and separate reviewer evidence; no fabricated receipt |
| unfiltered native proof suite fails | proof owner | stable `cargo nextest` pass for all 38 proof tests under the qualified command |
| Linux native evidence unavailable | platform owner | execute required native Linux lane or keep NO-GO |
| product migration/ownership/parity incomplete | implementation goal | complete reconciled DAG and final architecture/product gates |
| ledger must stay disarmed | coordinator | preserve `armed=false` until separate explicit dispatch authorization after fresh readiness recheck |

Resolved preparation defects include stale task graph status metadata, stale
taskfmt path/version enforcement, missing native receipt binding, incomplete
worker launch fixture provenance, unbounded observer-provider teardown,
noncanonical native target names, and ambiguous readiness verdict parsing. The
current code evidence is not a final GO because proof qualification and
calibration remain failed.

**Decision: NO-GO.** The failed exact-tag calibration, failed unfiltered proof
suite, absent accepted receipt, unavailable Linux evidence, and unfinished
product obligations prohibit GO.
The next implementation prompt is explicitly not authorized.
