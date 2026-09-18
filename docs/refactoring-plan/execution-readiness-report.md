# Terminal Components Refactor — Execution-Readiness Report

**Current authority:** This is the sole current readiness report (2026-09-18).
It is a preparation audit, not execution authorization.

## A. Current-state verdict

**NO-GO.**

`refactor/holla-parity` remains campaign/proof scaffolding, not completed
refactor work. This report is reconciled against source HEAD
`f5013f609aed1ba32ce60352b38fd0b1b11b063c` (tree
`e08a965702a4674e9e61970fb1c3923e05e1dec7`) observed before this docs-only
commit. The bounded raw outcomes are indexed in
[`evidence/current-preparation-2026-09-18.md`](evidence/current-preparation-2026-09-18.md).
No product parity or execution authorization follows from this reconciliation.

- Audit snapshot `main..a34a1cff`: 27 commits, zero apps/ or crates/ product-source changes.
- The reconciled branch remains campaign/proof scaffolding; production
  implementation behavior remains unchanged. `apps/` and `crates/` differ from
  `main` in four Rust files: two performance-test documentation command
  examples migrated from `cargo test` to `cargo nextest`, plus two comment-only
  removals of retired-document references. The reconciliation adds only docs,
  task contracts, scripts, and verification tooling; no product parity claim
  follows from these changes.
- All 73 task manifests remain pending.
- Shared architecture is substantial, but consumers still contain compatibility painters, duplicate state, and historical renderers.
- Frozen visual oracle is absent from this branch and its active gate; it
  remains available at the policy-protected `visual-baseline` tag.
- Planning validator passes: all 211 frozen bootstrap asset bindings are present and hash-valid.
- The worktree was not clean during this evidence collection. Preserved,
  out-of-scope changes were present in `baseline/before/MANIFEST.md`,
  `tools/refactor-proof/Cargo.toml`, and `tools/refactor-proof/src/lib.rs`.
  This documentation task did not stage or modify them. No clean-candidate
  proof is claimed.

