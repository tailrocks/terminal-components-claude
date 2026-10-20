# Termrock migration and acceptance matrix

Updated: 2026-10-20. Status: **in progress; all four applications incomplete**. This is the current migration record. Older audit reports remain evidence, not a complete inventory or acceptance declaration.

## Fixed sources and starting tips

| Source | Identity recorded before this work | Use |
| --- | --- | --- |
| `parity/termrock-visual-baseline` | `0b8f4e32b24a137987307600b10d892bec0619c3` (remote observation 2026-10-20 10:52:50 UTC) | PR #38 candidate branch |
| `design/interaction-polish` | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` (same observation) | Recover useful contracts; comparison branch |
| `visual-baseline` tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` | Frozen oracle; never move |
| Frozen source commit | `4169783ae883788186d161af09ba9c481bd84e5c`, tree `ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297` | Isolated baseline archive and CLI binaries |
| Tuiscotti | `a1ea27f757d75d07b05499276cd932e9f8e067ad` | Fixed PTY and frame-comparison tool |

The branch tips are observations, not immutable acceptance targets; record exact tested source commits in every run. Both branches must carry byte-identical paired-test sources and manifests. Do not update frozen snapshots to make a candidate green.

## Application migration matrix

| Application | Baseline screen and flow families to preserve | Shared owner required | Current evidence / missing work | Product | Library |
| --- | --- | --- | --- | --- | --- |
| Showcase | Shell/chrome and page navigation; component catalog pages for buttons, inputs/forms, lists, tables, trees, tabs, menus, dialogs/popups, feedback, text/code, layout; each demonstrated state/configuration and keyboard/pointer route | `termrock` widgets, overlays, runtime, text/layout/theme; Showcase supplies demo data and actions | Native root gate at recorded checkpoint: 1,479 passed, 615 skipped. Current external component checkpoint `e7c4be2e5` reports 1,479 root tests, 48 code-editor checks, clippy/fmt; not full workspace or frozen CLI acceptance. Live table/list state and footer ownership remain open. | INCOMPLETE | INCOMPLETE |
| Jackin (`jackin-preview`) | Intro/outro and first-use; menu and capsule; manager/cockpit; workspace, split panes, editor and completion; picker, drawer, settings, accounts, usage, launch, help, save/cancel and resize flows | `termrock` menu, picker, editor/completion, drawer, master-detail/layout, shell and shared input/focus; Jackin retains simulation/domain actions and branded intro/outro/fade | Frozen mono test coordinate correction independently verified; no full candidate journeys. Manager/picker consumer still has full-frame failure; generic shell disappears under open picker unless shared owner layering is fixed. | INCOMPLETE | INCOMPLETE |
| TablePro | Connection/catalogue, create/edit/reconnect; multiple query tabs and tab-list/picker; explorer/workbench; SQL editing/completion; results, error, explain/analyze, history; filter editor, dialogs, help, scroll and resize | `termrock` Picker and tab list, table/grid, editor/completion, dialogs, shell/layout; TablePro owns SQL, connection and query effects | Shared table component exists; consumer model is not fully wired/paired. Frozen editor Tab semantics differ from a prior candidate. Sort journeys are being added to the common real-CLI extension suite. Historical ANSI repaint paths remain. | INCOMPLETE | INCOMPLETE |
| Holla | Home/catalogue and Finder; first-use, monorepo, activities, Docker/disk cleanup, launch failure; parity discovery/history/files/browser/Git/Cargo/task sources; menus, prompts, help, resize and terminal modes | `termrock` shell, menu, picker/search, list, dialog, status, layout/scroll; Holla owns simulation/domain actions and data | Single Holla menu-hover live case passed three phases. Existing Holla native digest and broad CLI captures remain red. `*_plate` model dispatch and application paint helpers are production render paths pending replacement and proof. | INCOMPLETE | INCOMPLETE |

A component-level buffer test, migrated digest, or same-source control does not satisfy product acceptance. A screen that paints through a large private UI crate does not satisfy the library gate.

## Paired test system and coverage

The shared runner is `tools/frozen_baseline.py`; its harness patch and copied helper remain test-only. The test suite drives real frozen and candidate CLIs through the pinned Tuiscotti PTY, captures baseline frames, replays the frozen binary, then compares the candidate. Canonical decoded frames include every cell/style attribute and cursor state. Original expected data is immutable. Each successful case must produce ordered checkpoint receipts in every phase.

