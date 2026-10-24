pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
#[cfg(test)]
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_text as text;
pub(crate) use termrock_theme as theme;

pub use termrock_core::action::ActionKey;

pub(crate) use termrock_runtime::{PartStyle, SlotFn, first_row, shift};

pub mod accounts_detail;
pub mod accounts_drawer;
pub mod accounts_filter;
pub mod capsule_menu;
pub mod hintbar;
pub mod inspector_body;
pub mod keyhint;
pub mod manager_frame;
pub mod manager_split;
pub mod meter;
pub mod notice;
pub mod progress;
pub mod section;
pub mod session_chrome;
pub mod settings_page;
pub mod status;
pub mod usage_overview;
pub mod zoom_pane;

pub use accounts_detail::{
    AccountRole, AccountRow, AccountsDetail, DetailAlert, DetailHint, DetailTail,
    accounts_detail_summary_area, paint_accounts_detail,
};
pub use accounts_drawer::{
    AccountsDrawer, DrawerButton, DrawerInk, DrawerLine, DrawerLineProjection, DrawerMeter,
    DrawerProjectionError, DrawerProjectionRange, DrawerProjectionSource, DrawerTone, PaneChrome,
    PaneRow, paint_accounts_drawer, paint_accounts_drawer_lines_window,
    project_accounts_drawer_line,
};
pub use accounts_filter::{
    AccountsFilter, DetailInk, DetailLine, DetailSpan, FilterHint, FilterRow, TailTone,
    paint_accounts_filter,
};
pub use capsule_menu::{
    CapsuleMenu, CapsuleMenuFrame, ClipSpan, ClipTone, MenuEntry, paint_capsule_menu,
};
pub use hintbar::{DerivedHintBar, HintBar};
pub use inspector_body::{
    InspectLine, InspectSpan, InspectTone, InspectorBodyWidths, inspector_body_row_count,
    paint_inspector_body, paint_inspector_body_lines_window,
};
pub use keyhint::{ChordText, HintText, KeyHint};
pub use manager_frame::{
    ManagerFrame, ManagerFrameLayout, ManagerLeaf, paint_manager_frame,
    paint_manager_frame_with_layout,
};
pub use manager_split::{
    ManagerSplit, SplitAlert, SplitInstance, SplitMode, SplitMount, SplitWorkspace,
    paint_manager_detail_drawer, paint_manager_split,
};
pub use meter::{Meter, MeterTone, MeterVisual};
pub use notice::{Notice, NoticeCut, NoticeKind, NoticeSource};
pub use progress::{ProgressBar, Spinner};
pub use section::TextSection;
pub use session_chrome::{SessionChrome, SessionHint, SessionMenu, paint_session_chrome};
pub use settings_page::{
    SettingsAgentRow, SettingsAgents, SettingsBodyPaintError, SettingsCheck, SettingsCluster,
    SettingsEnv, SettingsEnvGroup, SettingsEnvPaintError, SettingsEnvRole, SettingsEnvRow,
    SettingsHint, SettingsHintOverflow, SettingsHintOverflowTone, SettingsMount, SettingsPage,
    SettingsPageChrome, SettingsSection, SettingsTab, SettingsTrust, SettingsTrustRow,
    SettingsTrustSourceKind, paint_settings_agents_body, paint_settings_page,
    paint_settings_page_with_agents, paint_settings_page_with_env, paint_settings_page_with_trust,
};
pub use status::{Emphasis, Group, MAX_ITEMS, StatusAction, StatusBar, StatusItem};
pub use usage_overview::{
    UsageAccount, UsageAccountLine, UsageDetailLine, UsageHint, UsageInk, UsageLine, UsageMark,
    UsageOverview, UsageProvider, UsageQuota, UsageSpan, UsageTrail, paint_usage_overview,
};
pub use zoom_pane::{ZoomHint, ZoomSpan, ZoomTab, ZoomTone, ZoomedPane, paint_zoomed_pane};
