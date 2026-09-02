//! Component implementations. Each widget is a plain state struct with
//! `render` (draws + registers hit/focus regions) and small `on_*` handlers
//! returning [`Outcome`](crate::core::event::Outcome).

pub mod brand;
pub mod button;
pub mod chips;
pub mod choice;
pub mod code;
pub mod completion;
pub mod dialog;
pub mod diff;
pub mod empty;
pub mod field_common;
pub mod grid;
pub mod hintbar;
pub mod input;
pub mod keyhint;
pub mod list;
pub mod menu;
pub mod panel;
pub mod picker;
pub mod progress;
pub mod props;
pub mod scrollbar;
pub mod segments;
pub mod select;
pub mod splitter;
pub mod statusbar;
pub mod steps;
pub mod table;
pub mod tabs;
pub mod textarea;
pub mod tree;
pub mod viewport;
