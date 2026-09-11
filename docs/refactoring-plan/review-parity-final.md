# Final-plan independent UI/UX parity review

Review date: 2026-09-11. Verdict: **changes required**. This is a planning review, not a candidate parity verdict. No terminal-components production source was edited, and no golden output was accepted.

## Reviewed identities and boundary

Independently resolved Git objects:

- Oracle: `02f5294bfdbf38004cc49130d0aff1d01f31434c` (`holla-fable-2026-09-10^{commit}`).
- Architectural main: `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- Planning checkout HEAD: `2e2401393c47360741ebd321679de08982dca50a`.

Read the complete planning goal and 24-section root plan; reviewed application reports/scenario contracts, component inventory/common contracts, parity synthesis, shell contributions, proof contract, task index/dependencies, and canonical task requirements/obligations with particular attention to 032–064. Direct source checks used `rtk git show`/`git grep` at the exact oracle/main objects. The 1,075-source/2,703-edge structural traceability result is not semantic proof of ordering or scenario executability.

The catalog is being edited concurrently. These hashes identify the review snapshot; later edits require re-review:

| Input | SHA-256 |
| --- | --- |
| Root plan | `91e0aa45b6ca74fd6fdaf9776776709dd5c9dde263894956c509b731cddc17ef` |
| Holla scenario register | `3e4f903ec7d32cf6857eb71114e815e0bf4193fc2d3862881afed5f62546c73e` |
| Jackin scenario register | `fcea4afda871e4a7fc89294ce84527a26b3aa7ca23471669daa80b884c4f1cd1` |
| TablePro scenario register | `a55c323119b0a8f2d608ea52337d5cde37bf56c4d29df760bd42a4687a315f3f` |
| Showcase scenario register | `5ae3740a07e26a1af140b343e61036b5d7653e50a84ef77789d386f00aef81ac` |
| Proof contract | `5358774be03e3cf24fb578121faefbc8aecdffe40876582ff056f6c529acf649` |
| Shell contribution contract | `f6e61281935fbaa5123cc66136861bd301f5954f695c058c2857d4545f3dccc5` |
| Canonical completion tree checksum-list digest | `77001340b666db3297d66f53d6d83013d1a975648fd5610a150b7dfe38f7e8f4` |

The last digest is SHA-256 of `rg --files refactoring-tasks/terminal-components/completion | LC_ALL=C sort | xargs shasum -a 256` output, including filenames. It identifies the files observed during this audit, not an atomic Git tree or acceptance receipt.

## Findings

### PARITY-FINAL-01 — High: Holla primary scenarios require downstream application repairs

**Owners:** TASK-041, TASK-043; also inspect TASK-042/044–049 for the same cross-route condition. **Status:** open.

TASK-041 owns complete `HO-HP02`; TASK-043 owns complete `HO-ROUTE-05`, `HO-ARGS`, `HO-TRUST` and `HO-GATES`. Their trusted obligations require every original assertion and every complete checkpoint to pass. Their writable scope and stated outcomes exclude the downstream Activity/provider/snapshot restoration owned by 044/047–049. Those downstream tasks depend on the earlier tasks.

Concrete source evidence:

- Oracle `src/bin/holla/app_tests_parity.rs:200` (`hp02_recents_learned_queries_query_editing_and_persistence_merge`) presses Enter after `cargo\ncheck`, obtains an Activity ID, and checks exact argv and persisted usage. The input draw is an Activity frame, not a Finder frame. `041/README.md:35` owns `HO-HP02`; `044/trusted/obligations.md:7` owns the missing Activity composition/output.
- Oracle `src/bin/holla/app_tests_flows.rs:310` (`nested_child_trusts_then_runs_in_the_child_directory`) accepts trust, submits arguments with Ctrl+S, asserts `TabKind::Activity`, and checks the rendered `mise run //apps/frontend:test`. `043/README.md:35` owns that complete `HO-ROUTE-05`; `044` is its downstream Activity owner.
- `HO-ARGS` explicitly requires a snapshot after accepted arguments; oracle `app_tests_rows.rs:665` and the register identify that port result. `HO-GATES` imports the entire Docker execution test, including execution/revalidation, despite its gate-only title. `HO-TRUST` imports all HP17 outcomes. Their complete transcript scope is broader than an assertion sweep over an unaccepted gate.
- Main `apps/holla/src/screens/activity.rs:33,107`, `domain/activity.rs:5`, and `app.rs:972` have the already documented different output/composition. Reusable component completion cannot itself replace these application compositions.