`CLAUDE.md -> AGENTS.md` is correct. The current peeled `visual-baseline` tag
remains `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; the local tag pointer and
remote pointer are unchanged.
The tag is unsigned, GitHub reports the release as `immutable: false`, and the
branch has no provider protection, so immutability is currently a repository
policy boundary, not a provider-enforced guarantee. No mutation is authorized.

Validation:

- Latest taskfmt `0.2.0` at `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`:
  73/73 numbered packages linted successfully.
- Plan validator: passes with `error_count: 0`; the exact reviewed
  `main-source.tar.gz` projection is present at SHA-256
  `ab9568812e86e3c8e18c1d945886d4b2387b1e175ad61ba4d60750b5d6f98119`.
- Pre-arm preflight exits `1` at ledger validation with `no current accepted
  verifier receipt exists`; the tag, branch, and worktree checks pass first.
- Current focused `NEXTEST_USER_CONFIG_FILE=none cargo nextest run --locked
  -p refactor-proof` exits `102` before tests because the preserved dirty
  proof dependency change would require a `Cargo.lock` update. An earlier
  same-day diagnostic reported `3 passed (2 binaries, 0.021s)` before those
  dirty proof changes appeared; it has no clean-tree receipt and is not an
  accepted gate.
- Baseline calibration has only a passing control. The full `7,550`-key /
  `30,200`-artifact replay is pending; the tag nextest parser rejects its
  `binary(visual_baseline)` override, so the replay requires an external
  corrected config. No calibration acceptance is claimed.

## B. Branch reconciliation

| Ref | Commit |
|---|---|
| main | 7b27732a |
| refactor/holla-parity | `f5013f609aed1ba32ce60352b38fd0b1b11b063c` (tree `e08a965702a4674e9e61970fb1c3923e05e1dec7`) |
| visual-baseline branch | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| visual-baseline tag peel | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |

Relationships:

- merge-base(main, refactor) = 7b27732a; the cleanup parent was main + 34
  history rooted there. Exact ahead-count is intentionally not used as a
  readiness claim; inspect the final commit graph at execution start.
- merge-base(visual-baseline, refactor) = cc14dd6b.
- The branches retain independent history; recompute exact absent-commit
  counts at execution start rather than treating a historical count as
  readiness evidence.
- Baseline and refactor were never reconciled.

Missing from the current branch:

- snapshots/: 30,200 frozen artifacts.
- tests/visual_baseline/.
- .config/nextest.toml.
- docs/baseline/.
- Frozen PTY visual suite.
- Final baseline commits including grouped-store migration and visual stabilization.

Do not merge baseline history wholesale. It contains the old source layout. Port only the oracle suite/config/store access needed by the current architecture.

## C. Refactoring-task audit

Status means current-candidate evidence, not ledger claims.

| Tasks | Status | Reason |
|---|---|---|
| 001 | Retired/non-qualifying | The host-lifecycle path is removed. Comparator smoke coverage remains, but fail-closed gates prevent acceptance until a subagent-only proof contract is independently implemented and reviewed. |
| 070 | Retired/non-qualifying | The former taskfmt-init host-bootstrap check is retired and fail-closed; source-runner work has no accepted subagent evidence. |
| 071 | Pending/blocked | Implementation exists, but no accepted verifier-subagent evidence exists. |
| 072 | Pending/blocked | Implementation exists, but dependency receipt and accepted verifier-subagent evidence are absent. |
| 002–005 | Blocked | Frozen oracle and qualified foundation unavailable. |
| 006 | Blocked | Complete baseline cannot be sealed without oracle captures. |
| 007 | Blocked | Historical/oracle identity reconciliation lacks active oracle authority. |
| 008 | Blocked | Depends on failed baseline/identity prerequisites. |
| 009–031 | Pending | Shared component/runtime work not executed and accepted. |
| 032–039 | Pending | Showcase work not executed against frozen oracle. |
| 040–050 | Pending | Holla work not executed against frozen oracle. |
| 051–057 | Pending | Jackin work not executed against frozen oracle. |
| 058–064 | Pending | TablePro work not executed against frozen oracle. |
| 065–069 | Pending | Closure gates have no completed upstream evidence. |
| 073 | Blocked | Depends on 008/072; generated registry authority is not qualified. |

No task is proven complete. “Retired/non-qualifying” describes removed
host-execution paths, not accepted task completion or dependency supersession.
The task definitions remain pending. All 73 `verify.toml` files now use either
repository-relative `WORKTREE` paths or exported external `$RUN_DIR` proof
paths; the validator rejects any reintroduction of legacy container namespaces
before dispatch.

The catalog itself is structurally healthy: the latest standalone taskfmt
`lint` passes all 73 numbered packages. Required CLAUDE.md symlinks are valid
repository policy.

## D. Dependency graph

Longest path, depth 35:

~~~text
001 → 070 → 071 → 072 → 002 → 006 → 008 → 073
→ 009 → 010 → 014 → 015 → 018 → 019 → 023 → 024
→ 027 → 028 → 030 → 031 → 040 → 041 → 042 → 043
→ 044 → 045 → 046 → 047 → 048 → 049 → 050
→ 065 → 066 → 068 → 069
~~~

Execution layers:

~~~text
L01  001
L02  070
L03  071
L04  072
L05  002 003 004 005 007
L06  006
L07  008
L08  073
L09  009 011 012
L10  010 013 029
L11  014
L12  015 016 017 021
L13  018 022 026
L14  019 020
L15  023
L16  024
L17  025 027
L18  028
L19  030
L20  031
L21  032 040 051 058
L22  033 041 052 059
L23  034 042 053 060
L24  035 043 054 061
L25  036 044 055 062
L26  037 045 056 063
L27  038 046 057 064
L28  039 047
L29  048
L30  049
L31  050
L32  065 067
L33  066
L34  068
L35  069
~~~

Parallelism is safe only in isolated worktrees. A single serial integrator must merge layer results and run gates.

## E. Visual-parity assessment

Current visual verification is insufficient.

Frozen oracle:

- 7,550 matrix keys.
- 30,200 files.
- Four artifacts per key: ANSI, text, PNG, HTML.
- Five sizes: 72x20, 80x24, 100x30, 120x40, 160x50.
- Five color modes: truecolor, 256, 16, none, nocolor.
- Real PTY interaction and settled-frame capture.

Current branch:

- 1,320 headless digest records.
- Static initial frames.
- No PTY.
- No PNG/HTML comparison.
- No terminal-parser or alternate-screen verification.
- No real NO_COLOR lane.
- Missing 72x20.
- No interaction-transition visual coverage.
- Cursor/focus/layer state is not in the digest.
- baseline/before/ is historical evidence, not an approved active oracle.

The frozen suite itself documents limitations: animation mid-frames, some Alt-key paths, legacy-only TablePro flows, and unstable Jackin outro text. Those need explicit behavioral tests or documented exclusions.

## F. Behavioral-parity assessment

Pixels do not prove:

- Focus owner and focus restoration.
- Cursor position and cursor visibility.
- Hit-region ownership.
- Hover, drag, wheel, and pointer routing.
- Active layer/modal ownership.
- Scroll offset across resize.
- Pending edits and stale-result invalidation.
- Completion/cancellation timing.
- Diagnostics absence.
- Jackin Container Info route.
- Jackin F10 last-menu behavior.
- Real PTY delivery of Ctrl+\.
- Holla pointer workflows.
- TablePro drawer reflow while state is active.
- App lifecycle and reconnect behavior.
- Strict performance thresholds.

Current tests cover much component behavior, but parity proof needs replayed state transitions, not only fresh-frame assertions.

## G. Scripts audit

| Script | Purpose / state | macOS, Grok, container | Recommendation |
|---|---|---|---|
| scripts/campaign-dispatch.sh | Per-task taskfmt lint/verify wrapper. | Runs only in a verifier subagent's host-local worktree and external run directory; requires an explicit full scope base, clean worktree, native build receipt, exact context set, and fresh logs. | Keep fail-closed; the missing per-check launcher remains a blocker. |
| scripts/campaign-init.sh | Branch/worktree/ledger bootstrap. | Must never move an existing ref or force-checkout a worktree. | Keep only after non-forcing guard repair. |
| scripts/campaign-install-taskfmt.sh | Installs latest taskfmt from the local checkout. | Exact source HEAD, version, and pinned executable SHA-256 are checked; preflight separately lints all 73 packages. | Keep. |
| scripts/campaign-absorb-planning.sh | Copies planning worktree and stages changes. | Obsolete branch-copy workflow. | Retire. |
| scripts/campaign-preflight.sh | Pre-arm checks. | Fails closed; uses latest taskfmt per-task lint, rejects legacy container paths, and validates the native comparator receipt against worktree HEAD. | Keep; ledger, oracle, context, and accepted-receipt blockers remain. |
| Retired container-path helper names | No such scripts are present in the current tree; old reports may still cite them. | Historical only. | Do not restore or invoke. |
| tools/refactor-proof/scripts/dev-tc-proof-host.sh | Removed host-lifecycle wrapper. | No current path; historical references are non-executable. | Do not restore or invoke. |
| tools/refactor-proof/scripts/dev-tc-proof.sh | Proof helper for verifier-owned checks. | Host-local and subagent-scoped. | Keep only if a verifier task requires it. |
| tools/refactor-proof/scripts/sync-binaries.sh | Legacy tracked-binary synchronizer. | Explicit fail-closed retirement; it refuses to overwrite the Python dispatcher. | Do not use for campaign authority. |
| tools/refactor-proof/scripts/test_freeze_vectors.py | Removed host-lifecycle vector tests. | No current path; historical references are non-executable. | Do not restore or invoke. |
| tools/refactor-proof/scripts/test_integrate_seal_vectors.py | Removed host-lifecycle vector tests. | No current path; historical references are non-executable. | Do not restore or invoke. |
| docs/refactoring-plan/evidence/validate-plan.py | Read-only catalog/DAG/traceability validator. | macOS-compatible; no container; parses the current shell/argv proof path contract. | Keep; make preflight fail closed. |

`campaign-build-proof.sh` is a separate host-local preparation helper. It runs
locked offline Cargo build for the standalone native `tc-proof` comparator and
writes a commit/path/hash receipt; taskfmt does not build it. The current
dispatcher validates the receipt and context filenames, but no trusted helper
yet materializes the per-check context/index/result/observer ABI.

No script directly runs `tuisnap accept`. All Bash scripts pass syntax checks.

## H. macOS execution prerequisites

Required:

- macOS with Darwin Seatbelt support.
- Xcode Command Line Tools: xcode-select, xcrun, clang, and SDK.
- Rust/Cargo and locked dependencies.
- cargo-nextest.
- Latest taskfmt revision:

~~~text
afd3b575dbcc7044620bec4b9493a74eca3e5ef2
~~~

- Latest taskfmt version: `0.2.0`.
- Latest taskfmt executable SHA-256:
  `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`.

- Python 3, Bash, Git, rg, file, and grep.
- tuisnap and the frozen grouped store.
- External per-run temp/build directories with sufficient disk.
- Isolated worktrees.
- No Docker or container runtime.

The active GitHub CI and performance workflows run on `ubuntu-latest`. Those
Linux gates are supplementary and cannot substitute for the required native
macOS proof, PTY execution, or frozen visual comparison.

All current task packages now contain host-local `WORKTREE` paths or exported
external `$RUN_DIR` proof paths. The validator and preflight reject legacy
`/task`, `/work`, `/proof`, and `/run` namespaces. Do not create mounts or
firmlinks to preserve the old contract. Proof checks use exported
`$RUN_DIR/contexts/CHK-NNN.json`; their
contexts, result paths, observer transport, and context index still need a
  trusted verifier-owned materializer. The native comparator must be built by
  the host-local helper and bound to its commit/path/hash receipt.

## I. Subagent execution architecture

One coordinator should own:

1. Immutable ref/tag verification.
2. Host preflight.
3. DAG scheduling.
4. Subagent assignment and worktree isolation.
5. Per-task taskfmt lint/verify evidence collection.
6. Serial integration.
7. Final gates.

Each task gets:

- One implementer in an isolated worktree.
- One focused test/behavior verifier.
- One independent review agent.
- A verifier subagent running taskfmt lint/verify.
- A reviewer subagent checking evidence and frozen-oracle reads.

Agents must not:

- Mutate visual-baseline.
- Run baseline acceptance/blessing.
- Promote through taskfmt lifecycle commands.
- Modify main.
- Share writable build/snapshot directories.

Only the serial coordinator may merge reviewed task commits. No host lifecycle
service or taskfmt promotion path exists. The verifier subagent must perform
native proof preparation before taskfmt: build the standalone comparator,
materialize immutable per-check contexts and the context index, bind observer
and result capabilities, then invoke only taskfmt's per-task commands.

## J. taskfmt strategy

Use only the latest standalone binary from
`/Users/donbeave/Projects/taskfmt/task-format`, source revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, version `0.2.0`. The current
binary identity is checked by exact `--version` output and executable SHA-256.

Allowed:

~~~text
TASKFMT=/absolute/path/to/qualified/taskfmt
"$TASKFMT" lint /absolute/catalog/terminal-components/completion/NNN
"$TASKFMT" verify --root /absolute/subagent-worktree \
  --task-dir /absolute/catalog/terminal-components/completion/NNN \
  --base "$SCOPE_BASE" --progress "" --log-dir "$RUN_DIR/taskfmt-logs"
~~~

The latest standalone lint passes all 73 numbered packages. This campaign uses
only per-task `lint` and `verify`; it does not invoke taskfmt progress,
runtime, host, lifecycle, dispatch, or promotion commands.

Do not use taskfmt lifecycle operations that create/reset workspaces or
promote refs. They are outside the latest standalone command surface and are
incompatible with this campaign.

`--root` and `--task-dir` do not rewrite paths. Every task `verify.toml` must
use host-local relative paths or explicit subagent paths; legacy `/task`,
`/work`, `/proof`, and `/run` entries are migration blockers.

The latest taskfmt executes `argv` from the supplied root and does not
interpolate `$RUN_DIR`; shell checks are run by bash. The dispatcher exports
`RUN_DIR` and validates filenames/run identity, but taskfmt itself validates
neither proof ABI nor result provenance. This is why a passing package lint is
not a ready verification run.

## K. Verification matrix

| Workstream | Tests | Visual proof | Behavioral proof | Gate |
|---|---|---|---|---|
| 001, 070–073 | Proof-vector nextest, subagent evidence, architecture probes | Frozen store integrity only | Ref identity, source binding, accounting, architecture ownership | Reviewed subagent evidence + latest taskfmt lint/verify |
| 002–008 | Capture/source identity and protected-authority tests | All 7,550 frozen keys; four artifacts each | PTY startup, dimensions, colors, capture provenance | Exact inventory; no candidate blessing |
| 009–031 | Component conformance, unit, architecture, compile-fail tests | Affected app families across 5 sizes × 5 colors | Focus, cursor, layers, hit regions, resize, editing, scrolling | Component tests + frozen comparisons |
| 032–039 | Showcase app/elapsed/perf tests | Showcase audit/fade/flows/hover/pages/resize | All 23 routes, dialogs, pickers, editor, progress, focus | Complete Showcase oracle |
| 040–050 | Holla app/domain/perf tests | Holla audit/concept/fade/flows/parity/resize | Finder, trust, arguments, activity, cancellation, cleanup, remote outcomes | Complete 34-world oracle |
| 051–057 | Jackin app/timing/manager/perf tests | Jackin accounts/audit/capsule/cockpit/editor/intro/manager/scenarios/settings/usage | Menus, Container Info, F10, PTY control delivery, reconnect, stale completion | Complete Jackin oracle |
| 058–064 | TablePro app/close/undo/quick-switcher/perf tests | TablePro ack/audit/connections/fade/overlays/query/resize/table/workbench | Safety gates, dirty state, drawer reflow, row identity, undo | Complete TablePro oracle |
| 065–069 | Architecture, API/docs, strict perf, full nextest, ancestry | Full frozen suite plus store integrity | No duplicate painters, exact test preservation, merge tree | Clean worktree and final receipt |

All Rust gates must use cargo nextest.

## L. Risks and blockers

Hard blockers:

- Frozen oracle absent from current branch and active gate.
- visual_baseline binary unavailable on HEAD.
- test(store_integrity) has no active test.
- `.campaign/ledger.json` is disarmed; no accepted current subagent
  verification evidence exists.
- `campaign-preflight.sh` previously looked for schema-invalid
  `status=accepted`/`verifier_verdict=PASS` rows. Its acceptance predicate is
  now aligned with the schema's `verified`/`VERIFIED` vocabulary; the current
  ledger remains blocked and no receipt is being manufactured by this fix.
  `scripts/test_campaign_ledger.py` proves the accepted and rejected row
  combinations deterministically.
- No accepted current verifier-subagent evidence exists for any task.
- The migrated task checks are not dispatch-ready until fresh verifier runs
  prove their referenced host-local inputs and outputs.
- No trusted per-check native launcher/materializer currently binds
  `TC_PROOF_CONTEXT_SHA256`, run/task/check IDs, source tree, oracle commit,
  result path, observer FDs/nonce, and context-index identity end-to-end.
  The dispatcher currently checks the exact context filenames, JSON object
  shape, and common `run_id`; that is necessary but not sufficient.
- The standalone native comparator is now fail-closed behind
  `campaign-build-proof.sh` and a commit/path/hash receipt. A local ignored
  binary/receipt exists, but no accepted verifier receipt exists for the
  current tree or any current task.
- Ledger candidate tree hash `4d3501a6` is not current HEAD; no reviewed
  subagent evidence binds the current branch.
- Current proof-worker identity is not yet bound to a reviewed subagent run.
- The observed out-of-scope dirty files are intentionally excluded from this
  documentation commit: `baseline/before/MANIFEST.md`,
  `tools/refactor-proof/Cargo.toml`, and `tools/refactor-proof/src/lib.rs`.
- The full baseline calibration remains pending. Its control passed, but the
  tag nextest parser rejected the `binary(visual_baseline)` override; an
  external corrected config is required before the full replay.
- Recorded remote performance CI for the pre-cleanup candidate failed three
  Jackin allocation-budget tests; no fresh performance qualification was run
  by this documentation/tooling cleanup.
- CI and app perf command generation now use `cargo nextest`; this cleanup did
  not add the missing frozen visual gate.
- `cargo-nextest 0.9.143` has no doctest runner. Rustdoc compilation remains
  available in CI, but executable doctest coverage is an explicit TASK-066
  blocker; no invalid `nextest --doc` or legacy test-runner fallback is
  permitted.
- Compatibility painters remain in Showcase, Jackin, Holla, and TablePro.
- Current generic-copy architecture checks miss dominant duplicate-paint patterns.
- Historical readiness documents retain superseded verdicts, but the current
  authority and routing are now explicit in `docs/refactoring-plan/README.md`.
- Historical sandbox scripts are container-path dependent and are not a valid
  host qualification path.
- Normal Jackin allocation/performance budgets still fail in the recorded
  candidate evidence (capsule, manager, and key-movement allocations); the
  flag-placement fixes and nextest migration do not qualify those budgets.

## M. Final long-running /goal execution plan

This is a contingent plan, not an executable prompt. While this report is
**NO-GO**, perform only non-authorizing catalog, preflight, taskfmt, proof, and
documentation checks. Do not create task worktrees, spawn implementers, run
task-owned verification, or integrate task commits. A future **GO** verdict is
necessary but not sufficient: dependency receipts, verifier evidence, reviewer
approval, branch gates, and the complete visual/behavioral gates remain
mandatory.

### Preparation gate

1. Freeze current branch SHA/tree and preserve all observed out-of-scope dirty files as explicit inputs.
2. Reconcile .campaign ledger, stale receipts, disarmed state, and current HEAD.
3. Confirm the restored `main-source.tar.gz` and all 211 frozen planning asset bindings remain byte/hash exact.
4. Make the frozen suite/config/store available read-only from tag-derived bytes.
5. Add the grouped visual suite without importing old product architecture.
6. Confirm every task `verify.toml` remains host-local and legacy namespaces stay rejected.
7. Implement and independently test the trusted native per-check context/result
   launcher, observer binding, and context-index receipt in a clean host-local
   verifier worktree.
8. Spawn subagent implementer/verifier/reviewer lanes with disjoint worktrees
   and run directories.
9. Keep CI and hidden test generation on nextest-only commands.
10. Qualify the latest taskfmt identity and per-task lint/verify wrapper.
11. Run the current code against the frozen oracle. Record drift. Do not bless.

Stop if any preparation gate fails.

### Implementation waves

**Conditional future DAG only.** These waves are descriptive planning, not
dispatch authority. While this report is **NO-GO**, no wave or task may start.
Dispatch requires readiness/dependency preflight and the accepted verifier
receipts required by `AGENTS.md` and the current task contracts.

- Wave 1: 001 → 070 → 071 → 072.
- Wave 2: 002, 003, 004, 005, 007 in isolated worktrees.
- Wave 3: 006 → 008 → 073.
- Wave 4: component layers 009–031 according to the DAG.
- Wave 5: app groups in parallel:
  - Showcase 032–039
  - Holla 040–050
  - Jackin 051–057
  - TablePro 058–064
- Wave 6: 065 and 067 parallel.
- Wave 7: 066 → 068 → 069.

After every task:

- Run latest taskfmt lint/verify.
- Run focused nextest.
- Run behavior replay.
- Run affected frozen visual families.
- Record exact receipt.
- Integrate serially.
- Never bless or regenerate the frozen oracle.

Final gates:

- Full locked workspace nextest.
- Full frozen PTY visual suite.
- Store integrity.
- Strict app/library performance.
- Architecture and duplicate-painter scans.
- Documentation/API checks.
- Clean worktree.
- Frozen tag unchanged.
- No baseline/perf generated changes.
- Receipt-bound final integration tree.

## N. Go / no-go evidence for the next execution goal

Satisfied:

- Ref/tag identities are known.
- Current frozen tag peel is recorded and no mutation is authorized.
- DAG and task catalog exist.
- Latest taskfmt lint passes 73/73 numbered packages.
- Shared component architecture and broad behavior tests exist.
- macOS host has the required general toolchain.

Not satisfied:

- Frozen visual authority is unavailable on the current branch; the
  policy-protected tag remains the source to import read-only.
- Provider-enforced visual-baseline immutability is absent; local tag/release policy is the guard.
- Subagent verification evidence is absent.
- Campaign ledger is disarmed and has no accepted current task evidence.
- Proof tree is dirty.
- The current proof worker is not bound to a reviewed subagent run.
- Consumer migration is incomplete.
- Compatibility renderers remain.
- Behavior and performance parity are not fully proven.

Smallest preparation goal:

> Make refactor/holla-parity a clean, unarmed, subagent-executable, frozen-oracle-backed campaign branch. Do not modify production behavior or the frozen tag. Pass plan validation, latest per-task taskfmt lint/verify, independent subagent review, strict refactor-proof compilation, and the full nextest/visual preflight.

Until that goal passes, do not launch the autonomous 73-task implementation campaign.

## O. Next step after this cleanup

Implement and independently test the trusted native per-check launcher and
materializer end-to-end: exact contexts/index, task/check/run/source/oracle
binding, observer transport, result ABI, and commit/hash receipts. Then make a
read-only import of the frozen suite, configuration, and grouped oracle store
from `refs/tags/visual-baseline`. Before any full replay, supply an external
corrected nextest config for the tag parser's rejected binary override. Do not
dispatch a refactoring task yet. The
execution workflow remains **Grok Build → isolated subagents → implementation
→ latest standalone taskfmt `lint`/`verify` → visual/behavioral gates → serial
integration**, with no containers and no taskfmt orchestration.

## P. Independent-feedback reconciliation

The requested `another-agent-feedback/**` tree is absent. Independent search
covered this worktree, its parent and sibling worktrees, the project tree,
ignored/untracked paths, all reachable refs, stash/reflog data, and unreachable
Git objects; the inventory is zero files. Therefore no external report finding
could be accepted, rejected, or superseded, and no external file was
overwritten. The findings below are the independent audits requested for the
same topics, reconciled against the current source and history.

The current audit nevertheless reconciled the requested high-risk topics:

| Topic | Disposition | Current evidence |
|---|---|---|
| Branch/baseline relationship | Integrated into this report | The reconciliation records the current branch relationship; prior cleanup ancestor is `c9eef7bd`; `main` is `7b27732a`; frozen tag peel is `4a79c0a2`; 30,200 frozen files and the PTY suite remain absent from HEAD. |
| Missing planning archive | Fixed from reviewed bytes | `docs/refactoring-plan/evidence/main-source.tar.gz` now matches the tracked TASK-072 trusted source and manifest hash; validator and all seven asset groups pass. |
| Latest taskfmt | Already correct; revalidated | Local source is clean at `afd3b575`; version `0.2.0`; executable hash matches; 73/73 lints pass. |
| Containers/taskfmt orchestration | Already fixed and retained | Active scripts and task contracts allow only standalone per-task `lint`/`verify`; historical host/container fixtures are non-authoritative. |
| Snapshot and behavioral parity | Still valid blockers | Static/current self-baselines do not replace the 30,200-file grouped store, PTY transitions, cursor/focus/hit ownership, or real component routes. Compatibility painters remain an ownership risk. |
| CI and performance | Integrated, blocker retained | Active CI/perf commands use nextest with corrected thread-flag placement; rustdoc remains a compile gate because nextest 0.9.143 has no doctest runner. Recorded normal Jackin allocation budgets still fail for capsule, manager, and key movement; no fresh qualification was claimed. |
| Stale goals/reports/runbooks | Integrated | `GOAL.md` is product intent only; obsolete root goals, handoffs, state/coordination files, superseded plans, reports, prompts, and runbooks were removed from this branch. Git history is recovery-only. Remaining source-bound historical evidence is non-authoritative and is retained only where current task contracts or machine ledgers require it. Current docs route through this report and the subagent-only contracts. |
| Documentation links | Integrated and requalified | Lychee 0.24.2 and the native semantic/path audit cover every tracked Markdown-formatted input; current results and the one narrow mail-data setting are recorded in §Q. |
| Proof compile/dispatcher consistency | Partially integrated; blocker retained | The plural `checks` parser, exact context-file set, external run directory, explicit scope base, clean worktree, and native binary receipt checks are now fail-closed. A native build helper records commit/path/hash. No trusted per-check context/result/observer launcher exists yet. |
| Scripts and package rebundling | Integrated | Obsolete container/taskfmt lifecycle paths are not execution authority; alternate Python rebundlers now fail closed instead of overwriting the dispatcher. Useful nextest/inventory checks remain. |
| DAG and task acceptance | Integrated; acceptance still absent | The current graph is acyclic and regenerated from current metadata; stale graph/title/context bindings were reconciled. Task metadata remains `pending`; only schema-valid `verified`/`VERIFIED` verifier receipts and integrated ancestry can establish completion. |
| macOS-native execution | Partially integrated; blocker retained | Native host build and receipt tooling is present, with no container path. Context provisioning, observer transport, result ABI, and the frozen-oracle read-only import still require an independently tested verifier launcher. |
| Visual-baseline immutability | Integrated as policy | Local/remote tag pointers are unchanged. Provider enforcement is not assumed because the tag is unsigned, release immutability is false, and branch protection is absent. |

The visual-baseline row records the current policy result; provider
enforcement is deliberately not assumed. Remaining historical reports that
mention old taskfmt pins, old oracle tags, old commands, or old execution
authorities remain source evidence only and are routed through the current
README. Retired plans, prompts, runbooks, and duplicate readiness reports were
removed rather than rewritten into false current results.

## Q. Documentation and link-integrity audit

This documentation pass was completed against the current tree on 2026-09-18.
The staged reconciliation tree contains **746 tracked Markdown-formatted
inputs**. The exact native Lychee `0.24.2` run over that input set completed
with 611 destinations: 603 successful, eight narrow mail-data exclusions, and
zero redirects, errors, timeouts, unknowns, or unsupported destinations. The
new evidence index is included in that checked input set.

The complete inventory contains **746 tracked Markdown-formatted inputs** (`.md`, `.mkd`,
`.mdx`, `.mdown`, `.mdwn`, `.mkdn`, `.mkdown`, `.markdown`, and `.mdc`),
including hidden, task, historical, and contributor documents. Lychee
`0.24.2` parsed 607 destinations in the native offline qualification run:
513 successes, 94 network exclusions, zero errors, zero timeouts, zero
unknowns, and zero unsupported destinations. The local resolver found no
missing repository file, directory, or fragment destination. The same complete
input set passed a fresh live Lychee run with `--cache=false`: 599 successful
destinations, four redirect occurrences, eight narrow mail-data exclusions,
and zero errors, timeouts, unknowns, or unsupported destinations. Cache-enabled
runs may report zero redirects because successful responses are reused as
`200` results; the cache does not weaken local or fragment checks.

The offline run necessarily did not exercise network destinations; its
exclusions are execution-mode output, not a repository allowlist. The live run
exercised the checkable external HTTP/HTTPS destinations and completed without
an error. CI retains the same live Lychee checks with no URL/path exclusions.

The only intentional Lychee category setting is `include_mail = false`: email
values in campaign metadata and reserved `example.test` fixtures are not
documentation destinations. There is no `.lycheeignore`, no broad URL/path
ignore, no private-link suppression, and no accepted-status wildcard. Cache
reuse is bounded to successful external results for one day; local paths and
fragments are resolved from the checkout. CI's cache key includes the Lychee
version, action version, arguments, configuration hash, runner, and commit.

The semantic audit reconciled the deleted `docs/sources/PLANNING_GOAL.md`
reference to an immutable `visual-baseline` commit URL in active task docs;
historical mentions are explicitly provenance-only. It corrected the Holla
stage contract's future target paths (`domain/cleanup.rs` and `sim/fs.rs`)
without falsely claiming those files exist today; current path normalization is
`apps/holla/src/domain/disk.rs`. It corrected the refactoring-plan authority
pointer, demoted stale plan/report findings, and reviewed the duplicated task
contracts. The architecture appendix's former state-ledger mirror requirements
and continuation-prompt references are explicitly historical provenance, not
active authority; exact ledger evidence remains recoverable from Git at the
cited commit. Current execution status routes through this readiness report,
the current task graph, task contracts, and `AGENTS.md`. The architecture
appendix's invalid `cargo nextest --doc` examples were also replaced with the
qualified `cargo build`/`cargo doc` workflow. Retained audit and review records
keep their original commands only as historical evidence and explicitly do not
authorize replay. No obsolete document was restored and no placeholder was
created.

The independent review also found legacy `taskfmt init` probes in the retained
host-bootstrap fixture copies for TASK-001/070/071/072. History review
classified them as archival qualification evidence: TASK-001/070 are
non-qualifying, and no current `verify.toml` invokes the host-bootstrap
lifecycle fixture. Current task instructions explicitly prohibit lifecycle
commands; those fixtures must not be executed as campaign infrastructure and
remain a replacement-work blocker, not an accepted taskfmt workflow.

A separate plain-text/path-reference audit found no missing current repository
destination. References to historical or future paths remain only where their
provenance or task target meaning is explicit; they are not treated as current
filesystem claims. `AGENTS.md` makes this semantic review rule mandatory.
The same audit corrected `completion/007`'s false absence claim and
`completion/037`'s Showcase test paths. It also reconciled the branch override
contract: `campaign-preflight.sh` now validates the configured integration
branch, and `campaign-init.sh` passes its configured worktree/branch into the
printed preflight command.

CI enforcement is the blocking `Markdown links` job in
[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml), triggered on push,
pull request, manual dispatch, and a weekly schedule. It enumerates the same
tracked extension set, uses the pinned Lychee action `v2.9.0` with Lychee
`0.24.2`, and reports source/destination diagnostics. `actionlint 1.7.12`
passes locally. The authoritative workflow remains **Grok Build → subagents →
implementation → taskfmt deterministic validation → visual/behavioral gates →
integration**; taskfmt and Lychee are verifiers, never orchestrators, and no
container path is part of this campaign.

All existing workflow actions are now full-SHA pinned; the pin updates are
security-hardening changes, not a relaxation of the link gate.
