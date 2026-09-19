# Execution readiness report

**Verdict: NO-GO.**

This is the sole current readiness authority for `refactor/holla-parity`. It
is a preparation gate, not an implementation prompt. Keep the campaign ledger
`armed = false`; do not dispatch production refactoring tasks.

## 1. Exact source and branch truth

The current pushed preparation payload before this documentation update is:

```text
branch: refactor/holla-parity
commit: c42c2ae301197baea6376156a17664d5863497b4
tree:   b841969e714a74740eccb3f7c47eab05cabafed8
parent: b62dd9c350c24deb95e6255cdf7b5987dca13514
```

Git facts independently read from refs:

```text
refs/heads/main:                  7b27732a8c3c131760ec3438f641cb3c11343a42
refs/remotes/origin/main:         7b27732a8c3c131760ec3438f641cb3c11343a42
refs/remotes/origin/refactor/...  c42c2ae301197baea6376156a17664d5863497b4
merge-base campaign/main:         7b27732a8c3c131760ec3438f641cb3c11343a42
local commits ahead of remote campaign: 0
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
NO-GO identities are not current bindings. The documentation commit itself
will change the tested tree; final verification must rebind its post-commit
identity.

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

At current campaign HEAD `c42c2ae3`, `tests/visual_baseline/` and
`.config/nextest.toml` exist. The candidate branch still lacks `snapshots/`
and `parity/evidence.tsv`; those remain external protected-oracle inputs.

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

Pre-documentation package-lint run at pushed code `247e47d5`:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/taskfmt-lint-247-direct/all.log`;
73/73 passed, exit 0; log SHA-256
`afecad09ba800c74fc841fd4d830fd8932d80e84894bd18c5bf5660522134124`.
Lint validates contract format only; it does not accept implementation.

Native proof runs:

| run | command/result |
| --- | --- |
| `final-37214cfb/refactor-proof` | `37214cfb`/`6445c996`; focused nextest 3/3 passed; log SHA-256 `0d649f4300204a5c9c9e2befe93f0cd88a6fa0ffcb0cf47d8beae0e79f8ef4c7` |
| `final-37214cfb/refactor-proof-full-serial` | `37214cfb`/`6445c996`; serial `cargo nextest run --locked -j 1 --package refactor-proof` 43/43 passed; log SHA-256 `77e6f97954eae5b88ef2d5725cdac8cab159381f127cd5172e3d2c646c7f71b4` |
| `preflight-current-clean` | historical clean run bound to `9346c104`/`00958932`; exit 1 with correct NO-GO refusal; result SHA-256 `ad1e78e8c2c5c14fd068befd42fa7774d115dfd83e91f41784ca2a22e56adf12` |
| `proof-full-bc4e5980`, `native-preparation-bc4e5980`, `adversarial-preparation-bc4e5980`, `taskfmt-lints-bc4e5980`, `final-checks-bc4e5980` | historical runs bound to superseded `bc4e5980`; provenance only |

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

The independent launcher review at
`/private/tmp/tc-known-good-control-review-9346.oX9Hqi/` rejected the visual
control for protected-target use; missing source/output freshness and link
checks; accepted target/config/manifest overrides; missing exact HTML and
provenance validation; incomplete tool identity; destructive scratch cleanup;
and `shfmt -d` failure. It is not a qualification receipt.

The independent adversarial proof audit
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/adversarial-fe802/adversarial-proof-contract-audit-fe802.md`
(SHA-256 `e92d7f517c05992afefc0d80bc3b4aef5250b22cb17865ddfb136e12b4ab7f19`)
remains rejected with AP-01–AP-05 open: observer provenance, result closure,
taskfmt sealing, trust-path consistency, and observer request-count binding.

## 5. Frozen calibration and parity blocker

The detailed calibration report
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/calibration-fe802/CALIBRATION-REPORT.md`
(SHA-256 `c1c8b3221cd51661eb36cd57a4c850ab3a093cb9a5647b16d39d88c33516d55e`)
is historical: it binds `fe802534`, not current `c42c2ae3`. Its NO-GO
findings remain provenance and cannot attest to the current or post-document
tree.

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
root. Fresh pre-documentation Lychee passed at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/lychee-docs-247e47d5`
(exit 0, 748 inputs; log SHA-256
`2e3b6e8d8f1f9a9740000b37cb83cec558485df985dadcfd0642052b968c956c`) and
qualified actionlint 1.7.12 passed at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/actionlint-docs-247e47d5`
(exit 0; log SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`).
Historical Lychee/actionlint runs remain bound to `9346c104`/`00958932`.
Required Linux execution is unavailable in this environment and therefore is
not claimed. CI is supplementary and cannot substitute for native macOS/Linux
evidence. No Docker, Podman, container, image, mount, firmlink, namespace, or
retired lifecycle path was used by this campaign.

## 7. Blocker register and decision

| blocker | owner | closure condition |
| --- | --- | --- |
| exact-tag known-good matrix fails | next implementation / proof owner | independently execute all 7,550 keys and 30,200 artifacts with exact-safe executable provenance and zero unexplained mismatch |
| observed corpus misses one key and HTML path differs | visual harness owner | repair source/harness provenance without normalization or oracle mutation; rerun complete tag control |
| no accepted current preparation receipt | verifier/reviewer | final clean tree gets independent `VERIFIED` verifier and separate reviewer evidence; no fabricated receipt |
| final proof receipt/review is absent | verifier/reviewer | rerun and independently seal native proof evidence for the exact final tree |
| Linux native evidence unavailable | platform owner | execute required native Linux lane or keep NO-GO |
| product migration/ownership/parity incomplete | implementation goal | complete reconciled DAG and final architecture/product gates |
| ledger must stay disarmed | coordinator | preserve `armed=false` until separate explicit dispatch authorization after fresh readiness recheck |

The ignored ledger is schema-valid but stale: `.campaign/ledger.json` has
`armed=false`, `integration_head=a292cf860d87c93fc329d16d2310ca86ad370d3d`,
catalog tree `10d44afa017c8f6e68da8b78805477335081efb0`, no preparation
receipt, four blocked task rows, and receipt keys `task-001`/`task-070`.
Its SHA-256 is
`9c26f11628fda8a1feb7b248f7a53b05741bf6dd3eaea1d2746c008d5ed52b06`.
It must not authorize the current branch.

Resolved preparation defects include stale task graph status metadata, stale
taskfmt path/version enforcement, missing native receipt binding, incomplete
worker launch fixture provenance, unbounded observer-provider teardown,
noncanonical native target names, and ambiguous readiness verdict parsing. The
proof suite now passes under the bounded serial command, but the current
documentation payload still requires final-tree rebinding and independent
receipt/reviewer evidence.

**Decision: NO-GO.** The failed exact-tag calibration, rejected visual
launcher/trust audit, absent accepted receipt and independent final review,
unavailable Linux evidence, and unfinished product obligations prohibit GO.
The next implementation prompt is explicitly not authorized.
