//! The page chrome paints its palette at the frame's colour level.
//!
//! The chrome blocks take a truecolour palette from the page. A terminal
//! with fewer colours cannot show those inks, so the block reduces them
//! before it paints: a reduced frame carries palette entries, never the
//! truecolour literals. The frozen reference derives its own reduced inks
//! the same way (`visual-baseline:src/theme.rs`, `nearest_16` and
//! `nearest_256`): the faint ink is Gray and the ghost ink is DarkGray at 16
//! colours.

use ratatui_core::layout::Position;
use ratatui_core::style::Color;
use termrock::feedback::{ChromeInk, SectionStrip};
use termrock::{ColorLevel, Theme};
use termrock_test_support::Scene;

const TABS: [&str; 3] = ["Playground", "Options", "States"];

fn ink() -> ChromeInk {
    ChromeInk {
        canvas: ChromeInk::rgb(0x000000),
        surface: ChromeInk::rgb(0x111111),
        ink: ChromeInk::rgb(0xffffff),
        secondary: ChromeInk::rgb(0xb3b3b3),
        muted: ChromeInk::rgb(0x8a8a8a),
        faint: ChromeInk::rgb(0x737373),
        rule: ChromeInk::rgb(0x3a3a3a),
        accent: ChromeInk::rgb(0x48e054),
        tab_active: ChromeInk::rgb(0x27272a),
    }
}

fn strip(level: ColorLevel) -> Scene {
    let mut scene = Scene::new("page-chrome", Theme::junie(), level, 40, 2);
    scene.draw(|ui, area| {
        SectionStrip::new(&TABS).ink(ink()).draw(ui, area);
    });
    scene
}

fn cell(scene: &Scene, x: u16, y: u16) -> (Color, Color) {
    let cell = scene.buffer().cell(Position::new(x, y)).expect("cell");
    (cell.fg, cell.bg)
}

#[test]
fn a_reduced_frame_paints_palette_entries_not_truecolour_literals() {
    for level in [ColorLevel::Ansi256, ColorLevel::Ansi16, ColorLevel::Mono] {
        let scene = strip(level);
        for y in 0..2u16 {
            for x in 0..40u16 {
                let (fg, bg) = cell(&scene, x, y);
                assert!(
                    !matches!(fg, Color::Rgb(..)) && !matches!(bg, Color::Rgb(..)),
                    "{level:?} cell ({x}, {y}) still paints a truecolour literal: {fg:?} on {bg:?}"
                );
            }
        }
    }
}

#[test]
fn the_sixteen_colour_strip_follows_the_reference_ink_table() {
    let scene = strip(ColorLevel::Ansi16);
    // The active tab: the page's primary ink on the overlay surface.
    assert_eq!(cell(&scene, 0, 0), (Color::White, Color::DarkGray));
    // An inactive tab: the secondary ink on the canvas.
    assert_eq!(cell(&scene, 14, 0), (Color::Gray, Color::Black));
    // The rule under the tabs: the subtle border ink.
    assert_eq!(cell(&scene, 30, 1), (Color::DarkGray, Color::Black));
    // The active tab's share of the rule: the accent.
    assert_eq!(cell(&scene, 0, 1), (Color::LightGreen, Color::Black));
}

#[test]
fn the_two_hundred_fifty_six_colour_strip_follows_the_reference_ink_table() {
    let scene = strip(ColorLevel::Ansi256);
    assert_eq!(
        cell(&scene, 0, 0),
        (Color::Indexed(231), Color::Indexed(235))
    );
    assert_eq!(
        cell(&scene, 14, 0),
        (Color::Indexed(249), Color::Indexed(16))
    );
    assert_eq!(cell(&scene, 0, 1), (Color::Indexed(78), Color::Indexed(16)));
}

#[test]
fn the_mono_strip_follows_the_frozen_mono_table() {
    let scene = strip(ColorLevel::Mono);
    // The frozen `--color none` table paints the primary ink on the overlay
    // surface black on dark gray.
    assert_eq!(cell(&scene, 0, 0), (Color::Black, Color::DarkGray));
    assert_eq!(cell(&scene, 14, 0), (Color::Gray, Color::Black));
}
