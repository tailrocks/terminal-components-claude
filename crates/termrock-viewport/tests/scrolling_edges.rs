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

use termrock_collections::ScrollState;
use termrock_runtime::{
    App, Color, Cx, Focusability, LayerSpec, Part, PartRef, Rect, ScrollRegion, Style, Ui,
};
use termrock_runtime::{Axis, Id, KeyCode, MouseKind, Position, Response};
use termrock_test_support::{Harness, Scene};
use termrock_theme::{ColorLevel, Modifier, Theme};

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

fn fg_at(buf: &termrock_render::Buffer, x: u16, y: u16) -> Color {
    buf.cell(Position::new(x, y)).map(|c| c.fg).unwrap()
}

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

struct SparseFadeLayerPage {
    open: bool,
}
const FADE_LAYER: Id = Id::root("fade.sparse.layer");
impl App for SparseFadeLayerPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.open {
            cx.open_layer(FADE_LAYER, LayerSpec::modal(FADE_LAYER));
            self.open = true;
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let style = ui.surface_style();
        ui.fill(area, style);
        ui.paint_str(Rect::new(area.x, area.y, area.width, 1), "UNDER", style);
        let _ = ui.layer(FADE_LAYER, |ui, layer_area| {
            let row = Rect::new(
                layer_area.x,
                layer_area.y.saturating_add(layer_area.height / 2),
                layer_area.width,
                1,
            );
            ui.paint_str(row, "middle", ui.surface_style());
            let view = state(5, 100, usize::from(layer_area.height));
            ui.scroll_edges(layer_area, &view);
        });
    }
}

#[test]
fn sparse_layer_fade_keeps_unwritten_edge_transparent() {
    let mut h = Harness::new(SparseFadeLayerPage { open: false }, Theme::junie(), 16, 12);
    h.draw();
    assert_eq!(
        h.buffer().cell(Position::new(0, 0)).unwrap().symbol(),
        "U",
        "an unwritten overlay edge must not cover the page"
    );
}

struct RoleFadeModalPage {
    open: bool,
    fade: bool,
}
const ROLE_FADE_MODAL: Id = Id::root("fade.role.modal");
impl App for RoleFadeModalPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.open {
            cx.open_layer(ROLE_FADE_MODAL, LayerSpec::modal(ROLE_FADE_MODAL));
            self.open = true;
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let style = ui.surface_style();
        ui.fill(area, style);
        for y in area.y..area.bottom() {
            ui.paint_str(Rect::new(area.x, y, area.width, 1), "role-aware", style);
        }
        if self.fade {
            ui.scroll_edges(area, &state(5, 100, usize::from(area.height)));
        }
        let _ = ui.layer(ROLE_FADE_MODAL, |_ui, _area| {});
    }
}

#[test]
fn scroll_fade_preserves_background_role_for_modal_dim() {
    let mut plain = Harness::new(
        RoleFadeModalPage {
            open: false,
            fade: false,
        },
        Theme::junie(),
        16,
        12,
    );
    let mut faded = Harness::new(
        RoleFadeModalPage {
            open: false,
            fade: true,
        },
        Theme::junie(),
        16,
        12,
    );
    plain.draw();
    faded.draw();
    for y in [0, 11] {
        let expected = plain.buffer().cell(Position::new(0, y)).unwrap();
        let actual = faded.buffer().cell(Position::new(0, y)).unwrap();
        assert_eq!(actual.bg, expected.bg, "background role at y={y}");
    }
}

