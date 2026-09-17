# Terminal Components Refactor — Execution-Readiness Report

**Current authority:** This is the sole current readiness report (2026-09-18).
It is a preparation audit, not execution authorization.

## A. Current-state verdict

**NO-GO.**

`refactor/holla-parity` remains campaign/proof scaffolding, not completed
refactor work. The pre-cleanup audit snapshot was `a34a1cff`; this cleanup
changes documentation/tooling contracts only and does not claim product
parity.

- Audit snapshot `main..a34a1cff`: 27 commits, zero apps/ or crates/ product-source changes.
- All 73 task manifests remain pending.
- Shared architecture is substantial, but consumers still contain compatibility painters, duplicate state, and historical renderers.
- Frozen visual oracle is absent.
- Planning validator fails.
- Current worktree is dirty:
  - tools/refactor-proof/architecture/source.py
  - tools/refactor-proof/bin/tc-proof

`CLAUDE.md -> AGENTS.md` is correct. The current peeled `visual-baseline` tag
remains `4a79c0a2`; its tag, release, and oracle store are immutable.

Validation:

- Latest taskfmt `0.2.0` at `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`:
  73/73 numbered packages linted successfully.
- Plan validator: fails with three missing main-source.tar.gz errors.
- Workspace nextest: **3,350 passed**, 154 binaries, 761.956 seconds.
- Strict `RUSTFLAGS=-Dwarnings` refactor-proof nextest: **17 passed**.
- The known slowest test remains the macOS capture-build qualification; the
  fresh workspace run completed in 761.956 seconds.

## B. Branch reconciliation (pre-cleanup audit snapshot)

| Ref | Commit |
|---|---|
| main | 7b27732a |
| refactor/holla-parity | a34a1cff |
| visual-baseline branch | 4a79c0a2 |
| visual-baseline tag peel | 4a79c0a2 |

Relationships:

- merge-base(main, refactor) = 7b27732a; refactor is main + 27 commits.
- merge-base(visual-baseline, refactor) = cc14dd6b.
- Baseline has 77 commits absent from refactor.
- Refactor has 801 commits absent from baseline.
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
| 001 | Partially complete | Proof/host code exists, but current tracked binary is not receipt-bound. |
| 070 | Partially complete | Source-runner implementation exists; host qualification and receipt integrity are incomplete. |
| 071 | Partially complete | Implementation exists; recorded verify exits 1 and requires /run firmlink. |
| 072 | Partially complete | Implementation exists; dependency receipt is TBD; no current accepted host proof. |
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

No task is proven complete. No task is proven obsolete or superseded. No task definition is inherently invalid, but the host/container path contract requires correction for autonomous macOS execution.

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
| scripts/campaign-dispatch.sh | Host status/prepare/freeze/verify/integrate wrapper. | Status is available; mutating commands require an armed ledger and a synced Mach-O host binary. | Keep fail-closed. |
| scripts/campaign-init.sh | Branch/worktree/ledger bootstrap. | Must never move an existing ref or force-checkout a worktree. | Keep only after non-forcing guard repair. |
| scripts/campaign-install-taskfmt.sh | Installs latest taskfmt from the local checkout. | Exact source HEAD, version, executable hash, and numeric-package lint are checked. | Keep. |
| scripts/campaign-absorb-planning.sh | Copies planning worktree and stages changes. | Obsolete branch-copy workflow. | Retire. |
| scripts/campaign-preflight.sh | Pre-arm checks. | Fails closed; uses latest taskfmt and `cargo nextest` discovery. | Keep; current plan validator remains a blocker. |
| scripts/fix-catalog-container-paths.py | Mass-rewrites literal container paths. | Unsafe broad mutation; not part of current host workflow. | Retire. |
| scripts/hybrid-verify-sandbox.sh | Synthetic `/task`, `/work`, `/proof` sandbox. | Requires root firmlinks; not macOS qualification. | Retire. |
| scripts/task-001-verify-sandbox.sh | TASK-001 container-path staging. | Same root/sudo blocker; not autonomous. | Retire. |
| tools/refactor-proof/scripts/dev-tc-proof-host.sh | Runs host proof binary. | macOS-compatible; no container. | Keep, simplify paths. |
| tools/refactor-proof/scripts/dev-tc-proof.sh | Runs proof binary. | macOS-compatible; no container. | Keep or merge with host wrapper. |
| tools/refactor-proof/scripts/sync-binaries.sh | Copies binaries into tracked tree. | Darwin-specific; no container. | Rewrite to external per-run artifacts. |
| tools/refactor-proof/scripts/test_freeze_vectors.py | Temporary-repository freeze tests. | macOS-compatible; no container. | Keep; parameterize paths. |
| tools/refactor-proof/scripts/test_integrate_seal_vectors.py | Temporary-repository seal/integrate tests. | macOS-compatible; no container. | Keep; parameterize paths. |
| docs/refactoring-plan/evidence/validate-plan.py | Read-only catalog/DAG/traceability validator. | macOS-compatible; no container. | Keep; make preflight fail closed. |

No script directly runs tuisnap accept. All Bash scripts pass syntax checks.

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

