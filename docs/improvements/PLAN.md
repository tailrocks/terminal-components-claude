# Improvements plan

Status: **Current phase implemented and verified (simulated Holla prototype); Later work open.** This document is
the checkbox tracker. Detailed contracts live in [small task files]().
The current phase implements Holla's simulated design, interactions, coverage
and the shared components needed by those flows. Other work remains tracked.

Read [current scope and delivery order](scope.md),
[shared acceptance contracts](shared-contracts.md) and the
[ready-to-use goal prompt](goal-prompt.md).
Historical research and evidence remain in the [audit reference](plan-reference-audit.md).

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

- [x] [H00 — Existing Holla design and journey coverage](h00-existing-holla-design-and-coverage.md)

Start its inventory first; finish it after the current slices integrate.

## Current — shared components and Holla coverage

- [x] [F02 — Picker eligibility and Holla target identity](f02-picker-action-eligibility-and-destructive-target-identity.md#current-phase) — Holla/shared slice only.
- [x] [F05 — TreeView identity for Holla file and disk trees](f05-preserve-treeview-target-across-lazy-insertion.md#current-phase).
- [x] [F08b — Picker Unicode editing and paste](f08b-picker-grapheme-editing-query-paste.md#current-phase) — Holla/shared slice only.
- [x] [F08d — Holla shortcut ownership and hints](f08d-unassigned-modifiers-perform-plain-actions.md#current-phase) — Holla/shared slice only.
- [x] [F10 — Exact styled Unicode text](f10-segment-logical-text-before-applying-styles.md#current-phase).
- [x] [F11 — Bound Holla output retention](f11-one-retention-aware-mutation-boundary.md#current-phase).
- [x] [F12 — Preserve selection and reading position](f12-rebase-retained-identity-never-silently-retarget-selection.md#current-phase).
- [x] [F13 — Keep rendered content fresh](f13-make-cache-validity-part-of-content-ownership.md#current-phase).
- [x] [F14 — Consistent text display and copy policy](f14-explicit-tab-control-geometry-and-copy-policy.md#current-phase).
- [x] [F15 — Incremental Holla output rendering](f15-incremental-live-output-work-measured-separately-from-idle-redraw.md#current-phase).
- [x] [F19 — Keyboard selection and copy](f19-keyboard-selection-in-the-existing-textviewport.md#current-phase).
- [x] [F20 — Search Holla output and previews](f20-find-in-read-only-output-through-existing-composition.md#current-phase) — Holla/shared slice only.
- [x] [F21 — Restore the shared terminal session](f21-restore-terminal-state-for-supported-job-control-suspension.md#current-phase).
- [x] [F22 — Trustworthy Holla visual captures](f22-separate-rasterizer-fidelity-from-terminal-correctness.md#current-phase).
- [x] [F23a — Holla state reachability coverage](f23a-state-reachability-design-d4.md#current-phase) — Holla/shared slice only.
- [x] [F23b — Shared terminal lifecycle proof](f23b-lifecycle-v01-v02.md#current-phase).
- [x] [F23c — Holla event freshness and resize](f23c-event-freshness-resize-v03.md#current-phase) — Holla/shared slice only.
- [x] [F23d — Holla input fairness](f23d-input-flood-fairness-v04.md#current-phase).
- [x] [F23e — Holla rendering performance proof](f23e-dirty-idle-performance-v05.md#current-phase) — Holla/shared slice only.
- [x] [F23f — Holla color and terminal proof](f23f-rendering-color-compatibility-v06-v07.md#current-phase) — Holla/shared slice only.
- [x] [F23g — Reproducible Holla evidence](f23g-evidence-provenance-v08.md#current-phase) — Holla/shared slice only.
- [x] [F23h — Holla facts paste and affected input regressions](f23h-precise-subsidiary-regressions-v09.md#current-phase) — Holla/shared slice only.

## Current — Holla simulated UI

<a id="existing-holla-functional-parity-audit"></a>

Each checkbox closes only the simulated UI slice. All operational and non-UI
remainders stay under Later. The [capability matrix](../parity/holla-parity-matrix.md)
still defines the full row contracts; partial representation is not Covered.

<a id="hp01"></a>

- [x] [HP01 — Discovery and stable actions](hp01-adaptive-discovery-and-stable-action-identity.md#current-phase--simulated-representation) — simulation only.

<a id="hp02"></a>

- [x] [HP02 — Search and remembered choices](hp02-search-query-editing-and-durable-usage-learning.md#current-phase--simulated-representation) — simulation only.

<a id="hp03"></a>

- [x] [HP03 — File search and resource actions](hp03-find-files-and-perform-resource-actions.md#current-phase--simulated-representation) — simulation only.

<a id="hp04"></a>

- [x] [HP04 — Folder browser and safe previews](hp04-browse-folders-and-safely-preview-files.md#current-phase--simulated-representation) — simulation only.

<a id="hp05"></a>

- [x] [HP05 — Git status, pull and push](hp05-current-repository-git-operations.md#current-phase--simulated-representation) — simulation only.

<a id="hp06"></a>

- [x] [HP06 — Git batches and hygiene](hp06-repository-batches-mirrors-and-hygiene.md#current-phase--simulated-representation) — simulation only.

<a id="hp07"></a>

- [x] [HP07 — Project-task choices](hp07-native-project-task-adapters.md#current-phase--simulated-representation) — simulation only.

<a id="hp08"></a>

- [x] [HP08 — Cargo choices and results](hp08-cargo-build-test-lint-and-clean.md#current-phase--simulated-representation) — simulation only.

<a id="hp09"></a>

- [x] [HP09 — Docker and Compose flows](hp09-docker-and-compose-outcomes.md#current-phase--simulated-representation) — simulation only.

<a id="hp10"></a>

- [x] [HP10 — Homebrew service controls](hp10-homebrew-service-lifecycle.md#current-phase--simulated-representation) — simulation only.

<a id="hp11"></a>

- [x] [HP11 — Gradle tasks and cleanup review](hp11-gradle-tasks-daemon-and-recursive-cleanup.md#current-phase--simulated-representation) — simulation only.

<a id="hp12"></a>

- [x] [HP12 — IntelliJ cleanup review](hp12-intellij-metadata-cleanup.md#current-phase--simulated-representation) — simulation only.

<a id="hp13"></a>

- [x] [HP13 — Upgrade plans and standalone actions](hp13-all-legacy-upgrade-managers.md#current-phase--simulated-representation) — simulation only.

<a id="hp14"></a>

- [x] [HP14 — Activity output and truthful results](hp14-task-execution-output-and-results.md#current-phase--simulated-representation) — simulation only.

<a id="hp15"></a>

- [x] [HP15 — Prompt input and cancellation UX](hp15-runtime-input-cancellation-and-terminal-ownership.md#current-phase--simulated-representation) — simulation only.

<a id="hp16"></a>

[HP16 — CLI list/run/doctor](hp16-cli-list-run-and-doctor-contract.md) is entirely deferred; see Later.

<a id="hp17"></a>

- [x] [HP17 — Custom-action and trust review](hp17-custom-actions-configuration-and-trust.md#current-phase--simulated-representation) — simulation only.

<a id="hp18"></a>

- [x] [HP18 — Disk scan progress and cache states](hp18-disk-measurement-progress-and-cache.md#current-phase--simulated-representation) — simulation only.

<a id="hp19"></a>

- [x] [HP19 — Disk tree and top files](hp19-disk-tree-top-files-and-selection.md#current-phase--simulated-representation) — simulation only.

<a id="hp20"></a>

- [x] [HP20 — Cleanup categories and eligibility](hp20-complete-cleanup-insight-taxonomy-and-guards.md#current-phase--simulated-representation) — simulation only.

<a id="hp21"></a>

- [x] [HP21 — Cleanup gates, dry run and recovery](hp21-deletion-authorization-dry-run-and-recovery-mode.md#current-phase--simulated-representation) — simulation only.

<a id="hp22"></a>

- [x] [HP22 — Cleanup progress and reports](hp22-cleanup-ownership-outcomes-and-operation-log.md#current-phase--simulated-representation) — simulation only.

<a id="hp23"></a>

- [x] [HP23 — Platform-aware prototype coverage](hp23-platform-and-terminal-capability-contract.md#current-phase--simulated-representation) — simulation only.

## Later — other applications and unrelated component work

These are preserved open findings, not rejected defects. The current goal does
not execute them unless a specific necessary dependency is demonstrated and
scoped under scope.md.

- [ ] [F01 — Modal-first paste routing](f01-modal-first-paste-routing.md#later)
- [ ] [F03 — Atomic ListBox mutation](f03-atomic-listbox-mutation.md#later)
- [ ] [F04 — Reconcile DataGrid read-only transitions before mutation](f04-reconcile-datagrid-read-only-transitions-before-mutation.md#later)
- [ ] [F06 — Preserve DataGrid cursor identity through local sort](f06-preserve-datagrid-cursor-identity-through-local-sort.md#later)
- [ ] [F07 — Checked row/schema ingestion](f07-checked-row-schema-ingestion.md#later)
- [ ] [F08a — Between upper-value paste](f08a-between-upper-value-paste.md#later)
- [ ] [F08c — Ctrl+Shift+Home/End](f08c-ctrl-shift-home-end.md#later)
- [ ] [F09 — Deliver Inspect copy without closing the modal](f09-deliver-inspect-copy-without-closing-the-modal.md#later)
- [ ] [F16 — Reach every showcase section at supported sizes](f16-reach-every-showcase-section-at-supported-sizes.md#later)
- [ ] [F17 — Empty allocations emit nothing](f17-empty-allocations-emit-nothing.md#later)
- [ ] [F18 — Distinguish read-only from disabled](f18-distinguish-read-only-from-disabled.md#later)

## Later — remaining portions of split tasks

- [ ] [F02 — TablePro safe tab-close identity and fallback](f02-picker-action-eligibility-and-destructive-target-identity.md#later)
- [ ] [F08b — TablePro picker-owner paste integration](f08b-picker-grapheme-editing-query-paste.md#later)
- [ ] [F08d — Other-application shortcut conformance](f08d-unassigned-modifiers-perform-plain-actions.md#later)
- [ ] [F20 — Jackin output search and shared-controller reuse](f20-find-in-read-only-output-through-existing-composition.md#later)
- [ ] [F23a — Full showcase state traversal](f23a-state-reachability-design-d4.md#later)
- [ ] [F23c — Other-application event and resize journeys](f23c-event-freshness-resize-v03.md#later)
- [ ] [F23e — Standalone list/table/grid performance coverage](f23e-dirty-idle-performance-v05.md#later)
- [ ] [F23f — Other-application rendering and platform evidence](f23f-rendering-color-compatibility-v06-v07.md#later)
- [ ] [F23g — Historical non-Holla evidence coverage](f23g-evidence-provenance-v08.md#later)
- [ ] [F23h — Standalone editor input, drag and search regressions](f23h-precise-subsidiary-regressions-v09.md#later)

## Later — Holla production and non-UI integration

Live providers, persistent stores, real process/filesystem ownership, native
platform probes and full CLI parity are not required to close the current goal.
No simulated result closes these checkboxes.

- [ ] [HP01 — Real executable/daemon probes and live discovery adapters](hp01-adaptive-discovery-and-stable-action-identity.md#later--operational-and-non-ui-integration)
- [ ] [HP02 — Durable usage-store I/O, real restart/migration and concurrent filesystem writer integration](hp02-search-query-editing-and-durable-usage-learning.md#later--operational-and-non-ui-integration)
- [ ] [HP03 — Real home indexing, OS open/reveal, OSC52/system clipboard transport and worker joins against live providers](hp03-find-files-and-perform-resource-actions.md#later--operational-and-non-ui-integration)
- [ ] [HP04 — Real filesystem listing/reads, descriptor validation and OS race/nonblocking probes](hp04-browse-folders-and-safely-preview-files.md#later--operational-and-non-ui-integration)
- [ ] [HP05 — Real git invocation, credentials, remotes and repository integration](hp05-current-repository-git-operations.md#later--operational-and-non-ui-integration)
- [ ] [HP06 — Real repository discovery, git operations, branch deletion and worktree/remote checks](hp06-repository-batches-mirrors-and-hygiene.md#later--operational-and-non-ui-integration)
- [ ] [HP07 — Live manifest discovery, runner probes, external task listing and actual project-task execution](hp07-native-project-task-adapters.md#later--operational-and-non-ui-integration)
- [ ] [HP08 — Actual Cargo process execution, target discovery and cleanup](hp08-cargo-build-test-lint-and-clean.md#later--operational-and-non-ui-integration)
- [ ] [HP09 — Docker daemon access, real Compose/log streams and destructive container/image/volume operations](hp09-docker-and-compose-outcomes.md#later--operational-and-non-ui-integration)
- [ ] [HP10 — Real brew services commands and durable service-cache I/O](hp10-homebrew-service-lifecycle.md#later--operational-and-non-ui-integration)
- [ ] [HP11 — Real Gradle commands, process probes/stops, filesystem traversal and cleanup](hp11-gradle-tasks-daemon-and-recursive-cleanup.md#later--operational-and-non-ui-integration)
- [ ] [HP12 — Real IDEA/filesystem discovery, Trash executor and operation-log writes](hp12-intellij-metadata-cleanup.md#later--operational-and-non-ui-integration)
- [ ] [HP13 — Real manager discovery, installation/upgrade commands and host mutation](hp13-all-legacy-upgrade-managers.md#later--operational-and-non-ui-integration)
- [ ] [HP14 — Production executor/adapters, actual child processes and aggregate/headless CLI contracts](hp14-task-execution-output-and-results.md#later--operational-and-non-ui-integration)
- [ ] [HP15 — Real task PTYs/stdin, process groups, TERM/KILL escalation, subreaper/reaping and OS-specific process integration](hp15-runtime-input-cancellation-and-terminal-ownership.md#later--operational-and-non-ui-integration)
- [ ] [HP16 — Full CLI list/run/doctor contract](hp16-cli-list-run-and-doctor-contract.md#later)
- [ ] [HP17 — Live global/project config reads, persistent approvals, filesystem migration and trust CLI operations](hp17-custom-actions-configuration-and-trust.md#later--operational-and-non-ui-integration)
- [ ] [HP18 — Live filesystem scanner, OS file metadata/dataless behavior, physical measurements and durable cache I/O](hp18-disk-measurement-progress-and-cache.md#later--operational-and-non-ui-integration)
- [ ] [HP19 — Live Spotlight queries/stat concurrency/timeouts, real disk traversal and physical post-cleanup scans](hp19-disk-tree-top-files-and-selection.md#later--operational-and-non-ui-integration)
- [ ] [HP20 — Live artifact discovery, process probes and filesystem sizing](hp20-complete-cleanup-insight-taxonomy-and-guards.md#later--operational-and-non-ui-integration)
- [ ] [HP21 — Actual canonical filesystem validation, Trash/permanent mutation, pathname-race probes and CLI bypass tests](hp21-deletion-authorization-dry-run-and-recovery-mode.md#later--operational-and-non-ui-integration)
- [ ] [HP22 — Real cleanup worker ownership, durable JSONL logging and physical capacity/OS integration](hp22-cleanup-ownership-outcomes-and-operation-log.md#later--operational-and-non-ui-integration)
- [ ] [HP23 — Production OS adapters, native filesystem/Trash/open/process probes on both operating systems and full CLI parity](hp23-platform-and-terminal-capability-contract.md#later--operational-and-non-ui-integration)

## Conditional — not selected for execution

- [ ] [O04 — matrix paste](o04-matrix-paste.md#later)
- [ ] [O05 — terminal extensions/OS clipboard](o05-terminal-extensions-os-clipboard.md#later)
- [ ] [O07 — read-only field APIs](o07-read-only-field-apis.md#later)

## Removed proposals

Generic widget/theme/glyph, keyed-collection, container/modal and worker
frameworks are retired, not unchecked implementation goals. Their historical
O01/O02/O03/O06 entries remain in the
[audit reference](plan-reference-audit.md#retired-framework-proposals).
Concrete task ownership, cancellation, stale-result rejection and shutdown
remain current where required by Holla.
