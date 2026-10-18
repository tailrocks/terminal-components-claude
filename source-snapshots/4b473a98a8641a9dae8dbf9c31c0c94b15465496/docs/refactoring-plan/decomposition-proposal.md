# Contingent execution decomposition

This proposal is retained as decomposition history. The reconciled 73-task catalog now exists; [task-index.tsv](task-index.tsv), canonical task packages and [proof-contract.md](proof-contract.md) supersede this document wherever they differ. In particular, TASK-001 now owns only comparator/host core; TASK-070 owns source/scenario runner operations; TASK-071 owns accounting; TASK-072 owns architecture verification; and TASK-073 establishes attributed conformance before component repairs. Independent qualification for those producers is required. Dynamic-disabled capture cancellation is already implemented and requires preservation/overlap proof, as ADJ-01 records. Do not execute this provisional label graph or revive its earlier umbrella scopes.

This is a planning proposal, not the task catalog. Do not execute these work streams or treat their labels as task IDs. Final package creation waits for the coordinator's reconciliation of historical early/middle/late coverage and the parity synthesis. The final catalog belongs at `refactoring-tasks/terminal-components/completion/NNN`; every row below must become one or more canonical task packages with exact evidence and scenario membership.

## Authoritative inputs and schema

The immutable UI oracle is `02f5294bfdbf38004cc49130d0aff1d01f31434c`. The architecture candidate is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Current inputs are the historical obligation ledger, architecture matrix, 54-family component matrix, and the Showcase/Holla/Jackin/TablePro documents and scenario registers. They are factual planning evidence, not already captured oracle frames or successful parity tests.

On 2026-09-11, local `donbeave/task-format` HEAD, `git ls-remote origin refs/heads/main`, and installed `taskfmt --version` all identified `52d9f1eb7721f409bc47beb9fced7997b5c13ede`. The planner directly inspected the repository README, AGENTS, full `docs/monitoring.md`, canonical task-template README/AGENTS/verify.toml, `harness/src/verifycfg.rs`, README frontmatter parser, gate contract, and trusted-input dispatch implementation. The current formats are `task/v5`, `verify/v2`, and `task-meta/v1`.

Canonical package constraints:

- README frontmatter has exactly `schema`, `id`, `title`, `kind`; dependencies do not belong there.
- `task.toml` has exactly `schema`, `status`, `dependencies`. Use fully qualified `terminal-components/completion/NNN` dependencies. Every execution package starts `pending`, never `draft` to bypass readiness.
- `verify.toml` declares nonempty narrow `writable_paths`, optional `forbidden_paths`/`forbidden_patterns`, and ordered checks with exactly one `phase = "gate"`. Every check uses one `argv` or `shell`, real expected results, and requirement/acceptance references.
- No verification command invokes `taskfmt verify` recursively. No `true`, success-printing placeholder, source-presence-only assertion, or listing-only command proves implemented behavior.
- `base_tree` cannot predict a future predecessor tree. `predecessor` can identify one task; it cannot encode the whole DAG. Multi-parent code ancestry needs an explicit host integration contract in addition to `task.toml` readiness.
- Non-gate acceptance blocks have one constrained Given/When/Then fence and typed `Verification` with `Covers` and `Check`. The gate block has no Gherkin or Covers. Commands belong only in verify.toml.

Evidence IDs require namespaces. `HIST:A01` and `ARCH:A01` are different obligations. Use `COMP:text-viewport` and `APP:HO-OUTPUT-SCROLL`, for example. The final traceability join must reject unqualified ambiguous IDs, missing IDs, duplicate ownership rows, and orphan requirements/scenarios. One primary correction owner is required for each obligation; baseline, non-regression, and integration owners are additional explicit roles. The central `traceability.tsv` columns are `source_namespace`, `source_id`, `task_id`, `requirement_id`, `acceptance_id`, `check_id`. This is an evidence join, not a replacement task schema. Rejected/deferred decisions still map to a forbidden-path/non-goal invariant or an evidence-backed disposition check; they are not silently dropped.

## Dependency types and branch materialization

