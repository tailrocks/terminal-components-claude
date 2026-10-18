//! tablepro captures (45): connections form/tree under `tablepro/connections/`,
//! table states under `tablepro/table/`, query flows under `tablepro/query/`,
//! the ack gate chain under `tablepro/ack/` (truecolor remapped + nocolor
//! added for the audit-flow matrix), overlays under `tablepro/overlays/`,
//! workbench chords/layouts under `tablepro/workbench/`. The production
//! workbench audit fixture keeps statics only in audit.rs
//! (`tablepro/audit/`, 5x5 matrix — the dedupe rule).
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs); only the store names were regrouped. Do not
//! hand-tune: drift against the approved frames means the port or the app
//! changed.

use crate::support::{Case, Color, TABLEPRO};

// ------------------------------------------------------------- connections --

crate::baseline_case!(tablepro_connections_default_120x40_truecolor => Case::new("tablepro/connections/default/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production"));
crate::baseline_case!(tablepro_connections_form_new_120x40_truecolor => Case::new("tablepro/connections/form_new/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["ctrl-n"]));
crate::baseline_case!(tablepro_connections_form_new_filled_120x40_truecolor => Case::new("tablepro/connections/form_new-filled/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["ctrl-n", "type:Staging replica"]));
// Boot focus is the tree (app.rs:147); ring order is render order:
// filter → tree → Connect → Edit → Duplicate → Delete (connections.rs:940-1114).
crate::baseline_case!(tablepro_connections_filter_120x40_truecolor => Case::new("tablepro/connections/filter/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["/", "type:stag"]));
crate::baseline_case!(tablepro_connections_delete_dialog_120x40_truecolor => Case::new("tablepro/connections/delete_dialog/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "d", "wait:Delete connection?"]));
crate::baseline_case!(tablepro_connections_duplicated_120x40_truecolor => Case::new("tablepro/connections/duplicated/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "ctrl-d", "wait:(Copy)"]));
// ctrl-n lands on the Name input; one BackTab wraps to the form tabs
// (ring: … → tabs → name → …), Right switches Basic → Advanced.
crate::baseline_case_with_variants!(
    tablepro_connections_form_advanced_120x40_truecolor => Case::new("tablepro/connections/form_advanced/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["ctrl-n", "wait:Name", "backtab", "right", "sleep:300"]),
    [
        Case::new("tablepro/connections/form_advanced/72x20/truecolor", TABLEPRO, &[], 72, 20, Color::Truecolor, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/72x20/256", TABLEPRO, &[], 72, 20, Color::Ansi256, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/72x20/16", TABLEPRO, &[], 72, 20, Color::Ansi16, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/72x20/none", TABLEPRO, &[], 72, 20, Color::None, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/72x20/nocolor", TABLEPRO, &[], 72, 20, Color::NoColorEnv, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/80x24/truecolor", TABLEPRO, &[], 80, 24, Color::Truecolor, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/80x24/256", TABLEPRO, &[], 80, 24, Color::Ansi256, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/80x24/16", TABLEPRO, &[], 80, 24, Color::Ansi16, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/80x24/none", TABLEPRO, &[], 80, 24, Color::None, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Case::new("tablepro/connections/form_advanced/80x24/nocolor", TABLEPRO, &[], 80, 24, Color::NoColorEnv, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
    ],
);
// Tree rows: 0 Personal, 1 Local PostgreSQL, 2 Scratch, 3 Acme, 4 Development,
// 5 Staging, 6 Analytics, 7 Production — detail card follows the cursor.
crate::baseline_case!(tablepro_connections_production_detail_120x40_truecolor => Case::new("tablepro/connections/production_detail/120x40/truecolor", TABLEPRO, &[], 120, 40, Color::Truecolor, "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "sleep:300"]));

// ------------------------------------------------------------------ table --

crate::baseline_case!(tablepro_table_data_120x40_truecolor => Case::new("tablepro/table/data/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders"]));
crate::baseline_case!(tablepro_table_structure_120x40_truecolor => Case::new("tablepro/table/structure/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "ctrl-d", "wait:Columns"]));
crate::baseline_case!(tablepro_table_sorted_120x40_truecolor => Case::new("tablepro/table/sorted/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "s", "wait:sort created_at ▴"]));
crate::baseline_case!(tablepro_table_filtered_120x40_truecolor => Case::new("tablepro/table/filtered/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "f", "backtab", "backtab", "enter", "ctrl-l", "type:pending", "enter", "wait:filtered (1)"]));
// t_sorted_filtered: filter status='pending' then sort the same column.
crate::baseline_case!(tablepro_table_sorted_filtered_120x40_truecolor => Case::new("tablepro/table/sorted-filtered/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "f", "backtab", "backtab", "enter", "ctrl-l", "type:pending", "enter", "wait:filtered (1)", "s", "sleep:300"]));
crate::baseline_case!(tablepro_table_cell_editing_120x40_truecolor => Case::new("tablepro/table/cell_editing/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter"]));
// t_dirty: 2 pending cell edits, committed (not in-edit, not Save dialog).
crate::baseline_case!(tablepro_table_dirty_120x40_truecolor => Case::new("tablepro/table/dirty/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "down", "enter", "ctrl-l", "type:shipped", "enter", "wait:2 pending"]));
crate::baseline_case!(tablepro_table_row_duplicated_120x40_truecolor => Case::new("tablepro/table/row_duplicated/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "alt-d"]));
crate::baseline_case!(tablepro_table_filter_editor_120x40_truecolor => Case::new("tablepro/table/filter_editor/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "ctrl-f", "wait:Add filter"]));
// End jumps the cell cursor to the last column; hscroll follows it
// (grid.rs:1033 + 847-858) → the ‹N tail window of the 14-column grid.
crate::baseline_case!(tablepro_table_scrolled_right_120x40_truecolor => Case::new("tablepro/table/scrolled_right/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "end", "wait:updated_at"]));

// ------------------------------------------------------------------ query --

crate::baseline_case!(tablepro_query_completion_120x40_truecolor => Case::new("tablepro/query/completion/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:SELECT * FROM ord", "wait:order_items"]));
crate::baseline_case!(tablepro_query_results_120x40_truecolor => Case::new("tablepro/query/results/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:SELECT * FROM ord", "enter", "type: WHERE st", "tab", "type: = 'pending' ORDER BY created_at DESC LIMIT 25", "escape", "ctrl-r", "wait:25 rows", "wait:Ctrl+X Explain"]).timeout(15_000));
crate::baseline_case!(tablepro_query_error_120x40_truecolor => Case::new("tablepro/query/error/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:SELECT * FROM missing_table", "escape", "ctrl-r", "sleep:800"]));
crate::baseline_case!(tablepro_query_explain_120x40_truecolor => Case::new("tablepro/query/explain/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:SELECT * FROM orders", "escape", "ctrl-x", "sleep:500"]));
crate::baseline_case!(tablepro_query_new_tab_120x40_truecolor => Case::new("tablepro/query/new_tab/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-t", "wait:Query 2"]));
// Dirty the fresh tab so ctrl-w asks (app.rs:625-637): `i` edits, `x` is too
// short to auto-open completion (model.rs:791-799), escape leaves edit mode.
crate::baseline_case!(tablepro_query_close_confirm_120x40_truecolor => Case::new("tablepro/query/close_confirm/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-t", "i", "type:x", "escape", "ctrl-w", "wait:Close tab with unsaved work?"]));
crate::baseline_case!(tablepro_query_explain_analyze_120x40_truecolor => Case::new("tablepro/query/explain_analyze/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:SELECT * FROM orders", "escape", "alt-x", "wait:EXPLAIN ANALYZE"]).timeout(15_000));
// Column-level completion (Member clause, model.rs:589): the dot keystroke
// must be the last edit with the cursor right after "o." (a cursor walk
// closes the popup mid-path; ctrl-space is not sendable; home on a leading-
// space text panics context() — model.rs:509). Placeholder x keeps offset 0
// occupied, then SELECT o. is inserted before it and x forward-deleted.
crate::baseline_case!(tablepro_query_completion_columns_120x40_truecolor => Case::new("tablepro/query/completion_columns/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:x FROM orders o", "home", "type:SELECT o.", "delete", "wait:is_gift"]));

// -------------------------------------------------------------------- ack --

crate::baseline_case!(tablepro_ack_gate_120x40_truecolor => Case::new("tablepro/ack/gate/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm"]));
crate::baseline_case!(tablepro_ack_armed_120x40_truecolor => Case::new("tablepro/ack/armed/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm", "enter", "type:orders", "enter", "right", "wait:Execute"]));
crate::baseline_case!(tablepro_ack_executed_120x40_truecolor => Case::new("tablepro/ack/executed/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm", "enter", "type:orders", "enter", "right", "enter", "wait:rows affected"]));
// DELETE without WHERE is dangerous (sql.rs:799-802): the gate carries the
// ON DELETE CASCADE risk line and the not-reversible copy the UPDATE gate
// lacks.
crate::baseline_case!(tablepro_ack_delete_gate_120x40_truecolor => Case::new("tablepro/ack/delete_gate/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:DELETE FROM orders", "escape", "ctrl-r", "wait:DELETE without WHERE"]));

// ---------------------------------------------------------------- overlays --

crate::baseline_case!(tablepro_overlays_history_tab_120x40_truecolor => Case::new("tablepro/overlays/history_tab/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-y", "wait:History"]));
crate::baseline_case!(tablepro_overlays_picker_open_120x40_truecolor => Case::new("tablepro/overlays/picker_open/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-o", "wait:Open Quickly"]));
crate::baseline_case!(tablepro_overlays_tablist_open_120x40_truecolor => Case::new("tablepro/overlays/tablist_open/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-g"]));
crate::baseline_case!(tablepro_overlays_safemode_picker_120x40_truecolor => Case::new("tablepro/overlays/safemode_picker/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-l", "wait:Safe Mode"]));
crate::baseline_case!(tablepro_overlays_help_overlay_120x40_truecolor => Case::new("tablepro/overlays/help_overlay/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["?", "wait:Keyboard"]));
// Three tabs: `0` refocuses the explorer (app.rs:572-579) so a second table
// can be opened; the overlay marks the active table tab.
crate::baseline_case!(tablepro_overlays_tablist_tables_120x40_truecolor => Case::new("tablepro/overlays/tablist_tables/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "0", "down", "enter", "wait:public › order_items", "ctrl-g", "wait:public.order_items"]));

// --------------------------------------------------------------- workbench --

crate::baseline_case!(tablepro_workbench_explorer_hidden_120x40_truecolor => Case::new("tablepro/workbench/explorer_hidden/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["ctrl-b"]));
crate::baseline_case!(tablepro_workbench_maximized_120x40_truecolor => Case::new("tablepro/workbench/maximized/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["z"]));
// Cell edit → pending → ctrl-s opens the commit review (app.rs:987): status of
// row 1, select-all + valid enum value, commit, then Save. Production runs
// Safe Mode, so the dialog carries the typed-token acknowledgement.
crate::baseline_case!(tablepro_workbench_commit_dialog_120x40_truecolor => Case::new("tablepro/workbench/commit_dialog/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "ctrl-s", "wait:Save changes?"]));
// request_quit only dialogs with unsaved work (app.rs:274-315) — dirty the
// query first, then q.
crate::baseline_case!(tablepro_workbench_quit_confirm_120x40_truecolor => Case::new("tablepro/workbench/quit_confirm/120x40/truecolor", TABLEPRO, &["--connect", "Production"], 120, 40, Color::Truecolor, "Query 1").sends(&["tab", "i", "type:x", "escape", "q", "wait:Quit TablePro?"]));
