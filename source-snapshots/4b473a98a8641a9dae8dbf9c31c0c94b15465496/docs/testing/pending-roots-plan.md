# Pending roots plan

This plan triages the 187 pending legacy roots into 8 ordered slices.
It was built at `origin/visual-baseline` tip
`04121e4bfa2aeeda3893e81f6bb9e84a04d34a29`
(`test(vb): add TablePro app-journey suite slice 1`).
All counts below were verified live at that commit.

## Words

- **Legacy root**: one old snapshot path, for example `holla/flows/git/merge`.
  The registry holds 302 roots in its union list.
- **Mapped root**: a root with a coverage row in at least one group slice.
  115 roots are mapped.
- **Pending root**: a root with no coverage row yet.
  187 roots are pending.
- **Multi-slice root**: a root triaged in more than one group slice.
  24 mapped roots sit in more than one slice.
- **Slice**: one implementation work item in this plan.
  It names root IDs, target files, new tests, dependencies, and risks.
- **VB target file**: one test file under
  `tests/harness/tests/visual_baseline/`.
  New rows and tests land there.
- **Est. new tests**: new test units, one per root.
  One unit means one registry row plus its checks.
  Journey tests bundle several checkpoints.
  Matrix tests map one test to one root.
  The count is a scheduling estimate, not a mandate.

## Slice table

Rows run in execution order.
S1 and T1 go first because their matrices exist and their fixtures are shared.

| Slice | Roots | VB target file(s) | Est. | Dependencies | Risks |
|---|---|---|---|---|---|
| 8A-S1 | 21: fade 10, flows 5, pages 3, resize 2, hover 1 | pointer.rs, showcase.rs, showcase_journeys.rs | 21 | None. First slice. | DataTable roots are not DataGrid roots. The triage notes pin this. Fade, hover, and resize matrices already exist, so that work is rows plus checks only. |
| 8A-T1 | 14: connections 4, workbench 4, resize 2, audit 1, fade 1, overlays 1, table 1 | tablepro.rs, tablepro_journeys.rs, audit.rs, pointer.rs | 14 | After S1. Reuses DataTable and select row patterns plus the tablepro_journeys slice 1 fixtures. | history_tab is a work tab, not a modal. explorer_hidden is a negative case. Small slice, but app-complete, so it stays whole. |
| 8A-J1 | 30: editor 7, scenarios 6, manager 5, settings 5, accounts 3, intro 3, audit 1 | jackin.rs, jackin_journeys.rs, audit.rs | 30 | After T1. Reuses form and save-preview row patterns. Internal order: intro and scenarios journeys, then manager and accounts, then editor and settings forms. | Both save_preview forms have no rows yet. Both launch pickers need a jackin Picker row. The accounts filter flow (enter begins, second enter commits) is unpinned. |
| 8A-H1 | 29: cleanup 11, disk 6, docker 6, cargo 2, gradle 2, idea 2 | holla_journeys.rs | 29 | After J1. Reuses journey harness patterns. Builds the holla fixture world first. | cleanup/gate-1 and cleanup/gate1 look like duplicates, and so do remote/gate-1 and gate2. Verify distinct captures before writing rows. linux_report is linux-only. |
| 8A-H2 | 22: files 7, task 4, query 3, scope 3, finder 2, browser 1, child 1, config 1 | holla_journeys.rs | 22 | After H1. Reuses the fixture world and journey patterns. | query/empty_preview needs a holla Empty row. The showcase row does not cover it. jump_error recovery is timing-sensitive. |
| 8A-H3 | 25: git 5, brew 4, remote 4, args 3, trust 3, upgrade 3, platforms 3 | holla_journeys.rs | 25 | After H2. Reuses the fixture world and journey patterns. | Trust and remote gates must use synthetic secrets only. Brew flows must use the sim world, never real brew. deploy_failed, pull_blocked, and push_rejected are failure-injection cases. |
| 8A-H4 | 21: parity 21 | holla.rs | 21 | After H1-H3. Parity rows compare against flow behavior. | parity/executor has no executor chip row yet. Each parity root spans two or more modes, so checks double. |
| 8A-H5 | 25: concept 9, fade 4, activities 3, menu 2, audit 2, resize 2, alternatives 1, help 1, quit 1 | holla.rs, holla_journeys.rs, pointer.rs, audit.rs | 25 | After H4. Last slice. Mixed matrix families plus chrome one-offs. | Fade matrices exist, so that work is rows only. Audit roots join the audit matrix in audit.rs. Resize uses pointer.rs sequences. activities jump and overlay need a holla modal row. |