Continue on an integration branch rooted at pinned main, subject to the coordinator's final topology adjudication. Preserve main's workspace, public facade, runtime, borrowed component model, and valuable safety tests. Transplant oracle domain fixtures and behavior through the accepted APIs. Do not merge either whole branch into the other or recreate root `src/widgets` under an application.

Three dependencies must not be conflated:

1. **Verification dependency:** the prerequisite task has a passing authoritative host verdict on its frozen candidate tree.
2. **Code dependency:** that exact tested change is integrated in the downstream starting tree. A monitor status of `done` does not establish this.
3. **Trust dependency:** the downstream host possesses the independently accepted verifier/reference bundle, with pinned source and hashes. A candidate-created success file does not establish this.

Task-format's monitor does not promote or push candidate code. Its documentation explicitly warns that a verified predecessor alone does not prepare the next repository baseline. More critically, the current dispatcher always clones `main` (`harness/src/cmds/run.rs:166`; `ops/git.rs:105`) and promotion always pushes `refs/heads/main` (`cmds/promote.rs:66`; `ops/git.rs:247`). There is no branch-selection setting that can safely point those operations at an integration branch in the original repository.

The chosen execution mode is the supported standalone verifier on isolated checkouts, with a trusted host coordinating the DAG and exact-tree integration. `taskfmt verify --help` exposes `--root`, `--task-dir`, `--base`, `--progress`, and `--log-dir`. Direct source inspection confirms `cmds/verify.rs:18` sets `enforce_task_contract: true`; `gate.rs:251` enforces package lint, scope, forbidden paths/patterns, every declared check and progress. It imposes no branch name. This preserves the canonical contract and completion gate without creating another remote or changing taskfmt.

The later host runner must implement this exact boundary:

1. Read the immutable catalog DAG and verify prerequisite host verdicts plus their actual integrated commit/tree ancestry. Materialize a fresh candidate checkout at the recorded integration parent and overlay the trusted inputs before recording the scope base. Capture original parent and overlay tree separately.
2. Give the executor only the candidate checkout, immutable task contract/trusted inputs and separate progress file. Executor-run `taskfmt verify` is feedback; host verification decides acceptance.
3. After execution, freeze the complete candidate tree, including relevant untracked files, into an immutable object. Reject hidden index flags and unsafe symlinks. Create a fresh host verification checkout of that exact object; the executor cannot mutate it while gates run.
4. Invoke the pinned `taskfmt verify` with explicit root, external read-only task directory, recorded scope base, progress file and separate log directory. Bind stdout, exit, check logs, task/trust digests, source tree, prerequisite evidence and command/tool fingerprints in host-owned evidence. Require exit 0 and final line `DONE`.
5. Verify that checks did not alter source/trusted inputs. Create the integration commit from that exact tested tree and recorded integration parent, with DCO signoff and required coauthor trailer. Advance only the named integration ref with an expected-parent lease. If its parent changed, stop and re-integrate/reverify; never replace the parent of an already gated tree silently.
6. Expose the new verified integration head as the next task base. At a parallel join, combine only accepted sibling changes in a clean host workspace, preserve all scope ownership, and rerun the union of impacted contracts plus build gates on the exact combined tree before exposing it downstream.

This mode uses taskfmt's catalog schema and standalone gate, not its monitor execution/promotion lifecycle. Standalone verification does not authorize writing monitor `done` status or fabricating run manifests. The host's external execution ledger records progress and accepted dependency evidence; task metadata remains the immutable dependency specification. B-HARNESS must implement and independently qualify the freeze/verification/integration runner before downstream dispatch. Monitor run/group/project and `taskfmt promote` are prohibited for this campaign because they hardcode original repository main when pointed there. A separate staging repository would be necessary only if a later instruction specifically required those monitor lifecycle APIs; it is not needed for the chosen supported gate mode.

Never cherry-pick blindly from an unverified working tree, integrate against a changed parent, or equate individually passing siblings with a passing combined tree. No execution runner, branch promotion, or remote creation runs during this planning session.

The initial execution precondition records the pinned main ancestor, current integration parent, clean candidate base, required upstream tool revision, full taskfmt fingerprint, and trust-root identity. Downstream preconditions query host-owned run evidence and verify prerequisite commit/tree ancestry in the actual starting Git tree. There is no legitimate precondition that merely checks a marker an executor can write.

