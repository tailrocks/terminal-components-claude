//! TASK-005 `TablePro` oracle observation adapter.
//!
//! Direct and subprocess lanes drive the production CLI (`tablepro_app::run`)
//! and the same `TableProApp::default` / `connect` constructors `run_with`
//! uses. Named visual surfaces are never seeded through `set_surface`.

#![forbid(unsafe_code)]

mod cli;
mod input;
mod launch;
mod observe;
mod scenario;

pub use cli::{HELP, invoke_cli, invoke_cli_with_env, parse_color_flag};
pub use input::{
    alt, backtab, click_text, ctrl, delete, down, end, enter, esc, function, home, hover_text,
    left, page_down, page_up, pointer_down_text, pointer_up_text, repeat_key, replace_paste, right,
    shift, space, tab, ticks, type_text, up, wheel_down_text,
};
pub use launch::{
    connect_fixture, launch_preset_c, launch_preset_l, launch_preset_q, launch_preset_ql,
    launch_preset_t, launch_preset_w, theme_for,
};
pub use observe::{Observation, observe};
pub use scenario::{
    ALL_SIZES, CANONICAL_SIZES, CHECKPOINT_INITIAL, COLOR_PROFILES, ColorProfile,
    FIXTURE_ANALYTICS, FIXTURE_LOCAL_POSTGRESQL, FIXTURE_PRODUCTION, FIXTURE_STAGING,
    ORACLE_KEY_COUNT, ORACLE_SCENARIOS, PRESET_C, PRESET_L, PRESET_Q, PRESET_QL, PRESET_T,
    PRESET_W, SCENARIO_TP001, SCENARIO_TP002, oracle_key,
};
