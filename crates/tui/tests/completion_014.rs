//! TASK-014: one scroll and fade primitive preserves hidden-edge cells and routing.
//!
//! The shared [`ScrollState`](junie_tui::ScrollState) model, the owner-keyed
//! [`ScrollRegion`](junie_tui::ScrollRegion) and the reusable
//! [`Ui::scroll_edges`](junie_tui::Ui::scroll_edges) painter keep exact
//! oracle fade arithmetic, thumb track/grab capture and boundary wheel
//! routing: hidden bars reserve no column and expose no pointer part while
//! wheel and reveal stay active.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines
    )
)]

use junie_tui::{
    App, Axis, Color, ColorLevel, Cx, Id, KeyCode, Modifier, MouseKind, Part, PartRef, Position,
    Rect, Response, ScrollRegion, ScrollState, Style, Theme, Ui,
};
use junie_tui_testing::{Harness, Scene};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn themes() -> [Theme; 2] {
    [Theme::junie(), Theme::paper()]
}

const BG: Color = Color::Rgb(20, 40, 60);
const FG: Color = Color::Rgb(120, 160, 200);
const OUTER: Color = Color::Rgb(75, 106, 137);
const INNER: Color = Color::Rgb(100, 136, 172);

fn state(offset: usize, content: usize, viewport: usize) -> ScrollState {
    let mut st = ScrollState::new(content);
    st.set_viewport(viewport);
    st.scroll_to(offset);
    st
}

fn paint_rows(ui: &mut Ui<'_>, area: Rect) {
    for y in area.y..area.bottom() {
        ui.paint_str(
            Rect::new(area.x, y, area.width, 1),
            "row-content-here",
            Style::new().fg(FG).bg(BG),
        );
    }
}

fn fg_at(buf: &junie_tui::Buffer, x: u16, y: u16) -> Color {
    buf.cell(Position::new(x, y)).map(|c| c.fg).unwrap()
}

/// Heights 3/4/11/12 across fits/top/middle/bottom: below four rows nothing
/// fades, four through eleven fade one outer row per hidden edge at 55%,
/// twelve add the inner row at 80%. Both themes, every capability.
#[test]
fn fade_heights_states_themes_and_levels_match_oracle() {
    for theme in themes() {
        for level in LEVELS {
            for rows in [3u16, 4, 11, 12] {
                let h = usize::from(rows);
                for (name, st) in [
                    ("fits", state(0, h, h)),
                    ("top", state(0, 100, h)),
                    ("middle", state(50, 100, h)),
                    ("bottom", state(100, 100, h)),
                ] {
                    let mut scene = Scene::new("fade_matrix", theme.clone(), level, 16, rows);
                    scene.draw(|ui, area| {
                        paint_rows(ui, area);
                        ui.scroll_edges(area, &st);
                    });
                    let buf = scene.buffer();
                    let up = rows >= 4 && name != "fits" && name != "top";
                    let down = rows >= 4 && name != "fits" && name != "bottom";
                    let deep = rows >= 12;
                    for y in 0..rows {
                        let want = if (y == 0 && up) || (y == rows - 1 && down) {
                            OUTER
                        } else if deep && ((y == 1 && up) || (y == rows - 2 && down)) {
                            INNER
                        } else {
                            FG
                        };
                        assert_eq!(
                            fg_at(buf, 0, y),
                            want,
                            "{name} rows={rows} y={y} level={level:?}"
                        );
                    }
                }
            }
        }
    }
}

