//! TP-058–080: safety, history, switcher, overlays, shell, lifecycle.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use core::time::Duration;

use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_C, PRESET_Q, PRESET_T, PRESET_W,
    alt, click_text, ctrl, down, end, enter, esc, function, home, hover_text, launch_preset_c,
    launch_preset_q, launch_preset_t, launch_preset_w, observe, page_down, replace_paste, tab,
    ticks, type_text, wheel_down_text,
};
use tablepro_app::Screen;

fn cap(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    scenario: &str,
    preset: &str,
    checkpoint: &str,
    profile: ColorProfile,
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

#[test]
fn tp058_062_safety_modes() {
    each_matrix(|width, height, profile| {
        for mode_index in 0..6 {
            let mut h = launch_preset_w(width, height, profile);
            ctrl(&mut h, 'l');
            home(&mut h);
            down(&mut h, mode_index);
            enter(&mut h);
            let mode = cap(&h, "TP-058", "mode", PRESET_W, profile);
            assert_eq!(mode.screen, Screen::Workbench);
            ctrl(&mut h, 't');
            type_text(&mut h, "i");
            type_text(&mut h, "SELECT id FROM orders LIMIT 1");
            esc(&mut h);
            ctrl(&mut h, 'r');
            let read_gate = cap(&h, "TP-058", "read_gate", PRESET_W, profile);
            esc(&mut h);
            ticks(&mut h, 7);
            ctrl(&mut h, 't');
            type_text(&mut h, "i");
            type_text(&mut h, "UPDATE orders SET status = 'paid' WHERE id = 1");
            esc(&mut h);
            ctrl(&mut h, 'r');
            let write_gate = cap(&h, "TP-058", "write_gate", PRESET_W, profile);
            esc(&mut h);
            ctrl(&mut h, 't');
            type_text(&mut h, "i");
            type_text(&mut h, "DELETE FROM orders");
            esc(&mut h);
            ctrl(&mut h, 'r');
            let danger = cap(&h, "TP-058", "danger_gate", PRESET_W, profile);
            assert_eq!(read_gate.screen, Screen::Workbench);
            assert_eq!(write_gate.screen, Screen::Workbench);
            assert_eq!(danger.screen, Screen::Workbench);
        }
        let mut token = launch_preset_q("DELETE FROM orders", width, height, profile);
        ctrl(&mut token, 'r');
        let gate = cap(&token, "TP-059", "gate", PRESET_Q, profile);
        enter(&mut token);
        type_text(&mut token, "wrong");
        enter(&mut token);
        let disabled = cap(&token, "TP-059", "disabled", PRESET_Q, profile);
        enter(&mut token);
        let still_open = cap(&token, "TP-059", "still_open", PRESET_Q, profile);
        replace_paste(&mut token, "orders");
        let armed = cap(&token, "TP-059", "armed", PRESET_Q, profile);
        enter(&mut token);
        let running = cap(&token, "TP-059", "running", PRESET_Q, profile);
        ticks(&mut token, 7);
        let executed = cap(&token, "TP-059", "executed", PRESET_Q, profile);
        for obs in [&gate, &disabled, &still_open, &armed, &running, &executed] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut mouse = launch_preset_q("DELETE FROM orders", width, height, profile);
        ctrl(&mut mouse, 'r');
        click_text(&mut mouse, "acknowledgement");
        let edit = cap(&mouse, "TP-060", "edit", PRESET_Q, profile);
        let _ = mouse.paste("orders");
        let armed_m = cap(&mouse, "TP-060", "armed", PRESET_Q, profile);
        hover_text(&mut mouse, "Execute");
        oracle_tablepro::pointer_down_text(&mut mouse, "Execute");
        let pressed = cap(&mouse, "TP-060", "pressed", PRESET_Q, profile);
        oracle_tablepro::pointer_up_text(&mut mouse, "Execute");
        let running_m = cap(&mouse, "TP-060", "running", PRESET_Q, profile);
        ticks(&mut mouse, 7);
        let executed_m = cap(&mouse, "TP-060", "executed", PRESET_Q, profile);
        for obs in [&edit, &armed_m, &pressed, &running_m, &executed_m] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut cancel = launch_preset_q("DELETE FROM orders", width, height, profile);
        ctrl(&mut cancel, 'r');
        let gate2 = cap(&cancel, "TP-061", "gate", PRESET_Q, profile);
        click_text(&mut cancel, "orders");
        let backdrop = cap(&cancel, "TP-061", "backdrop", PRESET_Q, profile);
        ctrl(&mut cancel, 't');
        let no_new = cap(&cancel, "TP-061", "no_new_tab", PRESET_Q, profile);
        esc(&mut cancel);
        let cancelled = cap(&cancel, "TP-061", "cancelled", PRESET_Q, profile);
        ticks(&mut cancel, 20);
        let no_exec = cap(&cancel, "TP-061", "no_execution", PRESET_Q, profile);
        for obs in [&gate2, &backdrop, &no_new, &cancelled, &no_exec] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut persist = launch_preset_w(width, height, profile);
        ctrl(&mut persist, 'l');
        let picker = cap(&persist, "TP-062", "picker", PRESET_W, profile);
        home(&mut persist);
        down(&mut persist, 5);
        enter(&mut persist);
        let readonly = cap(&persist, "TP-062", "readonly", PRESET_W, profile);
        ctrl(&mut persist, 'l');
        esc(&mut persist);
        let kept = cap(&persist, "TP-062", "kept", PRESET_W, profile);
        click_text(&mut persist, "Production");
        let connection_screen = cap(
            &persist,
            "TP-062",
            "connection_screen_oracle",
            PRESET_W,
            profile,
        );
        type_text(&mut persist, "/");
        type_text(&mut persist, "Production");
        enter(&mut persist);
        down(&mut persist, 1);
        enter(&mut persist);
        ticks(&mut persist, 12);
        let persisted = cap(&persist, "TP-062", "persisted", PRESET_W, profile);
        assert_eq!(picker.screen, Screen::Workbench);
        assert_eq!(readonly.screen, Screen::Workbench);
        assert_eq!(kept.screen, Screen::Workbench);
        assert!(matches!(
            connection_screen.screen,
            Screen::Workbench | Screen::Connections
        ));
        assert!(matches!(
            persisted.screen,
            Screen::Workbench | Screen::Connections
        ));
    });
}

#[test]
fn tp063_065_dirty_save_quit() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_t(width, height, profile);
        down(&mut h, 1);
        oracle_tablepro::right(&mut h, 8);
        enter(&mut h);
        replace_paste(&mut h, "PARITY");
        enter(&mut h);
        ctrl(&mut h, 'l');
        end(&mut h);
        enter(&mut h);
        ctrl(&mut h, 's');
        let refused = cap(&h, "TP-063", "refused", PRESET_T, profile);
        type_text(&mut h, "p");
        let preview = cap(&h, "TP-063", "preview_still_available", PRESET_T, profile);
        esc(&mut h);
        let pending = cap(&h, "TP-063", "pending_kept", PRESET_T, profile);
        for obs in [&refused, &preview, &pending] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut dirty = launch_preset_q("SELECT 1", width, height, profile);
        type_text(&mut dirty, "i");
        end(&mut dirty);
        type_text(&mut dirty, " ");
        esc(&mut dirty);
        ctrl(&mut dirty, 'w');
        let close_gate = cap(&dirty, "TP-064", "close_gate", PRESET_Q, profile);
        esc(&mut dirty);
        let kept = cap(&dirty, "TP-064", "kept", PRESET_Q, profile);
        type_text(&mut dirty, "q");
        let quit_gate = cap(&dirty, "TP-064", "quit_gate", PRESET_Q, profile);
        esc(&mut dirty);
        let kept_again = cap(&dirty, "TP-064", "kept_again", PRESET_Q, profile);
        ctrl(&mut dirty, 'c');
        let ctrl_c = cap(&dirty, "TP-064", "ctrl_c_gate", PRESET_Q, profile);
        enter(&mut dirty);
        let _ = cap(&dirty, "TP-064", "assert_exit", PRESET_Q, profile);
        for obs in [&close_gate, &kept, &quit_gate, &kept_again, &ctrl_c] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut mixed = launch_preset_t(width, height, profile);
        down(&mut mixed, 1);
        oracle_tablepro::right(&mut mixed, 8);
        enter(&mut mixed);
        replace_paste(&mut mixed, "PARITY");
        enter(&mut mixed);
        ctrl(&mut mixed, 't');
        type_text(&mut mixed, "i");
        type_text(&mut mixed, "SELECT 1");
        esc(&mut mixed);
        type_text(&mut mixed, "q");
        let mixed_dirty = cap(&mixed, "TP-065", "mixed_dirty", PRESET_T, profile);
        esc(&mut mixed);
        type_text(&mut mixed, "[");
        let original_grid = cap(&mixed, "TP-065", "original_grid", PRESET_T, profile);
        type_text(&mut mixed, "]");
        let original_query = cap(&mixed, "TP-065", "original_query", PRESET_T, profile);
        for obs in [&mixed_dirty, &original_grid, &original_query] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
    });
}