**Failure mode:** correct strict comparison blocks the earlier task until it performs work explicitly assigned to its descendant. Relaxing the comparison instead creates a premature parity claim. The shell-only contribution repair does not cover these Holla slices.

**Repair:** trace every primary transcript through all reached application surfaces and domain producers. Put complete scenario closure after those producers. Give the earlier slice explicit nonempty source-qualified whole-checkpoint contributions and/or concrete semantic assertions for the behavior it repairs, with immutable membership and real production input. Preserve every original parent checkpoint and assertion under the later whole-scenario owner. Do not merely move all early proof to app closure or use a potentially empty preservation intersection.

### PARITY-FINAL-02 — High: Jackin launch/config tasks close dependent surfaces before their owners

**Owners:** TASK-052, TASK-054, TASK-055. **Status:** open.

- `054/README.md:22,32,42` depends on TASK-053 and requires complete `JA-040` before TASK-055 may begin. `jackin-scenarios.tsv` `JA-040` explicitly captures every launch stage, handoff and **Capsule arrival**. Oracle `src/bin/jackin_preview/app_tests.rs:179` (`launch_runs_all_stages_and_hands_off_to_the_capsule`) confirms the transition and then exercises pane echo. `055/trusted/obligations.md:7` owns the live Capsule pane/tab topology, input, output and retained rendering. TASK-054 excludes other application flows and is described as a Cockpit slice, not a Capsule rewrite.
- TASK-052 owns complete `JA-022`, whose config edit opens the 1Password reference picker; TASK-053 owns the full nested 1Password flow. TASK-052 also owns `JA-028`, whose Settings flow includes Accounts handoff before TASK-053 repairs Accounts. See `052/trusted/obligations.md:17`, `053/trusted/obligations.md:7,17`, oracle `screens/config.rs:1201,1737,2159`, and the `JA-022`/`JA-028` register rows.

**Failure mode:** every intermediate full frame is mandatory, so these are not harmless references to a future surface. The earlier task must restore a downstream surface or remain red. Existing component prerequisites prove generic controls, not the missing Jackin composition.

**Repair:** make the required scope of the first usable Capsule/account/picker composition explicit and upstream, or assign the cross-surface full scenarios to the downstream task while freezing bounded full-checkpoint/semantic contributions for the earlier task. Name the exact task-owned passing prerequisites; do not rely on generic “all primary scenarios” or empty preservation sets.

### PARITY-FINAL-03 — High: TablePro task stages still require future Workbench, query and History frames

**Owners:** TASK-058, TASK-059, TASK-061, TASK-063. **Status:** open.

Three independently reproducible scope/dependency contradictions:

1. `058/trusted/obligations.md:17` owns complete `TP-005` and `TP-016`, including `cp(connected)` and `cp(workbench)` after 12 ticks. TASK-059 owns Workbench/explorer/empty-state restoration and depends on TASK-058. Oracle `src/bin/tablepro/workbench.rs:99` starts without tabs; `:1277` renders the framed `No open tabs` EmptyState with exact hints. Main `apps/tablepro/src/app.rs:1862` instead paints `No tab open`. This is a concrete downstream full-frame mismatch even without opening a table.
2. `059/trusted/obligations.md:17` owns complete `TP-026`, whose final `Ctrl+T;cp(new)` displays the query editor. TASK-061 depends through TASK-060 on TASK-059 and explicitly owns replacing the fixed three-row query Field. Oracle `src/bin/tablepro/tabs.rs:972` constructs CodeEditor, SQL highlighting/segments and a 38-percent split with minima 4/6; main `apps/tablepro/src/app.rs:1815` uses `fixed_flex_pair(inner,3)` and Field/TextInput. TASK-059 cannot close `TP-026` by repairing tabs alone.
3. `061/trusted/obligations.md:17` owns complete `TP-050`, whose final `Ctrl+Y;cp(history)` opens History. TASK-063 owns History and depends through TASK-062 on TASK-061. Main `apps/tablepro/src/app.rs:1848` paints only a heading and at most six SQL strings; oracle `tabs.rs:2272` supplies the actual History controls. `063/trusted/obligations.md:7` explicitly assigns that repair to TASK-063.