## Verification bootstrap and trust contract

The planner must settle the proof implementation and authority before marking packages execution-ready. The following responsibilities are separate even when one trusted runner implements their entry points.

| Responsibility | Implementation authority | Required proof | Downstream trust boundary |
| --- | --- | --- | --- |
| Tool qualification | Existing qualified `tui-snap` source plus actual reviewed external PR; coordinator records final commit | Existing and new frame, cursor, ANSI, PTY and store tests; mutation rejection; exact source/tree/lock pins | Tool binary/source and lock excluded from every terminal-components implementation task's writable paths |
| Oracle adapter and scenario expansion | Baseline preparation tasks may implement only reviewed adapters/orchestration in disposable oracle copies; canonical requirement set comes from planner | Original tests still pass; adaptation changes observation/time/input orchestration only; repeat independent flat replay equals first capture | An independent review qualifies the adapter; candidate code cannot select oracle actions, coordinates, expected frames, or normalization |
| Exact comparator, required-set validator, provenance verifier | Planner/trusted harness package; any initial implementation task is tested by separate immutable fixtures and negative probes | Each specified cell/state/provenance mutation fails; valid independent evidence passes; missing inputs fail | Executable verifier, manifests, font/profile, environment policy, and reference commit protected together, not only expected snapshots |
| Oracle bundle publication | Host outside executor writable scope after independent qualification | Oracle-source rebuild, deterministic repeat equality, complete expected scenario/checkpoint set, cryptographic artifact map | Host seals approved bundle; downstream package references a content-addressed, independently verified trust root |
| Candidate extraction seam | Each canonical production task explicitly owns untrusted extraction-only code in its already writable source/completion tests; TASK-002–006 own protected observation schema/identity mapping; TASK-070 owns runner qualification | Same production handler/render path and variable real state; independent wrong-state, constant-state, omission, source/binary swap and test-only-path mutants fail | Seam never becomes judge or accepted reference adapter; trusted runner builds frozen candidate and compares against independently sealed schema/reference. No unowned candidate-adapter receipt is required |
| Architecture gates | Planner-owned negative fixtures plus production checks implemented in bounded gate tasks | Renamed copies, dead calls, inert calls, and paint-over mutants fail while legitimate author examples and rain renderer pass | Final scope excludes gate/allowlist/fixture edits from component and app repair tasks |

The complete oracle bundle does not exist yet. Baseline tasks cannot honestly contain its future SHA-256. The final plan must use a concrete host sealing procedure whose immutable implementation independently rebuilds the pinned oracle and publishes a deterministic content-addressed manifest. It must not pretend the future hash is known, allow executors to update package inputs, or use editable candidate manifests as approval authority. If packages use preinstalled shared proof infrastructure, its exact installation command, revision and precondition must be specified. If infrastructure is produced by a prerequisite, its code/output and independent acceptance are explicit DAG products; downstream references must resolve to that accepted product, not a mutable branch path.

The verifier command contract must be finalized before package generation. Required operations are: preflight of tool/source/trust/prerequisite ancestry; required-set expansion validation; reference rebuild/capture; repeat equality; candidate production-view capture; candidate PTY replay; exact canonical frame/semantic comparison; negative mutation qualification; architecture ownership; executed historical-test inventory; and final closure. Each operation needs a real implementation or named prerequisite implementation authority. Specifying a future command without its implementation owner is not execution-ready.

Existing deterministic Rust commands remain real checks. For example, `cargo test --locked -p junie-tui --test keyboard_editor`, `cargo test --locked -p tablepro`, and `cargo run --locked -p xtask -- boundary` are concrete existing commands. They cannot substitute for missing new-oracle acceptance. `boundary` currently fails its capture/parity evidence checks in a clean main checkout; the evidence preparation/portability owner must close those exact defects rather than turn off those checks.

## Proposed bounded outcomes

Labels below are temporary decomposition keys. They intentionally do not assign final task numbers while evidence reconciliation continues. Every component outcome includes its explicit relevant oracle scenario set and preserved architecture tests. A component whose implementation already appears complete still receives bounded preservation proof; its owner must not invent a rewrite to justify the task.

