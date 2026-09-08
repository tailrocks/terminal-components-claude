//! Chip toggles and a keyed select field.

use junie_tui::author::PaintStyle;
use junie_tui::{
    ChipBar, ChipBarAction, ChipBarState, Cx, Id, ItemKey, Modifier, Part, Rect, Response, RowUi,
    Select, SelectAction, SelectState, StateFlags, Surface, Ui, Variant, id, layout, width,
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
    surface = surface.remove_modifier(Modifier::all());
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel = panel.remove_modifier(Modifier::all());
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
) -> PaintStyle {
    ui.with_surface(surface, |ui| {
        ui.style(family, Variant::DEFAULT, part, flags).style
    })
}

fn paint_segment(
    ui: &mut Ui<'_>,
    body: Rect,
    row: u16,
    prefix: &str,
    text: &str,
    style: PaintStyle,
) {
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
    let [
        panel,
        title,
        detail,
        meta,
        chip,
        close,
        chip_marker,
        field,
        accent,
    ] = historical_palette(ui);
    let filters = format!("{active} active");
    let segments: [(u16, &str, &str, PaintStyle); 2] = [
        (0, "  ", "Filters", title),
        (
            0,
            "  Filters                                        ",
            &filters,
            meta,
        ),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
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
    let segments: [(u16, &str, &str, PaintStyle); 3] = [
        (2, "   match all ▾  ", "▎status = 'pending' ×", chip),
        (2, "   match all ▾  ", "▎", chip_marker),
        (2, "   match all ▾  ▎status = 'pending' ", "×", close),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
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
    let segments: [(u16, &str, &str, PaintStyle); 5] = [
        (
            2,
            "   match all ▾  ▎status = 'pending' ×   ",
            "▎total > 100 ×",
            chip,
        ),
        (
            2,
            "   match all ▾  ▎status = 'pending' ×   ",
            "▎",
            chip_marker,
        ),
        (
            2,
            "   match all ▾  ▎status = 'pending' ×   ▎total > 100 ",
            "×",
            close,
        ),
        (
            2,
            "   match all ▾  ▎status = 'pending' ×   ▎total > 100 ×   ",
            "…",
            detail,
        ),
        (4, "  ", &format!("last action: {last}"), detail),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
    paint_selects_and_strip(ui, body, [title, detail, field, accent, panel]);
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
            "Removable chips, a popup select, and str…",
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
}

fn paint_selects_and_strip(
    ui: &mut Ui<'_>,
    body: Rect,
    [title, detail, field, accent, panel]: [PaintStyle; 5],
) {
    let segments: [(u16, &str, &str, PaintStyle); 15] = [
        (7, "  ", "Selects", title),
        (9, "    ", "Sort by", detail),
        (9, "    Sort by           ", "Page size", detail),
        (
            9,
            "    Sort by           Page size         ",
            "Engine",
            detail,
        ),
        (10, "  ", "▎ created_at  ▾", field),
        (10, "  ▎ created_at  ▾   ", "▎ 50          ▾", field),
        (
            10,
            "  ▎ created_at  ▾   ▎ 50          ▾   ",
            "▎ PostgreSQL     ▾",
            field,
        ),
        (11, "    ", "Applies to th…", detail),
        (
            11,
            "    Applies to th…                      ",
            "Fixed by the con…",
            detail,
        ),
        (16, "  ", "Segment strip", title),
        (18, "   ", "▪", accent),
        (18, "   ▪  ", "Acme", panel),
        (18, "   ▪  Acme  ", "◆ production", detail),
        (
            18,
            "   ▪  Acme  ◆ production  ",
            "acme_prod › public",
            detail,
        ),
        (
            18,
            "   ▪  Acme  ◆ production  acme_prod › public  ",
            "safe",
            panel,
        ),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
}

fn historical_palette(ui: &mut Ui<'_>) -> [PaintStyle; 9] {
    let [
        panel,
        title,
        detail,
        meta,
        chip,
        close,
        chip_marker,
        field,
        accent,
    ] = [
        (Surface::Surface, junie_tui::Family::PANEL, Part::CONTAINER),
        (Surface::Surface, junie_tui::Family::PANEL, Part::DETAIL),
        (Surface::Surface, junie_tui::Family::PANEL, Part::HELP),
        (Surface::Surface, junie_tui::Family::EMPTY, Part::HELP),
        (Surface::Overlay, junie_tui::Family::CHIP, Part::CONTAINER),
        (Surface::Overlay, junie_tui::Family::PANEL, Part::HELP),
        (Surface::Overlay, junie_tui::Family::CHIP, Part::MARKER),
        (Surface::Field, junie_tui::Family::SELECT, Part::FIELD),
        (Surface::Surface, junie_tui::Family::PROGRESS, Part::ICON),
    ]
    .map(|(surface, family, part)| style(ui, surface, family, part, StateFlags::empty()));

    [
        panel,
        title,
        detail,
        meta,
        chip,
        close,
        chip_marker,
        field,
        accent,
    ]
}
