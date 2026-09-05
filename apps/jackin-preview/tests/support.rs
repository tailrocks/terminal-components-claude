//! Shared public-runtime harness for Jackin integration scenarios.

use jackin_app::{App, Motion, Scenario};
use junie_tui::{
    Buffer, Id, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Position, Response, Theme,
};
use junie_tui_testing::Harness;

/// Public-runtime test session. The application remains owned by
/// `tui-next::Runtime`; tests access it through the runtime's public app
/// projection instead of calling a legacy `render`/`handle` pair.
pub struct H {
    /// Deterministic runtime and test buffer.
    pub harness: Harness<App>,
}

impl H {
    /// Build a pinned scenario and draw its first frame.
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

    /// Immutable app projection.
    pub fn app(&self) -> &App {
        self.harness.app()
    }

    /// Mutable app projection for fixture-only state setup.
    pub fn app_mut(&mut self) -> &mut App {
        self.harness.app_mut()
    }

    /// Draw the current public app.
    pub fn draw(&mut self) {
        self.harness.draw();
    }

    /// Send one key.
    pub fn key(&mut self, code: KeyCode) {
        let _ = self.harness.key(code);
    }

    /// Send one key with modifiers.
    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        let _ = self.harness.key_mod(code, mods);
    }

    /// Send a control key.
    pub fn ctrl(&mut self, c: char) {
        let _ = self.harness.ctrl(c);
    }

    /// Type text through the public input boundary.
    pub fn type_str(&mut self, s: &str) {
        let _ = self.harness.type_str(s);
    }

    /// Advance virtual time.
    pub fn ticks(&mut self, n: usize) {
        self.harness.ticks(n);
    }

    /// Send one pointer event.
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        let _ = self.harness.mouse(kind, x, y);
    }

    /// Press and release at a coordinate.
    pub fn click(&mut self, x: u16, y: u16) {
        let _ = self.harness.click(x, y);
    }

    /// Resize the public test terminal.
    pub fn resize(&mut self, w: u16, h: u16) {
        let _ = self.harness.resize(w, h);
    }

    /// Tab until an id owns focus.
    pub fn tab_to(&mut self, id: Id) {
        assert!(self.harness.tab_to(id), "focus never reached {id:?}");
    }

    /// Current focus id.
    pub fn focus(&self) -> Option<Id> {
        self.harness.focus()
    }

    /// Current frame text.
    pub fn text(&self) -> String {
        self.harness.text()
    }

    /// Find a grapheme-safe text coordinate.
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        self.harness.find(needle)
    }

    /// Public buffer projection for digest/perf assertions.
    pub fn buffer(&self) -> &Buffer {
        self.harness.buffer()
    }

    /// Forward one fully-formed public input event.
    pub fn handle(&mut self, input: Input) -> Response<()> {
        self.harness.handle(input)
    }

    /// Send a key event directly to the runtime.
    pub fn send_key(&mut self, code: KeyCode, mods: KeyModifiers) -> Response<()> {
        self.handle(Input::Key(Key { code, mods }))
    }

    /// Send a mouse event directly to the runtime.
    pub fn send_mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Response<()> {
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }
}
