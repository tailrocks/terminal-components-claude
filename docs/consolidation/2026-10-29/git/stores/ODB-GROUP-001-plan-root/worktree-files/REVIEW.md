# Source review and branch decisions

## Decision

Finish the work in pull request 38.
Keep its runtime contracts and most crate owners.
Correct the boundaries that still depend on application code or terminal backends.
Use pull request 26 for required appearance and interaction.

Do not merge the reference branch as a whole.
Its main package still contains the library and four applications.
Port the useful behavior through the target's typed API.

This review supports a specific completion plan.
It does not certify that every source line or possible state is correct.

## Fixed review inputs

| Input | Value |
| --- | --- |
| Repository | `tailrocks/terminal-components-claude` |
| Target pull request | [38](https://github.com/tailrocks/terminal-components-claude/pull/38) |
| Target branch | `parity/termrock-visual-baseline` |
| Target commit | `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4` |
| Reference pull request | [26](https://github.com/tailrocks/terminal-components-claude/pull/26) |
| Reference branch | `design/interaction-polish` |
| Reference commit | `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948` |
| Common ancestor | `cc14dd6beae526884aabdf897e309be837b4f504` |
| Review date | 26 October 2026, Asia/Ho_Chi_Minh |

Both pull requests were open drafts at review time.
The current target has 49 workspace members.
Its declared production library list has 20 crates.
Two additional crate manifests are outside that workspace list.
These are `termrock-e2e` and `termrock-visibility-tests`.

The review covered manifests, public boundaries, critical code, application paths, test gates, and four image pairs.
It also checked current GitHub CI evidence and the retained project contracts.
It did not execute the applications.

No `cargo` or `rustc` executable was available in the review environment.
No local Rust build, Rust test, benchmark, or live terminal capture ran.
A source defect, a remote CI failure, and a local result have different evidence types.

## Evidence terms

| Term | Meaning |
| --- | --- |
| Observed | Source, a manifest, an image, or a recorded result shows the condition. |
| Source defect | The code permits or causes the stated failure. Local runtime reproduction can still be absent. |
| Design gap | The API lacks a required boundary or capability. This is not a measured runtime failure. |
| Proposed | The plan defines a future change or test. |
| Not tested | This review did not execute the stated path. |

## What to keep

| Area | Source evidence | Decision |
| --- | --- | --- |
| Typed event results | [Response][response] separates flow, invalidation, state, and actions. | Keep the model. Remove manual reconstructions that lose metadata. |
| Component state | Components borrow configuration and use caller-owned semantic state. | Keep this direction. Do not restore mutable render-owned widget state. |
| Frame lifecycle | [Runtime][runtime] and [session][session] prepare, project, present, and acknowledge frames. | Keep the protocol and stale-frame rejection. |
| Style overrides | [Override contract][overrides] separates parts, scoped changes, and slot states. | Keep all supported override levels. |
| Collection identity | Shared collection, Grid, Tree, and Steps code uses stable keys and work counters. | Keep keys and bounded work. Make data-change rules explicit. |
| External consumer | [Detached consumer][external] has its own workspace and lockfile. | Keep it. Add leaf and packaged-consumer checks. |
| Component composition | [Adapters][adapters] already cover many component families. | Complete these adapters. Do not add another layout engine. |
| Terminal cleanup | [Session][session] has entry rollback and retryable cleanup steps. | Keep these mechanisms. Correct lifetime ownership and final error reporting. |
| Current SQL sort fix | [Domain sort][table-domain] uses `cmp_values_nulls_last`. | Keep the fix and its regression cases. Update stale prose. |
| Performance tests | [Collection performance tests][perf-collections] count visible-row reads and allocations. | Keep deterministic work limits. Repair their required CI path. |

## What to borrow from pull request 26

| Reference | Required behavior | Target owner |
| --- | --- | --- |
| [54 Showcase pages][reference-pages] | Preserve each real route, example, and required state. | Showcase consumers of shared components |
| [Button][reference-button] | Restore mnemonic and tooltip behavior with effective shortcut hints. | controls plus runtime-owned tooltip requests |
| [Showcase leave policy][reference-leave] | Let pages handle keys before quit. Protect unsaved edits and active work. | shell and application policy |
| [Input pump][reference-pump] | Bound event bursts. Preserve every event and reduce repeated presentation work. | session with runtime frame rules |
| [Unix suspend][reference-suspend] | Restore terminal modes before suspend. Re-enter and redraw after resume. | supported session adapter |
| [Target product contract][product] | Preserve palette roles, focus, selection, seams, scrollbars, and color modes. | theme, runtime, component owners |

Do not copy the reference's complete widget or runtime architecture.
Do not copy its process-global state without a lifetime review.
Do not retain a known defect only because a historical image contains it.

## Required findings

<a id="f01"></a>

### F01. Current CI fails

Priority: P1.
Evidence type: observed remote result.
Tasks: T02, T11, T27, T28, T29, T32, T36.

[CI run 40222193598][ci] used the exact target commit.
It has 100 jobs: 84 successful, 14 failed, and two skipped.
Six packages fail in both default and testing configurations.
The two required-result jobs also fail.

| Package | Directly inspected failure |
| --- | --- |
| `jackin-preview-app` | `jackin_visual_baseline` differs for `accounts-mixed 100 30 junie truecolor`. |
| `holla-ui` | Historical border and faint-color replays fail. The application visual digest also differs. |
| `showcase-ui` | The button page heading differs at 100×30 in 16 colors. The Overview digest also differs. |
| `tablepro-ui` | The history-tab visual digest differs at 120×40 in TrueColor. |
| `termrock-xtask` | `ownership_check_passes_on_workspace` fails. |
| `visual-harness` | `overlay_sites` reports 13 unlisted sites. |

One log per package was inspected in detail.
The other configuration also has a failed job.
Do not assume that both configurations have identical failed cases.

The ownership checker reports 199 problems.
This count is not 199 independently verified product defects.
The overlay report includes extracted test files and two production functions.
Correct scanner classification and real production gaps separately.
Do not increase exception limits to hide either class.

<a id="f02"></a>

### F02. Production crates do not inherit the declared lints

Priority: P1.
Evidence type: observed manifest condition.
Task: T06.

All 20 production libraries omit `[lints] workspace = true`.
The root [manifest][root-cargo] declares the lint policy.
Only 13 of the 49 workspace members inherit it.

Some source files have local lint attributes.
Those attributes do not activate the complete workspace policy.
Cargo requires explicit inheritance.
Add the inheritance, correct the resulting findings, and test the manifest rule.

<a id="f03"></a>

### F03. The runtime gates its clock module incorrectly

Priority: P1.
Evidence type: source defect; compiler reproduction not run.
Task: T03.

[Runtime lines 3–7][clock-gate] gate `mod time` behind `crossterm`.
The same file exports `ClockError` and `Moment` without that gate.
The clock module uses `core::time::Duration`.

Compile the clock in every runtime configuration.
Gate only the backend adapter.
Prove the fix with an isolated no-default-feature consumer.

<a id="f04"></a>

### F04. The facade and leaf crates do not have reliable feature isolation

Priority: P1.
Evidence type: observed manifests; resolved graph not built here.
Tasks: T03, T05, T34.

The [facade manifest][facade-cargo] sets `default = []`.
It still requires all 19 other production library crates.
That set includes the terminal session.

Core and runtime enable backend support by default.
Several families inherit the full Ratatui dependency.
An empty feature list does not make a dependency graph small.

Use the explicit feature design in [ARCHITECTURE.md](ARCHITECTURE.md).
Test each leaf outside workspace feature unification.
Keep session support explicit.

<a id="f05"></a>

### F05. Feedback owns application screens

Priority: P1.
Evidence type: observed ownership defect.
Task: T07.

The [feedback exports][feedback-exports] include account, settings, manager, capsule, and usage screen models.
[SessionChrome][session-chrome] requires instance, branch, account, and attachment data.
[SettingsMount][settings-page] contains mount scope and isolation data.

Move the product models and page assembly to the existing Jackin UI crates.
Keep generic status, progress, meter, notice, and hint components in feedback.
Extract reusable paint behavior only when a concrete consumer needs it.

Do not keep the same product model under a generic name.
Do not create a library-to-application dependency to preserve an old export.
Record each API retirement or compatibility adapter.

<a id="f06"></a>

### F06. Domain crates depend on the complete UI facade

Priority: P1.
Evidence type: observed ownership defect.
Task: T08.

Jackin domain code uses the facade for a text-wipe operation.
Jackin simulation code also uses presentation and widget types.
TablePro domain code owns editor, grid, and form UI state.
See the [Jackin domain manifest][jackin-domain] and [TablePro domain manifest][table-domain-cargo].

Use the smallest neutral owner for common values.
Move UI state and adapters to UI crates.
Map domain state to UI state in the presentation layer.
Keep SQL comparison and validation in the domain or SQL crate.

<a id="f07"></a>

### F07. Composition errors can disappear through a trait

Priority: P1.
Evidence type: observed source behavior; runtime reproduction not run.
Task: T10.

[Child lines 758–767][child-errors] return a zero size after a measurement error.
The trait draw method discards the draw result.
A shell can accept that child through `CompositionLeaf`.
The outer operation can then report success after the child disappears.

Extend the existing trait with a default validation hook.
Validate built-in nested content before measurement or registration.
Keep the exact depth limit and safe destruction behavior.
Test the trait-conversion path, not only direct `try_draw` calls.

<a id="f08"></a>

### F08. Repeated helper code has different contracts

Priority: P2.
Evidence type: observed duplication and an untested error risk.
Tasks: T09, T10, T11.

[Render PartStyle][render-helper] and [runtime PartStyle][runtime-helper] repeat patch-merge policy.
Facade `Leaf` and runtime `ClosureLeaf` repeat callback adaptation.
Some composite controls rebuild responses by hand.

The [completing editor][completion-response] selects one child response by `is_changed`.
That logic does not take the maximum invalidation from both children.
This review did not prove a current user journey that loses `Layout`.

Keep one pure patch-merge owner.
Use the existing `erase` and `map_action` operations.
Add only the typed effect-merge operation specified in the API rules.
Preserve parent identity, state, and action.

<a id="f09"></a>

### F09. Collection changes need an explicit revision contract

Priority: P2.
Evidence type: design gap with an existing supported invalidation path.
Task: T14.

[Collection reconciliation][reconcile] can skip work after it checks length and endpoint keys.
An interior reorder can keep all those values unchanged.
The current API has `invalidate()`.
An existing NavList reorder test calls it.

Do not report this shortcut as an unconditional new bug.
Make the caller's mutation obligation explicit.
Add a safe revision-bearing update path.
Keep the existing bounded-work contract.

<a id="f10"></a>

### F10. Button lost useful reference behavior

Priority: P2.
Evidence type: API gap.
Task: T12.

The [reference Button][reference-button] has mnemonic and tooltip behavior.
The [target Button][button] lacks these fields and builders.

Port the behavior through shared binding and tooltip ownership.
Keep borrowed labels, typed actions, runtime routing, and immutable draw.
A displayed shortcut must match the effective binding.

<a id="f11"></a>

### F11. Showcase handles Ctrl+C before the selected field

Priority: P1.
Evidence type: source-confirmed routing defect; live reproduction not run.
Tasks: T19, T22.

[Showcase key registration][showcase-keys] binds Ctrl+C in Capture and Bubble phases.
[Its update path][showcase-quit] calls `cx.quit()` before the active page update.
This path has no selection, dirty-state, or running-task check.

Use the shared leave policy.
Give the active layer and focused component the first applicable action.
Copy an allowed selection before quit.
Test each save, discard, cancel, stop, and stay result.

<a id="f12"></a>

### F12. Showcase still has 24 placeholder routes

Priority: P1.
Evidence type: observed source census.
Tasks: T22 through T26.

The source has 54 navigation entries.
It has 24 `PageId` values, one Selection route, and 29 Placeholder targets.
Five Placeholder targets already call live page code.
Thus 30 routes have content paths, and 24 still use `paint_placeholder`.

See [route declarations][showcase-routes] and [route tests][showcase-route-tests].
This count does not prove parity inside the 30 content routes.
The old `22 versus 54` statement in `docs/current-state.md` is stale.

Replace numeric exceptions with one typed registry.
Implement the exact missing pages in [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md).

<a id="f13"></a>

### F13. Coverage records exceed executable behavior proof

Priority: P2.
Evidence type: observed inventory condition.
Tasks: T01, T31.

The [coverage allowlist][coverage-list] has 4,511 disposition rows.
The categories contain 100 SC1, 1,008 SC2, 247 SC3, and 3,156 review rows.
The [proof registry][coverage-proofs] has six explicit behavior proofs.
All six concern Diff behavior.

This does not mean the repository has only six tests.
It means this registry proves only those six entries.
Require a real proof or a reviewed reason for each required public capability.
Do not mark a title or imported type as a complete demonstration.

<a id="f14"></a>

### F14. The active visual reference is still pending

Priority: P1.
Evidence type: observed manifest and harness conditions.
Tasks: T04, T32, T36.

The [oracle manifest][oracle] has `pending = true`.
Its source, harness, approval, and date fields are empty.
Its four application scenario counts are zero.

The harness can fall back to historical snapshots.
Ignored or uncaptured scenarios can remain outside a normal test run.
The reviewed visual-harness job skipped 29,073 scenarios.

Build and review a separate reference candidate.
Report capture, comparison, skip, pending, and mismatch counts separately.
A missing required capture must fail the claimed visual gate.

<a id="f15"></a>

### F15. Application paint code repairs theme values locally

Priority: P2.
Evidence type: observed design defect.
Task: T11.

[Showcase helpers][showcase-colors] replace one exact tint value with another.
The same code converts indexed colors into plain integers.
A separate seam constant compensates for a missing theme role.

Move these policies into semantic theme tokens and explicit color-level recipes.
Remove value-based app repairs after the shared recipe matches the reference.
Preserve useful custom-theme behavior.
Test TrueColor, 256, 16, and monochrome separately.

<a id="f16"></a>

### F16. Clipboard status cannot express external transfer failure

Priority: P2.
Evidence type: observed API gap.
Tasks: T09, T12, T13, T19, T20.

The [copy hook][clipboard] has type `Fn(&str)`.
It cannot return unsupported, refused, too-large, or transport-error status.
The runtime's local clipboard storage and redacted Debug behavior are useful.

Add a result-bearing adapter.
Keep local storage separate from external copy status.
Apply secret policy before transport.
Use a fake transport in tests.

The [shell keyboard flags][keyboard-flags] are process-global.
The [core keyboard module][core-keyboard] also has global capability and protocol state.
T09 creates a delivery snapshot for each runtime.
T20 owns session detection and protocol lifetime.
T19 removes the shell global mirror.
Prove that independent runtimes do not share hint state.

<a id="f17"></a>

### F17. Session cleanup needs one lifetime owner

Priority: P1.
Evidence type: observed design risk; PTY reproduction not run.
Task: T20.

[Session entry][session] installs a new chained panic hook on each entry.
Leave does not deactivate that hook.
The ESC bridge owns process stdin without a complete session lease.
[Bridge restoration][bridge-restore] removes state before the restore operation succeeds.

Add one active-session lease.
Keep cleanup state retryable where the operating system permits it.
Deactivate completed-session panic cleanup.
Return final restoration failures through an explicit fallible exit path.
Keep best-effort cleanup in `Drop`.

Do not promise cleanup after an uncatchable kill or process abort.

<a id="f18"></a>

### F18. Performance and test inventories contain old package assumptions

Priority: P1 for gate integrity.
Evidence type: local failure and observed configuration.
Tasks: T02, T32, T33.

The local command below exited 5 and ran zero tests.

~~~sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/test_perf_workflow.py
~~~

It requires a deleted `.github/workflows/perf.yml`.
The [test profiles][test-profiles] also name old `junie-tui` and `xtask` packages.

The [performance helper][perf-helper] permits a missing baseline.
Its allocation checks require a release build.
Its timing limit also requires `PERF_STRICT`.

Migrate existing tools and inventories.
Do not create a second competing evidence system.
Fail gate mode on an absent workload, absent baseline, or zero selected tests.

<a id="f19"></a>

### F19. Registry package use is not proved

Priority: P1 for library release.
Evidence type: observed manifests; package checks not run.
Task: T34.

Internal dependencies in the root manifest are path-only.
Release configuration is disabled.
The detached consumer uses a local path.
It proves useful API access, but not registry packaging.

Declare published crates and versions.
Mark internal tools and demonstrations `publish = false`.
Verify package content and consume staged packages without checkout paths.
Keep publication separate from package verification.

<a id="f20"></a>

### F20. Product documentation contains stale findings

Priority: P2.
Evidence type: observed source and prose mismatch.
Tasks: T01, T35.

The current TablePro sort path uses `cmp_values_nulls_last`.
It also contains a descending-order regression case.
The [application plan][old-plan] still describes an earlier broken comparator.

Retain the current fix.
Run its test before a fresh pass claim.
Correct the current-state document after the test result.
Do not create a new refactor task from an old defect report without a source check.

## Image evidence

Four corresponding PNG pairs were retrieved.
The four distinct reference images were viewed.
Each target file had the same hash as its reference file.
Each corresponding pair has identical bytes.
These files show historical design details.
They do not show current application output.

| Reference image | SHA-256 |
| --- | --- |
| `showcase/audit/inputs/120x40/truecolor.png` | `baa5efb092af797cf66b824b5089edda7fa3c05ff93bdd4c3d92a1f6943db6d3` |
| `showcase/flows/dialogs/open/120x40/truecolor.png` | `05f0da613e442a51c84f9c1e0fddfed6227d1ffda0c7562aad335f1ef6f2d47e` |
| `showcase/flows/inspector/open/72x20/nocolor.png` | `8e1bf49144b6e344c77f29d97f1a08e75ffe3522863993aa87aade1f27fafa3d` |
| `showcase/flows/inputs/selected/120x40/truecolor.png` | `78b6a13fa9eca9ad511a58970c5cf3a85712be5ab6e7cfbccedc9d45af7d2871` |

All paths are under `snapshots/`.
The images show distinct dark planes, compact headers, green focus treatment, and a dimmed modal background.
They also show older navigation groups.

The narrow inspector image has no visible inspector controls.
Do not call that a current application defect.
Require a semantic inspector assertion before a future capture can prove that state.

Static state specimens can show several focus glyphs.
Count live focus owners by semantic identity and layer.
Do not count glyphs in an image as a substitute.

## Review limits that remain in the implementation plan

The review did not execute all component states or all historical scenarios.
It did not measure latency, memory use, terminal restoration, or platform support.
It did not prove the correctness of every ownership-checker finding.

Tasks T01 and T31 require complete source-to-requirement and component coverage.
Task T36 requires current results at the final commit.
These tasks cannot close with historical checkboxes.

[response]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-core/src/response.rs#L7-L78
[runtime]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/runtime.rs
[session]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/session.rs#L291-L535
[overrides]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/contracts/overrides.md
[external]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/external-consumer-demo/Cargo.toml#L18-L31
[adapters]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock/src/composition/adapters.rs
[table-domain]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/tablepro-domain/src/domain.rs#L936-L939
[perf-collections]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/tests/perf_collections.rs#L98-L128
[reference-pages]: https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/bin/showcase/app.rs#L80-L136
[reference-button]: https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/widgets/button.rs#L14-L79
[reference-leave]: https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/bin/showcase/app.rs#L1031-L1050
[reference-pump]: https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/runtime.rs#L100-L159
[reference-suspend]: https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/runtime.rs#L303-L340
[product]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/product.md
[ci]: https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598
[root-cargo]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/Cargo.toml#L139-L187
[clock-gate]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/runtime.rs#L1-L7
[facade-cargo]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock/Cargo.toml#L8-L33
[feedback-exports]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-feedback/src/lib.rs#L17-L92
[session-chrome]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-feedback/src/session_chrome.rs#L23-L34
[settings-page]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-feedback/src/settings_page.rs#L38-L74
[jackin-domain]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/jackin-preview-domain/Cargo.toml
[table-domain-cargo]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/tablepro-domain/Cargo.toml
[child-errors]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock/src/composition.rs#L758-L767
[render-helper]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-render/src/lib.rs#L9-L54
[runtime-helper]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/author.rs#L38-L142
[completion-response]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-editors/src/completion.rs#L472-L484
[reconcile]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-collections/src/collection/reconcile.rs#L25-L30
[button]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-controls/src/button.rs#L76-L130
[showcase-keys]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L1252-L1264
[showcase-quit]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L2661-L2665
[showcase-routes]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L91-L148
[showcase-route-tests]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L3398-L3461
[coverage-list]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/coverage/allowlist.txt
[coverage-proofs]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/coverage/declarations_data.rs#L4708-L4757
[oracle]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/baselines/oracle/ORACLE.toml
[showcase-colors]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-demos/src/pages/mod.rs#L42-L61
[clipboard]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/runtime/clipboard.rs#L5-L43
[keyboard-flags]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-shell/src/keyboard.rs#L13-L35
[bridge-restore]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-session/src/esc_bridge.rs#L870-L878
[test-profiles]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/tools/test-inventory/profiles.json
[perf-helper]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-test-support/src/perf.rs#L366-L392
[old-plan]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/plan.md


[core-keyboard]: https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-core/src/keys.rs#L220-L227
