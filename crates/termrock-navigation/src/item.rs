use ratatui_core::style::Modifier;

use termrock_core::id::{ItemKey, Part};
use termrock_core::response::StateFlags;
use termrock_layout::layout::Track;
use termrock_layout::{ColumnSpec, solve_columns_head};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_text::measure::{grapheme_width, graphemes, middle_parts, width};
use termrock_theme::{FgStep, GlyphRole, Role, StyleDefaults, StylePatch};
use unicode_segmentation::UnicodeSegmentation;

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
    detail_col: u16,
    tag: u16,
    group: u16,
    metadata_gap: u16,
}

impl Default for ItemColumns {
    fn default() -> Self {
        Self {
            label: 0,
            detail_col: 0,
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
        // The frozen picker solves its label/detail columns with a fixed
        // two-column gap (tag `picker.rs` measure: `columns::solve(&cols,
        // row_w - 4, 2)`); the paint-time metadata gap starts at 1, so the
        // solver gap tracks it from that frozen default.
        let (label, detail_col) = solve_label(
            row_width,
            label_cap,
            detail_need,
            tag,
            group,
            metadata_gap.saturating_add(1),
        );
        Self {
            label,
            detail_col,
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
        let meta_room = cells.rect(4).width;
        // The frozen picker gives the detail `min(detail_col,
        // rx - (dx + 1))`: the solved detail column, capped one spare column
        // before the row's right edge after the tag and group widths. The
        // flex cell here also yields the trailing metadata tracks, so widen
        // the room by the metadata widths (never the configurable gap
        // before them) less the two spare columns.
        let gap_track = if self.tag > 0 || self.group > 0 {
            self.metadata_gap
        } else {
            1
        };
        let trailing = self.tag
            + if self.tag > 0 {
                if self.metadata_gap == 1 {
                    2
                } else {
                    self.metadata_gap
                }
            } else {
                0
            }
            + self.group
            + u16::from(self.group > 0);
        let room = self.detail_col.min(
            meta_room
                .saturating_add(trailing)
                .saturating_sub(gap_track)
                .saturating_sub(2),
        );
        if meta_room >= 4 {
            let mut meta = cells.cell_part_with_defaults(4, Part::META, picker_ink(FgStep::Muted));
            if !meta.authored_modifiers().contains(Modifier::BOLD) {
                meta.remove_modifier(Modifier::BOLD);
            }
            // The frozen picker elides an over-long detail to the column
            // room left of the one spare column before the row's right edge
            // with a trailing ellipsis (tag `picker.rs` paint: `room =
            // detail_col.min(rx.saturating_sub(dx + 1))`), instead of hard
            // clipping at the cell boundary. Elision writes the kept head
            // and the theme ellipsis glyph directly, so a warmed repaint
            // allocates nothing.
            if width(item.detail) > room {
                let budget = room.saturating_sub(1);
                let mut end = item.detail.len();
                let mut used = 0u16;
                for (offset, grapheme) in graphemes(item.detail) {
                    let glyph_width = grapheme_width(grapheme);
                    if used.saturating_add(glyph_width) > budget {
                        end = offset;
                        break;
                    }
                    used = used.saturating_add(glyph_width);
                }
                meta.text(&item.detail[..end]);
                meta.glyphs(GlyphRole::Ellipsis, 1);
            } else {
                meta.text_matched(item.detail, &[]);
            }
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

fn solve_label(
    row_width: u16,
    label_cap: u16,
    detail_need: u16,
    tag: u16,
    group: u16,
    solve_gap: u16,
) -> (u16, u16) {
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
    let solved = solve_columns_head(&cols[..n], row_width.saturating_sub(4), solve_gap);
    let [label, detail_col] = solved.unwrap_or([label_cap, 0]);
    (label, detail_col)
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
            // The frozen picker truncates the detail to the room left of the
            // one spare column before the row's right edge, eliding with a
            // trailing ellipsis (tag `picker.rs` paint: `room =
            // detail_col.min(rx.saturating_sub(dx + 1))`).
            let budget = row.remaining_width().saturating_sub(2);
            let line = termrock_text::truncate(item.detail, budget);
            row.meta(&line);
        }
        if !item.glyph.is_empty() {
            row.part(Part::ICON, 1).text(item.glyph);
        }
        row.label_matched(item.label, item.matched);
    }
}

/// Label overflow used by [`SemanticItemRow`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ItemLabelOverflow {
    /// Keep the start of a label and show an ellipsis at the end.
    End,
    /// Keep both ends of a label and put an ellipsis in the middle.
    #[default]
    Middle,
}

/// A semantic item row for containers that own their own gutter and row chrome.
///
/// Unlike [`ItemRow`], this adapter does not paint a gutter. Trees, lists, and
/// similar containers retain ownership of their disclosure, focus, hit region,
/// and selection geometry. Metadata is admitted only at the configured content
/// width and is laid out from the trailing edge inward.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SemanticItemRow {
    metadata_min_width: u16,
    metadata_gap: u16,
    minimum_label_width: u16,
    label_overflow: ItemLabelOverflow,
}

impl Default for SemanticItemRow {
    fn default() -> Self {
        Self {
            metadata_min_width: 44,
            metadata_gap: 2,
            minimum_label_width: 12,
            label_overflow: ItemLabelOverflow::Middle,
        }
    }
}

impl SemanticItemRow {
    pub const fn new() -> Self {
        Self {
            metadata_min_width: 44,
            metadata_gap: 2,
            minimum_label_width: 12,
            label_overflow: ItemLabelOverflow::Middle,
        }
    }

