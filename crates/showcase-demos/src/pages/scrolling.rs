//! Three independent scroll surfaces: prose, a long list, and a following log.

use termrock::{
    Cx, FrameRead, Id, Panel, Rect, Response, ScrollRegion, ScrollState, StateFlags, TextViewport,
    Track, Ui, ViewportAction, ViewportLine, ViewportState, id, layout,
};

use showcase_data::{PROSE, SCROLL_ROWS, log_lines};

use super::{Page, PageUpdate, frame};

const PROSE_VIEW: Id = id!("scrolling.prose");
const LIST_VIEW: Id = id!("scrolling.list");
const LOG_VIEW: Id = id!("scrolling.log");
const PROSE_PANEL: Id = id!("scrolling.prose.panel");
const LIST_PANEL: Id = id!("scrolling.list.panel");
const LOG_PANEL: Id = id!("scrolling.log.panel");
const REGION_PANEL: Id = id!("scrolling.region.panel");
const REGION: Id = id!("scrolling.region");
const REGION_LEN: usize = 48;

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

/// The three pane cards are built once each (§13): update builds the same
/// props the draw pass renders, with the live scroll label passed in.
fn prose_panel(meta: &str) -> Panel<'_> {
    Panel::new(PROSE_PANEL).title("Wrapped text").meta(meta)
}

fn list_panel(meta: &str) -> Panel<'_> {
    Panel::new(LIST_PANEL).title("Long list").meta(meta)
}

fn log_panel(meta: &str) -> Panel<'_> {
    Panel::new(LOG_PANEL).title("Log").meta(meta)
}

/// One raw `ScrollRegion` under the three viewports: the caller paints the
/// content rows itself and the component owns only the scrollbar.
fn region() -> ScrollRegion<'static> {
    ScrollRegion::new(REGION)
}

fn region_panel(meta: &str) -> Panel<'_> {
    Panel::new(REGION_PANEL).title("Raw region").meta(meta)
}

fn region_lines() -> Vec<String> {
    (1..=REGION_LEN)
        .map(|number| format!("region row {number:03} — scroll me with the wheel"))
        .collect()
}

