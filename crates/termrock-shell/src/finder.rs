//! The finder screen: a responsive tag plate.
//!
//! A finder plate is the body of a search screen: a list of tagged rows, the
//! chips that filter them, the preview card that explains the selected row and
//! the footer that names the action the row would run.
//!
//! The plate owns its layout. It knows the width bands, the column sizes, the
//! master/detail split, the fade at the overflowing edge and the scroll
//! affordances. An app supplies the words, the rows and a per-band policy, and
//! never measures the terminal itself.
//!
//! The policy is per band because the reference plates are: the same rows read
//! as one column on a narrow terminal and as a split card on a wide one. The
//! app names the layout for each band it wants and the plate picks the one
//! that fits.

use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_runtime::{
    Modifier, PaintStyle, Role, ScrollState, StylePatch, Ui, truncate, truncate_middle, width,
};

/// The width band a finder plate lays out for.
///
/// The bands are the reference widths of the finder screens. A terminal
/// narrower than the compact band cannot carry the plate at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderBand {
    /// Below the compact band: the plate does not fit.
    Narrow,
    /// The compact band: a single column of rows.
    Compact,
    /// The medium band: a single column or a split master/detail card.
    Medium,
    /// The wide band: the plain rows, or a split card where the policy says so.
    Wide,
}

/// The band a terminal width falls in.
///
/// The compact band starts at 80 columns and the medium band at 120. A
/// terminal narrower than the compact band carries no plate.
#[must_use]
pub const fn finder_band(columns: u16) -> FinderBand {
    if columns < 80 {
        FinderBand::Narrow
    } else if columns < 120 {
        FinderBand::Compact
    } else if columns < 160 {
        FinderBand::Medium
    } else {
        FinderBand::Wide
    }
}

/// A row of a finder plate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderRow {
    /// The row title.
    pub title: String,
    /// The kind word, such as `task` or `git`.
    pub kind: String,
    /// The scope word.
    pub scope_word: String,
    /// Whether the scope word reads as the local one.
    pub here: bool,
    /// The reason the row carries.
    pub reason: String,
    /// The risk word, when the row has one.
    pub risk: Option<String>,
    /// Whether the row is the selected one.
    pub selected: bool,
}

impl FinderRow {
    /// A row with only a title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: String::new(),
            scope_word: String::new(),
            here: true,
            reason: String::new(),
            risk: None,
            selected: false,
        }
    }

    /// Set the kind word.
    #[must_use]
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    /// Set the scope word and whether it reads as the local one.
    #[must_use]
    pub fn scope(mut self, word: impl Into<String>, here: bool) -> Self {
        self.scope_word = word.into();
        self.here = here;
        self
    }

    /// Set the reason.
    #[must_use]
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Set the risk word.
    #[must_use]
    pub fn risk(mut self, risk: impl Into<String>) -> Self {
        self.risk = Some(risk.into());
        self
    }

    /// Mark the row as the selected one.
    #[must_use]
    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// A line of a finder plate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinderLine {
    /// A section heading.
    Heading(String),
    /// An empty line.
    Blank,
    /// The filter chips of a section.
    Chips(Vec<String>),
    /// A tagged row.
    Row(FinderRow),
}

impl FinderLine {
    /// A heading line.
    #[must_use]
    pub fn heading(text: impl Into<String>) -> Self {
        Self::Heading(text.into())
    }

    /// A chip line.
    #[must_use]
    pub fn chips<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Chips(labels.into_iter().map(Into::into).collect())
    }

    /// A row line.
    #[must_use]
    pub fn row(row: FinderRow) -> Self {
        Self::Row(row)
    }
}

/// The tone of a preview value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderTone {
    /// The plain ink.
    Text,
    /// The muted ink.
    Muted,
    /// The secondary ink.
    Secondary,
    /// The meta ink.
    Meta,
    /// The danger ink.
    Danger,
}

/// A preview line of a finder plate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderPreview {
    /// The label the value sits under.
    pub label: String,
    /// The value.
    pub value: String,
    /// The tone of the value.
    pub tone: FinderTone,
    /// Whether the value wraps over several rows.
    pub wraps: bool,
}

impl FinderPreview {
    /// A preview line that does not wrap.
    #[must_use]
    pub fn new(label: impl Into<String>, value: impl Into<String>, tone: FinderTone) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            tone,
            wraps: false,
        }
    }

    /// Let the value wrap over several rows.
    #[must_use]
    pub fn wrapping(mut self) -> Self {
        self.wraps = true;
        self
    }
}

/// The content of a finder plate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FinderContent {
    /// The lines of the plate, top to bottom.
    pub lines: Vec<FinderLine>,
    /// The preview lines of the selected row.
    pub preview: Vec<FinderPreview>,
    /// The meta line the preview card carries.
    pub preview_meta: String,
}

