# TablePro live contracts

Status: focused changes implemented; product parity remains incomplete.

## Scope

Source started at `99b719d85a0714c88484240dbc9184f110b518d4`; integration base is `407d727f9f952bafcbb842e8be341ffb32120e4e`. The frozen product source remains `4169783ae883788186d161af09ba9c481bd84e5c`.

This increment changes tab activation focus, Delete behavior, and per-query execution metadata. It does not resolve picker editing, substring filtering, filtered-height measurement, historical render bypasses, or the complete TablePro comparison inventory.

## Frozen-source findings

The findings below were checked directly against the fixed `visual-baseline` source commit, not inferred from current snapshots:

1. `src/bin/tablepro/app.rs` handles `PickerAction::Secondary` for `TAB_LIST` by mapping the filtered row through `tab_index`, closing the workbench tab unconditionally, popping the picker, and calling `open_tab_list` again (lines 1499–1509). Choosing a tab calls `Workbench::set_active` and focuses `primary_focus()` (lines 1624–1631). Picker Delete therefore discards dirty query and pending-grid edits. The candidate now has a keyed picker-specific close path while normal close requests remain guarded.
2. `src/bin/tablepro/app.rs::tab_list_items` lowercases both query and label for substring matching, then renders each query's latest status (lines 2195–2223). The candidate still delegates filtering/height to the shared Picker and remains dependent on its unresolved substring and filtered-size contracts.
3. Frozen `QueryTab` stores `last_status: Option<(String, bool)>` and `last_duration: Option<u32>` separately (`src/bin/tablepro/tabs.rs`, lines 938–983). Empty runs set `nothing to run`; cancellation sets `cancelled`; completion records total simulated ticks as milliseconds, reports `error` or `Statement N/M failed`, and formats successful single/multiple statements as `ok · …` / `N statements · …` (lines 1045–1053, 1078–1088, 1128–1153). The candidate adds summary/error/duration storage and updates synchronous `run_query` outcomes, but it does not yet reproduce this multi-statement, empty-run, or simulated timing lifecycle. Its wall-clock duration can vary, so status-cell parity is unproven.
4. Frozen picker selection explicitly returns focus to the active tab's primary control; the initial failure report identified candidate activation that left focus at the dismissed Picker. The new real-Harness regression exercises selecting the prior query tab and verifies query-editor focus.

## Changes

- Choosing a tab now focuses a real query editor or table data grid after closing the picker. Structure and history views do not currently have focusable shared controls, so selection does not target synthetic controls; history-list extraction remains open.
- Delete now uses a keyed picker-specific unconditional close path, closes and reopens the picker, and resets its query and cursor. This discards dirty query edits and pending table-grid changes without a confirmation dialog, matching the frozen picker flow. Other close paths retain their dirty-work guard.
- `QueryTab` now retains its last execution summary, error flag, and duration. The tab list renders that summary after a query completes; active execution errors use `error`, and successful single-statement execution uses `ok · <duration>` with the frozen compact duration format. This is only a partial lifecycle match: multi-statement status, empty-run status, and frozen simulated duration remain open.
- Focused application tests drive the real TablePro app through activation, clean and dirty deletion (query edits and pending table edits), structure selection, and query execution outcomes.

The frozen source stores duration from its simulated execution ticks. This increment measures the synchronous candidate execution with `Instant`; its timing is live and has not been proven frame-identical to the frozen simulator. Exact duration parity remains open.

History remains a manually painted, noninteractive workbench body. The tab-activation fix focuses only real query editors and data grids; it intentionally leaves focus restoration to the layer runtime for structure/history views instead of focusing a synthetic component ID. A shared interactive history list and its focus contract remain a separate migration dependency.

The same source review found that tab filtering must use lowercase substring matching and picker height must follow filtered item count. This increment does not change either rule: the shared label filter and picker sizing contract still need a reviewed Termrock implementation. The painted-only structure control registration was removed from the app draw path; other production historical replay paths remain and block the library gate.

## Verification

Passed:

- `mise exec -- cargo fmt --check -p tablepro-domain -p tablepro-ui`
- `mise exec -- cargo nextest run -p tablepro-ui --test tab_list_contract -p tablepro-domain --lib` — 49 passed, 0 skipped.
- Follow-up reviewer fixes add dirty-query deletion, pending-table-edit deletion, and structure-focus regressions. The combined current run `mise exec -- cargo nextest run -p tablepro-ui --test tab_list_contract --test tab_identity_guard --test tab_ownership --test replacement_contract -p tablepro-domain --lib` passes 68/68, 0 skipped.
- `mise exec -- cargo clippy -p tablepro-domain -p tablepro-ui --all-targets -- -D warnings -A clippy::derivable_impls -A clippy::expect_used` — passed. The allowances cover pre-existing lints in `safe_mode.rs` and existing app tests; an unqualified strict run remains red on those lints.

The full TablePro/domain run at `407d727f9f952bafcbb842e8be341ffb32120e4e` and this change both produced the same three failures: the visual connection digest is `a78b53047e73f573` instead of stored `107bc81a6085a09b`; the 500-row insert test cannot focus its target; the 500×14 render test measures 152 allocations against a limit of 100. Logs: [base](../../../../termrock-cli-evidence-20261020/tablepro-base-407-known-failures.log) and [candidate](../../../../termrock-cli-evidence-20261020/tablepro-99-plus-live-contract-known-failures.log). These results establish that those three failures predate this increment.

No paired frozen-CLI comparison has passed for the complete TablePro flow set. No product-parity claim follows from these component tests.
