use ratatui_core::buffer::{Buffer, CellDiffOption};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use termrock_core::{Input, id::Id};
use termrock_runtime::{Anchor, App, LayerSize, LayerSpec, Runtime, Ui, UpdateCause, stub};
use termrock_theme::Theme;

const SCREEN: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 10,
};
const LOWER: Id = Id::root("underlay.lower");
const UPPER: Id = Id::root("underlay.upper");

struct StackApp;

impl App for StackApp {
    fn update(&mut self, cx: &mut termrock_runtime::Cx<'_>) -> termrock_core::Response<()> {
        if !cx.is_open(LOWER) {
            cx.open_layer(
                LOWER,
                LayerSpec::popover(LOWER, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 3)),
            );
        }
        if cx.update_cause() == UpdateCause::Tick && !cx.is_open(UPPER) {
            cx.open_layer(
                UPPER,
                LayerSpec::popover(LOWER, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 3)),
            );
        }
        termrock_core::Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.fill(ui.full(), Style::new().fg(Color::Red));
        let _ = ui.layer(UPPER, |ui, area| {
            ui.fill_keep_ink(area, Color::Black);
            ui.paint_str(
                Rect::new(area.x, area.y, 1, 1),
                "X",
                Style::new()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            );
        });
        let _ = ui.layer(LOWER, |ui, area| {
            ui.fill(
                area,
                Style::new()
                    .fg(Color::Blue)
                    .underline_color(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );
        });
    }
}

#[test]
fn admitted_upper_layer_inherits_nearest_ink_and_applies_explicit_paint() {
    let mut runtime = Runtime::new(StackApp, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(SCREEN);
    runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
    assert_eq!(buffer.cell((2, 2)).map(|cell| cell.fg), Some(Color::Blue));

    let _ = stub::deliver(&mut runtime, Input::Tick);
    assert!(
        runtime.open_spec(UPPER).is_some(),
        "upper layer must be admitted"
    );
    runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
    let upper_area = runtime.layer_area(UPPER).expect("upper layer was drawn");

    let overlap = buffer.cell((3, 3)).expect("overlap cell is present");
    assert_eq!(overlap.bg, Color::Black, "upper fill must render");
    assert_eq!(overlap.fg, Color::Blue, "ink must come from lower layer");
    assert_eq!(overlap.underline_color, Color::Green);
    assert!(overlap.modifier.contains(Modifier::BOLD));

    let patched = buffer
        .cell((upper_area.x, upper_area.y))
        .expect("explicitly painted cell is present");
    assert_eq!(patched.symbol(), "X");
    assert_eq!(patched.fg, Color::Yellow);
    assert_eq!(patched.bg, Color::Black);
    assert_eq!(patched.underline_color, Color::Green);
    assert!(patched.modifier.contains(Modifier::BOLD));
    assert!(patched.modifier.contains(Modifier::ITALIC));
}

#[test]
fn page_fill_resets_wide_transport_metadata_to_an_ordinary_space() {
    let area = Rect::new(0, 0, 8, 1);
    for diff_option in [
        CellDiffOption::Skip,
        CellDiffOption::AlwaysUpdate,
        CellDiffOption::ForcedWidth(std::num::NonZeroU16::new(4).expect("nonzero width")),
    ] {
        let mut runtime = Runtime::new(stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                let (raw, _) = ui.raw();
                if let Some(cell) = raw.cell_mut((0, 0)) {
                    cell.set_diff_option(diff_option).set_symbol("image");
                }
                ui.fill_keep_ink(ui.full(), Color::Black);
            })
            .commit_presented();
        let cell = buffer.cell((0, 0)).expect("filled cell is present");
        assert_eq!(cell.symbol(), " ");
        assert_eq!(cell.diff_option, CellDiffOption::None);
    }
}

struct RawOverride;

impl App for RawOverride {
    fn update(&mut self, cx: &mut termrock_runtime::Cx<'_>) -> termrock_core::Response<()> {
        const POPUP: Id = Id::root("underlay.raw-popup");
        if !cx.is_open(POPUP) {
            cx.open_layer(
                POPUP,
                LayerSpec::popover(POPUP, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 3)),
            );
        }
        termrock_core::Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        const POPUP: Id = Id::root("underlay.raw-popup");
        ui.fill(ui.full(), Style::new().fg(Color::Red));
        let _ = ui.layer(POPUP, |ui, area| {
            ui.fill_keep_ink(area, Color::Black);
            let (buffer, clipped) = ui.raw();
            if clipped.contains(Position::new(area.x, area.y))
                && let Some(cell) = buffer.cell_mut((area.x, area.y))
            {
                cell.reset();
                cell.set_symbol("R")
                    .set_style(Style::new().fg(Color::Magenta));
            }
        });
    }
}