fn range_label(state: &ScrollState) -> String {
    if !state.overflows() {
        return format!("showing all {REGION_LEN} rows");
    }
    let range = state.visible_range();
    format!(
        "rows {}–{} of {REGION_LEN}",
        range.start.saturating_add(1),
        range.end
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
pub struct ScrollingPage {
    prose: Vec<ViewportLine<'static>>,
    list: Vec<ViewportLine<'static>>,
    log: Vec<String>,
    region: Vec<String>,
    prose_state: ViewportState,
    list_state: ViewportState,
    log_state: ViewportState,
    region_state: ScrollState,
    last: &'static str,
}

impl ScrollingPage {
    pub fn new() -> Self {
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
            region: region_lines(),
            prose_state,
            list_state,
            log_state,
            region_state: ScrollState::default(),
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
    fn seek_paused(&mut self, frame: usize) {
        self.log = log_lines(409usize.saturating_add(frame.min(10_000)));
    }

    fn title(&self) -> &'static str {
        "Scrolling"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let _ = prose_panel("");
        let _ = list_panel("");
        let _ = log_panel("");
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
        let region_offset = self.region_state.offset();
        response |= region()
            .update(cx, &mut self.region_state, REGION_LEN)
            .erase();
        if self.region_state.offset() != region_offset {
            self.last = "region scrolled";
        }
        // The update pass builds the same four pane cards draw will render (§13).
        let log_meta = position_label(&self.log_state);
        let _ = prose_panel(&position_label(&self.prose_state));
        let _ = list_panel(&position_label(&self.list_state));
        let _ = log_panel(&log_meta);
        let _ = region_panel(&range_label(&self.region_state));
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Wheel under the pointer, keys on the focused container, thumb shows where you are",
            |ui, body| {
                let strips =
                    layout::rows(body, &[Track::Flex(1), Track::Fixed(1), Track::Fixed(7)]);
                let top = strips.first().copied().unwrap_or(body);
                let cols = columns(top);
                prose_panel(&position_label(&self.prose_state))
                    .draw(ui, cols[0], |ui, inner| self.draw_prose(ui, inner));

                list_panel(&position_label(&self.list_state))
                    .draw(ui, cols[1], |ui, inner| self.draw_list(ui, inner));

                let log_meta = position_label(&self.log_state);
                let log_meta = if log_meta.is_empty() {
                    String::new()
                } else {
                    format!("{log_meta} · following")
                };
                let log = string_lines(&self.log);
                log_panel(&log_meta).draw(ui, cols[2], |ui, inner| {
                    log_view().draw(ui, inner, &self.log_state, &log);
                });

                if let Some(strip) = strips.get(2).copied() {
                    region_panel(&range_label(&self.region_state)).draw(ui, strip, |ui, inner| {
                        self.draw_region(ui, inner);
                    });
                }

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

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(LOG_VIEW).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Scroll"), ("f", "Follow"), ("G", "End")]
        } else if ui.state(LIST_VIEW).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        } else {
            &[("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        }
    }
}

impl ScrollingPage {
    /// The raw region's rows are caller-painted: the component returns the
    /// content rect and the page walks the visible range inside it.
    fn draw_region(&self, ui: &mut Ui<'_>, inner: Rect) {
        let content = region().draw(ui, inner, &self.region_state, REGION_LEN);
        let view = ScrollRegion::view(&self.region_state, content, REGION_LEN);
        for index in view.visible_range() {
            let Some(line) = self.region.get(index) else {
                break;
            };
            let Ok(offset) = u16::try_from(index.saturating_sub(view.offset())) else {
                break;
            };
            if offset >= content.height {
                break;
            }
            let row = Rect {
                y: content.y.saturating_add(offset),
                height: 1,
                ..content
            };
            let _ = ui.paint_str(row, line, ui.surface_style());
        }
        ui.scroll_edges(content, &view);
    }

    fn draw_prose(&self, ui: &mut Ui<'_>, inner: Rect) {
        prose_view().draw(ui, inner, &self.prose_state, &self.prose);
    }
}

impl ScrollingPage {
    fn draw_list(&self, ui: &mut Ui<'_>, inner: Rect) {
        list_view().draw(ui, inner, &self.list_state, &self.list);
    }
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    use termrock::{App, KeyCode, Theme};
    use termrock_test_support::Harness;

    #[test]
    fn paused_seek_bounds_the_log_fixture() {
        let mut page = ScrollingPage::new();
        page.seek_paused(12);
        assert_eq!(page.log.len(), 421);
        page.seek_paused(usize::MAX);
        assert_eq!(page.log.len(), 10_409);
    }

    struct PageApp(ScrollingPage);

    impl App for PageApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            self.0.draw(ui, full);
        }
    }

    /// FIX-005-S3: at narrow widths the log pane must show the live
    /// follow-tail, not the retired hardcoded repaint (line-394-era
    /// timestamps such as `145.78s`).
    #[test]
    fn narrow_log_row_zero_shows_live_tail() {
        const W: u16 = 72;
        const H: u16 = 24;
        let top = Rect::new(0, 2, W, H.saturating_sub(10));
        let cols = columns(top);
        assert!(
            cols[2].width < 30,
            "precondition: log column must be narrow, got {}",
            cols[2].width
        );

        let mut h = Harness::new(PageApp(ScrollingPage::new()), Theme::junie(), W, H);
        // An unbound key: runs `update` (follow jumps to the tail) and draws.
        h.key(KeyCode::Null);

        // First inner row of the log pane: frame title + blank (2) plus the
        // titled-card top inset (2).
        let row = h.row(4);
        let offset = h.app().0.log_state.scroll().offset();
        let expected = &h.app().0.log[offset];
        // `log_lines` renders `{seconds:7.2}s  ...`; the seconds fragment is
        // the stable per-line identity (ASCII, up to and including `s`).
        let head = expected.trim_start();
        let end = head.find('s').map(|i| i + 1).unwrap_or(head.len());
        let needle = &head[..end];
        assert!(
            row.contains(&needle),
            "log row 0 must show the live tail line {offset} ({needle:?}), got {row:?}"
        );
        assert!(
            !row.contains("145.78s"),
            "log row 0 must not show the retired paint-over, got {row:?}"
        );
    }
}