`proof-contract.md` requires every task-owned/prerequisite scenario to pass. The preservation-intersection rule does not exempt any checkpoint of a primary complete scenario.

**Repair:** reconcile primary ownership with the actual last required producer, while defining passing stage contributions for connection completion, no-tabs/new-tab dispatch and query batch execution. If TASK-058 is intended to build the pristine Workbench too, state and verify that scope explicitly. If TASK-059 is intended to build the initial query page too, give it the real CodeEditor composition and a bounded proof, then keep later editing/result journeys with 061. Do not erase the final Workbench/query/History checkpoints or permit cropped frames.

### PARITY-FINAL-04 — High: Holla source-test replay contradicts its four-size assertion contract

**Owners:** TASK-003 and all downstream consumers of `HO-ROUTE-04`, `HO-ROUTE-12`, `HO-ACTIVITIES`; qualification contract TASK-070. **Status:** open.

`holla.md` defines `ROUTE` as importing the pinned test's exact fixture/mutations/events and executing **every original assertion**. `holla-scenarios.tsv:62,70` requires those source tests to replay at 80×24, 100×30, 120×40 and 160×50. The original assertions are not valid at every requested dimension:

- Oracle `src/bin/holla/app_tests_flows.rs:283`: `assert!(h.row(38).contains("attached"), ...)` in `activities_survive_navigation_and_merged_logs_keep_identity`. At height 24 or 30, row 38 does not exist.
- Oracle `src/bin/holla/app_tests_flows.rs:604`: `assert!(h.row(28).contains("unavailable"), ...)` in `hard_cases_stay_legible_and_report_failed_discovery`, whose native fixture is 100×30. At height 24, row 28 does not exist.
- Oracle `src/bin/holla/app_tests.rs:94` implements `row` with `lines().nth(y).unwrap_or("")`; these assertions therefore deterministically fail for the smaller expansion. Keeping the original constructor dimensions instead would leave the promised four-size expansion uncaptured.

This is an oracle preparation failure before any candidate repair, not an allowed candidate mismatch. A baseline owner currently has to reinterpret or drop original assertions to make the contract executable.

**Repair:** explicitly separate (a) unchanged source-native test execution with original dimensions, coordinate literals, fixture mutations and all assertions from (b) oracle-derived multi-size transcripts. Define a protected mapping for dimension-sensitive assertions/coordinates in (b), observing the same semantic owner with geometry resolved only from each size's oracle. Keep exact per-size frames, preserve the native assertion proof, and reject an unexplained dropped assertion. Add a qualification case using a source seed with fixed footer row and pointer coordinates so the rule cannot be implemented by silently ignoring failed source assertions.

## Checks that did not produce new findings

The plan explicitly preserves completed-click editing, read-only caret movement, current-cell Grid activation, hover suppression, down/up separation, modal capture, original selection source, viewport retention, cross-span graphemes, exact fades and breakpoint resizing. It distinguishes modeled-only fixtures from actual PTY reachability and forbids candidate-selected coordinates, expected-output regeneration and regional masks. Component repairs precede application chains through TASK-031. These are substantial contracts; this review does not claim they are implemented or runtime-proven.

The shell contribution scheme retains whole parent scenarios, requires nonempty whole-frame selections, and does not close parent parity from semantic state alone. No separate objection is raised against that scheme itself. Findings 01–03 identify places where its dependency reasoning has not yet been applied.

Host isolation, runner ABI and Rust architecture qualifier corrections are under separate review. This report neither duplicates nor approves them. The full application trace audit must be repeated after ownership changes; moving the listed cases alone does not prove all remaining primary scenarios are stage-valid.
