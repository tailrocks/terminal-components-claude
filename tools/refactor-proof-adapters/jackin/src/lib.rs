//! Reference observation adapter for Jackin oracle capture (TASK-004).
//!
//! The adapter constructs production [`App`](jackin_app::App) values through
//! [`App::for_scenario`](jackin_app::App::for_scenario) and observes the
//! public runtime draw path. It does not call the app-local historical
//! paint-over helpers and does not bless candidate frames as expected
//! artifacts.

#![forbid(unsafe_code)]

mod frame;
mod ja001;
mod ja002;
mod ja003;
mod ja004;
mod ja005;
mod ja006;
mod ja007;
mod ja008;
mod ja009;
mod ja010;
mod ja011;
mod ja012;
mod ja013;
mod ja014;
mod ja015;
mod ja016;
mod ja017;
mod ja018;
mod ja019;
mod ja020;
mod ja021;
mod ja022;
mod ja023;
mod ja024;
mod ja025;
mod ja026;
mod ja027;
mod ja028;
mod ja029;
mod ja030;
mod ja031;
mod ja032;
mod ja033;
mod ja034;
mod ja035;
mod ja036;
mod ja037;
mod ja038;
mod ja039;
mod ja040;
mod ja041;
mod ja042;
mod ja043;
mod ja044;
mod ja045;
mod ja046;
mod ja047;
mod ja048;
mod ja049;
mod ja050;
mod ja051;
mod ja052;
mod ja053;
mod ja054;
mod ja055;
mod ja056;
mod ja057;
mod ja058;
mod ja059;
mod ja060;
mod ja061;
mod ja062;
mod ja063;
mod ja064;
mod ja065;
mod ja066;
mod ja067;
mod ja068;
mod ja069;
mod ja070;
mod observe;

