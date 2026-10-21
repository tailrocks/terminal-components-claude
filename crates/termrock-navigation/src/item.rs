use ratatui_core::style::Modifier;

use termrock_core::id::{ItemKey, Part};
use termrock_core::response::StateFlags;
use termrock_layout::layout::Track;
use termrock_layout::{ColumnSpec, solve_columns_first};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_text::width;
use termrock_theme::{FgStep, Role, StyleDefaults, StylePatch};

fn picker_ink(role: FgStep) -> StyleDefaults<'static> {
    StyleDefaults::new(
        StylePatch::new()
            .set_fg(Role::Fg(role))
            .remove(Modifier::BOLD),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Item<'i> {
    pub glyph: &'i str,
    pub label: &'i str,
    pub matched: &'i [usize],
    pub detail: &'i str,
    pub insert: Option<&'i str>,
    pub tag: Option<&'i str>,
    pub group: Option<&'i str>,
    pub disabled: bool,
    pub key: ItemKey,
}

impl<'i> Item<'i> {
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
    #[must_use]
    pub const fn insert(mut self, text: &'i str) -> Self {
        self.insert = Some(text);
        self
    }
    #[must_use]
    pub const fn glyph(mut self, glyph: &'i str) -> Self {
        self.glyph = glyph;
        self
    }
    #[must_use]
    pub const fn matched(mut self, matched: &'i [usize]) -> Self {
        self.matched = matched;
        self
    }
    #[must_use]
    pub const fn detail(mut self, detail: &'i str) -> Self {
        self.detail = detail;
        self
    }
    #[must_use]
    pub const fn tag(mut self, tag: &'i str) -> Self {
        self.tag = Some(tag);
        self
    }
    #[must_use]
    pub const fn group(mut self, group: &'i str) -> Self {
        self.group = Some(group);
        self
    }
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub const fn insertion(self) -> &'i str {
        match self.insert {
            Some(text) => text,
            None => self.label,
        }
    }
}

pub trait AsItem {
    fn as_item(&self) -> Item<'_>;
}

impl AsItem for Item<'_> {
    fn as_item(&self) -> Item<'_> {
        *self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ItemRowLayout {
    #[default]
    Compact,
    Columns,
}

pub struct ItemColumns {
    label: u16,
    tag: u16,
    group: u16,
    metadata_gap: u16,
}

impl Default for ItemColumns {
    fn default() -> Self {
        Self {
            label: 0,
            tag: 0,
            group: 0,
            metadata_gap: 1,
        }
    }
}

