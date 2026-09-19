# Refactoring plan navigation

Status: **NO-GO. Preparation only. The campaign ledger remains `armed: false`.**

This directory is the canonical preparation package for `refactor/holla-parity`.
It does not authorize the implementation `/goal`, production task dispatch,
ledger arming, a push, or a merge. The sole readiness authority is
[`execution-readiness-report.md`](execution-readiness-report.md). A NO-GO in
that report is fail-closed.

## Exact candidate, rejected reports, and final seal

The last exact candidate independently sealed before this documentation repair
was:

| item | identity |
| --- | --- |
| branch | `refactor/holla-parity` |
| audited commit | `dd26d2e7cc4ebbf1d8f1af4c54e7081744549382` |
| audited tree | `ed1525875021c613fa9265239ad2f4835f9e1785` |
| audited parent | `a80c790e2ebba15e4a00e7cff4296eab2a5bccc5` |

The actual final exact-tree reports for that payload are:

- verifier: `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md`
  SHA-256 `a3390e74bd0dc85ca3e0ba62a62bb9c7b139851a952c4203781bde4c57462bc0`
- reviewer: `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md`
  SHA-256 `57051451aabf2a31fa1de31cddb4ee3c27f24c281df41755d3846769011fc46d`

Both bind `dd26d2e7` / `ed152587` / parent `a80c790e` and return
**REJECTED / NO-GO**. They are
external audit reports, not acceptance receipts, and do not authorize
dispatch, arming, merge, or product acceptance. The earlier
`a80c790e` / `d7654291` / `2c74b88b` reports are historical rejected evidence
only; they do not describe the sealed `dd26d2e7` payload.

This bounded repair changes only metadata and the evidence index in these four
canonical documents. It does not change code, scripts, task contracts, the
ledger, refs, the protected baseline, or product behavior. This docs-only
commit creates a new Git commit/tree after the sealed payload and therefore
invalidates affected evidence after the commit. A fresh exact-tree
verifier/reviewer seal must bind the post-commit HEAD/tree before any
acceptance or dispatch. The final seal uses these predetermined external
locations, so recording the evidence does not require another source edit:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
```

Any further relevant source, documentation, contract, tool, oracle, or
environment change invalidates affected evidence.

The earlier `a80c790e2ebba15e4a00e7cff4296eab2a5bccc5` / tree
`d765429108cd14109d420faf96ae481e1eae11e1` / parent `2c74b88b9fbfb4568911dc7d57a303a3cb5991cb`
audit and its `final-verifier-refresh-a80c79-2026-09-19` and
`final-reviewer-refresh-a80c79-2026-09-19` reports are historical provenance
only. They are superseded by the exact `dd26d2e7` reports above.

The pre-documentation inspection at `bb574d84` / `6e464830` and the
`74e4ec45` preparation roots are historical provenance only.

## Trust-path repair and independent review

Bernoulli (`gpt-5.6-luna`, max reasoning effort) independently rejected the
prior preparation tree `21f875318f61c30f55bf0a47b8daa4326d48540a7` after
reproducing two trust defects: taskfmt accepted a symlinked final or parent
alias, and the Rust verifier accepted symlinked parent components. Raw review
evidence is retained at:

```text
/tmp/campaign-review-evidence-21f87531.7EkYOO/
```

The defects were repaired, but this historical repair evidence is not
final-payload approval:

- `8e783592afd0a2c2f08076858a386a091a35e712` rejects taskfmt final and parent
  aliases in preflight/dispatch and adds adversarial coverage. Evidence:
  /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-path-repair-2026-09-19.
- `bb574d84bf25ff9179b42e951fca52068f2ab623` rejects symlinked Rust verifier
  trust-path parents while preserving the documented macOS system symlinks.
  Evidence:
  /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/symlink-parent-repair-2026-09-19.
  `final-identity.txt` binds that repair to `bb574d84` and tree `6e464830`;
  sealed nextest, Clippy, and rustdoc exit files are all zero.

The independent rejection is closed as a preparation defect only. The
verifier and reviewer reports above reject the audited exact candidate; a
fresh verifier and reviewer must still evaluate the exact post-refresh tree
and return `VERIFIED`. No accepted receipt exists.

## Authority and navigation

Resolve conflict in this order: repository policy and user scope; the frozen
visual oracle; current source, tests, contracts, and Git evidence; this report
and the campaign contracts; historical notes. Historical evidence cannot
override current source or authorize execution.

- [`execution-readiness-report.md`](execution-readiness-report.md) — current
  verdict, blockers, and acceptance gate.
- [`evidence/current-preparation-2026-09-19.md`](evidence/current-preparation-2026-09-19.md)
  — command/evidence index and provenance ledger.
- [`next-implementation-goal.md`](next-implementation-goal.md) — complete
  future prompt, explicitly **NOT AUTHORIZED FOR EXECUTION** while this report
  is NO-GO.
- [`campaign-policy.md`](campaign-policy.md) — branch, scope, and protected
  ref rules.
- [`subagent-only-policy.md`](subagent-only-policy.md) and
  [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — native
  roles, isolation, integration, and evidence sequence.
- [`path-contract.md`](path-contract.md) and
  [`proof-contract.md`](proof-contract.md) — native paths, trust inputs, and
  result/receipt binding.
- [`campaign-ledger.schema.json`](campaign-ledger.schema.json) — ledger shape.
- [`task-graph.json`](task-graph.json) — generated structure only; it contains
  no authorization or acceptance result.
- [`task-graph.md`](task-graph.md) and [`task-index.tsv`](task-index.tsv) —
  reconciled catalog and dependencies.
- [`../../refactoring-tasks/visual-validation.md`](../../refactoring-tasks/visual-validation.md)
  — exact visual/behavioral comparison contract.
- [`architecture.md`](architecture.md), the branch-diff reports, and the root
  architecture documents — product and target-architecture obligations.

## Frozen baseline and oracle

The protected identity is the peeled tag, not a branch name or release label:

| artifact | immutable identity |
| --- | --- |
| `refs/tags/visual-baseline^{commit}` | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| baseline commit tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| `snapshots/` tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| matrix keys | 7,550 |
| artifacts | 30,200: 7,550 each ANSI, plain text, PNG, HTML |

The exact read-only import and its independent manifest are external:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19
```

