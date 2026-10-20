use crate::ui::text::{WrapOpts, width, wrap_with};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    Section(String),
    Key { keys: String, action: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyReference {
    entries: Vec<Entry>,

    pub gap: u16,

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

    pub fn text(&self) -> String {
        self.layout(self.key_column(None))
    }

    pub fn lines(&self, w: usize) -> Vec<String> {
        wrap_with(
            &self.layout(self.key_column(Some(w))),
            w,
            WrapOpts::default(),
        )
    }
}

const SPECIMEN_W: usize = 32;

impl crate::ui::specimen::Specimen for KeyReference {
    fn specimen_id(&self) -> crate::core::id::WidgetId {
        crate::core::id::WidgetId::of("keyref.specimen")
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Overflow | Narrow)
    }

    fn specimen_height(&self, state: crate::ui::specimen::SpecimenState) -> u16 {
        let w = if state == crate::ui::specimen::SpecimenState::Narrow {
            24
        } else {
            SPECIMEN_W
        };
        self.sample(state).lines(w).len().max(1) as u16
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        ctx: &mut crate::ui::ctx::RenderCtx,
        bg: ratatui::style::Color,
    ) {
        let t = ctx.theme;
        crate::ui::ctx::fill(buf, area, ratatui::style::Style::new().bg(bg));
        let r = self.sample(state);
        let sections: Vec<&str> = r
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Section(s) => Some(s.as_str()),
                Entry::Key { .. } => None,
            })
            .collect();
        let key_w = r.key_column(Some(area.width as usize));
        for (i, line) in r.lines(area.width as usize).iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            if sections.contains(&line.as_str()) {
                let st = t
                    .secondary()
                    .bg(bg)
                    .add_modifier(ratatui::style::Modifier::BOLD);
                buf.set_string(area.x, y, line, st);
                continue;
            }
            buf.set_string(area.x, y, line, t.secondary().bg(bg));

            let head: String = line.chars().take(key_w).collect();
            if !head.starts_with(' ') {
                buf.set_string(area.x, y, head.trim_end(), t.key_hint_key().bg(bg));
            }
        }
    }
}

impl KeyReference {
    fn sample(&self, state: crate::ui::specimen::SpecimenState) -> Self {
        let mut r = self.clone();
        if state == crate::ui::specimen::SpecimenState::Overflow {
            r = r.key(
                "Ctrl+Shift+F",
                "find in every open file, and list each match under its file",
            );
        }
        r
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

        assert!(lines.contains(&"q          quit".to_owned()), "{lines:?}");
    }
}