/// Majority-background ties resolve to the last majority plane; reversed
/// and different-background cells are excluded from the fade.
#[test]
fn fade_background_majority_ties_and_exclusions_match_oracle() {
    for theme in themes() {
        // tied vote: top half BG, bottom half OTHER, equal counts. The
        // last majority wins, so the top outer row stays whole and the
        // bottom outer row fades toward OTHER.
        let other = Color::Rgb(10, 10, 40);
        let other_outer = Color::Rgb(71, 93, 128);
        let mut scene = Scene::new("fade_tie", theme.clone(), ColorLevel::TrueColor, 16, 4);
        scene.draw(|ui, area| {
            for y in 0..4 {
                let bg = if y < 2 { BG } else { other };
                ui.paint_str(
                    Rect::new(0, y, 16, 1),
                    "row-content-here!",
                    Style::new().fg(FG).bg(bg),
                );
            }
            ui.scroll_edges(area, &state(10, 30, 4));
        });
        assert_eq!(fg_at(scene.buffer(), 0, 0), FG, "tied-away row stays whole");
        assert_eq!(
            fg_at(scene.buffer(), 0, 3),
            other_outer,
            "last majority fades"
        );
        // reversed and different-background cells stay whole inside a faded row.
        let mut scene = Scene::new("fade_excl", theme, ColorLevel::TrueColor, 16, 6);
        scene.draw(|ui, area| {
            paint_rows(ui, area);
            ui.paint_str(
                Rect::new(0, 5, 2, 1),
                "ab",
                Style::new().fg(FG).bg(BG).add_modifier(Modifier::REVERSED),
            );
            ui.paint_str(
                Rect::new(2, 5, 1, 1),
                "c",
                Style::new().fg(FG).bg(Color::Rgb(40, 40, 0)),
            );
            ui.scroll_edges(area, &state(10, 30, 6));
        });
        let buf = scene.buffer();
        assert_eq!(fg_at(buf, 0, 5), FG, "reversed cell untouched");
        assert_eq!(fg_at(buf, 2, 5), FG, "marked cell untouched");
        assert_eq!(fg_at(buf, 3, 5), OUTER, "plain neighbour fades");
    }
}

/// Palettes without RGB cannot blend: the outermost row dims with `DIM`,
/// the inner row stays, and no colour is invented. Every capability.
#[test]
fn fade_non_rgb_dims_outer_row_only_at_every_level() {
    for theme in themes() {
        for level in LEVELS {
            for (fg, bg) in [
                (Color::Gray, Color::Black),
                (Color::Rgb(200, 200, 200), Color::Black),
                (Color::Gray, Color::Rgb(0, 0, 0)),
            ] {
                let mut scene = Scene::new("fade_dim", theme.clone(), level, 10, 12);
                scene.draw(|ui, area| {
                    for y in 0..12 {
                        ui.paint_str(
                            Rect::new(0, y, 10, 1),
                            "abcdefghij",
                            Style::new().fg(fg).bg(bg),
                        );
                    }
                    ui.scroll_edges(area, &state(5, 100, 12));
                });
                let buf = scene.buffer();
                let cell = |y: u16| buf.cell(Position::new(0, y)).cloned().unwrap_or_default();
                assert!(cell(0).modifier.contains(Modifier::DIM), "{fg:?} on {bg:?}");
                assert!(
                    !cell(1).modifier.contains(Modifier::DIM),
                    "{fg:?} on {bg:?}: inner row stays"
                );
                assert!(
                    cell(11).modifier.contains(Modifier::DIM),
                    "{fg:?} on {bg:?}"
                );
                assert_eq!(cell(0).fg, fg, "no colour invented");
                assert_eq!(cell(0).bg, bg, "no colour invented");
            }
        }
    }
}

/// Cursor, keep, selected, hovered and marked rows stay whole while the
/// plain neighbours on the same fade rows fade.
#[test]
fn fade_protects_cursor_keep_selected_hovered_and_marked_rows() {
    const CURSOR_ID: Id = Id::root("fade.protect");
    for theme in themes() {
        let mut scene = Scene::new("fade_protect", theme, ColorLevel::TrueColor, 16, 12);
        scene.draw(|ui, area| {
            paint_rows(ui, area);
            // selected row on its own plane (top outer fade row).
            ui.paint_str(
                Rect::new(0, 0, 16, 1),
                "row-content-here!",
                Style::new().fg(FG).bg(Color::Rgb(20, 60, 20)),
            );
            // emulated hover plane and marked cell on the bottom outer row.
            ui.paint_str(
                Rect::new(0, 11, 8, 1),
                "row-cont",
                Style::new().fg(FG).bg(Color::Rgb(60, 20, 60)),
            );
            ui.paint_str(
                Rect::new(8, 11, 1, 1),
                "e",
                Style::new().fg(FG).bg(Color::Rgb(40, 40, 0)),
            );
            // hardware cursor on the bottom inner fade row.
            ui.set_cursor(CURSOR_ID, Position::new(4, 10));
            // kept current row on the top inner fade row.
            ui.scroll_edges_except(area, &state(50, 100, 12), &[1]);
        });
        let buf = scene.buffer();
        assert_eq!(fg_at(buf, 0, 0), FG, "selected row untouched");
        assert_eq!(fg_at(buf, 0, 11), FG, "hovered plane untouched");
        assert_eq!(fg_at(buf, 8, 11), FG, "marked cell untouched");
        assert_eq!(fg_at(buf, 9, 11), OUTER, "plain neighbour fades");
        assert_eq!(fg_at(buf, 0, 10), FG, "cursor row untouched");
        assert_eq!(fg_at(buf, 0, 1), FG, "kept row untouched");
    }
}

