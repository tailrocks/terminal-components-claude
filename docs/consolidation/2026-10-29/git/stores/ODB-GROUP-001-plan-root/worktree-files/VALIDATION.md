# Validation and release gates

Use this file with [REVIEW.md](REVIEW.md), [PLAN.md](PLAN.md), [TASKS.md](TASKS.md), [API-RULES.md](API-RULES.md), [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md), [SOURCES.md](SOURCES.md), and [TESTING-RULES.md](TESTING-RULES.md). The testing rules govern conflicts about tests and source comments. Keep the other acceptance contracts in this file.

## 1. Decision rule

PR #38 is not ready for acceptance at the reviewed head. Complete the behavior, architecture, consumer, and validation tasks before merge or release. A crate count, a screenshot, or a green subset of CI does not prove completion.

The candidate is `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`. The comparison source from PR #26 is `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`. A result for either commit does not validate a later commit.

Use these labels in every report:

| Label | Meaning |
|---|---|
| Observed | Source, manifest, or named run establishes the statement. |
| Executed locally | This review ran the exact command in the local environment. |
| Remote CI evidence | A retrieved exact-head CI run supplies the result. |
| Proposed | The condition or command is part of future implementation. |
| Not run | No execution result exists in this review. |

The historical review environment had no local `cargo` or `rustc`. Rust compilation, Nextest, Clippy, packaging, PTY probes, and performance measurements did not run in that environment. The current bootstrap must discover available tools. Do not report a proposed command as a pass.

Complete T00, T37, T01, and T38 in that order. T00 records the old tests, harnesses, fixtures, doctests, and source comments as historical evidence. T37 removes their code and nonfunctional comments. T01 inventories the cleaned production source. T38 approves the functional test architecture. A zero-case test command is `NOT VERIFIED` and cannot satisfy acceptance. Follow [TESTING-RULES.md](TESTING-RULES.md) before any test or harness edit.

## 2. Current evidence

This section records evidence for the reviewed target SHA only. Test paths, assertions, and counts identify historical source inputs for T00. They are not commands or targets to retain or execute after T37.

### 2.1 Exact-head CI

[Run 40222193598](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598) records the candidate head exactly. It is a `pull_request` run, attempt 1. It completed with failure. The jobs API returns 100 jobs on page 1 and zero on page 2. There are 84 successes, 14 failures, and two skipped jobs.

Six package jobs fail in both default and testing configurations. `Required verdict` and `Required` also fail. The following table reports the directly inspected log for each package. The alternate configuration is known to fail; its exact failure set must be checked during repair.

| Package and reviewed job | Failure | Immediate acceptance task |
|---|---|---|
| [jackin-preview-app testing, 121257622585](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598/job/121257622585) | `jackin_visual_baseline`; `accounts-mixed 100 30 junie truecolor`; 347/348 tests pass | Explain the changed frame against the frozen target. Repair the source or obtain the specific visual-change decision. |
| [holla-ui default, 121257622374](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598/job/121257622374) | Three visual failures. The scrollbar foreground at `(78,16)` is `Indexed(238)` instead of `Indexed(235)`. `activities-multi 80 24 junie 16` also differs. 272/275 tests pass. | T00 records the failures and named cases. Repair the color/scroll contract. Prove replacements only after T38 review. |
| [showcase-ui testing, 121257622521](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598/job/121257622521) | Page heading reference fails for buttons, `100x30`, 16 colors, row 2. The candidate includes a scrollbar and lacks the reference marker. Overview baseline also differs. 217/219 tests pass. | Reconcile viewport, marker, and page heading behavior through the shared owners. |
| [tablepro-ui default, 121257622702](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598/job/121257622702) | `tablepro_visual_baseline`; `history-tab 120 40 junie truecolor`; 381/382 tests pass | Check shared list/detail composition, then capture and compare the changed frame. |
| [termrock-xtask default, 121257625735](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598/job/121257625735) | `ownership_check_passes_on_workspace`; 500 files scanned; 3,401 findings; 268 wrappers; 199 checker problems; 164/165 tests pass | Classify scanner errors, incomplete migrations, and real ownership growth. These are not 199 independently confirmed product bugs. |
| [visual-harness default, 121257625816](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598/job/121257625816) | `overlay_sites`; 13 unlisted sites; 33/34 executed tests pass; 29,073 skipped | T00 records the old scanner result. Classify the production sites from source. Design any replacement gate proof after T38. |

