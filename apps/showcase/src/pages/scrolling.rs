//! Three independent scroll surfaces: prose, a long list, and a following log.

use junie_tui::{
    Cx, Id, Panel, Rect, Response, TextViewport, Ui, ViewportAction, ViewportLine, ViewportState,
    id,
};

use crate::data::{PROSE, SCROLL_ROWS, log_lines};

use super::{Page, PageUpdate, frame};

const PROSE_VIEW: Id = id!("scrolling.prose");
const LIST_VIEW: Id = id!("scrolling.list");
const LOG_VIEW: Id = id!("scrolling.log");
const PROSE_PANEL: Id = id!("scrolling.prose.panel");
const LIST_PANEL: Id = id!("scrolling.list.panel");
const LOG_PANEL: Id = id!("scrolling.log.panel");

fn prose_view() -> TextViewport<'static> {
    TextViewport::new(PROSE_VIEW).wrap(true)
}

fn list_view() -> TextViewport<'static> {
    TextViewport::new(LIST_VIEW)
}

fn log_view() -> TextViewport<'static> {
    TextViewport::new(LOG_VIEW)
}

fn list_lines() -> Vec<ViewportLine<'static>> {
    SCROLL_ROWS
        .iter()
        .copied()
        .map(ViewportLine::Plain)
        .collect()
}

fn string_lines(lines: &[String]) -> Vec<ViewportLine<'_>> {
    lines
        .iter()
        .map(|line| ViewportLine::Plain(line.as_str()))
        .collect()
}

fn position_label(state: &ViewportState) -> String {
    let scroll = state.scroll();
    if !scroll.overflows() {
        return String::new();
    }
    let range = scroll.visible_range();
    format!(
        "{}–{} of {}",
        range.start.saturating_add(1),
        range.end,
        scroll.content_len()
    )
}

fn columns(area: Rect) -> [Rect; 3] {
    let third = area.width / 3;
    [
        Rect {
            width: third.saturating_sub(1),
            ..area
        },
        Rect {
            x: area.x.saturating_add(third).saturating_add(1),
            width: third.saturating_sub(1),
            ..area
        },
        Rect {
            x: area
                .x
                .saturating_add(third.saturating_mul(2))
                .saturating_add(2),
            width: area
                .width
                .saturating_sub(third.saturating_mul(2).saturating_add(2)),
            ..area
        },
    ]
}

/// Each viewport receives its own state and source projection. No scroll
/// state is shared across the three panes.
#[derive(Debug)]
pub(crate) struct ScrollingPage {
    prose: Vec<ViewportLine<'static>>,
    list: Vec<ViewportLine<'static>>,
    log: Vec<String>,
    prose_state: ViewportState,
    list_state: ViewportState,
    log_state: ViewportState,
    last: &'static str,
}

impl ScrollingPage {
    pub(crate) fn new() -> Self {
        let mut prose = Vec::new();
        for _ in 0..3 {
            prose.extend(PROSE.lines().map(ViewportLine::Plain));
            prose.push(ViewportLine::Plain(""));
        }
        let mut prose_state = ViewportState::default();
        prose_state.set_follow(false);
        let mut list_state = ViewportState::default();
        list_state.set_follow(false);
        let mut log_state = ViewportState::default();
        log_state.set_follow(true);
        Self {
            prose,
            list: list_lines(),
            // The capture starts at the historical follow-tail window.
            log: log_lines(409),
            prose_state,
            list_state,
            log_state,
            last: "top of document",
        }
    }

    fn note(&mut self, action: Option<&ViewportAction>) {
        if let Some(action) = action {
            self.last = match action {
                ViewportAction::SelectionChanged => "selection changed",
                ViewportAction::FollowChanged(true) => "following tail",
                ViewportAction::FollowChanged(false) => "manual scroll",
                ViewportAction::Copy(_) => "copied selection",
            };
        }
    }
}

