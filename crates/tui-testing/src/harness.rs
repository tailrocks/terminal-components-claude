//! The deterministic application harness (`COMPONENT_ARCHITECTURE.md` §16.4).
//!
//! `handle` runs `update` and then draws synchronously, so every assertion
//! shape of the existing suites survives and a test never observes the
//! one-frame pointer latency of §20.1.

use ratatui_core::backend::TestBackend;
use ratatui_core::buffer::{Buffer, Cell};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::terminal::Terminal;
use tui_next::{
    App, Axis, ColorLevel, Diagnostic, FocusRing, Id, Input, Invalidate, Key, KeyCode,
    KeyModifiers, LayerId, Mouse, MouseKind, Part, PartRef, Resolved, Response, Runtime,
    StateFlags, Theme,
};

use crate::digest::Scene;

/// A runtime plus a test terminal.
pub struct Harness<A: App> {
    rt: Runtime<A>,
    term: Terminal<TestBackend>,
    auto_draw: bool,
    theme_name: &'static str,
    color: ColorLevel,
}

impl<A: App> core::fmt::Debug for Harness<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Harness")
            .field("focus", &self.rt.focus())
            .field("auto_draw", &self.auto_draw)
            .finish_non_exhaustive()
    }
}

fn theme_label(theme: &Theme) -> &'static str {
    if *theme == Theme::junie() {
        "junie"
    } else if *theme == Theme::paper() {
        "paper"
    } else {
        "custom"
    }
}

impl<A: App> Harness<A> {
    /// Build and draw the first frame (twice: the first draw settles the
    /// initial focus, the second paints it).
    pub fn new(app: A, theme: Theme, w: u16, h: u16) -> Self {
        let theme_name = theme_label(&theme);
        let mut h = Harness {
            rt: Runtime::new(app, theme),
            term: Terminal::new(TestBackend::new(w, h)).expect("test terminal"),
            auto_draw: true,
            theme_name,
            color: ColorLevel::TrueColor,
        };
        h.draw();
        h.draw();
        h
    }

    /// Downgrade the theme to `level` and redraw.
    #[must_use]
    pub fn with_color(mut self, level: ColorLevel) -> Self {
        let t = self.rt.theme().downgrade(level);
        self.rt.set_theme(t);
        self.color = level;
        self.draw();
        self
    }

    /// `false`: `handle` does not draw; call `draw()` explicitly.
    #[must_use]
    pub const fn with_auto_draw(mut self, yes: bool) -> Self {
        self.auto_draw = yes;
        self
    }

    /// Handle one input, then draw (when auto-draw is on).
    pub fn handle(&mut self, input: Input) -> Response<()> {
        let r = self.rt.handle(input);
        if self.auto_draw {
            self.draw();
        }
        r
    }

    /// A key press.
    pub fn key(&mut self, code: KeyCode) -> Response<()> {
        self.key_mod(code, KeyModifiers::NONE)
    }

