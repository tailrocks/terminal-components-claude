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

    Root::new("jackin/intro/f300", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "300"], "Enter Skip"),
    Root::new("jackin/intro/f400", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "400"], "jackin❯"),




    Root::new("jackin/manager/tree_expanded", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space"]),
    Root::new("jackin/manager/detail_drawer", Bin::Jackin, RETURNING, "jackin❯").sends(&["tab"]),
    Root::new("jackin/manager/inspect", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "i", "wait:Container 7f3a"]),

    Root::new("jackin/editor/general", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General"]),



    Root::new("jackin/editor/save_preview", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "tab", "tab", "tab", "space", "ctrl-s", "wait:Save workspace"]),

    Root::new("jackin/cockpit/debug", Bin::Jackin, LAUNCH, "jackin❯").sends(&["t", "wait:run-2026"]),

    Root::new("jackin/capsule/split_vertical", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "\"", "wait:Split ↓ Below"]),
    Root::new("jackin/capsule/zoom", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "z", "wait:Zoomed · z restores the layout"]),





    Root::new("jackin/accounts/add_form", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["a", "wait:New account"]),




    Root::new("jackin/accounts/add_form_required", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["a", "wait:New account", "backtab", "enter", "wait:Required"]),
    Root::new("jackin/accounts/filter", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["/", "type:work", "enter", "wait:Accounts · filt"]),
    Root::new("jackin/usage/detail", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["u", "down", "enter", "wait:Back to list"]),
    Root::new("jackin/settings/route", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global"]),







    Root::new("jackin/intro/skipped", Bin::Jackin, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "jackin❯").sends(&["enter", "enter", "wait:Current directory"]),

    Root::new("jackin/editor/mounts", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "2", "enter", "sleep:300"]),


    Root::new("jackin/editor/mounts_dirty", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "2", "enter", "r", "i", "sleep:300"]),
    Root::new("jackin/editor/env", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "4", "enter", "sleep:300"]),



    Root::new("jackin/editor/auth", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "5", "enter", "sleep:300"]),
    Root::new("jackin/editor/roles", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "sleep:300"]),


    Root::new("jackin/settings/mounts", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:General", "2", "enter", "sleep:300"]),
    Root::new("jackin/settings/env", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN"]),
    Root::new("jackin/settings/agents", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "4", "enter", "wait:Agent runtime mode"]),
    Root::new("jackin/settings/trust", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "wait:github.com/chainargos/roles"]),


    Root::new("jackin/settings/save_preview", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-s", "wait:Save settings"]),




    Root::new("jackin/accounts/detail", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work"]),
    Root::new("jackin/accounts/drawer", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "enter", "wait:Tab Actions"]),
    Root::new("jackin/usage/overview", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["u", "wait:Usage › Overview"]),















    Root::new("jackin/manager/hard_launch_picker", Bin::Jackin, HARD, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]),
    Root::new("jackin/manager/hard_launch_locked", Bin::Jackin, HARD, "jackin❯").sends(&["down", "down", "down", "down", "down", "space", "down", "enter", "wait:▎ File"]),

    Root::new("jackin/audit/capsule", Bin::Jackin, CAPSULE, "jackin❯"),


    Root::new("jackin/audit/accounts", Bin::Jackin, ACCOUNTS, "jackin❯").live(interact::accounts_badge),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[

    Root::new("jackin/elements/menubar/file-hover", Bin::Jackin, RETURNING, "jackin❯").hover("File").pending(),
    Root::new("jackin/elements/menubar/file-pressed", Bin::Jackin, RETURNING, "jackin❯").press("File").pending(),
    Root::new("jackin/elements/manager/row-hover", Bin::Jackin, RETURNING, "jackin❯").hover("infra-control-plane").pending(),
    Root::new("jackin/elements/manager/row-pressed", Bin::Jackin, RETURNING, "jackin❯").press("infra-control-plane").pending(),
    Root::new("jackin/elements/manager/selected-hover", Bin::Jackin, RETURNING, "jackin❯").hover("Current directory").pending(),
    Root::new("jackin/elements/manager/new-workspace-hover", Bin::Jackin, RETURNING, "jackin❯").hover("+ New workspace").pending(),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[


    Root::new("jackin/manager/menu_open", Bin::Jackin, RETURNING, "jackin❯").sends(&["f10", "wait:New workspace…"]).dismiss_pending(),
    Root::new("jackin/manager/help_overlay", Bin::Jackin, RETURNING, "jackin❯").sends(&["?", "wait:Keyboard shortcuts"]).dismiss_pending(),


    Root::new("jackin/editor/quit_guard", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "tab", "tab", "tab", "space", "ctrl-q", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("jackin/settings/quit_guard", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-q", "wait:Unsaved changes"]).pending().dismiss_pending(),
    Root::new("jackin/manager/launch_picker", Bin::Jackin, RETURNING, "jackin❯").sends(&["enter", "wait:Launch · choose Agent"]).dismiss_pending(),
    Root::new("jackin/cockpit/info", Bin::Jackin, LAUNCH, "jackin❯").sends(&["i", "wait:Debug info"]).dismiss_pending(),


    Root::new("jackin/cockpit/cancel_confirm",Bin::Jackin, LAUNCH, "jackin❯").sends(&["x", "wait:Cancel the launch?"]).dismiss_pending(),


    Root::new("jackin/capsule/menu", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "m", "wait:Change title…"]).dismiss_pending(),
    Root::new("jackin/capsule/palette", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "space", "wait:Command palette"]).dismiss_pending(),

    Root::new("jackin/overlays/file_menu_click", Bin::Jackin, RETURNING, "jackin❯").click("File").pending().dismiss_pending(),


    Root::new("jackin/overlays/about", Bin::Jackin, RETURNING, "jackin❯").sends(&["f10", "left", "wait:About jackin-preview", "up", "enter", "wait:Design system"]).pending().dismiss_pending(),


    Root::new("jackin/overlays/manager_stop", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "t", "wait:Stop instance 7f3a?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_purge", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "p", "wait:Purge instance 7f3a?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_session_picker", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "space", "down", "a", "wait:New session · choose Agent"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_delete", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "d", "wait:Delete workspace payments-platform?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/manager_row_menu", Bin::Jackin, RETURNING, "jackin❯").sends(&["down", "shift-f10", "wait:Open on GitHub"]).pending().dismiss_pending(),


    Root::new("jackin/overlays/cockpit_quit", Bin::Jackin, LAUNCH, "jackin❯").sends(&["ctrl-q", "wait:Quitting force-stops the launch"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/cockpit_failure", Bin::Jackin, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "240"], "Launch failed").pending().dismiss_pending(),




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


    Root::new("jackin/overlays/accounts_filter", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["/", "wait:Filter accounts"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/accounts_remove", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work", "d", "wait:Remove account Work?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/accounts_row_menu", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["down", "down", "down", "down", "wait:Accounts › Claude › Work", "shift-f10", "wait:Show usage"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/accounts_op_picker", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["a", "wait:New account", "tab", "tab", "tab", "tab", "enter", "wait:Choose 1Password account"]).pending().dismiss_pending(),

    Root::new("jackin/overlays/config_mount_form", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "a", "wait:New mount"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_env_form", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "a", "wait:New global environment key"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_scope_picker", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "down", "s", "wait:Scope for"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_env_remove", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "down", "d", "wait:Delete environment variable EDITOR?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_env_op", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "down", "p", "wait:Choose 1Password account"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/config_role_picker", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "3", "enter", "wait:GH_TOKEN", "end", "enter", "wait:Add role override"]).pending().dismiss_pending(),

    Root::new("jackin/overlays/config_mount_browse", Bin::Jackin, ACCOUNTS, "jackin❯").sends(&["s", "wait:Settings › global", "2", "enter", "a", "wait:New mount", "tab", "enter", "wait:Choose the host directory"]).pending().dismiss_pending(),



    Root::new("jackin/overlays/editor_roles_filter", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "/", "wait:Filter roles"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_load_role", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "a", "wait:Load role"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_trust", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "down", "down", "down", "down", "enter", "wait:Trust this role source?"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_accounts_filter", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "5", "enter", "sleep:300", "/", "wait:Filter accounts"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_workdir", Bin::Jackin, RETURNING, "jackin❯").sends(&["e", "wait:General", "tab", "tab", "enter", "wait:Type Filter"]).pending().dismiss_pending(),






    Root::new("jackin/overlays/editor_save_failed", Bin::Jackin, HARD, "jackin❯").sends(&["e", "wait:General", "]", "enter", "r", "ctrl-s", "wait:Save workspace", "right", "enter", "advance:1000", "wait:Save failed"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/editor_role_load_error", Bin::Jackin, HARD, "jackin❯").sends(&["e", "wait:General", "3", "enter", "wait:Allowed roles", "a", "wait:Load role", "type:writer", "enter", "advance:1200", "wait:failed to load"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/settings_save_failed", Bin::Jackin, HARD, "jackin❯").sends(&["s", "wait:Settings › global", "5", "enter", "space", "down", "space", "ctrl-s", "wait:Save settings", "right", "enter", "advance:1000", "wait:Settings error"]).pending().dismiss_pending(),
    Root::new("jackin/overlays/cockpit_credentials", Bin::Jackin, HARD, "jackin❯").sends(&["enter", "wait:Launch · choose Agent", "down", "enter", "advance:2000", "wait:Credential source unavailable"]).pending().dismiss_pending(),



    Root::new("jackin/manager/new_workspace_prelude", Bin::Jackin, FIRST_400, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]).variants(&[
        Variant::new(120, 40, Color::Truecolor, FIRST_400, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
        Variant::new(80, 24, Color::Truecolor, FIRST_400, "jackin❯").sends(&["n", "wait:step 1 of 5 · Source"]),
    ]).dismiss_pending(),
    Root::new("jackin/capsule/new_tab", Bin::Jackin, CAPSULE, "jackin❯").sends(&["ctrl-b", "c", "wait:Type to search…"]).dismiss_pending(),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[

    Root::new("jackin/resize/manager_from160x50", Bin::Jackin, RETURNING, "jackin❯").timeout(15_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("jackin/resize/manager_from72x20", Bin::Jackin, RETURNING, "jackin❯").timeout(15_000).resize((72, 20), interact::resize_case).pending(),
    Root::new("jackin/resize/accounts_from160x50", Bin::Jackin, ACCOUNTS, "jackin❯").timeout(15_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("jackin/resize/accounts_from72x20", Bin::Jackin, ACCOUNTS, "jackin❯").timeout(15_000).resize((72, 20), interact::resize_case).pending(),
];
