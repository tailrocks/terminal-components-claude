use ratatui_core::style::Modifier;

use termrock_core::id::{ItemKey, Part};
use termrock_core::response::StateFlags;
use termrock_layout::layout::Track;
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_text::width;

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

#[derive(Clone, Copy)]
struct SolvedCol {
    min: u16,
    ideal: u16,
    max: u16,
    flex: u16,
    drop: bool,
}

fn solve_label(row_width: u16, label_cap: u16, detail_need: u16, tag: u16, group: u16) -> u16 {
    let mut cols = [SolvedCol {
        min: 0,
        ideal: 0,
        max: 0,
        flex: 0,
        drop: false,
    }; 4];
    let detail_min = detail_need.min(4);
    cols[0] = SolvedCol {
        min: label_cap.min(12),
        ideal: label_cap,
        max: label_cap,
        flex: 0,
        drop: false,
    };
    cols[1] = SolvedCol {
        min: detail_min,
        ideal: detail_need.max(detail_min),
        max: u16::MAX,
        flex: 1,
        drop: true,
    };
    let mut n = 2usize;
    if tag > 0 {
        cols[n] = SolvedCol {
            min: tag,
            ideal: tag,
            max: tag,
            flex: 0,
            drop: false,
        };
        n = n.saturating_add(1);
    }
    if group > 0 {
        cols[n] = SolvedCol {
            min: group,
            ideal: group,
            max: group,
            flex: 0,
            drop: false,
        };
        n = n.saturating_add(1);
    }
    let widths = solve_columns(&cols[..n], row_width.saturating_sub(4), 2);
    widths.first().copied().unwrap_or(label_cap)
}

fn solve_columns(cols: &[SolvedCol], avail: u16, gap: u16) -> [u16; 4] {
    let n = cols.len().min(4);
    let mut visible = [true; 4];
    let gaps = |visible: &[bool; 4]| -> u32 {
        gap as u32
            * visible
                .iter()
                .take(n)
                .filter(|on| **on)
                .count()
                .saturating_sub(1) as u32
    };
    let mins = |visible: &[bool; 4]| -> u32 {
        cols.iter()
            .take(n)
            .zip(visible.iter().take(n))
            .filter(|(_, on)| **on)
            .map(|(col, _)| u32::from(col.min))
            .sum()
    };
    while mins(&visible).saturating_add(gaps(&visible)) > u32::from(avail) {
        let victim = cols
            .iter()
            .take(n)
            .enumerate()
            .filter(|(i, col)| visible[*i] && col.drop)
            .min_by_key(|(i, _)| core::cmp::Reverse(*i))
            .map(|(i, _)| i);
        match victim {
            Some(i) => visible[i] = false,
            None => break,
        }
    }
    let room = u32::from(avail).saturating_sub(gaps(&visible));
    let mut w = [0u32; 4];
    for (i, col) in cols.iter().take(n).enumerate() {
        if visible[i] {
            w[i] = u32::from(col.min);
        }
    }
    let mut used: u32 = w.iter().take(n).sum();
    for i in (0..n).rev() {
        if used <= room {
            break;
        }
        let cut = (used - room).min(w[i]);
        w[i] -= cut;
        used -= cut;
    }
    let mut spare = room.saturating_sub(used);
    while spare > 0 {
        let mut grew = false;
        for i in 0..n {
            if spare == 0 {
                break;
            }
            if visible[i] && w[i] < u32::from(cols[i].ideal) {
                w[i] += 1;
                spare -= 1;
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    while spare > 0 {
        let mut open = [false; 4];
        let mut open_count = 0u32;
        let mut weight = 0u32;
        for i in 0..n {
            if visible[i] && cols[i].flex > 0 && w[i] < u32::from(cols[i].max) {
                open[i] = true;
                open_count = open_count.saturating_add(1);
                weight = weight.saturating_add(u32::from(cols[i].flex));
            }
        }
        if open_count == 0 || weight == 0 {
            break;
        }
        let round = spare;
        let mut given = 0u32;
        for i in 0..n {
            if !open[i] || spare == 0 {
                continue;
            }
            let share = (round.saturating_mul(u32::from(cols[i].flex)) / weight)
                .max(1)
                .min(u32::from(cols[i].max) - w[i])
                .min(spare);
            w[i] += share;
            spare -= share;
            given = given.saturating_add(share);
        }
        if given == 0 {
            break;
        }
    }
    for i in 0..n {
        if spare == 0 {
            break;
        }
        if visible[i] && w[i] < u32::from(cols[i].max) {
            let give = (u32::from(cols[i].max) - w[i]).min(spare);
            w[i] += give;
            spare -= give;
        }
    }
    let mut out = [0u16; 4];
    for i in 0..n {
        out[i] = u16::try_from(w[i]).unwrap_or(u16::MAX);
    }
    out
}

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
