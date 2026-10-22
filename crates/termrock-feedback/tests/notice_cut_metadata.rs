use ratatui_core::layout::Rect;
use termrock::author::VerticalWindow;
use termrock::feedback::{Notice, NoticeCut, NoticeSource};
use termrock::theme::{ColorLevel, GlyphRole, Theme};
use termrock_test_support::Scene;

fn collect_cuts<'a>(
    notice: Notice<'a>,
    theme: Theme,
    width: u16,
    logical_height: u16,
) -> Vec<NoticeCut<'a>> {
    let mut scene = Scene::new("notice-cut-metadata", theme, ColorLevel::TrueColor, 80, 24);
    let mut cuts = Vec::new();
    scene.draw(|ui, _| {
        notice.visit_cuts(ui, width, logical_height, |cut| cuts.push(cut));
    });
    cuts
}

fn summary<'a>(cuts: &[NoticeCut<'a>]) -> Vec<(NoticeSource, u16, u16, &'a str)> {
    cuts.iter()
        .map(|cut| {
            (
                cut.source(),
                cut.logical_row(),
                cut.inset(),
                cut.original_text(),
            )
        })
        .collect()
}

#[test]
fn each_cut_source_reports_its_full_borrowed_text_and_exact_logical_row() {
    let message = "message first second";
    let detail = "detail first second";
    let hint = "hint first second";
    let notice = Notice::warning(message)
        .message_lines(1)
        .detail(detail)
        .max_lines(1)
        .hint(hint)
        .marker("界");

    let cuts = collect_cuts(notice, Theme::junie(), 14, 3);

    assert_eq!(
        summary(&cuts),
        [
            (NoticeSource::Message, 0, 3, message),
            (NoticeSource::Detail, 1, 2, detail),
            (NoticeSource::Hint, 2, 2, hint),
        ]
    );
    assert_eq!(cuts[0].original_text().as_ptr(), message.as_ptr());
    assert_eq!(cuts[1].original_text().as_ptr(), detail.as_ptr());
    assert_eq!(cuts[2].original_text().as_ptr(), hint.as_ptr());
}

#[test]
fn fitting_and_suppressed_sources_are_not_reported_and_caps_keep_existing_semantics() {
    let fitting = Notice::new("fits").detail("also fits").hint("short hint");
    assert!(collect_cuts(fitting, Theme::junie(), 40, 5).is_empty());

    let empty_sources = Notice::new("").detail("").max_lines(2).hint("");
    assert!(collect_cuts(empty_sources, Theme::junie(), 8, 5).is_empty());

    let message_cap_zero = Notice::new("one two three").message_lines(0);
    assert_eq!(
        summary(&collect_cuts(message_cap_zero, Theme::junie(), 8, 4)),
        [(NoticeSource::Message, 0, 2, "one two three")]
    );

    let detail_suppressed = Notice::new("base")
        .detail("this detail would wrap across many rows")
        .max_lines(0);
    assert!(collect_cuts(detail_suppressed, Theme::junie(), 8, 5).is_empty());

    let detail_multiple_rows = Notice::new("base")
        .detail("first\nsecond\nthird")
        .max_lines(2);
    assert_eq!(
        summary(&collect_cuts(detail_multiple_rows, Theme::junie(), 40, 4)),
        [(NoticeSource::Detail, 2, 2, "first\nsecond\nthird")]
    );
}

