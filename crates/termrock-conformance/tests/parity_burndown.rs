//! PARITY burndown inventory for `control_states.rs` (Q03 / S6 reconcile).
//!
//! Each `#[ignore = "PARITY …"]` in `control_states.rs` maps to exactly one
//! `BD-xx` row below. A row is executable: the gate tests fail if the ignored
//! test disappears without its row burning, if a new ignore appears without a
//! row, if the row's owner drifts from `component-ownership.json`, or if a
//! deferred case id is not a real vendored registry id.
//!
//! Burn protocol: fix the owner crate, un-`ignore` the test, delete its row.
//! No W coverage may be claimed until this table is empty.

use std::collections::HashSet;

/// One deferred parity record: (`BD-xx`, deferred registry case ids,
/// ignored test fn, owning component, owner crate, reason).
struct Burndown {
    id: &'static str,
    cases: &'static [&'static str],
    test: &'static str,
    component: &'static str,
    owner: &'static str,
    reason: &'static str,
}

const BURNDOWN: &[Burndown] = &[
    Burndown {
        id: "BD-01",
        cases: &["W01-02"],
        test: "w01_brand_press_paints_distinct_from_hover",
        component: "W01",
        owner: "termrock-controls",
        reason: "pressed lockup repaints hover; BRAND recipe has no PRESSED rule",
    },
    Burndown {
        id: "BD-02",
        cases: &["W02-01"],
        test: "w02_button_disabled_suppresses_focus_gutter",
        component: "W02",
        owner: "termrock-controls",
        reason: "disabled+focused paints gutter; GUTTER recipe keys on FOCUSED alone",
    },
    Burndown {
        id: "BD-03",
        cases: &["W03-01"],
        test: "w03_checkbox_compact_markers",
        component: "W03",
        owner: "termrock-controls",
        reason: "narrow checkbox clips instead of compact marks; no compact branch",
    },
    Burndown {
        id: "BD-04",
        cases: &["W03-01"],
        test: "w03_checkbox_checked_focus_hover_independent",
        component: "W03",
        owner: "termrock-controls",
        reason: "checked+focus swallows the hover lift",
    },
    Burndown {
        id: "BD-05",
        cases: &["W04-01"],
        test: "w04_toggle_compact_markers",
        component: "W04",
        owner: "termrock-controls",
        reason: "narrow toggle clips instead of compact dots; no compact branch",
    },
    Burndown {
        id: "BD-06",
        cases: &["W05-02", "W05-04"],
        test: "w05_radio_per_option_disabled",
        component: "W05",
        owner: "termrock-controls",
        reason: "RadioGroup has no per-option disabled (whole-group only)",
    },
    Burndown {
        id: "BD-07",
        cases: &["W05-04"],
        test: "w05_radio_no_horizontal",
        component: "W05",
        owner: "termrock-controls",
        reason: "RadioGroup has no horizontal orientation (vertical only)",
    },
    Burndown {
        id: "BD-08",
        cases: &["W06-02", "W06-05"],
        test: "w06_chipbar_closable_keyboard_dead",
        component: "W06",
        owner: "termrock-navigation",
        reason: "closable ChipBar ignores all keys; triplicate Remove rejects binding table",
    },
    Burndown {
        id: "BD-09",
        cases: &["W06-02"],
        test: "w06_chipbar_close_press_then_reorder_or_delete",
        component: "W06",
        owner: "termrock-navigation",
        reason: "pressing × toggles; LABEL part covers CLOSE in hit-test",
    },
    Burndown {
        id: "BD-10",
        cases: &["W08-02"],
        test: "w08_input_masked_clicks_follow_display",
        component: "W08",
        owner: "termrock-fields",
        reason: "masked clicks use plaintext widths, not display graphemes",
    },
    Burndown {
        id: "BD-11",
        cases: &["W08-02"],
        test: "w08_input_down_only_does_not_edit",
        component: "W08",
        owner: "termrock-fields",
        reason: "mouse Down alone begins the draft; reference demands completed click",
    },
    Burndown {
        id: "BD-12",
        cases: &["W08-03"],
        test: "w08_input_navigation_paste_begins_editing",
        component: "W08",
        owner: "termrock-fields",
        reason: "paste while navigating is dropped; paste arm requires is_editing",
    },
    Burndown {
        id: "BD-13",
        cases: &["W08-07"],
        test: "w08_input_external_change_is_not_silently_overwritten",
        component: "W08",
        owner: "termrock-fields",
        reason: "commit overwrites external value changes; no revision check",
    },
    Burndown {
        id: "BD-14",
        cases: &["W09-01"],
        test: "w09_area_nav_enter_begins_without_newline",
        component: "W09",
        owner: "termrock-fields",
        reason: "nav Enter begins AND inserts a newline; reference begins only",
    },
    Burndown {
        id: "BD-15",
        cases: &["W09-03"],
        test: "w09_area_nav_page_home_end_scroll_without_editing",
        component: "W09",
        owner: "termrock-fields",
        reason: "nav PageUp/Home/End begin editing; reference scrolls the view",
    },
    Burndown {
        id: "BD-16",
        cases: &["W09-05"],
        test: "w09_area_wheel_holds_past_caret_until_it_moves",
        component: "W09",
        owner: "termrock-fields",
        reason: "wheel-while-editing snaps back; ensure_visible runs every update",
    },
    Burndown {
        id: "BD-17",
        cases: &["W10-05"],
        test: "w10_select_popup_edge_fade",
        component: "W10",
        owner: "termrock-overlays",
        reason: "select popup never fades scrolled edges; no scroll_edges call",
    },
    Burndown {
        id: "BD-18",
        cases: &["W11-04"],
        test: "w11_form_busy_enter_submit_blocked",
        component: "W11",
        owner: "termrock-forms",
        reason: "Enter submits while submit action is disabled; enter path ignores eligibility",
    },
    Burndown {
        id: "BD-20",
        cases: &["W13-02"],
        test: "w13_filter_backspace_removes_grapheme",
        component: "W13",
        owner: "termrock-navigation",
        reason: "query backspace pops one char and splits grapheme clusters",
    },
    Burndown {
        id: "BD-21",
        cases: &["W13-05"],
        test: "w13_filter_wide_trail_cells_clear",
        component: "W13",
        owner: "termrock-navigation",
        reason: "wide-char trail cells never cleared; stale glyphs leak",
    },
    Burndown {
        id: "BD-22",
        cases: &["W13-04"],
        test: "w13_filter_reseed_selects_first_eligible",
        component: "W13",
        owner: "termrock-navigation",
        reason: "reseed picks nearest-from-index; reference picks first eligible",
    },
    Burndown {
        id: "BD-23",
        cases: &["W16-02"],
        test: "w16_steps_running_spinner_cycles_phases",
        component: "W16",
        owner: "termrock-navigation",
        reason: "running step shows static Bullet; reference cycles a 10-phase spinner",
    },
    Burndown {
        id: "BD-24",
        cases: &["W17-01"],
        test: "w17_tabs_focused_pressed_bold",
        component: "W17",
        owner: "termrock-navigation",
        reason: "focused/pressed tabs render inactive; reference bolds the cursor tab",
    },
];

