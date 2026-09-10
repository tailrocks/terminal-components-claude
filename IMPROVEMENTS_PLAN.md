# Improvements plan

Status: **task split ready; implementation not yet verified.** This document is
the checkbox tracker. Detailed contracts live in [small task files](docs/improvements/).
The current phase implements Holla's simulated design, interactions, coverage
and the shared components needed by those flows. Other work remains tracked.

Read [current scope and delivery order](docs/improvements/scope.md),
[shared acceptance contracts](docs/improvements/shared-contracts.md) and the
[ready-to-use goal prompt](docs/improvements/goal-prompt.md).
Historical research and evidence remain in the [audit reference](docs/improvements-plan-reference.md).

## Tracking rules

A checkbox means the named phase/slice passed its implementation, tests and
required inspected visual evidence. It does not mean the entire original task
or production capability is complete. Update the linked task checklist and
Evidence section with each tracker change. Unverified existing behavior stays
unchecked until tested. Use an unchecked row plus an In progress/Blocked note
while working; keep deferred requirements visible.

Task files retain the full original findings and acceptance criteria. Scope.md
controls what executes now. Minimal caller migrations and regression checks for
a changed shared invariant are current work; unrelated application features are not.

## Current — existing Holla design

- [ ] [H00 — Existing Holla design and journey coverage](docs/improvements/h00-existing-holla-design-and-coverage.md)

Start its inventory first; finish it after the current slices integrate.

## Current — shared components and Holla coverage

