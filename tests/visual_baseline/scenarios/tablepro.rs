//! tablepro scenario tables: connections form/tree under
//! `tablepro/connections/`, table states under `tablepro/table/`, query
//! flows under `tablepro/query/`, the ack gate chain under `tablepro/ack/`,
//! overlays under `tablepro/overlays/`, workbench chords/layouts under
//! `tablepro/workbench/`, the table wheel-fade, the workbench resize pair and
//! the production audit fixture (`tablepro/audit/`).
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs); only the store names were regrouped. Do not
//! hand-tune: drift against the approved frames means the port or the app
//! changed.

use super::{Bin, Root, Variant, interact};
use crate::support::Color;

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[
    // ------------------------------------------------------------- connections --
    Root::new("tablepro/connections/default", Bin::Tablepro, &[], "Production"),
    Root::new("tablepro/connections/form_new", Bin::Tablepro, &[], "Production").sends(&["ctrl-n"]),
    Root::new("tablepro/connections/form_new-filled", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "type:Staging replica"]),
    // Boot focus is the tree (app.rs:147); ring order is render order:
    // filter → tree → Connect → Edit → Duplicate → Delete (connections.rs:940-1114).
    Root::new("tablepro/connections/filter", Bin::Tablepro, &[], "Production").sends(&["/", "type:stag"]),
    Root::new("tablepro/connections/delete_dialog", Bin::Tablepro, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "d", "wait:Delete connection?"]),
    Root::new("tablepro/connections/duplicated", Bin::Tablepro, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "ctrl-d", "wait:(Copy)"]),
    // ctrl-n lands on the Name input; one BackTab wraps to the form tabs
    // (ring: … → tabs → name → …), Right switches Basic → Advanced.
    Root::new("tablepro/connections/form_advanced", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "wait:Name", "backtab", "right", "sleep:300"]).variants(&[
        Variant::new(72, 20, Color::Truecolor, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(72, 20, Color::Ansi256, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(72, 20, Color::Ansi16, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(72, 20, Color::None, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(72, 20, Color::NoColorEnv, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(80, 24, Color::Truecolor, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(80, 24, Color::Ansi256, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(80, 24, Color::Ansi16, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(80, 24, Color::None, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
        Variant::new(80, 24, Color::NoColorEnv, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "e", "wait:Name", "backtab", "right", "sleep:300"]),
    ]),
    // Tree rows: 0 Personal, 1 Local PostgreSQL, 2 Scratch, 3 Acme, 4 Development,
    // 5 Staging, 6 Analytics, 7 Production — detail card follows the cursor.
    Root::new("tablepro/connections/production_detail", Bin::Tablepro, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "sleep:300"]),
    // ------------------------------------------------------------------ table --
    Root::new("tablepro/table/data", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders"]),
    Root::new("tablepro/table/structure", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "ctrl-d", "wait:Columns"]),
    Root::new("tablepro/table/sorted", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "s", "wait:sort created_at ▴"]),
    Root::new("tablepro/table/filtered", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "f", "backtab", "backtab", "enter", "ctrl-l", "type:pending", "enter", "wait:filtered (1)"]),
    // t_sorted_filtered: filter status='pending' then sort the same column.
    Root::new("tablepro/table/sorted-filtered", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "f", "backtab", "backtab", "enter", "ctrl-l", "type:pending", "enter", "wait:filtered (1)", "s", "sleep:300"]),
    Root::new("tablepro/table/cell_editing", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter"]),
    // t_dirty: 2 pending cell edits, committed (not in-edit, not Save dialog).
    Root::new("tablepro/table/dirty", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "down", "enter", "ctrl-l", "type:shipped", "enter", "wait:2 pending"]),
    Root::new("tablepro/table/row_duplicated", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "alt-d"]),
    Root::new("tablepro/table/filter_editor", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "ctrl-f", "wait:Add filter"]),
    // End jumps the cell cursor to the last column; hscroll follows it
    // (grid.rs:1033 + 847-858) → the ‹N tail window of the 14-column grid.
    Root::new("tablepro/table/scrolled_right", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "end", "wait:updated_at"]),
    // ------------------------------------------------------------------ query --
    Root::new("tablepro/query/completion", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:SELECT * FROM ord", "wait:order_items"]),
    Root::new("tablepro/query/results", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:SELECT * FROM ord", "enter", "type: WHERE st", "tab", "type: = 'pending' ORDER BY created_at DESC LIMIT 25", "escape", "ctrl-r", "wait:25 rows", "wait:Ctrl+X Explain"]).timeout(15_000),
    Root::new("tablepro/query/error", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:SELECT * FROM missing_table", "escape", "ctrl-r", "sleep:800"]),
    Root::new("tablepro/query/explain", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:SELECT * FROM orders", "escape", "ctrl-x", "sleep:500"]),
    Root::new("tablepro/query/new_tab", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-t", "wait:Query 2"]),
    // Dirty the fresh tab so ctrl-w asks (app.rs:625-637): `i` edits, `x` is too
    // short to auto-open completion (model.rs:791-799), escape leaves edit mode.
    Root::new("tablepro/query/close_confirm", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-t", "i", "type:x", "escape", "ctrl-w", "wait:Close tab with unsaved work?"]),
    Root::new("tablepro/query/explain_analyze", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:SELECT * FROM orders", "escape", "alt-x", "wait:EXPLAIN ANALYZE"]).timeout(15_000),
    // Column-level completion (Member clause, model.rs:589): the dot keystroke
    // must be the last edit with the cursor right after "o." (a cursor walk
    // closes the popup mid-path; ctrl-space is not sendable; home on a leading-
    // space text panics context() — model.rs:509). Placeholder x keeps offset 0
    // occupied, then SELECT o. is inserted before it and x forward-deleted.
    Root::new("tablepro/query/completion_columns", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:x FROM orders o", "home", "type:SELECT o.", "delete", "wait:is_gift"]),
    // --------------------------------------------------------------- workbench --
    Root::new("tablepro/workbench/explorer_hidden", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-b"]),
    Root::new("tablepro/workbench/maximized", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["z"]),
    // Cell edit → pending → ctrl-s opens the commit review (app.rs:987): status of
    // row 1, select-all + valid enum value, commit, then Save. Production runs
    // Safe Mode, so the dialog carries the typed-token acknowledgement.
    Root::new("tablepro/workbench/commit_dialog", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "ctrl-s", "wait:Save changes?"]),
    // request_quit only dialogs with unsaved work (app.rs:274-315) — dirty the
    // query first, then q.
    Root::new("tablepro/workbench/quit_confirm", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:x", "escape", "q", "wait:Quit TablePro?"]),
    // ------------------------------------------------------------------ audit --
    Root::new("tablepro/audit/production", Bin::Tablepro, &["--connect", "Production"], "Query 1"),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[
    // ---------------------------------------------------------------- overlays --
    Root::new("tablepro/overlays/history_tab", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-y", "wait:History"]),
    Root::new("tablepro/overlays/picker_open", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-o", "wait:Open Quickly"]),
    Root::new("tablepro/overlays/tablist_open", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-g"]),
    Root::new("tablepro/overlays/safemode_picker", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-l", "wait:Safe Mode"]),
    Root::new("tablepro/overlays/help_overlay", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["?", "wait:Keyboard"]),
    // Three tabs: `0` refocuses the explorer (app.rs:572-579) so a second table
    // can be opened; the overlay marks the active table tab.
    Root::new("tablepro/overlays/tablist_tables", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "0", "down", "enter", "wait:public › order_items", "ctrl-g", "wait:public.order_items"]),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[
    // -------------------------------------------------------------------- ack --
    Root::new("tablepro/ack/gate", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm"]),
    Root::new("tablepro/ack/armed", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm", "enter", "type:orders", "enter", "right", "wait:Execute"]),
    Root::new("tablepro/ack/executed", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm", "enter", "type:orders", "enter", "right", "enter", "wait:rows affected"]),
    // DELETE without WHERE is dangerous (sql.rs:799-802): the gate carries the
    // ON DELETE CASCADE risk line and the not-reversible copy the UPDATE gate
    // lacks.
    Root::new("tablepro/ack/delete_gate", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "i", "type:DELETE FROM orders", "escape", "ctrl-r", "wait:DELETE without WHERE"]),
    // --------------------------------------------------------------- fade --
    Root::new("tablepro/fade/table_wheel", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders"]).live(interact::table_wheel),
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[
    // `wait:S audit` pins the asynchronous schema load before the resize.
    Root::new("tablepro/resize/workbench_shrunk", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["wait:S audit"]).timeout(15_000).resize((120, 40), interact::tablepro_workbench_shrunk),
    Root::new("tablepro/resize/workbench_grown", Bin::Tablepro, &["--connect", "Production"], "Query 1").timeout(15_000).resize((80, 24), interact::tablepro_workbench_grown),
];