#[test]
fn tp066_071_history_switcher() {
    each_matrix(|width, height, profile| {
        let mut h = launch_preset_w(width, height, profile);
        ctrl(&mut h, 'y');
        let history = cap(&h, "TP-066", "history", PRESET_W, profile);
        type_text(&mut h, "/");
        type_text(&mut h, "orders error");
        let search = cap(&h, "TP-066", "search", PRESET_W, profile);
        esc(&mut h);
        let cancelled = cap(&h, "TP-066", "cancelled", PRESET_W, profile);
        type_text(&mut h, "s");
        let failed = cap(&h, "TP-066", "failed", PRESET_W, profile);
        type_text(&mut h, "c");
        let all_conn = cap(&h, "TP-066", "all_connections", PRESET_W, profile);
        type_text(&mut h, "/");
        type_text(&mut h, "zzzz_no_match");
        let empty = cap(&h, "TP-066", "empty", PRESET_W, profile);
        esc(&mut h);
        home(&mut h);
        end(&mut h);
        oracle_tablepro::page_up(&mut h, 1);
        page_down(&mut h, 1);
        let scroll = cap(&h, "TP-066", "scroll", PRESET_W, profile);
        for obs in [
            &history, &search, &cancelled, &failed, &all_conn, &empty, &scroll,
        ] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut open = launch_preset_w(width, height, profile);
        ctrl(&mut open, 'y');
        down(&mut open, 1);
        let detail = cap(&open, "TP-067", "detail", PRESET_W, profile);
        type_text(&mut open, "y");
        let copy = cap(&open, "TP-067", "copy", PRESET_W, profile);
        enter(&mut open);
        let opened = cap(&open, "TP-067", "opened", PRESET_W, profile);
        ctrl(&mut open, 'y');
        let reused = cap(&open, "TP-067", "reused_history", PRESET_W, profile);
        type_text(&mut open, "r");
        let rerun = cap(&open, "TP-067", "rerun_or_gate", PRESET_W, profile);
        esc(&mut open);
        ticks(&mut open, 7);
        ctrl(&mut open, 'y');
        click_text(&mut open, "Copy");
        let mouse_copy = cap(&open, "TP-067", "mouse_copy", PRESET_W, profile);
        click_text(&mut open, "Open");
        let mouse_open = cap(&open, "TP-067", "mouse_open", PRESET_W, profile);
        for obs in [
            &detail,
            &copy,
            &opened,
            &reused,
            &rerun,
            &mouse_copy,
            &mouse_open,
        ] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut det = launch_preset_w(width, height, profile);
        ctrl(&mut det, 'y');
        click_text(&mut det, "SELECT");
        let selected = cap(&det, "TP-068", "selected", PRESET_W, profile);
        wheel_down_text(&mut det, "SELECT", 8);
        let list_scroll = cap(&det, "TP-068", "list_scroll", PRESET_W, profile);
        click_text(&mut det, "SELECT");
        wheel_down_text(&mut det, "SELECT", 8);
        let detail_scroll = cap(&det, "TP-068", "detail_scroll", PRESET_W, profile);
        let bottom = cap(&det, "TP-068", "bottom", PRESET_W, profile);
        for obs in [&selected, &list_scroll, &detail_scroll, &bottom] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut sw = launch_preset_w(width, height, profile);
        ctrl(&mut sw, 'o');
        let open_sw = cap(&sw, "TP-069", "open", PRESET_W, profile);
        type_text(&mut sw, "ord");
        let ranked = cap(&sw, "TP-069", "ranked", PRESET_W, profile);
        tab(&mut sw);
        let s1 = cap(&sw, "TP-069", "scope1", PRESET_W, profile);
        tab(&mut sw);
        let s2 = cap(&sw, "TP-069", "scope2", PRESET_W, profile);
        tab(&mut sw);
        let s3 = cap(&sw, "TP-069", "scope3", PRESET_W, profile);
        tab(&mut sw);
        let s0 = cap(&sw, "TP-069", "scope0", PRESET_W, profile);
        esc(&mut sw);
        let clear = cap(&sw, "TP-069", "clear", PRESET_W, profile);
        esc(&mut sw);
        let closed = cap(&sw, "TP-069", "closed", PRESET_W, profile);
        ctrl(&mut sw, 'p');
        let alias = cap(&sw, "TP-069", "alias", PRESET_W, profile);
        for obs in [
            &open_sw, &ranked, &s1, &s2, &s3, &s0, &clear, &closed, &alias,
        ] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        for target in [
            "orders",
            "active_customers",
            "audit",
            "acme_prod",
            "Query 1",
        ] {
            let mut t = launch_preset_w(width, height, profile);
            ctrl(&mut t, 'o');
            type_text(&mut t, target);
            let candidate = cap(&t, "TP-070", "candidate", PRESET_W, profile);
            enter(&mut t);
            let opened_t = cap(&t, "TP-070", "opened", PRESET_W, profile);
            assert_eq!(candidate.screen, Screen::Workbench);
            assert_eq!(opened_t.screen, Screen::Workbench);
        }
        let mut empty = launch_preset_w(width, height, profile);
        ctrl(&mut empty, 'o');
        type_text(&mut empty, "zzzz_no_match");
        let empty_sw = cap(&empty, "TP-071", "empty", PRESET_W, profile);
        enter(&mut empty);
        let stays = cap(&empty, "TP-071", "stays_open", PRESET_W, profile);
        esc(&mut empty);
        type_text(&mut empty, "orders");
        alt(&mut empty, '\n');
        enter(&mut empty);
        let opened_alt = cap(&empty, "TP-071", "opened_alt", PRESET_W, profile);
        ctrl(&mut empty, 'o');
        type_text(&mut empty, "customers");
        hover_text(&mut empty, "customers");
        let hovered = cap(&empty, "TP-071", "hover", PRESET_W, profile);
        click_text(&mut empty, "customers");
        let opened_mouse = cap(&empty, "TP-071", "opened_mouse", PRESET_W, profile);
        for obs in [&empty_sw, &stays, &opened_alt, &hovered, &opened_mouse] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        for chord in ['o', 'g'] {
            let mut paste = launch_preset_w(width, height, profile);
            ctrl(&mut paste, 't');
            type_text(&mut paste, "i");
            type_text(&mut paste, "SELECT ");
            ctrl(&mut paste, chord);
            let over = cap(
                &paste,
                "TP-071",
                "picker_over_editing_query",
                PRESET_W,
                profile,
            );
            let _ = paste.paste("PARITY");
            let fallthrough = cap(&paste, "TP-071", "paste_fallthrough", PRESET_W, profile);
            esc(&mut paste);
            let revealed = cap(&paste, "TP-071", "editor_revealed", PRESET_W, profile);
            for obs in [&over, &fallthrough, &revealed] {
                assert_eq!(obs.screen, Screen::Workbench);
            }
        }
    });
}