pub use frame::{ObservedCell, ObservedCursor, ObservedFrame};
pub use ja001::{
    JA001_FIRST_LEAF_COLOR, JA001_FIRST_LEAF_HEIGHT, JA001_FIRST_LEAF_SIZE, JA001_FIRST_LEAF_WIDTH,
    JA001_ID, JA001_REMAINING_SIZES, JA001_SIZES, Ja001Capture, expected_route,
    ja001_paused_frame0_all_sizes_truecolor, ja001_paused_frame0_at,
    ja001_paused_frame0_remaining_colors, ja001_paused_frame0_remaining_sizes_truecolor,
    ja001_paused_frame0_truecolor, ja001_worlds,
};
pub use ja002::{
    JA002_ID, ja002_frames, ja002_paused_first_use, knock_caption, production_intro_message,
};
pub use ja003::{JA003_ID, JA003_SIZES, Ja003Capture, ja003_first_use_full};
pub use ja004::{JA004_ID, JA004_SIZES, Ja004Capture, ja004_first_use_reduced_and_quit};
pub use ja005::{
    JA005_ID, JA005_PREFIX, JA005_VARIANTS, Ja005Capture, ja005_returning_manager_keys,
};
pub use ja006::{JA006_ID, Ja006Capture, ja006_returning_manager_pointer};
pub use ja007::{
    JA007_ID, JA007_LABELS, JA007_SIZES, Ja007Capture, ja007_returning_row_activation,
};
pub use ja008::{JA008_ID, JA008_SIZES, JA008_WORLDS, Ja008Capture, ja008_workspace_actions};
pub use ja009::{JA009_ID, JA009_SIZES, Ja009Capture, ja009_instance_actions};
pub use ja010::{JA010_ID, Ja010Capture, ja010_hard_cases_refresh_scroll};
pub use ja011::{JA011_ID, JA011_SIZES, Ja011Capture, ja011_launch_picker};
pub use ja012::{JA012_ID, Ja012Capture, ja012_prelude_pending_editor};
pub use ja013::{JA013_ID, JA013_SIZES, Ja013Capture, ja013_prelude_git_and_rewind};
pub use ja014::{JA014_ID, JA014_SIZES, Ja014Capture, ja014_prelude_name_validation};
pub use ja015::{JA015_ID, Ja015Capture, ja015_prelude_browser};
pub use ja016::{JA016_ID, Ja016Capture, ja016_editor_tabs};
pub use ja017::{JA017_ID, JA017_SIZES, Ja017Capture, ja017_editor_name_edit};
pub use ja018::{JA018_ID, JA018_SIZES, Ja018Capture, ja018_editor_save_replay};
pub use ja019::{JA019_ID, JA019_SIZES, JA019_VARIANT_NAMES, Ja019Capture, ja019_mount_actions};
pub use ja020::{JA020_ID, JA020_SIZES, Ja020Capture, ja020_role_actions};
pub use ja021::{JA021_FIXTURE_SECRET, JA021_ID, Ja021Capture, ja021_env_masked_replay};
pub use ja022::{JA022_ID, JA022_SIZES, JA022_VARIANT_NAMES, Ja022Capture, ja022_env_actions};
pub use ja023::{JA023_ID, Ja023Capture, ja023_accounts_tab_replay};
pub use ja024::{JA024_ID, Ja024Capture, ja024_hundred_roles_replay};
pub use ja025::{JA025_ID, JA025_SIZES, Ja025Capture, ja025_leave_branches};
pub use ja026::{JA026_ALIASES, JA026_ID, JA026_TABS, Ja026Capture, ja026_settings_tabs};
pub use ja027::{
    JA027_ENV_KEYS, JA027_ID, JA027_MOUNT_KEYS, JA027_SIZES, Ja027Capture,
    ja027_settings_global_attempts,
};
pub use ja028::{JA028_ID, JA028_SIZES, Ja028Capture, ja028_settings_trust_walk};
pub use ja029::{JA029_ID, JA029_SIZES, Ja029Capture, ja029_trust_save_retry};
pub use ja030::{
    JA030_ID, JA030_PREFIX, JA030_PREFIX_NO_ESC, Ja030Capture, ja030_accounts_navigation,
};
pub use ja031::{JA031_ID, Ja031Capture, ja031_accounts_pointer};
pub use ja032::{JA032_ID, JA032_SIZES, Ja032Capture, ja032_one_password_replay};
pub use ja033::{JA033_FIXTURE_KEY, JA033_ID, Ja033Capture, ja033_plain_key_replay};
pub use ja034::{JA034_ID, JA034_SIZES, Ja034Capture, ja034_registration_grid};
pub use ja035::{
    JA035_ID, JA035_SIZES, JA035_VARIANT_NAMES, JA035_WORK_ID, Ja035Capture, ja035_account_actions,
};
pub use ja036::{JA036_ID, JA036_VARIANT_DOWNS, Ja036Capture, ja036_hard_cases_refresh};
pub use ja037::{
    JA037_ID, JA037_SIZES, Ja037Capture, OpErrorCapture, ja037_error_taxonomy, ja037_op_errors,
};
pub use ja038::{JA038_ID, JA038_SIZES, Ja038Capture, ja038_op_item_picker};
pub use ja039::{JA039_ID, Ja039Capture, ja039_usage_handoff};
pub use ja040::{JA040_ID, JA040_TICK_BOUND, Ja040Capture, ja040_launch_stages};
pub use ja041::{JA041_ID, JA041_SIZES, Ja041Capture, ja041_cockpit_overlays};
pub use ja042::{JA042_ID, Ja042Capture, ja042_launch_failure};
pub use ja043::{
    JA043_ID, JA043_SIZES, JA043_TICK_BOUND, Ja043Capture, ja043_blocked_run, ja043_launch_blocks,
    ja043_locked_run,
};
pub use ja044::{JA044_ID, JA044_SIZES, JA044_VARIANT_NAMES, Ja044Capture, ja044_launch_cancel};
pub use ja045::{JA045_ID, JA045_SIZES, Ja045Capture, ja045_effective_accounts};
pub use ja046::{JA046_DIGITS, JA046_ID, JA046_TABS, Ja046Capture, ja046_tab_switch};
pub use ja047::{JA047_ID, JA047_SIZES, JA047_TIMEOUT_TICKS, Ja047Capture, ja047_prefix_keys};
pub use ja048::{JA048_ID, JA048_VARIANTS, Ja048Capture, ja048_topology_growth};
pub use ja049::{
    JA049_FOCUS, JA049_ID, JA049_LEFT_NEEDLES, JA049_RIGHT_NEEDLES, Ja049Capture,
    ja049_pane_geometry,
};
pub use ja050::{JA050_ID, JA050_VARIANT_NAMES, Ja050Capture, ja050_pane_typing};
pub use ja051::{JA051_ID, JA051_NEEDLES, Ja051Capture, ja051_scrollback_copy};
pub use ja052::{JA052_ID, Ja052Capture, ja052_tab_menu};
pub use ja053::{JA053_ID, JA053_MENUS, Ja053Capture, ja053_menu_bar};
pub use ja054::{JA054_ID, JA054_SIZES, Ja054Capture, ja054_palette};
pub use ja055::{JA055_CHIPS, JA055_ID, Ja055Capture, ja055_usage_chips_info};
pub use ja056::{JA056_ID, JA056_SIZES, JA056_VARIANTS, Ja056Capture, ja056_close_attempts};
pub use ja057::{DaemonTabs, JA057_ID, JA057_SIZES, Ja057Capture, ja057_takeover_substrate};
pub use ja058::{
    JA058_ID, JA058_OUTRO_CAPTION, JA058_SIZES, Ja058Capture, ja058_detach_and_still_inside,
};
pub use ja059::{JA059_ID, JA059_SIZES, Ja059Capture, ja059_intro_ritual};
pub use ja060::{JA060_ID, JA060_PINNED_FRAME, JA060_SIZES, Ja060Capture, ja060_motion_policies};
pub use ja061::{
    JA061_ENTERED_AT_MS, JA061_ID, JA061_OUTRO_CAPTION, JA061_SIZES, Ja061Capture,
    ja061_outro_ritual,
};
pub use ja062::{
    JA062_FAILURE_NOTICE, JA062_ID, JA062_SIZES, Ja062Capture, ja062_launch_and_failure,
};
pub use ja063::{JA063_ID, JA063_SIZES, Ja063Capture, ja063_help_overlays};
pub use ja064::{
    JA064_ID, JA064_PALETTE_FOCUS, JA064_SIZES, Ja064Capture, ja064_overlay_dismissal,
};
pub use ja065::{JA065_ID, JA065_SIZES, JA065_TREE_HOVER, Ja065Capture, ja065_hover_and_focus};
pub use ja066::{JA066_ID, JA066_SIZES, Ja066Capture, ja066_too_small};
pub use ja067::{JA067_ID, JA067_SIZES, Ja067Capture, ja067_color_identities};
pub use ja068::{JA068_ID, JA068_PINNED_FRAME, JA068_SIZES, Ja068Capture, ja068_cli_contract};
pub use ja069::{JA069_ID, JA069_MILESTONES, JA069_SIZES, Ja069Capture, ja069_complete_journey};
pub use ja070::{JA070_ID, JA070_SIZES, JA070_UNKNOWN_REASON, Ja070Capture, ja070_arbiter};
pub use observe::{
    CaptureColor, DirectSession, EPOCH_SECS, HELPER_TICK_MS, HISTORICAL_PAINT_SIZE, MOTION_SEED,
    Viewport, color_label, motion_name, route_name,
};

pub use jackin_app::{App, Motion, Route, Scenario};
pub use junie_tui::{Axis, ColorLevel, Id, KeyCode, KeyModifiers, MouseKind, Rect};