### Baseline and execution foundation

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| B-HARNESS | Qualified tool, trusted exact comparator/provenance validator, concrete host promotion/sealing protocol, and common direct/PTY runner pass independent positive and negative fixtures | Actual external tool PR and final reviewed tool revision; reconciled required-set definitions |
| B-SHOWCASE | Every SC scenario and 23-page expansion captures repeatable oracle events, cells, cursor and semantics; exact clock adapter boundaries qualified | B-HARNESS |
| B-HOLLA | All 34 oracle worlds, HP/route/sweep cases, CLI/lifecycle and private semantic assertions captured with frozen numeric events | B-HARNESS |
| B-JACKIN | Eight worlds, every route/overlay/config/capsule scenario, motion phases and modeled-only cases captured in their honest lanes | B-HARNESS |
| B-TABLEPRO | All TP fixtures/action routes, palette/resize/clock boundaries and domain observations captured | B-HARNESS |
| B-COMPONENTS | Every 54-family row has baseline or evidence-backed non-applicability, common state axes, geometry and semantic proof; all app bundles sealed | B-SHOWCASE, B-HOLLA, B-JACKIN, B-TABLEPRO |
| B-INVENTORY | Exact historical and oracle test identities/profile requirements reconciled; required inventory approved; moved cases retain identity through a relocation map | B-HARNESS; final historical ledger |
| B-TEST-DISPOSITION | Every current test/baseline has a frozen compatible, oracle-conflicting or obsolete disposition; new oracle expectations and staged failure accounting replace conflicting candidate product assertions without weakening immutable archives | B-INVENTORY, B-COMPONENTS |

No component/application implementation starts before B-COMPONENTS and B-TEST-DISPOSITION complete. This deliberately closes the full before/after contract before repair. Every component row below additionally depends on B-TEST-DISPOSITION; that shared prerequisite is omitted from table cells only for readability, never from final task.toml. Independent oracle capture work can run in parallel because it modifies separate adapter modules and artifact namespaces; B-HARNESS alone owns shared harness schema/dispatcher/lock files.

## Current test conflicts and coherent intermediate gates

The refactor cannot preserve a candidate product expectation that contradicts the immutable oracle. Known examples include Holla Ctrl+A activity cycling, scope-before-query Escape and frame-as-milliseconds; Showcase's 22-page/older-oracle matrix; TablePro Surface-only and changed-confirmation expectations; and changed Meter baselines. Preserve compatible architecture, ownership and safety assertions. Preserve old immutable artifacts as historical evidence, with their original provenance. Neither category licenses treating obsolete candidate hashes as current UI acceptance.

B-TEST-DISPOSITION creates a source-qualified conflict register before production repair. Each entry contains exact test identity/profile, old expectation and producing SHA, oracle replacement evidence, historical rationale, architectural assertions retained, primary correction task, and exact replacement check/scenario IDs. Baseline changes require independently captured oracle content and trusted review. Component/application tasks have no writable authority over expected snapshots, baseline loaders, conflict membership or comparison rules.

The transition has three explicit categories:

- Compatible existing tests remain active and must pass in every relevant intermediate task. No blanket permission to rewrite existing tests follows from a product conflict elsewhere.
- Oracle-conflicting product tests are replaced by trusted oracle assertions through the preparation task. Their former bytes remain archived with exact relocation/disposition records. Their oracle replacements execute from the start; a failure is recorded as unresolved migration state owned by a specific future task, not a passing assertion or skipped test.
- Obsolete architectural paths receive explicit historical disposition and current replacement proof. Retired clone benchmarking must not revive cloning. A rejected future feature remains a non-goal invariant rather than a silently dropped requirement.

Each task's green acceptance gate requires all its owned and prerequisite oracle scenarios, all affected compatible regressions, whole-workspace compile/format/lint gates, and the exact source-qualified test accounting check. An additional full diagnostic test sweep records unresolved future-owner failures. The accounting gate fails on any unregistered failure, absent test, filtered/skipped execution, changed expected failure identity, or regression of a previously closed scenario. It may recognize only the immutable, task-stage-specific set of still-unfinished future obligations; it cannot claim those failures passed. A newly fixed future scenario is recorded and thereafter cannot regress.