## Reconciliation

- Union roots: 302.
- Mapped roots: 115.
- Pending roots: 187.
- Slice roots sum: 21 + 14 + 30 + 29 + 22 + 25 + 21 + 25 = 187.
- Every pending root sits in exactly one slice.
- The 115 mapped and 24 multi-slice baseline comes from Phase-5c.
- Pending roots by app: holla 122, jackin 30, showcase 21, tablepro 14.
- All 187 pending roots already have capture literals in the VB suite.
- Slice work authors rows plus checks, not recaptures.

## Root IDs

### 8A-S1 (21)

- showcase/fade/datagrid/wheel
- showcase/fade/diff/wheel
- showcase/fade/editor/wheel
- showcase/fade/lists/wheel-fade
- showcase/fade/scroll/page
- showcase/fade/scrolling/wheel-fade
- showcase/fade/sidebars/wheel
- showcase/fade/terminal/scrollback
- showcase/fade/textarea/wheel
- showcase/fade/trees/wheel-fade
- showcase/flows/editable/committed
- showcase/flows/editable/editing
- showcase/flows/inspector/open
- showcase/flows/inspector/scrolled
- showcase/flows/tables/selected
- showcase/hover/tables
- showcase/pages/editable
- showcase/pages/overview
- showcase/pages/tables
- showcase/resize/overview_grown
- showcase/resize/overview_shrunk

### 8A-T1 (14)

- tablepro/audit/production
- tablepro/connections/default
- tablepro/connections/delete_dialog
- tablepro/connections/duplicated
- tablepro/connections/production_detail
- tablepro/fade/table_wheel
- tablepro/overlays/history_tab
- tablepro/resize/workbench_grown
- tablepro/resize/workbench_shrunk
- tablepro/table/structure
- tablepro/workbench/commit_dialog
- tablepro/workbench/explorer_hidden
- tablepro/workbench/maximized
- tablepro/workbench/quit_confirm

### 8A-J1 (30)

- jackin/accounts/detail
- jackin/accounts/drawer
- jackin/accounts/filter
- jackin/audit/accounts
- jackin/editor/auth
- jackin/editor/env
- jackin/editor/general
- jackin/editor/mounts
- jackin/editor/mounts_dirty
- jackin/editor/roles
- jackin/editor/save_preview
- jackin/intro/f300
- jackin/intro/f400
- jackin/intro/skipped
- jackin/manager/detail_drawer
- jackin/manager/hard_launch_locked
- jackin/manager/hard_launch_picker
- jackin/manager/launch_picker
- jackin/manager/tree_expanded
- jackin/scenarios/first-use
- jackin/scenarios/hard-cases
- jackin/scenarios/launch-failure
- jackin/scenarios/launch-running
- jackin/scenarios/outro-last
- jackin/scenarios/returning
- jackin/settings/agents
- jackin/settings/mounts
- jackin/settings/route
- jackin/settings/save_preview
- jackin/settings/trust

### 8A-H1 (29)

- holla/flows/cargo/clean
- holla/flows/cargo/clean_confirm
- holla/flows/cleanup/artifacts
- holla/flows/cleanup/categories
- holla/flows/cleanup/derived_data
- holla/flows/cleanup/gate-1
- holla/flows/cleanup/gate1
- holla/flows/cleanup/gate2
- holla/flows/cleanup/gate2_typed
- holla/flows/cleanup/history
- holla/flows/cleanup/linux_report
- holla/flows/cleanup/plan
- holla/flows/cleanup/report
- holla/flows/disk/scan_complete
- holla/flows/disk/top_files
- holla/flows/disk/tree
- holla/flows/disk/tree_apparent
- holla/flows/disk/tree_selected
- holla/flows/disk/tree_unfolded
- holla/flows/docker/done
- holla/flows/docker/drift
- holla/flows/docker/gate2_typed
- holla/flows/docker/remove_failed
- holla/flows/docker/remove_gate1
- holla/flows/docker/remove_gate2
- holla/flows/gradle/cleanup
- holla/flows/gradle/gate1
- holla/flows/idea/cleanup
- holla/flows/idea/gate2