/// Headers, footers and the scrollbar column are outside the content rect
/// and never fade, with content painted before the bar.
#[test]
fn fade_skips_headers_footers_and_bar_column() {
    const ID: Id = Id::root("fade.chrome");
    for theme in themes() {
        let paint = |fade: bool| {
            let mut scene = Scene::new("fade_chrome", theme.clone(), ColorLevel::TrueColor, 12, 8);
            scene.draw(|ui, area| {
                let st = state(10, 100, 6);
                let content = ScrollRegion::new(ID).draw(ui, area, &st, 100);
                // content inset by a header row and a footer row.
                let rows = Rect::new(content.x, content.y + 1, content.width, 6);
                ui.paint_str(
                    Rect::new(content.x, content.y, content.width, 1),
                    "header-----",
                    Style::new().fg(FG).bg(BG),
                );
                paint_rows(ui, rows);
                ui.paint_str(
                    Rect::new(content.x, content.bottom() - 1, content.width, 1),
                    "footer-----",
                    Style::new().fg(FG).bg(BG),
                );
                if fade {
                    ui.scroll_edges(rows, &st);
                }
            });
            scene
        };
        let plain = paint(false);
        let faded = paint(true);
        let (plain_buf, faded_buf) = (plain.buffer(), faded.buffer());
        // header row, footer row and bar column are byte-identical.
        for x in 0..11 {
            for y in [0u16, 7] {
                assert_eq!(
                    faded_buf.cell(Position::new(x, y)),
                    plain_buf.cell(Position::new(x, y)),
                    "chrome at ({x}, {y})"
                );
            }
        }
        for y in 0..8 {
            assert_eq!(
                faded_buf.cell(Position::new(11, y)),
                plain_buf.cell(Position::new(11, y)),
                "bar column at row {y}"
            );
        }
        // the content edges fade.
        assert_eq!(fg_at(faded_buf, 0, 1), OUTER, "top content edge fades");
        assert_eq!(fg_at(faded_buf, 0, 6), OUTER, "bottom content edge fades");
        assert_eq!(fg_at(faded_buf, 0, 3), FG, "content middle stays");
    }
}

/// NEGATIVE: the fade blends foregrounds only. Glyphs, backgrounds, hit
/// areas and selection visuals are identical with the fade on or off.
#[test]
fn fade_never_changes_glyphs_backgrounds_hits_or_selection() {
    const ID: Id = Id::root("fade.negative");
    struct Page {
        fade: bool,
    }
    impl App for Page {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            ScrollRegion::new(ID)
                .update(cx, &mut ScrollState::new(100), 100)
                .erase()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let area = Rect::new(0, 0, 12, 8);
            let st = state(10, 100, 8);
            let content = ScrollRegion::new(ID).draw(ui, area, &st, 100);
            paint_rows(ui, content);
            ui.paint_str(
                Rect::new(content.x, 3, content.width, 1),
                "selected-row!",
                Style::new().fg(FG).bg(Color::Rgb(20, 60, 20)),
            );
            if self.fade {
                ui.scroll_edges(content, &st);
            }
        }
    }
    let run = |fade: bool| {
        let mut h = Harness::new(Page { fade }, Theme::junie(), 12, 8);
        h.draw();
        (
            h.buffer().clone(),
            h.area_of_part(ID, PartRef::of(Part::CONTAINER)),
            h.area_of_part(ID, PartRef::of(Part::TRACK)),
            h.area_of_part(ID, PartRef::of(Part::THUMB)),
            h.text(),
        )
    };
    let plain = run(false);
    let faded = run(true);
    assert_eq!(plain.1, faded.1, "container hits unchanged");
    assert_eq!(plain.2, faded.2, "track hits unchanged");
    assert_eq!(plain.3, faded.3, "thumb hits unchanged");
    assert_eq!(plain.4, faded.4, "glyphs unchanged");
    for y in 0..8 {
        for x in 0..12 {
            let pos = Position::new(x, y);
            let a = plain.0.cell(pos).cloned().unwrap_or_default();
            let b = faded.0.cell(pos).cloned().unwrap_or_default();
            assert_eq!(a.symbol(), b.symbol(), "glyph at ({x}, {y})");
            assert_eq!(a.bg, b.bg, "background at ({x}, {y})");
            let added = b.modifier.difference(a.modifier);
            assert!(
                added == Modifier::empty() || added == Modifier::DIM,
                "only DIM may be added at ({x}, {y})"
            );
        }
    }
    // the selected row is byte-identical: selection visuals never dim.
    for x in 0..11 {
        assert_eq!(
            faded.0.cell(Position::new(x, 3)),
            plain.0.cell(Position::new(x, 3)),
            "selected cell ({x}, 3)"
        );
    }
}

