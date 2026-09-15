//! Actual accepted application art composed with an actual library control.
use junie_tui::{App, Button, Cx, Id, KeyCode, Rect, Response, Theme, Ui};
use junie_tui_testing::Harness;

const CONTROL: Id = Id::root("architecture.bootstrap.rain-control");

#[derive(Debug)]
struct RainApp { frame: u64, activated: u32, theme: Theme }
impl RainApp {
    fn control() -> Button<'static> { Button::new(CONTROL, "Keep library ownership") }
}
impl App for RainApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Self::control().update(cx).on_action(|_| self.activated += 1)
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let control = Self::control().draw(ui, Rect::new(2, 2, 26, 1));
        let (buffer, area) = ui.raw();
        jackin_app::rain::paint_atmosphere(buffer, area, &[control], self.frame, true, false, &self.theme);
    }
}

#[test]
fn architecture_bootstrap_real_rain_keeps_library_control() {
    let mut first = Harness::new(RainApp { frame: 16, activated: 0, theme: Theme::junie() }, Theme::junie(), 80, 24);
    let mut second = Harness::new(RainApp { frame: 17, activated: 0, theme: Theme::junie() }, Theme::junie(), 80, 24);
    let area = first.area_of(CONTROL).expect("real library registration");
    assert_eq!(Some(area), second.area_of(CONTROL));
    assert_eq!(first.focus(), second.focus());
    let mut outside_changes = 0;
    for y in 0..24 {
        for x in 0..80 {
            if area.contains((x, y).into()) {
                assert_eq!(first.cell(x, y), second.cell(x, y));
            } else if first.cell(x, y) != second.cell(x, y) {
                outside_changes += 1;
            }
        }
    }
    assert!(outside_changes > 0, "actual rain cells must change");
    let _ = first.click_id(CONTROL);
    let _ = second.click_id(CONTROL);
    let _ = first.key(KeyCode::Enter);
    let _ = second.key(KeyCode::Enter);
    assert_eq!(first.app().activated, 2);
    assert_eq!(first.app().activated, second.app().activated);
    println!("ARCHRAIN|{outside_changes}|{:?}|{}", area, first.app().activated);
}
