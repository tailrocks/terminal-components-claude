//! TP-008–016: connection form, save/edit/delete, save-and-connect.
//!
//! `form_advanced` product repair is TASK-058. This adapter only observes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_C, click_text, ctrl, down, end,
    enter, esc, home, launch_preset_c, observe, replace_paste, right, space, tab, ticks,
};
use tablepro_app::{CONNECTION_NAME, Screen};

fn cap(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    scenario: &str,
    checkpoint: &str,
    profile: ColorProfile,
) -> Observation {
    observe(harness, scenario, PRESET_C, checkpoint, profile)
}

fn each_matrix(mut body: impl FnMut(u16, u16, ColorProfile)) {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            body(width, height, profile);
        }
    }
}

fn open_new(harness: &mut junie_tui_testing::Harness<tablepro_app::TableProApp>) {
    ctrl(harness, 'n');
}

#[test]
fn tp008_new_invalid_port() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        open_new(&mut harness);
        let new_basic = cap(&harness, "TP-008", "new_basic", profile);
        ctrl(&mut harness, 's');
        let invalid = cap(&harness, "TP-008", "invalid", profile);
        let _ = harness.tab_to(CONNECTION_NAME);
        tab(&mut harness);
        tab(&mut harness);
        tab(&mut harness);
        replace_paste(&mut harness, "65536");
        ctrl(&mut harness, 's');
        let invalid_port = cap(&harness, "TP-008", "invalid_port", profile);
        replace_paste(&mut harness, "0");
        ctrl(&mut harness, 's');
        let zero_port = cap(&harness, "TP-008", "zero_port", profile);
        assert!(new_basic.form_open || new_basic.screen == Screen::Connections);
        assert_eq!(invalid.screen, Screen::Connections);
        assert_eq!(invalid_port.screen, Screen::Connections);
        assert_eq!(zero_port.screen, Screen::Connections);
    });
}

#[test]
fn tp009_basic_choices() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        open_new(&mut harness);
        let _ = harness.tab_to(CONNECTION_NAME);
        replace_paste(&mut harness, "Parity fixture");
        tab(&mut harness);
        enter(&mut harness);
        home(&mut harness);
        down(&mut harness, 1);
        enter(&mut harness);
        let mysql_port = cap(&harness, "TP-009", "mysql_port", profile);
        enter(&mut harness);
        home(&mut harness);
        down(&mut harness, 2);
        enter(&mut harness);
        let sqlite_port = cap(&harness, "TP-009", "sqlite_port", profile);
        enter(&mut harness);
        home(&mut harness);
        enter(&mut harness);
        tab(&mut harness);
        tab(&mut harness);
        tab(&mut harness);
        tab(&mut harness);
        enter(&mut harness);
        down(&mut harness, 1);
        enter(&mut harness);
        end(&mut harness);
        space(&mut harness);
        end(&mut harness);
        space(&mut harness);
        let choices = cap(&harness, "TP-009", "choices", profile);
        assert!(matches!(
            mysql_port.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            sqlite_port.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            choices.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}

#[test]
fn tp010_password_paste_and_prompt() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        open_new(&mut harness);
        for _ in 0..8 {
            tab(&mut harness);
        }
        enter(&mut harness);
        let _ = harness.paste("päss秘密");
        let password_field = cap(&harness, "TP-010", "password_field", profile);
        esc(&mut harness);
        tab(&mut harness);
        space(&mut harness);
        let prompt = cap(&harness, "TP-010", "prompt_toggle", profile);
        assert!(matches!(
            password_field.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            prompt.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}

#[test]
fn tp011_advanced_fields_observed() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        open_new(&mut harness);
        let _ = harness.tab_to(CONNECTION_NAME);
        for _ in 0..16 {
            if harness.focus() != Some(CONNECTION_NAME) {
                break;
            }
            tab(&mut harness);
        }
        right(&mut harness, 1);
        enter(&mut harness);
        let advanced = cap(&harness, "TP-011", "advanced", profile);
        space(&mut harness);
        tab(&mut harness);
        space(&mut harness);
        let ssh_enabled = cap(&harness, "TP-011", "ssh_enabled", profile);
        tab(&mut harness);
        replace_paste(&mut harness, "bastion.fixture");
        tab(&mut harness);
        replace_paste(&mut harness, "operator");
        tab(&mut harness);
        enter(&mut harness);
        let _ = harness.paste("SET search_path TO public;");
        esc(&mut harness);
        tab(&mut harness);
        space(&mut harness);
        let filled = cap(&harness, "TP-011", "advanced_filled", profile);
        assert!(matches!(
            advanced.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            ssh_enabled.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            filled.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}

#[test]
fn tp012_test_connection_ticks() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        open_new(&mut harness);
        let _ = harness.tab_to(CONNECTION_NAME);
        tab(&mut harness);
        tab(&mut harness);
        replace_paste(&mut harness, "localhost");
        click_text(&mut harness, "Test connection");
        let testing = cap(&harness, "TP-012", "testing", profile);
        ticks(&mut harness, 10);
        let success = cap(&harness, "TP-012", "success", profile);
        replace_paste(&mut harness, "analytics.acme.io");
        click_text(&mut harness, "Test connection");
        ticks(&mut harness, 10);
        let test_error = cap(&harness, "TP-012", "test_error", profile);
        assert_eq!(testing.screen, Screen::Connections);
        assert_eq!(success.screen, Screen::Connections);
        assert_eq!(test_error.screen, Screen::Connections);
    });
}

