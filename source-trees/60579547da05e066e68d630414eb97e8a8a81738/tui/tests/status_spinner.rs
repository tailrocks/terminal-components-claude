//! Real `StatusBar` cells, measurement, priority and hit geometry with composed spinners.
use junie_tui::{
    App, Constraints, Cx, Id, ItemKey, Part, PartRef, Rect, Response, Spinner, StatusAction,
    StatusBar, StatusItem, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("composed.status");
const KEY: ItemKey = ItemKey::num(1);
struct Page {
    items: Vec<StatusItem<'static>>,
    frame: usize,
    expected_width: Option<u16>,
    chosen: Option<ItemKey>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = StatusBar::new(ID).left(&self.items).update(cx);
        if let Some(StatusAction::Chose(key)) = response.take_action() {
            self.chosen = Some(key);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let bar = StatusBar::new(ID).left(&self.items);
        if let Some(expected) = self.expected_width {
            assert_eq!(
                bar.measure(ui, Constraints::loose(u16::MAX, u16::MAX))
                    .preferred
                    .0,
                expected
            );
        }
        bar.draw(ui, Rect::new(0, 0, ui.full().width, 1));
        Spinner::new(Id::root("standalone.spinner"))
            .frame(self.frame)
            .draw(ui, Rect::new(0, 1, ui.full().width, 1));
    }
}
fn page(
    items: Vec<StatusItem<'static>>,
    frame: usize,
    frames: &'static [&'static str],
    width: u16,
    expected_width: Option<u16>,
) -> Harness<Page> {
    let mut theme = Theme::junie();
    theme.design.motion.spinner_frames = frames;
    Harness::new(
        Page {
            items,
            frame,
            expected_width,
            chosen: None,
        },
        theme,
        width,
        2,
    )
}
#[test]
fn canonical_custom_frames_measure_and_paint_one_gap_with_clickable_geometry() {
    for (frame, glyph, glyph_width) in [(0, "界", 2), (1, "*", 1), (2, "界", 2)] {
        let mut h = page(
            vec![StatusItem::new("scan").spinner(frame).key(KEY)],
            frame,
            &["界", "*"],
            30,
            Some(glyph_width + 7),
        );
        assert_eq!(h.cell(1, 0).symbol(), glyph);
        assert_eq!(h.cell(0, 1).symbol(), glyph);
        assert_eq!(h.cell(1 + glyph_width, 0).symbol(), " ");
        assert_eq!(h.cell(2 + glyph_width, 0).symbol(), "s");
        assert_eq!(
            h.area_of_part(ID, PartRef::item(Part::LABEL, KEY))
                .map(|area| area.width),
            Some(glyph_width + 5)
        );
        let _ = h.click_part(ID, PartRef::item(Part::LABEL, KEY));
        assert_eq!(h.app().chosen, Some(KEY));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn empty_sequences_and_empty_glyphs_add_no_phantom_gap() {
    for frames in [&[][..], &[""][..]] {
        let h = page(
            vec![StatusItem::new("scan").spinner(usize::MAX).key(KEY)],
            usize::MAX,
            frames,
            30,
            Some(6),
        );
        assert_eq!(h.cell(1, 0).symbol(), "s");
        assert_eq!(
            h.area_of_part(ID, PartRef::item(Part::LABEL, KEY))
                .map(|area| area.width),
            Some(4)
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    let h = page(
        vec![StatusItem::new("").spinner(0).key(KEY)],
        0,
        &["界"],
        30,
        Some(4),
    );
    assert_eq!(
        h.area_of_part(ID, PartRef::item(Part::LABEL, KEY))
            .map(|area| area.width),
        Some(2)
    );
}
#[test]
fn actual_spinner_width_participates_in_priority_drop_and_tiny_clipping() {
    let h = page(
        vec![
            StatusItem::new("host").priority(9),
            StatusItem::new("scan").spinner(0).priority(1).key(KEY),
        ],
        0,
        &["界"],
        12,
        None,
    );
    assert!(h.text().contains("host"));
    assert!(!h.text().contains("scan"));
    assert!(
        h.area_of_part(ID, PartRef::item(Part::LABEL, KEY))
            .is_none()
    );
    for width in 0..=8 {
        let h = page(
            vec![StatusItem::new("scan").spinner(0).key(KEY)],
            0,
            &["界"],
            width,
            None,
        );
        assert!(
            h.diagnostics().is_empty(),
            "width={width}: {:?}",
            h.diagnostics()
        );
        if let Some(area) = h.area_of_part(ID, PartRef::item(Part::LABEL, KEY)) {
            assert!(area.right() <= width);
        }
    }
}