Do not put unconditional `cargo test -p holla` success in H-SHELL while later Holla route tasks still own known oracle failures. Use real focused checks plus the complete executed accounting report, then require the unfiltered full package suite at H-CLOSE. The same rule applies to the other app chains. At X-FINAL the allowed unresolved set is empty and unfiltered full workspace tests are mandatory. No test is ignored, filtered out, deleted to make a gate pass, or left as an expected failure at completion. Every intermediate tree remains buildable and testable; known migration failures are transparent evidence, never a parity pass claim.

### Reusable runtime and component outcomes

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| C-SESSION | Terminal lifecycle, input normalization, color ceiling, monotonic clock/feedback and scheduler preserve exact executable behavior and backend-free boundaries | B-COMPONENTS |
| C-RUNTIME | Real focus, hit/capture, disabled ownership, typing, pending-input publication and resize freshness match oracle while retaining accepted runtime invariants; include disabling an already captured top owner | C-SESSION |
| C-THEME | Semantic paint/override/capability conversion and glyph policies reproduce oracle cells without RGB inference or lost authored provenance; preserve existing ColorLevel::narrow_to and do not introduce Ord merely to match stale prose | B-COMPONENTS |
| C-LAYOUT | Shared allocation, clipping, zero/tiny/nonzero-origin geometry and split seams preserve oracle positions and pointer regions | B-COMPONENTS |
| C-TEXT | Complete logical graphemes cross style boundaries; tab/control display/copy and editing modifier grammar preserve oracle behavior through shared text core | B-COMPONENTS |
| C-SCROLL | One ScrollRegion boundary/thumb model and reusable hidden-edge fade reproduce exact blends, exclusions and routing at all required heights | C-RUNTIME, C-THEME, C-LAYOUT |
| C-ROWS | RowUi/ColumnsUi/custom part overrides propagate all accepted borrowed precedence channels; dynamic keyed identity and visible-only work proven | C-THEME, C-TEXT |
| C-FIELDS | TextInput/TextArea/Field/FieldControl restore navigation versus edit, completed-click caret, selection and secrets/validation lifecycle | C-RUNTIME, C-TEXT, C-SCROLL |
| C-LAYER-CORE | Runtime layer anchor/backdrop/modal stack preserves dismissal/capture/focus lifetime without depending on concrete Form/Dialog wrappers | C-RUNTIME, C-LAYOUT |
| C-CHOICES | Checkbox/Toggle/RadioGroup/Select preserve controlled value, chosen versus cursor, popup behavior and mono pressed-label geometry | C-FIELDS, C-ROWS, C-LAYER-CORE |
| C-FORM | Declared Form fields/sections/pairs preserve traversal, hidden drafts, commit-validation-submit order and failure focus | C-CHOICES |
| C-COLLECTIONS | List/NavList/Tree/Steps/ChipBar/Tabs preserve keyed selection, branch policies, live rows, fades and close/reorder behavior | C-ROWS, C-SCROLL |
| C-OUTPUT | TextViewport absorbs ScrollPanel output/prose policy, keyboard selection/marks/find composition, retention rebasing, cache invalidation and append work | C-TEXT, C-SCROLL |
| C-GRID | Generic Grid row/cell modes preserve Table/DataTable capability set, completed-click editing, ragged cell hooks, keyed sorting and exact scrolling | C-FIELDS, C-ROWS, C-SCROLL |
| C-LAYERS | Dialog/Menu/ContextMenu/MenuBar preserve measured body/facts, submenu state, exact dismissal, top input ownership and focus restoration through established runtime layers | C-LAYER-CORE, C-FORM |
| C-PICKERS | FilterList/Picker/PickerChain/CommandPalette/Completion preserve borrowed source identity, query, unavailable/error/back and editor-popup routing | C-COLLECTIONS, C-LAYERS, C-FIELDS |
| C-CODE | CodeEditor restores readonly caret/selection, highlighting/diagnostics, find/completion and editor-scroll/cursor trajectories | C-FIELDS, C-PICKERS, C-OUTPUT |
| C-DIFF | DiffView preserves unified/review/narrow fallback, projection budget, tab alignment, selection/copy and mode restoration | C-OUTPUT, C-LAYOUT |
| C-CHROME | Button/Brand/Panel/Props/EmptyState/TooSmall preserve static/action chrome, borrowed rich values, owner inheritance and tiny geometry through public components | C-THEME, C-LAYOUT, C-RUNTIME, C-SCROLL |
| C-STATUS | StatusBar/HintBar/KeyHint preserve live item hover, group priorities, truncation, action text/chords and effective binding ownership | C-CHROME |
| C-MOTION | Progress/Spinner/Meter preserve exact state glyphs, phase boundaries, fill/rest thresholds and authored semantics; Meter's historically changed snapshots cannot serve as new-oracle proof | C-THEME, C-SESSION |
| C-DISCOVERY | Binding metadata drives HelpOverlay, menus and hints through the same effective owner/action resolver; Wizard retains state/focus without domain execution | C-PICKERS, C-STATUS |
| C-CONFORMANCE | Generated component registry drives exact PARTS equality and component-versus-row attribution across real states; missing Probe/Dialog/Props/PropsList and unsupported extras fail | All component rows above |

