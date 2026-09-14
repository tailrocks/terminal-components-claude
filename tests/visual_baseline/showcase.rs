//! showcase captures (151): 18 non-audit pages under `showcase/pages/`
//! (progress statics restored via `--motion paused`, §5), keyboard/mouse
//! flows under `showcase/flows/` (18 remapped + 10 from the §3.7 table + 5
//! post-flag + 10 audit hole closures). The 5 audit pages
//! (buttons/diff/forms/inputs/textareas) keep statics only in audit.rs
//! (`showcase/audit/`, 5×5 matrix — the dedupe rule). Every showcase
//! representative root runs the shared 5×5 canonical matrix; the pointer
//! group applies the same matrix to its six resize roots.
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs); only the store names were regrouped
//! (`showcase_<leaf>` → `showcase/<sub_group>/<leaf>`). Do not hand-tune:
//! drift against the approved frames means the port or the app changed.

use crate::support::{Case, Color, SHOWCASE};

const BOOT: &str = "Junie Design system";

// ------------------------------------------------------------------ pages --

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
// `wait:739.63s` is the boot stream's end-state marker (the 2000th log
// line's timestamp). Without it wait_idle can fire mid-stream and the gated
// frame depends on flush timing.
crate::baseline_case!(showcase_pages_scrolling_default_120x40_truecolor => Case::new("showcase/pages/scrolling/120x40/truecolor", SHOWCASE, &["--page", "scrolling"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:739.63s"]).timeout(180000));
// `wait:7 of 7` is the step-rail end-state; 30s is tight under load.
crate::baseline_case!(showcase_pages_terminal_default_120x40_truecolor => Case::new("showcase/pages/terminal/120x40/truecolor", SHOWCASE, &["--page", "terminal"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:7 of 7"]).timeout(60000));

// ------------------------------------------------------------------ flows --

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
// `wait:739.63s` is the boot stream's end-state marker (the 2000th log
// line's timestamp, unique on screen): without it the sends can begin during
// a >200 ms stall mid-stream (wait_idle fires early under load) and race the
// stream tail, leaving the frame's final cursor position — embedded in the
// gated HTML — dependent on flush timing.
crate::baseline_case!(showcase_flows_scrolling_scrolled_120x40_truecolor => Case::new("showcase/flows/scrolling/scrolled/120x40/truecolor", SHOWCASE, &["--page", "scrolling"], 120, 40, Color::Truecolor, BOOT).sends(&["wait:739.63s", "tab", "down", "down", "down"]).timeout(180000));
crate::baseline_case!(showcase_flows_settings_toggled_120x40_truecolor => Case::new("showcase/flows/settings/toggled/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "space"]));
crate::baseline_case!(showcase_flows_help_overlay_120x40_truecolor => Case::new("showcase/flows/help/overlay/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["?"]));
crate::baseline_case!(showcase_flows_inspector_open_120x40_truecolor => Case::new("showcase/flows/inspector/open/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["i"]));

// ------------------------------------------------------- new flows (§3.7) --

// S1: the prompt dialog (second button), input prefilled and validated.
crate::baseline_case!(showcase_flows_dialogs_prompt_120x40_truecolor => Case::new("showcase/flows/dialogs/prompt/120x40/truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:Shown in the task list"]));
// S2: the destructive dialog (fourth button), Cancel focused first.
crate::baseline_case!(showcase_flows_dialogs_destructive_120x40_truecolor => Case::new("showcase/flows/dialogs/destructive/120x40/truecolor", SHOWCASE, &["--page", "dialogs"], 120, 40, Color::Truecolor, BOOT).sends(&["d", "wait:Delete branch?"]));
// S3: required field filled, Ctrl+S, then wait out the 1.8 s wall-clock
// Busy window — the Done message is the stable end state (the bash
// `sleep:1200` predates the 1800 ms busy constant; `wait:` is exact).
crate::baseline_case!(showcase_flows_forms_valid_120x40_truecolor => Case::new("showcase/flows/forms/valid/120x40/truecolor", SHOWCASE, &["--page", "forms"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "type:Fix the login redirect loop", "enter", "ctrl-s", "wait:Task created ✓"]));
// S4: `]` cycles to the next page; the header crumb is the unique proof.
crate::baseline_case!(showcase_flows_nav_cycled_120x40_truecolor => Case::new("showcase/flows/nav/cycled/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["]", "wait:/ Components / Buttons"]));
// S5: nav owns the keyboard; `G` lands the cursor on the last page (the
// state proof is the focused gutter row in the frame — no text changes).
crate::baseline_case!(showcase_flows_nav_end_120x40_truecolor => Case::new("showcase/flows/nav/end/120x40/truecolor", SHOWCASE, &["--page", "overview"], 120, 40, Color::Truecolor, BOOT).sends(&["0", "G"]));
// S6: the Level picker (third button) is not searchable; one down from the
// current level, Enter sets it — the footer status confirms the choice.
crate::baseline_case!(showcase_flows_pickers_level_120x40_truecolor => Case::new("showcase/flows/pickers/level/120x40/truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "enter", "down", "enter", "wait:Chose Safe Mode (Full)"]));
// S7: two typed characters open the completion popup (i = edit mode).
crate::baseline_case!(showcase_flows_editor_completion_120x40_truecolor => Case::new("showcase/flows/editor/completion/120x40/truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "i", "type:cl", "wait:Client"]));
// S8: the menu bar is the page's first stop; Enter opens the File menu.
crate::baseline_case!(showcase_flows_chrome_menu_120x40_truecolor => Case::new("showcase/flows/chrome/menu/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "wait:New tab"]));
// S10: cell edit committed; the panel meta counts edits.
crate::baseline_case!(showcase_flows_editable_committed_120x40_truecolor => Case::new("showcase/flows/editable/committed/120x40/truecolor", SHOWCASE, &["--page", "editabletables"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "type:x", "enter", "wait:1 edits"]));
// S11: fifth stop is the Auto-merge toggle; space twice round-trips it —
// the value is back to off and the `· unsaved` title proves the toggles.
crate::baseline_case!(showcase_flows_settings_toggled_off_120x40_truecolor => Case::new("showcase/flows/settings/toggled_off/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "space", "space", "wait:General · unsaved"]));

// ------------------------------------------- audit hole closures (§holes) --

// editor_diag: `}` jumps to the second block, Ctrl+R runs it; the run is 10
// ticks of 80 ms, then finish_run flags the unwrap() warning — the `77 ms`
// is computed (40 + 1·37), not measured, so the wait needle is exact.
crate::baseline_case!(showcase_flows_editor_diag_120x40_truecolor => Case::new("showcase/flows/editor/diag/120x40/truecolor", SHOWCASE, &["--page", "codeeditor"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "}", "ctrl-r", "wait:Block ran in 77 ms"]));
// editor_running: the in-flight run pinned — Ctrl+R sets run_ticks=10 and
// the running block, paused motion never decrements it, and the gutter
// spinner reads interaction.tick (code.rs:748), frozen at frame 0.
crate::baseline_case!(showcase_flows_editor_running_120x40_truecolor => Case::new("showcase/flows/editor/running/120x40/truecolor", SHOWCASE, &["--page", "codeeditor", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "}", "ctrl-r"]));
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
crate::baseline_case!(showcase_flows_datagrid_pending_120x40_truecolor => Case::new("showcase/flows/datagrid/pending/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(GRID_EDITS));
// datagrid_failed: the "server" rejects seats > 500 (grid.rs finish_commit)
// — deterministic: row 1001 is marked `!`, row 1002 stays `•`. The 4-tick
// saving window is ridden out by the wait.
crate::baseline_case!(showcase_flows_datagrid_failed_120x40_truecolor => Case::new("showcase/flows/datagrid/failed/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "ctrl-s", "wait:Save failed"]));
// datagrid_preview: `p` opens the Pending changes facts dialog with the
// exact UPDATE statements.
crate::baseline_case!(showcase_flows_datagrid_preview_120x40_truecolor => Case::new("showcase/flows/datagrid/preview/120x40/truecolor", SHOWCASE, &["--page", "datagrid"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "p", "wait:UPDATE customers SET seats = 600"]));
// pickers_tabs: the Switch tab picker (second button).
crate::baseline_case!(showcase_flows_pickers_tabs_120x40_truecolor => Case::new("showcase/flows/pickers/tabs/120x40/truecolor", SHOWCASE, &["--page", "pickers"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "tab", "enter", "wait:Filter tabs…"]));
// chips_select_open: second stop is the Sort by select; Enter opens its
// popup (`customer` exists only inside the popup).
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
// settings_members: tabs bar → Members, then into the table, cursor row 2.
crate::baseline_case!(showcase_flows_settings_members_120x40_truecolor => Case::new("showcase/flows/settings/members/120x40/truecolor", SHOWCASE, &["--page", "settings"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "right", "tab", "down", "wait:cursor: Jonas Weber"]));
// chrome_menu_view: File menu open, `right` switches to the View menu.
crate::baseline_case!(showcase_flows_chrome_menu_view_120x40_truecolor => Case::new("showcase/flows/chrome/menu_view/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "enter", "right", "wait:Zoom pane"]));
// chrome_focus: the menu bar focused with no menu open (the bar's hints
// are the proof — `← → Menu` only renders in this state).
crate::baseline_case!(showcase_flows_chrome_focus_120x40_truecolor => Case::new("showcase/flows/chrome/focus/120x40/truecolor", SHOWCASE, &["--page", "chrome"], 120, 40, Color::Truecolor, BOOT).sends(&["tab", "wait:← → Menu"]));

// --------------------------------------------------- post-flag (§5/§3.7) --
//
// `--motion paused` pins `interaction.tick` at `--frame N` and freezes every
// tick-derived renderer (spinner, indeterminate bar,Refreshing meter);
// `App::with_motion` fast-forwards the pages synchronously, so frame N is
// exactly the post-N-ticks state with no wall-clock involvement. The four
// progress statics restore the matrix entries the live spinner made
// uncapturable.

crate::baseline_case!(showcase_pages_progress_default_120x40_truecolor => Case::new("showcase/pages/progress/120x40/truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT));
// build = 80 × 0.006 = 48 %, spinner pinned at frame 80.
crate::baseline_case!(showcase_flows_progress_mid_120x40_truecolor => Case::new("showcase/flows/progress/mid/120x40/truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, BOOT));
// 200 × 0.006 ≥ 1.0: the bar is Done (100% ✓). The transient "Build
// finished" status is dropped by the fast-forward (wall-clock artifact).
crate::baseline_case!(showcase_flows_progress_done_120x40_truecolor => Case::new("showcase/flows/progress/done/120x40/truecolor", SHOWCASE, &["--page", "progress", "--motion", "paused", "--frame", "200"], 120, 40, Color::Truecolor, BOOT));
// `r` under paused motion: the pipeline has started (log line, running
// chrome, Cancel enabled) but no task tick ever fires — the just-started
// state the legacy `f_taskrunner_running` shot could not pin.
crate::baseline_case!(showcase_flows_taskrunner_running_120x40_truecolor => Case::new("showcase/flows/taskrunner/running/120x40/truecolor", SHOWCASE, &["--page", "taskrunner", "--motion", "paused"], 120, 40, Color::Truecolor, BOOT).sends(&["r"]));
// 400 boot lines + 800 ticks = 1200 log lines, mid-stream, follow-tail on —
// replaces the ~2.5 min boot wait for a mid-stream frame.
crate::baseline_case!(showcase_flows_scrolling_mid_120x40_truecolor => Case::new("showcase/flows/scrolling/mid/120x40/truecolor", SHOWCASE, &["--page", "scrolling", "--motion", "paused", "--frame", "800"], 120, 40, Color::Truecolor, BOOT));
// 60 ticks into the staged run: Resolve + Pull done, Build container
// running (24 of 40), the rail meta and layer lines all pinned.
crate::baseline_case!(showcase_flows_terminal_mid_120x40_truecolor => Case::new("showcase/flows/terminal/mid/120x40/truecolor", SHOWCASE, &["--page", "terminal", "--motion", "paused", "--frame", "60"], 120, 40, Color::Truecolor, BOOT));
