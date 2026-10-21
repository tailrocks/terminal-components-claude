//! The finder screen: the grid, the query row, the notes and the action rows.
//!
//! A finder screen is a query row, a content region under it and the notes
//! around the content. The screen owns that grid: the app never measures the
//! terminal to place the query field, the rows or the note.
//!
//! The words stay with the app. The screen owns the geometry and the ink, the
//! app owns the content and the policy.

use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_fields::{TextInput, TextInputState};
use termrock_runtime::{
    Family, FgStep, ItemKey, Modifier, Part, Role, StateFlags, StylePatch, Surface, Ui, Variant,
    truncate, truncate_middle, width,
};

/// The finder grey: the ink the finder chrome and the finder plates share.
pub const FINDER_GREY: StylePatch =
    StylePatch::new().set_fg(Role::Custom(Color::Rgb(0x8a, 0x8a, 0x8a)));

/// The grid of a finder screen.
///
/// The query row carries the search field, the content region starts under the
/// gap row and the note row is the last row of the content region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderGrid {
    top_margin: u16,
    query_rows: u16,
    query_gap: u16,
    bottom_margin: u16,
}

impl Default for FinderGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl FinderGrid {
    /// The default grid: one query row, one gap row, the content below.
    ///
    /// The screen leaves two rows at the top of the shell body and one row at
    /// the bottom for its own chrome.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            top_margin: 2,
            query_rows: 1,
            query_gap: 1,
            bottom_margin: 1,
        }
    }

    /// The rows the screen leaves at the top of the shell body.
    #[must_use]
    pub const fn top_margin(mut self, rows: u16) -> Self {
        self.top_margin = rows;
        self
    }

    /// The rows the screen leaves at the bottom of the shell body.
    #[must_use]
    pub const fn bottom_margin(mut self, rows: u16) -> Self {
        self.bottom_margin = rows;
        self
    }

    /// The area the screen draws in, inside the shell body region.
    #[must_use]
    pub fn area(&self, body: Rect) -> Rect {
        Rect::new(
            body.x,
            body.y.saturating_add(self.top_margin),
            body.width,
            body.height
                .saturating_sub(self.top_margin)
                .saturating_sub(self.bottom_margin),
        )
    }

    /// The rows the query field takes.
    #[must_use]
    pub const fn query_rows(mut self, rows: u16) -> Self {
        self.query_rows = rows;
        self
    }

    /// The rows between the query field and the content.
    #[must_use]
    pub const fn query_gap(mut self, rows: u16) -> Self {
        self.query_gap = rows;
        self
    }

    /// The query row of the area.
    #[must_use]
    pub fn query_area(&self, area: Rect) -> Rect {
        Rect::new(area.x, area.y, area.width, self.query_rows)
    }

    /// The content region of the area.
    #[must_use]
    pub fn content_area(&self, area: Rect) -> Rect {
        let top = area
            .y
            .saturating_add(self.query_rows)
            .saturating_add(self.query_gap);
        Rect::new(area.x, top, area.width, area.bottom().saturating_sub(top))
    }

    /// The content region below `rows` rows of lead.
    #[must_use]
    pub fn content_below(&self, area: Rect, rows: u16) -> Rect {
        let content = self.content_area(area);
        Rect::new(
            content.x,
            content.y.saturating_add(rows),
            content.width,
            content.height.saturating_sub(rows),
        )
    }

    /// Paint the query row: the input field and the trailing chip.
    ///
    /// The chip sits at the right edge of the row. The field takes the rest.
    pub fn draw_query(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        input: TextInput<'_>,
        state: &TextInputState,
        value: &str,
        chip: &str,
    ) {
        if area.is_empty() {
            return;
        }
        let field = ui
            .style(
                Family::INPUT,
                Variant::DEFAULT,
                Part::FIELD,
                StateFlags::empty(),
            )
            .style;
        let chip_w = width(chip);
        ui.fill(Rect::new(area.x, area.y, area.width, 1), field);
        input.value(value).draw(
            ui,
            Rect::new(area.x, area.y, area.width.saturating_sub(chip_w), 1),
            state,
        );
        let chip_style = ui.paint_patch(&FINDER_GREY).with_bg_from(field);
        ui.paint_str(
            Rect::new(
                area.right().saturating_sub(chip_w),
                area.y,
                chip_w.min(area.width),
                1,
            ),
            chip,
            chip_style,
        );
    }

    /// Paint the empty-state note on the first row of the content region.
    pub fn draw_empty(&self, ui: &mut Ui<'_>, content: Rect, text: &str, role: Role) {
        if content.is_empty() {
            return;
        }
        let style = ui.paint_patch(&StylePatch::new().set_fg(role));
        ui.paint_str(
            Rect::new(
                content.x.saturating_add(2),
                content.y,
                content.width.saturating_sub(4),
                1,
            ),
            text,
            style,
        );
    }

    /// Paint the note on the last row of the content region.
    ///
    /// The note only shows when the content leaves room for it: `occupied` is
    /// the number of rows the content takes.
    pub fn draw_note(
        &self,
        ui: &mut Ui<'_>,
        content: Rect,
        occupied: usize,
        text: &str,
        role: Role,
    ) -> bool {
        if occupied.saturating_add(1) >= usize::from(content.height) {
            return false;
        }
        let style = ui.paint_patch(&StylePatch::new().set_fg(role));
        ui.paint_str(
            Rect::new(
                content.x.saturating_add(2),
                content.bottom().saturating_sub(1),
                content.width.saturating_sub(2),
                1,
            ),
            text,
            style,
        );
        true
    }
}