C-LAYER-CORE precedes Select/Form; concrete Dialog/Menu wrappers follow Form. This removes the Select/Form/Dialog dependency cycle without inventing a new layer abstraction. C-CONFORMANCE's attribution observation belongs in existing StyledQuery/conformance infrastructure; it does not authorize a future `patch_part` hook that history never accepted. Completed source tests are preserved, but historical green results cannot close newly demonstrated captured-owner, props-scanner, or registry holes.

### Showcase restoration

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| S-SHELL | Live public shell/chrome/navigation serves 23 pages with exact sidebar thresholds, help/inspector/focus/persistence and no compatibility paint-over | C-COLLECTIONS, C-CHROME, C-STATUS, C-LAYERS, C-DISCOVERY |
| S-FIELDS | All enabled Inputs fields, complete Forms reviewer/reset/submit, TextAreas and Buttons matrix are live; expired Buttons example packaging resolved | S-SHELL, C-FORM |
| S-DATA | Customer Grid model/queue/SQL/row effects, Tables and Editable Tables use visible data as actual interaction model | S-FIELDS, C-GRID |
| S-SETTINGS | General, Members and Environment provide original editing/save/dirty/selection/prompt flows | S-DATA, C-PICKERS |
| S-OUTPUT | Missing Diff page plus editor/panels/scrolling/terminal/task runner/progress journeys use shared output/editor/diff components | S-SETTINGS, C-CODE, C-DIFF, C-MOTION |
| S-COLLECTIONS | Lists/Trees/Sidebars/Chips and Overview preserve all enumerated page actions and default geometry through live reusable controls | S-OUTPUT |
| S-OVERLAYS | Pickers/Dialogs/Chrome preserve real modal/menu/action journeys and exact outside/focus behavior | S-COLLECTIONS |
| S-CLOSE | Full SC direct/PTY inventory passes on one app tree; no inert replacement control survives; validation-only scope | S-OVERLAYS |

These serialize shared `apps/showcase/src/app.rs`, page registry and app tests. Component fixes remain owned by C tasks; an app task cannot patch shared internals outside its contract. S-CLOSE cannot absorb unidentified earlier defects.

