use jackin_preview_app::{App, Motion, Scenario};
use termrock::{
    Buffer, Id, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Position, Response, Theme,
};
use termrock_test_support::Harness;

#[derive(Debug)]
pub struct H {
    pub harness: Harness<App>,
}

impl H {
    pub fn new(scenario: Scenario, motion: Motion, frame: u64, w: u16, h: u16) -> Self {
        Self {
            harness: Harness::new(
                App::for_scenario_at(scenario, motion, frame),
                Theme::junie(),
                w,
                h,
            ),
        }
    }

    /// A harness whose theme is narrowed to a colour level.
    pub fn new_with_color(
        scenario: Scenario,
        motion: Motion,
        frame: u64,
        w: u16,
        h: u16,
        level: termrock::ColorLevel,
    ) -> Self {
        Self {
            harness: Harness::new(
                App::for_scenario_at(scenario, motion, frame),
                Theme::junie().downgrade(level),
                w,
                h,
            ),
        }
    }

    pub fn app(&self) -> &App {
        self.harness.app()
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.harness.app_mut()
    }

    pub fn draw(&mut self) {
        self.harness.draw();
    }

    pub fn key(&mut self, code: KeyCode) {
        let _ = self.harness.key(code);
    }

    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        let _ = self.harness.key_mod(code, mods);
    }

    pub fn ctrl(&mut self, c: char) {
        let _ = self.harness.ctrl(c);
    }

    pub fn type_str(&mut self, s: &str) {
        let _ = self.harness.type_str(s);
    }

    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            let _ = self.harness.advance(std::time::Duration::from_millis(200));
        }
    }

    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        let _ = self.harness.mouse(kind, x, y);
    }

    pub fn click(&mut self, x: u16, y: u16) {
        let _ = self.harness.click(x, y);
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        let _ = self.harness.resize(w, h);
    }

    pub fn tab_to(&mut self, id: Id) {
        assert!(self.harness.tab_to(id), "focus never reached {id:?}");
    }

    pub fn focus(&self) -> Option<Id> {
        self.harness.focus()
    }

    pub fn text(&self) -> String {
        self.harness.text()
    }

    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        self.harness.find(needle)
    }

    pub fn buffer(&self) -> &Buffer {
        self.harness.buffer()
    }

    pub fn handle(&mut self, input: Input) -> Response<()> {
        self.harness.handle(input)
    }

    pub fn send_key(&mut self, code: KeyCode, mods: KeyModifiers) -> Response<()> {
        self.handle(Input::Key(Key { code, mods }))
    }

    pub fn send_mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Response<()> {
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }
}