The failing assertions are [shared digest, line 284](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/src/digest.rs#L284), [Holla visual, line 101](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/holla-ui/tests/visual.rs#L101), [Showcase headings, line 38](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/tests/page_headings.rs#L38), [ownership, lines 15–31](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-xtask/tests/ownership.rs#L15-L31), and [overlay inventory, lines 487–540](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/visual-harness/tests/visual_baseline/overlay_sites.rs#L487-L540).

### 2.2 Local outcomes

| Command or inspection | Result |
|---|---|
| `git rev-parse HEAD` in the candidate checkout | Exact candidate SHA confirmed. |
| `command -v cargo` and `command -v rustc` | No path returned. No Rust toolchain was available. |
| `PYTHONDONTWRITEBYTECODE=1 python3 tools/test_perf_workflow.py` | Exit 5. `setUpClass` raises `FileNotFoundError` for `.github/workflows/perf.yml`. Zero tests run. |
| `git ls-tree HEAD .github/workflows/perf.yml` | No tracked entry. The missing workflow is not a sparse-checkout omission. |
| Python `tomllib` inspection of the workspace and production policy | 49 workspace members. All 20 declared production libraries lack `[lints] workspace = true`. Thirteen workspace members inherit lints. |
| `git status --short` after this review | Empty. The source checkout was not changed. |

The target-SHA performance workflow test expected a deleted workflow and old package names. T00 records its source and the old inventory as historical inputs. T37 removes test-only tools and inventory. Rebuild required production gates through their owned tasks and approved plans. Do not add a second source of truth.

## 3. Evidence record and gate integrity

Every required run must record:

1. Candidate commit and source tree. Record a dirty-source digest if the tree is not clean.
2. Reference commit, expected artifact digest, and comparison policy when relevant.
3. Exact command, working directory, environment variables that change behavior, tool versions, and target.
4. Package, feature, target, test, and checkpoint inventories selected before execution.
5. Exit status, passed/failed/skipped counts, complete logs, and artifact hashes.
6. Independent reviewer and disposition of each material finding.

After T38, each approved plan defines stable case IDs. Reject empty selections, duplicate IDs, unknown or missing results, and an expected case reported as skipped. Add cases to the active inventory only under an approved plan. T00 records removed tests and their disposition as historical migration evidence. Do not use test counts as a quality KPI.

Preserve these gate failure requirements: removing a required case, marking a required case skipped, removing a package or lint inheritance, adding an illegal production dependency or unlisted overlay, or changing an approved frame must fail the relevant gate. T00 records existing mutation checks as historical evidence. Do not retain or run their old test code after T37. Recreate needed gate checks only under a T38-approved architecture and an independently reviewed task plan. Prove consequential failure modes; do not use test counts as a quality measure.

Preserve [AGENTS.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/AGENTS.md#L1-L20) and every safety contract. Keep essential safety rationale in maintained Markdown and design records. Follow [TESTING-RULES.md](TESTING-RULES.md) for source comments and required tool text. Do not add blanket `allow(unsafe_code)`, `allow(clippy::all)`, or a broad exception increase to pass a gate. Use an operation-level exception with a specific invariant when the implementation requires it.

## 4. Crate and feature checks

### 4.1 Complete production inventory

Use the 20-package inventory in [acceptance_policy.json](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/tools/acceptance_policy.json#L1-L25). Its current members are:

`termrock-core`, `termrock-layout`, `termrock-theme`, `termrock-text`, `termrock-render`, `termrock-collections`, `termrock-runtime`, `termrock-controls`, `termrock-fields`, `termrock-feedback`, `termrock-navigation`, `termrock-viewport`, `termrock-overlays`, `termrock-grid`, `termrock-editors`, `termrock-forms`, `termrock-terminal`, `termrock-session`, `termrock-shell`, and `termrock`.

The policy must change with an approved crate-boundary change. Compare it against Cargo metadata and the published-package list. Fail if a production crate has no assigned owner, check, or packaging decision.

### 4.2 Isolated builds

**Proposed acceptance.** Build each public crate in a separate invocation with:

- No default features, if the crate advertises this configuration.
- Default features.
- Each public feature by itself with defaults disabled.
- All supported features.
- Any additional documented feature combination that changes platform support or API shape.

Use `--lib` first. Tests, examples, and workspace members can enable dependencies that hide a broken production graph. Then run the applicable all-targets, Nextest, and documentation-consumer checks separately.

Create a temporary detached consumer for each leaf crate. Its only Termrock dependency must be that crate. It must import and use a public value or operation. Put its target directory outside the repository. Do not add the facade or test support to make a leaf probe compile. Capture the normal/build dependency graph and activated features.

The [Cargo resolver rules](https://doc.rust-lang.org/cargo/reference/resolver.html#features) require separate invocations when feature unification would hide isolation. `--workspace --all-features` does not replace these probes.

The runtime's [conditional time module and unconditional exports](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/runtime.rs#L1-L8) are a first reproduction target. Run this command once the toolchain is available:

```sh
cargo check --locked -p termrock-runtime --lib --no-default-features
```

This command is not a reported result. The source indicates a feature defect; no compiler output was obtained here.

### 4.3 Dependency and lint acceptance

The no-backend build must exclude session IO, PTY code, and backend adapters. A controls-only consumer must not pull editors, grids, apps, visual harnesses, or repository task tools. Record actual dependency edges; do not infer this from crate names or `default = []`.

The facade currently has unconditional family and session dependencies despite its empty default list. See [facade manifest, lines 8–33](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock/Cargo.toml#L8-L33).

All production packages must inherit the selected workspace lint policy. The [root policy](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/Cargo.toml#L139-L162) is not inherited automatically. [Cargo requires an explicit package lints table](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table). Resolve newly exposed issues before declaring this gate complete.

## 5. Behavior and external consumer checks

Plan enough scenarios to prove each selected change. Prefer integration scenarios through a real production path and public API. Add an owner-level scenario only when it gives necessary evidence. One scenario may observe both owner and consumer behavior. Do not require a fixed number of tests. Use [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md) and [API-RULES.md](API-RULES.md) to select behavior, not to set a test-count target. Each plan needs independent Sol medium review before any test write.

| Contract | Required observable behavior |
|---|---|
| Stable identity | Sort, filter, reorder, insert, delete, and replace data. Preserve the live selected/focused identity or apply the documented fallback. |
| Text editing | Grapheme-safe movement, selection, backspace, delete, paste, and truncation. Include combining marks, wide characters, and joined emoji. |
| Input ownership | Exactly one owner receives activation. Preserve unhandled input. A modal blocks the background. A disabled control does not activate. |
| Layout | Empty, one-item, ragged, narrow, zero-area, resized, and scrolled content. Test clipping and hit regions together. |
| Runtime frames | Reject stale preparation or receipts. A failed presentation does not commit the new hit/focus state. Idle waits do not redraw without a due change. |
| Mutations | Validation failure leaves prior state intact. Stale actions and old request generations cannot act on a new owner. |
| Composition | Custom theme, custom row/cell content, controlled state, nested overlay, and independent state instances work through public APIs. |

Retain the detached [external consumer source](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/external-consumer-demo/Cargo.toml#L11-L29) if T00 confirms that it is a maintained consumer example rather than a test-only target. Its target-SHA CI jobs are historical evidence. Do not retain their old test code or job selections after T37. Use it for a reviewed consumer scenario after T38. Add leaf and package consumers only through their owned tasks. The path consumer alone does not prove publication.

Documentation examples must compile through registered consumer targets. The library manifests set `doctest = false`; do not claim that ordinary Nextest runs execute Rustdoc examples. T00 records the old documentation inventory test as historical evidence. T37 removes it. After T38, register or update a consumer compile target through an approved task plan whenever an example changes.

## 6. Terminal, runtime, and async checks

### 6.1 Terminal ownership

Prove terminal lifecycle in isolated processes with a PTY or the native platform equivalent. T00 records the old Nextest process-isolation configuration as historical evidence. An approved replacement case must define its process isolation and capture method.

Test normal quit and each partial-entry failure. Test read, poll, draw, protocol-write, and interrupted-syscall failures. Test terminal EOF, unwind panic, repeated sessions, denied nested/concurrent sessions, and panic after a session ends.

Observe raw mode, alternate screen, cursor shape/visibility, wrap, mouse capture, paste mode, and keyboard protocol depth. Check stdin identity, descriptor closure, and cleanup output order.

Install a host panic hook in one process-isolated probe. That hook must observe restored real stdin, termios, and screen modes.

The source has useful guarded entry and retryable session leave. Preserve them. It also chains a new global panic hook on each entry, with no deactivation on leave. See [session.rs, lines 394–455](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/session.rs#L394-L455) and [lines 514–535](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/session.rs#L514-L535). Add a live-session cleanup contract. An ended session must emit no later cleanup bytes.

Bridge restore consumes its stored state before `dup2` succeeds and ignores restoration errors. See [esc_bridge.rs, lines 866–878](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/esc_bridge.rs#L866-L878). Add an explicit fallible finish path. Preserve retryable ownership on failure where possible. Keep best-effort cleanup in `Drop`. Prove that repeated or concurrent entry cannot replace another live session's process stdin.

Do not promise restoration after an uncatchable kill or abort. State supported failure behavior precisely.

### 6.2 Event bursts and reference behavior

PR #26 supplies historical bounded-batching behavior and semantic-test evidence. T00 records it as a source input. Do not copy its test code. Adapt the required behavior only after the candidate frame lifecycle is stable and T38 approves the replacement architecture.

Require every input event to be handled in order. Use a default cap of 64 events and a 16 ms batch admission budget. Check elapsed time between deliveries. Admit no further event after the budget expires. A synchronous handler can exceed the budget; do not promise a hard real-time deadline. Test cap and deadline boundaries with a fake clock.

Batch only already-ready compatible events. Settle, present, and acknowledge whenever the runtime blocks further input or needs current geometry. Never fabricate an acknowledgement.

A batch may reduce presentations; it must not discard navigation steps. Activation, text changes, owner changes, modal transitions, and resize need explicit boundary rules. Compare delivered event counts, final state, and required intermediate frames against unbatched delivery. Do not require 120 events in two frames from the candidate. Measure actual presentation savings after its readiness contract passes. Preserve the existing no-periodic-idle-redraw behavior.

### 6.3 Async scope

The current runtime is synchronous. Do not add Tokio to core or claim async support from simulated jobs. First prove an external host can drive the production runtime without session helpers or test-only APIs.

If a selected real consumer needs an async adapter, require a bounded queue, request-generation identity, cancellation, error delivery, and joined shutdown. Test late completion after a newer request, route close, duplicate result, queue saturation, worker panic/error, and shutdown during IO. Apply results on the UI owner only. A stale completion must not mutate current state. Do not block an async executor worker with terminal reads. Use the session thread or a dedicated blocking IO owner.

## 7. Performance acceptance

### 7.1 Preserve existing deterministic bounds

These values are historical source assertions recorded by T00. T37 removes their old test code. They were not measured locally in this review. Keep the performance contracts. After T38, prove them with reviewed cases and the selected release workload.

| Workload | Existing invariant | Source |
|---|---|---|
| Borrowed picker at 1,000 and 100,000 items | Exactly 38 item reads for 19 visible rows; equal region counts; zero warm allocations and bytes | [perf_collections.rs:98–128](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/tests/perf_collections.rs#L98-L128) |
| Append to a 10,000/100,000-line viewport | Allocation count does not grow with prior line count; only appended lines are indexed | [225–244](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/tests/perf_collections.rs#L225-L244) |
| Warm 1,000/100,000-line viewport render | Zero allocations; no prefix reindex; equal visible-row work | [273–298](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/tests/perf_collections.rs#L273-L298) |
| Viewport width invalidation | Cold, changed-width, and invalidated layouts index the required lines exactly | [300–342](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/tests/perf_collections.rs#L300-L342) |
| Grid with 50 versus 500 rows | Equal cell reads and region counts; no access beyond visible row 29; fewer than 100 measured allocations | [761–774](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/tests/perf_collections.rs#L761-L774) |
| Warm scroll-edge rendering | Zero allocations over 200 frames | [scroll_edges_allocations.rs:46–56](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/tests/scroll_edges_allocations.rs#L46-L56) |

Extend list/grid scale cases to 100,000 rows where the public source model supports it. Require a bound based on visible rows, visible columns, and declared overscan. Do not demand constant work for an operation that must inspect all rows, such as a full sort. Measure full-data changes separately from warm draw.

### 7.2 Required release job

Use release builds with `PERF_FULL=1`. Do not set `PERF_ITERS`, `PERF_BLESS`, or visual blessing variables in the required job. Include controls, grapheme editing, collections, viewport append/invalidation, focus/hit routing, overlay lifecycle, and one representative frame/input journey from each app.

The current helper returns success for a missing baseline. Allocation/byte baseline checks are blocking only outside debug builds. Timing is blocking only with `PERF_STRICT`. See [perf.rs:366–392](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/src/perf.rs#L366-L392). Add a gate mode that rejects missing baselines and incomplete workload selection. Retain local exploratory reporting as a separate mode if needed.

Count allocations, requested allocation bytes, source accessor calls, indexed lines, visible rows, hit regions, and focus stops. Do not label requested allocation bytes as retained memory. The existing allocator is thread-local. It does not count worker-thread allocations. Add worker/process measurements only when a real worker path enters scope.

### 7.3 Relative timing budget

The following is a proposed budget, not a reported result. Use a controlled runner and paired reference/candidate runs. Preserve the historical 20% time-regression boundary for stable workloads after the noise floor is established. Keep the historical 1.5 ratio for the 100,000-versus-1,000 warm collection comparison where T00 confirms its contract. Record raw samples and confidence intervals. A noisy run needs repeat evidence; it does not justify automatic baseline approval.

Require zero increase for existing zero-allocation contracts. For nonzero allocation baselines, require no increase unless a reviewed API change explains the new cost. Keep cold construction, steady draw, mutation, and first-frame latency separate. Do not claim a frame rate, CPU saving, memory saving, or speedup from this static review.

## 8. Visual and semantic proof

Use immutable reference source, approved visual artifacts, and binaries. Record their hashes. T00 records the current paired runner, CLI, and manifests as historical evidence. T37 removes old test-only harness code. Keep a current runner only if T00 proves that it serves a supported production or diagnostic path. T38 and an approved task plan must define any replacement harness before its first code edit.

The comparison must use the same selected cases, dimensions, color modes, event sequence, clock steps, and ordered frame checkpoints. Compare cell symbols, foreground/background, attributes, cursor position/shape/visibility, and logical stage identity. A screenshot alone cannot establish interaction or cursor correctness.

Run approved reference capture, replay, and candidate comparison in separate evidence directories. The candidate must not write to expectations. A missing, duplicated, reordered, extra, skipped, or failed checkpoint leaves coverage incomplete. Hash reference artifacts before and after the run. Keep original failed expectations and actual/diff artifacts.

A permitted reference exception must name the exact case, checkpoint, expected hash, actual hash, source/binary hashes, reason, and human decision. Do not generalize one stale snapshot exception to nearby cases. Do not bless all changed frames after a refactor. If product behavior changes deliberately, record that contract separately from historical parity.

Use a changed-component matrix for fast iteration. Before final acceptance, run the complete approved scenario matrix and ordinary required checks. The target-SHA visual-harness run skipped 29,073 cases. That historical run does not provide complete CLI parity evidence and its old test target is not an instruction to execute after T37.

## 9. Platforms, MSRV, and CI generation

Define the support contract before release. At minimum, qualify Linux on the declared architecture. Add macOS native session probes if macOS is claimed. Compile platform-neutral components on Windows without Unix session code. Qualify a Windows session adapter only if it is part of the supported release.

The current session includes Unix `std::os::fd`, PTY, and termios code unconditionally. See [esc_bridge.rs:10–19](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/esc_bridge.rs#L10-L19) and [1048–1147](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/esc_bridge.rs#L1048-L1147). A Linux-only CI pass is insufficient for a cross-platform claim.

The workspace declares Rust 1.99. The toolchain pins 1.99.1. Build supported consumers with 1.99.0 and 1.99.1. If 1.99.0 fails because an essential dependency or language/library operation requires a newer version, change the declared MSRV deliberately and explain the reason. Do not infer MSRV from the developer pin.

[ci.yml line 1](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/.github/workflows/ci.yml#L1) identifies Velnor Actions 0.1.48 as the generator. Edit [.velnor/config.toml](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/.velnor/config.toml#L1-L63). Regenerate with `mise exec -- velnor-actions generate`. [mise.toml line 9](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/mise.toml#L9) pins that tool. [The generated-file check](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/.github/workflows/ci.yml#L448) uses its `generate --output-dir` interface. These commands did not run in this review. Do not hand-edit generated YAML as the maintained fix.

Add independent build profiles, production-consumer jobs, release performance bounds, and claimed OS/MSRV jobs through the owned generator inputs. Make the required verdict depend on every selected gate. Keep optional caches optional. Upload failure artifacts even when tests fail. Reject a plan that silently marks a changed production package unaffected. Prove the generated files are fresh and the verdict fails when a required gate fails or disappears.

## 10. Package and release acceptance

The current internal dependencies are path-only. See [Cargo.toml:70–120](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/Cargo.toml#L70-L120). [Cargo does not permit path-only registry dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#local-paths-in-published-crates). Release is currently disabled in Velnor.

Complete these gates in order:

1. Select the published crate list and version policy. Set `publish = false` on non-public apps, test support, conformance, and repository tools as appropriate.
2. Add compatible internal version requirements, descriptions, license coverage, repository/README metadata, and feature documentation. Inspect every package file inventory.
3. Package public crates in dependency order. Verify the packaged source. Reject missing files, workspace-only includes, fixture dependencies, or wrong metadata.
4. Install the staged `.crate` set through an isolated local registry. Use clean consumer manifests with no source path, repository patch, or workspace membership.
5. Run a leaf consumer for every public crate. Run a facade consumer for custom theme, custom component, collection, modal, and supported session behavior.
6. Run registry dry runs after sibling dependencies are available in the selected registry. An initial staged test may use a local registry. Do not call a crates.io dry run complete if it could not resolve unpublished sibling versions.
7. Review public API and feature changes, migration notes, package size, dependency closure, and license/advisory results. Resolve actual findings. Do not invent a vulnerability or reject a dependency based on age alone.
8. Bind all final evidence to the release commit and package hashes. Re-fetch PR feedback and required checks at that commit. Obtain the separately required release/merge authorization.

Release publishing must remain disabled until the selected package set passes. Publication order, retries, and partial-publication recovery must be explicit. Re-running a release job must not overwrite an existing version or tag with different source.

## 11. Commands and future helper specifications

### 11.1 Build and test command selection

The old command list named packages, tests, and harnesses at the target SHA. T00 records it as historical evidence. Do not execute a test or harness target removed by T37. T01 records the cleaned production inventory. T38 approves the test architecture. Only then may an independently reviewed task plan select replacement test commands and case IDs.

Use the current package and feature inventory when selecting build checks. A zero-case test command is `NOT VERIFIED`. Do not treat a successful empty command as evidence. Use `cargo nextest` for approved Rust test targets. Record exact case IDs, source SHA, command, environment, logs, artifacts, and independent result. Do not use test count, coverage, or speed as a quality KPI.

Use `PERF_FULL=1` only for the approved release performance run. Use `PERF_STRICT=1` only on a controlled timing runner. Do not enable blessing variables during validation.

T00 records the old frozen runner's CLI and inventory as historical evidence. Preserve approved immutable visual artifacts separately. Do not retain or execute old test-only runner code after T37. Record the CLI and inventory of a replacement only after its architecture and task plan pass review. Do not substitute a screenshot-only comparison for approved semantic proof.

### 11.2 Future scripts: required behavior, not existing commands

The following helpers are proposed. Their filenames and CLI are not current repository commands. Extend existing tools where possible. Record the final interface in this file after implementation.

| Proposed helper | Inputs | Required output and failure behavior |
|---|---|---|
| Isolated-consumer check | Production policy, Cargo metadata, feature contract, target/MSRV matrix | One detached probe per crate/configuration; dependency graph; compiler output; inventory totals. Fail on missing probes, accidental facade/test features, or compilation errors. Extend `tools/acceptance_architecture.py` or a tightly scoped companion. |
| Approved functional-case inventory | T01 production requirements, T38 architecture, reviewed task plans | Current approved case IDs and results. Fail on missing, duplicate, unknown, empty, or skipped required selections. Rebuild gate proof under a reviewed plan; do not retain old inventory or mutation-test code. |
| Performance gate mode | Reviewed workload inventory, baselines, profile/environment manifest | All required workloads, raw samples, allocation/work counters, comparison result. Fail on a missing baseline, workload, or required bound. Rebuild proof under an approved task plan. Extend `tools/acceptance_architecture.py` or a tightly scoped companion only when its production ownership is confirmed. |
| Package-consumer check | Published-package allowlist, version graph, staged `.crate` files | Reviewed package inventories and clean local-registry consumer runs. Fail if a source checkout path is needed or a package is missing. |
| Final evidence check | Candidate SHA, required job/test/case inventories, package hashes, review dispositions | One machine-readable verdict with incomplete reasons. Fail if a result is stale, missing, failed, or bound to a different source. Do not make this a second maintained status document. |

## 12. Completion record

Acceptance requires all selected contracts in [TASKS.md](TASKS.md) and [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md). Complete all mandatory checks above. Resolve review feedback. Record remaining work explicitly. A missing test does not prove completion. A broad scanner that no longer finds source text does not prove completion.

Every required test lane needs selected approved cases. Record a zero-case lane as `NOT VERIFIED`; it cannot pass final acceptance.

The final report must separate product parity, architecture/reuse, runtime/platform support, performance, and release readiness. State each result at the final commit. If a required environment is unavailable, state the exact missing proof and keep that acceptance gate open.