The import records the tag commit/tree, snapshot tree, 30,200-file hash
manifest, four 7,550-key lists, and zero symlinks. It is read-only. No
candidate output has been used as an expected artifact.

Three identities must not be conflated:

1. `4a79c0a2` is the protected artifact/oracle commit.
2. `89218626011f2f82c4e87c4dfd5868a4c5f3e284`, tree
   `6fccf997cd742071ebcff0e0a00e89404ef95ca8`, is the Git source identified
   as the snapshot producer. Its `snapshots/` tree is the same
   `3f0261c32849e26feda24d87697de4a7ce6b8375`, with all 30,200 artifacts. Its
   external control root is
   `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19`.
3. `02f5294bfdbf38004cc49130d0aff1d01f31434c` (tree
   `efa2b409b77077caf5c639f7f4c6154cbadbce5`) remains the distinct
   architecture/source oracle. It is not the visual artifact tag.

The earlier `3570a2ed23444dddf1eddcdcc49b654b169038fe` claim is corrected, not
authoritative: its tree is `77d6a559536d6a1d733b52d3b78b5b49112315cb` and its
snapshot corpus is only 28,580 artifacts / 7,145 keys. It cannot calibrate the
complete gate.

## Qualified tools and checks

The only qualified standalone taskfmt is:

```text
source: /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
version: 0.2.0
binary: /tmp/taskfmt-latest-install/bin/taskfmt
binary SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Only `taskfmt lint "$TASK_DIR"` and the documented standalone `taskfmt verify`
form are permitted. Taskfmt is not an orchestrator, workspace manager, ref
manager, container launcher, or acceptance authority.

The qualified taskfmt checkout also contains an upstream-only, opt-in Docker
integration. At revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2` / tree
`b7d90bd8adbe6c341a08fc485099ee8cf1584431`, its exact source evidence is
`harness/tests/docker_itest.rs`, `harness/tests/run_docker_itest.sh`,
`harness/README.md`, and `docs/monitoring.md`. The runner requires
`TASKFMT_ITEST_DOCKER=1`, invokes `cargo test`, and requires a live Docker
daemon and harness image. The platform audit evidence is
`/tmp/taskfmt-platform-audit.log` (full taskfmt workspace nextest stopped at
test-list creation with `docker_itest: SKIP`; exit 104) and
`/tmp/taskfmt-platform-audit-native.log` (native taskfmt crates: 268 passed,
7 binaries, exit 0). This campaign did not run Docker and makes no claim that
the upstream Docker integration passed. Campaign policy excludes that path;
it is upstream-only provenance, not a campaign blocker or acceptance input.