    #[must_use]
    pub const fn metadata_min_width(mut self, width: u16) -> Self {
        self.metadata_min_width = width;
        self
    }

    #[must_use]
    pub const fn metadata_gap(mut self, gap: u16) -> Self {
        self.metadata_gap = gap;
        self
    }

    #[must_use]
    pub const fn minimum_label_width(mut self, width: u16) -> Self {
        self.minimum_label_width = width;
        self
    }

    #[must_use]
    pub const fn label_overflow(mut self, overflow: ItemLabelOverflow) -> Self {
        self.label_overflow = overflow;
        self
    }
}

impl<T: AsItem> RowFn<T> for SemanticItemRow {
    fn row(&self, value: &T, row: &mut RowUi<'_>) {
        let item = value.as_item();
        let content_width = row.remaining_width();
        row.leading_part(Part::ICON, 1, 1, |cell| {
            cell.text(if item.glyph.is_empty() {
                " "
            } else {
                item.glyph
            });
        });

        if content_width >= self.metadata_min_width {
            let mut metadata = [None; 3];
            let mut budget = row
                .remaining_width()
                .saturating_sub(self.minimum_label_width);

            // Allocate from the right edge inward. This keeps the application
            // identity label's minimum width before optional metadata can grow.
            for (slot, text) in [
                (2usize, item.group.unwrap_or("")),
                (1usize, item.tag.unwrap_or("")),
                (0usize, item.detail),
            ] {
                if text.is_empty() {
                    continue;
                }
                let text_width = width(text);
                let field_budget = budget.saturating_sub(self.metadata_gap).saturating_sub(1);
                if field_budget == 0 {
                    continue;
                }
                let field_width = text_width.min(field_budget);
                metadata[slot] = Some((text, field_width));
                budget = budget
                    .saturating_sub(field_width)
                    .saturating_sub(self.metadata_gap);
            }

            for field in metadata.into_iter().rev().flatten() {
                let (text, field_width) = field;
                if width(text) > field_width {
                    let truncated = termrock_text::truncate_middle(text, field_width);
                    row.meta_with_gap(&truncated, self.metadata_gap);
                } else {
                    row.meta_with_gap(text, self.metadata_gap);
                }
            }
        }

        if self.label_overflow == ItemLabelOverflow::End {
            row.label_matched(item.label, item.matched);
            return;
        }

        let available = row.remaining_width();
        let parts = middle_parts(item.label, available);
        if parts[1].is_empty() {
            row.label_matched(item.label, item.matched);
            return;
        }

        let head_count = parts[0].graphemes(true).count();
        let tail_count = parts[2].graphemes(true).count();
        let total_count = item.label.graphemes(true).count();
        let tail_start = total_count.saturating_sub(tail_count);
        let mut matched = Vec::with_capacity(item.matched.len());
        for index in item.matched.iter().copied() {
            if index < head_count {
                matched.push(index);
            } else if index >= tail_start {
                matched.push(
                    head_count
                        .saturating_add(1)
                        .saturating_add(index - tail_start),
                );
            }
        }
        let mut label = String::with_capacity(parts.iter().map(|part| part.len()).sum());
        for part in parts {
            label.push_str(part);
        }
        row.label_matched(&label, &matched);
    }
}