/// The lead block of a finder screen: a title and two muted lines.
///
/// The lead sits above the rows of the screen. It needs three rows: a shorter
/// area carries no lead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderLead<'a> {
    /// The title of the lead.
    pub title: &'a str,
    /// The line under the title.
    pub subtitle: &'a str,
    /// The last line of the lead.
    pub hint: &'a str,
}

impl<'a> FinderLead<'a> {
    /// A lead with the three lines.
    #[must_use]
    pub const fn new(title: &'a str, subtitle: &'a str, hint: &'a str) -> Self {
        Self {
            title,
            subtitle,
            hint,
        }
    }

    /// Paint the lead. Returns the rows it used; zero when the area is short.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> u16 {
        if area.height < 3 {
            return 0;
        }
        let title = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
        let muted = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        ui.paint_str(self.line(area, 0), self.title, title);
        ui.paint_str(self.line(area, 1), self.subtitle, muted);
        ui.paint_str(self.line(area, 2), self.hint, muted);
        3
    }

    /// The row the lead paints on.
    fn line(&self, area: Rect, row: u16) -> Rect {
        Rect::new(
            area.x.saturating_add(2),
            area.y.saturating_add(row),
            area.width.saturating_sub(2),
            1,
        )
    }
}

/// The words an action row paints.
///
/// An action row shows what an action would do: its title, the warning mark
/// when it is not ready, the scope tag when it leaves the local scope and the
/// reason it carries.
pub trait ActionRow {
    /// The title of the action.
    fn title(&self) -> &str;

    /// The reason the action carries.
    fn reason(&self) -> &str;

    /// Whether the action can run as it is.
    fn ready(&self) -> bool;

    /// The scope tag of the action, when it leaves the local scope.
    fn scope_tag(&self) -> Option<String>;
}

/// Paint an action row.
///
/// The row carries the focus gutter, the title, the warning mark of an action
/// that is not ready, the scope tag and the reason.
pub fn paint_action_row<T: ActionRow>(
    ui: &mut Ui<'_>,
    area: Rect,
    flags: StateFlags,
    _key: ItemKey,
    row: &T,
) {
    let style = ui
        .style(Family::LIST, Variant::DEFAULT, Part::ROW, flags)
        .style;
    ui.fill(area, style);
    let gutter = ui
        .style(Family::LIST, Variant::DEFAULT, Part::GUTTER, flags)
        .style;
    ui.paint_str(Rect::new(area.x, area.y, 1, 1), "▎", gutter);
    let mut x = area.x.saturating_add(2);
    let title = row.title();
    let title_width = width(title);
    if x.saturating_add(title_width) < area.right() {
        ui.paint_str(Rect::new(x, area.y, title_width, 1), title, style);
        x = x.saturating_add(title_width);
    } else {
        let budget = area.right().saturating_sub(x).saturating_sub(1);
        ui.paint_str(
            Rect::new(x, area.y, budget, 1),
            &truncate(title, budget),
            style,
        );
        return;
    }
    if !row.ready() && x.saturating_add(2) < area.right() {
        let warning = ui
            .paint_patch(&StylePatch::new().set_fg(Role::Warning).add(Modifier::BOLD))
            .with_bg_from(style);
        ui.paint_str(Rect::new(x.saturating_add(1), area.y, 1, 1), "▲", warning);
        x = x.saturating_add(2);
    }
    let reason = row.reason();
    let reason_width = width(reason);
    let reason_x = if area.width > reason_width.saturating_add(24)
        && area.right() > x.saturating_add(2).saturating_add(reason_width)
    {
        area.right().saturating_sub(reason_width.saturating_add(1))
    } else {
        area.right()
    };
    if let Some(tag) = row.scope_tag() {
        let budget = reason_x.saturating_sub(x.saturating_add(3));
        if budget >= 10 {
            let faint = ui
                .paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)))
                .with_bg_from(style);
            ui.paint_str(
                Rect::new(x.saturating_add(1), area.y, budget, 1),
                &truncate_middle(&tag, budget),
                faint,
            );
        }
    }
    if reason_x < area.right() {
        let muted = ui.paint_patch(&FINDER_GREY).with_bg_from(style);
        ui.paint_str(Rect::new(reason_x, area.y, reason_width, 1), reason, muted);
    }
}

