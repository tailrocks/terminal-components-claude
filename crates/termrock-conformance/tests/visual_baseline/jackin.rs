#![doc = r" jackin-preview captures (18 + 40): 6 non-audit scenarios (16 statics)"]
#![doc = r" under `jackin/scenarios/`, intro phase frames under `jackin/intro/`, and"]
#![doc = r" the interactive captures under `jackin/{manager,editor,cockpit,capsule,"]
#![doc = r" accounts,usage,settings,intro}/` (J1-J23 plus the hole groups: editor and"]
#![doc = r" settings non-General tabs, accounts detail/drawer, usage overview,"]
#![doc = r" hard-cases launch states, prelude 80x24). The two audit fixtures"]
#![doc = r" (accounts-mixed, capsule-multi) keep statics only in audit.rs"]
#![doc = r" (`jackin/audit/`, 5x5 matrix — the dedupe rule)."]
#![doc = r""]
#![doc = r" Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,"]
#![doc = r" sends, CAP_TIMEOUTs); only the store names were regrouped. Do not"]
#![doc = r" hand-tune: drift against the approved frames means the port or the app"]
#![doc = r" changed."]
#![doc = r""]
#![doc = r" Interactive-group notes (spec §3.4, verified against src/bin/jackin_preview):"]
#![doc = r" - Manager/editor captures boot `returning`, not `accounts-mixed`: the same"]
#![doc = r"   populated world, but accounts-mixed starts on the Accounts route"]
#![doc = r"   (app.rs:248) while returning joins straight into the Manager."]
#![doc = r" - `launch_picker`: Enter on a Manager workspace row opens the agent"]
#![doc = r#"   picker (manager.rs:361); there is no launch "prelude" route — the"#]
#![doc = r"   Prelude route is the new-workspace flow (`n`, covered by"]
#![doc = r"   `new_workspace_prelude`)."]
#![doc = r" - `editor/save_preview`: the save completes via a +900 ms virtual-time"]
#![doc = r"   job (editor.rs:450) that never fires under paused motion; under reduced"]
#![doc = r"   the post-save Manager frame is wall-clock-nondeterministic (drawer"]
#![doc = r"   `ago()` labels). The deterministic boundary is the preview dialog."]
#![doc = r" - `cockpit/{info,debug}`: the cockpit has no back/detach keys — `b` opens"]
#![doc = r"   the build log (empty at frame 40), `d` toggles the debug chip, `i` the"]
#![doc = r"   container info dialog (cockpit.rs:604-640)."]

use crate::support::{Case, Color, JACKIN};

