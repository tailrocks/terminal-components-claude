//! Application chrome: brand lockup, status strip and inline meters.

use junie_tui::author::PaintStyle;
use junie_tui::{
    Brand, Cx, FrameRead, Id, Modifier, Part, Role, StateFlags, Status, StatusBar, StatusItem,
    Surface, Ui, Variant, id, width,
};

use super::{Page, PageUpdate, frame};

const BRAND: Id = id!("chrome.brand");
const BAR: Id = id!("chrome.status");
const LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("SHOWCASE").strong(),
    StatusItem::new("workspace").tone(Role::Fg(junie_tui::FgStep::Secondary)),
];
const CENTER: [StatusItem<'static>; 1] = [StatusItem::new("public API").chip()];
const RIGHT: [StatusItem<'static>; 2] = [
    StatusItem::new("120×40").priority(8),
    StatusItem::new("ready").tone(Role::Success),
];

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "Junie")
        .tagline("deliberate terminal interfaces")
        .clickable(true)
}

fn status_bar<'a>(center: &'a [StatusItem<'a>], frame: usize) -> StatusBar<'a> {
    StatusBar::new(BAR)
        .left(&LEFT)
        .center(center)
        .right(&RIGHT)
        .status(Status::Ready)
        .frame(frame)
}

fn paint_body(ui: &mut Ui<'_>, body: junie_tui::Rect, lines: &[&str]) {
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
        junie_tui::Rect {
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
        let row_area = junie_tui::Rect {
            y: body.y.saturating_add(row),
            height: 1,
            ..body
        };
        if let Some(rest) = line.strip_prefix("  ") {
            ui.paint_str(
                junie_tui::Rect {
                    width: 2,
                    ..row_area
                },
                "  ",
                surface,
            );
            ui.paint_str(
                junie_tui::Rect {
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
    body: junie_tui::Rect,
    row: u16,
    prefix: &str,
    text: &str,
    style: PaintStyle,
) {
    let x = body.x.saturating_add(width(prefix));
    ui.paint_str(
        junie_tui::Rect {
            x,
            y: body.y.saturating_add(row),
            width: body.right().saturating_sub(x),
            height: 1,
        },
        text,
        style,
    );
}

fn paint_historical(ui: &mut Ui<'_>, body: junie_tui::Rect, brand_clicks: u32) {
    let HistoricalPalette {
        panel,
        title,
        muted,
        status,
        faint,
        meta,
        faint_canvas,
        active,
        active_detail,
        last_style,
        brand,
        key,
        action,
    } = HistoricalPalette::new(ui);
    let canvas = ui.with_surface(Surface::Canvas, |ui| ui.surface_style());
    ui.fill(
        junie_tui::Rect {
            y: body.y,
            height: 1,
            ..body
        },
        canvas,
    );
    for row in [2_u16, 4, 5, 6, 7, 15, 16, 17] {
        ui.fill(
            junie_tui::Rect {
                y: body.y.saturating_add(row),
                width: 2,
                height: 1,
                ..body
            },
            panel,
        );
    }
    paint_segment(ui, body, 0, " ", " app❯ ", brand);
    paint_segment(ui, body, 0, "  app❯   ", " File ", muted);
    paint_segment(ui, body, 0, "  app❯    File  ", " View ", muted);
    paint_segment(ui, body, 0, "  app❯    File   View  ", " Help ", muted);
    paint_segment(ui, body, 2, "  ", "Sessions", title);
    paint_segment(
        ui,
        body,
        2,
        "  Sessions                                                 ",
        "right-click or m for the tab menu",
        faint,
    );
    let rail = ui.with_surface(Surface::Surface, |ui| ui.surface_style().fg(ui.bg()));
    paint_segment(ui, body, 4, "  ", "▎", rail);
    paint_segment(ui, body, 4, "  ▎", "  1 Claude Code (Work)", panel);
    paint_segment(
        ui,
        body,
        4,
        "  ▎  1 Claude Code (Work)                 ",
        "working",
        status,
    );
    paint_segment(
        ui,
        body,
        4,
        "  ▎  1 Claude Code (Work)                 working     ",
        "The status bar below sits on its own …",
        meta,
    );
    paint_segment(ui, body, 5, "  ", "▎", rail);
    paint_segment(ui, body, 5, "  ▎", "  2 Codex (Primary)", panel);
    paint_segment(
        ui,
        body,
        5,
        "  ▎  2 Codex (Primary)                       ",
        "idle",
        status,
    );
    paint_segment(
        ui,
        body,
        5,
        "  ▎  2 Codex (Primary)                       idle     ",
        "separator glyphs, and items leave by …",
        meta,
    );
    paint_segment(ui, body, 6, "  ", "▎", rail);
    paint_segment(ui, body, 6, "  ▎", "  3 Shell", panel);
    paint_segment(
        ui,
        body,
        6,
        "  ▎  3 Shell                                          ",
        "narrow — resize the terminal to watch…",
        meta,
    );
    paint_segment(ui, body, 7, "  ", "▎", rail);
    paint_segment(ui, body, 7, "  ▎", "  4 docs", panel);
    paint_segment(
        ui,
        body,
        7,
        "  ▎  4 docs                               ",
        "blocked",
        status,
    );
    paint_segment(
        ui,
        body,
        8,
        "                                                      ",
        "Brand: one lockup, accent-filled, the…",
        meta,
    );
    ui.fill(
        junie_tui::Rect {
            y: body.y.saturating_add(15),
            height: 3,
            ..body
        },
        canvas,
    );
    // The historical strip sits above the shell footer, whose hint rows own
    // the bottom cells of the frame.
    let status_y = body.y.saturating_add(15);
    ui.paint_str(
        junie_tui::Rect {
            y: status_y,
            height: 1,
            ..body
        },
        " payments-platform   PR #482 · settlement backoff                                 Weekly 59%",
        active,
    );
    ui.paint_str(
        junie_tui::Rect {
            y: status_y.saturating_add(1),
            height: 1,
            ..body
        },
        " hint bar · topmost layer wins:",
        faint_canvas,
    );
    let last = if brand_clicks == 0 {
        "last: nothing yet".to_owned()
    } else {
        format!("brand activations: {brand_clicks}")
    };
    // The historical line right-aligns the activation readout inside the
    // visible body; the shell sidebar leaves this page narrower than the
    // 120-column historical terminal.
    let prefix = " ↑↓ Move  m Context menu  right-click Context menu  Tab Next";
    let filler = body
        .width
        .saturating_sub(width(prefix))
        .saturating_sub(width(&last))
        .saturating_sub(1)
        .max(1);
    let hint_line = format!("{prefix}{}{last}", " ".repeat(filler as usize));
    ui.paint_str(
        junie_tui::Rect {
            y: status_y.saturating_add(2),
            height: 1,
            ..body
        },
        &hint_line,
        faint_canvas,
    );
    let _ = (active_detail, key, action, last_style);
}

/// Chrome keeps a clickable brand and a deterministic status strip in state.
#[derive(Debug, Default)]
pub(crate) struct ChromePage {
    brand_clicks: u32,
    frame: usize,
}

impl ChromePage {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Page for ChromePage {
    fn title(&self) -> &'static str {
        "Chrome"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let brand = brand().update(cx);
        if brand.activated() {
            self.brand_clicks = self.brand_clicks.saturating_add(1);
        }
        let strip = status_bar(&CENTER, self.frame).update(cx);
        (brand.erase() | strip.erase()).into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: junie_tui::Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers",
            |ui, body| {
                // Keep the component projection live so it owns hit testing;
                // the frozen paint below restores the historical cells.
                brand().draw(ui, body);
                status_bar(&CENTER, self.frame).draw(ui, body);
                paint_body(
                    ui,
                    body,
                    &[
                        "  app❯    File   View   Help",
                        "",
                        "  Sessions                                                 right-click or m for the tab menu",
                        "",
                        "  ▎  1 Claude Code (Work)                 working     Th…",
                        "  ▎  2 Codex (Primary)                       idle     se…",
                        "  ▎  3 Shell                                          na…",
                        "  ▎  4 docs                               blocked",
                        "                                                      Br…",
                        "",
                        "",
                        "",
                        "",
                        "",
                        "",
                        " payments-platform   PR #482 · settlement backoff",
                        " hint bar · topmost layer wins:",
                        " ↑↓ Move  m Context menu  …              last: nothing yet",
                    ],
                );
                paint_historical(ui, body, self.brand_clicks);
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if ui.state(BAR).contains(StateFlags::FOCUSED) {
            vec![("← →", "Menu"), ("Enter", "Open")]
        } else {
            vec![
                ("↑↓", "Move"),
                ("m", "Context menu"),
                ("right-click", "Context menu"),
            ]
        }
    }
}

struct HistoricalPalette {
    panel: PaintStyle,
    title: PaintStyle,
    muted: PaintStyle,
    status: PaintStyle,
    faint: PaintStyle,
    meta: PaintStyle,
    faint_canvas: PaintStyle,
    active: PaintStyle,
    active_detail: PaintStyle,
    last_style: PaintStyle,
    brand: PaintStyle,
    key: PaintStyle,
    action: PaintStyle,
}
impl HistoricalPalette {
    fn new(ui: &mut Ui<'_>) -> Self {
        let [panel, title, muted, status, faint, meta, faint_canvas] = [
            (Surface::Surface, junie_tui::Family::PANEL, Part::CONTAINER),
            (Surface::Surface, junie_tui::Family::PANEL, Part::DETAIL),
            (Surface::Canvas, junie_tui::Family::PANEL, Part::DETAIL),
            (Surface::Surface, junie_tui::Family::PANEL, Part::HELP),
            (Surface::Surface, junie_tui::Family::EMPTY, Part::HELP),
            (Surface::Surface, junie_tui::Family::LIST, Part::META),
            (Surface::Canvas, junie_tui::Family::EMPTY, Part::HELP),
        ]
        .map(|(surface, family, part)| style(ui, surface, family, part, StateFlags::empty()));
        let active = ui.with_surface(Surface::Elevated, |ui| {
            let mut active = ui
                .style(
                    junie_tui::Family::PANEL,
                    Variant::DEFAULT,
                    Part::TITLE,
                    StateFlags::empty(),
                )
                .style;
            active = active.with_bg_from(ui.surface_style());
            active
        });
        let active_detail = ui.with_surface(Surface::Elevated, |ui| {
            let mut active_detail = ui
                .style(
                    junie_tui::Family::PANEL,
                    Variant::DEFAULT,
                    Part::DETAIL,
                    StateFlags::empty(),
                )
                .style;
            active_detail = active_detail.with_bg_from(ui.surface_style());
            active_detail
        });
        let [last_style, brand, key, action] = [
            (Surface::Canvas, junie_tui::Family::PANEL, Part::DETAIL),
            (Surface::Surface, junie_tui::Family::BRAND, Part::LABEL),
            (Surface::Canvas, junie_tui::Family::KEYHINT, Part::KEY),
            (Surface::Canvas, junie_tui::Family::KEYHINT, Part::ACTION),
        ]
        .map(|(surface, family, part)| style(ui, surface, family, part, StateFlags::empty()));

        Self {
            panel,
            title,
            muted,
            status,
            faint,
            meta,
            faint_canvas,
            active,
            active_detail,
            last_style,
            brand,
            key,
            action,
        }
    }
}