/// The label ink of the scope band: the secondary ink on black.
pub const BAND_LABEL: StylePatch = StylePatch::new()
    .set_fg(Role::Custom(Color::Rgb(0xb3, 0xb3, 0xb3)))
    .set_bg(Role::Custom(Color::Rgb(0, 0, 0)))
    .remove(Modifier::BOLD);

/// The meta ink of the scope band.
pub const BAND_META: StylePatch = StylePatch::new()
    .set_fg(Role::Custom(Color::Rgb(0x73, 0x73, 0x73)))
    .remove(Modifier::BOLD);

/// The rule ink of the scope band.
pub const BAND_RULE: StylePatch =
    StylePatch::new().set_fg(Role::Custom(Color::Rgb(0x3a, 0x3a, 0x3a)));

/// A chip of the scope band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandChip<'a> {
    /// The name of the chip.
    pub name: &'a str,
    /// The mark the chip carries.
    pub mark: &'a str,
    /// The ink of the mark.
    pub mark_fg: Color,
}

impl<'a> BandChip<'a> {
    /// A chip with the name and the mark.
    #[must_use]
    pub const fn new(name: &'a str, mark: &'a str, mark_fg: Color) -> Self {
        Self {
            name,
            mark,
            mark_fg,
        }
    }
}

/// The right marker of the scope band: the running count and the overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandMarker<'a> {
    /// The count, painted with the label ink.
    pub count: &'a str,
    /// The overflow, painted after the count.
    pub more: &'a str,
}

impl<'a> BandMarker<'a> {
    /// A marker with the count and the overflow.
    #[must_use]
    pub const fn new(count: &'a str, more: &'a str) -> Self {
        Self { count, more }
    }
}

/// The scope band of a finder screen.
///
/// The band carries the scope tab, the chips that sit on its row and the
/// marker at the right. The active tab wears the accent underline; the rest of
/// the row wears the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBand<'a> {
    /// The label of the scope tab.
    pub tab: &'a str,
    /// Whether the band draws the tab itself.
    pub draw_tab: bool,
    /// The gutter before the tab, with the text it carries.
    pub gutter: Option<&'a str>,
    /// Whether the band carries the rule under its row.
    pub rule: bool,
    /// The chips that sit on the tab row.
    pub chips: &'a [BandChip<'a>],
    /// The marker at the right of the row.
    pub marker: Option<BandMarker<'a>>,
}

impl<'a> ScopeBand<'a> {
    /// A band with the scope tab and nothing else.
    #[must_use]
    pub const fn new(tab: &'a str) -> Self {
        Self {
            tab,
            draw_tab: true,
            gutter: None,
            rule: true,
            chips: &[],
            marker: None,
        }
    }

    /// Set whether the band draws the tab itself.
    #[must_use]
    pub const fn draw_tab(mut self, draw: bool) -> Self {
        self.draw_tab = draw;
        self
    }

    /// Set the gutter before the tab.
    #[must_use]
    pub const fn gutter(mut self, gutter: &'a str) -> Self {
        self.gutter = Some(gutter);
        self
    }

    /// Set whether the band carries the rule under its row.
    #[must_use]
    pub const fn rule(mut self, rule: bool) -> Self {
        self.rule = rule;
        self
    }

