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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_border_set_is_pure_ascii() {
        let s = ASCII;
        for g in [
            s.top_left,
            s.top_right,
            s.bottom_left,
            s.bottom_right,
            s.vertical_left,
            s.vertical_right,
            s.horizontal_top,
            s.horizontal_bottom,
        ] {
            assert!(g.is_ascii() && g.len() == 1, "{g:?}");
            assert_eq!(unicode_width::UnicodeWidthStr::width(g), 1);
        }
    }
}