Current scripts additionally require root-created synthetic firmlinks such as /run; Grok cannot provision those autonomously. Rewrite the path adapter before execution.

## I. Grok Build execution architecture

One /goal coordinator should own:

1. Immutable ref/tag verification.
2. Host preflight.
3. DAG scheduling.
4. Taskfmt invocation.
5. Serial integration.
6. Receipt collection.
7. Final gates.

Each task gets:

- One implementer in an isolated worktree.
- One focused test/behavior verifier.
- One independent review agent.
- Host-owned taskfmt verification.
- Host-owned frozen-oracle comparison.

Agents must not:

- Mutate visual-baseline.
- Run baseline acceptance/blessing.
- Promote through taskfmt lifecycle commands.
- Modify main.
- Share writable build/snapshot directories.

Only the serial integrator may merge task commits and update campaign receipts.

## J. taskfmt strategy

Use only the latest standalone binary from
`/Users/donbeave/Projects/taskfmt/task-format`, source revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, version `0.2.0`. The current
binary identity is checked by exact `--version` output and executable SHA-256.

Allowed:

~~~text
taskfmt lint refactoring-tasks/terminal-components/completion/[0-9][0-9][0-9]
taskfmt init --task-dir /absolute/task --out /absolute/run/progress.md
taskfmt verify --root /absolute/work --task-dir /absolute/task \
  --base "$SCOPE_BASE" --progress "$PROGRESS" --log-dir "$LOG_DIR"
taskfmt-host lint --json
~~~

The latest standalone lint passes all 73 numbered packages. The latest
standalone CLI has no `project`, `group`, `fingerprint`, `progress-init`, or
`--config` interface. Older commands and receipts are historical only.

Do not use taskfmt lifecycle operations that create/reset workspaces or
promote refs. They are outside the latest standalone command surface and are
incompatible with this campaign.

`--root` and `--task-dir` do not rewrite literal `/task`, `/work`, `/proof`, or
`/run` paths. A host-owned adapter must resolve these paths into isolated local
directories while preserving trust boundaries.

## K. Verification matrix

| Workstream | Tests | Visual proof | Behavioral proof | Gate |
|---|---|---|---|---|
| 001, 070–073 | Proof-vector nextest, host receipts, architecture probes | Frozen store integrity only | Ref identity, sealing, source binding, accounting, architecture ownership | Host receipt + latest taskfmt verify |
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

- Frozen oracle absent from current branch.
- visual_baseline binary unavailable on HEAD.
- test(store_integrity) has no active test.
- Plan validator fails on missing docs/refactoring-plan/evidence/main-source.tar.gz.
- .campaign/ledger.json is disarmed; no accepted current host receipt exists.
- TASK-071 verify record says exit 1, scope failure, and missing /run firmlink.
- TASK-072 dependency receipt is TBD.
- Ledger candidate tree hash `4d3501a6` is not current HEAD; no accepted host
  receipt binds the current branch.
- Current proof binary hash differs from the recorded receipt.
- Current worktree is dirty.
- Recorded remote performance CI for the pre-cleanup candidate failed three
  Jackin allocation-budget tests; no fresh performance qualification was run
  by this documentation/tooling cleanup.
- CI and app perf command generation now use `cargo nextest`; this cleanup did
  not add the missing frozen visual gate.
- Compatibility painters remain in Showcase, Jackin, Holla, and TablePro.
- Current generic-copy architecture checks miss dominant duplicate-paint patterns.
- Historical readiness documents retain superseded verdicts, but the current
  authority and routing are now explicit in `docs/refactoring-plan/README.md`.
- Historical sandbox scripts are container-path dependent and are not a valid
  host qualification path.

## M. Final long-running /goal execution plan

### Preparation gate

1. Freeze current branch SHA and preserve the two dirty proof files as explicit inputs.
2. Reconcile .campaign ledger, stale receipts, disarmed state, and current HEAD.
3. Recover the exact reviewed main-source.tar.gz asset or establish a separately verified equivalent.
4. Make the frozen suite/config/store available read-only from tag-derived bytes.
5. Add the grouped visual suite without importing old product architecture.
6. Rewrite host path handling; remove /run and /task root-firmlink dependence.
7. Rewrite preflight/dispatch fail-closed.
8. Keep CI and hidden test generation on nextest-only commands.
9. Qualify the host verifier and latest taskfmt identity.
10. Run the current code against the frozen oracle. Record drift. Do not bless.

Stop if any preparation gate fails.

### Implementation waves

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

- Frozen visual authority is unavailable.
- Plan validator is red.
- Host proof receipts are stale/failed.
- Campaign ledger is disarmed and has no accepted current host receipt.
- Proof tree is dirty.
- The current proof binary is not receipt-bound to an accepted host run.
- Consumer migration is incomplete.
- Compatibility renderers remain.
- Behavior and performance parity are not fully proven.

Smallest preparation goal:

> Make refactor/holla-parity a clean, unarmed, host-executable, frozen-oracle-backed campaign branch. Do not modify production behavior or the frozen tag. Pass plan validation, latest taskfmt verification, host receipt qualification, strict refactor-proof compilation, and the full nextest/visual preflight.

Until that goal passes, do not launch the autonomous 73-task implementation campaign.