crate::baseline_case_with_variants!(
    jackin_scenarios_first_use_default_120x40_truecolor => Case::new("jackin/scenarios/first-use/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
    [
        Case::new("jackin/scenarios/first-use/80x24/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"),
        Case::new("jackin/scenarios/first-use/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
        Case::new("jackin/scenarios/first-use/120x40/none", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"),
        Case::new("jackin/scenarios/first-use/120x40/nocolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::NoColorEnv, "jackin❯"),
        Case::new("jackin/scenarios/first-use/72x20/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 72, 20, Color::Truecolor, "jackin❯"),
    ],
);
crate::baseline_case_with_variants!(
    jackin_scenarios_returning_default_120x40_truecolor => Case::new("jackin/scenarios/returning/120x40/truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
    [
        Case::new("jackin/scenarios/returning/80x24/truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"),
        Case::new("jackin/scenarios/returning/120x40/truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
    ],
);
crate::baseline_case_with_variants!(
    jackin_scenarios_launch_running_default_120x40_truecolor => Case::new("jackin/scenarios/launch-running/120x40/truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
    [
        Case::new("jackin/scenarios/launch-running/80x24/truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"),
        Case::new("jackin/scenarios/launch-running/120x40/truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
    ],
);

crate::baseline_case_with_variants!(
    jackin_scenarios_launch_failure_default_120x40_truecolor => Case::new("jackin/scenarios/launch-failure/120x40/truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], 120, 40, Color::Truecolor, "Launch failed"),
    [
        Case::new("jackin/scenarios/launch-failure/80x24/truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], 80, 24, Color::Truecolor, "Launch failed"),
        Case::new("jackin/scenarios/launch-failure/120x40/truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], 120, 40, Color::Truecolor, "Launch failed"),
    ],
);
crate::baseline_case_with_variants!(
    jackin_scenarios_hard_cases_default_120x40_truecolor => Case::new("jackin/scenarios/hard-cases/120x40/truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
    [
        Case::new("jackin/scenarios/hard-cases/80x24/truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯").sends(&["wait:of 16"]),
        Case::new("jackin/scenarios/hard-cases/120x40/truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"),
        Case::new("jackin/scenarios/hard-cases/120x40/none", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"),
    ],
);
crate::baseline_case_with_variants!(
    jackin_scenarios_outro_last_default_120x40_truecolor => Case::new("jackin/scenarios/outro-last/120x40/truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "Enter Skip"),
    [
        Case::new("jackin/scenarios/outro-last/80x24/truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "Enter Skip"),
        Case::new("jackin/scenarios/outro-last/120x40/truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "Enter Skip"),
    ],
);

crate::baseline_case!(jackin_intro_f300_120x40_truecolor => Case::new("jackin/intro/f300/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "300"], 120, 40, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_intro_f400_120x40_truecolor => Case::new("jackin/intro/f400/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "400"], 120, 40, Color::Truecolor, "jackin❯"));

const RETURNING: &[&str] = &[
    "--scenario",
    "returning",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const ACCOUNTS: &[&str] = &[
    "--scenario",
    "accounts-mixed",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const LAUNCH: &[&str] = &[
    "--scenario",
    "launch-running",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const CAPSULE: &[&str] = &[
    "--scenario",
    "capsule-multi",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRST_400: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "400",
];

crate::baseline_case!(jackin_manager_tree_expanded_120x40_truecolor => Case::new("jackin/manager/tree_expanded/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "space"]));
crate::baseline_case!(jackin_manager_detail_drawer_120x40_truecolor => Case::new("jackin/manager/detail_drawer/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["tab"]));
crate::baseline_case!(jackin_manager_menu_open_120x40_truecolor => Case::new("jackin/manager/menu_open/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["f10", "wait:New workspace…"]));
crate::baseline_case!(jackin_manager_help_overlay_120x40_truecolor => Case::new("jackin/manager/help_overlay/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["?", "wait:Keyboard shortcuts"]));
crate::baseline_case!(jackin_manager_quit_confirm_120x40_truecolor => Case::new("jackin/manager/quit_confirm/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-q", "wait:Exit jackin❯?"]));
crate::baseline_case!(jackin_manager_launch_picker_120x40_truecolor => Case::new("jackin/manager/launch_picker/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]));
crate::baseline_case_with_variants!(
    jackin_manager_new_workspace_prelude_120x40_truecolor => Case::new("jackin/manager/new_workspace_prelude/120x40/truecolor", JACKIN, FIRST_400, 120, 40, Color::Truecolor, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
    [
        Case::new("jackin/manager/new_workspace_prelude/120x40/truecolor", JACKIN, FIRST_400, 120, 40, Color::Truecolor, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
        Case::new("jackin/manager/new_workspace_prelude/80x24/truecolor", JACKIN, FIRST_400, 80, 24, Color::Truecolor, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
    ],
);
crate::baseline_case!(jackin_manager_inspect_120x40_truecolor => Case::new("jackin/manager/inspect/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "space", "down", "i", "wait:Container 7f3a"]));

crate::baseline_case!(jackin_editor_general_120x40_truecolor => Case::new("jackin/editor/general/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General"]));

crate::baseline_case!(jackin_editor_save_preview_120x40_truecolor => Case::new("jackin/editor/save_preview/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General", "down", "down", "down", "space", "ctrl-s", "wait:Save workspace"]));

crate::baseline_case!(jackin_cockpit_info_120x40_truecolor => Case::new("jackin/cockpit/info/120x40/truecolor", JACKIN, LAUNCH, 120, 40, Color::Truecolor, "jackin❯").sends(&["i", "wait:Debug info"]));
crate::baseline_case!(jackin_cockpit_cancel_confirm_120x40_truecolor => Case::new("jackin/cockpit/cancel_confirm/120x40/truecolor", JACKIN, LAUNCH, 120, 40, Color::Truecolor, "jackin❯").sends(&["c", "wait:Cancel the launch?"]));
crate::baseline_case!(jackin_cockpit_debug_120x40_truecolor => Case::new("jackin/cockpit/debug/120x40/truecolor", JACKIN, LAUNCH, 120, 40, Color::Truecolor, "jackin❯").sends(&["d", "wait:run-2026"]));

crate::baseline_case!(jackin_capsule_menu_120x40_truecolor => Case::new("jackin/capsule/menu/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "m", "wait:Change title…"]));
crate::baseline_case!(jackin_capsule_new_tab_120x40_truecolor => Case::new("jackin/capsule/new_tab/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "c", "wait:New tab"]));
crate::baseline_case!(jackin_capsule_split_vertical_120x40_truecolor => Case::new("jackin/capsule/split_vertical/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "\"", "wait:Split ↓ Below"]));
crate::baseline_case!(jackin_capsule_zoom_120x40_truecolor => Case::new("jackin/capsule/zoom/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "z", "wait:Zoomed · z restores the layout"]));
crate::baseline_case!(jackin_capsule_palette_120x40_truecolor => Case::new("jackin/capsule/palette/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "space", "wait:Command palette"]));

crate::baseline_case!(jackin_accounts_add_form_120x40_truecolor => Case::new("jackin/accounts/add_form/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["a", "wait:New account"]));

crate::baseline_case!(jackin_accounts_add_form_required_120x40_truecolor => Case::new("jackin/accounts/add_form_required/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["a", "wait:New account", "enter", "enter", "wait:Required"]));
crate::baseline_case!(jackin_accounts_filter_120x40_truecolor => Case::new("jackin/accounts/filter/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["/", "enter", "type:work", "enter", "wait:Accounts · filt"]));
crate::baseline_case!(jackin_usage_detail_120x40_truecolor => Case::new("jackin/usage/detail/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["u", "down", "enter", "wait:Back to list"]));
crate::baseline_case!(jackin_settings_route_120x40_truecolor => Case::new("jackin/settings/route/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global"]));

crate::baseline_case!(jackin_intro_skipped_120x40_truecolor => Case::new("jackin/intro/skipped/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯").sends(&["enter", "enter", "wait:Current directory"]));

const HARD: &[&str] = &[
    "--scenario",
    "hard-cases",
    "--motion",
    "paused",
    "--frame",
    "40",
];

crate::baseline_case!(jackin_editor_mounts_120x40_truecolor => Case::new("jackin/editor/mounts/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General", "2", "enter", "sleep:300"]));

crate::baseline_case!(jackin_editor_mounts_dirty_120x40_truecolor => Case::new("jackin/editor/mounts_dirty/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General", "2", "enter", "r", "i", "sleep:300"]));
crate::baseline_case!(jackin_editor_env_120x40_truecolor => Case::new("jackin/editor/env/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General", "4", "enter", "sleep:300"]));

crate::baseline_case!(jackin_editor_auth_120x40_truecolor => Case::new("jackin/editor/auth/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General", "5", "enter", "sleep:300"]));
crate::baseline_case!(jackin_editor_roles_120x40_truecolor => Case::new("jackin/editor/roles/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:General", "3", "enter", "sleep:300"]));

crate::baseline_case!(jackin_settings_mounts_120x40_truecolor => Case::new("jackin/settings/mounts/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:General", "2", "enter", "sleep:300"]));
crate::baseline_case!(jackin_settings_env_120x40_truecolor => Case::new("jackin/settings/env/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN"]));
crate::baseline_case!(jackin_settings_agents_120x40_truecolor => Case::new("jackin/settings/agents/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "4", "enter", "wait:Agent runtime mode"]));
crate::baseline_case!(jackin_settings_trust_120x40_truecolor => Case::new("jackin/settings/trust/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "wait:github.com/chainargos/roles"]));

crate::baseline_case!(jackin_settings_save_preview_120x40_truecolor => Case::new("jackin/settings/save_preview/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-s", "wait:Save settings"]));

crate::baseline_case!(jackin_accounts_detail_120x40_truecolor => Case::new("jackin/accounts/detail/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work"]));
crate::baseline_case!(jackin_accounts_drawer_120x40_truecolor => Case::new("jackin/accounts/drawer/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "down", "down", "down", "enter", "wait:Tab Actions"]));
crate::baseline_case!(jackin_usage_overview_120x40_truecolor => Case::new("jackin/usage/overview/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["u", "wait:Usage › Overview"]));

crate::baseline_case!(jackin_manager_hard_launch_picker_120x40_truecolor => Case::new("jackin/manager/hard_launch_picker/120x40/truecolor", JACKIN, HARD, 120, 40, Color::Truecolor, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]));
crate::baseline_case!(jackin_manager_hard_launch_locked_120x40_truecolor => Case::new("jackin/manager/hard_launch_locked/120x40/truecolor", JACKIN, HARD, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "down", "down", "down", "down", "space", "down", "enter", "wait:▎ File"]));