/// Clipping bounds the fade: sentinel cells outside the content rect stay
/// byte-identical, and an over-tall area cannot escape the buffer.
#[test]
fn fade_clipping_bounds_to_visible_content() {
    for theme in themes() {
        let mut scene = Scene::new("fade_clip", theme, ColorLevel::TrueColor, 12, 6);
        let sentinel = Style::new().fg(Color::Yellow).bg(Color::Blue);
        scene.draw_over(
            |buf| {
                for y in 0..6 {
                    buf.set_string(0, y, "ssssssssssss", sentinel);
                }
            },
            |ui, _| {
                let content = Rect::new(2, 0, 6, 6);
                paint_rows(ui, content);
                // an area taller than the buffer and wider than the content.
                ui.scroll_edges(Rect::new(0, 0, 12, 40), &state(10, 100, 6));
            },
        );
        let buf = scene.buffer();
        for y in 0..6 {
            for x in [0u16, 1, 8, 9, 10, 11] {
                let c = buf.cell(Position::new(x, y)).cloned().unwrap_or_default();
                assert_eq!(c.symbol(), "s", "sentinel glyph at ({x}, {y})");
                assert_eq!(c.fg, Color::Yellow, "sentinel fg at ({x}, {y})");
                assert_eq!(c.bg, Color::Blue, "sentinel bg at ({x}, {y})");
            }
        }
        assert_eq!(fg_at(buf, 2, 0), OUTER, "content edge still fades");
        assert_eq!(fg_at(buf, 2, 5), OUTER, "content edge still fades");
    }
}

struct FadeResizePage;
impl App for FadeResizePage {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let view = ScrollRegion::view(&state(5, 100, 0), area, 100);
        paint_rows(ui, area);
        ui.scroll_edges(area, &view);
    }
}

/// Resizing across the twelve-row threshold flips the fade depth: eleven
/// rows fade one row per edge, twelve fade two.
#[test]
fn fade_depth_flips_at_twelve_rows_on_resize() {
    let mut h = Harness::new(FadeResizePage, Theme::junie(), 16, 11);
    h.draw();
    assert_eq!(fg_at(h.buffer(), 0, 0), OUTER);
    assert_eq!(fg_at(h.buffer(), 0, 1), FG, "eleven rows keep a single row");
    let _ = h.resize(16, 12);
    h.draw();
    assert_eq!(fg_at(h.buffer(), 0, 0), OUTER);
    assert_eq!(
        fg_at(h.buffer(), 0, 1),
        INNER,
        "twelve rows fade the inner row"
    );
    assert_eq!(fg_at(h.buffer(), 0, 10), INNER);
    assert_eq!(fg_at(h.buffer(), 0, 11), OUTER);
    let _ = h.resize(16, 4);
    h.draw();
    assert_eq!(fg_at(h.buffer(), 0, 0), OUTER);
    assert_eq!(fg_at(h.buffer(), 0, 1), FG, "four rows keep a single row");
}

