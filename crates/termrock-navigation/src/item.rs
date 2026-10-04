use ratatui::style::Modifier;

use termrock_core::id::{ItemKey, Part};
use termrock_core::response::StateFlags;
use termrock_layout::Track;
use termrock_runtime::{RowFn, RowUi};
use termrock_text::width;

/// Borrowed semantic data shared by picker and completion rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Item<'i> {
    /// One-cell kind glyph.
    pub glyph: &'i str,
    /// Visible and searchable label.
    pub label: &'i str,
    /// Grapheme ordinals in `label` that matched.
    pub matched: &'i [usize],
    /// Secondary description.
    pub detail: &'i str,
    /// Text inserted by completion; `None` inserts `label`.
    pub insert: Option<&'i str>,
    /// Optional trailing tag.
    pub tag: Option<&'i str>,
    /// Optional group label.
    pub group: Option<&'i str>,
    /// Whether activation is refused.
    pub disabled: bool,
    /// Stable semantic identity.
    pub key: ItemKey,
}

impl<'i> Item<'i> {
    /// A minimal enabled item. Optional columns are empty and completion inserts the label.
    pub const fn new(key: ItemKey, label: &'i str) -> Self {
        Self {
            glyph: "",
            label,
            matched: &[],
            detail: "",
            insert: None,
            tag: None,
            group: None,
            disabled: false,
            key,
        }
    }
    /// Set the completion insertion text independently of the label.
    #[must_use]
    pub const fn insert(mut self, text: &'i str) -> Self {
        self.insert = Some(text);
        self
    }
    /// Set the glyph.
    #[must_use]
    pub const fn glyph(mut self, glyph: &'i str) -> Self {
        self.glyph = glyph;
        self
    }
    /// Set matched grapheme ordinals.
    #[must_use]
    pub const fn matched(mut self, matched: &'i [usize]) -> Self {
        self.matched = matched;
        self
    }
    /// Set detail.
    #[must_use]
    pub const fn detail(mut self, detail: &'i str) -> Self {
        self.detail = detail;
        self
    }
    /// Set a tag.
    #[must_use]
    pub const fn tag(mut self, tag: &'i str) -> Self {
        self.tag = Some(tag);
        self
    }
    /// Set a group.
    #[must_use]
    pub const fn group(mut self, group: &'i str) -> Self {
        self.group = Some(group);
        self
    }
    /// Disable activation.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Completion insertion text, falling back to the visible label.
    pub const fn insertion(self) -> &'i str {
        match self.insert {
            Some(text) => text,
            None => self.label,
        }
    }
}

/// Convert a domain item to the borrowed semantic picker/completion view.
pub trait AsItem {
    /// Borrow this value as an item.
    fn as_item(&self) -> Item<'_>;
}

impl AsItem for Item<'_> {
    fn as_item(&self) -> Item<'_> {
        *self
    }
}

/// Built-in semantic item layout. Custom row callbacks remain authoritative.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ItemRowLayout {
    /// Existing compact label with trailing glyph and metadata.
    #[default]
    Compact,
    /// Leading glyph, aligned label/detail/tag/group columns and matched emphasis.
    Columns,
}

/// One measurement across the complete semantic projection, not only visible rows.
#[derive(Default)]
pub struct ItemColumns {
    label: u16,
    tag: u16,
    group: u16,
}

impl ItemColumns {
    pub fn measure<'a>(items: impl Iterator<Item = Item<'a>>, row_width: u16) -> Self {
        let mut columns = Self::default();
        for item in items {
            columns.label = columns.label.max(width(item.label));
            columns.tag = columns.tag.max(width(item.tag.unwrap_or("")));
            columns.group = columns.group.max(width(item.group.unwrap_or("")));
        }
        let limit = u16::try_from(u32::from(row_width).saturating_mul(45) / 100)
            .unwrap_or(u16::MAX)
            .max(6);
        columns.label = columns.label.clamp(6, limit);
        columns
    }

    pub fn paint(&self, item: Item<'_>, show_group: bool, row: &mut RowUi<'_>) {
        let focused = row.flags().contains(StateFlags::FOCUSED);
        row.gutter();
        let mut cells = row.columns_with_gap(
            &[
                Track::Fixed(1),
                Track::Fixed(1),
                Track::Fixed(self.label),
                Track::Fixed(2),
                Track::Flex(1),
                Track::Fixed(1),
                Track::Fixed(self.tag),
                Track::Fixed(if self.tag > 0 { 2 } else { 0 }),
                Track::Fixed(self.group),
                Track::Fixed(u16::from(self.group > 0)),
            ],
            0,
        );
        {
            let mut icon = cells.cell_part(0, Part::ICON);
            if !icon.authored_modifiers().contains(Modifier::BOLD) {
                icon.remove_modifier(Modifier::BOLD);
            }
            icon.text(item.glyph);
        }
        {
            let mut label = cells.cell_part(2, Part::LABEL);
            if !focused && !label.authored_modifiers().contains(Modifier::BOLD) {
                label.remove_modifier(Modifier::BOLD);
            }
            label.text_matched(item.label, item.matched);
        }
        if cells.rect(4).width >= 4 {
            let mut meta = cells.cell_part(4, Part::META);
            if !meta.authored_modifiers().contains(Modifier::BOLD) {
                meta.remove_modifier(Modifier::BOLD);
            }
            meta.text_matched(item.detail, &[]);
        }
        {
            let mut tag = cells.cell_part(6, Part::META);
            if !tag.authored_modifiers().contains(Modifier::BOLD) {
                tag.remove_modifier(Modifier::BOLD);
            }
            tag.text(item.tag.unwrap_or(""));
        }
        if show_group {
            let mut group = cells.cell_part(8, Part::META);
            if !group.authored_modifiers().contains(Modifier::BOLD) {
                group.remove_modifier(Modifier::BOLD);
            }
            group.text(item.group.unwrap_or(""));
        }
    }
}

/// Default semantic row painter.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ItemRow;

impl<T: AsItem> RowFn<T> for ItemRow {
    fn row(&self, value: &T, row: &mut RowUi<'_>) {
        let item = value.as_item();
        row.gutter();
        if let Some(tag) = item.tag {
            row.meta(tag);
        }
        if !item.detail.is_empty() {
            row.meta(item.detail);
        }
        if !item.glyph.is_empty() {
            row.part(Part::ICON, 1).text(item.glyph);
        }
        row.label(item.label);
    }
}
