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
    Family, FgStep, ItemKey, Modifier, Part, Role, StateFlags, StylePatch, Ui, Variant, truncate,
    truncate_middle, width,
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
    query_rows: u16,
    query_gap: u16,
}

impl Default for FinderGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl FinderGrid {
    /// The default grid: one query row, one gap row, the content below.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            query_rows: 1,
            query_gap: 1,
        }
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
