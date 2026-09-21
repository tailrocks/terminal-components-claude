//! TP-045–057: SQL editor, completion, run, explain, results.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unicode_not_nfc
)]

use junie_tui::{KeyCode, KeyModifiers};
use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_Q, PRESET_QL, alt, click_text,
    ctrl, down, end, enter, esc, home, hover_text, launch_preset_q, launch_preset_ql, left,
    observe, page_down, replace_paste, right, shift, tab, ticks, type_text, wheel_down_text,
};
use tablepro_app::Screen;

fn cap_q(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    scenario: &str,
    checkpoint: &str,
    profile: ColorProfile,
    preset: &str,
) -> Observation {
    observe(harness, scenario, preset, checkpoint, profile)
}

fn each_matrix(mut body: impl FnMut(u16, u16, ColorProfile)) {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            body(width, height, profile);
        }
    }
}

fn wb(obs: &Observation) {
    assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
}

#[test]
fn tp045_editor_multiline_find_split() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_q(
            "SELECT id, order_number\nFROM orders\nWHERE id > 10;",
            width,
            height,
            profile,
        );
        let nav = cap_q(&h, "TP-045", "nav", profile, PRESET_Q);
        type_text(&mut h, "i");
        end(&mut h);
        enter(&mut h);
        let _ = h.paste("-- 文本 é 👩‍💻");
        let edit = cap_q(&h, "TP-045", "edit", profile, PRESET_Q);
        esc(&mut h);
        ctrl(&mut h, 'f');
        type_text(&mut h, "orders");
        let find = cap_q(&h, "TP-045", "find", profile, PRESET_Q);
        enter(&mut h);
        let matched = cap_q(&h, "TP-045", "match", profile, PRESET_Q);
        esc(&mut h);
        let find_closed = cap_q(&h, "TP-045", "find_closed", profile, PRESET_Q);
        let _ = h.key_mod(KeyCode::Up, KeyModifiers::CONTROL);
        let split_up = cap_q(&h, "TP-045", "split_up", profile, PRESET_Q);
        let _ = h.key_mod(KeyCode::Down, KeyModifiers::CONTROL);
        let split_down = cap_q(&h, "TP-045", "split_down", profile, PRESET_Q);
        for obs in [
            &nav,
            &edit,
            &find,
            &matched,
            &find_closed,
            &split_up,
            &split_down,
        ] {
            wb(obs);
        }
    });
}

#[test]
fn tp046_selection_paste_undo() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_q("SELECT * FROM orders LIMIT 5;", width, height, profile);
        type_text(&mut h, "i");
        home(&mut h);
        shift(&mut h, KeyCode::End);
        let selection = cap_q(&h, "TP-046", "selection", profile, PRESET_Q);
        let _ = h.paste("SELECT id FROM customers LIMIT 2;");
        let replaced = cap_q(&h, "TP-046", "replaced", profile, PRESET_Q);
        ctrl(&mut h, 'z');
        let undo = cap_q(&h, "TP-046", "undo", profile, PRESET_Q);
        ctrl(&mut h, 'y');
        let history = cap_q(&h, "TP-046", "history_global_chord", profile, PRESET_Q);
        esc(&mut h);
        let nav = cap_q(&h, "TP-046", "nav", profile, PRESET_Q);
        for obs in [&selection, &replaced, &undo, &history, &nav] {
            wb(obs);
        }
    });
}

#[test]
fn tp047_048_completion() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_q("", width, height, profile);
        type_text(&mut h, "i");
        type_text(&mut h, "SELECT * FROM ord");
        let auto = cap_q(&h, "TP-047", "auto", profile, PRESET_Q);
        down(&mut h, 1);
        let selected = cap_q(&h, "TP-047", "selected", profile, PRESET_Q);
        enter(&mut h);
        let accepted = cap_q(&h, "TP-047", "accepted", profile, PRESET_Q);
        type_text(&mut h, " WHERE ");
        ctrl(&mut h, ' ');
        let manual = cap_q(&h, "TP-047", "manual", profile, PRESET_Q);
        esc(&mut h);
        let dismissed = cap_q(&h, "TP-047", "dismissed", profile, PRESET_Q);
        esc(&mut h);
        let nav = cap_q(&h, "TP-047", "nav", profile, PRESET_Q);
        for obs in [&auto, &selected, &accepted, &manual, &dismissed, &nav] {
            wb(obs);
        }
        let mut alias = launch_preset_q("", width, height, profile);
        type_text(&mut alias, "i");
        type_text(&mut alias, "SELECT o. FROM orders o");
        home(&mut alias);
        right(&mut alias, 9);
        ctrl(&mut alias, ' ');
        let alias_cp = cap_q(&alias, "TP-048", "alias", profile, PRESET_Q);
        hover_text(&mut alias, "order");
        let hovered = cap_q(&alias, "TP-048", "hover", profile, PRESET_Q);
        click_text(&mut alias, "order");
        let accepted2 = cap_q(&alias, "TP-048", "accepted", profile, PRESET_Q);
        esc(&mut alias);
        ctrl(&mut alias, 'f');
        let no_leak = cap_q(&alias, "TP-048", "find_no_popup_leak", profile, PRESET_Q);
        for obs in [&alias_cp, &hovered, &accepted2, &no_leak] {
            wb(obs);
        }
    });
}

