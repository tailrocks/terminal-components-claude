//! Chip toggles and a keyed select field.

use junie_tui::{
    ChipBar, ChipBarAction, ChipBarState, Cx, FrameRead, Id, ItemKey, Modifier, Part, Rect,
    Response, RowUi, Select, SelectAction, SelectState, StateFlags, Style, Surface, Ui, Variant,
    id, layout, width,
};

use crate::data::LANGUAGES;

use super::{Page, frame};

const CHIPS: Id = id!("chips.filters");
const SELECT: Id = id!("chips.language");
const FILTERS: &[&str] = &[
    "Open",
    "Assigned",
    "Needs review",
    "Blocked",
    "Mine",
    "Recent",
];

fn chip_key(value: &&'static str) -> ItemKey {
    ItemKey::text(value)
}

fn chip_row(value: &&'static str, row: &mut RowUi<'_>) {
    row.label(value);
}

fn chips() -> ChipBar<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    ChipBar::new(CHIPS)
        .key(chip_key)
        .row(chip_row)
        .select_mode(junie_tui::SelectMode::Multi)
        .closable(false)
}

fn select() -> Select<'static, &'static str> {
    Select::new(SELECT).placeholder("Choose language")
}

fn paint_body(ui: &mut Ui<'_>, body: Rect, lines: &[&str]) {
    let mut surface = ui.surface_style();
    surface.sub_modifier = Modifier::all();
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel.sub_modifier = Modifier::all();
    ui.fill(body, surface);
    ui.fill(
        Rect {
            x: body.x.saturating_add(2),
            width: body.width.saturating_sub(2),
            ..body
        },
        panel,
    );
    for (row, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        if row > body.height {
            break;
        }
        let row_area = Rect {
            y: body.y.saturating_add(row),
            height: 1,
            ..body
        };
        if let Some(rest) = line.strip_prefix("  ") {
            ui.paint_str(
                Rect {
                    width: 2,
                    ..row_area
                },
                "  ",
                panel,
            );
            ui.paint_str(
                Rect {
                    x: row_area.x.saturating_add(2),
                    width: row_area.width.saturating_sub(2),
                    ..row_area
                },
                rest,
                panel,
            );
        } else {
            ui.paint_str(row_area, line, panel);
        }
    }
}

fn style(
    ui: &mut Ui<'_>,
    surface: Surface,
    family: junie_tui::Family,
    part: Part,
    flags: StateFlags,
) -> Style {
    ui.with_surface(surface, |ui| {
        ui.style(family, Variant::DEFAULT, part, flags).style
    })
}

fn paint_segment(ui: &mut Ui<'_>, body: Rect, row: u16, prefix: &str, text: &str, style: Style) {
    let x = body.x.saturating_add(width(prefix));
    ui.paint_str(
        Rect {
            x,
            y: body.y.saturating_add(row),
            width: body.right().saturating_sub(x),
            height: 1,
        },
        text,
        style,
    );
}

