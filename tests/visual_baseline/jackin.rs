//! jackin-preview captures (18 + 40): 6 non-audit scenarios (16 statics)
//! under `jackin/scenarios/`, intro phase frames under `jackin/intro/`, and
//! the interactive captures under `jackin/{manager,editor,cockpit,capsule,
//! accounts,usage,settings,intro}/` (J1-J23 plus the hole groups: editor and
//! settings non-General tabs, accounts detail/drawer, usage overview,
//! hard-cases launch states, prelude 80x24). The two audit fixtures
//! (accounts-mixed, capsule-multi) keep statics only in audit.rs
//! (`jackin/audit/`, 5x5 matrix — the dedupe rule).
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs); only the store names were regrouped. Do not
//! hand-tune: drift against the approved frames means the port or the app
//! changed.
//!
//! Interactive-group notes (spec §3.4, verified against src/bin/jackin_preview):
//! - Manager/editor captures boot `returning`, not `accounts-mixed`: the same
//!   populated world, but accounts-mixed starts on the Accounts route
//!   (app.rs:248) while returning joins straight into the Manager.
//! - `launch_picker`: Enter on a Manager workspace row opens the agent
//!   picker (manager.rs:361); there is no launch "prelude" route — the
//!   Prelude route is the new-workspace flow (`n`, covered by
//!   `new_workspace_prelude`).
//! - `editor/save_preview`: the save completes via a +900 ms virtual-time
//!   job (editor.rs:450) that never fires under paused motion; under reduced
//!   the post-save Manager frame is wall-clock-nondeterministic (drawer
//!   `ago()` labels). The deterministic boundary is the preview dialog.
//! - `cockpit/{info,debug}`: the cockpit has no back/detach keys — `b` opens
//!   the build log (empty at frame 40), `d` toggles the debug chip, `i` the
//!   container info dialog (cockpit.rs:604-640).

use crate::support::{Case, Color, JACKIN};

// -------------------------------------------------------------- scenarios --

