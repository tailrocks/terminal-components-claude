//! jackin-preview captures (18 + 40): 6 non-audit scenarios (16 statics)
//! under `jackin/scenarios/`, intro phase frames under `jackin/intro/`, and
//! the interactive captures under `jackin/{manager,editor,cockpit,capsule,
//! accounts,usage,settings,intro}/` (J1-J23 plus the hole groups: editor and
//! settings non-General tabs, accounts detail/drawer, usage overview,
//! hard-cases launch states, prelude 80x24). The two audit fixtures
//! (accounts-mixed, capsule-multi) keep statics only in the audit rows
//! below (`jackin/audit/`, 5x5 matrix — the dedupe rule).
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
//!   job (editor.rs:450) that fires under paused motion only on an
//!   explicit `advance:<ms>` step (V12; `overlays/editor_save_failed` uses
//!   it); under reduced the post-save Manager frame is
//!   wall-clock-nondeterministic (drawer `ago()` labels). This row's
//!   boundary is the preview dialog.
//!   Tab walks General → Name → Choose… → Keep awake: arrows no longer
//!   leave the Name field, which takes focus already editing (1da20cc01).
//! - `cockpit/{info,debug}`: the cockpit has no back/detach keys — `b` opens
//!   the build log (empty at frame 40), `t` toggles the debug chip (c3515f652), `i` the
//!   container info dialog (cockpit.rs:604-640).

use super::{Bin, Root, Variant, interact};
use crate::support::Color;

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