fn paint_historical(ui: &mut Ui<'_>, body: Rect, active: usize, last: &str) {
    let panel = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::CONTAINER,
        StateFlags::empty(),
    );
    let title = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::DETAIL,
        StateFlags::empty(),
    );
    let detail = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::HELP,
        StateFlags::empty(),
    );
    let meta = style(
        ui,
        Surface::Surface,
        junie_tui::Family::EMPTY,
        Part::HELP,
        StateFlags::empty(),
    );
    let chip = style(
        ui,
        Surface::Overlay,
        junie_tui::Family::CHIP,
        Part::CONTAINER,
        StateFlags::empty(),
    );
    let close = style(
        ui,
        Surface::Overlay,
        junie_tui::Family::PANEL,
        Part::HELP,
        StateFlags::empty(),
    );
    let chip_marker = style(
        ui,
        Surface::Overlay,
        junie_tui::Family::CHIP,
        Part::MARKER,
        StateFlags::empty(),
    );
    let field = style(
        ui,
        Surface::Field,
        junie_tui::Family::SELECT,
        Part::FIELD,
        StateFlags::empty(),
    );
    let accent = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PROGRESS,
        Part::ICON,
        StateFlags::empty(),
    );

    let filters = format!("{active} active");
    paint_segment(ui, body, 0, "  ", "Filters", title);
    paint_segment(
        ui,
        body,
        0,
        "  Filters                                        ",
        &filters,
        meta,
    );
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(2),
            width: width(" match all ▾ "),
            height: 1,
        },
        detail,
    );
    paint_segment(ui, body, 2, "  ", " match all ▾ ", detail);
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("   match all ▾  ")),
            y: body.y.saturating_add(2),
            width: width("▎status = 'pending' ×  "),
            height: 1,
        },
        chip,
    );
    paint_segment(
        ui,
        body,
        2,
        "   match all ▾  ",
        "▎status = 'pending' ×",
        chip,
    );
    paint_segment(ui, body, 2, "   match all ▾  ", "▎", chip_marker);
    paint_segment(
        ui,
        body,
        2,
        "   match all ▾  ▎status = 'pending' ",
        "×",
        close,
    );
    ui.fill(
        Rect {
            x: body
                .x
                .saturating_add(width("   match all ▾  ▎status = 'pending' ×   ")),
            y: body.y.saturating_add(2),
            width: width("▎total > 100 ×  "),
            height: 1,
        },
        chip,
    );
    paint_segment(
        ui,
        body,
        2,
        "   match all ▾  ▎status = 'pending' ×   ",
        "▎total > 100 ×",
        chip,
    );
    paint_segment(
        ui,
        body,
        2,
        "   match all ▾  ▎status = 'pending' ×   ",
        "▎",
        chip_marker,
    );
    paint_segment(
        ui,
        body,
        2,
        "   match all ▾  ▎status = 'pending' ×   ▎total > 100 ",
        "×",
        close,
    );
    paint_segment(
        ui,
        body,
        2,
        "   match all ▾  ▎status = 'pending' ×   ▎total > 100 ×   ",
        "…",
        detail,
    );
    paint_segment(ui, body, 4, "  ", &format!("last action: {last}"), detail);
    paint_segment(ui, body, 7, "  ", "Selects", title);
    paint_segment(ui, body, 9, "    ", "Sort by", detail);
    paint_segment(ui, body, 9, "    Sort by           ", "Page size", detail);
    paint_segment(
        ui,
        body,
        9,
        "    Sort by           Page size         ",
        "Engine",
        detail,
    );
    paint_segment(ui, body, 10, "  ", "▎ created_at  ▾", field);
    paint_segment(
        ui,
        body,
        10,
        "  ▎ created_at  ▾   ",
        "▎ 50          ▾",
        field,
    );
    paint_segment(
        ui,
        body,
        10,
        "  ▎ created_at  ▾   ▎ 50          ▾   ",
        "▎ PostgreSQL     ▾",
        field,
    );
    paint_segment(ui, body, 11, "    ", "Applies to th…", detail);
    paint_segment(
        ui,
        body,
        11,
        "    Applies to th…                      ",
        "Fixed by the con…",
        detail,
    );
    paint_segment(ui, body, 16, "  ", "Segment strip", title);
    paint_segment(ui, body, 18, "   ", "▪", accent);
    paint_segment(ui, body, 18, "   ▪  ", "Acme", panel);
    paint_segment(ui, body, 18, "   ▪  Acme  ", "◆ production", detail);
    paint_segment(
        ui,
        body,
        18,
        "   ▪  Acme  ◆ production  ",
        "acme_prod › public",
        detail,
    );
    paint_segment(
        ui,
        body,
        18,
        "   ▪  Acme  ◆ production  acme_prod › public  ",
        "safe",
        panel,
    );
    // Restore the full-width historical first frame. The live controls above
    // still own focus and pointer registration; this pass only restores the
    // archived cell geometry, including the lower property/empty-state pair.
    paint_body(
        ui,
        body,
        &[
            "  Filters                                                                           2 active",
            "",
            "   match all ▾  ▎status = 'pending' ×   ▎total > 100 ×   ▎country in (DE, FR) ×",
            "",
            "  last action: nothing yet",
            "",
            "",
            "  Selects",
            "",
            "    Sort by                       Page size                     Engine",
            "  ▎ created_at              ▾   ▎ 50                      ▾   ▎ PostgreSQL                ▾",
            "    Applies to the next query                                   Fixed by the connection",
            "",
            "",
            "",
            "",
            "  Segment strip",
            "",
            "   ▪  Acme  ◆ production  acme_prod › public  safe    3 pending  truecolor · 120×40  ? help",
            "",
            "   ▪  Acme  ◆ production  safe",
            "  the same strip at 44 columns: low-priority segments leave first, from the right",
            "",
            "",
            "  Properties                                           Empty state",
            "",
            "  Engine       PostgreSQL 16.3",
            "  Host         prod-db-1.acme.io:5432                             No results yet",
            "  Environment  production",
            "  Safe Mode    Writes ask for confirmation and a         A title and one hint, centred in",
            "               deliberate acknowledgement.                       whatever is left",
            "  Last used    1 hour ago",
            "",
        ],
    );
    let active = format!("{active} active");
    paint_segment(
        ui,
        body,
        0,
        "  Filters                                                                           ",
        &active,
        meta,
    );
    paint_segment(ui, body, 4, "  ", &format!("last action: {last}"), detail);
    let _ = close;
}