#[test]
fn tp072_080_overlays_shell() {
    each_matrix(|width, height, profile| {
        for preset_c in [true, false] {
            let mut h = if preset_c {
                launch_preset_c(width, height, profile)
            } else {
                launch_preset_w(width, height, profile)
            };
            type_text(&mut h, "?");
            let help = cap(
                &h,
                "TP-072",
                "help",
                if preset_c { PRESET_C } else { PRESET_W },
                profile,
            );
            let ring = h.ring().entries().len().max(1);
            for _ in 0..ring {
                tab(&mut h);
            }
            page_down(&mut h, 1);
            let scrolled = cap(&h, "TP-072", "scrolled", help.preset.as_str(), profile);
            end(&mut h);
            let bottom = cap(&h, "TP-072", "bottom", help.preset.as_str(), profile);
            wheel_down_text(&mut h, "TablePro", 1);
            let wheel = cap(&h, "TP-072", "wheel", help.preset.as_str(), profile);
            esc(&mut h);
            let restored = cap(&h, "TP-072", "restored", help.preset.as_str(), profile);
            assert!(!help.text.is_empty());
            assert!(!scrolled.text.is_empty());
            assert!(!bottom.text.is_empty());
            assert!(!wheel.text.is_empty());
            assert!(!restored.text.is_empty());
        }
        let mut nest = launch_preset_t(width, height, profile);
        ctrl(&mut nest, 'f');
        enter(&mut nest);
        let nested = cap(&nest, "TP-073", "nested", PRESET_T, profile);
        let _ = nest.resize(80, 24);
        let narrow = cap(&nest, "TP-073", "narrow", PRESET_T, profile);
        wheel_down_text(&mut nest, "Column", 8);
        let scroll = cap(&nest, "TP-073", "scroll", PRESET_T, profile);
        esc(&mut nest);
        let select_closed = cap(&nest, "TP-073", "select_closed", PRESET_T, profile);
        esc(&mut nest);
        let filter_closed = cap(&nest, "TP-073", "filter_closed", PRESET_T, profile);
        let _ = nest.resize(160, 50);
        let restored = cap(&nest, "TP-073", "restored", PRESET_T, profile);
        for obs in [
            &nested,
            &narrow,
            &scroll,
            &select_closed,
            &filter_closed,
            &restored,
        ] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut color = launch_preset_c(width, height, profile);
        let initial = cap(&color, "TP-075", "initial", PRESET_C, profile);
        hover_text(&mut color, "Local PostgreSQL");
        let hovered = cap(&color, "TP-075", "hover", PRESET_C, profile);
        type_text(&mut color, "i");
        let selected = cap(&color, "TP-075", "selected", PRESET_C, profile);
        type_text(&mut color, "?");
        let overlay = cap(&color, "TP-075", "overlay", PRESET_C, profile);
        assert_eq!(initial.profile, profile.label());
        assert_eq!(hovered.screen, Screen::Connections);
        assert_eq!(selected.screen, Screen::Connections);
        assert_eq!(overlay.screen, Screen::Connections);
        let mut routing = launch_preset_t(width, height, profile);
        function(&mut routing, 10);
        let noop = cap(&routing, "TP-077", "noop", PRESET_T, profile);
        type_text(&mut routing, "?");
        let _ = routing.paste("SELECT 1");
        let no_leak = cap(&routing, "TP-077", "no_leak", PRESET_T, profile);
        esc(&mut routing);
        ctrl(&mut routing, 'd');
        alt(&mut routing, 'd');
        let structure = cap(
            &routing,
            "TP-077",
            "structure_no_duplicate",
            PRESET_T,
            profile,
        );
        ctrl(&mut routing, 'd');
        down(&mut routing, 1);
        oracle_tablepro::right(&mut routing, 8);
        enter(&mut routing);
        ctrl(&mut routing, 'l');
        type_text(&mut routing, "q?z[]");
        let literal = cap(&routing, "TP-077", "literal_input", PRESET_T, profile);
        for obs in [&noop, &no_leak, &structure, &literal] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
        let mut strip = launch_preset_w(width, height, profile);
        hover_text(&mut strip, "Safe");
        let hover_safe = cap(&strip, "TP-080", "hover_safe", PRESET_W, profile);
        click_text(&mut strip, "Safe");
        let safety = cap(&strip, "TP-080", "safety", PRESET_W, profile);
        esc(&mut strip);
        click_text(&mut strip, "acme");
        let scope = cap(&strip, "TP-080", "scope_switcher", PRESET_W, profile);
        esc(&mut strip);
        click_text(&mut strip, "?");
        let help = cap(&strip, "TP-080", "help", PRESET_W, profile);
        esc(&mut strip);
        click_text(&mut strip, "Production");
        let connections = cap(&strip, "TP-080", "connections", PRESET_W, profile);
        assert_eq!(hover_safe.screen, Screen::Workbench);
        assert_eq!(safety.screen, Screen::Workbench);
        assert_eq!(scope.screen, Screen::Workbench);
        assert_eq!(help.screen, Screen::Workbench);
        assert!(matches!(
            connections.screen,
            Screen::Workbench | Screen::Connections
        ));
    });
    for (width, height) in [
        (71, 20),
        (72, 19),
        (72, 20),
        (80, 24),
        (100, 30),
        (120, 40),
        (160, 50),
    ] {
        for profile in COLOR_PROFILES {
            let mut h = launch_preset_t(width, height, profile);
            let initial = cap(&h, "TP-074", "initial", PRESET_T, profile);
            let _ = h.resize(71, 20);
            let too_narrow = cap(&h, "TP-074", "too_narrow", PRESET_T, profile);
            ctrl(&mut h, 't');
            let noop = cap(&h, "TP-074", "noop", PRESET_T, profile);
            let _ = h.resize(72, 19);
            let too_short = cap(&h, "TP-074", "too_short", PRESET_T, profile);
            let _ = h.resize(72, 20);
            let minimum = cap(&h, "TP-074", "minimum", PRESET_T, profile);
            let _ = h.resize(160, 50);
            let recovered = cap(&h, "TP-074", "recovered", PRESET_T, profile);
            let _ = h.resize(80, 24);
            let drawer = cap(&h, "TP-074", "drawer", PRESET_T, profile);
            assert_eq!(initial.width, width);
            assert!(!too_narrow.text.is_empty());
            assert!(!noop.text.is_empty());
            assert!(!too_short.text.is_empty());
            assert!(!minimum.text.is_empty());
            assert!(!recovered.text.is_empty());
            assert!(!drawer.text.is_empty());
        }
    }
    let mut flash = launch_preset_t(120, 40, ColorProfile::Truecolor);
    click_text(&mut flash, "order_number");
    let t0 = cap(&flash, "TP-076", "t0", PRESET_T, ColorProfile::Truecolor);
    let _ = flash.advance(Duration::from_millis(139));
    let flash139 = cap(
        &flash,
        "TP-076",
        "flash139",
        PRESET_T,
        ColorProfile::Truecolor,
    );
    let _ = flash.advance(Duration::from_millis(1));
    ticks(&mut flash, 1);
    let flash140 = cap(
        &flash,
        "TP-076",
        "flash140",
        PRESET_T,
        ColorProfile::Truecolor,
    );
    let _ = flash.advance(Duration::from_millis(4859));
    let status4999 = cap(
        &flash,
        "TP-076",
        "status4999",
        PRESET_T,
        ColorProfile::Truecolor,
    );
    let _ = flash.advance(Duration::from_millis(2));
    ticks(&mut flash, 1);
    let expired = cap(
        &flash,
        "TP-076",
        "status_expired",
        PRESET_T,
        ColorProfile::Truecolor,
    );
    for obs in [&t0, &flash139, &flash140, &status4999, &expired] {
        assert_eq!(obs.screen, Screen::Workbench);
    }
    for profile in COLOR_PROFILES {
        let mut life = launch_preset_c(120, 40, profile);
        type_text(&mut life, "q");
        let _ = cap(&life, "TP-078", "assert_exit", PRESET_C, profile);
        let mut life2 = launch_preset_c(120, 40, profile);
        ctrl(&mut life2, 'c');
        let _ = cap(&life2, "TP-078", "assert_exit", PRESET_C, profile);
        let mut life3 = launch_preset_w(120, 40, profile);
        let _ = life3.resize(60, 15);
        type_text(&mut life3, "q");
        let _ = cap(&life3, "TP-078", "assert_exit", PRESET_W, profile);
    }
    each_matrix(|width, height, profile| {
        for object in ["customers", "orders", "events"] {
            let mut h = launch_preset_w(width, height, profile);
            ctrl(&mut h, 'o');
            type_text(&mut h, object);
            enter(&mut h);
            let open = cap(&h, "TP-079", "open", PRESET_W, profile);
            home(&mut h);
            end(&mut h);
            let _ = h.key_mod(junie_tui::KeyCode::End, junie_tui::KeyModifiers::CONTROL);
            let edge = cap(&h, "TP-079", "edge", PRESET_W, profile);
            enter(&mut h);
            let edit = cap(&h, "TP-079", "edit_or_readonly", PRESET_W, profile);
            esc(&mut h);
            for obs in [&open, &edge, &edit] {
                assert_eq!(obs.screen, Screen::Workbench);
            }
        }
    });
}
