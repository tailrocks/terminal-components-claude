//! Component implementations. Each widget is a plain state struct with
//! `render` (draws + registers hit/focus regions) and small `on_*` handlers
//! returning [`Outcome`](crate::core::event::Outcome).

pub mod button;
pub mod choice;
pub mod dialog;
pub mod field_common;
pub mod input;
pub mod list;
pub mod panel;
pub mod progress;
pub mod scrollbar;
pub mod table;
pub mod tabs;
pub mod textarea;
pub mod tree;