impl FinderContent {
    /// A plate with the given lines.
    #[must_use]
    pub fn new(lines: Vec<FinderLine>) -> Self {
        Self {
            lines,
            preview: Vec::new(),
            preview_meta: String::new(),
        }
    }

    /// Set the preview lines.
    #[must_use]
    pub fn preview(mut self, preview: Vec<FinderPreview>) -> Self {
        self.preview = preview;
        self
    }

    /// Set the meta line of the preview card.
    #[must_use]
    pub fn preview_meta(mut self, meta: impl Into<String>) -> Self {
        self.preview_meta = meta.into();
        self
    }

    /// The rows of the plate.
    fn rows(&self) -> Vec<&FinderRow> {
        self.lines
            .iter()
            .filter_map(|line| match line {
                FinderLine::Row(row) => Some(row),
                _ => None,
            })
            .collect()
    }
}

/// How a finder plate sizes its columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderColumns {
    /// The frozen reference columns.
    Frozen,
    /// Columns sized from the rows and the room the plate has.
    Auto,
}

/// The title a card preview carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderPreviewTitle {
    /// The title of the selected row.
    SelectedRow,
    /// The first chip label of the plate.
    FirstChip,
}

/// The layout a finder plate uses in one band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderLayout {
    /// A single column of rows, with the footer that names the action.
    List {
        /// How the columns are sized.
        columns: FinderColumns,
    },
    /// A single column of rows whose `Runs` value is fitted to the row.
    FittedList {
        /// How the columns are sized.
        columns: FinderColumns,
    },
    /// A single column of file rows: the plain rows and the footer, no fade.
    FileList {
        /// How the columns are sized.
        columns: FinderColumns,
    },
    /// A split master/detail card.
    Card {
        /// How the columns are sized.
        columns: FinderColumns,
        /// The title the preview card carries.
        title: FinderPreviewTitle,
    },
    /// A split master/detail card whose chips keep the accent fill.
    ///
    /// The preview card of this layout carries the first chip label.
    FileCard {
        /// How the columns are sized.
        columns: FinderColumns,
    },
}

/// The per-band layout policy of a finder plate.
///
/// A band with no layout carries no plate: the plate leaves the row to the
/// next candidate. A band may name a fallback layout, which the plate tries
/// when the first one cannot paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderPolicy {
    compact: [Option<FinderLayout>; 2],
    medium: [Option<FinderLayout>; 2],
    wide: [Option<FinderLayout>; 2],
}

impl Default for FinderPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl FinderPolicy {
    /// A policy with no layout in any band.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            compact: [None, None],
            medium: [None, None],
            wide: [None, None],
        }
    }

    /// The layout the compact band uses.
    #[must_use]
    pub const fn compact(mut self, layout: FinderLayout) -> Self {
        self.compact[0] = Some(layout);
        self
    }

    /// The fallback the compact band uses when the first layout cannot paint.
    #[must_use]
    pub const fn compact_fallback(mut self, layout: FinderLayout) -> Self {
        self.compact[1] = Some(layout);
        self
    }

    /// The layout the medium band uses.
    #[must_use]
    pub const fn medium(mut self, layout: FinderLayout) -> Self {
        self.medium[0] = Some(layout);
        self
    }

    /// The fallback the medium band uses when the first layout cannot paint.
    #[must_use]
    pub const fn medium_fallback(mut self, layout: FinderLayout) -> Self {
        self.medium[1] = Some(layout);
        self
    }

    /// The layout the wide band uses.
    #[must_use]
    pub const fn wide(mut self, layout: FinderLayout) -> Self {
        self.wide[0] = Some(layout);
        self
    }

    /// The fallback the wide band uses when the first layout cannot paint.
    #[must_use]
    pub const fn wide_fallback(mut self, layout: FinderLayout) -> Self {
        self.wide[1] = Some(layout);
        self
    }

    /// The layouts of a band, in the order they are tried.
    #[must_use]
    pub fn layouts(&self, band: FinderBand) -> [Option<FinderLayout>; 2] {
        match band {
            FinderBand::Narrow => [None, None],
            FinderBand::Compact => self.compact,
            FinderBand::Medium => self.medium,
            FinderBand::Wide => self.wide,
        }
    }
}

/// The finder plate block.
///
/// The plate paints the content it is given with the layout the policy names
/// for the band the terminal is in. It reports whether it painted, so a screen
/// with several candidates can offer the next one.
#[derive(Debug, Clone, Default)]
pub struct FinderPlate {
    policy: FinderPolicy,
}

impl FinderPlate {
    /// A plate with no policy. It never paints.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the per-band layout policy.
    #[must_use]
    pub fn policy(mut self, policy: FinderPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The band the terminal is in.
    #[must_use]
    pub fn band(&self, ui: &Ui<'_>) -> FinderBand {
        finder_band(ui.full().width)
    }

    /// Paint the plate into `area`.
    ///
    /// `true` when the plate painted. A band the policy does not cover, an
    /// empty area and a plate without the preview the layout needs all leave
    /// the area untouched.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, content: &FinderContent) -> bool {
        let band = finder_band(ui.full().width);
        for layout in self.policy.layouts(band) {
            let Some(layout) = layout else { continue };
            if draw_layout(ui, area, content, layout) {
                return true;
            }
        }
        false
    }
}

