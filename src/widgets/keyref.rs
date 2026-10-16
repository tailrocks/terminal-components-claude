//! A keyboard reference: key combinations in one column, what they do in
//! the next, grouped into optional sections.
//!
//! Built for help overlays. The entries are laid out as aligned text whose
//! key column is separated by a wide gap, so [`crate::ui::text::wrap`]
//! hangs a wrapped description under the description column instead of
//! returning to column 0 (see [`WrapOpts::hang_at_gap`]). The text drops
//! straight into a dialog's text body, which wraps it at whatever width the
//! dialog gets.

use crate::ui::text::{WrapOpts, width, wrap_with};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    Section(String),
    Key { keys: String, action: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyReference {
    entries: Vec<Entry>,
    /// Spaces between the key column and the description column. Values
    /// below 3 are raised to 3: that is the gap a wrapped description hangs
    /// under ([`WrapOpts::default`]). Default `3`.
    pub gap: u16,
    /// The key column never takes more than this share of the width, in
    /// percent; a longer key pushes its own description along instead of
    /// widening every row. Default `40`.
    pub key_max_share: u16,
}

impl Default for KeyReference {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyReference {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            gap: 3,
            key_max_share: 40,
        }
    }

    /// Start a titled group; a blank line separates it from the previous one.
    pub fn section(mut self, title: &str) -> Self {
        self.entries.push(Entry::Section(title.to_owned()));
        self
    }

    pub fn key(mut self, keys: &str, action: &str) -> Self {
        self.entries.push(Entry::Key {
            keys: keys.to_owned(),
            action: action.to_owned(),
        });
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn key_max_share(mut self, percent: u16) -> Self {
        self.key_max_share = percent.min(100);
        self
    }

    fn key_column(&self, total: Option<usize>) -> usize {
        let widest = self
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Key { keys, .. } => Some(width(keys)),
                Entry::Section(_) => None,
            })
            .max()
            .unwrap_or(0);
        match total {
            Some(w) => widest.min(w * self.key_max_share as usize / 100),
            None => widest,
        }
    }

    fn layout(&self, key_w: usize) -> String {
        let gap = " ".repeat(self.gap.max(3) as usize);
        let mut out: Vec<String> = Vec::new();
        for (i, e) in self.entries.iter().enumerate() {
            match e {
                Entry::Section(title) => {
                    if i > 0 {
                        out.push(String::new());
                    }
                    out.push(title.clone());
                }
                Entry::Key { keys, action } => {
                    let pad = key_w.saturating_sub(width(keys));
                    out.push(format!("{keys}{}{gap}{action}", " ".repeat(pad)));
                }
            }
        }
        out.join("\n")
    }

    /// The reference as aligned text, for a body that wraps it later (a
    /// dialog's text body). The key column is as wide as the widest key.
    pub fn text(&self) -> String {
        self.layout(self.key_column(None))
    }

    /// The reference wrapped to `w` cells: descriptions hang under their
    /// column, and the key column is capped at `key_max_share`.
    pub fn lines(&self, w: usize) -> Vec<String> {
        wrap_with(
            &self.layout(self.key_column(Some(w))),
            w,
            WrapOpts::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::text::wrap;

    fn sample() -> KeyReference {
        KeyReference::new()
            .key(
                "Ctrl+D",
                "Data / Structure · Ctrl+F filter (grid) or find (editor)",
            )
            .key("q", "quit")
    }

    #[test]
    fn a_wrapped_description_hangs_under_its_column() {
        for lines in [sample().lines(40), wrap(&sample().text(), 40)] {
            assert_eq!(lines[0], "Ctrl+D   Data / Structure · Ctrl+F");
            assert!(
                lines[1].starts_with("         filter"),
                "continuation hangs under the description: {lines:?}"
            );
            assert_eq!(lines.last().unwrap(), "q        quit");
        }
    }

    #[test]
    fn sections_are_separated_and_the_key_column_is_capped() {
        let r = KeyReference::new()
            .section("Editor")
            .key("a very long key chord here", "x")
            .section("Grid")
            .key("q", "quit");
        let lines = r.lines(20);
        assert_eq!(lines[0], "Editor");
        assert!(lines.contains(&String::new()));
        // the 8-cell cap (40% of 20) leaves "q" its own short column
        assert!(lines.contains(&"q          quit".to_owned()), "{lines:?}");
    }
}
