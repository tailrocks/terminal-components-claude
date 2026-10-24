#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "focused public-rendering regression assertions"
)]

use std::cell::RefCell;
use std::rc::Rc;

use ratatui_core::style::Color;
use termrock_core::{Id, Response};
use termrock_runtime::{App, Cx, LayerSpec, Rect, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{FgStep, Role, StylePatch, Surface, Theme};

const OVERLAY: Id = Id::root("recolor_provenance.overlay");
const AREA: Rect = Rect::new(0, 0, 12, 2);

struct View {
    identity: bool,
    layer_probe: bool,
    changed_channel: Option<bool>,
}

impl App for View {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.layer_probe && cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(OVERLAY, LayerSpec::modal(OVERLAY));
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Field)),
        );
        ui.paint_str(Rect::new(1, 0, 1, 1), "R", style);
        if self.layer_probe {
            ui.layer(OVERLAY, |ui, area| {
                ui.recolor_pair(area, |fg, bg| (fg, bg));
            });
        } else if self.identity {
            ui.recolor_fg(AREA, |fg, _| Some(fg));
            ui.dim_layer(AREA, 1);
        } else if let Some(change_fg) = self.changed_channel {
            ui.recolor_pair(AREA, |fg, bg| {
                if change_fg {
                    (Color::Rgb(1, 2, 3), bg)
                } else {
                    (fg, Color::Rgb(4, 5, 6))
                }
            });
            ui.dim_layer(AREA, 1);
        } else {
            ui.dim_layer(AREA, 1);
        }
    }
}