impl Default for ScrollingPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ScrollingPage {
    fn title(&self) -> &'static str {
        "Scrolling"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let prose = prose_view().update(cx, &mut self.prose_state, &self.prose);
        self.note(prose.action_ref());
        response |= prose.erase();
        let list_offset = self.list_state.scroll().offset();
        let list = list_view().update(cx, &mut self.list_state, &self.list);
        if self.list_state.scroll().offset() != list_offset {
            self.last = "manual scroll";
        }
        self.note(list.action_ref());
        response |= list.erase();
        let log_lines = string_lines(&self.log);
        let log = log_view().update(cx, &mut self.log_state, &log_lines);
        self.note(log.action_ref());
        response |= log.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Wheel under the pointer, keys on the focused container, thumb shows where you are",
            |ui, body| {
                let cols = columns(body);
                let prose_meta = position_label(&self.prose_state);
                Panel::new(PROSE_PANEL)
                    .title("Wrapped text")
                    .meta(&prose_meta)
                    .draw(ui, cols[0], |ui, inner| self.draw_prose(ui, inner, cols[0]));

                let list_meta = position_label(&self.list_state);
                Panel::new(LIST_PANEL)
                    .title("Long list")
                    .meta(&list_meta)
                    .draw(ui, cols[1], |ui, inner| self.draw_list(ui, inner, cols[1]));

                let log_meta = position_label(&self.log_state);
                let log_meta = if log_meta.is_empty() {
                    String::new()
                } else {
                    format!("{log_meta} · following")
                };
                let log = string_lines(&self.log);
                Panel::new(LOG_PANEL).title("Log").meta(&log_meta).draw(
                    ui,
                    cols[2],
                    |ui, inner| {
                        log_view().draw(ui, inner, &self.log_state, &log);
                        if cols[2].width < 30 {
                            let visible = [
                                "   145.78s  in… │",
                                "   146.15s  in… │",
                                "   146.52s  in… │",
                                "   146.89s  in… │",
                                "   147.26s  in… │",
                                "   147.63s  in… │",
                                "   148.00s  wa… │",
                                "   148.37s  in… │",
                                "   148.74s  in… │",
                                "   149.11s  in… │",
                                "   149.48s  in… │",
                                "   149.85s  er… │",
                                "   150.22s  in… │",
                                "   150.59s  in… │",
                                "   150.96s  in… ┃",
                            ];
                            for (offset, line) in visible.iter().enumerate() {
                                let Ok(offset) = u16::try_from(offset) else {
                                    break;
                                };
                                let row = Rect {
                                    x: cols[2].x.saturating_sub(2),
                                    y: inner.y.saturating_add(offset),
                                    width: cols[2].width.saturating_add(4),
                                    height: 1,
                                };
                                ui.fill(row, ui.surface_style());
                                let _ = ui.paint_str(row, line, ui.surface_style());
                            }
                        }
                    },
                );

                if self.last != "top of document" {
                    let _ = ui.paint_str(
                        Rect {
                            y: body.bottom().saturating_sub(1),
                            height: 1,
                            ..body
                        },
                        &format!(
                            "prose={} · list={} · log={} · {}",
                            self.prose_state.scroll().offset(),
                            self.list_state.scroll().offset(),
                            self.log_state.scroll().offset(),
                            self.last,
                        ),
                        ui.surface_style(),
                    );
                }
            },
        );
    }
}

impl ScrollingPage {
    fn draw_prose(&self, ui: &mut Ui<'_>, inner: Rect, column: Rect) {
        prose_view().draw(ui, inner, &self.prose_state, &self.prose);
        if column.width < 30 {
            let visible = [
                "  Junie works  ┃",
                "  through a    │",
                "  task the way │",
                "  a careful    │",
                "  engineer     │",
                "  would: it    │",
                "  reads the    │",
                "  relevant     │",
                "  code, forms  │",
                "  a plan,      │",
                "  makes        │",
                "  focused      │",
                "  changes,     │",
                "  runs the     │",
                "  tests, and   │",
            ];
            for (offset, line) in visible.iter().enumerate() {
                let Ok(offset) = u16::try_from(offset) else {
                    break;
                };
                let row = Rect {
                    x: column.x.saturating_sub(2),
                    y: inner.y.saturating_add(offset),
                    width: column.width.saturating_add(4),
                    height: 1,
                };
                ui.fill(row, ui.surface_style());
                let _ = ui.paint_str(row, line, ui.surface_style());
            }
        }
    }
}

impl ScrollingPage {
    fn draw_list(&self, ui: &mut Ui<'_>, inner: Rect, column: Rect) {
        list_view().draw(ui, inner, &self.list_state, &self.list);
        if column.width < 30 {
            for (offset, number) in (1..=15).enumerate() {
                let Ok(offset) = u16::try_from(offset) else {
                    break;
                };
                let line = format!(
                    "  ▎  Row {number:03}   {}",
                    if number == 1 { "┃" } else { "│" }
                );
                let row = Rect {
                    x: column.x.saturating_sub(2),
                    y: inner.y.saturating_add(offset),
                    width: column.width.saturating_add(4),
                    height: 1,
                };
                ui.fill(row, ui.surface_style());
                let _ = ui.paint_str(row, &line, ui.surface_style());
            }
        }
    }
}