#[test]
fn split_span_graphemes_use_style_of_first_byte() {
    struct SplitSpanPage;
    impl App for SplitSpanPage {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ui.paint_spans(
                Rect::new(0, 0, 8, 1),
                &[
                    termrock_runtime::Span::new("e"),
                    termrock_runtime::Span::new("\u{301}x").bold(),
                ],
                ui.surface_style(),
            );
        }
    }
    let mut h = Harness::new(SplitSpanPage, Theme::junie(), 8, 2);
    h.draw();
    assert_eq!(
        h.buffer().cell(Position::new(0, 0)).unwrap().symbol(),
        "e\u{301}"
    );
    assert!(
        !h.buffer()
            .cell(Position::new(0, 0))
            .unwrap()
            .modifier
            .contains(Modifier::BOLD)
    );
    assert_eq!(h.buffer().cell(Position::new(1, 0)).unwrap().symbol(), "x");
    assert!(
        h.buffer()
            .cell(Position::new(1, 0))
            .unwrap()
            .modifier
            .contains(Modifier::BOLD)
    );

    for grapheme in ["👨‍👩‍👧‍👦", "🇻🇳"] {
        let split = grapheme.char_indices().nth(1).map(|(at, _)| at).unwrap();
        let (first, rest) = grapheme.split_at(split);
        let mut scene = Scene::new("span-grapheme", Theme::junie(), ColorLevel::TrueColor, 8, 1);
        scene.draw(|ui, _area| {
            ui.paint_spans(
                Rect::new(0, 0, 8, 1),
                &[
                    termrock_runtime::Span::new(first),
                    termrock_runtime::Span::new(rest).bold(),
                ],
                Style::new(),
            );
        });
        assert_eq!(
            scene.buffer().cell(Position::new(0, 0)).unwrap().symbol(),
            grapheme
        );
        assert!(
            !scene
                .buffer()
                .cell(Position::new(0, 0))
                .unwrap()
                .modifier
                .contains(Modifier::BOLD)
        );
    }
}