/// The frozen reference columns of the finder plates.
const FROZEN_COLUMNS: TagColumns = TagColumns {
    label: 12,
    kind: 7,
    scope: 4,
    reason: 65,
};

/// The columns a finder plate paints its rows with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TagColumns {
    label: u16,
    kind: u16,
    scope: u16,
    reason: u16,
}

impl TagColumns {
    fn resolve(columns: FinderColumns, rows: &[&FinderRow], content_w: u16) -> Self {
        match columns {
            FinderColumns::Frozen => FROZEN_COLUMNS,
            FinderColumns::Auto => tag_columns(rows, content_w),
        }
    }
}

/// The columns the rows need, sized from the rows and the room the plate has.
fn tag_columns(rows: &[&FinderRow], content_w: u16) -> TagColumns {
    let max_label = rows.iter().map(|row| width(&row.title)).max().unwrap_or(14);
    let kind = rows.iter().map(|row| width(&row.kind)).max().unwrap_or(4);
    let scope = rows
        .iter()
        .map(|row| width(&row.scope_word))
        .max()
        .unwrap_or(4);
    let risk = rows
        .iter()
        .filter_map(|row| row.risk.as_deref())
        .map(width)
        .max()
        .unwrap_or(0);
    let label_cap = max_label.min(content_w.saturating_mul(36) / 100).max(14);
    let gaps = 2u16.saturating_mul(3u16.saturating_add(u16::from(risk > 0)));
    let mut label = label_cap;
    let fixed = label
        .saturating_add(kind)
        .saturating_add(scope)
        .saturating_add(risk)
        .saturating_add(gaps);
    let need = fixed.saturating_add(12);
    if need > content_w {
        label = label.saturating_sub(need.saturating_sub(content_w));
    }
    let reason = content_w.saturating_sub(
        label
            .saturating_add(kind)
            .saturating_add(scope)
            .saturating_add(risk)
            .saturating_add(gaps),
    );
    TagColumns {
        label,
        kind,
        scope,
        reason,
    }
}

/// The master and detail columns of a split card.
///
/// `None` when the area cannot carry both: the detail column needs 34 columns
/// and the master column 20.
fn split_detail(area: Rect) -> Option<(Rect, Rect)> {
    if area.width < 80 || area.height == 0 {
        return None;
    }
    let detail_w = u16::try_from(u32::from(area.width).saturating_mul(34) / 100).unwrap_or(0);
    let detail_w = detail_w.clamp(34, 48);
    let gap = 2u16;
    let master_w = area.width.saturating_sub(detail_w.saturating_add(gap));
    if master_w < 20 {
        return None;
    }
    Some((
        Rect::new(area.x, area.y, master_w, area.height),
        Rect::new(
            area.x.saturating_add(master_w).saturating_add(gap),
            area.y,
            detail_w,
            area.height,
        ),
    ))
}

fn draw_layout(ui: &mut Ui<'_>, area: Rect, content: &FinderContent, layout: FinderLayout) -> bool {
    match layout {
        FinderLayout::List { columns } => draw_tag_list(ui, area, content, columns),
        FinderLayout::FittedList { columns } => {
            if area.is_empty() {
                return false;
            }
            let mut fitted = content.clone();
            fit_narrow_runs(&mut fitted, area.width.saturating_sub(3));
            draw_tag_list(ui, area, &fitted, columns)
        }
        FinderLayout::FileList { columns } => draw_file_list(ui, area, content, columns),
        FinderLayout::Card { columns, title } => {
            draw_card(ui, area, content, columns, title, false)
        }
        FinderLayout::FileCard { columns } => draw_card(
            ui,
            area,
            content,
            columns,
            FinderPreviewTitle::FirstChip,
            true,
        ),
    }
}

