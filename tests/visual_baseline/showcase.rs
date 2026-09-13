//! showcase captures (151): 18 non-audit pages under `showcase/pages/`
//! (progress statics restored via `--motion paused`, §5), keyboard/mouse
//! flows under `showcase/flows/` (18 remapped + 10 from the §3.7 table + 5
//! post-flag + 10 audit hole closures + 32 from the four 8-combo audit-flow
//! variant matrices below). The 5 audit pages
//! (buttons/diff/forms/inputs/textareas) keep statics only in audit.rs
//! (`showcase/audit/`, 5×5 matrix — the dedupe rule).
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs); only the store names were regrouped
//! (`showcase_<leaf>` → `showcase/<sub_group>/<leaf>`). Do not hand-tune:
//! drift against the approved frames means the port or the app changed.

use crate::support::{self, Case, Color, SHOWCASE};

const BOOT: &str = "Junie Design system";

// ------------------------------------------------------------------ pages --

crate::baseline_case!(showcase_pages_overview_default_80x24_truecolor => Case::new("showcase/pages/overview_default_80x24_truecolor", SHOWCASE, &["--page", "overview"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_overview_default_80x24_none => Case::new("showcase/pages/overview_default_80x24_none", SHOWCASE, &["--page", "overview"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_overview_default_120x40_truecolor => Case::new("showcase/pages/overview_default_120x40_truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_overview_default_120x40_none => Case::new("showcase/pages/overview_default_120x40_none", SHOWCASE, &["--page", "overview"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_lists_default_80x24_truecolor => Case::new("showcase/pages/lists_default_80x24_truecolor", SHOWCASE, &["--page", "lists"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_lists_default_80x24_none => Case::new("showcase/pages/lists_default_80x24_none", SHOWCASE, &["--page", "lists"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_lists_default_120x40_truecolor => Case::new("showcase/pages/lists_default_120x40_truecolor", SHOWCASE, &["--page", "lists"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_lists_default_120x40_none => Case::new("showcase/pages/lists_default_120x40_none", SHOWCASE, &["--page", "lists"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_trees_default_80x24_truecolor => Case::new("showcase/pages/trees_default_80x24_truecolor", SHOWCASE, &["--page", "trees"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_trees_default_80x24_none => Case::new("showcase/pages/trees_default_80x24_none", SHOWCASE, &["--page", "trees"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_trees_default_120x40_truecolor => Case::new("showcase/pages/trees_default_120x40_truecolor", SHOWCASE, &["--page", "trees"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_trees_default_120x40_none => Case::new("showcase/pages/trees_default_120x40_none", SHOWCASE, &["--page", "trees"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_tables_default_80x24_truecolor => Case::new("showcase/pages/tables_default_80x24_truecolor", SHOWCASE, &["--page", "tables"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_tables_default_80x24_none => Case::new("showcase/pages/tables_default_80x24_none", SHOWCASE, &["--page", "tables"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_tables_default_120x40_truecolor => Case::new("showcase/pages/tables_default_120x40_truecolor", SHOWCASE, &["--page", "tables"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_tables_default_120x40_none => Case::new("showcase/pages/tables_default_120x40_none", SHOWCASE, &["--page", "tables"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_editable_default_80x24_truecolor => Case::new("showcase/pages/editable_default_80x24_truecolor", SHOWCASE, &["--page", "editabletables"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_editable_default_80x24_none => Case::new("showcase/pages/editable_default_80x24_none", SHOWCASE, &["--page", "editabletables"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_editable_default_120x40_truecolor => Case::new("showcase/pages/editable_default_120x40_truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_editable_default_120x40_none => Case::new("showcase/pages/editable_default_120x40_none", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_panels_default_80x24_truecolor => Case::new("showcase/pages/panels_default_80x24_truecolor", SHOWCASE, &["--page", "panels"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_panels_default_80x24_none => Case::new("showcase/pages/panels_default_80x24_none", SHOWCASE, &["--page", "panels"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_panels_default_120x40_truecolor => Case::new("showcase/pages/panels_default_120x40_truecolor", SHOWCASE, &["--page", "panels"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_panels_default_120x40_none => Case::new("showcase/pages/panels_default_120x40_none", SHOWCASE, &["--page", "panels"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_sidebars_default_80x24_truecolor => Case::new("showcase/pages/sidebars_default_80x24_truecolor", SHOWCASE, &["--page", "sidebars"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_sidebars_default_80x24_none => Case::new("showcase/pages/sidebars_default_80x24_none", SHOWCASE, &["--page", "sidebars"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_sidebars_default_120x40_truecolor => Case::new("showcase/pages/sidebars_default_120x40_truecolor", SHOWCASE, &["--page", "sidebars"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_sidebars_default_120x40_none => Case::new("showcase/pages/sidebars_default_120x40_none", SHOWCASE, &["--page", "sidebars"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_dialogs_default_80x24_truecolor => Case::new("showcase/pages/dialogs_default_80x24_truecolor", SHOWCASE, &["--page", "dialogs"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_dialogs_default_80x24_none => Case::new("showcase/pages/dialogs_default_80x24_none", SHOWCASE, &["--page", "dialogs"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_dialogs_default_120x40_truecolor => Case::new("showcase/pages/dialogs_default_120x40_truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_dialogs_default_120x40_none => Case::new("showcase/pages/dialogs_default_120x40_none", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_codeeditor_default_80x24_truecolor => Case::new("showcase/pages/codeeditor_default_80x24_truecolor", SHOWCASE, &["--page", "codeeditor"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_codeeditor_default_80x24_none => Case::new("showcase/pages/codeeditor_default_80x24_none", SHOWCASE, &["--page", "codeeditor"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_codeeditor_default_120x40_truecolor => Case::new("showcase/pages/codeeditor_default_120x40_truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_codeeditor_default_120x40_none => Case::new("showcase/pages/codeeditor_default_120x40_none", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_datagrid_default_80x24_truecolor => Case::new("showcase/pages/datagrid_default_80x24_truecolor", SHOWCASE, &["--page", "datagrid"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_datagrid_default_80x24_none => Case::new("showcase/pages/datagrid_default_80x24_none", SHOWCASE, &["--page", "datagrid"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_datagrid_default_120x40_truecolor => Case::new("showcase/pages/datagrid_default_120x40_truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_datagrid_default_120x40_none => Case::new("showcase/pages/datagrid_default_120x40_none", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_chips_default_80x24_truecolor => Case::new("showcase/pages/chips_default_80x24_truecolor", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_chips_default_80x24_none => Case::new("showcase/pages/chips_default_80x24_none", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_chips_default_120x40_truecolor => Case::new("showcase/pages/chips_default_120x40_truecolor", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_chips_default_120x40_none => Case::new("showcase/pages/chips_default_120x40_none", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_pickers_default_80x24_truecolor => Case::new("showcase/pages/pickers_default_80x24_truecolor", SHOWCASE, &["--page", "pickers"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_pickers_default_80x24_none => Case::new("showcase/pages/pickers_default_80x24_none", SHOWCASE, &["--page", "pickers"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_pickers_default_120x40_truecolor => Case::new("showcase/pages/pickers_default_120x40_truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_pickers_default_120x40_none => Case::new("showcase/pages/pickers_default_120x40_none", SHOWCASE, &["--page", "pickers"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_chrome_default_80x24_truecolor => Case::new("showcase/pages/chrome_default_80x24_truecolor", SHOWCASE, &["--page", "chrome"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_chrome_default_80x24_none => Case::new("showcase/pages/chrome_default_80x24_none", SHOWCASE, &["--page", "chrome"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_chrome_default_120x40_truecolor => Case::new("showcase/pages/chrome_default_120x40_truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_chrome_default_120x40_none => Case::new("showcase/pages/chrome_default_120x40_none", SHOWCASE, &["--page", "chrome"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_settings_default_80x24_truecolor => Case::new("showcase/pages/settings_default_80x24_truecolor", SHOWCASE, &["--page", "settings"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_settings_default_80x24_none => Case::new("showcase/pages/settings_default_80x24_none", SHOWCASE, &["--page", "settings"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_settings_default_120x40_truecolor => Case::new("showcase/pages/settings_default_120x40_truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_settings_default_120x40_none => Case::new("showcase/pages/settings_default_120x40_none", SHOWCASE, &["--page", "settings"], 120, 40, Color::None, BOOT));
crate::baseline_case!(showcase_pages_taskrunner_default_80x24_truecolor => Case::new("showcase/pages/taskrunner_default_80x24_truecolor", SHOWCASE, &["--page", "taskrunner"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_taskrunner_default_80x24_none => Case::new("showcase/pages/taskrunner_default_80x24_none", SHOWCASE, &["--page", "taskrunner"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_taskrunner_default_120x40_truecolor => Case::new("showcase/pages/taskrunner_default_120x40_truecolor", SHOWCASE, &["--page", "taskrunner"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_taskrunner_default_120x40_none => Case::new("showcase/pages/taskrunner_default_120x40_none", SHOWCASE, &["--page", "taskrunner"], 120, 40, Color::None, BOOT));
// `wait:739.63s` is the boot stream's end-state marker (the 2000th log
// line's timestamp). Without it wait_idle can fire mid-stream and the gated
// frame depends on flush timing.
crate::baseline_case!(showcase_pages_scrolling_default_80x24_truecolor => Case::new("showcase/pages/scrolling_default_80x24_truecolor", SHOWCASE, &["--page", "scrolling"], 80, 24, Color::Truecolor, BOOT).sends(&["wait:739.63s"]).timeout(180000));
crate::baseline_case!(showcase_pages_scrolling_default_80x24_none => Case::new("showcase/pages/scrolling_default_80x24_none", SHOWCASE, &["--page", "scrolling"], 80, 24, Color::None, BOOT).sends(&["wait:739.63s"]).timeout(180000));
crate::baseline_case!(showcase_pages_scrolling_default_120x40_truecolor => Case::new("showcase/pages/scrolling_default_120x40_truecolor", SHOWCASE, &["--page", "scrolling"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:739.63s"]).timeout(180000));
crate::baseline_case!(showcase_pages_scrolling_default_120x40_none => Case::new("showcase/pages/scrolling_default_120x40_none", SHOWCASE, &["--page", "scrolling"], 120, 40, Color::None, BOOT).sends(&["wait:739.63s"]).timeout(180000));
// `wait:7 of 7` is the step-rail end-state; 30s is tight under load.
crate::baseline_case!(showcase_pages_terminal_default_80x24_truecolor => Case::new("showcase/pages/terminal_default_80x24_truecolor", SHOWCASE, &["--page", "terminal"], 80, 24, Color::Truecolor, BOOT).sends(&["wait:7 of 7"]).timeout(60000));
crate::baseline_case!(showcase_pages_terminal_default_80x24_none => Case::new("showcase/pages/terminal_default_80x24_none", SHOWCASE, &["--page", "terminal"], 80, 24, Color::None, BOOT).sends(&["wait:7 of 7"]).timeout(60000));
crate::baseline_case!(showcase_pages_terminal_default_120x40_truecolor => Case::new("showcase/pages/terminal_default_120x40_truecolor", SHOWCASE, &["--page", "terminal"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:7 of 7"]).timeout(60000));
crate::baseline_case!(showcase_pages_terminal_default_120x40_none => Case::new("showcase/pages/terminal_default_120x40_none", SHOWCASE, &["--page", "terminal"], 120, 40, Color::None, BOOT).sends(&["wait:7 of 7"]).timeout(60000));
crate::baseline_case!(showcase_pages_datagrid_default_120x40_256 => Case::new("showcase/pages/datagrid_default_120x40_256", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Ansi256, BOOT));
crate::baseline_case!(showcase_pages_datagrid_default_120x40_16 => Case::new("showcase/pages/datagrid_default_120x40_16", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Ansi16, BOOT));
crate::baseline_case!(showcase_pages_overview_default_120x40_nocolor => Case::new("showcase/pages/overview_default_120x40_nocolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::NoColorEnv, BOOT));
crate::baseline_case!(showcase_pages_overview_default_72x20_truecolor => Case::new("showcase/pages/overview_default_72x20_truecolor", SHOWCASE, &["--page", "overview"], 72, 20, Color::Truecolor, BOOT));

// ------------------------------------------------------------------ flows --

crate::baseline_case!(showcase_flows_inputs_editing_120x40_truecolor => Case::new("showcase/flows/inputs_editing_120x40_truecolor", SHOWCASE, &["--page", "inputs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:EDIT"]));
crate::baseline_case!(showcase_flows_inputs_selected_120x40_truecolor => Case::new("showcase/flows/inputs_selected_120x40_truecolor", SHOWCASE, &["--page", "inputs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "ctrl-l", "wait:EDIT"]));
crate::baseline_case!(showcase_flows_forms_invalid_120x40_truecolor => Case::new("showcase/flows/forms_invalid_120x40_truecolor", SHOWCASE, &["--page", "forms"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "ctrl-s", "wait:Required"]));
crate::baseline_case!(showcase_flows_diff_review_120x40_truecolor => Case::new("showcase/flows/diff_review_120x40_truecolor", SHOWCASE, &["--page", "diff"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:● Review"]));
crate::baseline_case!(showcase_flows_diff_empty_120x40_truecolor => Case::new("showcase/flows/diff_empty_120x40_truecolor", SHOWCASE, &["--page", "diff"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "tab", "enter", "wait:No file selected"]));
crate::baseline_case!(showcase_flows_buttons_focus_120x40_truecolor => Case::new("showcase/flows/buttons_focus_120x40_truecolor", SHOWCASE, &["--page", "buttons"], 120, 40, Color::Truecolor, BOOT).sends(&["tab"]));
crate::baseline_case!(showcase_flows_lists_moved_120x40_truecolor => Case::new("showcase/flows/lists_moved_120x40_truecolor", SHOWCASE, &["--page", "lists"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "down", "down"]));
crate::baseline_case!(showcase_flows_trees_expanded_120x40_truecolor => Case::new("showcase/flows/trees_expanded_120x40_truecolor", SHOWCASE, &["--page", "trees"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right"]));
crate::baseline_case!(showcase_flows_tables_selected_120x40_truecolor => Case::new("showcase/flows/tables_selected_120x40_truecolor", SHOWCASE, &["--page", "tables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "down", "down"]));
crate::baseline_case!(showcase_flows_editable_editing_120x40_truecolor => Case::new("showcase/flows/editable_editing_120x40_truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter"]));
crate::baseline_case!(showcase_flows_datagrid_selected_120x40_truecolor => Case::new("showcase/flows/datagrid_selected_120x40_truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "down", "right"]));
crate::baseline_case!(showcase_flows_dialogs_open_120x40_truecolor => Case::new("showcase/flows/dialogs_open_120x40_truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter"]));
crate::baseline_case!(showcase_flows_pickers_open_120x40_truecolor => Case::new("showcase/flows/pickers_open_120x40_truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter"]));
crate::baseline_case!(showcase_flows_chips_toggled_120x40_truecolor => Case::new("showcase/flows/chips_toggled_120x40_truecolor", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "space"]));
// `wait:739.63s` is the boot stream's end-state marker (the 2000th log
// line's timestamp, unique on screen): without it the sends can begin during
// a >200 ms stall mid-stream (wait_idle fires early under load) and race the
// stream tail, leaving the frame's final cursor position — embedded in the
// gated HTML — dependent on flush timing.
crate::baseline_case!(showcase_flows_scrolling_scrolled_120x40_truecolor => Case::new("showcase/flows/scrolling_scrolled_120x40_truecolor", SHOWCASE, &["--page", "scrolling"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:739.63s", "tab", "down", "down", "down"]).timeout(180000));
crate::baseline_case!(showcase_flows_settings_toggled_120x40_truecolor => Case::new("showcase/flows/settings_toggled_120x40_truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "space"]));
crate::baseline_case!(showcase_flows_help_overlay_120x40_truecolor => Case::new("showcase/flows/help_overlay_120x40_truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["?"]));
crate::baseline_case!(showcase_flows_inspector_open_120x40_truecolor => Case::new("showcase/flows/inspector_open_120x40_truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["i"]));

// ------------------------------------------------------- new flows (§3.7) --

// S1: the prompt dialog (second button), input prefilled and validated.
crate::baseline_case!(showcase_flows_dialogs_prompt_120x40_truecolor => Case::new("showcase/flows/dialogs_prompt_120x40_truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:Shown in the task list"]));
// S2: the destructive dialog (fourth button), Cancel focused first.
crate::baseline_case!(showcase_flows_dialogs_destructive_120x40_truecolor => Case::new("showcase/flows/dialogs_destructive_120x40_truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "tab", "enter", "wait:This cannot be undone"]));
// S3: required field filled, Ctrl+S, then wait out the 1.8 s wall-clock
// Busy window — the Done message is the stable end state (the bash
// `sleep:1200` predates the 1800 ms busy constant; `wait:` is exact).
crate::baseline_case!(showcase_flows_forms_valid_120x40_truecolor => Case::new("showcase/flows/forms_valid_120x40_truecolor", SHOWCASE, &["--page", "forms"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "type:Fix the login redirect loop", "enter", "ctrl-s", "wait:Task created ✓"]));
// S4: `]` cycles to the next page; the header crumb is the unique proof.
crate::baseline_case!(showcase_flows_nav_cycled_120x40_truecolor => Case::new("showcase/flows/nav_cycled_120x40_truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["]", "wait:/ Components / Buttons"]));
// S5: nav owns the keyboard; `G` lands the cursor on the last page (the
// state proof is the focused gutter row in the frame — no text changes).
crate::baseline_case!(showcase_flows_nav_end_120x40_truecolor => Case::new("showcase/flows/nav_end_120x40_truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["0", "G"]));
// S6: the Level picker (third button) is not searchable; one down from the
// current level, Enter sets it — the footer status confirms the choice.
crate::baseline_case!(showcase_flows_pickers_level_120x40_truecolor => Case::new("showcase/flows/pickers_level_120x40_truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "enter", "down", "enter", "wait:Chose Safe Mode (Full)"]));
// S7: two typed characters open the completion popup (i = edit mode).
crate::baseline_case!(showcase_flows_editor_completion_120x40_truecolor => Case::new("showcase/flows/editor_completion_120x40_truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "i", "type:cl", "wait: items"]));
// S8: the menu bar is the page's first stop; Enter opens the File menu.
crate::baseline_case!(showcase_flows_chrome_menu_120x40_truecolor => Case::new("showcase/flows/chrome_menu_120x40_truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:New tab"]));
// S10: cell edit committed; the panel meta counts edits.
crate::baseline_case!(showcase_flows_editable_committed_120x40_truecolor => Case::new("showcase/flows/editable_committed_120x40_truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "type:x", "enter", "wait:1 edits"]));
// S11: fifth stop is the Auto-merge toggle; space twice round-trips it —
// the value is back to off and the `· unsaved` title proves the toggles.
crate::baseline_case!(showcase_flows_settings_toggled_off_120x40_truecolor => Case::new("showcase/flows/settings_toggled_off_120x40_truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "space", "space", "wait:General · unsaved"]));

// ------------------------------------------- audit hole closures (§holes) --

// editor_diag: `}` jumps to the second block, Ctrl+R runs it; the run is 10
// ticks of 80 ms, then finish_run flags the unwrap() warning — the `77 ms`
// is computed (40 + 1·37), not measured, so the wait needle is exact.
crate::baseline_case!(showcase_flows_editor_diag_120x40_truecolor => Case::new("showcase/flows/editor_diag_120x40_truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "}", "ctrl-r", "wait:Block ran in 77 ms"]));
// editor_running: the in-flight run pinned — Ctrl+R sets run_ticks=10 and
// the running block, paused motion never decrements it, and the gutter
// spinner reads interaction.tick (code.rs:748), frozen at frame 0.
crate::baseline_case!(showcase_flows_editor_running_120x40_truecolor => Case::new("showcase/flows/editor_running_120x40_truecolor", SHOWCASE, &["--page", "codeeditor", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "}", "ctrl-r"]));
// datagrid_pending: seats of rows 1001/1002 edited (600 and 12) — the
// pending queue marks both rows `•`; ctrl-l selects the cell text so the
// typed value replaces rather than appends.
const GRID_EDITS: &[&str] = &[
    "tab",
    "right",
    "right",
    "right",
    "enter",
    "ctrl-l",
    "type:600",
    "enter",
    "down",
    "enter",
    "ctrl-l",
    "type:12",
    "enter",
    "wait:2 pending",
];
crate::baseline_case!(showcase_flows_datagrid_pending_120x40_truecolor => Case::new("showcase/flows/datagrid_pending_120x40_truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(GRID_EDITS));
// datagrid_failed: the "server" rejects seats > 500 (grid.rs finish_commit)
// — deterministic: row 1001 is marked `!`, row 1002 stays `•`. The 4-tick
// saving window is ridden out by the wait.
crate::baseline_case!(showcase_flows_datagrid_failed_120x40_truecolor => Case::new("showcase/flows/datagrid_failed_120x40_truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "ctrl-s", "wait:Save failed"]));
// datagrid_preview: `p` opens the Pending changes facts dialog with the
// exact UPDATE statements.
crate::baseline_case!(showcase_flows_datagrid_preview_120x40_truecolor => Case::new("showcase/flows/datagrid_preview_120x40_truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "p", "wait:UPDATE customers SET seats = 600"]));
// pickers_tabs: the Switch tab picker (second button).
crate::baseline_case!(showcase_flows_pickers_tabs_120x40_truecolor => Case::new("showcase/flows/pickers_tabs_120x40_truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:Filter tabs…"]));
// chips_select_open: second stop is the Sort by select; Enter opens its
// popup (`customer` exists only inside the popup).
crate::baseline_case!(showcase_flows_chips_select_open_120x40_truecolor => Case::new("showcase/flows/chips_select_open_120x40_truecolor", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:customer"]));
// settings_members: tabs bar → Members, then into the table, cursor row 2.
crate::baseline_case!(showcase_flows_settings_members_120x40_truecolor => Case::new("showcase/flows/settings_members_120x40_truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "tab", "down", "wait:cursor: Jonas Weber"]));
// chrome_menu_view: File menu open, `right` switches to the View menu.
crate::baseline_case!(showcase_flows_chrome_menu_view_120x40_truecolor => Case::new("showcase/flows/chrome_menu_view_120x40_truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "right", "wait:Zoom pane"]));
// chrome_focus: the menu bar focused with no menu open (the bar's hints
// are the proof — `← → Menu` only renders in this state).
crate::baseline_case!(showcase_flows_chrome_focus_120x40_truecolor => Case::new("showcase/flows/chrome_focus_120x40_truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "wait:← → Menu"]));

// --------------------------------------------------- post-flag (§5/§3.7) --
//
// `--motion paused` pins `interaction.tick` at `--frame N` and freezes every
// tick-derived renderer (spinner, indeterminate bar,Refreshing meter);
// `App::with_motion` fast-forwards the pages synchronously, so frame N is
// exactly the post-N-ticks state with no wall-clock involvement. The four
// progress statics restore the matrix entries the live spinner made
// uncapturable.

crate::baseline_case!(showcase_pages_progress_default_80x24_truecolor => Case::new("showcase/pages/progress_default_80x24_truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused"], 80, 24, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_progress_default_80x24_none => Case::new("showcase/pages/progress_default_80x24_none", SHOWCASE, &["--page", "progress", "--motion", "paused"], 80, 24, Color::None, BOOT));
crate::baseline_case!(showcase_pages_progress_default_120x40_truecolor => Case::new("showcase/pages/progress_default_120x40_truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_progress_default_120x40_none => Case::new("showcase/pages/progress_default_120x40_none", SHOWCASE, &["--page", "progress", "--motion", "paused"], 120, 40, Color::None, BOOT));
// build = 80 × 0.006 = 48 %, spinner pinned at frame 80.
crate::baseline_case!(showcase_flows_progress_mid_120x40_truecolor => Case::new("showcase/flows/progress_mid_120x40_truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, BOOT));
// 200 × 0.006 ≥ 1.0: the bar is Done (100% ✓). The transient "Build
// finished" status is dropped by the fast-forward (wall-clock artifact).
crate::baseline_case!(showcase_flows_progress_done_120x40_truecolor => Case::new("showcase/flows/progress_done_120x40_truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused", "--frame", "200"], 120, 40, Color::Truecolor, BOOT));
// `r` under paused motion: the pipeline has started (log line, running
// chrome, Cancel enabled) but no task tick ever fires — the just-started
// state the legacy `f_taskrunner_running` shot could not pin.
crate::baseline_case!(showcase_flows_taskrunner_running_120x40_truecolor => Case::new("showcase/flows/taskrunner_running_120x40_truecolor", SHOWCASE, &["--page", "taskrunner", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT).sends(&["r"]));
// 400 boot lines + 800 ticks = 1200 log lines, mid-stream, follow-tail on —
// replaces the ~2.5 min boot wait for a mid-stream frame.
crate::baseline_case!(showcase_flows_scrolling_mid_120x40_truecolor => Case::new("showcase/flows/scrolling_mid_120x40_truecolor", SHOWCASE, &["--page", "scrolling", "--motion", "paused", "--frame", "800"], 120, 40, Color::Truecolor, BOOT));
// 60 ticks into the staged run: Resolve + Pull done, Build container
// running (24 of 40), the rail meta and layer lines all pinned.
crate::baseline_case!(showcase_flows_terminal_mid_120x40_truecolor => Case::new("showcase/flows/terminal_mid_120x40_truecolor", SHOWCASE, &["--page", "terminal", "--motion", "paused", "--frame", "60"], 120, 40, Color::Truecolor, BOOT));

// ------------------------------------------------- audit-flow variants --
//
// The four keyboard flows proven at 120x40/truecolor re-run at the remaining
// 8 size×colour combos of the audit-flow matrix ({80x24,160x50} ×
// {truecolor,none,nocolor} + 120x40 × {none,nocolor}); the drag-select flow
// variants live in pointer.rs (they need the live session).

fn flow_variants(leaf: &str, args: &'static [&'static str], sends: &'static [&'static str]) {
    let mut failures = Vec::new();
    for (cols, rows, color) in support::FLOW_VARIANTS {
        let name = support::showcase_flow_name(leaf, cols, rows, color);
        let case =
            Case::dynamic(name.clone(), SHOWCASE, args, cols, rows, color, BOOT).sends(sends);
        if !support::collect_matrix(&name, || support::run_and_assert(&case)) {
            failures.push(name);
        }
    }
    support::finish_matrix(&failures);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_diff_review_variants() {
    flow_variants(
        support::FLOW_LEAF_DIFF_REVIEW,
        &["--page", "diff"],
        &["tab", "enter", "wait:● Review"],
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_diff_empty_variants() {
    flow_variants(
        support::FLOW_LEAF_DIFF_EMPTY,
        &["--page", "diff"],
        &["tab", "enter", "tab", "enter", "wait:No file selected"],
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_forms_invalid_variants() {
    flow_variants(
        support::FLOW_LEAF_FORMS_INVALID,
        &["--page", "forms"],
        &["tab", "ctrl-s", "wait:Required"],
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_inputs_selected_variants() {
    flow_variants(
        support::FLOW_LEAF_INPUTS_SELECTED,
        &["--page", "inputs"],
        &["tab", "enter", "ctrl-l", "wait:EDIT"],
    );
}
