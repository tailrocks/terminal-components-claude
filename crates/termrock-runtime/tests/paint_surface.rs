#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "focused public author-surface assertions"
)]

use std::cell::{Cell, RefCell};

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use termrock::author::PaintSurface;
use termrock_core::{Id, Response};
use termrock_runtime::{Anchor, App, Cx, LayerSize, LayerSpec, ScreenAlign, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Role, StylePatch, Theme};

const FRAME: Rect = Rect::new(0, 0, 12, 6);
const POPUP: Rect = Rect::new(4, 2, 4, 2);
const CLIP: Rect = Rect::new(5, 2, 2, 2);
const POPUP_LAYER: Id = Id::root("paint_surface.popup");

fn rich_style() -> Style {
    Style::new()
        .fg(Color::Red)
        .bg(Color::Blue)
        .underline_color(Color::Green)
        .add_modifier(Modifier::BOLD | Modifier::ITALIC)
}

#[test]
fn surface_owns_absolute_geometry_and_keeps_scalar_and_grapheme_writes_distinct() {
    let area = Rect::new(3, 4, 8, 3);
    let mut surface = PaintSurface::new(area);
    assert_eq!(surface.area(), area);
    surface.symbol(Position::new(2, 4), "outside", rich_style());
    assert!(surface.cell(Position::new(2, 4)).is_none());

    surface.text(3, 4, "e\u{301}x", rich_style());
    assert_eq!(surface.cell(Position::new(3, 4)).unwrap().symbol, "e");
    assert_eq!(surface.cell(Position::new(4, 4)).unwrap().symbol, "\u{301}");
    assert_eq!(surface.cell(Position::new(5, 4)).unwrap().symbol, "x");

    surface.string(3, 5, "界e\u{301}x", rich_style());
    assert_eq!(surface.cell(Position::new(3, 5)).unwrap().symbol, "界");
    assert_eq!(surface.cell(Position::new(4, 5)).unwrap().symbol, " ");
    assert_eq!(
        surface.cell(Position::new(5, 5)).unwrap().symbol,
        "e\u{301}"
    );
    assert_eq!(surface.cell(Position::new(6, 5)).unwrap().symbol, "x");
    assert!(surface.cell(Position::new(2, 4)).is_none());

    let clone = surface.clone();
    assert_eq!(surface, clone);
    assert!(format!("{surface:?}").contains("PaintSurface"));
}

#[test]
fn string_ignores_outside_starts_and_keeps_valid_right_edge_truncation() {
    let area = Rect::new(3, 4, 8, 3);
    let mut surface = PaintSurface::new(area);
    surface.fill(area, rich_style());
    surface.symbol(Position::new(3, 4), "S", rich_style());
    let before = surface.clone();

    for start in [
        Position::new(2, 4),  // before the owned x origin
        Position::new(3, 3),  // above the owned y origin
        Position::new(3, 7),  // below the owned y range
        Position::new(11, 4), // at the exclusive right edge
        Position::new(12, 4), // beyond the right edge
    ] {
        surface.string(start.x, start.y, "outside", rich_style());
        assert_eq!(
            surface, before,
            "outside start {start:?} changed owned cells"
        );
        assert!(
            surface.cell(start).is_none(),
            "outside start {start:?} unexpectedly has a cell"
        );
    }

    // A valid start still writes the narrow prefix Ratatui can measure.
    surface.string(10, 4, "x界", rich_style());
    assert_eq!(surface.cell(Position::new(10, 4)).unwrap().symbol, "x");
    let last_cell = surface.cell(Position::new(10, 4)).unwrap();
    surface.string(10, 4, "界", rich_style());
    assert_eq!(surface.cell(Position::new(10, 4)).unwrap(), last_cell);
}