### 8A-H2 (22)

- holla/flows/browser/hidden
- holla/flows/child/query
- holla/flows/config/page
- holla/flows/files/actions
- holla/flows/files/jump
- holla/flows/files/jump_error
- holla/flows/files/jumped
- holla/flows/files/preview_control
- holla/flows/files/results
- holla/flows/files/unicode
- holla/flows/finder/query
- holla/flows/finder/query-selected
- holla/flows/query/empty_preview
- holla/flows/query/redone
- holla/flows/query/undone
- holla/flows/scope/children
- holla/flows/scope/parent
- holla/flows/scope/system
- holla/flows/task/input_cancelled
- holla/flows/task/input_killed
- holla/flows/task/sources
- holla/flows/task/sources_diagnostic

### 8A-H3 (25)

- holla/flows/args
- holla/flows/args/clone
- holla/flows/args/page
- holla/flows/brew/services
- holla/flows/brew/services_stop_failed
- holla/flows/brew/upgrade_batch
- holla/flows/brew/upgrade_plan
- holla/flows/git/batch
- holla/flows/git/batch_picker
- holla/flows/git/merge
- holla/flows/git/pull_blocked
- holla/flows/git/push_rejected
- holla/flows/platforms/files
- holla/flows/platforms/linux_gate1
- holla/flows/platforms/linux_gate2
- holla/flows/remote/gate-1
- holla/flows/remote/gate2
- holla/flows/remote/gate2_typed
- holla/flows/remote/query
- holla/flows/trust/accepted
- holla/flows/trust/deploy_failed
- holla/flows/trust/prompt
- holla/flows/upgrade/confirm
- holla/flows/upgrade/excluded
- holla/flows/upgrade/facts

### 8A-H4 (21)

- holla/parity/brew-services
- holla/parity/browser
- holla/parity/cargo
- holla/parity/cleanup-results
- holla/parity/custom-actions
- holla/parity/delete-safety
- holla/parity/disk-navigation
- holla/parity/docker
- holla/parity/executor
- holla/parity/files
- holla/parity/git-batch
- holla/parity/git-current
- holla/parity/gradle
- holla/parity/history
- holla/parity/idea
- holla/parity/insights
- holla/parity/platforms
- holla/parity/platforms-linux
- holla/parity/task-input
- holla/parity/task-sources
- holla/parity/upgrade-managers

### 8A-H5 (25)

- holla/audit/rust
- holla/audit/upgrade
- holla/concept/activities-multi
- holla/concept/disk-cleanup
- holla/concept/docker-cleanup
- holla/concept/first-use
- holla/concept/hard-cases
- holla/concept/launch-failure
- holla/concept/monorepo-child
- holla/concept/monorepo-root
- holla/concept/remote-host
- holla/fade/browser_wheel
- holla/fade/cleanup_list_wheel
- holla/fade/executor_burst-end
- holla/fade/trust_body_wheel
- holla/flows/activities/empty
- holla/flows/activities/jump
- holla/flows/activities/overlay
- holla/flows/alternatives
- holla/flows/help/overlay
- holla/flows/menu/file
- holla/flows/menu/go
- holla/flows/quit/confirm
- holla/resize/rust-dirty_grown
- holla/resize/rust-dirty_shrunk

## Machine companion

`pending-roots-plan.json` sits next to this file.
It holds an array of slice objects.
Each object holds `slice`, `roots`, `target_files`, and `est_tests`.

## Validation

- The JSON parses.
- Every pending root appears in exactly one slice.
- A script checks both facts and shows the counts.
- The scope-check script passes.
- No production paths changed.
- Only the two plan files are staged.
