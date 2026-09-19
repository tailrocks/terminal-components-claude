# Current preparation evidence — 2026-09-19

Status: **NO-GO.** This file is a provenance index, not a verifier receipt,
task acceptance, dispatch authorization, ledger-arm operation, or product
completion claim.

## Source and refs

```text
tested preparation payload: bc4e5980256f1fa2c66d673790c99610b610fd16
tested tree:                1a388126806c701ff4420f17023a8b792bfd4b98
tested parent:              1da58a09f420b653195b5d8015ed5ca22deb8a6
branch:                     refactor/holla-parity
local main:                 7b27732a8c3c131760ec3438f641cb3c11343a42
remote main:                7b27732a8c3c131760ec3438f641cb3c11343a42
remote campaign tip:       1da58a09f420b653195b5d8015ed5ca22deb8a6
merge-base with main:       7b27732a8c3c131760ec3438f641cb3c11343a42
```

The documentation commit containing this index changes the final tree. A
fresh verifier and reviewer must bind the exact post-commit identity. No
receipt may attest to a future commit containing itself. Final report paths:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
```

Any relevant source, documentation, contract, script, schema, tool,
comparator, oracle, environment, or generated-output change invalidates
affected evidence.

## Oracle identity and inventory

```text
refs/tags/visual-baseline^{commit}: 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
baseline commit tree:              0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
snapshots tree:                    3f0261c32849e26feda24d87697de4a7ce6b8375
keys:                              7,550
ANSI/plain/PNG/HTML:               7,550 each
total artifacts:                   30,200
```

Read-only import:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`

Manifest:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19/sha256.manifest`

Manifest SHA-256:
`95e1f38220bd2fd09da44d3b98590543d1f03837b50e0069bf53bd1f73893637`.
The source identity for the complete snapshot producer is
`89218626011f2f82c4e87c4dfd5868a4c5f3e284` / tree
`6fccf997cd742071ebcff0e0a00e89404ef95ca8`, with the same snapshot tree.

## Qualified tools and command evidence

Taskfmt qualification:

```text
source: /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
source tree: b7d90bd8adbe6c341a08fc485099ee8cf1584431
version: 0.2.0
binary: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-qualified-2026-09-19/install/bin/taskfmt
binary SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

The qualified binary is a regular, single-link executable. Only standalone
`lint` and `verify` operations are allowed. Current lint evidence:

```text
run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-lints-bc4e5980
packages: 73
passed: 73
failed: 0
```

Current native proof evidence:

| external run | evidence |
| --- | --- |
| `proof-full-bc4e5980` | unfiltered nextest exit 100; 37 passed, 1 bounded observer-hang test failed |
| `native-preparation-bc4e5980` | build, prepare, validate, and explicit preflight exit 0; comparator, receipt, contexts, results, index, and observer bind to current tree |
| `adversarial-preparation-bc4e5980` | preparation guards, proof paths, dispatch authorization, and ledger contracts exit 0 |
| `final-checks-bc4e5980` | plan/graph validators exit 0; pre-arm preflight exit 1 on canonical NO-GO |

Raw log hashes:

```text
proof-full-bc4e5980/fmt.log: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
proof-full-bc4e5980/nextest.log: 1bc2e17a848c64595225b172a60816fe878cd27efeb0bf2c5593e26781ea277c
proof-full-bc4e5980/nextest.exit: eea8254c7500ba3de996aa8ad6af399183f04e17d4a8102fde539dbc93a90012
native-preparation-bc4e5980/logs/build.log: 71ea7062bdc25f8bac3d5bef9ef200fa16d1e8e04bd172996ea2627ad1a4c125
native-preparation-bc4e5980/logs/prepare.log: ef068b42f2ba8c0cd3b16a57f4d8e6cc35fe7c92b4cb7633b2c5892be8f0f55e
native-preparation-bc4e5980/logs/validate.log: ef068b42f2ba8c0cd3b16a57f4d8e6cc35fe7c92b4cb7633b2c5892be8f0f55e
native-preparation-bc4e5980/logs/preflight-proof-preparation-explicit.log: fd876a177cd90d4b70ae49ae654fc40d2cd8fd4085ec359828392498efe5d6d5
adversarial-preparation-bc4e5980/preparation-guards.log: e0648f54c9e94357300c05f49585d8791e1d345fad6ebbbeafe7da3ca3203194
adversarial-preparation-bc4e5980/proof-paths.log: af0f83d417cddb93aa373dac58e016082a4331c99d9c271b2ee2785386d27aa5
adversarial-preparation-bc4e5980/dispatch-auth.log: eac0ff019e4587c69a808fa505c2d54ff150007f55449ae673adb71e424bf434
adversarial-preparation-bc4e5980/ledger-contracts.log: 4672dc6ea0d9b583a4da52c1dfc62823eba564c5f8a306771cef97634c5cd66a
final-checks-bc4e5980/plan-validator.log: 5e1d34e81d7e06db01debfb488afc9957f2a4f0bae7a6938d05b41ae93bf61a8
final-checks-bc4e5980/graph-derive.log: 755ba0eca43a62b28a6442121df47ad6797551ace743d1948cd7e4625bce8dde
final-checks-bc4e5980/preflight.log: ca2beee8c16bc0c3adf0531c62af2eaab9cc390644a2dee15c01a0417d50dec8
```

