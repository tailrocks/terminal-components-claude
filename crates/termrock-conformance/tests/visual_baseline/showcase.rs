use crate::support::{Case, Color, SHOWCASE};

const BOOT: &str = "Junie Design system";

crate::baseline_case!(showcase_pages_overview_default_120x40_truecolor => Case::new("showcase/pages/overview/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_lists_default_120x40_truecolor => Case::new("showcase/pages/lists/120x40/truecolor", SHOWCASE, &["--page", "lists"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_trees_default_120x40_truecolor => Case::new("showcase/pages/trees/120x40/truecolor", SHOWCASE, &["--page", "trees"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_tables_default_120x40_truecolor => Case::new("showcase/pages/tables/120x40/truecolor", SHOWCASE, &["--page", "tables"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_editable_default_120x40_truecolor => Case::new("showcase/pages/editable/120x40/truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_panels_default_120x40_truecolor => Case::new("showcase/pages/panels/120x40/truecolor", SHOWCASE, &["--page", "panels"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_sidebars_default_120x40_truecolor => Case::new("showcase/pages/sidebars/120x40/truecolor", SHOWCASE, &["--page", "sidebars"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_dialogs_default_120x40_truecolor => Case::new("showcase/pages/dialogs/120x40/truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_codeeditor_default_120x40_truecolor => Case::new("showcase/pages/codeeditor/120x40/truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_datagrid_default_120x40_truecolor => Case::new("showcase/pages/datagrid/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_chips_default_120x40_truecolor => Case::new("showcase/pages/chips/120x40/truecolor", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_pickers_default_120x40_truecolor => Case::new("showcase/pages/pickers/120x40/truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_chrome_default_120x40_truecolor => Case::new("showcase/pages/chrome/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_settings_default_120x40_truecolor => Case::new("showcase/pages/settings/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT));
crate::baseline_case!(showcase_pages_taskrunner_default_120x40_truecolor => Case::new("showcase/pages/taskrunner/120x40/truecolor", SHOWCASE, &["--page", "taskrunner"], 120, 40, Color::Truecolor, BOOT));

crate::baseline_case!(showcase_pages_scrolling_default_120x40_truecolor => Case::new("showcase/pages/scrolling/120x40/truecolor", SHOWCASE, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:739.63s"]));

crate::baseline_case!(showcase_pages_terminal_default_120x40_truecolor => Case::new("showcase/pages/terminal/120x40/truecolor", SHOWCASE, &["--page", "terminal"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:7 of 7"]).timeout(60000));

crate::baseline_case!(showcase_flows_inputs_editing_120x40_truecolor => Case::new("showcase/flows/inputs/editing/120x40/truecolor", SHOWCASE, &["--page", "inputs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:EDIT"]));
crate::baseline_case!(showcase_flows_inputs_selected_120x40_truecolor => Case::new("showcase/flows/inputs/selected/120x40/truecolor", SHOWCASE, &["--page", "inputs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "ctrl-l", "wait:EDIT"]));
crate::baseline_case!(showcase_flows_forms_invalid_120x40_truecolor => Case::new("showcase/flows/forms/invalid/120x40/truecolor", SHOWCASE, &["--page", "forms"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "ctrl-s", "wait:Required"]));
crate::baseline_case!(showcase_flows_diff_review_120x40_truecolor => Case::new("showcase/flows/diff/review/120x40/truecolor", SHOWCASE, &["--page", "diff"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:● Review"]));
crate::baseline_case!(showcase_flows_diff_empty_120x40_truecolor => Case::new("showcase/flows/diff/empty/120x40/truecolor", SHOWCASE, &["--page", "diff"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "tab", "enter", "wait:No file selected"]));
crate::baseline_case!(showcase_flows_buttons_focus_120x40_truecolor => Case::new("showcase/flows/buttons/focus/120x40/truecolor", SHOWCASE, &["--page", "buttons"], 120, 40, Color::Truecolor, BOOT).sends(&["tab"]));
crate::baseline_case!(showcase_flows_lists_moved_120x40_truecolor => Case::new("showcase/flows/lists/moved/120x40/truecolor", SHOWCASE, &["--page", "lists"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "down", "down"]));
crate::baseline_case!(showcase_flows_trees_expanded_120x40_truecolor => Case::new("showcase/flows/trees/expanded/120x40/truecolor", SHOWCASE, &["--page", "trees"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right"]));
crate::baseline_case!(showcase_flows_tables_selected_120x40_truecolor => Case::new("showcase/flows/tables/selected/120x40/truecolor", SHOWCASE, &["--page", "tables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "down", "down"]));
crate::baseline_case!(showcase_flows_editable_editing_120x40_truecolor => Case::new("showcase/flows/editable/editing/120x40/truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter"]));
crate::baseline_case!(showcase_flows_datagrid_selected_120x40_truecolor => Case::new("showcase/flows/datagrid/selected/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "down", "right"]));
crate::baseline_case!(showcase_flows_dialogs_open_120x40_truecolor => Case::new("showcase/flows/dialogs/open/120x40/truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter"]));
crate::baseline_case!(showcase_flows_pickers_open_120x40_truecolor => Case::new("showcase/flows/pickers/open/120x40/truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter"]));
crate::baseline_case!(showcase_flows_chips_toggled_120x40_truecolor => Case::new("showcase/flows/chips/toggled/120x40/truecolor", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "space"]));

crate::baseline_case!(showcase_flows_scrolling_scrolled_120x40_truecolor => Case::new("showcase/flows/scrolling/scrolled/120x40/truecolor", SHOWCASE, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:739.63s", "tab", "down", "down", "down"]));
crate::baseline_case!(showcase_flows_settings_toggled_120x40_truecolor => Case::new("showcase/flows/settings/toggled/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "space"]));
crate::baseline_case!(showcase_flows_help_overlay_120x40_truecolor => Case::new("showcase/flows/help/overlay/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["?"]));
crate::baseline_case!(showcase_flows_inspector_open_120x40_truecolor => Case::new("showcase/flows/inspector/open/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["i"]));

crate::baseline_case!(showcase_flows_dialogs_prompt_120x40_truecolor => Case::new("showcase/flows/dialogs/prompt/120x40/truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:Shown in the task list"]));

crate::baseline_case!(showcase_flows_dialogs_destructive_120x40_truecolor => Case::new("showcase/flows/dialogs/destructive/120x40/truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["d", "wait:Delete branch?"]));

crate::baseline_case!(showcase_flows_forms_valid_120x40_truecolor => Case::new("showcase/flows/forms/valid/120x40/truecolor", SHOWCASE, &["--page", "forms"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "type:Fix the login redirect loop", "enter", "ctrl-s", "wait:Task created ✓"]));

crate::baseline_case!(showcase_flows_nav_cycled_120x40_truecolor => Case::new("showcase/flows/nav/cycled/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["]", "wait:/ Components / Buttons"]));

crate::baseline_case!(showcase_flows_nav_end_120x40_truecolor => Case::new("showcase/flows/nav/end/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["0", "G"]));

crate::baseline_case!(showcase_flows_pickers_level_120x40_truecolor => Case::new("showcase/flows/pickers/level/120x40/truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "enter", "down", "enter", "wait:Chose Safe Mode (Full)"]));

crate::baseline_case!(showcase_flows_editor_completion_120x40_truecolor => Case::new("showcase/flows/editor/completion/120x40/truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "i", "type:cl", "wait:Client"]));

crate::baseline_case!(showcase_flows_chrome_menu_120x40_truecolor => Case::new("showcase/flows/chrome/menu/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:New tab"]));

crate::baseline_case!(showcase_flows_editable_committed_120x40_truecolor => Case::new("showcase/flows/editable/committed/120x40/truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "type:x", "enter", "wait:1 edits"]));

crate::baseline_case!(showcase_flows_settings_toggled_off_120x40_truecolor => Case::new("showcase/flows/settings/toggled_off/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "space", "space", "wait:General · unsaved"]));

crate::baseline_case!(showcase_flows_editor_diag_120x40_truecolor => Case::new("showcase/flows/editor/diag/120x40/truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "}", "ctrl-r", "wait:Block ran in 77 ms"]));

crate::baseline_case!(showcase_flows_editor_running_120x40_truecolor => Case::new("showcase/flows/editor/running/120x40/truecolor", SHOWCASE, &["--page", "codeeditor", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "}", "ctrl-r"]));

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
crate::baseline_case!(showcase_flows_datagrid_pending_120x40_truecolor => Case::new("showcase/flows/datagrid/pending/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(GRID_EDITS));

crate::baseline_case!(showcase_flows_datagrid_failed_120x40_truecolor => Case::new("showcase/flows/datagrid/failed/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "ctrl-s", "wait:Save failed"]));

crate::baseline_case!(showcase_flows_datagrid_preview_120x40_truecolor => Case::new("showcase/flows/datagrid/preview/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "p", "wait:UPDATE customers SET seats = 600"]));

crate::baseline_case!(showcase_flows_pickers_tabs_120x40_truecolor => Case::new("showcase/flows/pickers/tabs/120x40/truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:Filter tabs…"]));

crate::baseline_case_with_variants!(
    showcase_flows_chips_select_open_120x40_truecolor => Case::new("showcase/flows/chips/select_open/120x40/truecolor", SHOWCASE, &["--page", "chipsselects"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:customer"]),
    [
        Case::new("showcase/flows/chips/select_open/72x20/truecolor", SHOWCASE, &["--page", "chipsselects"], 72, 20, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/72x20/256", SHOWCASE, &["--page", "chipsselects"], 72, 20, Color::Ansi256, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/72x20/16", SHOWCASE, &["--page", "chipsselects"], 72, 20, Color::Ansi16, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/72x20/none", SHOWCASE, &["--page", "chipsselects"], 72, 20, Color::None, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/72x20/nocolor", SHOWCASE, &["--page", "chipsselects"], 72, 20, Color::NoColorEnv, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/80x24/truecolor", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/80x24/256", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::Ansi256, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/80x24/16", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::Ansi16, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/80x24/none", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::None, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/80x24/nocolor", SHOWCASE, &["--page", "chipsselects"], 80, 24, Color::NoColorEnv, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/100x30/truecolor", SHOWCASE, &["--page", "chipsselects"], 100, 30, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/100x30/256", SHOWCASE, &["--page", "chipsselects"], 100, 30, Color::Ansi256, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/100x30/16", SHOWCASE, &["--page", "chipsselects"], 100, 30, Color::Ansi16, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/100x30/none", SHOWCASE, &["--page", "chipsselects"], 100, 30, Color::None, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Case::new("showcase/flows/chips/select_open/100x30/nocolor", SHOWCASE, &["--page", "chipsselects"], 100, 30, Color::NoColorEnv, BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
    ],
);

crate::baseline_case!(showcase_flows_settings_members_120x40_truecolor => Case::new("showcase/flows/settings/members/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "tab", "down", "wait:cursor: Jonas Weber"]));

crate::baseline_case!(showcase_flows_chrome_menu_view_120x40_truecolor => Case::new("showcase/flows/chrome/menu_view/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "right", "wait:Zoom pane"]));

crate::baseline_case!(showcase_flows_chrome_focus_120x40_truecolor => Case::new("showcase/flows/chrome/focus/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "wait:← → Menu"]));

crate::baseline_case!(showcase_pages_progress_default_120x40_truecolor => Case::new("showcase/pages/progress/120x40/truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT));

crate::baseline_case!(showcase_flows_progress_mid_120x40_truecolor => Case::new("showcase/flows/progress/mid/120x40/truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, BOOT));

crate::baseline_case!(showcase_flows_progress_done_120x40_truecolor => Case::new("showcase/flows/progress/done/120x40/truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused", "--frame", "200"], 120, 40, Color::Truecolor, BOOT));

crate::baseline_case!(showcase_flows_taskrunner_running_120x40_truecolor => Case::new("showcase/flows/taskrunner/running/120x40/truecolor", SHOWCASE, &["--page", "taskrunner", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT).sends(&["r"]));

crate::baseline_case!(showcase_flows_scrolling_mid_120x40_truecolor => Case::new("showcase/flows/scrolling/mid/120x40/truecolor", SHOWCASE, &["--page", "scrolling", "--motion", "paused", "--frame", "800"], 120, 40, Color::Truecolor, BOOT));

crate::baseline_case!(showcase_flows_terminal_mid_120x40_truecolor => Case::new("showcase/flows/terminal/mid/120x40/truecolor", SHOWCASE, &["--page", "terminal", "--motion", "paused", "--frame", "60"], 120, 40, Color::Truecolor, BOOT));
