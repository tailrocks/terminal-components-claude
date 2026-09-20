# Refactoring preparation authority

Status: **NO-GO.** Preparation only. The ledger must remain `armed: false`.
No production task may be dispatched, and this branch must not be merged into
`main`, while the readiness report is NO-GO.

This directory is the canonical preparation package for
`refactor/holla-parity`. The authority order is:

1. repository policy and the user’s preparation boundary;
2. the immutable `visual-baseline` oracle;
3. current source, tests, contracts, Git history, and executable evidence;
4. [`execution-readiness-report.md`](execution-readiness-report.md);
5. task descriptions and historical reports.

Historical evidence never authorizes execution. The generated structural graph
contains no status, acceptance result, or dispatch authority.

## Assessed pre-documentation payload and branch truth

The clean payload assessed immediately before this documentation repair is:

```text
branch:   refactor/holla-parity
commit:   3701841a34cf790dee5ad0c970fd76955e7916d9
tree:     26074440bb5bfbad6fa66a28c5e46cc88e2e7d56
parent:   e2f537d95b2462de6c4f2b517af3e5753e2a7f5e
local main:   7b27732a8c3c131760ec3438f641cb3c11343a42
remote main:  7b27732a8c3c131760ec3438f641cb3c11343a42
remote campaign tip: 3701841a34cf790dee5ad0c970fd76955e7916d9
merge-base with main: 7b27732a8c3c131760ec3438f641cb3c11343a42
campaign commits ahead of origin/refactor/holla-parity: 0
```

The campaign is a descendant of actual local and remote `main`; the remote
campaign ref equals this pushed payload. No ref was reset, rewritten,
force-pushed, pruned, or moved. The directory name `.worktrees/main` was not
used to identify `main`.

The four canonical documents are source-controlled preparation metadata. This
repair changes the assessed payload, so the final verifier must bind the exact
post-documentation HEAD/tree. Evidence is invalidated by any relevant source,
documentation, task contract, schema, script, generated workflow, tool,
comparator, oracle, environment, or generated-output change. The
evidence-sealing protocol is:

```text
commit preparation payload
→ commit canonical documentation
→ freeze the final tree
→ generate external verifier/reviewer reports for that exact tree
→ record their paths and hashes without another tracked edit
```

## Intervening commits after the prior preparation payload

The prior preparation payload was `1fb2b71c23edc6dd7b6d81b2e97b6bd5d4265717`.
The commits through assessed `3701841a` are unreviewed and are not
preparation acceptance evidence:

| commit | observed change | disposition |
| --- | --- | --- |
| `320e9e7d` | Rebound preparation documents to a CI repair | superseded metadata |
| `874d1ff4` | Changed visual-baseline CI exclusions and Rust-version handling | unreviewed CI/test-contract change |
| `9abd3ec3` | Normal revert restoring full workspace/visual coverage | preserve normal revert semantics; unreviewed |
| `aabd4d75` | Pinned Rust 1.98.1 and added mise/workflow requirements | unreviewed toolchain/CI change |
| `9ce787ed` | Generated the velnor-workflow Actions layout | unreviewed generated workflow |
| `d9ddc94a` | Refreshed Cargo.lock pins | unreviewed dependency input |
| `dca523da` | Regenerated workflow state after lockfile drift | unreviewed generated workflow |
| `d8eb545f` | Retargeted generated Actions links | unreviewed policy documentation |
| `356d65e3` | Declared extra gates and generated-workflow inputs | unreviewed CI/toolchain change |
| `afe429c8` | Retired Rust 1.88 claims and changed related contract text | unreviewed source payload |
| `baa147d0` | Rebound bless-guard to generated workflow/mise paths | unreviewed CI/xtask change |
| `e2f537d9` | Split visual size/color bundles into individual native test cases | unreviewed visual-test harness change |
| `3701841a` | Rebound bless-guard and native gates to generated workflow paths | unreviewed CI/xtask/generated-workflow change |

No listed commit authorizes dispatch or changes `NO-GO`/`armed: false`.

## Frozen visual authority

The protected oracle is the peeled tag, not a branch or release label:

