//! Border sets: ratatui's, re-exported, plus the ASCII set
//! (`COMPONENT_ARCHITECTURE.md` §11.2, §22 R‑11, Adjudication M2).

pub use ratatui_core::symbols::border::{DOUBLE, PLAIN, ROUNDED, Set};

/// The theme's border set: a type alias of ratatui's eight-field set, so a
/// theme hands over `border::PLAIN` / `ROUNDED` / `DOUBLE` directly.
pub type BorderSet = Set<'static>;

/// Pure-ASCII border set, for terminals and fonts without box-drawing glyphs.
/// Not shipped by ratatui; declared here as a plain `const` because
/// `BorderSet` is a type alias of a foreign type and can carry no inherent
/// items (§11.2). Opt in with
/// `Theme::junie().builder().borders_set(border::ASCII).build()`.
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
            assert_eq!(crate::text::width(g), 1);
        }
    }
}