crate::baseline_case!(jackin_scenarios_first_use_default_80x24_truecolor => Case::new("jackin/scenarios/first-use/80x24/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_scenarios_first_use_default_120x40_truecolor => Case::new("jackin/scenarios/first-use/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_scenarios_returning_default_80x24_truecolor => Case::new("jackin/scenarios/returning/80x24/truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_scenarios_returning_default_120x40_truecolor => Case::new("jackin/scenarios/returning/120x40/truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_scenarios_launch_running_default_80x24_truecolor => Case::new("jackin/scenarios/launch-running/80x24/truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_scenarios_launch_running_default_120x40_truecolor => Case::new("jackin/scenarios/launch-running/120x40/truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
// launch-failure seeks past the Network stage: durations [14,18,26,30,8,92,
// 22,20,…] ticks (sim/launch.rs:188) put the FailNetwork failure at run tick
// 231 (sim/launch.rs:294); frame 40 was still the Credentials stage and
// rendered byte-identical to launch-running (false claim). Frame 240 shows
// the frozen failure with the "Launch failed" dialog open.
crate::baseline_case!(jackin_scenarios_launch_failure_default_80x24_truecolor => Case::new("jackin/scenarios/launch-failure/80x24/truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], 80, 24, Color::Truecolor, "Launch failed"));
crate::baseline_case!(jackin_scenarios_launch_failure_default_120x40_truecolor => Case::new("jackin/scenarios/launch-failure/120x40/truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], 120, 40, Color::Truecolor, "Launch failed"));
crate::baseline_case!(jackin_scenarios_hard_cases_default_80x24_truecolor => Case::new("jackin/scenarios/hard-cases/80x24/truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯").sends(&["wait:of 16"]));
crate::baseline_case!(jackin_scenarios_hard_cases_default_120x40_truecolor => Case::new("jackin/scenarios/hard-cases/120x40/truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_scenarios_outro_last_default_80x24_truecolor => Case::new("jackin/scenarios/outro-last/80x24/truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_scenarios_outro_last_default_120x40_truecolor => Case::new("jackin/scenarios/outro-last/120x40/truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_scenarios_first_use_default_120x40_none => Case::new("jackin/scenarios/first-use/120x40/none", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"));
crate::baseline_case!(jackin_scenarios_hard_cases_default_120x40_none => Case::new("jackin/scenarios/hard-cases/120x40/none", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"));
crate::baseline_case!(jackin_scenarios_first_use_default_120x40_nocolor => Case::new("jackin/scenarios/first-use/120x40/nocolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::NoColorEnv, "jackin❯"));
crate::baseline_case!(jackin_scenarios_first_use_default_72x20_truecolor => Case::new("jackin/scenarios/first-use/72x20/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 72, 20, Color::Truecolor, "jackin❯"));

// ------------------------------------------------------------------ intro --

crate::baseline_case!(jackin_intro_f300_120x40_truecolor => Case::new("jackin/intro/f300/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "300"], 120, 40, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_intro_f400_120x40_truecolor => Case::new("jackin/intro/f400/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "400"], 120, 40, Color::Truecolor, "jackin❯"));

// ------------------------------------------------------------ interactive --
//
// J1-J23 (spec §3.4): live keyboard flows on paused fixture frames, one
// capture per target state. All at 120x40 truecolor; boot needle `jackin❯`.

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

// ---------------------------------------------------------------- manager --
// The boot tree is CurrentDir + 4 collapsed workspaces; the drawer already
// lists the two daemon-backed instances of payments-platform, so expansion
// evidence is the third (superseded) instance row.

crate::baseline_case!(jackin_manager_tree_expanded_120x40_truecolor => Case::new("jackin/manager/tree_expanded/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "space"]));
crate::baseline_case!(jackin_manager_detail_drawer_120x40_truecolor => Case::new("jackin/manager/detail_drawer/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["tab"]));
crate::baseline_case!(jackin_manager_menu_open_120x40_truecolor => Case::new("jackin/manager/menu_open/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["f10", "wait:New workspace…"]));
crate::baseline_case!(jackin_manager_help_overlay_120x40_truecolor => Case::new("jackin/manager/help_overlay/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["?", "wait:Keyboard shortcuts"]));
crate::baseline_case!(jackin_manager_quit_confirm_120x40_truecolor => Case::new("jackin/manager/quit_confirm/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-q", "wait:Exit jackin❯?"]));
crate::baseline_case!(jackin_manager_launch_picker_120x40_truecolor => Case::new("jackin/manager/launch_picker/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]));
crate::baseline_case!(jackin_manager_new_workspace_prelude_120x40_truecolor => Case::new("jackin/manager/new_workspace_prelude/120x40/truecolor", JACKIN, FIRST_400, 120, 40, Color::Truecolor, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]));
crate::baseline_case!(jackin_manager_inspect_120x40_truecolor => Case::new("jackin/manager/inspect/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "space", "down", "i", "wait:Container 7f3a"]));

// ----------------------------------------------------------------- editor --

crate::baseline_case!(jackin_editor_general_120x40_truecolor => Case::new("jackin/editor/general/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit"]));
// TABS → NAME → WORKDIR → KEEP_AWAKE: three downs, then space toggles the
// checkbox (payments-platform ships keep_awake=true), Ctrl+S opens the save
// preview (the terminal state reachable deterministically — see header).
crate::baseline_case!(jackin_editor_save_preview_120x40_truecolor => Case::new("jackin/editor/save_preview/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit", "down", "down", "down", "space", "ctrl-s", "wait:Save workspace"]));

// ---------------------------------------------------------------- cockpit --

crate::baseline_case!(jackin_cockpit_info_120x40_truecolor => Case::new("jackin/cockpit/info/120x40/truecolor", JACKIN, LAUNCH, 120, 40, Color::Truecolor, "jackin❯").sends(&["i", "wait:Debug info"]));
crate::baseline_case!(jackin_cockpit_cancel_confirm_120x40_truecolor => Case::new("jackin/cockpit/cancel_confirm/120x40/truecolor", JACKIN, LAUNCH, 120, 40, Color::Truecolor, "jackin❯").sends(&["c", "wait:Cancel the launch?"]));
crate::baseline_case!(jackin_cockpit_debug_120x40_truecolor => Case::new("jackin/cockpit/debug/120x40/truecolor", JACKIN, LAUNCH, 120, 40, Color::Truecolor, "jackin❯").sends(&["d", "wait:run-2026"]));

// ---------------------------------------------------------------- capsule --
// tmux-style prefix: Ctrl+B arms it (frozen virtual clock never times it
// out), the next key is the command.

crate::baseline_case!(jackin_capsule_menu_120x40_truecolor => Case::new("jackin/capsule/menu/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "m", "wait:Change title…"]));
crate::baseline_case!(jackin_capsule_new_tab_120x40_truecolor => Case::new("jackin/capsule/new_tab/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "c", "wait:New tab"]));
crate::baseline_case!(jackin_capsule_split_vertical_120x40_truecolor => Case::new("jackin/capsule/split_vertical/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "\"", "wait:Split ↓ Below"]));
crate::baseline_case!(jackin_capsule_zoom_120x40_truecolor => Case::new("jackin/capsule/zoom/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "z", "wait:Zoomed · z restores the layout"]));
crate::baseline_case!(jackin_capsule_palette_120x40_truecolor => Case::new("jackin/capsule/palette/120x40/truecolor", JACKIN, CAPSULE, 120, 40, Color::Truecolor, "jackin❯").sends(&["ctrl-b", "space", "wait:Command palette"]));

// --------------------------------------------------- accounts/usage/settings --
// accounts-mixed boots directly into the Accounts route, so `a` / `/` act
// immediately (no `c` prefix needed). The filter input starts non-editing:
// Enter begins the edit, the second Enter commits and applies. The applied
// filter renders in the tree title, truncated to the panel (`Accounts · filt…`).

crate::baseline_case!(jackin_accounts_add_form_120x40_truecolor => Case::new("jackin/accounts/add_form/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["a", "wait:New account"]));
// j_accounts_form_ref: empty required Display name — Enter starts edit,
// Enter commits and TextInput::validate paints "Required".
crate::baseline_case!(jackin_accounts_add_form_required_120x40_truecolor => Case::new("jackin/accounts/add_form_required/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["a", "wait:New account", "enter", "enter", "wait:Required"]));
crate::baseline_case!(jackin_accounts_filter_120x40_truecolor => Case::new("jackin/accounts/filter/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["/", "enter", "type:work", "enter", "wait:Accounts · filt"]));
crate::baseline_case!(jackin_usage_detail_120x40_truecolor => Case::new("jackin/usage/detail/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["u", "down", "enter", "wait:Back to list"]));
crate::baseline_case!(jackin_settings_route_120x40_truecolor => Case::new("jackin/settings/route/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global"]));

// ------------------------------------------------------------------- intro --
// Paused, not reduced: under reduced the intro auto-finishes at tick 45
// (~1.5 s wall), and a slow parallel runner can land the skip Enter after
// that, opening the launch picker instead — observed as gate drift under
// --test-threads 8. Under paused the guard is bypassed (app.rs:514) and the
// two-step skip (phrases → warp → done, rain.rs:560-568) is tick-frozen:
// Enter×2 skips deterministically to the (empty) Manager.

crate::baseline_case!(jackin_intro_skipped_120x40_truecolor => Case::new("jackin/intro/skipped/120x40/truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯").sends(&["enter", "enter", "wait:Current directory"]));

// ------------------------------------------------------------ hole groups --
// Follow-up coverage (old-state counterparts the first pass missed).
// Editor/settings tab keys: digits 1–5 jump while TABS is focused, Enter
// moves into the body (editor.rs:1634, settings.rs:848).

const HARD: &[&str] = &[
    "--scenario",
    "hard-cases",
    "--motion",
    "paused",
    "--frame",
    "40",
];

// ----------------------------------------------------------- editor tabs --

crate::baseline_case!(jackin_editor_mounts_120x40_truecolor => Case::new("jackin/editor/mounts/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit", "2", "enter", "wait:/workspace/libs"]));
// r toggles rw→ro, i cycles isolation worktree→clone on the first mount:
// one changed row (`• 1 change`, row marked `•`), mount row hints.
crate::baseline_case!(jackin_editor_mounts_dirty_120x40_truecolor => Case::new("jackin/editor/mounts_dirty/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit", "2", "enter", "r", "i", "wait:1 change"]));
crate::baseline_case!(jackin_editor_env_120x40_truecolor => Case::new("jackin/editor/env/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit", "4", "enter", "wait:DATABASE_URL"]));
// The old "Auth" tab is the Accounts tab in this design (per-provider
// account enable / prefer); there is no add-override form here — account
// registration lives on the Accounts route (accounts/add_form).
crate::baseline_case!(jackin_editor_auth_120x40_truecolor => Case::new("jackin/editor/auth/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit", "5", "enter", "wait:Active accounts"]));
crate::baseline_case!(jackin_editor_roles_120x40_truecolor => Case::new("jackin/editor/roles/120x40/truecolor", JACKIN, RETURNING, 120, 40, Color::Truecolor, "jackin❯").sends(&["e", "wait:Workspaces › payments-platform › edit", "3", "enter", "wait:Allowed roles"]));

// --------------------------------------------------------- settings tabs --
// The old "Auth" settings tab is "Agents" here (per-agent auth mode).

crate::baseline_case!(jackin_settings_mounts_120x40_truecolor => Case::new("jackin/settings/mounts/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "wait:cargo-registry"]));
crate::baseline_case!(jackin_settings_env_120x40_truecolor => Case::new("jackin/settings/env/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN"]));
crate::baseline_case!(jackin_settings_agents_120x40_truecolor => Case::new("jackin/settings/agents/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "4", "enter", "wait:Agent runtime mode"]));
crate::baseline_case!(jackin_settings_trust_120x40_truecolor => Case::new("jackin/settings/trust/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "wait:github.com/chainargos/roles"]));
// space flips chainargos trusted→untrusted, down+space flips acme-labs
// untrusted→trusted: 2 changes, then the Save settings preview dialog.
crate::baseline_case!(jackin_settings_save_preview_120x40_truecolor => Case::new("jackin/settings/save_preview/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-s", "wait:Save settings"]));

// ------------------------------------------- accounts detail / usage aggregate --
// down×4 from Overview lands on Claude · Work (the exhausted 1Password
// account — the richest inspector); Enter then focuses the inspector
// (drawer_open), a distinct state from the plain selection.

crate::baseline_case!(jackin_accounts_detail_120x40_truecolor => Case::new("jackin/accounts/detail/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work"]));
crate::baseline_case!(jackin_accounts_drawer_120x40_truecolor => Case::new("jackin/accounts/drawer/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "down", "down", "down", "enter", "wait:Tab Actions"]));
crate::baseline_case!(jackin_usage_overview_120x40_truecolor => Case::new("jackin/usage/overview/120x40/truecolor", JACKIN, ACCOUNTS, 120, 40, Color::Truecolor, "jackin❯").sends(&["u", "wait:Usage › Overview"]));

// ------------------------------------------------- hard-cases interactive --
// hard-cases boots to the Manager in degraded chrome (! instance index
// unreadable ▲ daemon stale). The cred-locked cockpit of the old
// j_hard_launch_picker is not deterministically capturable: under paused the
// run never advances (cockpit.rs:551), under reduced the credentials spinner
// rotates every 80 virtual ms so the 400 ms content-settle never holds.
// The reachable deterministic states are the picker and the Capsule attach.
// locked attaches to jk-e0e0, the running data-pipeline instance whose
// daemon never answers (fixtures.rs:1565-1572) — the 7f3a attach renders
// byte-identical to the capsule-multi audit frame (the Capsule route does
// not draw the host degraded badges), so it is not a distinct state.
// With no daemon the PANES focus stop is unreachable, so the post-render
// ensure_valid (focus.rs:94) moves focus to the menubar on the next tick —
// a late repaint that races wait_stable (the audit-badge class): wait for
// the focus-corrected `▎ File` menubar before settling.

crate::baseline_case!(jackin_manager_hard_launch_picker_120x40_truecolor => Case::new("jackin/manager/hard_launch_picker/120x40/truecolor", JACKIN, HARD, 120, 40, Color::Truecolor, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]));
crate::baseline_case!(jackin_manager_hard_launch_locked_120x40_truecolor => Case::new("jackin/manager/hard_launch_locked/120x40/truecolor", JACKIN, HARD, 120, 40, Color::Truecolor, "jackin❯").sends(&["down", "down", "down", "down", "down", "space", "down", "enter", "wait:▎ File"]));

// -------------------------------------------------------------- prelude 80 --

crate::baseline_case!(jackin_manager_new_workspace_prelude_80x24_truecolor => Case::new("jackin/manager/new_workspace_prelude/80x24/truecolor", JACKIN, FIRST_400, 80, 24, Color::Truecolor, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]));
