pub use ratatui_core::symbols::border::{DOUBLE, PLAIN, ROUNDED, Set};

pub type BorderSet = Set<'static>;

pub const ASCII: Set<'static> = Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};
