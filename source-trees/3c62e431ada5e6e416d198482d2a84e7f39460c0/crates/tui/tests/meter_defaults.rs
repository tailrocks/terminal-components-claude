//! Meter role defaults must remain below caller customization.
use junie_tui::{
    ColorLevel, Family, Id, Meter, MeterTone, MeterVisual, Part, Role, StylePatch, Theme,
};
use junie_tui_testing::Scene;
use ratatui_core::style::Color;
const ID: Id = Id::root("meter.defaults");
const SCOPE: [junie_tui::OverlayRule; 1] = [(
    Family::METER,
    junie_tui::Variant::DEFAULT,
    Part::THUMB,
    junie_tui::StateFlags::empty(),
    StylePatch::new()
        .set_fg(Role::Custom(Color::Blue))
        .set_bg(Role::Custom(Color::Cyan)),
)];
#[test]
fn thumb_local_override_survives_every_tone_and_visual() {
    let patches = [(
        Part::THUMB,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Red))
            .set_bg(Role::Custom(Color::Blue)),
    )];
    for tone in [
        MeterTone::Low,
        MeterTone::Medium,
        MeterTone::High,
        MeterTone::Stale,
        MeterTone::Unknown,
        MeterTone::Series(2),
    ] {
        for visual in [MeterVisual::Line, MeterVisual::Block] {
            let mut scene = Scene::new(
                "meter_override",
                Theme::junie(),
                ColorLevel::TrueColor,
                30,
                1,
            );
            scene.draw(|ui, area| {
                Meter::new(ID)
                    .ratio(0.5)
                    .value("50%")
                    .tone(tone)
                    .visual(visual)
                    .patch_part(&patches)
                    .draw(ui, area);
            });
            assert_eq!(
                scene.buffer().cell((0, 0)).map(|c| (c.fg, c.bg)),
                Some((Color::Red, Color::Blue)),
                "{tone:?} {visual:?}"
            );
        }
    }
}
#[test]
fn capture_unchanged_default_cells() {
    let mut hashes = Vec::new();
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for visual in [MeterVisual::Line, MeterVisual::Block] {
                let mut scene = Scene::new("meter_base", theme.clone(), level, 30, 1);
                scene.draw(|ui, area| {
                    Meter::new(ID)
                        .ratio(0.5)
                        .value("50%")
                        .visual(visual)
                        .draw(ui, area);
                });
                hashes.push(scene.digest());
            }
        }
    }
    // Pinned after d52ee38b/29ed8e5e: the junie palette meters its unfilled
    // share through the reference lift, which moves the junie truecolor and
    // ansi256 rows plus the block ansi16 row; mono and the paper palette keep
    // their pinned values (paper fills with an explicit color).
    assert_eq!(
        hashes,
        [
            10_772_714_535_040_698_230,
            7_536_098_631_154_068_646,
            17_260_405_213_946_973_771,
            8_246_685_266_941_457_616,
            14_818_118_036_683_959_920,
            16_562_180_527_847_283_635,
            3_973_918_168_754_095_168,
            1_980_463_968_318_773_821,
            13_755_095_515_169_827_076,
            15_886_380_626_526_385_761,
            15_862_199_869_115_132_544,
            4_620_123_589_686_111_913,
            17_447_104_656_905_611_786,
            2_205_268_342_521_374_261,
            7_812_222_032_887_116_840,
            6_390_485_324_615_177_875
        ]
    );
}

#[test]
fn authored_defaults_stay_below_theme_scope_and_instance_for_each_tone() {
    let defaults = [(
        Part::THUMB,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Yellow))
            .set_bg(Role::Custom(Color::Magenta)),
    )];

    for tone in [
        MeterTone::Low,
        MeterTone::Medium,
        MeterTone::High,
        MeterTone::Stale,
        MeterTone::Unknown,
        MeterTone::Series(2),
    ] {
        for visual in [MeterVisual::Line, MeterVisual::Block] {
            for level in 0..4 {
                let theme = if level > 0 {
                    Theme::junie().override_family(Family::METER, |r| {
                        r.part(Part::THUMB).base(
                            StylePatch::new()
                                .set_fg(Role::Custom(Color::Red))
                                .set_bg(Role::Custom(Color::Green)),
                        );
                    })
                } else {
                    Theme::junie()
                };
                let local = StylePatch::new()
                    .set_fg(Role::Custom(Color::White))
                    .set_bg(Role::Custom(Color::Black));
                let mut scene = Scene::new("meter_precedence", theme, ColorLevel::TrueColor, 30, 1);
                scene.draw(|ui, area| {
                    let paint = |ui: &mut junie_tui::Ui<'_>| {
                        let m = Meter::new(ID)
                            .ratio(0.5)
                            .value("50%")
                            .tone(tone)
                            .visual(visual)
                            .part_defaults(&defaults);
                        let m = if level == 3 { m.patch(&local) } else { m };
                        m.draw(ui, area);
                    };
                    if level >= 2 {
                        ui.with_overlay(&junie_tui::Overlay::new(&SCOPE), paint);
                    } else {
                        paint(ui);
                    }
                });
                let expected = match level {
                    0 => (Color::Yellow, Color::Magenta),
                    1 => (Color::Red, Color::Green),
                    2 => (Color::Blue, Color::Cyan),
                    _ => (Color::White, Color::Black),
                };
                assert_eq!(
                    scene.buffer().cell((0, 0)).map(|c| (c.fg, c.bg)),
                    Some(expected),
                    "{tone:?} {visual:?} level{level}"
                );
            }
        }
    }
}

#[test]
fn authored_label_and_icon_defaults_yield_to_readiness_and_explicit_parts() {
    let defaults = [
        (
            Part::LABEL,
            StylePatch::new().set_fg(Role::Custom(Color::Yellow)),
        ),
        (
            Part::ICON,
            StylePatch::new().set_glyph(junie_tui::GlyphRole::WarningMark),
        ),
    ];
    for status in [junie_tui::Status::Ready, junie_tui::Status::Error] {
        let mut scene = Scene::new(
            "meter_authorparts",
            Theme::junie(),
            ColorLevel::TrueColor,
            30,
            1,
        );
        scene.draw(|ui, area| {
            Meter::new(ID)
                .ratio(0.5)
                .value("X")
                .status(status)
                .part_defaults(&defaults)
                .draw(ui, area);
        });
        assert!(
            scene
                .buffer()
                .content
                .iter()
                .any(|c| c.symbol() == "X" && c.fg == Color::Yellow)
        );
        let expected = if status == junie_tui::Status::Error {
            "!"
        } else {
            "▲"
        };
        assert!(
            scene
                .buffer()
                .content
                .iter()
                .any(|c| c.symbol() == expected)
        );
    }
}
