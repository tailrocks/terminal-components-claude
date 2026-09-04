//! Three scroll surfaces from the legacy scrolling page.
//!
//! Prose, a 120-row list, and a following log remain separate stateful
//! viewports. Their source lines are app-owned fixtures; TextViewport owns
//! only scroll, selection, follow-tail, and pointer capture state.

use tui_next::{
    Cx, Id, Panel, PanelKind, Rect, Response, TextViewport, Ui, ViewportAction,
    ViewportLine, ViewportState, id, layout,
};

use crate::data::{PROSE, SCROLL_ROWS};

use super::{Page, frame, lines};

const PROSE_VIEW: Id = id!("scrolling.prose");
const LIST_VIEW: Id = id!("scrolling.list");
const LOG_VIEW: Id = id!("scrolling.log");

fn prose_view() -> TextViewport<'static> {
    TextViewport::new(PROSE_VIEW).wrap(true)
}

fn list_view() -> TextViewport<'static> {
    TextViewport::new(LIST_VIEW)
}

fn log_view() -> TextViewport<'static> {
    TextViewport::new(LOG_VIEW)
}

fn rows() -> Vec<ViewportLine<'static>> {
    SCROLL_ROWS.iter().copied().map(ViewportLine::Plain).collect()
}

fn log_rows() -> Vec<ViewportLine<'static>> {
    // Keep the legacy log fixture visible without manufacturing borrowed
    // strings in every frame. The terminal page renders the full owned log.
    [
        "  0.00s  info   Resolving workspace members",
        "  0.37s  info   Fetching crates.io index",
        "  0.74s  info   Compiling proc-macro2 v1.0.86",
        "  1.11s  info   Compiling serde v1.0.210",
        "  1.48s  warn   unused import: std::fmt",
        "  1.85s  info   Compiling tokio v1.40.0",
        "  2.22s  info   Running unittests src/lib.rs",
        "  2.59s  info   test api::auth::tests::rejects_expired ... ok",
        "  2.96s  info   test db::pool::tests::reuses_connections ... ok",
        "  3.33s  error  test checkout::places_order ... FAILED",
        "  3.70s  info   test workers::scheduler::tests::respects_timezone ... ok",
        "  4.07s  info   Linking target/debug/deps/app-4f2c1b",
    ]
    .into_iter()
    .map(ViewportLine::Plain)
    .chain((0..388).map(|index| {
        // A stable repeated fixture keeps the view long enough to exercise
        // the thumb and follow-tail semantics at every capture size.
        let _ = index;
        ViewportLine::Plain("  4.44s  info   test worker::step ... ok")
    }))
    .collect()
}

/// Independent viewport state for each legacy scrolling pane.
#[derive(Debug)]
pub(crate) struct ScrollingPage {
    prose: Vec<ViewportLine<'static>>,
    list: Vec<ViewportLine<'static>>,
    log: Vec<ViewportLine<'static>>,
    prose_state: ViewportState,
    list_state: ViewportState,
    log_state: ViewportState,
    last: &'static str,
}

impl ScrollingPage {
    pub(crate) fn new() -> Self {
        let mut prose_state = ViewportState::default();
        prose_state.set_follow(false);
        let mut list_state = ViewportState::default();
        list_state.set_follow(false);
        let mut log_state = ViewportState::default();
        log_state.set_follow(true);
        Self {
            prose: PROSE.lines().map(ViewportLine::Plain).collect(),
            list: rows(),
            log: log_rows(),
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

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let prose = prose_view().update(cx, &mut self.prose_state, &self.prose);
        self.note(prose.action_ref());
        response |= prose.erase();
        let list = list_view().update(cx, &mut self.list_state, &self.list);
        self.note(list.action_ref());
        response |= list.erase();
        let log = log_view().update(cx, &mut self.log_state, &self.log);
        self.note(log.action_ref());
        response |= log.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "wheel · PageUp/PageDown · drag scrollbar · select text", |ui, body| {
            let (left, rest) = layout::split_h(body, body.width / 3);
            let (middle, right) = layout::split_h(rest, rest.width / 2);
            let panels = [
                (left, PROSE_VIEW, "Wrapped text", "wheel · selection"),
                (middle, LIST_VIEW, "Long list", "Row 001 … Row 120"),
                (right, LOG_VIEW, "Log", "following tail"),
            ];
            let prose_inner = draw_panel(ui, panels[0].0, panels[0].1, panels[0].2, panels[0].3);
            prose_view().draw(ui, prose_inner, &self.prose_state, &self.prose);
            let list_inner = draw_panel(ui, panels[1].0, panels[1].1, panels[1].2, panels[1].3);
            list_view().draw(ui, list_inner, &self.list_state, &self.list);
            let log_inner = draw_panel(ui, panels[2].0, panels[2].1, panels[2].2, panels[2].3);
            log_view().draw(ui, log_inner, &self.log_state, &self.log);
            let status = Rect { y: body.bottom().saturating_sub(1), height: 1, ..body };
            let summary = format!(
                "prose={} · list={} · log={} · {}",
                self.prose_state.scroll().offset(),
                self.list_state.scroll().offset(),
                self.log_state.scroll().offset(),
                self.last,
            );
            let _ = ui.paint_str(status, &summary, ui.surface_style());
            lines(
                ui,
                Rect { y: status.y.saturating_sub(1), height: 1, ..body },
                &["Each pane keeps its own offset and stable pointer-capture track."],
            );
        });
    }
}

fn draw_panel(
    ui: &mut Ui<'_>,
    area: Rect,
    id: Id,
    title: &'static str,
    meta: &'static str,
) -> Rect {
    let mut inner = Rect::ZERO;
    Panel::new(id)
        .kind(PanelKind::Card)
        .title(title)
        .meta(meta)
        .draw(ui, area, |_, body| inner = body);
    inner
}