impl ItemColumns {
    pub(crate) fn measure<'a>(
        items: impl Iterator<Item = Item<'a>>,
        row_width: u16,
        metadata_gap: u16,
    ) -> Self {
        let mut label_need = 0u16;
        let mut detail_need = 0u16;
        let mut tag = 0u16;
        let mut group = 0u16;
        for item in items {
            label_need = label_need.max(width(item.label));
            detail_need = detail_need.max(width(item.detail));
            tag = tag.max(width(item.tag.unwrap_or("")));
            group = group.max(width(item.group.unwrap_or("")));
        }
        // Baseline `columns::solve`: the label and the detail share spare
        // width, so a long label stops short of the 45% cap.
        label_need = label_need.max(6);
        let cap = u16::try_from(u32::from(row_width).saturating_mul(45) / 100)
            .unwrap_or(u16::MAX)
            .max(6);
        let label_cap = label_need.min(cap);
        let label = solve_label(row_width, label_cap, detail_need, tag, group);
        Self {
            label,
            tag,
            group,
            metadata_gap,
        }
    }

    #[must_use]
    pub const fn metadata_gap(mut self, gap: u16) -> Self {
        self.metadata_gap = gap;
        self
    }

    pub(crate) fn paint(&self, item: Item<'_>, show_group: bool, row: &mut RowUi<'_>) {
        let focused = row.flags().contains(StateFlags::FOCUSED);
        row.gutter();
        let mut cells = row.columns_with_gap(
            &[
                Track::Fixed(1),
                Track::Fixed(1),
                Track::Fixed(self.label),
                Track::Fixed(2),
                Track::Flex(1),
                Track::Fixed(if self.tag > 0 || self.group > 0 {
                    self.metadata_gap
                } else {
                    1
                }),
                Track::Fixed(self.tag),
                Track::Fixed(if self.tag > 0 {
                    if self.metadata_gap == 1 {
                        2
                    } else {
                        self.metadata_gap
                    }
                } else {
                    0
                }),
                Track::Fixed(self.group),
                Track::Fixed(u16::from(self.group > 0)),
            ],
            0,
        );

        {
            let icon_ink = if focused {
                FgStep::Primary
            } else {
                FgStep::Muted
            };
            let mut icon = cells.cell_part_with_defaults(0, Part::ICON, picker_ink(icon_ink));
            if !icon.authored_modifiers().contains(Modifier::BOLD) {
                icon.remove_modifier(Modifier::BOLD);
            }
            icon.text(if item.glyph.is_empty() {
                " "
            } else {
                item.glyph
            });
        }
        {
            let mut label = cells.cell_part(2, Part::LABEL);
            if !focused && !label.authored_modifiers().contains(Modifier::BOLD) {
                label.remove_modifier(Modifier::BOLD);
            }
            label.text_matched(item.label, item.matched);
        }
        if cells.rect(4).width >= 4 {
            let mut meta = cells.cell_part_with_defaults(4, Part::META, picker_ink(FgStep::Muted));
            if !meta.authored_modifiers().contains(Modifier::BOLD) {
                meta.remove_modifier(Modifier::BOLD);
            }
            meta.text_matched(item.detail, &[]);
        }
        {
            let mut tag =
                cells.cell_part_with_defaults(6, Part::META, picker_ink(FgStep::Secondary));
            if !tag.authored_modifiers().contains(Modifier::BOLD) {
                tag.remove_modifier(Modifier::BOLD);
            }
            tag.text(item.tag.unwrap_or(""));
        }
        if show_group {
            let mut group = cells.cell_part_with_defaults(8, Part::META, picker_ink(FgStep::Muted));
            if !group.authored_modifiers().contains(Modifier::BOLD) {
                group.remove_modifier(Modifier::BOLD);
            }
            group.text(item.group.unwrap_or(""));
        }
    }
}

fn solve_label(row_width: u16, label_cap: u16, detail_need: u16, tag: u16, group: u16) -> u16 {
    let mut cols = [ColumnSpec::default(); 4];
    let detail_min = detail_need.min(4);
    cols[0] = ColumnSpec::new(label_cap.min(12), label_cap, label_cap, 0, None);
    cols[1] = ColumnSpec::new(
        detail_min,
        detail_need.max(detail_min),
        u16::MAX,
        1,
        Some(0),
    );
    let mut n = 2usize;
    if tag > 0 {
        cols[n] = ColumnSpec::new(tag, tag, tag, 0, None);
        n = n.saturating_add(1);
    }
    if group > 0 {
        cols[n] = ColumnSpec::new(group, group, group, 0, None);
        n = n.saturating_add(1);
    }
    solve_columns_first(&cols[..n], row_width.saturating_sub(4), 2).unwrap_or(label_cap)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ItemRow;

impl<T: AsItem> RowFn<T> for ItemRow {
    fn row(&self, value: &T, row: &mut RowUi<'_>) {
        let item = value.as_item();
        row.gutter();
        if let Some(tag) = item.tag {
            row.meta_with_defaults(tag, picker_ink(FgStep::Secondary));
        }
        if !item.detail.is_empty() {
            row.meta(item.detail);
        }
        if !item.glyph.is_empty() {
            row.part(Part::ICON, 1).text(item.glyph);
        }
        row.label_matched(item.label, item.matched);
    }
}
