# Current preparation evidence — 2026-09-19

Status: **NO-GO.** This file is a provenance index, not a verifier receipt,
task acceptance, dispatch authorization, ledger-arm operation, or product
completion claim.

## Source and refs

```text
tested preparation payload: e10fd942d8350f33f4c29761c322090494d178a4
tested tree:                c336ab40031a5cae2f936bc04fe2fcc62f9d8101
tested parent:              30fbf36feea797d1320c4ab21c491f8c944935bf
branch:                     refactor/holla-parity
local main:                 7b27732a8c3c131760ec3438f641cb3c11343a42
remote main:                7b27732a8c3c131760ec3438f641cb3c11343a42
remote campaign tip:       f5013f609aed1ba32ce60352b38fd0b1b11b063c
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
run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-lints-de2f1295
packages: 73
passed: 73
failed: 0
```

Current native proof evidence:

| external run | evidence |
| --- | --- |
| `proof-full-e10fd942-rerun` | `nextest.exit=0`; 38 passed, 0 skipped |
| `native-preparation-e10fd942-clean` | `build.exit=0`; comparator, receipt, contexts, results, index, and observer external and bound |
| `native-launch-regression-e10fd942` | `native-launch.exit=0`; `native launch regression: PASS` |
| `native-preparation-e10fd942-clean` preflight | `preflight-proof-preparation.exit=0`; exact native proof member bindings pass |
| `target-invariant-verify` | `fmt.exit=0`; targeted nextest exit 0; 1 passed, 37 skipped |

Raw log hashes:

```text
proof-full-e10fd942-rerun/fmt.log:       e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
proof-full-e10fd942-rerun/nextest.log:   0a316130db91f2dbb643df3e58f9b3d89e4a98d6df518908afa33cbab3d55c0d
native-preparation-e10fd942-clean/build.log: e887dcf58721c0e579dae20a7f297109f5e9324251c35c3760ad6328db94159b
native-preparation-e10fd942-clean/prepare.log: 785c0a78b9abbc994f38cb069fb0b87766e03f0cb9ab11a372362f4f497de9d1
native-preparation-e10fd942-clean/validate.log: 785c0a78b9abbc994f38cb069fb0b87766e03f0cb9ab11a372362f4f497de9d1
native-preparation-e10fd942-clean/preflight-proof-preparation.log: 8e5466715415c0d4761db11cb88e34f8c43f80f8ee020f3a5087587fcc541b95
native-launch-regression-e10fd942/native-launch.log: 8af2c83cfcb72e21f367577ad64cbe70a24faf6a07409d738a80c8f80f934e0a
```

Native binary SHA-256:
`22b9f20a9883f00e6dc7bcbc57c818207b95f9c66350e05c74e3ef38caf81b11`.
Native build receipt SHA-256:
`277f9d2343952ded185ba03d7b1e96779ad997d134fe5463b5e7291651186a88`.
Native proof-preparation SHA-256:
`70f16d1674eb569ae3ce392bc22a59a79e5b01db3682503f834e910667421209`.
Context index SHA-256:
`76465ebde015a5f5855a840c639c361938711d24f682775292584905f7c7d71b`.

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
ShellCheck, actionlint, plan, graph, taskfmt lint, proof format, and proof
nextest evidence pass for their tested payloads. Required native Linux
execution was unavailable; no cross-platform pass is claimed. The campaign
ledger remains schema-controlled and disarmed; no accepted preparation or
production receipt exists. Any preflight failure due absent current accepted
evidence is correct fail-closed behavior.

The first final proof wrapper run for `e10fd942` recorded exit 100 with only 37
reported passes and no failure detail; it is rejected. The uncensored rerun
above is the current proof result: 38 passed, exit 0.

Final verifier and reviewer must independently inspect the final clean tree and
raw proof/calibration evidence. Their paths are the two `final-seal-2026-09-19`
files listed above. Until both current reports explicitly return VERIFIED and
the exact-tag gate passes, the decision remains **NO-GO**.