- Original frozen inventory: 28,895 ignored scenarios plus 20 ordinary checks. The original stored corpus has 7,675 approvals and 21,220 pending rows. `--all` fails closed on the missing references; a selected case is partial.
- Ordinary checks: 11 source-only, 9 frozen-CLI, 9 candidate-CLI. `isolation::env_isolation` belongs in both real-binary groups. The same guard suite and runner files must exist byte-for-byte on both comparison branches.
- Registered extension inventory: 174 cases. The current pilot exercised 36 cases and emitted 174 ordered receipts in each reported phase. Extension coverage supplements the original inventory; it does not replace or rename original scenarios.
- Latest recorded candidate full-corpus attempt: 240 scenarios red. It is not acceptance. Most recent candidate reports also include a first Holla menu-hover cell mismatch before the shared title-hover correction; the one-case Holla rerun then passed. Re-run exact current branch heads.
- Resize approval conflict remains unresolved: pinned frozen Showcase `overview_grown` output is 19,454 bytes and matches the frozen overview page, while committed resize approval is 18,474 bytes; grown and shrunk approvals are identical. Preserve both. Obtain an independent diagnosis and a recorded reference decision; do not bless or alter either artifact.
- Required next expansion: meaningful intermediate checkpoints after input, resize and controlled time; adjacent/breakpoint sizes, changed/reordered data, empty/large data, repeated open/close, pointer/pressed states, text edits, supported color modes and wide/combining text. Existing registered checkpoints are not proof of unregistered flows.

Evidence details and limits: [frozen CLI parity](../audits/2026-10-20-frozen-cli-parity.md), [full CLI runner](../audits/2026-10-20-full-cli-runner.md), [live frame support](../audits/2026-10-20-live-frame-support.md). Audit files are superseded only where this matrix says so; underlying red evidence remains active.

## Production replay-path inventory

Audit searched production crates for `historical_*`, `*_plate`, `RUNS`, ANSI includes, frame decoding, raw-buffer writes and direct paint. Findings below are confirmed replay or fixture-selected render families, not just name matches. Ordinary semantic demo data, icons, terminal viewers, and genuine layout breakpoints require individual review and must not be removed by naming heuristic.

| Production path | Observed mechanism and purpose | Replacement owner / status |
| --- | --- | --- |
| `crates/jackin-preview-app/src/app/historical_*.rs` (42 replay files: `historical_accounts_add_form`, `historical_accounts_add_form_required`, `historical_accounts_detail`, `historical_accounts_detail_80`, `historical_accounts_drawer`, `historical_accounts_filter`, `historical_accounts_filter_80`, `historical_audit_accounts_80`, `historical_capsule_menu`, `historical_capsule_menu_80`, `historical_capsule_new_tab`, `historical_capsule_palette`, `historical_capsule_split_vertical`, `historical_capsule_zoom`, `historical_cockpit_cancel`, `historical_cockpit_info`, `historical_editor`, `historical_editor_auth`, `historical_editor_env`, `historical_editor_mounts`, `historical_editor_mounts_dirty`, `historical_editor_roles`, `historical_editor_save`, `historical_hard_cases_80`, `historical_hard_launch_locked`, `historical_hard_launch_picker`, `historical_help_overlay`, `historical_inspect`, `historical_intro_f300`, `historical_launch_picker`, `historical_manager_menu`, `historical_settings_agents`, `historical_settings_env`, `historical_settings_mounts`, `historical_settings_mounts_80`, `historical_settings_route`, `historical_settings_route_80`, `historical_settings_save`, `historical_settings_trust`, `historical_usage_detail`, `historical_usage_overview`) | `RUNS` arrays encode positioned text and colors; paint those cells after scenario/state/viewport selection. These are production captured-frame replays. `historical_paint.rs` is a separate historical style-recipe helper, not a `RUNS` frame; review it as theme ownership, not as proven replay. Likewise Showcase `historical_palette` functions return semantic theme styles; the name alone does not establish frame replay. | Move behavior into shared widgets/blocks and feed Jackin domain data/actions. All 42 replay files remain unresolved unless removed with a paired journey. |
| `crates/jackin-preview-presentation/src/{capsule.rs,launch_running.rs}` | Separate presentation-level positioned `RUNS` renderers; proves replay is not limited to app crate. | Shared component composition; unresolved. |
| `crates/tablepro-ui/src/app.rs` and 18 `crates/tablepro-ui/src/app/historical_*.rs` modules (`historical_close_confirm`, `historical_delete_dialog`, `historical_filter_editor`, `historical_form_advanced`, `historical_form_new`, `historical_help_overlay`, `historical_history_tab`, `historical_picker_open`, `historical_query_analyze`, `historical_query_completion`, `historical_query_error`, `historical_query_explain`, `historical_query_results`, `historical_safemode_picker`, `historical_table_scrolled_right`, `historical_tablist_open`, `historical_workbench_explorer_hidden`, `historical_workbench_maximized`) | Production predicates select fixed scenario/size/state frames; modules parse `RUNS` from `include_str!` ANSI or hold fixed runs; app repaints after normal UI. | Shared Picker/tab list/editor/table/dialog/layout fixes; TablePro supplies query and connection data. All unresolved. `historical_empty_query` is also a production selection predicate and needs source-level replacement review. |
| `crates/showcase-demos/src/pages/selectable/{frames.rs,delete_confirm.ansi,bulk_mixed.ansi}` | Imports two approved ANSI frames, decodes cells, resets and writes raw buffer for selectable demo cases. | Real shared selectable/table/list components with inert reference-state data; unresolved. |
| `crates/holla-sim/src/catalogue.rs` plus `crates/holla-ui/src/screens/home.rs` | Scenario and exact viewport/time `*_plate` dispatch selects named fixture view; home helpers paint dedicated fixed families, including parity files/browser and tag lists. Some catalogue functions build semantic domain view data, so review each function before removing it; exact dimension/time selection plus direct plate painters are not accepted as final architecture. | Shared Holla screens/components driven by simulation state and domain view data; unresolved. |
| `crates/termrock-e2e/src/lib.rs` | Test/evidence frame conversion and fixture-generation helpers. Search hit is test support, not by itself production replay; verify dependency/build reachability before any removal. | Test-only harness ownership, never app render path. Audit reachability remains open. |
| `tools/parity_replay.py`, `tools/parity_approve.py`, `tools/test-inventory/historical.json`, `crates/termrock-e2e/**`, `crates/visual-harness/**`, `crates/termrock-conformance/tests/visual_baseline/**`, `crates/showcase/tests/tuiscotti_qualification.rs` | Reference frame management, source inventories, and real-CLI/test harness checks. These are verification paths, not app render paths, and reference data belongs here. Verify dependency edges so no app binary links test-only capture code. | Keep reference artifacts and capture in test system; no production dependency permitted. Dependency/build graph proof pending. |

