//! TP-027–044: table grid, filter, structure. Preset T is W+Ctrl+O+orders+Enter.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use junie_tui::KeyCode;
use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_T, alt, click_text, ctrl, down,
    end, enter, esc, home, hover_text, launch_preset_t, left, observe, page_down, replace_paste,
    right, shift, space, tab, ticks, type_text, wheel_down_text,
};
use tablepro_app::Screen;

fn cap(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    scenario: &str,
    checkpoint: &str,
    profile: ColorProfile,
) -> Observation {
    observe(harness, scenario, PRESET_T, checkpoint, profile)
}

fn each_matrix(mut body: impl FnMut(u16, u16, ColorProfile)) {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            body(width, height, profile);
        }
    }
}

fn assert_wb(obs: &Observation) {
    assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
}

#[test]
fn tp027_grid_traversal_scroll() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        right(&mut h, 1);
        page_down(&mut h, 1);
        let middle = cap(&h, "TP-027", "middle", profile);
        ctrl(&mut h, 'e');
        end(&mut h);
        let bottom = cap(&h, "TP-027", "bottom_right", profile);
        down(&mut h, 1);
        right(&mut h, 1);
        let clamped = cap(&h, "TP-027", "clamped", profile);
        home(&mut h);
        ctrl(&mut h, 'a');
        let top = cap(&h, "TP-027", "top_left", profile);
        home(&mut h);
        end(&mut h);
        let edges = cap(&h, "TP-027", "column_edges", profile);
        for obs in [&middle, &bottom, &clamped, &top, &edges] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp028_grid_mouse_scroll() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        hover_text(&mut h, "order_number");
        let header = cap(&h, "TP-028", "header_hover", profile);
        wheel_down_text(&mut h, "order_number", 12);
        let vscroll = cap(&h, "TP-028", "vscroll", profile);
        let hscroll = cap(&h, "TP-028", "hscroll", profile);
        let bottom = cap(&h, "TP-028", "bottom", profile);
        let right_edge = cap(&h, "TP-028", "right", profile);
        for obs in [&header, &vscroll, &hscroll, &bottom, &right_edge] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp029_grid_selection_copy() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        space(&mut h);
        down(&mut h, 1);
        space(&mut h);
        let selected = cap(&h, "TP-029", "selected", profile);
        for _ in 0..3 {
            shift(&mut h, KeyCode::Down);
        }
        let range = cap(&h, "TP-029", "range", profile);
        type_text(&mut h, "y");
        let copy = cap(&h, "TP-029", "copy", profile);
        type_text(&mut h, "Y");
        let copy_headers = cap(&h, "TP-029", "copy_headers", profile);
        esc(&mut h);
        let cleared = cap(&h, "TP-029", "cleared", profile);
        for obs in [&selected, &range, &copy, &copy_headers, &cleared] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp030_grid_sort() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        right(&mut h, 1);
        type_text(&mut h, "s");
        let asc = cap(&h, "TP-030", "ascending", profile);
        type_text(&mut h, "s");
        let desc = cap(&h, "TP-030", "descending", profile);
        type_text(&mut h, "s");
        let cleared = cap(&h, "TP-030", "cleared", profile);
        click_text(&mut h, "order_number");
        let mouse_sorted = cap(&h, "TP-030", "mouse_sorted", profile);
        type_text(&mut h, "S");
        let clear_sort = cap(&h, "TP-030", "clear_sort", profile);
        for obs in [&asc, &desc, &cleared, &mouse_sorted, &clear_sort] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp031_inline_edit_tab_clicks() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        right(&mut h, 8);
        enter(&mut h);
        ctrl(&mut h, 'l');
        let editing = cap(&h, "TP-031", "editing", profile);
        let _ = h.paste("PARITY-订单");
        let draft = cap(&h, "TP-031", "draft", profile);
        esc(&mut h);
        let cancelled = cap(&h, "TP-031", "cancelled", profile);
        enter(&mut h);
        ctrl(&mut h, 'l');
        let _ = h.paste("PARITY-订单");
        tab(&mut h);
        let committed = cap(&h, "TP-031", "committed_next", profile);
        oracle_tablepro::backtab(&mut h);
        let previous = cap(&h, "TP-031", "previous_source_focus", profile);
        for obs in [&editing, &draft, &cancelled, &committed, &previous] {
            assert_wb(obs);
        }
        let mut clicks = launch_preset_t(width, height, profile);
        down(&mut clicks, 1);
        right(&mut clicks, 8);
        click_text(&mut clicks, "USD");
        let current = cap(&clicks, "TP-031", "current_cell_click_edit", profile);
        esc(&mut clicks);
        left(&mut clicks, 1);
        click_text(&mut clicks, "USD");
        let different = cap(&clicks, "TP-031", "different_cell_click_move", profile);
        click_text(&mut clicks, "USD");
        let repeated = cap(&clicks, "TP-031", "repeated_cell_click_edit", profile);
        for obs in [&current, &different, &repeated] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp032_typed_cells_nullability() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        enter(&mut h);
        let pk = cap(&h, "TP-032", "primary_key", profile);
        esc(&mut h);
        right(&mut h, 6);
        enter(&mut h);
        let _ = h.paste("not-a-number");
        enter(&mut h);
        let invalid = cap(&h, "TP-032", "invalid_number", profile);
        esc(&mut h);
        oracle_tablepro::delete(&mut h);
        let null_request = cap(&h, "TP-032", "null_request", profile);
        for obs in [&pk, &invalid, &null_request] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp033_insert_duplicate_delete_undo() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        type_text(&mut h, "+");
        let inserted = cap(&h, "TP-033", "inserted", profile);
        alt(&mut h, 'd');
        let duplicated = cap(&h, "TP-033", "duplicated", profile);
        type_text(&mut h, "-");
        let deleted = cap(&h, "TP-033", "deleted", profile);
        type_text(&mut h, "u");
        let undo_delete = cap(&h, "TP-033", "undo_delete", profile);
        type_text(&mut h, "u");
        let undo_dup = cap(&h, "TP-033", "undo_duplicate", profile);
        type_text(&mut h, "u");
        let undo_ins = cap(&h, "TP-033", "undo_insert", profile);
        for obs in [
            &inserted,
            &duplicated,
            &deleted,
            &undo_delete,
            &undo_dup,
            &undo_ins,
        ] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp034_036_pending_save_discard() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        right(&mut h, 8);
        enter(&mut h);
        replace_paste(&mut h, "PARITY-1");
        enter(&mut h);
        type_text(&mut h, "p");
        let sql_preview = cap(&h, "TP-034", "sql_preview", profile);
        esc(&mut h);
        ctrl(&mut h, 's');
        let save_gate = cap(&h, "TP-034", "save_gate", profile);
        enter(&mut h);
        type_text(&mut h, "orders");
        enter(&mut h);
        let armed = cap(&h, "TP-034", "armed", profile);
        enter(&mut h);
        let saving = cap(&h, "TP-034", "saving", profile);
        ticks(&mut h, 4);
        let still = cap(&h, "TP-034", "still_pending", profile);
        ticks(&mut h, 1);
        let saved = cap(&h, "TP-034", "saved", profile);
        ctrl(&mut h, 'y');
        let history = cap(&h, "TP-034", "history_entry", profile);
        for obs in [
            &sql_preview,
            &save_gate,
            &armed,
            &saving,
            &still,
            &saved,
            &history,
        ] {
            assert_wb(obs);
        }
        let mut mixed = launch_preset_t(width, height, profile);
        down(&mut mixed, 1);
        right(&mut mixed, 8);
        enter(&mut mixed);
        replace_paste(&mut mixed, "PARITY-1");
        enter(&mut mixed);
        type_text(&mut mixed, "+");
        alt(&mut mixed, 'd');
        type_text(&mut mixed, "-");
        type_text(&mut mixed, "p");
        let mixed_preview = cap(&mixed, "TP-035", "mixed_preview", profile);
        esc(&mut mixed);
        ctrl(&mut mixed, 's');
        let delete_review = cap(&mixed, "TP-035", "delete_review", profile);
        esc(&mut mixed);
        let pending = cap(&mixed, "TP-035", "pending_preserved", profile);
        for obs in [&mixed_preview, &delete_review, &pending] {
            assert_wb(obs);
        }
        let mut disc = launch_preset_t(width, height, profile);
        down(&mut disc, 1);
        right(&mut disc, 8);
        enter(&mut disc);
        replace_paste(&mut disc, "PARITY-1");
        enter(&mut disc);
        oracle_tablepro::function(&mut disc, 5);
        let refresh_gate = cap(&disc, "TP-036", "refresh_gate", profile);
        esc(&mut disc);
        let kept = cap(&disc, "TP-036", "kept", profile);
        type_text(&mut disc, "U");
        let discard_gate = cap(&disc, "TP-036", "discard_gate", profile);
        click_text(&mut disc, "Discard");
        let discarded = cap(&disc, "TP-036", "discarded", profile);
        oracle_tablepro::function(&mut disc, 5);
        let refreshed = cap(&disc, "TP-036", "refreshed", profile);
        for obs in [&refresh_gate, &kept, &discard_gate, &discarded, &refreshed] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp037_038_viewer_fetch() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        right(&mut h, 4);
        ctrl(&mut h, ']');
        let reference = cap(&h, "TP-037", "reference_target", profile);
        ctrl(&mut h, 'o');
        type_text(&mut h, "events");
        enter(&mut h);
        down(&mut h, 1);
        right(&mut h, 10);
        enter(&mut h);
        let json = cap(&h, "TP-037", "json_viewer", profile);
        page_down(&mut h, 1);
        let scroll = cap(&h, "TP-037", "viewer_scroll", profile);
        esc(&mut h);
        let restored = cap(&h, "TP-037", "focus_restored", profile);
        for obs in [&reference, &json, &scroll, &restored] {
            assert_wb(obs);
        }
        let mut notes = launch_preset_t(width, height, profile);
        down(&mut notes, 1);
        right(&mut notes, 8);
        let selected = cap(&notes, "TP-037", "notes_keyboard_selected", profile);
        click_text(&mut notes, "notes");
        let actual_click = cap(&notes, "TP-037", "notes_actual_click", profile);
        enter(&mut notes);
        let actual_enter = cap(&notes, "TP-037", "notes_actual_enter", profile);
        let _ = notes.paste("PARITY-1");
        let no_mut = cap(&notes, "TP-037", "notes_paste_no_mutation", profile);
        for obs in [&selected, &actual_click, &actual_enter, &no_mut] {
            assert_wb(obs);
        }
        let mut fetch = launch_preset_t(width, height, profile);
        type_text(&mut fetch, "G");
        let fetch_row = cap(&fetch, "TP-038", "fetch_row", profile);
        enter(&mut fetch);
        let fetch_status = cap(&fetch, "TP-038", "fetch_status", profile);
        down(&mut fetch, 1);
        let bottom = cap(&fetch, "TP-038", "bottom_boundary", profile);
        for obs in [&fetch_row, &fetch_status, &bottom] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp039_042_filter_editor() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        ctrl(&mut h, 'f');
        let filter = cap(&h, "TP-039", "filter", profile);
        tab(&mut h);
        enter(&mut h);
        type_text(&mut h, "1");
        tab(&mut h);
        click_text(&mut h, "Add filter");
        let chip = cap(&h, "TP-039", "chip", profile);
        click_text(&mut h, "1");
        let edit = cap(&h, "TP-039", "edit", profile);
        enter(&mut h);
        ctrl(&mut h, 'l');
        let _ = h.paste("2");
        tab(&mut h);
        click_text(&mut h, "Cancel");
        let original = cap(&h, "TP-039", "original", profile);
        click_text(&mut h, "1");
        enter(&mut h);
        ctrl(&mut h, 'l');
        let _ = h.paste("2");
        tab(&mut h);
        click_text(&mut h, "Update filter");
        let updated = cap(&h, "TP-039", "updated", profile);
        for obs in [&filter, &chip, &edit, &original, &updated] {
            assert_wb(obs);
        }
        let mut ops = launch_preset_t(width, height, profile);
        ctrl(&mut ops, 'f');
        let between = cap(&ops, "TP-040", "between", profile);
        enter(&mut ops);
        ctrl(&mut ops, 'l');
        let _ = ops.paste("10");
        tab(&mut ops);
        enter(&mut ops);
        ctrl(&mut ops, 'l');
        let _ = ops.paste("20");
        let upper_ignored = cap(&ops, "TP-040", "upper_paste_ignored", profile);
        type_text(&mut ops, "20");
        let upper_typed = cap(&ops, "TP-040", "upper_typed", profile);
        tab(&mut ops);
        click_text(&mut ops, "Add filter");
        let ranged = cap(&ops, "TP-040", "ranged", profile);
        ctrl(&mut ops, 'f');
        let no_value = cap(&ops, "TP-040", "no_value", profile);
        click_text(&mut ops, "Add filter");
        let null_filter = cap(&ops, "TP-040", "null_filter", profile);
        for obs in [
            &between,
            &upper_ignored,
            &upper_typed,
            &ranged,
            &no_value,
            &null_filter,
        ] {
            assert_wb(obs);
        }
        let mut early = launch_preset_t(width, height, profile);
        ctrl(&mut early, 'f');
        replace_paste(&mut early, "10");
        let early_apply = cap(&early, "TP-040", "lower_enter_applied_early", profile);
        assert_wb(&early_apply);
        let mut invalid = launch_preset_t(width, height, profile);
        ctrl(&mut invalid, 'f');
        enter(&mut invalid);
        ctrl(&mut invalid, 'l');
        let _ = invalid.paste("not-number");
        tab(&mut invalid);
        click_text(&mut invalid, "Add filter");
        let invalid_value = cap(&invalid, "TP-041", "invalid_value_oracle", profile);
        esc(&mut invalid);
        ctrl(&mut invalid, 'f');
        enter(&mut invalid);
        ctrl(&mut invalid, 'l');
        let _ = invalid.paste("999999999");
        tab(&mut invalid);
        click_text(&mut invalid, "Add filter");
        let no_rows = cap(&invalid, "TP-041", "no_rows", profile);
        type_text(&mut invalid, "F");
        let reset = cap(&invalid, "TP-041", "reset", profile);
        for obs in [&invalid_value, &no_rows, &reset] {
            assert_wb(obs);
        }
        let mut chips = launch_preset_t(width, height, profile);
        down(&mut chips, 1);
        type_text(&mut chips, "f");
        let prefill = cap(&chips, "TP-042", "cell_prefill", profile);
        click_text(&mut chips, "Add filter");
        ctrl(&mut chips, 'f');
        enter(&mut chips);
        ctrl(&mut chips, 'l');
        let _ = chips.paste("1");
        tab(&mut chips);
        click_text(&mut chips, "Add filter");
        let all = cap(&chips, "TP-042", "all", profile);
        let inventory = cap(&chips, "TP-042", "chip_focus_inventory", profile);
        click_text(&mut chips, "ALL");
        click_text(&mut chips, "ANY");
        let any = cap(&chips, "TP-042", "any", profile);
        enter(&mut chips);
        let editor = cap(&chips, "TP-042", "chip_editor_not_lead", profile);
        esc(&mut chips);
        space(&mut chips);
        let disabled = cap(&chips, "TP-042", "disabled", profile);
        oracle_tablepro::delete(&mut chips);
        let removed = cap(&chips, "TP-042", "removed", profile);
        type_text(&mut chips, "X");
        let cleared = cap(&chips, "TP-042", "cleared", profile);
        for obs in [
            &prefill, &all, &inventory, &any, &editor, &disabled, &removed, &cleared,
        ] {
            assert_wb(obs);
        }
    });
}