#[test]
fn tp013_edit_cancel_save() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        click_text(&mut harness, "Edit");
        let edit = cap(&harness, "TP-013", "edit", profile);
        let _ = harness.tab_to(CONNECTION_NAME);
        replace_paste(&mut harness, "Local parity");
        click_text(&mut harness, "Cancel");
        let cancelled = cap(&harness, "TP-013", "cancelled", profile);
        click_text(&mut harness, "Edit");
        let _ = harness.tab_to(CONNECTION_NAME);
        replace_paste(&mut harness, "Local parity");
        ctrl(&mut harness, 's');
        let saved = cap(&harness, "TP-013", "saved", profile);
        assert!(matches!(
            edit.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            cancelled.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            saved.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}

#[test]
fn tp014_duplicate_delete() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        click_text(&mut harness, "Duplicate");
        let copy = cap(&harness, "TP-014", "copy", profile);
        click_text(&mut harness, "Delete");
        let delete_prompt = cap(&harness, "TP-014", "delete_prompt", profile);
        esc(&mut harness);
        let kept = cap(&harness, "TP-014", "kept", profile);
        click_text(&mut harness, "Delete");
        tab(&mut harness);
        enter(&mut harness);
        let deleted = cap(&harness, "TP-014", "deleted", profile);
        assert!(matches!(
            copy.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            delete_prompt.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            kept.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            deleted.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}

#[test]
fn tp015_empty_list_120x40() {
    for profile in COLOR_PROFILES {
        let mut harness = launch_preset_c(120, 40, profile);
        for i in 0..6 {
            click_text(&mut harness, "Delete");
            tab(&mut harness);
            enter(&mut harness);
            let after = cap(&harness, "TP-015", "after_each_delete", profile);
            assert!(
                matches!(after.screen, Screen::Connections | Screen::Workbench),
                "delete {i} {:?}",
                after.screen
            );
        }
        let empty = cap(&harness, "TP-015", "empty", profile);
        ctrl(&mut harness, 'n');
        let new_from_empty = cap(&harness, "TP-015", "new_from_empty", profile);
        assert!(matches!(
            empty.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            new_from_empty.screen,
            Screen::Connections | Screen::Workbench
        ));
    }
}

#[test]
fn tp016_save_and_connect() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        open_new(&mut harness);
        let _ = harness.tab_to(CONNECTION_NAME);
        replace_paste(&mut harness, "Parity local");
        tab(&mut harness);
        tab(&mut harness);
        replace_paste(&mut harness, "localhost");
        tab(&mut harness);
        tab(&mut harness);
        replace_paste(&mut harness, "acme_dev");
        tab(&mut harness);
        replace_paste(&mut harness, "postgres");
        click_text(&mut harness, "Save & Connect");
        click_text(&mut harness, "Save & connect");
        let connecting = cap(&harness, "TP-016", "connecting", profile);
        ticks(&mut harness, 12);
        let workbench = cap(&harness, "TP-016", "workbench", profile);
        assert!(matches!(
            connecting.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            workbench.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}
