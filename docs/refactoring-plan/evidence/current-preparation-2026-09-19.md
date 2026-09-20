# Current preparation evidence — updated 2026-09-20

Status: **NO-GO.** This file is a provenance index, not a verifier receipt,
task acceptance, dispatch authorization, ledger-arm operation, or product
completion claim.

## Assessed pre-documentation source and refs

```text
assessed pre-documentation payload: 4abd4d7bac18d4d56b601b257e40de98422382a1
assessed pre-documentation tree:    2dc5fdd763321174348a454e866b67eb82dc89ee2
parent:                              b384179cafa5b5da052cf65d9570b713f29471d5
branch:                     refactor/holla-parity
local main:                 7b27732a8c3c131760ec3438f641cb3c11343a42
remote main:                7b27732a8c3c131760ec3438f641cb3c11343a42
remote campaign tip:       4abd4d7bac18d4d56b601b257e40de98422382a1
merge-base with main:       7b27732a8c3c131760ec3438f641cb3c11343a42
```

The worktree was clean at this assessment. This rebinding changes only the
four canonical documents. Any later
documentation commit has a different commit/tree and must be independently
bound. No receipt may attest to a future commit containing itself. Final report
paths:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
```

Any relevant source, documentation, contract, script, schema, tool,
comparator, oracle, environment, or generated-output change invalidates
affected evidence.

All results listed below bind to other source commits/trees, not assessed
`4abd4d7b`/`2dc5fd`; they are historical provenance, not current passes or
receipts. The later preparation commits are `28dc673b`, `6c8f5905`,
`5d311e06`, `c21a1fe5`, `6c4535e7`, `b384179c`, and `4abd4d7b`; historical
evidence cannot be reused across them.

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

At the assessed payload before this documentation rebinding, `tests/visual_baseline/` and
`.config/nextest.toml` exist. The candidate branch has no `snapshots/`
directory and no `parity/evidence.tsv`; the imported oracle remains external
and read-only.

## Qualified tool identity and historical command evidence

Prior results below are historical; current exact-source qualification is
listed separately and still requires final-tree verifier/reviewer acceptance.

Taskfmt qualification:

```text
source: /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
source tree: b7d90bd8adbe6c341a08fc485099ee8cf1584431
version: 0.2.0
binary: /Users/donbeave/.cargo/bin/taskfmt
binary SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

The qualified binary is a regular, single-link executable. Only standalone
`lint` and `verify` operations are allowed. Current exact-source lint evidence
at the assessed payload:

```text
run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/taskfmt-lint-4abd4d7b-f978.log
log: taskfmt-lint-4abd4d7b-f978.log
log SHA-256: ae211906387b9161213e1fbb640fea14a62616d60a1391c7ba72046c8969ceef
contracts: 77 (73 numbered + 4 trusted fixtures)
passed: 77
failed: 0
exit: 0
```

Historical native proof evidence from code tree `37214cfb`, invalid for the
assessed payload:

| external run | evidence |
| --- | --- |
| `final-37214cfb/refactor-proof` | `37214cfb`/`6445c996`; focused nextest 3/3 passed; `nextest.log` SHA-256 `0d649f4300204a5c9c9e2befe93f0cd88a6fa0ffcb0cf47d8beae0e79f8ef4c7` |
| `final-37214cfb/refactor-proof-full-serial` | `37214cfb`/`6445c996`; serial nextest 43/43 passed; `nextest.log` SHA-256 `77e6f97954eae5b88ef2d5725cdac8cab159381f127cd5172e3d2c646c7f71b4` |
| `preflight-current-clean` | historical clean run bound to `9346c104`/`00958932`; exit 1 with correct NO-GO refusal; `result.txt` SHA-256 `ad1e78e8c2c5c14fd068befd42fa7774d115dfd83e91f41784ca2a22e56adf12` |
| `ddce97ec-review` | rejected lifecycle experiment; clippy passed but provider-hang nextest failed at 4.395027375s; raw log SHA-256 `582256c4519180658f4f969afee145c144eb81140cebe2f2813882b40fbaaccd` |
| `proof-full-bc4e5980`, `native-preparation-bc4e5980`, `adversarial-preparation-bc4e5980`, `final-checks-bc4e5980` | historical runs bound to superseded `bc4e5980`; provenance only |