| identity | value |
| --- | --- |
| `refs/tags/visual-baseline^{commit}` | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| tag commit tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| `snapshots/` tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| matrix keys | 7,550 |
| artifacts | 30,200: 7,550 ANSI, 7,550 plain, 7,550 PNG, 7,550 HTML |

The exact read-only import is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`.
Its independent manifest is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19/sha256.manifest`;
SHA-256:
`95e1f38220bd2fd09da44d3b98590543d1f03837b50e0069bf53bd1f73893637`.
The import has no symlinks. Protected refs, snapshots, grouped stores,
fixtures, and expected artifacts were not changed.

The complete snapshot-producing source is commit
`89218626011f2f82c4e87c4dfd5868a4c5f3e284`, tree
`6fccf997cd742071ebcff0e0a00e89404ef95ca8`, with the same snapshot tree.
Its control run is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19`.
That run executed 302 cases and produced the complete count, but was not
clean: 298 passed, 4 failed, 2 skipped, 1 leaky, exit 100. No output was
blessed or normalized.

At the assessed pre-documentation candidate `3701841a`, `tests/visual_baseline/` and
`.config/nextest.toml` exist. The candidate branch still has no `snapshots/`
directory and no `parity/evidence.tsv`; those remain protected external oracle
inputs, not candidate-generated expected output.

Parity is exact at cells/styles/layout/ANSI/plain/PNG/HTML and interactive
state transitions. PTY checks must include setup, input, resize, settled state,
exit, restoration, and cleanup. A compatibility painter, copied oracle frame,
or duplicate renderer cannot satisfy the gate.

## Catalog, architecture, and remaining product scope

Machine-checked catalog at the assessed payload (structural only):

| measure | value |
| --- | ---: |
| direct task packages | 73 |
| recursive `verify.toml` files | 77 |
| direct checks | 506: 27 argv, 479 shell |
| recursive checks | 526 |
| dependency edges | 276 |
| maximum dependency depth | 35 |
| file-conflict pairs | 193 |
| serialization pairs | 0 |
| source obligations | 1,174 |
| traceability rows | 3,256 |
| accepted production tasks | 0 |

`TASK-001` and `TASK-070` are retired fail-closed lifecycle/bootstrap
contracts. `TASK-071` and `TASK-072` remain qualification prerequisites.
`TASK-002`–`TASK-069` and `TASK-073` remain valid implementation obligations,
subject to fresh reconciliation by the next goal.

The target architecture is caller-owned state, borrowed props, read-only draw,
runtime-owned routing/focus/layers/pointer/cursor, and one reusable
implementation per component family. Remaining obligations include Showcase
compatibility painting and fixed-grid text, Jackin projections, TablePro
legacy painters, reduced Holla route/scenario coverage, ownership migration,
application integration, behavior, visual parity, performance, and API/static
closure. Preparation did not implement product refactoring.

## Native proof and tools

Qualified standalone taskfmt:

```text
source:   /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
tree:     b7d90bd8adbe6c341a08fc485099ee8cf1584431
version:  0.2.0
binary:   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-qualified-2026-09-19/install/bin/taskfmt
SHA-256:  f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Only standalone `taskfmt lint "$TASK_DIR"` and the documented standalone
`taskfmt verify --root ... --task-dir ... --base ... --progress "" --log-dir ...`
are allowed. Taskfmt is not an orchestrator, workspace manager, ref manager,
container launcher, or acceptance authority.

The native Rust launcher/materializer has implementation and contract claims
for source/tree, scope, oracle, contracts, tool hashes, observer transport,
results, and receipts. Those claims are not qualified: the independent
launcher review rejected the current visual control for protected-target use,
missing source/output freshness and link controls, accepted target/config/
manifest overrides, incomplete HTML/provenance validation, incomplete tool
identity, destructive scratch cleanup, and failed `shfmt -d`. The native
threat model remains limited to integrity/detection controls; it does not
claim isolation from a hostile same-user process.

Historical qualification evidence — invalid for the assessed payload:

- Historical taskfmt lint at pushed code `247e47d5`: 73/73 packages passed, exit 0;
  log `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/taskfmt-lint-247-direct/all.log`,
  SHA-256 `afecad09ba800c74fc841fd4d830fd8932d80e84894bd18c5bf5660522134124`.