const HARD: &[&str] = &[
    "--scenario",
    "hard-cases",
    "--motion",
    "paused",
    "--frame",
    "40",
];

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[
    // -------------------------------------------------------------- scenarios --
    Root::new("jackin/scenarios/first-use", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(120, 40, Color::None, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(120, 40, Color::NoColorEnv, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(72, 20, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯"),
    ]),
    Root::new("jackin/scenarios/returning", Bin::Jackin, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], "jackin❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], "jackin❯"),
    ]),
    Root::new("jackin/scenarios/launch-running", Bin::Jackin, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], "jackin❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], "jackin❯"),
    ]),
    // launch-failure seeks past the Network stage: durations [14,18,26,30,8,92,
    // 22,20,…] ticks (sim/launch.rs:188) put the FailNetwork failure at run tick
    // 231 (sim/launch.rs:294); frame 40 was still the Credentials stage and
    // rendered byte-identical to launch-running (false claim). Frame 240 shows
    // the frozen failure with the "Launch failed" dialog open.
    Root::new("jackin/scenarios/launch-failure", Bin::Jackin, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], "Launch failed").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], "Launch failed"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], "Launch failed"),
    ]),
    Root::new("jackin/scenarios/hard-cases", Bin::Jackin, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "jackin❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "jackin❯").sends(&["wait:of 16"]),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "jackin❯"),
        Variant::new(120, 40, Color::None, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "jackin❯"),
    ]),
    Root::new("jackin/scenarios/outro-last", Bin::Jackin, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], "Enter Skip").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], "Enter Skip"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], "Enter Skip"),
    ]),
    // ------------------------------------------------------------------ intro --
    Root::new("jackin/intro/f300", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "300"], "Enter Skip"),
    Root::new("jackin/intro/f400", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "400"], "jackin❯"),
    // ---------------------------------------------------------------- manager --
    // The boot tree is CurrentDir + 4 collapsed workspaces; the drawer already
    // lists the two daemon-backed instances of payments-platform, so expansion
    // evidence is the third (superseded) instance row.
    Root::new("jackin/manager/tree_expanded", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space"]),
    Root::new("jackin/manager/detail_drawer", Bin::Jackin, RETURNING, "jackin❯").sends(&["tab"]),
    Root::new("jackin/manager/inspect", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "i", "wait:Container 7f3a"]),
    // ----------------------------------------------------------------- editor --
    Root::new("jackin/editor/general", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General"]),
    // TABS → NAME → WORKDIR → KEEP_AWAKE: three downs, then space toggles the
    // checkbox (payments-platform ships keep_awake=true), Ctrl+S opens the save
    // preview (the terminal state reachable deterministically — see header).
    Root::new("jackin/editor/save_preview", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "tab", "tab", "tab", "space", "ctrl-s", "wait:Save workspace"]),
    // ---------------------------------------------------------------- cockpit --
    Root::new("jackin/cockpit/debug", Bin::Jackin, LAUNCH, "jackin❯").sends(&["t", "wait:run-2026"]),
    // ---------------------------------------------------------------- capsule --
    Root::new("jackin/capsule/split_vertical", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "\"", "wait:Split ↓ Below"]),
    Root::new("jackin/capsule/zoom", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "z", "wait:Zoomed · z restores the layout"]),
    // --------------------------------------------------- accounts/usage/settings --
    // accounts-mixed boots directly into the Accounts route, so `a` / `/` act
    // immediately (no `c` prefix needed). The filter input starts non-editing:
    // Enter begins the edit, the second Enter commits and applies. The applied
    // filter renders in the tree title, truncated to the panel (`Accounts · filt…`).
    Root::new("jackin/accounts/add_form", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["a", "wait:New account"]),
    // j_accounts_form_ref: empty required Display name. An edit that only
    // passes the field does not judge it (ValidateOn::Change, f7e8b7538);
    // Save does: Shift+Tab from the name wraps to Save, Enter, and the
    // form marks the name "Required" and focuses it.
    Root::new("jackin/accounts/add_form_required", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["a", "wait:New account", "backtab", "enter", "wait:Required"]),
    Root::new("jackin/accounts/filter", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["/", "type:work", "enter", "wait:Accounts · filt"]),
    Root::new("jackin/usage/detail", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["u", "down", "enter", "wait:Back to list"]),
    Root::new("jackin/settings/route", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global"]),
    // ------------------------------------------------------------------- intro --
    // Paused, not reduced: under reduced the intro auto-finishes at tick 45
    // (~1.5 s wall), and a slow parallel runner can land the skip Enter after
    // that, opening the launch picker instead — observed as gate drift under
    // --test-threads 8. Under paused the guard is bypassed (app.rs:514) and the
    // two-step skip (phrases → warp → done, rain.rs:560-568) is tick-frozen:
    // Enter×2 skips deterministically to the (empty) Manager.
    Root::new("jackin/intro/skipped", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯").sends(&["enter", "enter", "wait:Current directory"]),
    // ----------------------------------------------------------- editor tabs --
    Root::new("jackin/editor/mounts", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "2", "enter", "sleep:300"]),
    // r toggles rw→ro, i cycles isolation worktree→clone on the first mount:
    // one changed row (`• 1 change`, row marked `•`), mount row hints.
    Root::new("jackin/editor/mounts_dirty", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "2", "enter", "r", "i", "sleep:300"]),
    Root::new("jackin/editor/env", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "4", "enter", "sleep:300"]),
    // The old "Auth" tab is the Accounts tab in this design (per-provider
    // account enable / prefer); there is no add-override form here — account
    // registration lives on the Accounts route (accounts/add_form).
    Root::new("jackin/editor/auth", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "5", "enter", "sleep:300"]),
    Root::new("jackin/editor/roles", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "sleep:300"]),
    // --------------------------------------------------------- settings tabs --
    // The old "Auth" settings tab is "Agents" here (per-agent auth mode).
    Root::new("jackin/settings/mounts", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:General", "2", "enter", "sleep:300"]),
    Root::new("jackin/settings/env", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN"]),
    Root::new("jackin/settings/agents", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "4", "enter", "wait:Agent runtime mode"]),
    Root::new("jackin/settings/trust", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "wait:github.com/chainargos/roles"]),
    // space flips chainargos trusted→untrusted, down+space flips acme-labs
    // untrusted→trusted: 2 changes, then the Save settings preview dialog.
    Root::new("jackin/settings/save_preview", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-s", "wait:Save settings"]),
    // ------------------------------------------- accounts detail / usage aggregate --
    // down×4 from Overview lands on Claude · Work (the exhausted 1Password
    // account — the richest inspector); Enter then focuses the inspector
    // (drawer_open), a distinct state from the plain selection.
    Root::new("jackin/accounts/detail", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work"]),
    Root::new("jackin/accounts/drawer", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "enter", "wait:Tab Actions"]),
    Root::new("jackin/usage/overview", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["u", "wait:Usage › Overview"]),
    // ------------------------------------------------- hard-cases interactive --
    // hard-cases boots to the Manager in degraded chrome (! instance index
    // unreadable ▲ daemon stale). The cred-locked cockpit of the old
    // j_hard_launch_picker: under paused the run advances only on an explicit
    // `advance:<ms>` step (V12, `overlays/cockpit_credentials`); under reduced
    // the credentials spinner rotates every 80 virtual ms so the 400 ms
    // content-settle never holds. Here: the picker and the Capsule attach.
    // locked attaches to jk-e0e0, the running data-pipeline instance whose
    // daemon never answers (fixtures.rs:1565-1572) — the 7f3a attach renders
    // byte-identical to the capsule-multi audit frame (the Capsule route does
    // not draw the host degraded badges), so it is not a distinct state.
    // With no daemon the PANES focus stop is unreachable, so the post-render
    // ensure_valid (focus.rs:94) moves focus to the menubar on the next tick —
    // a late repaint that races wait_stable (the audit-badge class): wait for
    // the focus-corrected `▎ File` menubar before settling.
    Root::new("jackin/manager/hard_launch_picker", Bin::Jackin, HARD, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]),
    Root::new("jackin/manager/hard_launch_locked", Bin::Jackin, HARD, "jackin❯").sends(&["down", "down", "down", "down", "down", "space", "down", "enter", "wait:▎ File"]),
    // ------------------------------------------------------------------ audit --
    Root::new("jackin/audit/capsule", Bin::Jackin, CAPSULE, "jackin❯"),
    // accounts-mixed repaints every 500 ms under paused motion; the scroll
    // badge needs its own wait where the list overflows (interact::accounts_badge).
    Root::new("jackin/audit/accounts", Bin::Jackin, ACCOUNTS, "jackin❯").live(interact::accounts_badge),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[
    // Pending coverage (plan §C2): hover and held press (paused fixtures).
    Root::new("jackin/elements/menubar/file-hover", Bin::Jackin, RETURNING, "jackin❯").hover("File").pending(),
    Root::new("jackin/elements/menubar/file-pressed", Bin::Jackin, RETURNING, "jackin❯").press("File").pending(),
    Root::new("jackin/elements/manager/row-hover", Bin::Jackin, RETURNING, "jackin❯").hover("infra-control-plane").pending(),
    Root::new("jackin/elements/manager/row-pressed", Bin::Jackin, RETURNING, "jackin❯").press("infra-control-plane").pending(),
    Root::new("jackin/elements/manager/selected-hover", Bin::Jackin, RETURNING, "jackin❯").hover("Current directory").pending(),
    Root::new("jackin/elements/manager/new-workspace-hover", Bin::Jackin, RETURNING, "jackin❯").hover("+ New workspace").pending(),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[
    // Open overlays captured by existing journeys (names unchanged), with
    // pending dismissal scenarios (`-dismissed-esc`, `-dismissed-outside`).
    Root::new("jackin/manager/menu_open", Bin::Jackin, RETURNING, "jackin❯").sends(&["f10", "wait:New workspace…"]).dismiss_pending(),
    Root::new("jackin/manager/help_overlay", Bin::Jackin, RETURNING, "jackin❯").sends(&["?", "wait:Keyboard shortcuts"]).dismiss_pending(),
    // The quit guard (8a8c20253 dropped `manager/quit_confirm`: a clean
    // Ctrl+Q quits at once). Unsaved edits ask Save / Discard / Cancel.
    Root::new("jackin/editor/quit_guard", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "tab", "tab", "tab", "space", "ctrl-q", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("jackin/settings/quit_guard", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-q", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("jackin/manager/launch_picker", Bin::Jackin, RETURNING, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]).dismiss_pending(),
    Root::new("jackin/cockpit/info", Bin::Jackin, LAUNCH, "jackin❯").sends(&["i", "wait:Debug info"]).dismiss_pending(),
    // `x` cancels the launch since c3515f652 (one meaning per letter; `c`
    // stays Accounts' letter). The dialog copy is unchanged.
    Root::new("jackin/cockpit/cancel_confirm",Bin::Jackin, LAUNCH, "jackin❯").sends(&["x", "wait:Cancel the launch?"]).dismiss_pending(),
    // tmux-style prefix: Ctrl+B arms it (frozen virtual clock never times it
    // out), the next key is the command.
    Root::new("jackin/capsule/menu", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "m", "wait:Change title…"]).dismiss_pending(),
    Root::new("jackin/capsule/palette", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "space", "wait:Command palette"]).dismiss_pending(),
    // Pending: the menu opened by pointer, with both dismissals.
    Root::new("jackin/overlays/file_menu_click", Bin::Jackin, RETURNING, "jackin❯").click("File").pending().dismiss_pending(),
    // V4: every overlay site in src/bin/jackin_preview has a row
    // (overlay_sites). Help › About jackin-preview.
    Root::new("jackin/overlays/about", Bin::Jackin, RETURNING, "jackin❯").sends(&["f10", "left", "wait:About jackin-preview", "up", "enter", "wait:Design system"]).pending().dismiss_pending(),
    // Manager: an instance's stop, purge and session picker; a
    // workspace's delete; the row context menu.
    Root::new("jackin/overlays/manager_stop", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "t", "wait:Stop instance 7f3a?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_purge", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "p", "wait:Purge instance 7f3a?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_session_picker", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "a", "wait:New session · choose Agent"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_delete", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "d", "wait:Delete workspace payments-platform?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_row_menu", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "shift-f10", "wait:Open on GitHub"]).pending().dismiss_pending(),
    // Cockpit: Ctrl+Q asks before quitting a launch; a failed launch
    // explains itself.
    Root::new("jackin/overlays/cockpit_quit", Bin::Jackin, LAUNCH, "jackin❯").sends(&["ctrl-q", "wait:Quitting force-stops the launch"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/cockpit_failure", Bin::Jackin, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], "Launch failed").pending().dismiss_pending(),
    // Capsule (Ctrl+B prefix): the provider picker after the agent, the
    // palette's split picker, usage, debug info, rename, close, quit and
    // its discard gate, and the View menu's GitHub context and change
    // inspector.
    Root::new("jackin/overlays/capsule_provider", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "c", "wait:New tab", "enter", "wait:Account for Claude Code"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_split_picker", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "space", "wait:Command palette", "type:Split pane", "enter", "wait:→ Right"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_usage", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "u", "wait:read-only projection"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_info", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "i", "wait:Debug info"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_rename", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", ",", "wait:Change tab title"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_close", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "x", "wait:Close pane?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_quit", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-q", "wait:Unsaved work — quit?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_discard", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-q", "wait:quit?", "down", "down", "down", "enter", "wait:Quit and discard changes?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_github", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "f10", "right", "right", "wait:GitHub context", "up", "up", "enter", "wait:Open PR…"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_inspect", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "f10", "right", "right", "wait:Inspect changes", "up", "enter", "wait:Inspect changes ·"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/capsule_about", Bin::Jackin, CAPSULE, "jackin❯").live(interact::capsule_brand_about).pending().dismiss_pending(),
    // Accounts: the filter prompt, remove, the row menu, and the add
    // form's 1Password chooser.
    Root::new("jackin/overlays/accounts_filter", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["/", "wait:Filter accounts"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/accounts_remove", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work", "d", "wait:Remove account Work?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/accounts_row_menu", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work", "shift-f10", "wait:Show usage"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/accounts_op_picker", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["a", "wait:New account", "tab", "tab", "tab", "tab", "enter", "wait:Choose 1Password account"]).pending().dismiss_pending(),
    // Global settings mounts and environment (config.rs).
    Root::new("jackin/overlays/config_mount_form", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "a", "wait:New mount"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_env_form", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "a", "wait:New global environment key"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_scope_picker", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "down", "s", "wait:Scope for"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_env_remove", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "down", "d", "wait:Delete environment variable EDITOR?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_env_op", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "down", "p", "wait:Choose 1Password account"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_role_picker", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "end", "enter", "wait:Add role override"]).pending().dismiss_pending(),
    // The mount form's host-directory chooser stacks the file browser.
    Root::new("jackin/overlays/config_mount_browse", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "a", "wait:New mount", "tab", "enter", "wait:Choose the host directory"]).pending().dismiss_pending(),
    // Editor: the Roles filter, Load role picker and trust gate (data-eng
    // has a load error), the Accounts filter, and the working-directory
    // picker (General: Name, then Choose…).
    Root::new("jackin/overlays/editor_roles_filter", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "/", "wait:Filter roles"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_load_role", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "a", "wait:Load role"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_trust", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "down", "down", "down", "down", "enter", "wait:Trust this role source?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_accounts_filter", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "5", "enter", "sleep:300", "/", "wait:Filter accounts"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_workdir", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "tab", "tab", "enter", "wait:Type Filter"]).pending().dismiss_pending(),
    // Layers that open only from a job on virtual time (V12): the paused
    // capture runs the world clock for one explicit step (`advance:<ms>`,
    // `junie_tui::clock::ClockStep`), then the frame is still. hard-cases
    // fails the first save, fails the trusted `writer` role's load and
    // keeps 1Password locked (Codex · Primary launches into the
    // credential error).
    Root::new("jackin/overlays/editor_save_failed", Bin::Jackin, HARD, "jackin❯").sends(&["e", "wait:General", "]", "enter", "r", "ctrl-s", "wait:Save workspace", "right", "enter", "advance:1000", "wait:Save failed"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_role_load_error", Bin::Jackin, HARD, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "a", "wait:Load role", "type:writer", "enter", "advance:1200", "wait:failed to load"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/settings_save_failed", Bin::Jackin, HARD, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-s", "wait:Save settings", "right", "enter", "advance:1000", "wait:Settings error"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/cockpit_credentials", Bin::Jackin, HARD, "jackin❯").sends(&["enter", "wait:Launch · choose Agent", "down", "enter", "advance:2000", "wait:Credential source unavailable"]).pending().dismiss_pending(),
    // Surfaces whose state is an open overlay, moved here with both
    // dismissals (V4; names and approvals unchanged): the new-workspace
    // prelude's first step and the capsule's new-tab agent picker.
    Root::new("jackin/manager/new_workspace_prelude", Bin::Jackin, FIRST_400, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]).variants(&[
        Variant::new(120, 40, Color::Truecolor, FIRST_400, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
        Variant::new(80, 24, Color::Truecolor, FIRST_400, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
    ]).dismiss_pending(),
    Root::new("jackin/capsule/new_tab", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "c", "wait:New tab"]).dismiss_pending(),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[
    // Pending: jackin had no resize rows (plan §C2).
    Root::new("jackin/resize/manager_from160x50", Bin::Jackin, RETURNING, "jackin❯").timeout(15_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("jackin/resize/manager_from72x20", Bin::Jackin, RETURNING, "jackin❯").timeout(15_000).resize((72, 20), interact::resize_case).pending(),
    Root::new("jackin/resize/accounts_from160x50", Bin::Jackin, ACCOUNTS, "jackin❯").timeout(15_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("jackin/resize/accounts_from72x20", Bin::Jackin, ACCOUNTS, "jackin❯").timeout(15_000).resize((72, 20), interact::resize_case).pending(),
];