    /// Set the chips that sit on the tab row.
    #[must_use]
    pub const fn chips(mut self, chips: &'a [BandChip<'a>]) -> Self {
        self.chips = chips;
        self
    }

    /// Set the marker at the right of the row.
    #[must_use]
    pub const fn marker(mut self, marker: BandMarker<'a>) -> Self {
        self.marker = Some(marker);
        self
    }

    /// Paint the band. `true` when the band painted.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> bool {
        let band_width = area.width.saturating_sub(2);
        if band_width == 0 || area.height < 4 {
            return false;
        }
        let x = area.x.saturating_add(1);
        let y = area.y.saturating_add(2);
        let gutter = self
            .gutter
            .map_or(0, |text| band_width.min(text.chars().count() as u16));
        if let Some(text) = self.gutter {
            let meta = ui.paint_patch(&BAND_META);
            ui.paint_str(Rect::new(x, y, gutter, 1), text, meta);
        }
        let tab_w = 1u16
            .saturating_add(width(self.tab))
            .saturating_add(2)
            .min(band_width.saturating_sub(gutter));
        let tab_x = x.saturating_add(gutter);
        if self.draw_tab {
            let plate = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::Surface(Surface::Overlay))
                    .add(Modifier::BOLD),
            );
            let tab = Rect::new(tab_x, y, tab_w, 1);
            ui.fill(tab, plate);
            ui.paint_str(tab, &format!(" {}", self.tab), plate);
        }
        let accent = ui.paint_patch(&StylePatch::new().set_fg(Role::Accent));
        let rule = ui.paint_patch(&BAND_RULE);
        let under = y.saturating_add(1);
        if self.rule {
            if gutter > 0 {
                ui.paint_str(
                    Rect::new(x, under, gutter, 1),
                    &"─".repeat(usize::from(gutter)),
                    rule,
                );
            }
            ui.paint_str(
                Rect::new(tab_x, under, tab_w, 1),
                &"━".repeat(usize::from(tab_w)),
                accent,
            );
            let rest = band_width.saturating_sub(gutter).saturating_sub(tab_w);
            if rest > 0 {
                ui.paint_str(
                    Rect::new(tab_x.saturating_add(tab_w), under, rest, 1),
                    &"─".repeat(usize::from(rest)),
                    rule,
                );
            }
        }
        let marker_x = area.x.saturating_add(area.width.saturating_sub(9));
        let clip = match self.marker {
            Some(_) => Rect::new(area.x, area.y, marker_x.saturating_sub(area.x), area.height),
            None => area,
        };
        let chips_x = tab_x.saturating_add(tab_w).saturating_add(1);
        self.draw_chips(ui, clip, y, chips_x);
        if let Some(marker) = self.marker {
            let label = ui.paint_patch(&BAND_LABEL);
            ui.paint_str(Rect::new(marker_x, y, 4, 1), marker.count, label);
            ui.paint_str(
                Rect::new(marker_x.saturating_add(5), y, 2, 1),
                marker.more,
                label,
            );
        }
        true
    }

    /// The chips of the band, from `x` to the right of the clip.
    fn draw_chips(&self, ui: &mut Ui<'_>, clip: Rect, y: u16, mut x: u16) {
        if self.chips.is_empty() {
            return;
        }
        let label = ui.paint_patch(&BAND_LABEL);
        for chip in self.chips {
            let text = format!(" {} ", chip.name);
            let text_w = width(&text);
            let mark_w = width(chip.mark);
            let close_w = width(" ×  ");
            let need = text_w.saturating_add(mark_w).saturating_add(close_w);
            if x.saturating_add(need) > clip.right() {
                break;
            }
            ui.paint_str(Rect::new(x, y, text_w, 1), &text, label);
            x = x.saturating_add(text_w);
            let mark_style = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Custom(chip.mark_fg))
                    .set_bg(Role::Custom(Color::Rgb(0, 0, 0)))
                    .remove(Modifier::BOLD),
            );
            ui.paint_str(Rect::new(x, y, mark_w, 1), chip.mark, mark_style);
            x = x.saturating_add(mark_w);
            ui.paint_str(Rect::new(x, y, close_w, 1), " ×  ", label);
            x = x.saturating_add(close_w).saturating_add(1);
        }
    }
}