const OWNERSHIP_JSON: &str = include_str!("../../../component-ownership.json");
const REGISTRY_JSON: &str = include_str!("../../../tests/conformance/required_cases.json");
const MIRROR_SRC: &str = include_str!("control_states.rs");

/// Ownership gate: every row's component resolves to its row owner.
#[test]
fn burndown_owners_match_ownership_map() {
    let doc: serde_json::Value =
        serde_json::from_str(OWNERSHIP_JSON).expect("ownership map must parse");
    for row in BURNDOWN {
        let found = doc["components"]
            .as_array()
            .expect("components array")
            .iter()
            .find(|c| c["id"].as_str() == Some(row.component));
        let entry = found
            .unwrap_or_else(|| panic!("{}: {} missing from ownership map", row.id, row.component));
        assert_eq!(
            entry["owner"].as_str(),
            Some(row.owner),
            "{}: owner drift for {}",
            row.id,
            row.component
        );
    }
}

/// Vocabulary gate: every deferred case id is a real vendored registry id.
#[test]
fn burndown_cases_exist_in_registry() {
    let doc: serde_json::Value = serde_json::from_str(REGISTRY_JSON).expect("registry must parse");
    let ids: HashSet<&str> = doc["cases"]
        .as_array()
        .expect("cases array")
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();
    assert_eq!(
        ids.iter().filter(|id| id.starts_with('W')).count(),
        222,
        "registry must carry the full 222-case W vocabulary"
    );
    for row in BURNDOWN {
        for case in row.cases {
            assert!(
                ids.contains(case),
                "{}: {case} is not a registry case id",
                row.id
            );
        }
    }
}

/// Drift gate: the inventory matches the actual `#[ignore]`s one-to-one.
#[test]
fn burndown_matches_ignored_tests() {
    let ignore_lines: Vec<&str> = MIRROR_SRC
        .lines()
        .filter(|l| l.trim_start().starts_with("#[ignore = \"PARITY"))
        .collect();
    assert_eq!(
        ignore_lines.len(),
        BURNDOWN.len(),
        "ignore count drifted: {} PARITY ignores vs {} burndown rows",
        ignore_lines.len(),
        BURNDOWN.len()
    );
    let deferred_refs: usize = BURNDOWN.iter().map(|row| row.cases.len()).sum();
    assert_eq!(deferred_refs, 25, "deferred case references must total 25");
    for row in BURNDOWN {
        assert!(
            !row.reason.is_empty(),
            "{}: burndown row needs a reason",
            row.id
        );
        let head = format!("PARITY {}", row.cases.join("/"));
        assert!(
            ignore_lines.iter().any(|l| l.contains(&head)),
            "{} ({}): no PARITY ignore headed `{head}`",
            row.id,
            row.reason
        );
        assert!(
            MIRROR_SRC.contains(&format!("fn {}(", row.test)),
            "{}: ignored test fn {} missing",
            row.id,
            row.test
        );
    }
}