#[test]
fn tp043_044_structure() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        ctrl(&mut h, 'd');
        let columns = cap(&h, "TP-043", "columns", profile);
        tab(&mut h);
        right(&mut h, 1);
        enter(&mut h);
        let indexes = cap(&h, "TP-043", "indexes", profile);
        right(&mut h, 1);
        enter(&mut h);
        let fks = cap(&h, "TP-043", "foreign_keys", profile);
        right(&mut h, 1);
        enter(&mut h);
        let ddl = cap(&h, "TP-043", "ddl", profile);
        page_down(&mut h, 1);
        end(&mut h);
        let ddl_bottom = cap(&h, "TP-043", "ddl_bottom", profile);
        ctrl(&mut h, 'd');
        let data = cap(&h, "TP-043", "data", profile);
        for obs in [&columns, &indexes, &fks, &ddl, &ddl_bottom, &data] {
            assert_wb(obs);
        }
        let mut mouse = launch_preset_t(width, height, profile);
        click_text(&mut mouse, "Structure");
        click_text(&mut mouse, "Indexes");
        hover_text(&mut mouse, "Name");
        click_text(&mut mouse, "Name");
        let sorted = cap(&mouse, "TP-044", "sorted", profile);
        wheel_down_text(&mut mouse, "Name", 8);
        let scrolled = cap(&mouse, "TP-044", "scrolled", profile);
        type_text(&mut mouse, "+");
        type_text(&mut mouse, "-");
        alt(&mut mouse, 'd');
        let read_only = cap(&mouse, "TP-044", "read_only_noop", profile);
        click_text(&mut mouse, "Data");
        let data_mode = cap(&mouse, "TP-044", "data", profile);
        for obs in [&sorted, &scrolled, &read_only, &data_mode] {
            assert_wb(obs);
        }
    });
}