### Exact-tag HTML provenance blocker (Galileo audit)

Galileo's independent exact-tag calibration audit identified a
path-dependent HTML provenance blocker, not a product conclusion. The frozen
HTML from tag `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` / tree
`0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` embeds the literal executable
provenance `/Users/donbeave/Projects/terminal-components-claude/target/debug/<bin>`.
The verifier-owned real target in the exact-tag run root
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-baseline-4a79c0a2`
embeds `RUN/target/debug/<bin>` instead. HTML is therefore not yet a
qualified exact comparison between those builds.

The tag's `tests/visual_baseline/support.rs` uses compile-time
`env!("CARGO_BIN_EXE_*")` at lines 35–38 and builds case argv at lines
423–438. Its `tuisnap` PTY/frame/render/grouped path preserves grouped
expected/actual/diff/report artifacts and performs byte comparisons. Because
the executable path is compiled into the binary, `exec -a`, environment
changes, or a wrapper process cannot repair this provenance after build. The
historical baseline checkout's `target` is a protected shared-cache symlink to
`/Users/donbeave/Library/Caches/mbx/targets/v1/8ebb271f62cb4d1ddf4b4e663c9a473e6cc42793d9dfc918efefcd0a8b9ea8e3`; the path contract forbids using that symlink as the verifier target.

The exact-tag run was incomplete/failed; its raw evidence remains under the
run root above. Galileo observed an HTML byte mismatch during the tag control,
and no output was normalized or blessed. This is a preparation/provenance
blocker, not product acceptance or a product regression. Closure requires a
reviewed exact-safe provenance strategy or an explicitly authorized external
path that makes the frozen and verifier executable identities comparable,
followed by a clean exact-tag rerun and byte comparison. Until then the
preparation remains **NO-GO**.

The detailed exact-candidate audit evidence before this documentation refresh
is raw and rejected, not an accepted receipt for this documentation tree.
Report root:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-verifier-a80c79-2026-09-19
```

Historical tool and pre-refresh evidence identities recorded under that root:

| check | historical evidence and result |
| --- | --- |
| taskfmt | source `/Users/donbeave/Projects/taskfmt/task-format`, revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, source tree `b7d90bd8`, version `0.2.0`, binary `/tmp/taskfmt-latest-install/bin/taskfmt`, SHA-256 `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`; `taskfmt-lints-summary.txt` and `taskfmt-lints.exit` record 73/73 standalone lints, exit 0 |
| proof | `proof-build-target`; `tc-proof-native-build/v1` receipt binds commit `a80c790e` / tree `d7654291`; binary SHA-256 `192423f5c5175cdf1eca9bc963b8e732eff02c39c4dd404c380a0285effbba20`; `proof-nextest.log` / `.exit` record 28 passed, exit 0 |
| static | `fmt.*`, `clippy.*`, `rustdoc.*`, `bash-n.*`, `shellcheck.*`, and `shfmt.*`; all qualified checks exit 0 |
| catalog | `plan-validator.*` and `rebundle.*`; 73 direct task packages, 77 recursive `verify.toml` files, 506 direct checks, 526 recursive checks, 1,174 source obligations, 3,256 traceability rows, 276 edges, depth 35; exits 0 |
| docs | `actionlint.*` and `lychee.*`; Lychee 0.24.2 reports 612 total, 604 successful, 8 excluded, 0 errors; exit 0 |
| expected gate failures | `preflight.log` / `.exit`: exit 1 because readiness is not exact GO; `boundary.log` / `.exit`: exit 1 because `parity/evidence.tsv` is missing |

These passing observations do not override the rejected reports or prove
acceptance. Their affected evidence is invalid after this docs-only commit;
the predetermined final seal must rerun and rebind what it uses.