/// A single column of rows with the footer that names the action.
fn draw_tag_list(
    ui: &mut Ui<'_>,
    area: Rect,
    content: &FinderContent,
    columns: FinderColumns,
) -> bool {
    if area.is_empty() {
        return false;
    }
    let Some(text) = narrow_action_line(content) else {
        return false;
    };
    let rows = content.rows();
    let footer_y = area.bottom().saturating_sub(1);
    // Short lists never reach this row. A longer list clips here instead of covering it.
    let gap_y = footer_y.saturating_sub(1);
    let slots = usize::from(gap_y.saturating_sub(area.y));
    let overflow = slots > 0 && content.lines.len() > slots;
    // The thumb owns the last column. Leave the cell before it blank, so a truncated reason stops one earlier.
    let content_w = area.width.saturating_sub(if overflow { 5 } else { 4 });
    let cols = TagColumns::resolve(columns, &rows, content_w);
    let shown = if overflow { slots } else { content.lines.len() };
    let mut y = area.y;
    for (index, line) in content.lines.iter().enumerate().take(shown) {
        if y >= gap_y {
            break;
        }
        let fade = if overflow {
            preview_fade(index, shown)
        } else {
            None
        };
        if let Some((num, den)) = fade {
            ui.fill(
                Rect::new(area.x, y, area.width.saturating_sub(1), 1),
                ink(ui, mix_on(WHITE, BLACK, num, den), BLACK, false),
            );
        }
        match line {
            FinderLine::Heading(text) => paint_tag_heading(ui, area, y, text, fade),
            FinderLine::Blank => {}
            FinderLine::Chips(labels) => paint_tag_chips(ui, area, y, labels, fade),
            FinderLine::Row(row) => paint_tag_row(ui, area, y, row, &cols, fade),
        }
        y = y.saturating_add(1);
    }
    if overflow {
        paint_narrow_scrollbar(
            ui,
            Rect::new(
                area.right().saturating_sub(1),
                area.y,
                1,
                u16::try_from(shown).unwrap_or(u16::MAX),
            ),
            content.lines.len(),
        );
    }
    ui.paint_str(
        Rect::new(
            area.x.saturating_add(3),
            footer_y,
            area.width.saturating_sub(3),
            1,
        ),
        &text,
        ink(ui, TAG_GREY_FG, BLACK, false),
    );
    true
}

/// A single column of file rows. The chips keep the accent fill and the rows
/// carry no fade.
fn draw_file_list(
    ui: &mut Ui<'_>,
    area: Rect,
    content: &FinderContent,
    columns: FinderColumns,
) -> bool {
    if area.is_empty() {
        return false;
    }
    let Some(text) = narrow_action_line(content) else {
        return false;
    };
    let rows = content.rows();
    let cols = TagColumns::resolve(columns, &rows, area.width.saturating_sub(4));
    let footer_y = area.bottom().saturating_sub(1);
    let gap_y = footer_y.saturating_sub(1);
    let mut y = area.y;
    for line in &content.lines {
        if y >= gap_y {
            break;
        }
        match line {
            FinderLine::Heading(text) => paint_tag_heading(ui, area, y, text, None),
            FinderLine::Blank => {}
            FinderLine::Chips(labels) => paint_tag_chips_selected(ui, area, y, labels),
            FinderLine::Row(row) => paint_tag_row(ui, area, y, row, &cols, None),
        }
        y = y.saturating_add(1);
    }
    ui.paint_str(
        Rect::new(
            area.x.saturating_add(3),
            footer_y,
            area.width.saturating_sub(3),
            1,
        ),
        &text,
        ink(ui, TAG_GREY_FG, BLACK, false),
    );
    true
}

/// A split master/detail card.
fn draw_card(
    ui: &mut Ui<'_>,
    area: Rect,
    content: &FinderContent,
    columns: FinderColumns,
    title: FinderPreviewTitle,
    selected_chips: bool,
) -> bool {
    let Some((master, detail)) = split_detail(area) else {
        return false;
    };
    let rows = content.rows();
    let cols = TagColumns::resolve(columns, &rows, master.width.saturating_sub(4));
    let mut y = master.y;
    for line in &content.lines {
        if y >= master.bottom() {
            break;
        }
        match line {
            FinderLine::Heading(text) => paint_tag_heading(ui, master, y, text, None),
            FinderLine::Blank => {}
            FinderLine::Chips(labels) => {
                if selected_chips {
                    paint_tag_chips_selected(ui, master, y, labels);
                } else {
                    paint_tag_chips(ui, master, y, labels, None);
                }
            }
            FinderLine::Row(row) => paint_tag_row(ui, master, y, row, &cols, None),
        }
        y = y.saturating_add(1);
    }
    let title = match title {
        FinderPreviewTitle::SelectedRow => None,
        FinderPreviewTitle::FirstChip => content.lines.iter().find_map(|line| match line {
            FinderLine::Chips(labels) => labels.first().map(String::as_str),
            _ => None,
        }),
    };
    paint_tag_preview(ui, detail, content, &rows, title);
    true
}

/// The footer line that names the action the row would run.
fn narrow_action_line(content: &FinderContent) -> Option<String> {
    let value = |label: &str| {
        content
            .preview
            .iter()
            .find(|line| line.label == label)
            .map(|line| line.value.as_str())
    };
    Some(format!(
        "→ {} · {} · {}",
        value("Runs")?,
        value("Scope")?,
        value("Risk")?
    ))
}

