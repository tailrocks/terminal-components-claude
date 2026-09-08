//! Menu shortcut casing is presentation only, including dropdown measurement.
use junie_tui::{
    ActionKey, App, Chord, ChordCase, Cx, Id, KeyCode, KeyModifiers, Menu, MenuAction, MenuBar,
    MenuItem, MenuState, Rect, Response, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("case.menu");
const QUIT: ActionKey = ActionKey::application("case.quit");
const ITEMS: &[MenuItem<'static>] =
    &[MenuItem::new(QUIT, "Quit").chord(Chord::with(KeyCode::Char('q'), KeyModifiers::CONTROL))];
const MENUS: &[Menu<'static>] = &[Menu::new("File", ITEMS)];
struct Page {
    state: MenuState,
    case: Option<ChordCase>,
    chosen: Option<ActionKey>,
}
fn bar(case: Option<ChordCase>) -> MenuBar<'static> {
    let bar = MenuBar::new(ID, MENUS);
    case.map_or(bar, |case| MenuBar::new(ID, MENUS).chord_case(case))
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = bar(self.case).update(cx, &mut self.state);
        if cx.update_cause() == UpdateCause::Bootstrap {
            let _ = bar(self.case).open_menu(cx, &mut self.state, 0);
        }
        if let Some(MenuAction::Chosen(action)) = response.take_action() {
            self.chosen = Some(action);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let _ = bar(self.case).draw(ui, Rect::new(0, 0, 40, 8), &self.state);
    }
}
fn page(case: Option<ChordCase>, width: u16) -> Harness<Page> {
    Harness::new(
        Page {
            state: MenuState::default(),
            case,
            chosen: None,
        },
        Theme::junie(),
        width,
        8,
    )
}
#[test]
fn uppercase_shortcut_still_accepts_physical_lowercase_control_q() {
    let mut h = page(Some(ChordCase::UppercaseAscii), 40);
    assert!(h.text().contains("Ctrl+Q"), "{}", h.text());
    assert!(!h.text().contains("Ctrl+q"));
    let _ = h.ctrl('q');
    assert_eq!(h.app().chosen, Some(QUIT));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn explicit_preserve_matches_default_buffers_and_geometry() {
    for width in [8, 12, 20, 40] {
        let default = page(None, width);
        let preserve = page(Some(ChordCase::Preserve), width);
        assert_eq!(default.buffer(), preserve.buffer());
        assert_eq!(default.focus(), preserve.focus());
        assert_eq!(default.diagnostics(), preserve.diagnostics());
        let upper = page(Some(ChordCase::UppercaseAscii), width);
        assert_eq!(default.text().replace("Ctrl+q", "Ctrl+Q"), upper.text());
    }
}