The preceding `$RUN` table is historical evidence bound to the superseded
`a80c790e` / `d7654291` payload; it is not current exact-tree evidence.
Superseded pre-documentation paths remain useful only as historical provenance:
the old `27/27` and `27 passed` proof-nextest logs, proof build
`proof-preparation-final-2026-09-19` with binary SHA-256
`f85400a16dc131fe0f59bfc90b5ec22cadf97de02833a216ed81385bf70a1b44`,
`static-final-2026-09-19`, `shell-final-current-2026-09-19`,
`docs-final-current-retry-2026-09-19`, and
`taskfmt-lints-final-retry-2026-09-19` are not current evidence. Native Linux
evidence is unavailable. The current exact-tree workspace support result is:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/workspace-nextest-final-dd26-2026-09-19/result-summary.txt
SHA-256: 6971f496cfc91b097a5ec677ad128dc48e7d55e83aebb7ab09af1df8804c8143
binds: dd26d2e7cc4ebbf1d8f1af4c54e7081744549382 / ed1525875021c613fa9265239ad2f4835f9e1785 / parent a80c790e2ebba15e4a00e7cff4296eab2a5bccc5
result: cargo nextest 3342 passed (139 binaries, exit 0)
```

This is supporting Rust workspace evidence only, not a preparation receipt or
parity acceptance. `xtask boundary` still fails because `parity/evidence.tsv`
is missing.

## Task and architecture state

The generated catalog has 73 direct task packages (`001`–`073`) and 77
recursive `verify.toml` files, including four nested bootstrap fixtures. It has
506 direct checks (27 `argv`, 479 `shell`) and 526 recursive checks, 276
dependency edges, and maximum dependency depth 35. Plan validation reports
1,174 source obligations, 3,256 traceability rows, and zero structural errors.
All package lints passed; no production task is accepted.

- `TASK-001` and `TASK-070` are retired/non-qualifying fail-closed historical
  lifecycle tasks. They are not dispatchable implementation work.
- `TASK-071` and `TASK-072` remain blocked qualification prerequisites until
  fresh, independently reviewed receipts exist.
- `TASK-002`–`TASK-069` and `TASK-073` contain the remaining implementation,
  ownership, behavior, visual, performance, and closure obligations.

The target architecture is caller-owned state, borrowed props, read-only draw,
runtime-owned routing/focus/layers/pointer/cursor, and one reusable owner per
component family. The current product still has duplicate/compatibility
painters and missing application behavior. Examples are the Showcase grid and
chrome, Jackin projections, and TablePro tree/filter/frame painters. These are
future implementation obligations, not preparation changes.

## Evidence and blockers

The evidence index preserves raw paths, the latest exact-tree rejected
verifier/reviewer reports, calibration roots, and invalidation rules. The
latest exact-tree rejected reports are:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
SHA-256: a3390e74bd0dc85ca3e0ba62a62bb9c7b139851a952c4203781bde4c57462bc0
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
SHA-256: 57051451aabf2a31fa1de31cddb4ee3c27f24c281df41755d3846769011fc46d
```

They bind `dd26d2e7` / `ed152587` / parent `a80c790e` and return
**REJECTED / NO-GO**. They are rejected evidence only: they do not issue a
receipt or authorize GO. The a80/d765 reports are historical and superseded;
this docs-only repair creates a new tree, so the predetermined final seal must
bind that post-commit tree. No accepted receipt exists.

The tracked
[`current-preparation-2026-09-18.md`](evidence/current-preparation-2026-09-18.md)
is explicitly superseded by this 2026-09-19 index. It remains unchanged as
historical provenance, including its original bindings and raw evidence root;
its observations do not authorize the current tree.

Preparation blockers remain: clean post-refresh verifier/reviewer
sealing; complete native proof/result/receipt qualification; path-dependent
HTML provenance for the exact-tag calibration; a clean deterministic
892-source calibration; the missing `parity/evidence.tsv`; full
visual/behavioral/product parity; unavailable native Linux; absent
`TASK-071`/`TASK-072` dependency receipts; and no accepted preparation
receipt. The upstream Docker-only taskfmt path is policy-excluded and is not a
campaign blocker; it was not run and is not claimed as qualified. The 892 control is 298/302
passed with 4 failures, 2 skipped, 1 leaky, exit 100, 7,550
ANSI/plain/PNG/HTML artifacts each (30,200 total), and 17 diff files. This is
coverage evidence, not a calibration pass. The final preparation remains
NO-GO, the refreshed ledger is disarmed, and the next goal remains **NOT
AUTHORIZED FOR EXECUTION**.

**Decision: NO-GO.**
