#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]
//! Temporary adjudication probe for d1ac0c0dac pad-painting drift.
//! Not part of the permanent suite.

use termrock::{Button, ColorLevel, Id, Rect, Theme, Ui};

const BTN: Id = Id::root("render.button");

fn draw_button(ui: &mut Ui<'_>, area: Rect) {
    Button::new(BTN, "Run task")
        .variant(termrock::Variant::PRIMARY)
        .draw(ui, area);
}

#[test]
fn probe_button_default_frame() {
    for (name, theme, color, w, h) in [
        ("120x40 junie truecolor", Theme::junie(), ColorLevel::TrueColor, 120, 40),
        ("40x10 junie truecolor", Theme::junie(), ColorLevel::TrueColor, 40, 10),
    ] {
        let mut scene = termrock_test_support::Scene::new("probe", theme, color, w, h);
        scene.draw(|ui, area| draw_button(ui, area));
        let digest = scene.digest();
        eprintln!("probe {name}: digest {digest:016x}");
        let buf = scene.buffer();
        for x in 0..16u16 {
            let c = buf.cell((x, 0)).expect("cell");
            eprintln!(
                "  x={x:2} sym={:?} fg={:?} bg={:?} mods={}",
                c.symbol(),
                c.fg,
                c.bg,
                c.modifier.bits()
            );
        }
    }
}

#[test]
fn probe_button_pressed_frame() {
    let mut scene = termrock_test_support::Scene::new(
        "probe-pressed",
        Theme::junie(),
        ColorLevel::TrueColor,
        120,
        40,
    );
    scene.draw(|ui, area| {
        ui.reference(
            Some(
                termrock::ReferenceTarget::new(
                    BTN,
                    termrock::ReferenceState::PRESSED | termrock::ReferenceState::FOCUSED,
                ),
            ),
            |ui| {
                Button::new(BTN, "Run task")
                    .variant(termrock::Variant::PRIMARY)
                    .draw(ui, area);
            },
        );
    });
    let digest = scene.digest();
    eprintln!("probe pressed: digest {digest:016x}");
    let buf = scene.buffer();
    for x in 0..16u16 {
        let c = buf.cell((x, 0)).expect("cell");
        eprintln!(
            "  x={x:2} sym={:?} fg={:?} bg={:?} mods={}",
            c.symbol(),
            c.fg,
            c.bg,
            c.modifier.bits()
        );
    }
}