/// Shrink `Runs` so `→ {runs} · {scope} · {risk}` fits the action row.
fn fit_narrow_runs(content: &mut FinderContent, room: u16) {
    let scope = content
        .preview
        .iter()
        .find(|line| line.label == "Scope")
        .map(|line| line.value.clone());
    let risk = content
        .preview
        .iter()
        .find(|line| line.label == "Risk")
        .map(|line| line.value.clone());
    let (Some(scope), Some(risk)) = (scope, risk) else {
        return;
    };
    let suffix_w = width("→ ")
        .saturating_add(width(" · "))
        .saturating_add(width(&scope))
        .saturating_add(width(" · "))
        .saturating_add(width(&risk));
    let budget = room.saturating_sub(suffix_w);
    if let Some(runs) = content.preview.iter_mut().find(|line| line.label == "Runs") {
        runs.value = truncate_middle(&runs.value, budget);
    }
}

const PREVIEW_BG: Color = Color::Rgb(0x11, 0x11, 0x11);
const SELECT_BG: Color = Color::Rgb(0x2f, 0x5a, 0xa8);
const TAG_GREY_FG: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const SECONDARY_FG: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const META_FG: Color = Color::Rgb(0x73, 0x73, 0x73);
const RULE_FG: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const DANGER_FG: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const WHITE: Color = Color::Rgb(0xff, 0xff, 0xff);
const BLACK: Color = Color::Rgb(0, 0, 0);
const SCROLL_THUMB: Color = Color::Rgb(0x80, 0x80, 0x80);
const SCROLL_TRACK: Color = Color::Rgb(0x47, 0x47, 0x47);

fn ink(ui: &Ui<'_>, fg: Color, bg: Color, bold: bool) -> PaintStyle {
    let patch = StylePatch::new()
        .set_fg(Role::Custom(fg))
        .set_bg(Role::Custom(bg));
    let patch = if bold {
        patch.add(Modifier::BOLD)
    } else {
        patch.remove(Modifier::BOLD)
    };
    ui.paint_patch(&patch)
}

/// A section heading.
fn paint_tag_heading(ui: &mut Ui<'_>, master: Rect, y: u16, text: &str, fade: Option<(u32, u32)>) {
    let style = ink(ui, faded(TAG_GREY_FG, fade), BLACK, false);
    ui.paint_str(
        Rect::new(master.x.saturating_add(3), y, width(text), 1),
        text,
        style,
    );
}

/// The filter chips of a section.
fn paint_tag_chips(
    ui: &mut Ui<'_>,
    master: Rect,
    y: u16,
    labels: &[String],
    fade: Option<(u32, u32)>,
) {
    // Unfocused explore gutter is ink-on-ink, so the cell reads as an empty black column.
    ui.paint_str(
        Rect::new(master.x, y, 1, 1),
        " ",
        ink(ui, BLACK, BLACK, false),
    );
    let style = ink(ui, faded(WHITE, fade), BLACK, false);
    let mut x = master.x.saturating_add(3);
    for label in labels {
        let label_w = width(label);
        if x.saturating_add(label_w) >= master.right() {
            break;
        }
        ui.paint_str(Rect::new(x.saturating_add(1), y, label_w, 1), label, style);
        x = x.saturating_add(label_w).saturating_add(3);
    }
}

/// The selected explore chips. The gutter is accent-on-accent. `›` sits in the gap before the first chip.
fn paint_tag_chips_selected(ui: &mut Ui<'_>, master: Rect, y: u16, labels: &[String]) {
    let fill = ink(ui, WHITE, SELECT_BG, true);
    ui.fill(Rect::new(master.x, y, master.width, 1), fill);
    ui.paint_str(
        Rect::new(master.x, y, 1, 1),
        " ",
        ink(ui, SELECT_BG, SELECT_BG, true),
    );
    ui.paint_str(Rect::new(master.x.saturating_add(3), y, 1, 1), "›", fill);
    let mut x = master.x.saturating_add(3);
    for label in labels {
        let label_w = width(label);
        if x.saturating_add(label_w) >= master.right() {
            break;
        }
        ui.paint_str(Rect::new(x.saturating_add(1), y, label_w, 1), label, fill);
        x = x.saturating_add(label_w).saturating_add(3);
    }
}

