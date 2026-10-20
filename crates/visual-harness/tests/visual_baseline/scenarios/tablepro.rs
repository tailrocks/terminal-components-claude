use super::{Bin, Root, Variant, interact};
use crate::support::Color;

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[

    Root::new("tablepro/connections/default", Bin::Tablepro, &[], "Production"),
    Root::new("tablepro/connections/form_new", Bin::Tablepro, &[], "Production").sends(&["ctrl-n"]),
    Root::new("tablepro/connections/form_new-filled", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "type:Staging replica", "wait:▎ Staging replica"]),


    Root::new("tablepro/connections/filter", Bin::Tablepro, &[], "Production").sends(&["/", "type:stag", "wait:▎ stag"]),
    Root::new("tablepro/connections/duplicated", Bin::Tablepro, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "alt-d", "wait:(Copy)"]),


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


    Root::new("tablepro/connections/production_detail", Bin::Tablepro, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "sleep:300"]),

    Root::new("tablepro/table/data", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders"]),
    Root::new("tablepro/table/structure", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "ctrl-d", "wait:Columns"]),
    Root::new("tablepro/table/sorted", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "right", "s", "wait:sort created_at ▴"]),
    Root::new("tablepro/table/filtered", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "f", "backtab", "backtab", "type:pending", "enter", "wait:filtered (1)"]),

    Root::new("tablepro/table/sorted-filtered", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "f", "backtab", "backtab", "type:pending", "enter", "wait:filtered (1)", "s", "sleep:300"]),
    Root::new("tablepro/table/cell_editing", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter"]),

    Root::new("tablepro/table/dirty", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "down", "enter", "ctrl-l", "type:shipped", "enter", "wait:2 pending"]),
    Root::new("tablepro/table/row_duplicated", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "alt-d"]),


    Root::new("tablepro/table/scrolled_right", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "end", "wait:updated_at"]),

    Root::new("tablepro/query/results", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:SELECT * FROM ord", "enter", "type: WHERE st", "tab", "type: = 'pending' ORDER BY created_at DESC LIMIT 25", "escape", "ctrl-r", "wait:25 rows", "wait:Ctrl+X Explain"]).timeout(15_000),
    Root::new("tablepro/query/error", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:SELECT * FROM missing_table", "wait:FROM missing_table", "escape", "ctrl-r", "sleep:800"]),
    Root::new("tablepro/query/explain", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:SELECT * FROM orders", "wait:FROM orders", "escape", "ctrl-x", "sleep:500"]),
    Root::new("tablepro/query/new_tab", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-t", "wait:Query 2"]),
    Root::new("tablepro/query/explain_analyze", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:SELECT * FROM orders", "escape", "alt-x", "wait:EXPLAIN ANALYZE"]).timeout(15_000),





    Root::new("tablepro/query/completion_columns", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:x FROM orders o", "home", "type:SELECT o.", "delete", "wait:is_gift"]),

    Root::new("tablepro/workbench/explorer_hidden", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-b"]),
    Root::new("tablepro/workbench/maximized", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["z"]),

    Root::new("tablepro/audit/production", Bin::Tablepro, &["--connect", "Production"], "Query 1"),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[



    Root::new("tablepro/elements/connections/connect-hover", Bin::Tablepro, &[], "Production").hover("Connect    Edit").min_size(100, 30).pending(),
    Root::new("tablepro/elements/connections/connect-pressed", Bin::Tablepro, &["--motion", "paused"], "Production").press("Connect    Edit").min_size(100, 30).pending(),
    Root::new("tablepro/elements/connections/row-hover", Bin::Tablepro, &[], "Production").hover("Staging").pending(),
    Root::new("tablepro/elements/connections/selected-hover", Bin::Tablepro, &[], "Production").sends(&["down"]).hover("Local PostgreSQL").pending(),
    Root::new("tablepro/elements/workbench/tab-hover", Bin::Tablepro, &["--connect", "Production"], "Query 1").hover("Query 1").pending(),
    Root::new("tablepro/elements/workbench/tab-pressed", Bin::Tablepro, &["--connect", "Production", "--motion", "paused", "--frame", "40"], "Query 1").press("Query 1").pending(),
    Root::new("tablepro/elements/table/selected-hover", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders"]).hover("9157cff3").pending(),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[

    Root::new("tablepro/overlays/history_tab", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-y", "wait:History"]),
    Root::new("tablepro/overlays/picker_open", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-o", "wait:Open Quickly"]).dismiss_pending(),
    Root::new("tablepro/overlays/tablist_open", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-g"]).dismiss_pending(),
    Root::new("tablepro/overlays/safemode_picker", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-l", "wait:Safe Mode"]).dismiss_pending(),
    Root::new("tablepro/overlays/help_overlay", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["?", "wait:Keyboard"]).dismiss_pending(),


    Root::new("tablepro/overlays/tablist_tables", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "0", "down", "enter", "wait:public › order_items", "ctrl-g", "wait:public.order_items"]).dismiss_pending(),


    Root::new("tablepro/connections/delete_dialog", Bin::Tablepro, &[], "Production").sends(&["down", "down", "down", "down", "down", "down", "down", "d", "wait:Delete connection?"]).dismiss_pending(),
    Root::new("tablepro/table/filter_editor", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "ctrl-f", "wait:Add filter"]).dismiss_pending(),
    Root::new("tablepro/query/completion", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:SELECT * FROM ord", "wait:ORDER BY"]).dismiss_pending(),



    Root::new("tablepro/query/close_confirm", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["ctrl-t", "type:x", "escape", "ctrl-w", "wait:Close tab with unsaved work?"]).dismiss_pending(),
    Root::new("tablepro/ack/gate", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm"]).dismiss_pending(),



    Root::new("tablepro/workbench/commit_dialog", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "ctrl-s", "wait:Save changes?"]).dismiss_pending(),


    Root::new("tablepro/workbench/quit_confirm", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:x", "escape", "q", "wait:Quit TablePro?"]).dismiss_pending(),

    Root::new("tablepro/overlays/explorer_context", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["wait:S public", "shift-f10"]).pending().dismiss_pending(),
    Root::new("tablepro/overlays/connections_context", Bin::Tablepro, &[], "Production").sends(&["down", "shift-f10"]).pending().dismiss_pending(),

    Root::new("tablepro/overlays/sql_preview", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders", "home", "right", "right", "right", "right", "enter", "ctrl-l", "type:paid", "enter", "p", "wait:SQL preview"]).pending().dismiss_pending(),




    Root::new("tablepro/connections/form_leave_esc", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "type:Staging replica", "tab", "escape", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("tablepro/connections/form_leave_edit", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "type:Staging replica", "tab", "e", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("tablepro/connections/form_leave_new", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "type:Staging replica", "tab", "ctrl-n", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("tablepro/connections/form_quit_guard", Bin::Tablepro, &[], "Production").sends(&["ctrl-n", "type:Staging replica", "ctrl-c", "wait:Unsaved changes"]).pending().dismiss_pending(),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[





    Root::new("tablepro/ack/armed", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm", "type:orders", "tab", "right", "wait:▎Execute"]),
    Root::new("tablepro/ack/executed", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:UPDATE orders SET status = 'paid' WHERE id = 'x'", "escape", "ctrl-r", "wait:Type orders to confirm", "type:orders", "tab", "right", "enter", "wait:rows affected"]),



    Root::new("tablepro/ack/delete_gate", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["tab", "type:DELETE FROM orders", "escape", "ctrl-r", "wait:DELETE without WHERE"]),

    Root::new("tablepro/fade/table_wheel", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["down", "down", "down", "down", "down", "enter", "wait:public › orders"]).live(interact::table_wheel),
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[

    Root::new("tablepro/resize/workbench_shrunk", Bin::Tablepro, &["--connect", "Production"], "Query 1").sends(&["wait:S audit"]).timeout(15_000).resize((120, 40), interact::tablepro_workbench_shrunk),
    Root::new("tablepro/resize/workbench_grown", Bin::Tablepro, &["--connect", "Production"], "Query 1").timeout(15_000).resize((80, 24), interact::tablepro_workbench_grown),

    Root::new("tablepro/resize/connections_from160x50", Bin::Tablepro, &[], "Production").timeout(15_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("tablepro/resize/connections_from72x20", Bin::Tablepro, &[], "Production").timeout(15_000).resize((72, 20), interact::resize_case).pending(),
];