#[test]
fn tp049_053_run_batch_cancel() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_q("", width, height, profile);
        ctrl(&mut h, 'r');
        let nothing = cap_q(&h, "TP-049", "nothing", profile, PRESET_Q);
        type_text(&mut h, "i");
        type_text(&mut h, "SELECT * FROM orders LIMIT 5;");
        esc(&mut h);
        ctrl(&mut h, 'r');
        let running0 = cap_q(&h, "TP-049", "running0", profile, PRESET_Q);
        ticks(&mut h, 3);
        let running3 = cap_q(&h, "TP-049", "running3", profile, PRESET_Q);
        ctrl(&mut h, 'r');
        let already = cap_q(&h, "TP-049", "already_running", profile, PRESET_Q);
        ticks(&mut h, 4);
        let rows = cap_q(&h, "TP-049", "rows", profile, PRESET_Q);
        for obs in [&nothing, &running0, &running3, &already, &rows] {
            wb(obs);
        }
        let mut batch = launch_preset_ql(
            "SELECT id FROM orders LIMIT 2;\nSELECT nope FROM orders;\nSELECT id FROM customers LIMIT 2;",
            width,
            height,
            profile,
        );
        alt(&mut batch, 'r');
        let batch_start = cap_q(&batch, "TP-050", "batch_start", profile, PRESET_QL);
        ticks(&mut batch, 7);
        let first = cap_q(&batch, "TP-050", "first_complete", profile, PRESET_QL);
        ticks(&mut batch, 5);
        let second = cap_q(&batch, "TP-050", "second_failed", profile, PRESET_QL);
        ticks(&mut batch, 10);
        let stopped = cap_q(&batch, "TP-050", "stopped", profile, PRESET_QL);
        ctrl(&mut batch, 'y');
        let history = cap_q(&batch, "TP-050", "history", profile, PRESET_QL);
        for obs in [&batch_start, &first, &second, &stopped, &history] {
            wb(obs);
        }
        let mut cur = launch_preset_ql(
            "SELECT id FROM orders LIMIT 2;\nSELECT id FROM customers LIMIT 3;",
            width,
            height,
            profile,
        );
        type_text(&mut cur, "i");
        home(&mut cur);
        let _ = cur.key_mod(KeyCode::Home, KeyModifiers::CONTROL);
        ctrl(&mut cur, 'r');
        let first_running = cap_q(&cur, "TP-051", "first_running", profile, PRESET_QL);
        ticks(&mut cur, 7);
        let first_rows = cap_q(&cur, "TP-051", "first_rows", profile, PRESET_QL);
        esc(&mut cur);
        down(&mut cur, 1);
        home(&mut cur);
        shift(&mut cur, KeyCode::End);
        ctrl(&mut cur, 'r');
        ticks(&mut cur, 7);
        let selection_rows = cap_q(&cur, "TP-051", "selection_rows", profile, PRESET_QL);
        for obs in [&first_running, &first_rows, &selection_rows] {
            wb(obs);
        }
        let mut err = launch_preset_ql("SELECT nope FROM orders", width, height, profile);
        ctrl(&mut err, 'r');
        ticks(&mut err, 7);
        let unknown = cap_q(&err, "TP-052", "unknown_column", profile, PRESET_QL);
        type_text(&mut err, "i");
        replace_paste(&mut err, "SELEC broken");
        esc(&mut err);
        ctrl(&mut err, 'r');
        ticks(&mut err, 7);
        let parse_error = cap_q(&err, "TP-052", "parse_error", profile, PRESET_QL);
        type_text(&mut err, "i");
        type_text(&mut err, "x");
        let cleared = cap_q(&err, "TP-052", "diagnostic_cleared", profile, PRESET_QL);
        for obs in [&unknown, &parse_error, &cleared] {
            wb(obs);
        }
        let mut cancel = launch_preset_q("SELECT * FROM orders", width, height, profile);
        ctrl(&mut cancel, 'r');
        ticks(&mut cancel, 3);
        esc(&mut cancel);
        let cancelled = cap_q(&cancel, "TP-053", "cancelled", profile, PRESET_Q);
        ctrl(&mut cancel, 'r');
        ticks(&mut cancel, 2);
        ctrl(&mut cancel, 'c');
        let ctrl_c = cap_q(&cancel, "TP-053", "ctrl_c_cancelled", profile, PRESET_Q);
        ticks(&mut cancel, 10);
        let no_late = cap_q(&cancel, "TP-053", "no_late_result", profile, PRESET_Q);
        for obs in [&cancelled, &ctrl_c, &no_late] {
            wb(obs);
        }
    });
}