#[test]
fn logical_height_cuts_are_distinct_from_physical_window_crops() {
    let message = "row zero\nrow one\nrow two";
    let notice = Notice::new(message).message_lines(3);

    assert!(collect_cuts(notice, Theme::junie(), 30, 3).is_empty());
    assert_eq!(
        summary(&collect_cuts(notice, Theme::junie(), 30, 2)),
        [(NoticeSource::Message, 1, 2, message)]
    );

    let physical_crop = VerticalWindow::new(Rect::new(4, 9, 30, 1), 3, 1)
        .expect("one physical row cropped from a three-row logical notice");
    assert_eq!(physical_crop.visible_source_range(), 1..2);
    assert!(
        collect_cuts(
            notice,
            Theme::junie(),
            physical_crop.visible_body().width,
            u16::try_from(physical_crop.logical_rows()).expect("fixture height fits u16"),
        )
        .is_empty(),
        "a physical viewport edge must not create a logical cut"
    );

    let logical_cap = VerticalWindow::new(Rect::new(4, 9, 30, 1), 2, 1)
        .expect("a one-row view of a two-row logical cap");
    let cuts = collect_cuts(
        notice,
        Theme::junie(),
        logical_cap.visible_body().width,
        u16::try_from(logical_cap.logical_rows()).expect("fixture height fits u16"),
    );
    assert_eq!(summary(&cuts), [(NoticeSource::Message, 1, 2, message)]);
    assert_eq!(
        logical_cap.physical_row_for_source(usize::from(cuts[0].logical_row())),
        Some(9)
    );

    let top_crop = VerticalWindow::new(Rect::new(4, 9, 30, 1), 2, 0)
        .expect("the physical crop excludes the logically cut second row");
    let top_cuts = collect_cuts(
        notice,
        Theme::junie(),
        top_crop.visible_body().width,
        u16::try_from(top_crop.logical_rows()).expect("fixture height fits u16"),
    );
    assert_eq!(summary(&top_cuts), [(NoticeSource::Message, 1, 2, message)]);
    assert_eq!(
        top_crop.physical_row_for_source(usize::from(top_cuts[0].logical_row())),
        None,
        "the later window adapter must suppress a logically cut off-window row"
    );
}

#[test]
fn metadata_ignores_the_visible_mark_and_uses_unicode_and_custom_marker_geometry() {
    let message = "界e\u{301}\nnext";
    let marked = Notice::new(message)
        .message_lines(2)
        .marker("界")
        .mark_cut(true);
    let hidden_mark = marked.mark_cut(false);
    let expected = [
        (NoticeSource::Message, 0, 3, message),
        (NoticeSource::Message, 1, 3, message),
    ];

    assert_eq!(
        summary(&collect_cuts(marked, Theme::junie(), 3, 2)),
        expected
    );
    assert_eq!(
        summary(&collect_cuts(hidden_mark, Theme::junie(), 3, 2)),
        expected
    );
    assert!(
        collect_cuts(hidden_mark, Theme::junie(), 0, 2).is_empty(),
        "zero-width drawing area is inert, as in draw"
    );

    let mut theme = Theme::junie();
    theme.design.glyphs.set(GlyphRole::WarningMark, "界");
    let theme_marker = Notice::warning("themed marker").message_lines(1);
    assert_eq!(
        summary(&collect_cuts(theme_marker, theme, 3, 1)),
        [(NoticeSource::Message, 0, 3, "themed marker")]
    );

    let narrow = Notice::new("narrow").message_lines(1);
    assert_eq!(
        summary(&collect_cuts(narrow, Theme::junie(), 1, 1)),
        [(NoticeSource::Message, 0, 2, "narrow")]
    );
}

#[test]
fn observing_cut_metadata_does_not_paint_register_or_move_the_cursor() {
    let notice = Notice::new("message first second").message_lines(1);
    let mut baseline = Scene::new(
        "notice-cut-baseline",
        Theme::junie(),
        ColorLevel::TrueColor,
        28,
        4,
    );
    baseline.draw(|ui, area| notice.draw(ui, area));

    let mut observed = Scene::new(
        "notice-cut-observed",
        Theme::junie(),
        ColorLevel::TrueColor,
        28,
        4,
    );
    observed.draw(|ui, area| {
        notice.visit_cuts(ui, area.width, area.height, |_| {});
        notice.draw(ui, area);
    });

    for y in 0..baseline.area().height {
        for x in 0..baseline.area().width {
            assert_eq!(
                baseline.buffer().cell((x, y)),
                observed.buffer().cell((x, y)),
                "metadata observation changed cell ({x},{y})"
            );
        }
    }
    assert_eq!(baseline.cursor_position(), observed.cursor_position());
    assert_eq!(
        baseline
            .registry()
            .expect("baseline registry")
            .regions()
            .len(),
        0
    );
    assert_eq!(
        observed
            .registry()
            .expect("observed registry")
            .regions()
            .len(),
        0
    );
}
