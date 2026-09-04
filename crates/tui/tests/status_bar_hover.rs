//! Regression coverage for live and forced `StatusBar` hover state.

use tui_next::{
    App, Cx, Family, FrameRead, Id, ItemKey, KeyCode, MouseKind, Part, PartRef, Response, Role,
    StateFlags, StatusBar, StatusItem, StylePatch, Surface, Theme, Ui,
};
use tui_next_testing::Harness;

const BAR: Id = Id::root("status.hover");
const FIRST: ItemKey = ItemKey::num(1);
const SECOND: ItemKey = ItemKey::num(2);
const ITEMS: [StatusItem<'static>; 3] = [
    StatusItem::new("alpha").key(FIRST),
    StatusItem::new("plain"),
    StatusItem::new("beta").key(SECOND),
];

#[derive(Default)]
struct StatusApp {
    hover_samples: [Option<PartRef>; 4],
    updates: usize,
    forced: Option<StateFlags>,
}

impl StatusApp {
    fn bar() -> StatusBar<'static> {
        StatusBar::new(BAR).left(&ITEMS)
    }
}

impl App for StatusApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(sample) = self.hover_samples.get_mut(self.updates) {
            *sample = FrameRead::hovered_part(cx, BAR);
        }
        self.updates = self.updates.saturating_add(1);
        let _ = Self::bar().update(cx);
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let bar = match self.forced {
            Some(flags) => Self::bar().state_override(flags),
            None => Self::bar(),
        };
        bar.draw(ui, area);
    }
}

fn center(area: ratatui_core::layout::Rect) -> (u16, u16) {
    (
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    )
}

fn hover_theme() -> Theme {
    Theme::junie().override_family(Family::STATUSBAR, |r| {
        r.part(Part::LABEL).when(
            StateFlags::HOVERED,
            StylePatch::new().set_bg(Role::Surface(Surface::Elevated)),
        );
    })
}

fn text_start<A: App>(h: &Harness<A>, text: &str) -> u16 {
    let row = h.row(0);
    assert!(row.contains(text));
    row.find(text)
        .and_then(|x| u16::try_from(x).ok())
        .unwrap_or_default()
}

#[test]
fn live_hover_moves_between_keyed_labels_and_keyboard_suppression() {
    let theme = hover_theme();
    let base_bg = theme.bg(Surface::Surface);
    let hover_bg = theme.bg(Surface::Elevated);
    let mut h = Harness::new(StatusApp::default(), theme, 32, 2);
    let first_part = PartRef::item(Part::LABEL, FIRST);
    let second_part = PartRef::item(Part::LABEL, SECOND);
    let first_area = h.area_of_part(BAR, first_part).unwrap_or_default();
    let second_area = h.area_of_part(BAR, second_part).unwrap_or_default();
    let plain_x = text_start(&h, "plain");

    let _ = h.mouse(MouseKind::Move, center(first_area).0, center(first_area).1);
    assert_eq!(h.app().hover_samples[0], Some(first_part));
    assert_eq!(h.cell(first_area.x, first_area.y).bg, hover_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, base_bg);

    let _ = h.mouse(
        MouseKind::Move,
        center(second_area).0,
        center(second_area).1,
    );
    assert_eq!(h.app().hover_samples[1], Some(second_part));
    assert_eq!(h.cell(first_area.x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, hover_bg);

    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.app().hover_samples[2], None);
    assert_eq!(h.cell(first_area.x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, base_bg);

    let _ = h.mouse(MouseKind::Move, center(first_area).0, center(first_area).1);
    assert_eq!(h.app().hover_samples[3], Some(first_part));
    assert_eq!(h.cell(first_area.x, first_area.y).bg, hover_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, base_bg);
}

#[test]
fn forced_hover_reaches_keyed_and_unkeyed_labels_and_clears_stale_live_hover() {
    let theme = hover_theme();
    let base_bg = theme.bg(Surface::Surface);
    let hover_bg = theme.bg(Surface::Elevated);
    let mut h = Harness::new(StatusApp::default(), theme, 32, 2);
    let first_part = PartRef::item(Part::LABEL, FIRST);
    let second_part = PartRef::item(Part::LABEL, SECOND);
    let first_area = h.area_of_part(BAR, first_part).unwrap_or_default();
    let second_area = h.area_of_part(BAR, second_part).unwrap_or_default();
    let plain_x = text_start(&h, "plain");

    let _ = h.mouse(
        MouseKind::Move,
        center(second_area).0,
        center(second_area).1,
    );
    assert_eq!(h.cell(first_area.x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, hover_bg);

    h.app_mut().forced = Some(StateFlags::HOVERED);
    h.draw();
    assert_eq!(h.cell(first_area.x, first_area.y).bg, hover_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, hover_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, hover_bg);

    h.app_mut().forced = Some(StateFlags::empty());
    h.draw();
    assert_eq!(h.cell(first_area.x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(plain_x, first_area.y).bg, base_bg);
    assert_eq!(h.cell(second_area.x, second_area.y).bg, base_bg);
}