- Proof source `37214cfb` has the historical code tree
  `6445c9969ef2f028ff17eb24608de966f1e60b62`. Its focused native nextest run
  passed 3/3 at
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/final-37214cfb/refactor-proof/nextest.log`
  (SHA-256 `0d649f4300204a5c9c9e2befe93f0cd88a6fa0ffcb0cf47d8beae0e79f8ef4c7`),
  and its serial full run passed 43/43 at
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/final-37214cfb/refactor-proof-full-serial/nextest.log`
  (SHA-256 `77e6f97954eae5b88ef2d5725cdac8cab159381f127cd5172e3d2c646c7f71b4`,
  `cargo nextest run --locked -j 1 --package refactor-proof`). These are
  historical to the current documentation payload and require final-tree
  rebinding. The unqualified `ddce97ec` experiment was reverted: its
  provider-hang measured 4.395027375s against a strict `<4s` bound.
- `proof-full-bc4e5980`, `native-preparation-bc4e5980`,
  `adversarial-preparation-bc4e5980`, `taskfmt-lints-bc4e5980`, and
  `final-checks-bc4e5980` are historical runs bound to superseded source;
  they remain provenance only.

The adversarial audit at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/adversarial-fe802/adversarial-proof-contract-audit-fe802.md`
(SHA-256 `e92d7f517c05992afefc0d80bc3b4aef5250b22cb17865ddfb136e12b4ab7f19`)
is rejected with AP-01–AP-05 open: observer provenance, result closure,
taskfmt sealing, trust-path consistency, and observer request-count binding.

Raw runs are under
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19`.
They are supporting evidence, not acceptance receipts.

The clean pre-arm preflight run
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/preflight-current-clean`
exited 1 with the correct NO-GO refusal. Its result SHA-256 is
`ad1e78e8c2c5c14fd068befd42fa7774d115dfd83e91f41784ca2a22e56adf12`; it is
bound to historical `9346c104`/`00958932`, so it cannot authorize the current
or post-documentation tree. Historical pre-documentation Lychee log
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/lychee-docs-247e47d5/lychee.log`
passed with SHA-256 `2e3b6e8d8f1f9a9740000b37cb83cec558485df985dadcfd0642052b968c956c`;
qualified actionlint 1.7.12 log
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/actionlint-docs-247e47d5/actionlint.log`
passed with SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
No listed evidence binds assessed `3701841a`/`2607444`; no exact-source
verifier/reviewer receipt or preparation acceptance receipt exists.

## Readiness and navigation

- [`execution-readiness-report.md`](execution-readiness-report.md) — sole
  current verdict and blocker register.
- [`evidence/current-preparation-2026-09-19.md`](evidence/current-preparation-2026-09-19.md)
  — command, identity, oracle, calibration, and evidence index.
- [`next-implementation-goal.md`](next-implementation-goal.md) — complete
  future prompt, explicitly **NOT AUTHORIZED FOR EXECUTION**.
- [`task-graph.json`](task-graph.json), [`task-graph.md`](task-graph.md), and
  [`task-index.tsv`](task-index.tsv) — structural catalog only.
- [`campaign-policy.md`](campaign-policy.md), [`path-contract.md`](path-contract.md),
  [`proof-contract.md`](proof-contract.md), and
  [`../../refactoring-tasks/visual-validation.md`](../../refactoring-tasks/visual-validation.md)
  — execution, trust, and parity contracts.
- [`architecture.md`](architecture.md), `COMPONENT_ARCHITECTURE.md`, `DESIGN.md`,
  `GOAL.md` — architecture/product obligations, not execution authority.

The final independent verifier and separate reviewer must inspect the clean
post-documentation tree, rerun decisive checks, inspect raw evidence, and
return explicit decisions. Their predetermined external paths are:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
```

Because exact-tag calibration, the unfiltered proof suite, platform evidence,
and product parity remain failed or unavailable, no honest preparation receipt
or GO can exist in this goal. The next goal must revalidate readiness before
any authorization or arming operation.
