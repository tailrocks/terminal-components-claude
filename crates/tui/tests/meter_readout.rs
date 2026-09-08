//! Shared meter suffix and leading activity geometry.
use junie_tui::{ColorLevel, Constraints, FrameRead, Id, Meter, MeterVisual, Part, Status, Theme};
use junie_tui_testing::Scene;
use std::cell::Cell;
const ID: Id = Id::root("meter.readout");
#[test]
fn fixed_minimum_suffix_matches_reference_line_and_block_extent() {
    for visual in [MeterVisual::Line, MeterVisual::Block] {
        let seen = Cell::new(0);
        let track = |_: &mut junie_tui::Ui<'_>, rect: junie_tui::Rect| seen.set(rect.width);
        let mut scene = Scene::new("meter_suffix", Theme::junie(), ColorLevel::TrueColor, 30, 1);
        scene.draw(|ui, area| {
            Meter::new(ID)
                .ratio(0.5)
                .value("50%")
                .visual(visual)
                .suffix_width(2)
                .slot(Part::TRACK, &track)
                .draw(ui, area);
        });
        assert_eq!(
            seen.get(),
            if visual == MeterVisual::Line { 24 } else { 28 }
        );
    }
}
#[test]
fn leading_busy_activity_is_prefix_not_suffix_and_measure_reserves_it() {
    let mut scene = Scene::new(
        "meter_activity",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        1,
    );
    scene.draw(|ui, area| {
        let meter = Meter::new(ID)
            .ratio(0.5)
            .value("refreshing")
            .status(Status::Loading)
            .leading_activity(true)
            .suffix_width(2);
        assert_eq!(
            meter.measure(ui, Constraints::loose(100, 1)).preferred.0,
            ui.design().size.meter_track.max(6) + 15
        );
        meter.draw(ui, area);
    });
    assert_eq!(
        scene.buffer().cell((16, 0)).map(junie_tui::Cell::symbol),
        Some("⠋")
    );
    assert_eq!(
        scene.buffer().cell((18, 0)).map(junie_tui::Cell::symbol),
        Some("r")
    );
    assert_eq!(
        scene.buffer().cell((29, 0)).map(junie_tui::Cell::symbol),
        Some(" ")
    );
}
#[test]
fn custom_wide_suffix_and_icon_slot_keep_authoritative_budgets() {
    let mut theme = Theme::junie();
    theme.design.motion.spinner_frames = &["ab"];
    let seen = Cell::new(junie_tui::Rect::default());
    let icon = |ui: &mut junie_tui::Ui<'_>, rect: junie_tui::Rect| {
        seen.set(rect);
        ui.paint_str(rect, "ZZ", ui.surface_style());
    };
    let mut scene = Scene::new("meter_wide_activity", theme, ColorLevel::TrueColor, 30, 1);
    scene.draw(|ui, area| {
        Meter::new(ID)
            .ratio(0.5)
            .value("X")
            .status(Status::Busy)
            .leading_activity(true)
            .suffix_width(2)
            .slot(Part::ICON, &icon)
            .draw(ui, area);
    });
    assert_eq!(seen.get().width, 2);
    assert_eq!(
        scene.buffer().cell((27, 0)).map(junie_tui::Cell::symbol),
        Some("X")
    );
}

#[test]
fn minimum_suffix_never_clips_a_wider_authored_glyph() {
    let theme = Theme::junie()
        .builder()
        .glyph(junie_tui::GlyphRole::WarningMark, "!!")
        .build();
    let defaults = [(
        Part::ICON,
        junie_tui::StylePatch::new().set_glyph(junie_tui::GlyphRole::WarningMark),
    )];
    let mut scene = Scene::new("meter_wide_suffix", theme, ColorLevel::TrueColor, 30, 1);
    scene.draw(|ui, area| {
        let m = Meter::new(ID)
            .ratio(0.5)
            .value("X")
            .part_defaults(&defaults)
            .suffix_width(1);
        assert_eq!(m.measure(ui, Constraints::loose(100, 1)).min.0, 11);
        m.draw(ui, area);
    });
    assert_eq!(
        scene.buffer().cell((28, 0)).map(junie_tui::Cell::symbol),
        Some("!")
    );
    assert_eq!(
        scene.buffer().cell((29, 0)).map(junie_tui::Cell::symbol),
        Some("!")
    );
}
#[test]
fn value_only_readout_reserves_marker_space_under_clipping() {
    let mut scene = Scene::new(
        "meter_value_only",
        Theme::junie(),
        ColorLevel::TrueColor,
        4,
        1,
    );
    scene.draw(|ui, area| {
        let m = Meter::new(ID)
            .value("abcdef")
            .status(Status::Error)
            .suffix_width(2);
        assert_eq!(m.measure(ui, Constraints::loose(100, 1)).preferred, (8, 1));
        m.draw(ui, area);
    });
    assert_eq!(
        scene.buffer().cell((3, 0)).map(junie_tui::Cell::symbol),
        Some("!")
    );
}