### Holla restoration

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| H-SHELL | Permanent Here/page stack, persistent activity/plan tabs, CLI frame semantics, exact keys, menus and restoration work on main's App/Cx/Ui | C-COLLECTIONS, C-CHROME, C-DISCOVERY |
| H-FINDER | Grouped finder/discovery, typed query/scopes, split/drawer previews, alternatives/pin/alias/hide and usage worlds match oracle | H-SHELL, C-PICKERS, C-OUTPUT |
| H-FILES | Files index/browser/jump/preview/resource actions use exact virtual filesystem and shared view controls | H-FINDER, C-DIFF |
| H-REVIEW | Arguments, trust, plan review and gate1/gate2 restore exact UI while preserving one-shot/current-target checks | H-FILES, C-FORM, C-LAYERS |
| H-ACTIVITY | Output streams/find/selection/stdin/EOF/cancellation and executing-plan output/tab lifetime match oracle World ownership | H-REVIEW, C-CODE |
| H-DISK | Disk scan/tree/Spotlight/insight categories preserve stable identity, facts, progress and platform guards | H-ACTIVITY |
| H-CLEANUP | Trash/permanent/dry-run review, ownership, target validation, results/reports/history and quit settlement match simulated oracle | H-DISK |
| H-BUILD-TASKS | Git current/batch, native task adapters, Cargo, Gradle and IDEA fixture actions preserve exact membership/argv/cwd/outcome/error worlds | H-CLEANUP |
| H-SERVICES | Docker/Compose/Brew/PG/SSH/monitor and platform snapshot routes preserve exact target/command/failure/attachment outcomes | H-BUILD-TASKS |
| H-CONFIG-UPGRADE | Custom config/trust diagnostics, upgrade manager plans and every remaining emitted snapshot/report parameter route preserve exact oracle finite outcomes | H-SERVICES |
| H-CLOSE | All HO expansions, 34 worlds, lifecycle/resize/color and previously existing useful main safety tests pass together; validation-only scope | H-CONFIG-UPGRADE |

No real provider operations or deferred HP16 CLI expansion belong here. Holla's shared app router/world/catalog files serialize these tasks unless the final plan extracts a behavior-preserving module boundary as an explicit part of H-SHELL.

### Jackin restoration

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| J-SHELL | Rituals, manager topology, host routing/menus/hints, selection/detail and prelude preserve oracle routes and legitimate rain art | C-COLLECTIONS, C-CHROME, C-DISCOVERY, C-MOTION |
| J-CONFIG | Editor/Settings and shared mounts/environments/roles/accounts configuration preserve forms, dirty state, save preview/error/retry and secret ownership | J-SHELL, C-FORM, C-GRID |
| J-ACCOUNTS | Accounts/Usage and 1Password loading/error/back/retry/secret flows preserve world jobs and shared picker/dialog ownership | J-CONFIG, C-PICKERS |
| J-LAUNCH | Cockpit stage/log/credentials/failure/cancel/handoff and exact motion/clock transitions match oracle | J-ACCOUNTS, C-OUTPUT |
| J-CAPSULE | Live pane/tab topology, prefix keys, selection/copy, split resize, rename/close/takeover and output retention preserve oracle | J-LAUNCH, C-CODE |
| J-INSPECT | Compact/advanced inspect Diff and nested capsule/dirty-exit/export/keep/purge dialogs preserve exact state and focus | J-CAPSULE, C-DIFF |
| J-CLOSE | All JA scenarios, world/motion/color axes, lifecycle and no-clone performance obligations pass together | J-INSPECT |

### TablePro restoration

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| T-CONNECT | Connection tree/filter/actions/forms/test/retry/save/duplicate/delete and shell identity strip use live controls | C-FORM, C-COLLECTIONS, C-CHROME, C-STATUS |
| T-WORKBENCH | Explorer lazy/filter/preview/promote, tabs/drawers/maximize/empty and complete dispatched keymap match oracle | T-CONNECT, C-PICKERS |
| T-DATA | Live grid editing/typed cells/pending SQL/undo/filter operators/chips/Structure modes replace decorative controls | T-WORKBENCH, C-GRID |
| T-QUERY | CodeEditor and editor/result SplitPane restore completion/statement execution/results/Explain/error/diagnostics/cancellation | T-DATA, C-CODE |
| T-SAFETY | Every declared overlay is mounted; six safety modes/save/discard/dirty ownership preserve exact gates and current target identity | T-QUERY, C-LAYERS |
| T-HISTORY | History scope/search/failed/list/detail/copy/open/rerun and switcher/help/tab-list journeys use real handlers/components | T-SAFETY |
| T-CLOSE | All TP scenarios, CLI/NO_COLOR/resize/clock/lifecycle and compatible existing stable-identity/safety tests pass together | T-HISTORY |

### Architectural closure and final integration