struct DragPage {
    scroll: ScrollState,
    content: usize,
    captured: bool,
    foreign_capture: bool,
}
const DRAG_ID: Id = Id::root("fade.drag");
impl DragPage {
    fn new(content: usize) -> Self {
        DragPage {
            scroll: ScrollState::new(content),
            content,
            captured: false,
            foreign_capture: false,
        }
    }
}
impl App for DragPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let content = self.content;
        let r = ScrollRegion::new(DRAG_ID).update(cx, &mut self.scroll, content);
        self.captured = cx.capture_owner() == Some(DRAG_ID);
        self.foreign_capture = cx.capture_owner().is_some_and(|o| o != DRAG_ID);
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ScrollRegion::new(DRAG_ID).draw(ui, Rect::new(2, 2, 5, 10), &self.scroll, self.content);
    }
}

/// A press on the bare track claims no capture; the following drag falls
/// back to press semantics so the thumb follows the pointer and saturates
/// at both endpoints instead of snapping.
#[test]
fn thumb_drag_from_bare_track_follows_pointer_and_saturates() {
    let mut h = Harness::new(DragPage::new(100), Theme::junie(), 10, 15);
    let _ = h.click(6, 11);
    assert_eq!(h.app().scroll.offset(), 90);
    // bare-track press jumps without capturing, then the drag follows.
    let _ = h.mouse(MouseKind::Down, 6, 2);
    assert_eq!(h.app().scroll.offset(), 0);
    assert!(!h.app().captured, "a track press claims no capture");
    let _ = h.mouse(MouseKind::Drag, 6, 11);
    assert_eq!(h.app().scroll.offset(), 90, "drag follows to the end");
    assert!(!h.app().captured);
    let _ = h.mouse(MouseKind::Up, 6, 11);
    // and back to the top through the same fallback.
    let _ = h.mouse(MouseKind::Down, 6, 11);
    assert_eq!(h.app().scroll.offset(), 90);
    let _ = h.mouse(MouseKind::Drag, 6, 2);
    assert_eq!(h.app().scroll.offset(), 0, "drag follows to the start");
    let _ = h.mouse(MouseKind::Up, 6, 2);
    // intermediate motion follows row by row instead of snapping.
    let _ = h.mouse(MouseKind::Down, 6, 5);
    assert_eq!(h.app().scroll.offset(), 30);
    let _ = h.mouse(MouseKind::Drag, 6, 6);
    assert_eq!(h.app().scroll.offset(), 40, "one row follows one row");
    let _ = h.mouse(MouseKind::Up, 6, 6);
    assert!(!h.app().foreign_capture, "exactly one capture owner");
    assert!(h.diagnostics().is_empty());
}

/// A press on the thumb grabs it: the view does not move, one row of
/// pointer motion moves the thumb one row, and the region holds the only
/// capture until release.
#[test]
fn thumb_drag_from_thumb_retains_grab_offset_and_single_capture() {
    let mut h = Harness::new(DragPage::new(30), Theme::junie(), 10, 15);
    // a press inside the thumb grabs without moving the view. The first
    // update applies the drawn viewport, so the thumb reads (0, 3) after.
    let _ = h.mouse(MouseKind::Down, 6, 3);
    assert_eq!(h.app().scroll.thumb(10), (0, 3));
    assert_eq!(h.app().scroll.offset(), 0);
    assert!(h.app().captured, "the thumb holds the capture");
    // one row of motion moves the thumb one row, not a jump.
    let _ = h.mouse(MouseKind::Drag, 6, 4);
    assert_eq!(h.app().scroll.offset(), 3);
    assert_eq!(h.app().scroll.thumb(10).0, 1, "grab offset retained");
    assert!(h.app().captured);
    // dragging to the bottom endpoint reaches the end exactly.
    let _ = h.mouse(MouseKind::Drag, 6, 11);
    assert_eq!(h.app().scroll.offset(), 20);
    assert!(h.app().scroll.at_end());
    let _ = h.mouse(MouseKind::Up, 6, 11);
    assert!(!h.app().captured, "release ends the capture");
    assert!(!h.app().foreign_capture, "exactly one capture owner");
    assert!(h.diagnostics().is_empty());
}