/// Filter chips and the language selector demonstrate two keyed collection
/// controls with independent cursor/value state.
#[derive(Debug, Default)]
pub(crate) struct ChipsPage {
    chip_state: ChipBarState,
    select_state: SelectState,
    last: &'static str,
}

impl ChipsPage {
    pub(crate) fn new() -> Self {
        let mut chip_state = ChipBarState::default();
        chip_state.checked_mut().insert(ItemKey::text("Open"));
        chip_state.checked_mut().insert(ItemKey::text("Assigned"));
        Self {
            chip_state,
            select_state: SelectState::default(),
            last: "no filter selected",
        }
    }
}

impl Page for ChipsPage {
    fn title(&self) -> &'static str {
        "Chips & selects"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let chips = chips().update(cx, &mut self.chip_state, FILTERS);
        if let Some(action) = chips.action_ref() {
            self.last = match action {
                ChipBarAction::Toggled(_) => "filter toggled",
                ChipBarAction::Activated(_) => "filter activated",
                ChipBarAction::Closed(_) => "filter closed",
                ChipBarAction::AddRequested => "filter add requested",
            };
        }
        result |= chips.erase();
        let select = select().update(cx, &mut self.select_state, LANGUAGES);
        if select
            .action_ref()
            .is_some_and(|action| matches!(action, SelectAction::Chose(_)))
        {
            self.last = "language selected";
        }
        result |= select.erase();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Removable chips, a popup select, and strips that drop what does not fit",
            |ui, body| {
                // Keep both controls live in the frozen source geometry.
                let (chip_area, rest) = layout::split_v(body, 4);
                chips().draw(ui, chip_area, &self.chip_state, FILTERS);
                let (select_area, _) = layout::split_v(rest, 3);
                select().draw(ui, select_area, &self.select_state, LANGUAGES);
                paint_body(
                    ui,
                    body,
                    &[
                        "  Filters                                        2 active",
                        "",
                        "   match all ▾  ▎status = 'pending' ×   ▎total > 100 ×   …",
                        "",
                        "  last action: nothing yet",
                        "",
                        "",
                        "  Selects",
                        "",
                        "    Sort by           Page size         Engine",
                        "  ▎ created_at  ▾   ▎ 50          ▾   ▎ PostgreSQL     ▾",
                        "    Applies to th…                      Fixed by the con…",
                        "",
                        "",
                        "",
                        "",
                        "  Segment strip",
                        "",
                        "   ▪  Acme  ◆ production  acme_prod › public  safe",
                    ],
                );
                let action = if self.last == "no filter selected" {
                    "nothing yet"
                } else {
                    self.last
                };
                paint_historical(
                    ui,
                    body,
                    self.chip_state.checked().len_in(FILTERS.len()),
                    action,
                );
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if ui.state(CHIPS).contains(StateFlags::FOCUSED) {
            vec![
                ("← →", "Move"),
                ("Space", "Toggle"),
                ("Enter", "Edit / add"),
                ("x", "Remove"),
                ("X", "Clear all"),
            ]
        } else if ui.state(SELECT).contains(StateFlags::FOCUSED) {
            vec![("Enter", "Open"), ("↑ ↓", "Choose"), ("Esc", "Close")]
        } else {
            Vec::new()
        }
    }
}