#[test]
fn style_and_restyle_merge_only_named_channels_and_leave_noop_cells_alone() {
    let mut surface = PaintSurface::new(Rect::new(0, 0, 2, 1));
    surface.symbol(Position::new(0, 0), "A", rich_style());
    surface.symbol(Position::new(1, 0), "B", rich_style());

    surface.style_area(
        Rect::new(0, 0, 1, 1),
        Style::new()
            .fg(Color::Reset)
            .underline_color(Color::Reset)
            .remove_modifier(Modifier::BOLD),
    );
    let before_noop = surface.cell(Position::new(0, 0)).unwrap();
    let visited = RefCell::new(Vec::new());
    surface.restyle(Rect::new(0, 0, 4, 1), |pos, view| {
        visited.borrow_mut().push(pos);
        (view.symbol == "B").then_some(
            Style::new()
                .bg(Color::Reset)
                .add_modifier(Modifier::UNDERLINED)
                .remove_modifier(Modifier::ITALIC),
        )
    });
    assert_eq!(
        visited.into_inner(),
        vec![Position::new(0, 0), Position::new(1, 0)]
    );

    let a = surface.cell(Position::new(0, 0)).unwrap();
    assert_eq!(a, before_noop);
    assert_eq!(a.fg(), Color::Reset);
    assert_eq!(a.bg(), Color::Blue);
    assert_eq!(a.style.underline_color, Some(Color::Reset));
    assert_eq!(a.modifier(), Modifier::ITALIC);

    let b = surface.cell(Position::new(1, 0)).unwrap();
    assert_eq!(b.symbol, "B");
    assert_eq!(b.fg(), Color::Red);
    assert_eq!(b.bg(), Color::Reset);
    assert_eq!(b.style.underline_color, Some(Color::Green));
    assert_eq!(b.modifier(), Modifier::BOLD | Modifier::UNDERLINED);

    surface.fill(Rect::new(1, 0, 1, 1), Style::new().bg(Color::Green));
    let filled = surface.cell(Position::new(1, 0)).unwrap();
    assert_eq!(filled.symbol, " ");
    assert_eq!(filled.fg(), Color::Red);
    assert_eq!(filled.bg(), Color::Green);
    assert_eq!(filled.modifier(), Modifier::BOLD | Modifier::UNDERLINED);
    assert!(surface.cell(Position::new(2, 0)).is_none());
}

#[test]
fn empty_and_last_representable_regions_are_safe_noops_at_the_boundary() {
    let empty = Rect::new(9, 9, 0, 2);
    let mut surface = PaintSurface::new(empty);
    surface.symbol(Position::new(9, 9), "x", rich_style());
    surface.text(9, 9, "xy", rich_style());
    surface.string(9, 9, "xy", rich_style());
    surface.fill(Rect::new(0, 0, u16::MAX, u16::MAX), rich_style());
    surface.style_area(Rect::new(0, 0, u16::MAX, u16::MAX), rich_style());
    let callback_called = Cell::new(false);
    surface.restyle(Rect::new(0, 0, u16::MAX, u16::MAX), |_, _| {
        callback_called.set(true);
        Some(rich_style())
    });
    assert!(!callback_called.get());
    assert!(surface.cell(Position::new(9, 9)).is_none());

    let edge = Rect::new(u16::MAX - 1, u16::MAX - 1, 1, 1);
    let mut surface = PaintSurface::new(edge);
    surface.text(u16::MAX - 1, u16::MAX - 1, "AB", rich_style());
    assert_eq!(
        surface
            .cell(Position::new(u16::MAX - 1, u16::MAX - 1))
            .unwrap()
            .symbol,
        "A"
    );
    assert!(surface.cell(Position::new(u16::MAX, u16::MAX)).is_none());
}

#[derive(Clone, Copy)]
enum PaintPath {
    Surface,
    DirectUi,
}

struct SurfacePage {
    path: PaintPath,
    popup: bool,
    surface: PaintSurface,
}