/// A tagged row.
fn paint_tag_row(
    ui: &mut Ui<'_>,
    master: Rect,
    y: u16,
    row: &FinderRow,
    cols: &TagColumns,
    fade: Option<(u32, u32)>,
) {
    // A selected row keeps the solid accent. Fade is only the unselected tail.
    let fade = if row.selected { None } else { fade };
    let bg = if row.selected { SELECT_BG } else { BLACK };
    let title_style = ink(ui, faded(WHITE, fade), bg, row.selected);
    if row.selected {
        ui.fill(Rect::new(master.x, y, master.width, 1), title_style);
        ui.paint_str(
            Rect::new(master.x.saturating_add(1), y, 1, 1),
            "›",
            title_style,
        );
    }
    let mut x = master.x.saturating_add(3);
    let title = truncate(&row.title, cols.label);
    ui.paint_str(
        Rect::new(x, y, width(&title).max(1), 1),
        &title,
        title_style,
    );
    x = x.saturating_add(cols.label).saturating_add(2);
    // Tag row chrome bolds only the label. Kind, scope, reason, and risk stay plain.
    let plain = ink(ui, faded(WHITE, fade), bg, false);
    let kind_style = if row.selected {
        plain
    } else {
        ink(ui, faded(TAG_GREY_FG, fade), BLACK, false)
    };
    ui.paint_str(
        Rect::new(x, y, cols.kind, 1),
        &fit_cells(&row.kind, cols.kind),
        kind_style,
    );
    x = x.saturating_add(cols.kind).saturating_add(2);
    let scope_fg = if row.selected {
        WHITE
    } else if row.here {
        TAG_GREY_FG
    } else {
        SECONDARY_FG
    };
    let scope_style = if row.selected {
        plain
    } else {
        ink(ui, faded(scope_fg, fade), BLACK, false)
    };
    ui.paint_str(
        Rect::new(x, y, cols.scope, 1),
        &fit_cells(&row.scope_word, cols.scope),
        scope_style,
    );
    x = x.saturating_add(cols.scope).saturating_add(2);
    if cols.reason > 0 {
        let reason = truncate_sep(&row.reason, cols.reason);
        let reason_style = if row.selected {
            plain
        } else {
            ink(ui, faded(TAG_GREY_FG, fade), BLACK, false)
        };
        ui.paint_str(
            Rect::new(x, y, width(&reason).max(1), 1),
            &reason,
            reason_style,
        );
    }
    if let Some(risk) = row.risk.as_deref() {
        let risk_w = width(risk);
        let risk_x = master.right().saturating_sub(risk_w.saturating_add(1));
        let risk_style = if row.selected {
            plain
        } else {
            ink(ui, faded(SECONDARY_FG, fade), BLACK, false)
        };
        ui.paint_str(Rect::new(risk_x, y, risk_w, 1), risk, risk_style);
    }
}

/// The preview card of the selected row.
fn paint_tag_preview(
    ui: &mut Ui<'_>,
    detail: Rect,
    content: &FinderContent,
    rows: &[&FinderRow],
    title_override: Option<&str>,
) {
    if detail.width < 8 || detail.height == 0 {
        return;
    }
    let label_w = content
        .preview
        .iter()
        .map(|line| width(&line.label))
        .max()
        .unwrap_or(4)
        .saturating_add(2);
    let inner_w = detail.width.saturating_sub(4);
    let value_w = inner_w.saturating_sub(label_w.saturating_add(1));
    if value_w == 0 {
        return;
    }
    let flat = flatten_preview(&content.preview, value_w);
    let card_h = u16::try_from(flat.len())
        .unwrap_or(u16::MAX)
        .saturating_add(3)
        .min(detail.height);
    let card = Rect::new(detail.x, detail.y, detail.width, card_h);
    let canvas = ink(ui, WHITE, PREVIEW_BG, false);
    ui.fill(card, canvas);
    let inner_x = detail.x.saturating_add(2);
    let title = title_override.unwrap_or_else(|| {
        rows.iter()
            .find(|row| row.selected)
            .map(|row| row.title.as_str())
            .unwrap_or("")
    });
    let meta_w = width(&content.preview_meta);
    let title_w = inner_w.saturating_sub(meta_w).saturating_sub(1);
    let title = truncate(title, title_w);
    ui.paint_str(
        Rect::new(inner_x, detail.y, width(&title).max(1), 1),
        &title,
        ink(ui, SECONDARY_FG, PREVIEW_BG, false),
    );
    if meta_w > 0 {
        ui.paint_str(
            Rect::new(
                inner_x.saturating_add(inner_w).saturating_sub(meta_w),
                detail.y,
                meta_w,
                1,
            ),
            &content.preview_meta,
            ink(ui, META_FG, PREVIEW_BG, false),
        );
    }
    let rule_y = detail.y.saturating_add(1);
    if rule_y < card.bottom() {
        ui.paint_str(
            Rect::new(inner_x, rule_y, inner_w, 1),
            &"─".repeat(usize::from(inner_w)),
            ink(ui, RULE_FG, PREVIEW_BG, false),
        );
    }
    let slots = usize::from(card_h.saturating_sub(3));
    let shown = flat.len().min(slots);
    let overflow = flat.len() > slots && slots > 0;
    let value_x = inner_x.saturating_add(label_w);
    let content_w = inner_w.saturating_sub(u16::from(overflow));
    for (index, (label, value, tone)) in flat.iter().enumerate().take(shown) {
        let y = detail
            .y
            .saturating_add(2)
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        if y >= card.bottom() {
            break;
        }
        let fade = overflow.then(|| preview_fade(index, shown)).flatten();
        if let Some((num, den)) = fade {
            ui.fill(
                Rect::new(inner_x, y, content_w, 1),
                ink(ui, mix_color(WHITE, num, den), PREVIEW_BG, false),
            );
        }
        if !label.is_empty() {
            ui.paint_str(
                Rect::new(inner_x, y, width(label), 1),
                label,
                ink(
                    ui,
                    mix_color(TAG_GREY_FG, fade_num(fade), fade_den(fade)),
                    PREVIEW_BG,
                    false,
                ),
            );
        }
        if value.is_empty() {
            continue;
        }
        let fg = mix_color(tone_color(*tone), fade_num(fade), fade_den(fade));
        ui.paint_str(
            Rect::new(value_x, y, width(value).max(1), 1),
            value,
            ink(ui, fg, PREVIEW_BG, false),
        );
    }
    if overflow {
        paint_preview_scrollbar(
            ui,
            Rect::new(
                detail.x.saturating_add(detail.width).saturating_sub(3),
                detail.y.saturating_add(2),
                1,
                u16::try_from(shown).unwrap_or(u16::MAX),
            ),
            flat.len(),
        );
    }
}