- [ ] [F02 — Picker eligibility and Holla target identity](docs/improvements/f02-picker-action-eligibility-and-destructive-target-identity.md#current-phase) — Holla/shared slice only.
- [ ] [F05 — TreeView identity for Holla file and disk trees](docs/improvements/f05-preserve-treeview-target-across-lazy-insertion.md#current-phase).
- [ ] [F08b — Picker Unicode editing and paste](docs/improvements/f08b-picker-grapheme-editing-query-paste.md#current-phase) — Holla/shared slice only.
- [ ] [F08d — Holla shortcut ownership and hints](docs/improvements/f08d-unassigned-modifiers-perform-plain-actions.md#current-phase) — Holla/shared slice only.
- [ ] [F10 — Exact styled Unicode text](docs/improvements/f10-segment-logical-text-before-applying-styles.md#current-phase).
- [ ] [F11 — Bound Holla output retention](docs/improvements/f11-one-retention-aware-mutation-boundary.md#current-phase).
- [ ] [F12 — Preserve selection and reading position](docs/improvements/f12-rebase-retained-identity-never-silently-retarget-selection.md#current-phase).
- [ ] [F13 — Keep rendered content fresh](docs/improvements/f13-make-cache-validity-part-of-content-ownership.md#current-phase).
- [ ] [F14 — Consistent text display and copy policy](docs/improvements/f14-explicit-tab-control-geometry-and-copy-policy.md#current-phase).
- [ ] [F15 — Incremental Holla output rendering](docs/improvements/f15-incremental-live-output-work-measured-separately-from-idle-redraw.md#current-phase).
- [ ] [F19 — Keyboard selection and copy](docs/improvements/f19-keyboard-selection-in-the-existing-textviewport.md#current-phase).
- [ ] [F20 — Search Holla output and previews](docs/improvements/f20-find-in-read-only-output-through-existing-composition.md#current-phase) — Holla/shared slice only.
- [ ] [F21 — Restore the shared terminal session](docs/improvements/f21-restore-terminal-state-for-supported-job-control-suspension.md#current-phase).
- [ ] [F22 — Trustworthy Holla visual captures](docs/improvements/f22-separate-rasterizer-fidelity-from-terminal-correctness.md#current-phase).
- [ ] [F23a — Holla state reachability coverage](docs/improvements/f23a-state-reachability-design-d4.md#current-phase) — Holla/shared slice only.
- [ ] [F23b — Shared terminal lifecycle proof](docs/improvements/f23b-lifecycle-v01-v02.md#current-phase).
- [ ] [F23c — Holla event freshness and resize](docs/improvements/f23c-event-freshness-resize-v03.md#current-phase) — Holla/shared slice only.
- [ ] [F23d — Holla input fairness](docs/improvements/f23d-input-flood-fairness-v04.md#current-phase).
- [ ] [F23e — Holla rendering performance proof](docs/improvements/f23e-dirty-idle-performance-v05.md#current-phase) — Holla/shared slice only.
- [ ] [F23f — Holla color and terminal proof](docs/improvements/f23f-rendering-color-compatibility-v06-v07.md#current-phase) — Holla/shared slice only.
- [ ] [F23g — Reproducible Holla evidence](docs/improvements/f23g-evidence-provenance-v08.md#current-phase) — Holla/shared slice only.
- [ ] [F23h — Holla facts paste and affected input regressions](docs/improvements/f23h-precise-subsidiary-regressions-v09.md#current-phase) — Holla/shared slice only.

## Current — Holla simulated UI

<a id="existing-holla-functional-parity-audit"></a>

Each checkbox closes only the simulated UI slice. All operational and non-UI
remainders stay under Later. The [capability matrix](docs/holla-parity-matrix.md)
still defines the full row contracts; partial representation is not Covered.

<a id="hp01"></a>

- [ ] [HP01 — Discovery and stable actions](docs/improvements/hp01-adaptive-discovery-and-stable-action-identity.md#current-phase--simulated-representation) — simulation only.

<a id="hp02"></a>

- [ ] [HP02 — Search and remembered choices](docs/improvements/hp02-search-query-editing-and-durable-usage-learning.md#current-phase--simulated-representation) — simulation only.

<a id="hp03"></a>

- [ ] [HP03 — File search and resource actions](docs/improvements/hp03-find-files-and-perform-resource-actions.md#current-phase--simulated-representation) — simulation only.

<a id="hp04"></a>

- [ ] [HP04 — Folder browser and safe previews](docs/improvements/hp04-browse-folders-and-safely-preview-files.md#current-phase--simulated-representation) — simulation only.

<a id="hp05"></a>

- [ ] [HP05 — Git status, pull and push](docs/improvements/hp05-current-repository-git-operations.md#current-phase--simulated-representation) — simulation only.

<a id="hp06"></a>

- [ ] [HP06 — Git batches and hygiene](docs/improvements/hp06-repository-batches-mirrors-and-hygiene.md#current-phase--simulated-representation) — simulation only.

<a id="hp07"></a>

- [ ] [HP07 — Project-task choices](docs/improvements/hp07-native-project-task-adapters.md#current-phase--simulated-representation) — simulation only.

<a id="hp08"></a>

- [ ] [HP08 — Cargo choices and results](docs/improvements/hp08-cargo-build-test-lint-and-clean.md#current-phase--simulated-representation) — simulation only.

<a id="hp09"></a>

- [ ] [HP09 — Docker and Compose flows](docs/improvements/hp09-docker-and-compose-outcomes.md#current-phase--simulated-representation) — simulation only.

<a id="hp10"></a>

- [ ] [HP10 — Homebrew service controls](docs/improvements/hp10-homebrew-service-lifecycle.md#current-phase--simulated-representation) — simulation only.

<a id="hp11"></a>

- [ ] [HP11 — Gradle tasks and cleanup review](docs/improvements/hp11-gradle-tasks-daemon-and-recursive-cleanup.md#current-phase--simulated-representation) — simulation only.

<a id="hp12"></a>

- [ ] [HP12 — IntelliJ cleanup review](docs/improvements/hp12-intellij-metadata-cleanup.md#current-phase--simulated-representation) — simulation only.

<a id="hp13"></a>

- [ ] [HP13 — Upgrade plans and standalone actions](docs/improvements/hp13-all-legacy-upgrade-managers.md#current-phase--simulated-representation) — simulation only.

<a id="hp14"></a>

- [ ] [HP14 — Activity output and truthful results](docs/improvements/hp14-task-execution-output-and-results.md#current-phase--simulated-representation) — simulation only.

<a id="hp15"></a>

- [ ] [HP15 — Prompt input and cancellation UX](docs/improvements/hp15-runtime-input-cancellation-and-terminal-ownership.md#current-phase--simulated-representation) — simulation only.

<a id="hp16"></a>

[HP16 — CLI list/run/doctor](docs/improvements/hp16-cli-list-run-and-doctor-contract.md) is entirely deferred; see Later.

<a id="hp17"></a>

- [ ] [HP17 — Custom-action and trust review](docs/improvements/hp17-custom-actions-configuration-and-trust.md#current-phase--simulated-representation) — simulation only.

<a id="hp18"></a>

- [ ] [HP18 — Disk scan progress and cache states](docs/improvements/hp18-disk-measurement-progress-and-cache.md#current-phase--simulated-representation) — simulation only.

<a id="hp19"></a>

- [ ] [HP19 — Disk tree and top files](docs/improvements/hp19-disk-tree-top-files-and-selection.md#current-phase--simulated-representation) — simulation only.

<a id="hp20"></a>

- [ ] [HP20 — Cleanup categories and eligibility](docs/improvements/hp20-complete-cleanup-insight-taxonomy-and-guards.md#current-phase--simulated-representation) — simulation only.

<a id="hp21"></a>

- [ ] [HP21 — Cleanup gates, dry run and recovery](docs/improvements/hp21-deletion-authorization-dry-run-and-recovery-mode.md#current-phase--simulated-representation) — simulation only.

<a id="hp22"></a>

- [ ] [HP22 — Cleanup progress and reports](docs/improvements/hp22-cleanup-ownership-outcomes-and-operation-log.md#current-phase--simulated-representation) — simulation only.

<a id="hp23"></a>

- [ ] [HP23 — Platform-aware prototype coverage](docs/improvements/hp23-platform-and-terminal-capability-contract.md#current-phase--simulated-representation) — simulation only.

## Later — other applications and unrelated component work

These are preserved open findings, not rejected defects. The current goal does
not execute them unless a specific necessary dependency is demonstrated and
scoped under scope.md.

- [ ] [F01 — Modal-first paste routing](docs/improvements/f01-modal-first-paste-routing.md#later)
- [ ] [F03 — Atomic ListBox mutation](docs/improvements/f03-atomic-listbox-mutation.md#later)
- [ ] [F04 — Reconcile DataGrid read-only transitions before mutation](docs/improvements/f04-reconcile-datagrid-read-only-transitions-before-mutation.md#later)
- [ ] [F06 — Preserve DataGrid cursor identity through local sort](docs/improvements/f06-preserve-datagrid-cursor-identity-through-local-sort.md#later)
- [ ] [F07 — Checked row/schema ingestion](docs/improvements/f07-checked-row-schema-ingestion.md#later)
- [ ] [F08a — Between upper-value paste](docs/improvements/f08a-between-upper-value-paste.md#later)
- [ ] [F08c — Ctrl+Shift+Home/End](docs/improvements/f08c-ctrl-shift-home-end.md#later)
- [ ] [F09 — Deliver Inspect copy without closing the modal](docs/improvements/f09-deliver-inspect-copy-without-closing-the-modal.md#later)
- [ ] [F16 — Reach every showcase section at supported sizes](docs/improvements/f16-reach-every-showcase-section-at-supported-sizes.md#later)
- [ ] [F17 — Empty allocations emit nothing](docs/improvements/f17-empty-allocations-emit-nothing.md#later)
- [ ] [F18 — Distinguish read-only from disabled](docs/improvements/f18-distinguish-read-only-from-disabled.md#later)

## Later — remaining portions of split tasks

- [ ] [F02 — TablePro safe tab-close identity and fallback](docs/improvements/f02-picker-action-eligibility-and-destructive-target-identity.md#later)
- [ ] [F08b — TablePro picker-owner paste integration](docs/improvements/f08b-picker-grapheme-editing-query-paste.md#later)
- [ ] [F08d — Other-application shortcut conformance](docs/improvements/f08d-unassigned-modifiers-perform-plain-actions.md#later)
- [ ] [F20 — Jackin output search and shared-controller reuse](docs/improvements/f20-find-in-read-only-output-through-existing-composition.md#later)
- [ ] [F23a — Full showcase state traversal](docs/improvements/f23a-state-reachability-design-d4.md#later)
- [ ] [F23c — Other-application event and resize journeys](docs/improvements/f23c-event-freshness-resize-v03.md#later)
- [ ] [F23e — Standalone list/table/grid performance coverage](docs/improvements/f23e-dirty-idle-performance-v05.md#later)
- [ ] [F23f — Other-application rendering and platform evidence](docs/improvements/f23f-rendering-color-compatibility-v06-v07.md#later)
- [ ] [F23g — Historical non-Holla evidence coverage](docs/improvements/f23g-evidence-provenance-v08.md#later)
- [ ] [F23h — Standalone editor input, drag and search regressions](docs/improvements/f23h-precise-subsidiary-regressions-v09.md#later)

## Later — Holla production and non-UI integration

Live providers, persistent stores, real process/filesystem ownership, native
platform probes and full CLI parity are not required to close the current goal.
No simulated result closes these checkboxes.

- [ ] [HP01 — Real executable/daemon probes and live discovery adapters](docs/improvements/hp01-adaptive-discovery-and-stable-action-identity.md#later--operational-and-non-ui-integration)
- [ ] [HP02 — Durable usage-store I/O, real restart/migration and concurrent filesystem writer integration](docs/improvements/hp02-search-query-editing-and-durable-usage-learning.md#later--operational-and-non-ui-integration)
- [ ] [HP03 — Real home indexing, OS open/reveal, OSC52/system clipboard transport and worker joins against live providers](docs/improvements/hp03-find-files-and-perform-resource-actions.md#later--operational-and-non-ui-integration)
- [ ] [HP04 — Real filesystem listing/reads, descriptor validation and OS race/nonblocking probes](docs/improvements/hp04-browse-folders-and-safely-preview-files.md#later--operational-and-non-ui-integration)
- [ ] [HP05 — Real git invocation, credentials, remotes and repository integration](docs/improvements/hp05-current-repository-git-operations.md#later--operational-and-non-ui-integration)
- [ ] [HP06 — Real repository discovery, git operations, branch deletion and worktree/remote checks](docs/improvements/hp06-repository-batches-mirrors-and-hygiene.md#later--operational-and-non-ui-integration)
- [ ] [HP07 — Live manifest discovery, runner probes, external task listing and actual project-task execution](docs/improvements/hp07-native-project-task-adapters.md#later--operational-and-non-ui-integration)
- [ ] [HP08 — Actual Cargo process execution, target discovery and cleanup](docs/improvements/hp08-cargo-build-test-lint-and-clean.md#later--operational-and-non-ui-integration)
- [ ] [HP09 — Docker daemon access, real Compose/log streams and destructive container/image/volume operations](docs/improvements/hp09-docker-and-compose-outcomes.md#later--operational-and-non-ui-integration)
- [ ] [HP10 — Real brew services commands and durable service-cache I/O](docs/improvements/hp10-homebrew-service-lifecycle.md#later--operational-and-non-ui-integration)
- [ ] [HP11 — Real Gradle commands, process probes/stops, filesystem traversal and cleanup](docs/improvements/hp11-gradle-tasks-daemon-and-recursive-cleanup.md#later--operational-and-non-ui-integration)
- [ ] [HP12 — Real IDEA/filesystem discovery, Trash executor and operation-log writes](docs/improvements/hp12-intellij-metadata-cleanup.md#later--operational-and-non-ui-integration)
- [ ] [HP13 — Real manager discovery, installation/upgrade commands and host mutation](docs/improvements/hp13-all-legacy-upgrade-managers.md#later--operational-and-non-ui-integration)
- [ ] [HP14 — Production executor/adapters, actual child processes and aggregate/headless CLI contracts](docs/improvements/hp14-task-execution-output-and-results.md#later--operational-and-non-ui-integration)
- [ ] [HP15 — Real task PTYs/stdin, process groups, TERM/KILL escalation, subreaper/reaping and OS-specific process integration](docs/improvements/hp15-runtime-input-cancellation-and-terminal-ownership.md#later--operational-and-non-ui-integration)
- [ ] [HP16 — Full CLI list/run/doctor contract](docs/improvements/hp16-cli-list-run-and-doctor-contract.md#later)
- [ ] [HP17 — Live global/project config reads, persistent approvals, filesystem migration and trust CLI operations](docs/improvements/hp17-custom-actions-configuration-and-trust.md#later--operational-and-non-ui-integration)
- [ ] [HP18 — Live filesystem scanner, OS file metadata/dataless behavior, physical measurements and durable cache I/O](docs/improvements/hp18-disk-measurement-progress-and-cache.md#later--operational-and-non-ui-integration)
- [ ] [HP19 — Live Spotlight queries/stat concurrency/timeouts, real disk traversal and physical post-cleanup scans](docs/improvements/hp19-disk-tree-top-files-and-selection.md#later--operational-and-non-ui-integration)
- [ ] [HP20 — Live artifact discovery, process probes and filesystem sizing](docs/improvements/hp20-complete-cleanup-insight-taxonomy-and-guards.md#later--operational-and-non-ui-integration)
- [ ] [HP21 — Actual canonical filesystem validation, Trash/permanent mutation, pathname-race probes and CLI bypass tests](docs/improvements/hp21-deletion-authorization-dry-run-and-recovery-mode.md#later--operational-and-non-ui-integration)
- [ ] [HP22 — Real cleanup worker ownership, durable JSONL logging and physical capacity/OS integration](docs/improvements/hp22-cleanup-ownership-outcomes-and-operation-log.md#later--operational-and-non-ui-integration)
- [ ] [HP23 — Production OS adapters, native filesystem/Trash/open/process probes on both operating systems and full CLI parity](docs/improvements/hp23-platform-and-terminal-capability-contract.md#later--operational-and-non-ui-integration)

## Conditional — not selected for execution

- [ ] [O04 — matrix paste](docs/improvements/o04-matrix-paste.md#later)
- [ ] [O05 — terminal extensions/OS clipboard](docs/improvements/o05-terminal-extensions-os-clipboard.md#later)
- [ ] [O07 — read-only field APIs](docs/improvements/o07-read-only-field-apis.md#later)

## Removed proposals

Generic widget/theme/glyph, keyed-collection, container/modal and worker
frameworks are retired, not unchecked implementation goals. Their historical
O01/O02/O03/O06 entries remain in the
[audit reference](docs/improvements-plan-reference.md#retired-framework-proposals).
Concrete task ownership, cancellation, stale-result rejection and shutdown
remain current where required by Holla.