This is a scoped source inventory from the recorded branch tree, not a proof that *all* helper, presentation, build-script or generated-code paths were exhausted. Inspection of `crates/termrock-e2e/build.rs` found suite-file hashing and `rerun-if-changed` generation only; it does not generate application render code. The earlier 35-site ownership review and 85/88 audit finding counts are findings, not path counts or a complete coverage denominator. Continue repository-wide dependency/build/generated-source tracing. Do not claim replay-free architecture until each production path above has a replacement and the full audit records zero remaining paths.

## Architecture and independent validation

| Gate | Evidence | Status |
| --- | --- | --- |
| Independent production crates | Prior exact checkpoint: 19 production packages, 1,313 passed, 4 skipped; package clippy green | Partial; rerun at current integrated commit |
| Root default members | 1,479 passed, 615 skipped at prior checkpoint | Partial; six members only |
| Full workspace | 3,795 tests ran, 3,760 passed, 35 failed; workspace clippy failed at `crates/tablepro-ui/src/app.rs:9284` (`expect_used`) | RED; failures recorded in [independent validation](../audits/2026-10-20-independent-workspace-validation.md) |
| Code editor completion | 48 focused tests green at `e7c4be2e5` per independent checkpoint | Component only; consumer and outside-click ownership remain open |
| Master-detail | Shared geometry/state API and 374 tests, 3 skipped at recorded checkpoint; does not include floating-layer manager conversion | Component only; app use and full frame evidence absent |
| External public consumer | Not yet demonstrated for proposed composition API with unrelated domain data and no private app helpers | RED / missing |
| Ownership/replay audit | Existing exception ratchet and 35-site report; 85/88 prior audit findings are not a complete path inventory | RED / partial |

The new `MasterDetail` owns layout geometry/state; modal-layer conversion remains separate. The completion adapter owns editor/popup lifecycle; outside clicks currently dismiss without forwarding to owner. Picker policy/title-style and MenuBar changes with native fixture conflicts are deferred WIP at the cited checkpoints; do not infer parity from tests that only assert a proposed style.

## Decisions and next evidence

1. Keep frozen source/tag/Tuiscotti IDs fixed. Record commit, tree, binary hashes, environment, size, mode, scenario inventory, receipts and result for every candidate run.
2. Keep runner code, test helper and manifests identical on both branches. A shared file diff or a test inventory mismatch blocks branch comparison.
3. Keep each app INCOMPLETE until original plus extension inventory and all required ordinary checks pass on the exact same candidate commit, with no skipped or missing case and a reviewed reference-data decision.
4. Keep library gate separate: remove replay from production, prove public shared APIs own mechanics, compile an external consumer, and complete source/dependency/build/generated-code review.
5. Resolve red behavior in shared components or domain model; retain baseline flows. Never select a renderer by test label, exact viewport/clock, terminal detector or captured accent. Never repaint a widget in app code or register absent controls to move focus/cursor.
6. Continue in order: TablePro Picker/tab-list real consumer and replay removal; Showcase real button/catalogue paths; Jackin semantic shell/workspace/picker/editor; Holla semantic catalogue/layout. Expand shared blocks only for demonstrated consumers.

No application has full product or library acceptance. This matrix becomes current at its merge commit; append exact commits and evidence without relabeling partial checks green.