const OUTER_ID: Id = Id::root("fade.nested.outer");
const INNER_ID: Id = Id::root("fade.nested.inner");
struct NestedPage {
    outer: ScrollState,
    inner: ScrollState,
}
impl App for NestedPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = ScrollRegion::new(OUTER_ID).update(cx, &mut self.outer, 100);
        r |= ScrollRegion::new(INNER_ID).update(cx, &mut self.inner, 50);
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let outer = ScrollRegion::new(OUTER_ID).draw(ui, Rect::new(0, 0, 20, 12), &self.outer, 100);
        ScrollRegion::new(INNER_ID).draw(ui, Rect::new(2, 2, 10, 8), &self.inner, 50);
        let _ = outer;
    }
}

/// A boundary wheel over the nested owner is consumed there: no parent
/// chaining, no repaint, no reveal, no focus move. Motion still routes to
/// the nearest eligible owner.
#[test]
fn boundary_wheel_consumes_without_chaining_repaint_or_focus_move() {
    let mut h = Harness::new(
        NestedPage {
            outer: ScrollState::new(100),
            inner: ScrollState::new(50),
        },
        Theme::junie(),
        20,
        12,
    );
    assert_eq!(h.focus(), None);
    // inner at the top: wheel up is consumed without repaint or chaining.
    let r = h.wheel(Axis::V, -3, 5, 5);
    assert!(
        r.is_consumed() && !r.is_changed(),
        "boundary consumes quietly"
    );
    assert_eq!(h.app().inner.offset(), 0);
    assert_eq!(h.app().outer.offset(), 0, "no parent chaining");
    assert_eq!(
        h.app().inner.pending_reveal(),
        None,
        "a wheel never reveals"
    );
    assert_eq!(h.focus(), None, "no focus move");
    // wheel down over the inner owner scrolls the inner owner only.
    let r = h.wheel(Axis::V, 3, 5, 5);
    assert!(r.is_consumed() && r.is_changed());
    assert!(h.app().inner.offset() > 0);
    assert_eq!(h.app().outer.offset(), 0);
    // wheel over the outer-only area scrolls the outer owner.
    let r = h.wheel(Axis::V, 3, 15, 5);
    assert!(r.is_consumed() && r.is_changed());
    assert!(h.app().outer.offset() > 0);
    let inner = h.app().inner.offset();
    // inner at the end: wheel down is consumed without chaining.
    h.app_mut().inner.jump_end();
    h.draw();
    assert!(h.app().inner.at_end());
    let outer = h.app().outer.offset();
    let r = h.wheel(Axis::V, 3, 5, 5);
    assert!(
        r.is_consumed() && !r.is_changed(),
        "end boundary consumes quietly"
    );
    assert_eq!(h.app().inner.offset(), h.app().inner.max_offset());
    assert_eq!(h.app().outer.offset(), outer, "no parent chaining");
    assert_eq!(h.app().inner.offset(), h.app().inner.max_offset());
    assert!(inner > 0);
    assert!(h.diagnostics().is_empty());
}

const HIDE_ID: Id = Id::root("fade.hide");
struct HidePage {
    scroll: ScrollState,
    visible: bool,
    captured: bool,
    content_width: std::cell::Cell<u16>,
}
impl App for HidePage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = ScrollRegion::new(HIDE_ID)
            .scrollbar_visible(self.visible)
            .update(cx, &mut self.scroll, 100);
        self.captured = cx.capture_owner() == Some(HIDE_ID);
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let content = ScrollRegion::new(HIDE_ID)
            .scrollbar_visible(self.visible)
            .draw(ui, Rect::new(2, 1, 6, 8), &self.scroll, 100);
        self.content_width.set(content.width);
    }
}