| Key | Observable outcome and principal scope | Hard prerequisites |
| --- | --- | --- |
| X-OWNERSHIP | Generic visible pixels and interaction have the same reusable production owner; forbidden legacy imports/copies/paint-over and method-based props-helper loopholes rejected with negative fixtures | All component tasks, S-CLOSE, H-CLOSE, J-CLOSE, T-CLOSE |
| X-TESTS | Exact historical/oracle executions and qualified scenario identities survive Buttons/render-target relocations; no unresolved required inventory entries | B-INVENTORY, all app closures |
| X-PERF | Real Showcase style share measurement and all preserved allocation/byte/dirty-append/visible-work counters meet accepted thresholds | All app closures; C-THEME, C-OUTPUT, C-COLLECTIONS |
| X-API | Public facade/author keyed example/docs/registry/examples/backend-free boundaries complete; doc-check covers §§18–20 and real signatures; obsolete deferrals and clone name reconciled; future semver baseline prepared | X-OWNERSHIP, X-TESTS, X-PERF |
| X-FINAL | Tested integration head passes whole required oracle direct/PTY set, authoritative stable/MSRV/build/lint/docs/API/performance gates, trust immutability and ancestry/merge-readiness checks | X-API and every remaining leaf |

X-FINAL is validation only. It does not grant source-fix scope, reference blessing, publishing or merge authority. A failure goes to its exact owner; if a new obligation is discovered, the planner repairs the catalog and reruns review before execution resumes.

## Shared-file serialization and parallelism

Final packages use file ownership, not broad component directory ownership, to permit real concurrency. An immutable conflict report derives every pair of overlapping writable paths and requires an ordering edge or explicit independent-worktree integration strategy.

- Shared `Cargo.toml`, `Cargo.lock`, crate `lib.rs`, component `mod.rs`, `author.rs`, common test registries, `xtask/src/main.rs`, and CI workflows have a single declared writer at a time. A task needing one of these paths carries the corresponding hard ordering edge.
- C-RUNTIME owns runtime/focus/hit/capture/session changes; layer component tasks consume its public contract. C-THEME owns common paint style/token changes; C-ROWS owns row override plumbing and consumer adaptation. Keep changes to a consumer's same file ordered behind C-ROWS.
- C-TEXT precedes all editor/output consumers. C-SCROLL defines the reusable fade contract before any consumer adds fades. C-OUTPUT precedes Diff and Code integration. Do not duplicate missing shared features in an app task to unblock that app.
- App work may run in parallel across four application directories after reusable prerequisites. Within each app, default to the declared serial chain because app routing/state files and same-package tests overlap. Shared public API additions must finish before that parallel wave.
- X-TESTS owns final render-test consolidation and associated CI invocation changes. X-API follows it for documentation/named-test cleanup. X-PERF may run beside X-TESTS only when its exact benchmark files do not overlap the relocation.
- Independent workers use separate worktrees and target/artifact directories. Output directories and trust-bundle namespaces include source tree and task identity to prevent artifact collisions.

## Structural critical path and completion rules

No durations have been measured, so a duration-weighted critical path cannot honestly be claimed. The structural path first runs B-HARNESS, the slowest complete app capture, B-COMPONENTS, then the longest reusable chain. Under this proposal that chain is session/runtime, scroll, fields, choices/form, concrete layers, pickers, discovery, then Holla's serial restoration. All app chains begin only once their actual prerequisites are integrated; Holla has the longest proposed serial reconstruction because its 11 main worlds omit 23 oracle worlds and whole route families. The final join is app closure, ownership/test/performance closure, API closure, then X-FINAL.

Final generation must compute a real topological order and longest dependency path from task.toml rather than copying this prose. Record all equally long structural paths and distinguish dependency depth from elapsed-time predictions. Independent standalone candidate checkouts may execute in parallel; integration ref updates always serialize and joins must be tested explicitly.

The final graph is ready only when every proposed umbrella split is resolved; every HIST/ARCH/COMP/APP row has exact canonical owners; every AC names runnable proof and protected input authority; all taskfmt lint checks pass; shared writable-path conflicts are accounted for; host promotion/trust materialization is concrete; and independent reviewers cannot identify a material uncovered obligation. The proposal itself does not satisfy those gates.