#[test]
fn no_op_foreground_recolor_keeps_semantic_role_and_full_cell() {
    let expected = Harness::new(
        View {
            identity: false,
            layer_probe: false,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );
    let actual = Harness::new(
        View {
            identity: true,
            layer_probe: false,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );

    assert_eq!(actual.cell(1, 0), expected.cell(1, 0));
}

#[test]
fn identity_pair_recolor_keeps_unpainted_overlay_transparent() {
    let harness = Harness::new(
        View {
            identity: false,
            layer_probe: true,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );

    assert_eq!(harness.cell(1, 0).symbol(), "R");
}

#[test]
fn changing_one_pair_channel_keeps_other_semantic_role_through_dimming() {
    let baseline = Harness::new(
        View {
            identity: false,
            layer_probe: false,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );

    for change_fg in [true, false] {
        let changed = Harness::new(
            View {
                identity: false,
                layer_probe: false,
                changed_channel: Some(change_fg),
            },
            Theme::junie(),
            12,
            2,
        );
        let expected = baseline.cell(1, 0);
        let actual = changed.cell(1, 0);
        assert_eq!(actual.symbol(), expected.symbol());
        assert_eq!(actual.modifier, expected.modifier);
        if change_fg {
            assert_ne!(actual.fg, expected.fg);
            assert_eq!(actual.bg, expected.bg);
        } else {
            assert_eq!(actual.fg, expected.fg);
            assert_ne!(actual.bg, expected.bg);
        }
    }
}

#[test]
fn rgb_recolor_visits_only_rgb_channels_in_foreground_then_background_order() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let callback_calls = Rc::clone(&calls);
    let harness = Harness::new(
        RgbCallbackView {
            calls: callback_calls,
        },
        Theme::junie(),
        12,
        1,
    );

    assert_eq!(
        *calls.borrow(),
        vec![Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6)]
    );
    assert_eq!(harness.cell(1, 0).fg, Color::Rgb(7, 8, 9));
    assert_eq!(harness.cell(1, 0).bg, Color::Rgb(10, 11, 12));
}

struct RgbCallbackView {
    calls: Rc<RefCell<Vec<Color>>>,
}

impl App for RgbCallbackView {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let style = termrock_theme::PaintStyle::new()
            .fg(Color::Rgb(1, 2, 3))
            .bg(Color::Rgb(4, 5, 6));
        ui.paint_str(Rect::new(1, 0, 1, 1), "C", style);
        let calls = Rc::clone(&self.calls);
        ui.recolor_rgb(Rect::new(1, 0, 1, 1), move |color| {
            calls.borrow_mut().push(color);
            match color {
                Color::Rgb(1, 2, 3) => Color::Rgb(7, 8, 9),
                Color::Rgb(4, 5, 6) => Color::Rgb(10, 11, 12),
                other => other,
            }
        });
    }
}

use ratatui_core::layout::Position;
use ratatui_core::style::{Modifier, Style};
use termrock_runtime::{Anchor, LayerSize};
use termrock_theme::ColorLevel;

const LOWER_LAYER: Id = Id::root("recolor_provenance.stack.lower");
const UPPER_LAYER: Id = Id::root("recolor_provenance.stack.upper");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetintOperation {
    LegacyRaw,
    RecolorForeground,
    DirectReference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Probe {
    Matching,
    NoMatch,
    EqualColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Geometry {
    Offset,
    Tiny,
    Clamped,
}

impl Geometry {
    fn screen(self) -> (u16, u16) {
        match self {
            Self::Offset => (36, 14),
            Self::Tiny => (1, 1),
            Self::Clamped => (10, 7),
        }
    }

    fn lower_spec(self) -> LayerSpec {
        match self {
            Self::Offset => LayerSpec::popover(LOWER_LAYER, Anchor::Point(Position::new(2, 0)))
                .size(LayerSize::Fixed(24, 10)),
            Self::Tiny => LayerSpec::popover(LOWER_LAYER, Anchor::Point(Position::new(0, 0)))
                .size(LayerSize::Fixed(1, 1)),
            Self::Clamped => LayerSpec::popover(LOWER_LAYER, Anchor::Point(Position::new(1, 1)))
                .size(LayerSize::Fixed(6, 4)),
        }
    }

    fn upper_spec(self) -> LayerSpec {
        match self {
            Self::Offset => LayerSpec::popover(LOWER_LAYER, Anchor::Point(Position::new(8, 2)))
                .size(LayerSize::Fixed(12, 5)),
            Self::Tiny => LayerSpec::popover(LOWER_LAYER, Anchor::Point(Position::new(0, 0)))
                .size(LayerSize::Fixed(1, 1)),
            Self::Clamped => LayerSpec::popover(LOWER_LAYER, Anchor::Point(Position::new(8, 6)))
                .size(LayerSize::Fixed(14, 4)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ScanStats {
    visited: usize,
    matching: usize,
    changed: usize,
}

#[derive(Clone, Copy, Debug)]
struct Facts {
    from: Color,
    to: Color,
    glyph_fg: Color,
    no_match_fg: Color,
    page_fg: Color,
    page_underline: Color,
    lower_fg: Color,
    lower_underline: Color,
    upper_bg: Color,
}

struct LayeredRetint {
    geometry: Geometry,
    operation: RetintOperation,
    from: Color,
    to: Color,
    glyph_fg: Color,
    no_match_fg: Color,
    page_fg: Color,
    page_bg: Color,
    page_underline: Color,
    lower_fg: Color,
    lower_bg: Color,
    lower_underline: Color,
    upper_bg: Color,
    stats: Rc<RefCell<ScanStats>>,
}

impl App for LayeredRetint {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(LOWER_LAYER, self.geometry.lower_spec());
            cx.open_layer(UPPER_LAYER, self.geometry.upper_spec());
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let screen = ui.full();
        ui.fill(
            screen,
            Style::new()
                .fg(self.page_fg)
                .bg(self.page_bg)
                .underline_color(self.page_underline)
                .add_modifier(Modifier::BOLD),
        );
        if !screen.is_empty() {
            ui.paint_str(
                Rect::new(screen.x, screen.y, 1, 1),
                "P",
                Style::new()
                    .fg(self.page_fg)
                    .bg(self.page_bg)
                    .underline_color(self.page_underline)
                    .add_modifier(Modifier::BOLD),
            );
            ui.paint_str(
                Rect::new(
                    screen.right().saturating_sub(1),
                    screen.bottom().saturating_sub(1),
                    1,
                    1,
                ),
                "Q",
                Style::new()
                    .fg(self.page_fg)
                    .bg(self.page_bg)
                    .underline_color(self.page_underline)
                    .add_modifier(Modifier::BOLD),
            );
        }

        // Draw the upper layer first in App order; Runtime still composites the
        // opened lower layer before this upper layer.
        let _ = ui.layer(UPPER_LAYER, |ui, area| {
            ui.fill_keep_ink(area, self.upper_bg);
            let x = if area.width >= 4 {
                area.x.saturating_add(1)
            } else {
                area.x
            };
            let target = Rect::new(x, area.y, 1, 1);
            let target_fg = if self.operation == RetintOperation::DirectReference
                && self.glyph_fg == self.from
            {
                self.to
            } else {
                self.glyph_fg
            };
            ui.paint_str(
                target,
                "M",
                Style::new().fg(target_fg).add_modifier(Modifier::ITALIC),
            );

            if area.width >= 4 {
                ui.paint_str(
                    Rect::new(x.saturating_add(1), area.y, 1, 1),
                    "N",
                    Style::new().fg(self.no_match_fg),
                );
            }

            match self.operation {
                RetintOperation::LegacyRaw => {
                    // This deliberately mirrors the current one-clip raw
                    // acquisition and foreground-only scan.
                    let stats = Rc::clone(&self.stats);
                    let (buffer, clipped) = ui.raw();
                    for pos in clipped.positions() {
                        let Some(cell) = buffer.cell_mut(pos) else {
                            continue;
                        };
                        let fg = cell.fg;
                        let mut stats = stats.borrow_mut();
                        stats.visited += 1;
                        if fg == self.from {
                            stats.matching += 1;
                            if fg != self.to {
                                stats.changed += 1;
                            }
                            cell.set_fg(self.to);
                        }
                    }
                }
                RetintOperation::RecolorForeground => {
                    let from = self.from;
                    let to = self.to;
                    let stats = Rc::clone(&self.stats);
                    ui.recolor_fg(ui.full(), move |fg, _bg| {
                        let mut stats = stats.borrow_mut();
                        stats.visited += 1;
                        if fg == from {
                            stats.matching += 1;
                            if fg != to {
                                stats.changed += 1;
                            }
                            Some(to)
                        } else {
                            None
                        }
                    });
                }
                RetintOperation::DirectReference => {}
            }
        });

        let _ = ui.layer(LOWER_LAYER, |ui, area| {
            ui.fill(
                area,
                Style::new()
                    .fg(self.lower_fg)
                    .bg(self.lower_bg)
                    .underline_color(self.lower_underline)
                    .add_modifier(Modifier::UNDERLINED),
            );
        });
    }
}

struct Rendered {
    harness: Harness<LayeredRetint>,
    stats: Rc<RefCell<ScanStats>>,
    facts: Facts,
}

fn choose_color(excluded: &[Color], candidates: [Color; 9]) -> Color {
    candidates
        .into_iter()
        .find(|candidate| !excluded.contains(candidate))
        .unwrap_or(Color::Reset)
}

fn render_layered(
    geometry: Geometry,
    level: ColorLevel,
    probe: Probe,
    operation: RetintOperation,
) -> Rendered {
    let theme = Theme::junie().for_level(level);
    let colors = theme.color;
    let (from, to, glyph_fg) = match probe {
        Probe::Matching => (colors.danger, colors.warning, colors.danger),
        Probe::NoMatch => (Color::Rgb(1, 2, 3), colors.warning, colors.danger),
        Probe::EqualColor => (colors.danger, colors.danger, colors.danger),
    };
    let no_match_fg = choose_color(
        &[from, glyph_fg],
        [
            colors.fg[0],
            colors.fg[1],
            colors.fg[2],
            colors.fg[4],
            colors.info,
            colors.success,
            Color::Rgb(7, 8, 9),
            Color::Rgb(10, 11, 12),
            Color::White,
        ],
    );
    let page_fg = choose_color(
        &[from, to, glyph_fg, no_match_fg],
        [
            colors.fg[0],
            colors.fg[1],
            colors.fg[2],
            colors.fg[3],
            colors.fg[4],
            colors.info,
            colors.success,
            Color::Rgb(13, 14, 15),
            Color::White,
        ],
    );
    let lower_fg = choose_color(
        &[from, to, glyph_fg, page_fg],
        [
            colors.fg[4],
            colors.fg[3],
            colors.fg[2],
            colors.info,
            colors.success,
            Color::Rgb(17, 18, 19),
            Color::Rgb(20, 21, 22),
            Color::DarkGray,
            Color::Black,
        ],
    );
    let stats = Rc::new(RefCell::new(ScanStats::default()));
    let app = LayeredRetint {
        geometry,
        operation,
        from,
        to,
        glyph_fg,
        no_match_fg,
        page_fg,
        page_bg: colors.surfaces[0],
        page_underline: colors.info,
        lower_fg,
        lower_bg: colors.surfaces[1],
        lower_underline: colors.success,
        upper_bg: colors.surfaces[4],
        stats: Rc::clone(&stats),
    };
    let (width, height) = geometry.screen();
    let harness = Harness::new(app, theme, width, height);
    Rendered {
        harness,
        stats,
        facts: Facts {
            from,
            to,
            glyph_fg,
            no_match_fg,
            page_fg,
            page_underline: colors.info,
            lower_fg,
            lower_underline: colors.success,
            upper_bg: colors.surfaces[4],
        },
    }
}

fn layer_areas(rendered: &Rendered) -> (Rect, Rect, Rect) {
    let screen = *rendered.harness.buffer().area();
    let lower = rendered
        .harness
        .layer_area(LOWER_LAYER)
        .expect("lower layer was opened and drawn");
    let upper = rendered
        .harness
        .layer_area(UPPER_LAYER)
        .expect("upper layer was opened and drawn");
    assert!(!lower.is_empty(), "resolved lower layer must be nonempty");
    assert!(!upper.is_empty(), "resolved upper layer must be nonempty");
    assert_eq!(lower.intersection(screen), lower);
    assert_eq!(upper.intersection(screen), upper);
    (screen, lower, upper)
}

fn target_position(upper: Rect) -> Position {
    Position::new(
        if upper.width >= 4 {
            upper.x.saturating_add(1)
        } else {
            upper.x
        },
        upper.y,
    )
}

fn assert_scan(rendered: &Rendered, area: Rect, expected_matches: usize) {
    let stats = *rendered.stats.borrow();
    assert_eq!(
        stats.visited,
        usize::from(area.width) * usize::from(area.height)
    );
    assert_eq!(stats.matching, expected_matches);
    let expected_changes = if rendered.facts.from == rendered.facts.to {
        0
    } else {
        expected_matches
    };
    assert_eq!(stats.changed, expected_changes);
}

fn verify_layered_case(level: ColorLevel, probe: Probe, geometry: Geometry) {
    let legacy = render_layered(geometry, level, probe, RetintOperation::LegacyRaw);
    let recolored = render_layered(geometry, level, probe, RetintOperation::RecolorForeground);
    let direct = render_layered(geometry, level, probe, RetintOperation::DirectReference);
    let (screen, lower, upper) = layer_areas(&legacy);
    let (_, recolored_lower, recolored_upper) = layer_areas(&recolored);
    let (_, direct_lower, direct_upper) = layer_areas(&direct);
    assert_eq!(lower, recolored_lower);
    assert_eq!(lower, direct_lower);
    assert_eq!(upper, recolored_upper);
    assert_eq!(upper, direct_upper);

    let facts = legacy.facts;
    assert_ne!(facts.page_underline, facts.lower_underline);
    let target = target_position(upper);
    assert!(lower.contains(target), "target must overlap both layers");
    assert_eq!(recolored.facts.from, facts.from);
    assert_eq!(recolored.facts.to, facts.to);
    assert_eq!(direct.facts.glyph_fg, facts.glyph_fg);
    assert_eq!(direct.facts.page_fg, facts.page_fg);
    assert_eq!(direct.facts.no_match_fg, facts.no_match_fg);

    let expected_matches = usize::from(facts.glyph_fg == facts.from);
    assert_scan(&legacy, upper, expected_matches);
    assert_scan(&recolored, upper, expected_matches);
    assert_eq!(*direct.stats.borrow(), ScanStats::default());
    if probe == Probe::EqualColor {
        assert_eq!(facts.from, facts.to, "equal-color control must be a no-op");
        assert_eq!(
            expected_matches, 1,
            "equal-color cell still exercises the predicate"
        );
    }
    if level == ColorLevel::Mono && probe == Probe::Matching {
        assert_eq!(
            facts.from, facts.to,
            "Junie Mono danger and warning alias here"
        );
        assert_eq!(expected_matches, 1);
        assert_eq!(recolored.stats.borrow().changed, 0);
    }
    if probe == Probe::NoMatch {
        assert_ne!(facts.glyph_fg, facts.from);
        assert_eq!(expected_matches, 0, "no-match control must not be vacuous");
    }

    assert_eq!(
        recolored.harness.buffer(),
        direct.harness.buffer(),
        "full-screen Cells must match for {:?} {:?} {:?}",
        level,
        probe,
        geometry,
    );
    let target_cell = direct
        .harness
        .buffer()
        .cell(target)
        .expect("target cell is in the resolved upper layer");
    let expected_target_fg = if facts.glyph_fg == facts.from {
        facts.to
    } else {
        facts.glyph_fg
    };
    assert_eq!(target_cell.symbol(), "M");
    assert_eq!(target_cell.fg, expected_target_fg);
    assert_eq!(target_cell.bg, facts.upper_bg);
    assert_eq!(target_cell.underline_color, facts.lower_underline);
    assert_eq!(
        target_cell.modifier,
        Modifier::UNDERLINED | Modifier::ITALIC
    );

    let legacy_target = legacy
        .harness
        .buffer()
        .cell(target)
        .expect("legacy target cell is present");
    assert_eq!(legacy_target.symbol(), "M");
    assert_eq!(legacy_target.fg, expected_target_fg);
    assert_eq!(legacy_target.bg, facts.upper_bg);
    assert_eq!(legacy_target.underline_color, facts.page_underline);
    assert_eq!(legacy_target.modifier, Modifier::BOLD | Modifier::ITALIC);
    assert_ne!(legacy_target, target_cell);

    if upper.width >= 4 {
        let no_match = Position::new(target.x.saturating_add(1), target.y);
        let blank = Position::new(target.x.saturating_add(2), target.y);
        assert!(upper.contains(no_match));
        assert!(upper.contains(blank));
        assert!(lower.contains(no_match));
        assert!(lower.contains(blank));

        let n_cell = direct
            .harness
            .buffer()
            .cell(no_match)
            .expect("no-match glyph cell is present");
        assert_eq!(n_cell.symbol(), "N");
        assert_eq!(n_cell.fg, facts.no_match_fg);
        assert_ne!(n_cell.fg, facts.from);
        assert_eq!(n_cell.bg, facts.upper_bg);
        assert_eq!(n_cell.underline_color, facts.lower_underline);
        assert_eq!(n_cell.modifier, Modifier::UNDERLINED);

        let blank_cell = direct
            .harness
            .buffer()
            .cell(blank)
            .expect("blank provenance witness is present");
        assert_eq!(blank_cell.symbol(), " ");
        assert_eq!(blank_cell.fg, facts.lower_fg);
        assert_eq!(blank_cell.bg, facts.upper_bg);
        assert_eq!(blank_cell.underline_color, facts.lower_underline);
        assert_eq!(blank_cell.modifier, Modifier::UNDERLINED);

        let legacy_blank = legacy
            .harness
            .buffer()
            .cell(blank)
            .expect("legacy blank witness is present");
        assert_eq!(legacy_blank.symbol(), " ");
        assert_eq!(legacy_blank.fg, facts.page_fg);
        assert_eq!(legacy_blank.bg, facts.upper_bg);
        assert_eq!(legacy_blank.underline_color, facts.page_underline);
        assert_eq!(legacy_blank.modifier, Modifier::BOLD);
        assert_ne!(legacy_blank, blank_cell);
    }

    for pos in screen.positions().filter(|pos| !upper.contains(*pos)) {
        assert_eq!(
            legacy.harness.buffer().cell(pos),
            direct.harness.buffer().cell(pos),
            "legacy operation must leave complete Cells outside its resolved clip unchanged at {pos:?}",
        );
        assert_eq!(
            recolored.harness.buffer().cell(pos),
            direct.harness.buffer().cell(pos),
            "recolor operation must leave complete Cells outside its resolved clip unchanged at {pos:?}",
        );
    }

    if geometry != Geometry::Tiny {
        let sentinel_candidates = [
            (Position::new(screen.x, screen.y), "P"),
            (
                Position::new(
                    screen.right().saturating_sub(1),
                    screen.bottom().saturating_sub(1),
                ),
                "Q",
            ),
        ];
        let outside_sentinels: Vec<_> = sentinel_candidates
            .into_iter()
            .filter(|(pos, _)| !lower.contains(*pos) && !upper.contains(*pos))
            .collect();
        assert!(
            !outside_sentinels.is_empty(),
            "geometry must retain a sentinel outside both layers"
        );
        for (pos, symbol) in outside_sentinels {
            assert_eq!(direct.harness.buffer().cell(pos).unwrap().symbol(), symbol);
            assert_eq!(
                legacy.harness.buffer().cell(pos),
                direct.harness.buffer().cell(pos)
            );
            assert_eq!(
                recolored.harness.buffer().cell(pos),
                direct.harness.buffer().cell(pos)
            );
        }
    }

    match geometry {
        Geometry::Offset => {
            assert_ne!(upper.x, screen.x);
            assert_ne!(upper.y, screen.y);
            assert!(upper.right() < screen.right());
            assert!(upper.bottom() < screen.bottom());
        }
        Geometry::Tiny => {
            assert_eq!(screen, Rect::new(0, 0, 1, 1));
            assert_eq!(upper, screen);
            assert_eq!(lower, screen);
        }
        Geometry::Clamped => {
            assert!(
                upper.width < 14,
                "requested width must be reduced by resolver"
            );
            assert_eq!(upper.width, screen.width);
            assert!(upper.contains(target));
        }
    }
}

#[test]
fn foreground_recolor_matches_direct_layer_composition_across_capabilities() {
    let levels = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    let probes = [Probe::Matching, Probe::NoMatch, Probe::EqualColor];
    for level in levels {
        for probe in probes {
            verify_layered_case(level, probe, Geometry::Offset);
        }
    }
}

#[test]
fn raw_recolor_and_direct_reference_use_actual_tiny_and_clamped_layer_geometry() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        verify_layered_case(level, Probe::Matching, Geometry::Tiny);
    }
    for probe in [Probe::Matching, Probe::NoMatch, Probe::EqualColor] {
        verify_layered_case(ColorLevel::TrueColor, probe, Geometry::Clamped);
    }
}
