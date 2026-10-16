//! showcase scenario tables: 18 legacy pages + `selectable` under
//! `showcase/pages/`, keyboard and pointer flows under `showcase/flows/`
//! (journeys), wheel scroll-fade under `showcase/fade/`, hover under
//! `showcase/hover/` (elements), the overview resize pair, and the 5 audit
//! fixtures (buttons/diff/forms/inputs/textareas) under `showcase/audit/`.
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs) through `showcase.rs`, `pointer.rs` and `audit.rs`;
//! only the store names were regrouped. Do not hand-tune: drift against the
//! approved frames means the port or the app changed.

use super::{Bin, Root, Variant, interact};
use crate::support::Color;

const BOOT: &str = "Junie Design system";

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

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[
    // ------------------------------------------------------------------ pages --
    Root::new("showcase/pages/overview", Bin::Showcase, &["--page", "overview"], BOOT),
    Root::new("showcase/pages/lists", Bin::Showcase, &["--page", "lists"], BOOT),
    Root::new("showcase/pages/trees", Bin::Showcase, &["--page", "trees"], BOOT),
    Root::new("showcase/pages/tables", Bin::Showcase, &["--page", "tables"], BOOT),
    Root::new("showcase/pages/editable", Bin::Showcase, &["--page", "editabletables"], BOOT),
    Root::new("showcase/pages/panels", Bin::Showcase, &["--page", "panels"], BOOT),
    Root::new("showcase/pages/sidebars", Bin::Showcase, &["--page", "sidebars"], BOOT),
    Root::new("showcase/pages/dialogs", Bin::Showcase, &["--page", "dialogs"], BOOT),
    Root::new("showcase/pages/codeeditor", Bin::Showcase, &["--page", "codeeditor"], BOOT),
    Root::new("showcase/pages/datagrid", Bin::Showcase, &["--page", "datagrid"], BOOT),
    Root::new("showcase/pages/chips", Bin::Showcase, &["--page", "chipsselects"], BOOT),
    Root::new("showcase/pages/pickers", Bin::Showcase, &["--page", "pickers"], BOOT),
    Root::new("showcase/pages/chrome", Bin::Showcase, &["--page", "chrome"], BOOT),
    Root::new("showcase/pages/settings", Bin::Showcase, &["--page", "settings"], BOOT),
    Root::new("showcase/pages/taskrunner", Bin::Showcase, &["--page", "taskrunner"], BOOT),
    // Paused frame 1600 fast-forwards the 400-line log to its deterministic
    // 2000-line endpoint; `wait:739.63s` proves that endpoint on every axis.
    Root::new("showcase/pages/scrolling", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s"]),
    // `✓ 07 Ready` is the step-rail end-state at every width (the rail's
    // `7 of 7` counter compacts to `7/7` at 72 columns); 30s is tight under load.
    Root::new("showcase/pages/terminal", Bin::Showcase, &["--page", "terminal"], BOOT).sends(&["wait:✓ 07 Ready"]).timeout(60000),
    // Selectable table (pages/selectable.rs): members table in multi-selection
    // mode beside a member editor (stacked under the table below 120 columns,
    // so readiness waits on a table row). Default state: nothing selected.
    Root::new("showcase/pages/selectable", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno"]),
    // --------------------------------------------------- post-flag (§5/§3.7) --
    //
    // `--motion paused` pins `interaction.tick` at `--frame N` and freezes every
    // tick-derived renderer (spinner, indeterminate bar,Refreshing meter);
    // `App::with_motion` fast-forwards the pages synchronously, so frame N is
    // exactly the post-N-ticks state with no wall-clock involvement. The four
    // progress statics restore the matrix entries the live spinner made
    // uncapturable.
    Root::new("showcase/pages/progress", Bin::Showcase, &["--page", "progress", "--motion", "paused"], BOOT),
    // ------------------------------------------------------------------ audit --
    Root::new("showcase/audit/buttons", Bin::Showcase, &["--page", "buttons"], BOOT),
    Root::new("showcase/audit/diff", Bin::Showcase, &["--page", "diff"], BOOT),
    Root::new("showcase/audit/forms", Bin::Showcase, &["--page", "forms"], BOOT),
    Root::new("showcase/audit/inputs", Bin::Showcase, &["--page", "inputs"], BOOT),
    Root::new("showcase/audit/textareas", Bin::Showcase, &["--page", "textareas"], BOOT),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[
    // ------------------------------------------------------------------ hover --
    Root::new("showcase/hover/buttons", Bin::Showcase, &["--page", "buttons"], BOOT).live(interact::hover_buttons),
    Root::new("showcase/hover/lists", Bin::Showcase, &["--page", "lists"], BOOT).live(interact::hover_lists),
    Root::new("showcase/hover/tables", Bin::Showcase, &["--page", "tables"], BOOT).live(interact::hover_tables),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[
    // ------------------------------------------------------------------ flows --
    Root::new("showcase/flows/inputs/editing", Bin::Showcase, &["--page", "inputs"], BOOT).sends(&["tab", "enter", "wait:EDIT"]),
    Root::new("showcase/flows/inputs/selected", Bin::Showcase, &["--page", "inputs"], BOOT).sends(&["tab", "enter", "ctrl-l", "wait:EDIT"]),
    Root::new("showcase/flows/forms/invalid", Bin::Showcase, &["--page", "forms"], BOOT).sends(&["tab", "ctrl-s", "wait:Required"]),
    Root::new("showcase/flows/diff/review", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "enter", "wait:● Review"]),
    Root::new("showcase/flows/diff/empty", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "enter", "tab", "enter", "wait:No file selected"]),
    Root::new("showcase/flows/buttons/focus", Bin::Showcase, &["--page", "buttons"], BOOT).sends(&["tab"]),
    Root::new("showcase/flows/lists/moved", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["tab", "down", "down"]),
    Root::new("showcase/flows/trees/expanded", Bin::Showcase, &["--page", "trees"], BOOT).sends(&["tab", "right"]),
    Root::new("showcase/flows/tables/selected", Bin::Showcase, &["--page", "tables"], BOOT).sends(&["tab", "down", "down"]),
    Root::new("showcase/flows/editable/editing", Bin::Showcase, &["--page", "editabletables"], BOOT).sends(&["tab", "enter"]),
    Root::new("showcase/flows/datagrid/selected", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "down", "right"]),
    Root::new("showcase/flows/dialogs/open", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "enter"]),
    Root::new("showcase/flows/pickers/open", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "enter"]),
    Root::new("showcase/flows/chips/toggled", Bin::Showcase, &["--page", "chipsselects"], BOOT).sends(&["tab", "space"]),
    // The same paused endpoint keeps the subsequent scroll actions deterministic.
    Root::new("showcase/flows/scrolling/scrolled", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s", "tab", "down", "down", "down"]),
    Root::new("showcase/flows/settings/toggled", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "space"]),
    Root::new("showcase/flows/help/overlay", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["?"]),
    Root::new("showcase/flows/inspector/open", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["i"]),
    // Selectable table journeys (keyboard only, so every size runs the same
    // steps): Tab focuses the table, the owner row is disabled, Space selects
    // Ari, Shift+Down extends over Cleo and Bo — two roles, so the editor bulk
    // edits with `— mixed —`; `d` on two selected rows asks `Delete 2 members?`.
    Root::new("showcase/flows/selectable/bulk_mixed", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "down", "space", "wait:1 selected", "shift-down", "shift-down", "wait:3 selected"]),
    Root::new("showcase/flows/selectable/delete_confirm", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "down", "space", "wait:1 selected", "shift-down", "wait:2 selected", "d", "wait:Delete 2 members?"]),
    // ------------------------------------------------------- new flows (§3.7) --

    // S1: the prompt dialog (second button), input prefilled and validated.
    Root::new("showcase/flows/dialogs/prompt", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "tab", "enter", "wait:Shown in the task list"]),
    // S2: the destructive dialog (fourth button), Cancel focused first.
    Root::new("showcase/flows/dialogs/destructive", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["d", "wait:Delete branch?"]),
    // S3: required field filled, Ctrl+S, then wait out the 1.8 s wall-clock
    // Busy window — the Done message is the stable end state (the bash
    // `sleep:1200` predates the 1800 ms busy constant; `wait:` is exact).
    Root::new("showcase/flows/forms/valid", Bin::Showcase, &["--page", "forms"], BOOT).sends(&["tab", "enter", "type:Fix the login redirect loop", "enter", "ctrl-s", "wait:Task created ✓"]),
    // S4: `]` cycles to the next page; the header crumb is the unique proof.
    Root::new("showcase/flows/nav/cycled", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["]", "wait:/ Components / Buttons"]),
    // S5: nav owns the keyboard; `G` lands the cursor on the last page (the
    // state proof is the focused gutter row in the frame — no text changes).
    Root::new("showcase/flows/nav/end", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["0", "G"]),
    // S6: the Level picker (third button) is not searchable; one down from the
    // current level, Enter sets it — the footer status confirms the choice.
    Root::new("showcase/flows/pickers/level", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "tab", "tab", "enter", "down", "enter", "wait:Chose Safe Mode (Full)"]),
    // S7: two typed characters open the completion popup (i = edit mode).
    Root::new("showcase/flows/editor/completion", Bin::Showcase, &["--page", "codeeditor"], BOOT).sends(&["tab", "i", "type:cl", "wait:Client"]),
    // S8: the menu bar is the page's first stop; Enter opens the File menu.
    Root::new("showcase/flows/chrome/menu", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "enter", "wait:New tab"]),
    // S10: cell edit committed; the panel meta counts edits.
    Root::new("showcase/flows/editable/committed", Bin::Showcase, &["--page", "editabletables"], BOOT).sends(&["tab", "enter", "type:x", "enter", "wait:1 edits"]),
    // S11: fifth stop is the Auto-merge toggle; space twice round-trips it —
    // the value is back to off and the `· unsaved` title proves the toggles.
    Root::new("showcase/flows/settings/toggled_off", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "space", "space", "wait:General · unsaved"]),
    // ------------------------------------------- audit hole closures (§holes) --

    // editor_diag: `}` jumps to the second block, Ctrl+R runs it; the run is 10
    // ticks of 80 ms, then finish_run flags the unwrap() warning — the `77 ms`
    // is computed (40 + 1·37), not measured, so the wait needle is exact.
    Root::new("showcase/flows/editor/diag", Bin::Showcase, &["--page", "codeeditor"], BOOT).sends(&["tab", "}", "ctrl-r", "wait:Block ran in 77 ms"]),
    // editor_running: the in-flight run pinned — Ctrl+R sets run_ticks=10 and
    // the running block, paused motion never decrements it, and the gutter
    // spinner reads interaction.tick (code.rs:748), frozen at frame 0.
    Root::new("showcase/flows/editor/running", Bin::Showcase, &["--page", "codeeditor", "--motion", "paused"], BOOT).sends(&["tab", "}", "ctrl-r"]),
    Root::new("showcase/flows/datagrid/pending", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(GRID_EDITS),
    // datagrid_failed: the "server" rejects seats > 500 (grid.rs finish_commit)
    // — deterministic: row 1001 is marked `!`, row 1002 stays `•`. The 4-tick
    // saving window is ridden out by the wait.
    Root::new("showcase/flows/datagrid/failed", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "ctrl-s", "wait:Save failed"]),
    // datagrid_preview: `p` opens the Pending changes facts dialog with the
    // exact UPDATE statements.
    Root::new("showcase/flows/datagrid/preview", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "p", "wait:UPDATE customers SET seats = 600"]),
    // pickers_tabs: the Switch tab picker (second button).
    Root::new("showcase/flows/pickers/tabs", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "tab", "enter", "wait:Filter tabs…"]),
    // chips_select_open: second stop is the Sort by select; Enter opens its
    // popup (`customer` exists only inside the popup).
    Root::new("showcase/flows/chips/select_open", Bin::Showcase, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:customer"]).variants(&[
        Variant::new(72, 20, Color::Truecolor, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(72, 20, Color::Ansi256, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(72, 20, Color::Ansi16, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(72, 20, Color::None, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(72, 20, Color::NoColorEnv, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(80, 24, Color::Truecolor, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(80, 24, Color::Ansi256, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(80, 24, Color::Ansi16, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(80, 24, Color::None, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(80, 24, Color::NoColorEnv, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(100, 30, Color::Truecolor, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(100, 30, Color::Ansi256, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(100, 30, Color::Ansi16, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(100, 30, Color::None, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
        Variant::new(100, 30, Color::NoColorEnv, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "enter", "wait:500"]),
    ]),
    // settings_members: tabs bar → Members, then into the table, cursor row 2.
    Root::new("showcase/flows/settings/members", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "right", "tab", "down", "wait:cursor: Jonas Weber"]),
    // chrome_menu_view: File menu open, `right` switches to the View menu.
    Root::new("showcase/flows/chrome/menu_view", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "enter", "right", "wait:Zoom pane"]),
    // chrome_focus: the menu bar focused with no menu open (the bar's hints
    // are the proof — `← → Menu` only renders in this state).
    Root::new("showcase/flows/chrome/focus", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "wait:← → Menu"]),
    // build = 80 × 0.006 = 48 %, spinner pinned at frame 80.
    Root::new("showcase/flows/progress/mid", Bin::Showcase, &["--page", "progress", "--motion", "paused", "--frame", "80"], BOOT),
    // 200 × 0.006 ≥ 1.0: the bar is Done (100% ✓). The transient "Build
    // finished" status is dropped by the fast-forward (wall-clock artifact).
    Root::new("showcase/flows/progress/done", Bin::Showcase, &["--page", "progress", "--motion", "paused", "--frame", "200"], BOOT),
    // `r` under paused motion: the pipeline has started (log line, running
    // chrome, Cancel enabled) but no task tick ever fires — the just-started
    // state the legacy `f_taskrunner_running` shot could not pin.
    Root::new("showcase/flows/taskrunner/running", Bin::Showcase, &["--page", "taskrunner", "--motion", "paused"], BOOT).sends(&["r"]),
    // 400 boot lines + 800 ticks = 1200 log lines, mid-stream, follow-tail on —
    // replaces the ~2.5 min boot wait for a mid-stream frame.
    Root::new("showcase/flows/scrolling/mid", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "800"], BOOT),
    // 60 ticks into the staged run: Resolve + Pull done, Build container
    // running (24 of 40), the rail meta and layer lines all pinned.
    Root::new("showcase/flows/terminal/mid", Bin::Showcase, &["--page", "terminal", "--motion", "paused", "--frame", "60"], BOOT),
    // ---------------------------------------------------------- pointer flows --
    Root::new("showcase/flows/diff/drag-selected", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "enter", "wait:● Review"]).live(interact::diff_drag_select),
    // S9: right click on a session row opens its context menu.
    Root::new("showcase/flows/chrome/context", Bin::Showcase, &["--page", "chrome"], BOOT).live(interact::chrome_context),
    // S12: the inspector registers no scroll region, so the capturable form is
    // the inspector open while a wheel scroll runs under it (the repainted
    // inspector shows the pointer's `mouse` row from the same interaction).
    Root::new("showcase/flows/inspector/scrolled", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["i"]).live(interact::wheel_python),
    // ------------------------------------------------------ wheel scroll-fade --
    Root::new("showcase/fade/lists/wheel-fade", Bin::Showcase, &["--page", "lists"], BOOT).live(interact::wheel_python),
    Root::new("showcase/fade/trees/wheel-fade", Bin::Showcase, &["--page", "trees"], BOOT).live(interact::trees_wheel),
    Root::new("showcase/fade/datagrid/wheel", Bin::Showcase, &["--page", "datagrid"], BOOT).live(interact::datagrid_wheel),
    // The code viewport under the wheel: 22 of 26 lines at boot.
    Root::new("showcase/fade/editor/wheel", Bin::Showcase, &["--page", "codeeditor"], BOOT).live(interact::editor_wheel),
    // The 28-line task description scrolls inside its 8-row viewport.
    Root::new("showcase/fade/textarea/wheel", Bin::Showcase, &["--page", "textareas"], BOOT).live(interact::textarea_wheel),
    // Review mode, then the wheel over the Old pane scrolls the 5-hunk diff.
    Root::new("showcase/fade/diff/wheel", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "enter", "wait:● Review"]).live(interact::diff_wheel),
    // `--frame 1600` seeks to the boot stream's end state (2000 lines), then
    // the wheel moves the follow-tail log off the tail.
    Root::new("showcase/fade/scrolling/wheel-fade", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).live(interact::scrolling_wheel_up),
    // Paged scroll through the wrapped prose; the frame seek keeps boot instant.
    Root::new("showcase/fade/scroll/page", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["tab", "pagedown"]).live(interact::none),
    // Minimum-size reflow: the nav list overflows its 12-row viewport.
    Root::new("showcase/fade/sidebars/wheel", Bin::Showcase, &["--page", "sidebars"], BOOT).live(interact::sidebars_wheel),
    // Frame-seeked mid-run terminal (60 ticks), wheel up into the scrollback.
    Root::new("showcase/fade/terminal/scrollback", Bin::Showcase, &["--page", "terminal", "--motion", "paused", "--frame", "60"], BOOT).live(interact::terminal_scrollback),
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[
    Root::new("showcase/resize/overview_shrunk", Bin::Showcase, &["--page", "overview"], BOOT).timeout(15_000).resize((120, 40), interact::showcase_overview_resize),
    Root::new("showcase/resize/overview_grown", Bin::Showcase, &["--page", "overview"], BOOT).timeout(15_000).resize((80, 24), interact::showcase_overview_resize),
];
