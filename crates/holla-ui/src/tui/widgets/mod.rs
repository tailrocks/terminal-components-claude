//! Component implementations. Each widget is a plain state struct with
//! `render` (draws + registers hit/focus regions) and small `on_*` handlers
//! returning [`Outcome`](crate::tui::core::event::Outcome).

pub mod brand;
pub mod button;
pub mod chips;
pub mod dialog;
pub mod empty;
pub mod field_common;
pub mod hintbar;
pub mod input;
pub mod keyhint;
pub mod menu;
pub mod panel;
pub mod picker;
pub mod progress;
pub mod props;
pub mod scrollbar;
pub mod segments;
pub mod select;
pub mod statusbar;
pub mod stock_dialog;
pub mod tabs;
pub mod tree;
pub mod viewport;