Native binary SHA-256:
`4177ade30c75526d52370d8d9439f8a7df14efca9d732cebf86e58f679051fc4`.
Native build receipt SHA-256:
`b16fc0f3bf67c27c6ab768c467c6ff024de4f60c74e05448c0d95164bb17d5f7`.
Native proof-preparation SHA-256:
`1e0fdea43bc4b085597998e1f4880c09c5733f35733021632538b491f531dfc2`.
Context index SHA-256:
`66f44088c5e6199428643c3e52ad1674cf14419837e1b0ee65d52821ef9cd515`.

The native tests exercise valid launch, direct worker injection, observer
binding, wrong nonce, replay/truncation/empty response, subprocess failure,
timeouts, provider hang teardown, result/context mutation, link substitution,
stale/cross-run identity, and canonical target-name enforcement. A worker’s
success string or zero taskfmt exit is never acceptance.

## Graph and architecture

```text
direct packages: 73
recursive verify.toml: 77
direct checks: 506
recursive checks: 526
dependency edges: 276
max depth: 35
file conflicts: 193
serialization pairs: 0
source obligations: 1,174
traceability rows: 3,256
accepted production tasks: 0
```

Plan validator and graph derivation exit 0. The generated graph is acyclic,
contains no status/acceptance authority, and retains all original task IDs.
`TASK-001`/`TASK-070` are retired fail-closed; `TASK-071`/`TASK-072` are
blocked qualification prerequisites; `TASK-002`–`TASK-069`/`TASK-073` remain
valid implementation obligations.

Target architecture obligations and product leaks are recorded in
`docs/refactoring-plan/architecture.md` and branch-diff reports. The current
source still has compatibility painters/ownership/application migration work;
preparation did not implement it.

## Complete calibration results

Exact-tag control run:

```text
run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-tag-control-4a79-2026-09-19
tests: 302 matrix tests started
result: all 302 failed; log ends `error: test run failed`
observed canonical artifacts: 7,549 ANSI, 7,549 TXT, 7,549 PNG, 7,549 HTML
missing observed key: tablepro/query/results/160x50/nocolor
log SHA-256: c4128ac22c6b99da3e2dee865c6fae2cafb8c3d1f2f9bfc60657a423460e5168
```

The complete expected corpus remained 7,550 keys and 30,200 artifacts. HTML
diffs include the absolute compiled executable path from the external target;
pixel output may be identical while byte output is not. Exact equality does
not permit broad normalization. The protected target symlink was not used to
hide this difference.

A fresh repeat was started in
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-tag-control-current-1da58a09`.
It was terminated after 1:03:39 at its exact process group (`pgid 26873`)
because no durable result appeared and it was consuming the host; no exit file
was written. It is incomplete evidence, not a pass.

Independent 892 control:

```text
302 cases; 298 passed; 4 failed; 2 skipped; 1 leaky; exit 100
7,550 artifacts of each type; 17 diff files
summary SHA-256: 33034862cfb3d7ae2676bbe01269db5ac0c1af6d3cf6e61896f408c36eb14d3c
```

These failures are blockers, not future acceptance. No expected output was
modified, accepted, masked, or replaced.

## Platform, ledger, and final decision

Native macOS preparation checks are available under the run root. Shell syntax,
ShellCheck, actionlint, plan, graph, taskfmt lint, and proof-format checks pass
for their tested payloads. The unfiltered proof nextest run is not passing:
37 tests passed and the bounded observer-hang test failed, exit 100. Required
native Linux execution was unavailable; no cross-platform pass is claimed. The campaign
ledger remains schema-controlled and disarmed; no accepted preparation or
production receipt exists. Any preflight failure due absent current accepted
evidence is correct fail-closed behavior.

The current unfiltered proof run for `bc4e5980` recorded exit 100 with 37
passes; `verifier::tests::provider_hang_after_acceptance_is_bounded_and_rejected`
failed its under-four-second assertion. The raw failure remains binding and is
not erased by focused tests or host-contention hypotheses.

Final verifier and reviewer must independently inspect the final clean tree and
raw proof/calibration evidence. Their paths are the two `final-seal-2026-09-19`
files listed above. Until both current reports explicitly return VERIFIED and
the exact-tag gate passes, the decision remains **NO-GO**.