    /// A key press with modifiers.
    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) -> Response<()> {
        self.handle(Input::Key(Key { code, mods }))
    }

    /// `Ctrl+c`.
    pub fn ctrl(&mut self, c: char) -> Response<()> {
        self.key_mod(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// `Alt+c`.
    pub fn alt(&mut self, c: char) -> Response<()> {
        self.key_mod(KeyCode::Char(c), KeyModifiers::ALT)
    }

    /// Type every character of `s`.
    pub fn type_str(&mut self, s: &str) -> Response<()> {
        let mut r = Response::ignored();
        for c in s.chars() {
            r |= self.key(KeyCode::Char(c));
        }
        r
    }

    /// Bracketed paste.
    pub fn paste(&mut self, s: &str) -> Response<()> {
        self.handle(Input::Paste(s.to_owned()))
    }

    /// A pointer event at `(x, y)`.
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Response<()> {
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }

    /// Press then release at `(x, y)`.
    pub fn click(&mut self, x: u16, y: u16) -> Response<()> {
        let mut r = self.mouse(MouseKind::Down, x, y);
        r |= self.mouse(MouseKind::Up, x, y);
        r
    }

    /// Click the centre of `area_of(id)`; `Response::ignored()` plus an
    /// `UnaddressableId` diagnostic when the id has no area.
    pub fn click_id(&mut self, id: Id) -> Response<()> {
        if let Some(a) = self.area_of(id) {
            self.click_rect(a)
        } else {
            self.rt
                .record_diagnostic(Diagnostic::UnaddressableId { id });
            Response::ignored()
        }
    }

    /// Click the centre of a component's sub-region.
    pub fn click_part(&mut self, id: Id, p: PartRef) -> Response<()> {
        if let Some(a) = self.area_of_part(id, p) {
            self.click_rect(a)
        } else {
            self.rt
                .record_diagnostic(Diagnostic::UnaddressableId { id });
            Response::ignored()
        }
    }

    fn click_rect(&mut self, a: Rect) -> Response<()> {
        let (x, y) = centre(a);
        self.click(x, y)
    }

    /// Two clicks inside the double-click window.
    pub fn double_click(&mut self, x: u16, y: u16) -> Response<()> {
        let mut r = self.click(x, y);
        r |= self.click(x, y);
        r
    }

    /// Secondary button down at `(x, y)`.
    pub fn secondary(&mut self, x: u16, y: u16) -> Response<()> {
        let mut r = self.mouse(MouseKind::Secondary, x, y);
        r |= self.mouse(MouseKind::SecondaryUp, x, y);
        r
    }

    /// Press at `from`, drag to `to`, release at `to`.
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) -> Response<()> {
        let mut r = self.mouse(MouseKind::Down, from.0, from.1);
        r |= self.mouse(MouseKind::Drag, to.0, to.1);
        r |= self.mouse(MouseKind::Up, to.0, to.1);
        r
    }

    /// Wheel motion at `(x, y)`.
    pub fn wheel(&mut self, axis: Axis, delta: i16, x: u16, y: u16) -> Response<()> {
        self.mouse(MouseKind::Wheel(axis, delta), x, y)
    }

    /// Resize the terminal.
    pub fn resize(&mut self, w: u16, h: u16) -> Response<()> {
        self.term.backend_mut().resize(w, h);
        let _ = self.term.clear();
        self.handle(Input::Resize(w, h))
    }

    /// Draw one frame.
    pub fn draw(&mut self) {
        let rt = &mut self.rt;
        self.term.draw(|f| rt.draw(f)).expect("draw");
    }

    /// Advance the virtual clock by `n` ticks.
    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            let _ = self.tick();
        }
    }

    /// One tick.
    pub fn tick(&mut self) -> Response<()> {
        self.handle(Input::Tick)
    }

    /// Last frame's area of `id`.
    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.rt.area_of(id)
    }

    /// Last frame's area of a sub-region.
    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect> {
        self.rt.area_of_part(id, p)
    }

    /// The focus ring.
    pub const fn ring(&self) -> &FocusRing {
        self.rt.ring()
    }

    /// The focused control.
    pub const fn focus(&self) -> Option<Id> {
        self.rt.focus()
    }

    /// Whether focus is painted.
    pub const fn focus_visible(&self) -> bool {
        self.rt.focus_visible()
    }

    /// The hovered control.
    pub fn hover(&self) -> Option<Id> {
        self.rt.hover()
    }

    /// Runtime-resolved flags for `id`.
    pub fn state_of(&self, id: Id) -> StateFlags {
        self.rt.state_of(id)
    }

    /// The top layer.
    pub fn top_layer(&self) -> LayerId {
        self.rt.top_layer()
    }

    /// Whether layer `id` is open.
    pub fn is_open(&self, id: Id) -> bool {
        self.rt.is_open(id)
    }

    /// Press Tab until `id` is focused; `false` if it never is.
    pub fn tab_to(&mut self, id: Id) -> bool {
        let budget = self.ring().entries().len().saturating_add(1);
        for _ in 0..budget {
            if self.focus() == Some(id) {
                break;
            }
            let _ = self.key(KeyCode::Tab);
        }
        if self.auto_draw {
            self.draw();
        }
        self.focus() == Some(id)
    }

    /// Drop focus (the harness `blur`).
    pub fn blur(&mut self) {
        self.rt.set_focus(None);
        if self.auto_draw {
            self.draw();
        }
    }

    /// Diagnostics since the last `handle`.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.rt.diagnostics()
    }

    /// The frame buffer.
    pub fn buffer(&self) -> &Buffer {
        self.term.backend().buffer()
    }

    /// Every row joined with newlines.
    pub fn text(&self) -> String {
        let area = *self.buffer().area();
        let mut out = String::new();
        for y in 0..area.height {
            if y > 0 {
                out.push('\n');
            }
            out.push_str(&self.row(y));
        }
        out
    }

    /// One row as text (wide cells contribute their symbol once).
    pub fn row(&self, y: u16) -> String {
        row_text(self.buffer(), y).0
    }

    /// The first `(x, y)` where `needle` starts, grapheme-accurate.
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let area = *self.buffer().area();
        (0..area.height).find_map(|y| {
            let (s, cols) = row_text(self.buffer(), y);
            s.find(needle)
                .map(|byte| (cols.get(byte).copied().unwrap_or(0), y))
        })
    }

    /// The first row containing `needle`.
    pub fn find_row(&self, needle: &str) -> Option<u16> {
        self.find(needle).map(|(_, y)| y)
    }

    /// Occurrences of `needle` across the frame.
    pub fn count(&self, needle: &str) -> usize {
        let area = *self.buffer().area();
        (0..area.height)
            .map(|y| row_text(self.buffer(), y).0.matches(needle).count())
            .sum()
    }

    /// The cell at `(x, y)`.
    pub fn cell(&self, x: u16, y: u16) -> &Cell {
        self.buffer().cell((x, y)).expect("cell inside the frame")
    }

    /// The cursor kept by the last draw.
    pub fn cursor(&self) -> Option<Position> {
        self.rt.cursor()
    }

    /// A digest scene of the current frame.
    pub fn snapshot(&self) -> Scene {
        Scene::from_buffer(
            "harness",
            self.theme_name,
            self.color,
            self.buffer().clone(),
        )
    }

    /// The application.
    pub const fn app(&self) -> &A {
        self.rt.app()
    }

    /// The application, mutably.
    pub const fn app_mut(&mut self) -> &mut A {
        self.rt.app_mut()
    }

    /// The runtime.
    pub const fn runtime(&self) -> &Runtime<A> {
        &self.rt
    }

    /// The runtime, mutably.
    pub const fn runtime_mut(&mut self) -> &mut Runtime<A> {
        &mut self.rt
    }

    /// Resolve `p` for `id` as the runtime would (default family).
    pub fn resolved(&self, id: Id, p: Part) -> Resolved {
        self.rt.resolved(id, p)
    }

    /// The invalidation of the last `handle`.
    pub const fn last_invalidate(&self) -> Invalidate {
        self.rt.last_invalidate()
    }

    /// Tags recorded through `Cx::record`.
    pub fn records(&self) -> &[&'static str] {
        self.rt.records()
    }
}

/// The centre cell of a rect.
pub fn centre(a: Rect) -> (u16, u16) {
    (
        a.x.saturating_add(a.width / 2)
            .min(a.right().saturating_sub(1)),
        a.y.saturating_add(a.height / 2)
            .min(a.bottom().saturating_sub(1)),
    )
}

/// One row as text plus a byte-offset → column map.
pub fn row_text(buf: &Buffer, y: u16) -> (String, Vec<u16>) {
    let area = *buf.area();
    let mut s = String::new();
    let mut cols = Vec::new();
    let mut x = area.x;
    while x < area.right() {
        let Some(c) = buf.cell((x, y)) else { break };
        let sym = c.symbol();
        for _ in 0..sym.len() {
            cols.push(x);
        }
        s.push_str(sym);
        let w = tui_next::text::width(sym).max(1);
        x = x.saturating_add(w);
    }
    cols.push(x);
    (s, cols)
}