impl SurfacePage {
    fn new(path: PaintPath, popup: bool) -> Self {
        let mut surface = PaintSurface::new(POPUP);
        surface.symbol(Position::new(4, 2), "A", rich_style());
        surface.symbol(
            Position::new(5, 2),
            "B",
            Style::new()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::UNDERLINED),
        );
        surface.symbol(
            Position::new(6, 3),
            "C",
            Style::new()
                .fg(Color::Yellow)
                .bg(Color::Blue)
                .underline_color(Color::Red),
        );
        surface.symbol(Position::new(7, 3), "D", Style::new().fg(Color::Magenta));
        Self {
            path,
            popup,
            surface,
        }
    }

    fn paint_cells(&self, ui: &mut Ui<'_>, area: Rect) {
        match self.path {
            PaintPath::Surface => self.surface.blit(ui, area),
            PaintPath::DirectUi => {
                ui.paint_cell(Position::new(4, 2), "A", rich_style());
                ui.paint_cell(
                    Position::new(5, 2),
                    "B",
                    Style::new()
                        .fg(Color::Cyan)
                        .bg(Color::Black)
                        .underline_color(Color::Reset)
                        .add_modifier(Modifier::UNDERLINED),
                );
                let reset = Style::new()
                    .fg(Color::Reset)
                    .bg(Color::Reset)
                    .underline_color(Color::Reset);
                ui.paint_cell(Position::new(6, 2), " ", reset);
                ui.paint_cell(Position::new(7, 2), " ", reset);
                ui.paint_cell(Position::new(4, 3), " ", reset);
                ui.paint_cell(Position::new(5, 3), " ", reset);
                ui.paint_cell(
                    Position::new(6, 3),
                    "C",
                    Style::new()
                        .fg(Color::Yellow)
                        .bg(Color::Blue)
                        .underline_color(Color::Red),
                );
                ui.paint_cell(
                    Position::new(7, 3),
                    "D",
                    Style::new()
                        .fg(Color::Magenta)
                        .bg(Color::Reset)
                        .underline_color(Color::Reset),
                );
            }
        }
    }
}

impl App for SurfacePage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.popup && cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(
                POPUP_LAYER,
                LayerSpec::popover(POPUP_LAYER, Anchor::Screen(ScreenAlign::Balanced))
                    .size(LayerSize::Fixed(4, 2)),
            );
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
        ui.fill(FRAME, base);

        if self.popup {
            let _ = ui.layer(POPUP_LAYER, |ui, area| {
                ui.with_area(CLIP, |ui| self.paint_cells(ui, area));
                ui.dim_layer(area, 1);
            });
        } else {
            ui.with_area(CLIP, |ui| self.paint_cells(ui, FRAME));
            ui.dim_layer(FRAME, 1);
        }
    }
}

#[test]
fn surface_blit_matches_known_ui_paints_through_page_clip_and_popover_layer() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for popup in [false, true] {
            let actual = Harness::new(
                SurfacePage::new(PaintPath::Surface, popup),
                Theme::junie().for_level(level),
                12,
                6,
            );
            let direct = Harness::new(
                SurfacePage::new(PaintPath::DirectUi, popup),
                Theme::junie().for_level(level),
                12,
                6,
            );

            for y in 0..6 {
                for x in 0..12 {
                    assert_eq!(
                        actual.cell(x, y),
                        direct.cell(x, y),
                        "logical Cell fields differ at ({x},{y}), level={level:?}, popup={popup}"
                    );
                    assert_eq!(
                        format!("{:?}", actual.cell(x, y)),
                        format!("{:?}", direct.cell(x, y)),
                        "native Cell representation differs at ({x},{y}), level={level:?}, popup={popup}"
                    );
                }
            }
            assert_eq!(
                actual.cell(0, 0),
                direct.cell(0, 0),
                "semantic base sentinel"
            );
            assert_eq!(actual.cell(4, 2).symbol(), " ", "left clipped sentinel");
            assert_eq!(actual.cell(5, 2).symbol(), "B", "first painted cell");
            assert_eq!(actual.cell(6, 3).symbol(), "C", "second painted cell");
            assert_eq!(actual.cell(7, 3).symbol(), " ", "right clipped sentinel");
            assert!(actual.diagnostics().is_empty());
            assert!(direct.diagnostics().is_empty());
        }
    }
}