/// Hiding mid-drag releases the thumb capture without scrolling; the
/// hidden bar reserves no column, exposes no pointer part, and still
/// routes wheel and reveal through the shared owner.
#[test]
fn hidden_bar_mid_drag_releases_and_keeps_wheel_reveal_without_parts() {
    let mut h = Harness::new(
        HidePage {
            scroll: ScrollState::new(100),
            visible: true,
            captured: false,
            content_width: std::cell::Cell::new(0),
        },
        Theme::junie(),
        10,
        12,
    );
    assert!(h.area_of_part(HIDE_ID, PartRef::of(Part::TRACK)).is_some());
    // press the thumb (track row 0 of the bar column) and hold it.
    let _ = h.mouse(MouseKind::Down, 7, 1);
    assert!(h.app().captured);
    // hiding mid-drag releases the capture without scrolling.
    h.app_mut().visible = false;
    let _ = h.mouse(MouseKind::Drag, 7, 5);
    assert!(!h.app().captured, "hide releases the drag capture");
    assert_eq!(h.app().scroll.offset(), 0);
    let _ = h.mouse(MouseKind::Up, 7, 5);
    h.draw();
    assert!(h.area_of_part(HIDE_ID, PartRef::of(Part::TRACK)).is_none());
    assert!(h.area_of_part(HIDE_ID, PartRef::of(Part::THUMB)).is_none());
    assert_eq!(h.app().content_width.get(), 6, "full content width");
    // a pointer where the bar was scrolls nothing.
    let _ = h.click(7, 3);
    assert_eq!(h.app().scroll.offset(), 0);
    // wheel and reveal stay active through the shared owner.
    let _ = h.wheel(Axis::V, 4, 3, 3);
    assert!(h.app().scroll.offset() > 0);
    h.app_mut().scroll.ensure_visible_on_next_layout(90);
    let _ = h.key(KeyCode::Null);
    assert_eq!(h.app().scroll.offset(), 83, "reveal applies while hidden");
    assert!(h.diagnostics().is_empty());
}

const TWIN_A: Id = Id::root("fade.twin.a");
const TWIN_B: Id = Id::root("fade.twin.b");
struct TwinPage;
impl App for TwinPage {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        for (id, x) in [(TWIN_A, 0u16), (TWIN_B, 10u16)] {
            let area = Rect::new(x, 0, 10, 8);
            let st = state(10, 100, 8);
            let content = ScrollRegion::new(id).draw(ui, area, &st, 100);
            paint_rows(ui, content);
            if id == TWIN_A {
                ui.scroll_edges(content, &st);
            }
        }
    }
}

/// NEGATIVE: one shared primitive, owner-keyed state. Fading and scrolling
/// one owner never touches the other: no per-app duplicate, no shared
/// mutable renderer.
#[test]
fn scroll_and_fade_state_never_leaks_between_owners() {
    let mut h = Harness::new(TwinPage, Theme::junie(), 20, 8);
    h.draw();
    let buf = h.buffer().clone();
    // twin A fades, twin B is pristine at the same rows.
    assert_eq!(fg_at(&buf, 0, 0), OUTER);
    assert_eq!(fg_at(&buf, 0, 7), OUTER);
    for y in 0..8 {
        assert_eq!(fg_at(&buf, 10, y), FG, "twin B row {y} untouched");
    }
    // bar parts stay under their owner's identity.
    assert_eq!(
        h.area_of_part(TWIN_A, PartRef::of(Part::TRACK)),
        Some(Rect::new(9, 0, 1, 8))
    );
    assert_eq!(
        h.area_of_part(TWIN_B, PartRef::of(Part::TRACK)),
        Some(Rect::new(19, 0, 1, 8))
    );
    assert!(h.diagnostics().is_empty());
}

/// Owners compose [`ScrollRegion`] directly under their own id: the bar
/// parts share the owner's identity, the container covers the full area,
/// and the content rect keeps the full width when everything fits.
#[test]
fn scroll_region_composes_directly_under_owner_identity() {
    const ID: Id = Id::root("fade.compose");
    struct Page {
        content: usize,
    }
    impl App for Page {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            ScrollRegion::new(ID)
                .update(cx, &mut ScrollState::new(self.content), self.content)
                .erase()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let mut st = ScrollState::new(self.content);
            st.set_viewport(6);
            ScrollRegion::new(ID).draw(ui, Rect::new(0, 0, 8, 6), &st, self.content);
        }
    }
    let mut h = Harness::new(Page { content: 100 }, Theme::junie(), 8, 6);
    h.draw();
    assert_eq!(
        h.area_of_part(ID, PartRef::of(Part::CONTAINER)),
        Some(Rect::new(0, 0, 8, 6))
    );
    assert_eq!(
        h.area_of_part(ID, PartRef::of(Part::TRACK)),
        Some(Rect::new(7, 0, 1, 6))
    );
    let thumb = h.area_of_part(ID, PartRef::of(Part::THUMB)).unwrap();
    assert_eq!(thumb.x, 7);
    assert!(thumb.y < 6 && thumb.bottom() <= 6);
    let mut scene = Scene::new("fade_fits", Theme::junie(), ColorLevel::TrueColor, 8, 6);
    scene.draw(|ui, area| {
        let st = state(0, 2, 6);
        let content = ScrollRegion::new(ID).draw(ui, area, &st, 2);
        assert_eq!(content, area, "fits keep the full area");
    });
}