#[test]
fn fade_background_majority_ties_and_exclusions_match_oracle() {
    for theme in themes() {
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

#[test]
fn fade_protects_cursor_keep_selected_hovered_and_marked_rows() {
    const CURSOR_ID: Id = Id::root("fade.protect");
    for theme in themes() {
        let mut scene = Scene::new("fade_protect", theme, ColorLevel::TrueColor, 16, 12);
        scene.draw(|ui, area| {
            paint_rows(ui, area);

            ui.paint_str(
                Rect::new(0, 0, 16, 1),
                "row-content-here!",
                Style::new().fg(FG).bg(Color::Rgb(20, 60, 20)),
            );

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

            ui.set_cursor(CURSOR_ID, Position::new(4, 10));

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

#[test]
fn fade_skips_headers_footers_and_bar_column() {
    const ID: Id = Id::root("fade.chrome");
    for theme in themes() {
        let paint = |fade: bool| {
            let mut scene = Scene::new("fade_chrome", theme.clone(), ColorLevel::TrueColor, 12, 8);
            scene.draw(|ui, area| {
                let st = state(10, 100, 6);
                let content = ScrollRegion::new(ID).draw(ui, area, &st, 100);

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

        assert_eq!(fg_at(faded_buf, 0, 1), OUTER, "top content edge fades");
        assert_eq!(fg_at(faded_buf, 0, 6), OUTER, "bottom content edge fades");
        assert_eq!(fg_at(faded_buf, 0, 3), FG, "content middle stays");
    }
}

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

    for x in 0..11 {
        assert_eq!(
            faded.0.cell(Position::new(x, 3)),
            plain.0.cell(Position::new(x, 3)),
            "selected cell ({x}, 3)"
        );
    }
}

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

#[test]
fn thumb_drag_from_bare_track_follows_pointer_and_saturates() {
    let mut h = Harness::new(DragPage::new(100), Theme::junie(), 10, 15);
    let _ = h.click(6, 11);
    assert_eq!(h.app().scroll.offset(), 90);

    let _ = h.mouse(MouseKind::Down, 6, 2);
    assert_eq!(h.app().scroll.offset(), 0);
    assert!(!h.app().captured, "a track press claims no capture");
    let _ = h.mouse(MouseKind::Drag, 6, 11);
    assert_eq!(h.app().scroll.offset(), 90, "drag follows to the end");
    assert!(!h.app().captured);
    let _ = h.mouse(MouseKind::Up, 6, 11);

    let _ = h.mouse(MouseKind::Down, 6, 11);
    assert_eq!(h.app().scroll.offset(), 90);
    let _ = h.mouse(MouseKind::Drag, 6, 2);
    assert_eq!(h.app().scroll.offset(), 0, "drag follows to the start");
    let _ = h.mouse(MouseKind::Up, 6, 2);

    let _ = h.mouse(MouseKind::Down, 6, 5);
    assert_eq!(h.app().scroll.offset(), 30);
    let _ = h.mouse(MouseKind::Drag, 6, 6);
    assert_eq!(h.app().scroll.offset(), 40, "one row follows one row");
    let _ = h.mouse(MouseKind::Up, 6, 6);
    assert!(!h.app().foreign_capture, "exactly one capture owner");
    assert!(h.diagnostics().is_empty());
}

struct InsetDragPage {
    scroll: ScrollState,
}

const INSET_DRAG_ID: Id = Id::root("fade.inset-drag");

impl App for InsetDragPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        ScrollRegion::new(INSET_DRAG_ID)
            .update(cx, &mut self.scroll, 100)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(
            INSET_DRAG_ID,
            Rect::new(1, 1, 8, 12),
            Focusability::Focusable,
        );
        ScrollRegion::new(INSET_DRAG_ID).draw(ui, Rect::new(2, 4, 5, 8), &self.scroll, 100);
    }
}

#[test]
fn bare_track_drag_uses_inset_track_origin() {
    let mut h = Harness::new(
        InsetDragPage {
            scroll: ScrollState::new(100),
        },
        Theme::junie(),
        12,
        14,
    );
    let _ = h.mouse(MouseKind::Down, 6, 8);
    assert_eq!(
        h.app().scroll.offset(),
        h.app().scroll.offset_for_track_pos(4, 8),
        "press is positioned relative to the inset track"
    );
    let _ = h.mouse(MouseKind::Drag, 6, 9);
    assert_eq!(
        h.app().scroll.offset(),
        h.app().scroll.offset_for_track_pos(5, 8),
        "one-row drag advances one row of the registered track"
    );
    assert!(h.diagnostics().is_empty());
}

#[test]
fn thumb_drag_from_thumb_retains_grab_offset_and_single_capture() {
    let mut h = Harness::new(DragPage::new(30), Theme::junie(), 10, 15);

    let _ = h.mouse(MouseKind::Down, 6, 3);
    assert_eq!(h.app().scroll.thumb(10), (0, 3));
    assert_eq!(h.app().scroll.offset(), 0);
    assert!(h.app().captured, "the thumb holds the capture");

    let _ = h.mouse(MouseKind::Drag, 6, 4);
    assert_eq!(h.app().scroll.offset(), 3);
    assert_eq!(h.app().scroll.thumb(10).0, 1, "grab offset retained");
    assert!(h.app().captured);

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

    let r = h.wheel(Axis::V, 3, 5, 5);
    assert!(r.is_consumed() && r.is_changed());
    assert!(h.app().inner.offset() > 0);
    assert_eq!(h.app().outer.offset(), 0);

    let r = h.wheel(Axis::V, 3, 15, 5);
    assert!(r.is_consumed() && r.is_changed());
    assert!(h.app().outer.offset() > 0);
    let inner = h.app().inner.offset();

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

    let _ = h.mouse(MouseKind::Down, 7, 1);
    assert!(h.app().captured);

    h.app_mut().visible = false;
    let _ = h.mouse(MouseKind::Drag, 7, 5);
    assert!(!h.app().captured, "hide releases the drag capture");
    assert_eq!(h.app().scroll.offset(), 0);
    let _ = h.mouse(MouseKind::Up, 7, 5);
    h.draw();
    assert!(h.area_of_part(HIDE_ID, PartRef::of(Part::TRACK)).is_none());
    assert!(h.area_of_part(HIDE_ID, PartRef::of(Part::THUMB)).is_none());
    assert_eq!(h.app().content_width.get(), 6, "full content width");

    let _ = h.click(7, 3);
    assert_eq!(h.app().scroll.offset(), 0);

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

#[test]
fn scroll_and_fade_state_never_leaks_between_owners() {
    let mut h = Harness::new(TwinPage, Theme::junie(), 20, 8);
    h.draw();
    let buf = h.buffer().clone();

    assert_eq!(fg_at(&buf, 0, 0), OUTER);
    assert_eq!(fg_at(&buf, 0, 7), OUTER);
    for y in 0..8 {
        assert_eq!(fg_at(&buf, 10, y), FG, "twin B row {y} untouched");
    }

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

#[test]
fn scroll_model_boundaries_match_oracle_extent_offset_viewport_reveal_clamp() {
    let mut st = ScrollState::new(100);
    st.set_viewport(20);
    assert_eq!(st.thumb(10), (0, 2));
    st.jump_end();
    assert_eq!(st.thumb(10), (8, 2));
    assert_eq!(st.offset_for_track_pos(0, 10), 0);
    let end = st.offset_for_track_pos(9, 10);
    assert_eq!(end, st.max_offset());
    st.jump_start();

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

    let mut shrink = ScrollState::new(100);
    shrink.set_viewport(10);
    shrink.scroll_to(90);
    shrink.set_content(5);
    assert_eq!(shrink.offset(), 0);

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