Current pre-documentation proof source `4abd4d7b` / tree `2dc5fdd7` passed the
full Rust proof package:

```text
command: cargo nextest run --locked -j 1 --package refactor-proof
result: 45 passed, exit 0 (3 binaries)
log: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/proof-nextest-4abd4d7b.log
log SHA-256: 6589e2d3aeb67bef443e61d50720f5b8f385b910079c8b71b89dd98cfda15252
```

AP-05 family/request binding passed 14 native tests, 15 Python protocol tests,
and the source/AST/bundle check. These results are invalidated by the
documentation commit and require final-tree reruns.

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

The independent launcher review
`/private/tmp/tc-known-good-control-review-9346.oX9Hqi/` rejected the visual
control for protected-target use; missing source/output freshness and link
controls; accepted `--target-dir`, `--config`, and `--manifest-path`
overrides; missing exact HTML/provenance validation; incomplete tool identity;
destructive scratch cleanup; and `shfmt -d` failure. It is not an acceptance
receipt.

The independent adversarial audit
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/adversarial-fe802/adversarial-proof-contract-audit-fe802.md`
(SHA-256 `e92d7f517c05992afefc0d80bc3b4aef5250b22cb17865ddfb136e12b4ab7f19`)
remains rejected with AP-01–AP-04 open: observer provenance, result closure,
taskfmt sealing, and trust-path consistency. AP-05 request-count/family
binding was repaired in `b384179c`; final exact-tree proof/reviewer evidence
is still absent.

## Static, documentation, and ledger evidence

Historical pre-documentation checks at pushed code `247e47d5`, invalid for the
assessed payload:

```text
Lychee run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/lychee-docs-247e47d5
Lychee result: exit=0, inputs=748
Lychee log SHA-256: 2e3b6e8d8f1f9a9740000b37cb83cec558485df985dadcfd0642052b968c956c

Actionlint run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/actionlint-docs-247e47d5
Actionlint version: 1.7.12
Actionlint result: exit=0
Actionlint log SHA-256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

These checks precede the documentation commit; they are not final-tree
verifier evidence. Historical Lychee/actionlint runs bind to
`9346c104`/`00958932` and remain provenance only.

The detailed calibration report
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/calibration-fe802/CALIBRATION-REPORT.md`
(SHA-256 `c1c8b3221cd51661eb36cd57a4c850ab3a093cb9a5647b16d39d88c33516d55e`)
is historical and binds `fe802534`, not the current post-correction tree. It remains NO-GO:
the complete known-good control did not establish exact parity, including the
missing TablePro key, HTML executable-path bytes, TablePro structural output,
Holla elapsed-time output, and unstable settled transitions.

The ignored ledger is schema-valid but stale:

```text
armed: false
integration_head: a292cf860d87c93fc329d16d2310ca86ad370d3d
catalog tree: 10d44afa017c8f6e68da8b78805477335081efb0
preparation: null
task rows: 4 (all BLOCKED)
receipt keys: task-001, task-070
SHA-256: 9c26f11628fda8a1feb7b248f7a53b05741bf6dd3eaea1d2746c008d5ed52b06
```

No accepted current preparation receipt exists. No evidence in this index
authorizes dispatch or ledger arming.

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

Native macOS preparation checks are available under the run root. The proof
suite at the identical code tree `37214cfb` passed 43/43 under the bounded
serial command recorded above; it is not yet a final-tree receipt. The
previous selected proof at `247e47d5` recorded 2/3 tests passed and
`verifier::tests::provider_hang_after_acceptance_is_bounded_and_rejected`
failed its under-four-second assertion, exit 100. Historical Lychee/actionlint
are pre-documentation evidence. Exact-source Lychee/actionlint and taskfmt still
require final-tree rebinding. Required native Linux execution was unavailable;
no cross-platform pass is claimed. The campaign ledger remains schema-
controlled and disarmed; no accepted preparation or production receipt exists.
Any preflight failure due absent current accepted evidence is correct
fail-closed behavior.

Final verifier and reviewer must independently inspect the final clean tree and
raw proof/calibration evidence. Their paths are the two `final-seal-2026-09-19`
files listed above. Until both current reports explicitly return VERIFIED and
the exact-tag gate passes, the decision remains **NO-GO**.