#[test]
fn raw_full_cell_write_cancels_underlay_style_intent() {
    let mut runtime = Runtime::new(RawOverride, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(SCREEN);
    runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
    let popup = runtime
        .layer_area(Id::root("underlay.raw-popup"))
        .expect("popup was admitted and drawn");
    let cell = buffer
        .cell((popup.x, popup.y))
        .expect("raw-overridden cell is visible");
    assert_eq!(cell.symbol(), "R");
    assert_eq!(cell.fg, Color::Magenta);
    assert_eq!(cell.bg, Color::Reset);
}

struct FadeModel {
    inherit: bool,
}

impl App for FadeModel {
    fn update(&mut self, cx: &mut termrock_runtime::Cx<'_>) -> termrock_core::Response<()> {
        const LOWER: Id = Id::root("underlay.fade-lower");
        const POPUP: Id = Id::root("underlay.fade-popup");
        if !cx.is_open(LOWER) {
            cx.open_layer(
                LOWER,
                LayerSpec::popover(LOWER, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 6)),
            );
        }
        if !cx.is_open(POPUP) {
            cx.open_layer(
                POPUP,
                LayerSpec::popover(LOWER, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 6)),
            );
        }
        termrock_core::Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        const LOWER: Id = Id::root("underlay.fade-lower");
        const POPUP: Id = Id::root("underlay.fade-popup");
        let base = Style::new().fg(Color::Rgb(200, 40, 40)).bg(Color::Black);
        ui.fill(ui.full(), base);
        let _ = ui.layer(LOWER, |ui, area| {
            ui.fill(
                area,
                Style::new().fg(Color::Rgb(40, 80, 220)).bg(Color::Black),
            );
        });
        let _ = ui.layer(POPUP, |ui, area| {
            if self.inherit {
                ui.fill_keep_ink(area, Color::Black);
            } else {
                ui.fill(
                    area,
                    Style::new().fg(Color::Rgb(40, 80, 220)).bg(Color::Black),
                );
            }
            ui.paint_str(
                Rect::new(area.x, area.y, 1, 1),
                "X",
                Style::new().fg(Color::Rgb(40, 200, 80)),
            );
            let mut scroll = termrock_runtime::ScrollState::new(100);
            scroll.set_viewport(6);
            scroll.scroll_by(1);
            ui.scroll_edges(area, &scroll);
        });
    }
}

#[test]
fn postfill_scroll_fade_is_not_overwritten_by_composition() {
    let draw = |inherit| {
        let area = Rect::new(0, 0, 40, 12);
        let mut runtime = Runtime::new(FadeModel { inherit }, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        buffer
    };

    assert_eq!(draw(true), draw(false));
}

struct ModifierModel {
    inherit: bool,
}

impl App for ModifierModel {
    fn update(&mut self, cx: &mut termrock_runtime::Cx<'_>) -> termrock_core::Response<()> {
        const POPUP: Id = Id::root("underlay.modifier-popup");
        if !cx.is_open(POPUP) {
            cx.open_layer(
                POPUP,
                LayerSpec::popover(POPUP, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 3)),
            );
        }
        termrock_core::Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        const POPUP: Id = Id::root("underlay.modifier-popup");
        let base = Style::new().fg(Color::Red).bg(Color::Black);
        ui.fill(ui.full(), base);
        let _ = ui.layer(POPUP, |ui, area| {
            if self.inherit {
                ui.fill_keep_ink(area, Color::Black);
            } else {
                ui.fill(area, base);
            }
            ui.paint_str(
                Rect::new(area.x, area.y, 1, 1),
                "X",
                Style {
                    add_modifier: Modifier::BOLD,
                    sub_modifier: Modifier::BOLD,
                    ..Style::new()
                },
            );
        });
    }
}

#[test]
fn overlapping_modifier_fields_keep_ratatui_remove_precedence() {
    let draw = |inherit| {
        let area = Rect::new(0, 0, 40, 10);
        let mut runtime = Runtime::new(ModifierModel { inherit }, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        buffer
    };

    assert_eq!(draw(true), draw(false));
}

struct MoveModel {
    inherit: bool,
}

impl App for MoveModel {
    fn update(&mut self, cx: &mut termrock_runtime::Cx<'_>) -> termrock_core::Response<()> {
        const POPUP: Id = Id::root("underlay.move-popup");
        if !cx.is_open(POPUP) {
            cx.open_layer(
                POPUP,
                LayerSpec::popover(POPUP, Anchor::Point(Position::new(1, 1)))
                    .size(LayerSize::Fixed(8, 3)),
            );
        }
        termrock_core::Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        const POPUP: Id = Id::root("underlay.move-popup");
        let base = Style::new()
            .fg(Color::Red)
            .bg(Color::Black)
            .underline_color(Color::Green)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        ui.fill(ui.full(), base);
        let _ = ui.layer(POPUP, |ui, area| {
            if self.inherit {
                ui.fill_keep_ink(area, Color::Black);
            } else {
                ui.fill(area, base);
            }
            let mut cell = termrock_runtime::CellUi::new(
                ui.reborrow(),
                Rect::new(area.x, area.y, 4, 1),
                termrock_theme::PaintStyle::new(),
            );
            cell.align(termrock_theme::Align::Right)
                .padding_style(termrock_theme::PaintStyle::new())
                .text("X");
        });
    }
}

#[test]
fn moved_label_vacated_cell_uses_explicit_reset_attributes() {
    let draw = |inherit| {
        let area = Rect::new(0, 0, 40, 10);
        let mut runtime = Runtime::new(MoveModel { inherit }, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        buffer
    };

    assert_eq!(draw(true), draw(false));
}