fn fade_num(fade: Option<(u32, u32)>) -> u32 {
    fade.map_or(1, |(num, _)| num)
}

fn fade_den(fade: Option<(u32, u32)>) -> u32 {
    fade.map_or(1, |(_, den)| den)
}

fn tone_color(tone: FinderTone) -> Color {
    match tone {
        FinderTone::Text => WHITE,
        FinderTone::Muted => TAG_GREY_FG,
        FinderTone::Secondary => SECONDARY_FG,
        FinderTone::Meta => META_FG,
        FinderTone::Danger => DANGER_FG,
    }
}

/// Bottom edge of an overflowing preview. `4/5` is the inner keep, `11/20` the outer.
fn preview_fade(index: usize, shown: usize) -> Option<(u32, u32)> {
    if shown < 4 {
        return None;
    }
    let from_end = shown.saturating_sub(index);
    if from_end == 1 {
        Some((11, 20))
    } else if shown >= 12 && from_end == 2 {
        Some((4, 5))
    } else {
        None
    }
}

fn mix_color(fg: Color, num: u32, den: u32) -> Color {
    mix_on(fg, PREVIEW_BG, num, den)
}

fn mix_on(fg: Color, back: Color, num: u32, den: u32) -> Color {
    if num == den {
        return fg;
    }
    let Color::Rgb(red, green, blue) = fg else {
        return fg;
    };
    let Color::Rgb(back_r, back_g, back_b) = back else {
        return fg;
    };
    Color::Rgb(
        mix_channel(red, back_r, num, den),
        mix_channel(green, back_g, num, den),
        mix_channel(blue, back_b, num, den),
    )
}

fn faded(color: Color, fade: Option<(u32, u32)>) -> Color {
    match fade {
        Some((num, den)) => mix_on(color, BLACK, num, den),
        None => color,
    }
}

fn mix_channel(fg: u8, bg: u8, num: u32, den: u32) -> u8 {
    let den = den.max(1);
    let mixed = u32::from(bg)
        .saturating_mul(den.saturating_sub(num))
        .saturating_add(u32::from(fg).saturating_mul(num));
    let rounded = mixed.saturating_add(den / 2) / den;
    u8::try_from(rounded.min(u32::from(u8::MAX))).unwrap_or(u8::MAX)
}

/// The scrollbar of the preview card.
fn paint_preview_scrollbar(ui: &mut Ui<'_>, area: Rect, content_len: usize) {
    if area.is_empty() {
        return;
    }
    let mut scroll = ScrollState::new(content_len);
    scroll.set_viewport(usize::from(area.height));
    let (start, len) = scroll.thumb(usize::from(area.height));
    let thumb_end = start.saturating_add(len);
    for row in 0..area.height {
        let on_thumb = (start..thumb_end).contains(&usize::from(row));
        let (glyph, color) = if on_thumb {
            ("┃", SCROLL_THUMB)
        } else {
            ("┆", SCROLL_TRACK)
        };
        ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            glyph,
            ink(ui, color, PREVIEW_BG, false),
        );
    }
}

/// The scrollbar of the single-column list. Same geometry as the preview thumb,
/// on black not `#111111`.
fn paint_narrow_scrollbar(ui: &mut Ui<'_>, area: Rect, content_len: usize) {
    if area.is_empty() {
        return;
    }
    let mut scroll = ScrollState::new(content_len);
    scroll.set_viewport(usize::from(area.height));
    let (start, len) = scroll.thumb(usize::from(area.height));
    let thumb_end = start.saturating_add(len);
    for row in 0..area.height {
        let on_thumb = (start..thumb_end).contains(&usize::from(row));
        let (glyph, color) = if on_thumb {
            ("┃", SCROLL_THUMB)
        } else {
            ("┆", SCROLL_TRACK)
        };
        ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            glyph,
            ink(ui, color, BLACK, false),
        );
    }
}