#[test]
fn tp054_057_results_explain_affected() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_q("SELECT * FROM orders LIMIT 5", width, height, profile);
        ctrl(&mut h, 'r');
        ticks(&mut h, 7);
        for _ in 0..8 {
            tab(&mut h);
        }
        type_text(&mut h, "p");
        let pinned = cap_q(&h, "TP-054", "pinned", profile, PRESET_Q);
        ctrl(&mut h, 'r');
        ticks(&mut h, 7);
        let retained = cap_q(&h, "TP-054", "retained", profile, PRESET_Q);
        left(&mut h, 1);
        enter(&mut h);
        let active_first = cap_q(&h, "TP-054", "active_first", profile, PRESET_Q);
        type_text(&mut h, ".");
        let unpinned = cap_q(&h, "TP-054", "unpinned", profile, PRESET_Q);
        type_text(&mut h, "x");
        let closed = cap_q(&h, "TP-054", "closed", profile, PRESET_Q);
        for obs in [&pinned, &retained, &active_first, &unpinned, &closed] {
            wb(obs);
        }
        let mut plan = launch_preset_q(
            "SELECT * FROM orders WHERE id > 10 ORDER BY id LIMIT 5",
            width,
            height,
            profile,
        );
        ctrl(&mut plan, 'x');
        ticks(&mut plan, 7);
        let tree = cap_q(&plan, "TP-055", "plan_tree", profile, PRESET_Q);
        right(&mut plan, 1);
        down(&mut plan, 1);
        let expanded = cap_q(&plan, "TP-055", "expanded", profile, PRESET_Q);
        type_text(&mut plan, "r");
        let raw = cap_q(&plan, "TP-055", "raw", profile, PRESET_Q);
        page_down(&mut plan, 1);
        let raw_scroll = cap_q(&plan, "TP-055", "raw_scroll", profile, PRESET_Q);
        type_text(&mut plan, "r");
        let tree_return = cap_q(&plan, "TP-055", "tree_return", profile, PRESET_Q);
        alt(&mut plan, 'x');
        ticks(&mut plan, 7);
        let analyze = cap_q(&plan, "TP-055", "analyze", profile, PRESET_Q);
        for obs in [&tree, &expanded, &raw, &raw_scroll, &tree_return, &analyze] {
            wb(obs);
        }
        let mut mouse = launch_preset_q("SELECT * FROM orders LIMIT 25", width, height, profile);
        ctrl(&mut mouse, 'r');
        ticks(&mut mouse, 7);
        hover_text(&mut mouse, "id");
        let hovered = cap_q(&mouse, "TP-056", "hover", profile, PRESET_Q);
        click_text(&mut mouse, "1");
        wheel_down_text(&mut mouse, "1", 6);
        let scroll = cap_q(&mouse, "TP-056", "scroll", profile, PRESET_Q);
        let resized = cap_q(&mouse, "TP-056", "resized", profile, PRESET_Q);
        let clamped = cap_q(&mouse, "TP-056", "clamped", profile, PRESET_Q);
        for obs in [&hovered, &scroll, &resized, &clamped] {
            wb(obs);
        }
        let mut aff = launch_preset_ql(
            "UPDATE orders SET status = 'paid' WHERE id = 1",
            width,
            height,
            profile,
        );
        ctrl(&mut aff, 'r');
        ticks(&mut aff, 7);
        let affected = cap_q(&aff, "TP-057", "affected", profile, PRESET_QL);
        ctrl(&mut aff, 't');
        type_text(&mut aff, "i");
        type_text(&mut aff, "SELECT * FROM orders WHERE id = 999999999");
        esc(&mut aff);
        ctrl(&mut aff, 'r');
        ticks(&mut aff, 7);
        let zero = cap_q(&aff, "TP-057", "zero_rows", profile, PRESET_QL);
        wb(&affected);
        wb(&zero);
    });
}