/// The shared model matches the oracle's extent/offset/viewport/reveal/
/// clamp behavior, including empty, single-cell and huge content.
#[test]
fn scroll_model_boundaries_match_oracle_extent_offset_viewport_reveal_clamp() {
    // thumb geometry and its inverse saturate at both endpoints.
    let mut st = ScrollState::new(100);
    st.set_viewport(20);
    assert_eq!(st.thumb(10), (0, 2));
    st.jump_end();
    assert_eq!(st.thumb(10), (8, 2));
    assert_eq!(st.offset_for_track_pos(0, 10), 0);
    let end = st.offset_for_track_pos(9, 10);
    assert_eq!(end, st.max_offset());
    st.jump_start();
    // degenerate content and viewport lengths never panic and stay sane.
    for (content, viewport) in [
        (0usize, 0usize),
        (0, 1),
        (1, 0),
        (1, 1),
        (1, 4),
        (100_000, 1),
        (100_000, 12),
    ] {
        let mut s = ScrollState::new(content);
        s.set_viewport(viewport);
        assert_eq!(s.max_offset(), content.saturating_sub(viewport));
        assert_eq!(s.overflows(), content > viewport && viewport > 0);
        let (start, len) = s.thumb(10);
        assert!(start + len <= 10, "thumb inside the track");
        s.scroll_to(usize::MAX);
        assert_eq!(s.offset(), s.max_offset());
        s.scroll_by(isize::MIN);
        assert_eq!(s.offset(), 0);
        s.page_down();
        s.page_up();
        s.jump_end();
        assert!(s.at_end());
        assert_eq!(
            s.visible_range().end - s.visible_range().start,
            content.min(viewport)
        );
    }
    // reveal moves minimally and clamps; a zero viewport reveals nothing.
    let mut reveal = ScrollState::new(50);
    reveal.set_viewport(10);
    reveal.ensure_visible(25);
    assert_eq!(reveal.offset(), 16);
    reveal.ensure_visible(5);
    assert_eq!(reveal.offset(), 5);
    reveal.ensure_visible(7);
    assert_eq!(reveal.offset(), 5, "visible rows need no scroll");
    let mut zero = ScrollState::new(50);
    zero.ensure_visible(25);
    assert_eq!(zero.offset(), 0, "zero viewport reveals nothing");
    zero.ensure_visible_on_next_layout(40);
    zero.apply_layout(0, 50);
    assert_eq!(
        zero.pending_reveal(),
        Some(40),
        "zero layout retains the reveal"
    );
    zero.apply_layout(10, 50);
    assert_eq!(zero.pending_reveal(), None);
    assert_eq!(zero.offset(), 31);
    // shrinking the content clamps the offset.
    let mut shrink = ScrollState::new(100);
    shrink.set_viewport(10);
    shrink.scroll_to(90);
    shrink.set_content(5);
    assert_eq!(shrink.offset(), 0);
    // the wheel rule: consumed even at the boundary, repaint only on move,
    // and a wheel never requests a reveal.
    let mut wheel = ScrollState::new(30);
    wheel.set_viewport(10);
    let resp = wheel.wheel(-3);
    assert!(resp.is_consumed() && !resp.is_changed());
    let resp = wheel.wheel(3);
    assert!(resp.is_consumed() && resp.is_changed());
    wheel.jump_end();
    let resp = wheel.wheel(3);
    assert!(resp.is_consumed() && !resp.is_changed());
    assert_eq!(wheel.pending_reveal(), None);
    assert!(wheel.headroom_v().up > 0 && wheel.headroom_v().down == 0);
}