fn flatten_preview(lines: &[FinderPreview], value_w: u16) -> Vec<(String, String, FinderTone)> {
    let mut flat = Vec::new();
    for line in lines {
        if line.wraps {
            let parts = wrap_words(&line.value, value_w);
            for (index, part) in parts.into_iter().enumerate() {
                flat.push((
                    if index == 0 {
                        line.label.clone()
                    } else {
                        String::new()
                    },
                    part,
                    line.tone,
                ));
            }
        } else {
            flat.push((
                line.label.clone(),
                truncate(&line.value, value_w),
                line.tone,
            ));
        }
    }
    flat
}

/// A text padded to exactly `cells` cells.
fn fit_cells(text: &str, cells: u16) -> String {
    let fitted = truncate(text, cells);
    let pad = usize::from(cells.saturating_sub(width(&fitted)));
    format!("{fitted}{}", " ".repeat(pad))
}

/// Truncate at a separator so the reason keeps its parts.
fn truncate_sep(text: &str, max: u16) -> String {
    if width(text) <= max {
        return text.to_owned();
    }
    if max < 4 {
        return truncate(text, max);
    }
    let mut best: Option<usize> = None;
    let mut pos = 0usize;
    while let Some(found) = text[pos..].find(" · ") {
        let at = pos.saturating_add(found);
        let text_width = width(text.get(..at).unwrap_or(""));
        if text_width < max && text_width.saturating_mul(3) >= max {
            best = Some(at);
        }
        if text_width.saturating_add(1) > max {
            break;
        }
        pos = at.saturating_add(3);
    }
    match best.and_then(|at| text.get(..at)) {
        Some(head) => format!("{head}…"),
        None => truncate(text, max),
    }
}

/// The words of a text, wrapped to a width.
fn wrap_words(text: &str, cells: u16) -> Vec<String> {
    let limit = usize::from(cells.max(1));
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0usize;
    for word in text.split(' ') {
        let word_w = usize::from(width(word));
        if line.is_empty() {
            if word_w <= limit {
                line = word.to_owned();
                used = word_w;
            } else {
                lines.push(truncate(word, u16::try_from(limit).unwrap_or(u16::MAX)));
            }
        } else if used.saturating_add(1).saturating_add(word_w) <= limit {
            line.push(' ');
            line.push_str(word);
            used = used.saturating_add(1).saturating_add(word_w);
        } else {
            lines.push(std::mem::take(&mut line));
            used = 0;
            if word_w <= limit {
                line = word.to_owned();
                used = word_w;
            } else {
                lines.push(truncate(word, u16::try_from(limit).unwrap_or(u16::MAX)));
            }
        }
    }
    lines.push(line);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_follow_the_reference_widths() {
        assert_eq!(finder_band(79), FinderBand::Narrow);
        assert_eq!(finder_band(80), FinderBand::Compact);
        assert_eq!(finder_band(119), FinderBand::Compact);
        assert_eq!(finder_band(120), FinderBand::Medium);
        assert_eq!(finder_band(159), FinderBand::Medium);
        assert_eq!(finder_band(160), FinderBand::Wide);
    }

    #[test]
    fn a_policy_names_the_layout_of_each_band() {
        let policy = FinderPolicy::new()
            .compact(FinderLayout::List {
                columns: FinderColumns::Auto,
            })
            .medium(FinderLayout::Card {
                columns: FinderColumns::Frozen,
                title: FinderPreviewTitle::SelectedRow,
            });
        assert_eq!(
            policy.layouts(FinderBand::Compact),
            [
                Some(FinderLayout::List {
                    columns: FinderColumns::Auto
                }),
                None
            ]
        );
        assert_eq!(
            policy.layouts(FinderBand::Medium),
            [
                Some(FinderLayout::Card {
                    columns: FinderColumns::Frozen,
                    title: FinderPreviewTitle::SelectedRow
                }),
                None
            ]
        );
        assert_eq!(policy.layouts(FinderBand::Wide), [None, None]);
        assert_eq!(policy.layouts(FinderBand::Narrow), [None, None]);
    }

    #[test]
    fn a_fallback_layout_sits_behind_the_first_one() {
        let policy = FinderPolicy::new()
            .compact(FinderLayout::List {
                columns: FinderColumns::Auto,
            })
            .compact_fallback(FinderLayout::Card {
                columns: FinderColumns::Auto,
                title: FinderPreviewTitle::SelectedRow,
            });
        let layouts = policy.layouts(FinderBand::Compact);
        assert!(layouts[0].is_some());
        assert!(layouts[1].is_some());
    }

    #[test]
    fn the_columns_of_a_plate_shrink_to_the_room_it_has() {
        let rows = [
            FinderRow::new("a very long task title").kind("task"),
            FinderRow::new("short").kind("task"),
        ];
        let refs: Vec<&FinderRow> = rows.iter().collect();
        let wide = tag_columns(&refs, 120);
        let narrow = tag_columns(&refs, 40);
        assert!(narrow.label < wide.label);
        assert_eq!(wide.label, 22);
        assert_eq!(narrow.label, 14);
        assert!(narrow.reason < wide.reason);
    }
}
