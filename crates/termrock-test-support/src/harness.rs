use ratatui_core::backend::TestBackend;
use ratatui_core::buffer::{Buffer, Cell};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::terminal::Terminal;
use termrock::{
    App, Axis, ColorLevel, Diagnostic, Family, FocusRing, Id, Input, Invalidate, Key, KeyCode,
    KeyModifiers, LayerId, Mouse, MouseKind, Part, PartRef, PreparedFrameError, Resolved, Response,
    Runtime, StateFlags, Theme, Variant,
};

use crate::digest::Scene;

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
    pub fn new(app: A, theme: Theme, w: u16, h: u16) -> Self {
        Self::new_with_feedback_clock(app, theme, w, h, termrock::FeedbackClock::Elapsed)
    }

    pub fn new_with_feedback_clock(
        app: A,
        theme: Theme,
        w: u16,
        h: u16,
        clock: termrock::FeedbackClock,
    ) -> Self {
        let theme_name = theme_label(&theme);
        let mut h = Harness {
            rt: Runtime::new_with_feedback_clock(app, theme, clock),
            term: Terminal::new(TestBackend::new(w, h)).expect("test terminal"),
            auto_draw: true,
            theme_name,
            color: ColorLevel::TrueColor,
        };
        let _ = h.rt.initialize();
        h.draw();
        h
    }

    pub fn activation_feedback(&self) -> Option<termrock::ActivationFeedback> {
        self.rt.activation_feedback()
    }

    #[must_use]
    pub fn with_color(mut self, level: ColorLevel) -> Self {
        let t = self.rt.theme().downgrade(level);
        self.rt.set_theme(t);
        self.color = level;
        self.draw();
        self
    }

    #[must_use]
    pub const fn with_auto_draw(mut self, yes: bool) -> Self {
        self.auto_draw = yes;
        self
    }

    pub fn handle(&mut self, input: Input) -> Response<()> {
        if self.rt.needs_present() || self.rt.needs_settle() {
            self.draw();
        }
        let r = self
            .rt
            .handle(input)
            .expect("harness publishes before input");
        if self.auto_draw {
            self.draw();
        }
        r
    }

    pub fn key(&mut self, code: KeyCode) -> Response<()> {
        self.key_mod(code, KeyModifiers::NONE)
    }

    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) -> Response<()> {
        self.handle(Input::Key(Key { code, mods }))
    }

    pub fn ctrl(&mut self, c: char) -> Response<()> {
        self.key_mod(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    pub fn alt(&mut self, c: char) -> Response<()> {
        self.key_mod(KeyCode::Char(c), KeyModifiers::ALT)
    }

    pub fn type_str(&mut self, s: &str) -> Response<()> {
        let mut r = Response::ignored();
        for c in s.chars() {
            r |= self.key(KeyCode::Char(c));
        }
        r
    }

    pub fn paste(&mut self, s: &str) -> Response<()> {
        self.handle(Input::Paste(s.to_owned()))
    }

    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Response<()> {
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }

    pub fn click(&mut self, x: u16, y: u16) -> Response<()> {
        let mut r = self.mouse(MouseKind::Down, x, y);
        r |= self.mouse(MouseKind::Up, x, y);
        r
    }

    pub fn click_id(&mut self, id: Id) -> Response<()> {
        if let Some(a) = self.area_of(id) {
            self.click_rect(a)
        } else {
            self.rt
                .record_diagnostic(Diagnostic::UnaddressableId { id });
            Response::ignored()
        }
    }

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

    pub fn double_click(&mut self, x: u16, y: u16) -> Response<()> {
        let mut r = self.click(x, y);
        r |= self.click(x, y);
        r
    }

    pub fn secondary(&mut self, x: u16, y: u16) -> Response<()> {
        let mut r = self.mouse(MouseKind::Secondary, x, y);
        r |= self.mouse(MouseKind::SecondaryUp, x, y);
        r
    }

    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) -> Response<()> {
        let mut r = self.mouse(MouseKind::Down, from.0, from.1);
        r |= self.mouse(MouseKind::Drag, to.0, to.1);
        r |= self.mouse(MouseKind::Up, to.0, to.1);
        r
    }

    pub fn wheel(&mut self, axis: Axis, delta: i16, x: u16, y: u16) -> Response<()> {
        self.mouse(MouseKind::Wheel(axis, delta), x, y)
    }

    pub fn resize(&mut self, w: u16, h: u16) -> Response<()> {
        self.term.backend_mut().resize(w, h);
        let _ = self.term.clear();
        self.handle(Input::Resize(w, h))
    }

    pub fn draw(&mut self) {
        for _ in 0..16 {
            let area = self.term.get_frame().area();
            let prepared = self.rt.prepare(area).expect("runtime prepares frame");
            let mut area_changed = false;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.term
                    .draw(|frame| {
                        if let Err(error) = self.rt.draw_prepared(prepared, frame) {
                            area_changed = matches!(error, PreparedFrameError::AreaChanged { .. });
                            panic!("prepared frame projection failed: {error}");
                        }
                    })
                    .expect("TestBackend draw succeeds");
            }));
            if result.is_err() && area_changed {
                continue;
            }
            if let Err(payload) = result {
                std::panic::resume_unwind(payload);
            }
            self.rt
                .acknowledge_presented(prepared)
                .expect("test backend acknowledges frame");
            return;
        }
        panic!("harness output did not settle: {:?}", self.rt.diagnostics());
    }

    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            let _ = self.tick();
        }
    }

    pub fn tick(&mut self) -> Response<()> {
        self.handle(Input::Tick)
    }

    pub fn now(&self) -> termrock::Moment {
        self.rt.now()
    }

    pub fn next_deadline(&self) -> Option<termrock::Moment> {
        self.rt.next_deadline()
    }

    pub fn advance(&mut self, elapsed: core::time::Duration) -> Response<()> {
        self.advance_to(self.rt.now().saturating_add(elapsed))
            .expect("saturating forward clock movement")
    }

    pub fn advance_to(
        &mut self,
        now: termrock::Moment,
    ) -> Result<Response<()>, termrock::ClockError> {
        self.rt.advance_to(now)?;
        let mut response = Response::ignored();
        for _ in 0..16 {
            if !self.rt.needs_settle() {
                break;
            }
            response |= self.rt.settle();
        }
        if self.auto_draw {
            self.draw();
        }
        Ok(response)
    }

    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.rt.area_of(id)
    }

    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect> {
        self.rt.area_of_part(id, p)
    }

    pub const fn ring(&self) -> &FocusRing {
        self.rt.ring()
    }

    pub const fn focus(&self) -> Option<Id> {
        self.rt.focus()
    }

    pub const fn focus_visible(&self) -> bool {
        self.rt.focus_visible()
    }

    pub fn hover(&self) -> Option<Id> {
        self.rt.hover()
    }

    pub fn state_of(&self, id: Id) -> StateFlags {
        self.rt.state_of(id)
    }

    pub fn top_layer(&self) -> LayerId {
        self.rt.top_layer()
    }

    pub fn layer_area(&self, id: Id) -> Option<Rect> {
        self.rt.layer_area(id)
    }

    pub fn is_open(&self, id: Id) -> bool {
        self.rt.is_open(id)
    }

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

    pub fn blur(&mut self) {
        self.rt.set_focus(None);
        if self.auto_draw {
            self.draw();
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.rt.diagnostics()
    }

    pub fn buffer(&self) -> &Buffer {
        self.term.backend().buffer()
    }

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

    pub fn row(&self, y: u16) -> String {
        row_text(self.buffer(), y).0
    }

    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let area = *self.buffer().area();
        (0..area.height).find_map(|y| {
            let (s, cols) = row_text(self.buffer(), y);
            s.find(needle)
                .map(|byte| (cols.get(byte).copied().unwrap_or(0), y))
        })
    }

    pub fn find_row(&self, needle: &str) -> Option<u16> {
        self.find(needle).map(|(_, y)| y)
    }

    pub fn count(&self, needle: &str) -> usize {
        let area = *self.buffer().area();
        (0..area.height)
            .map(|y| row_text(self.buffer(), y).0.matches(needle).count())
            .sum()
    }

    pub fn cell(&self, x: u16, y: u16) -> &Cell {
        self.buffer().cell((x, y)).expect("cell inside the frame")
    }

    pub fn cursor(&self) -> Option<Position> {
        self.rt.cursor()
    }

    pub fn cursor_bar(&self) -> bool {
        self.rt.cursor_bar()
    }

    pub fn snapshot(&self) -> Scene {
        Scene::from_buffer(
            "harness",
            self.theme_name,
            self.color,
            self.buffer().clone(),
        )
    }

    pub const fn app(&self) -> &A {
        self.rt.app()
    }

    pub fn app_mut(&mut self) -> &mut A {
        self.rt.app_mut()
    }

    pub const fn runtime(&self) -> &Runtime<A> {
        &self.rt
    }

    pub const fn runtime_mut(&mut self) -> &mut Runtime<A> {
        &mut self.rt
    }

    pub fn resolved(&self, id: Id, p: Part) -> Resolved {
        self.rt.resolved(id, p)
    }

    pub fn resolved_in(&self, f: Family, v: Variant, id: Id, p: Part) -> Resolved {
        self.rt.resolved_in(f, v, id, p)
    }

    pub const fn last_invalidate(&self) -> Invalidate {
        self.rt.last_invalidate()
    }

    pub fn records(&self) -> &[&'static str] {
        self.rt.records()
    }
}

pub fn centre(a: Rect) -> (u16, u16) {
    (
        a.x.saturating_add(a.width / 2)
            .min(a.right().saturating_sub(1)),
        a.y.saturating_add(a.height / 2)
            .min(a.bottom().saturating_sub(1)),
    )
}

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
        let w = termrock::width(sym).max(1);
        x = x.saturating_add(w);
    }
    cols.push(x);
    (s, cols)
}

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;
    use termrock::{Cx, Focusability, FrameRead, ItemKey, RowUi, Ui};

    use super::*;

    const LIST: Id = Id::root("harness.list");
    const BUTTON: Id = Id::root("harness.button");

    struct TwoFamilies;

    impl App for TwoFamilies {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let row = Rect::new(0, 0, 20, 1);
            ui.register_control(LIST, row, Focusability::Focusable);
            {
                let mut r = RowUi::new(
                    ui,
                    LIST,
                    Family::LIST,
                    Variant::DEFAULT,
                    ui.state(LIST),
                    ItemKey::index(0),
                    row,
                );
                r.gutter();
                r.label("row");
            }
            let bar = Rect::new(0, 1, 20, 1);
            ui.register_control(BUTTON, bar, Focusability::Focusable);
            let r = ui.style(
                Family::BUTTON,
                Variant::PRIMARY,
                Part::CONTAINER,
                ui.state(BUTTON),
            );
            ui.note_styled(BUTTON, Family::BUTTON, Variant::PRIMARY, Part::CONTAINER, r);
            ui.fill(bar, r.style);
        }
    }

    #[test]
    fn resolved_reports_the_family_the_component_actually_queried() {
        let theme = Theme::junie();
        let mut h = Harness::new(TwoFamilies, theme.clone(), 20, 3);
        h.draw();

        let list_label = h.resolved(LIST, Part::LABEL);
        assert_eq!(
            list_label,
            theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                h.state_of(LIST),
                termrock::Surface::Canvas
            )
        );

        let btn = h.resolved(BUTTON, Part::CONTAINER);
        assert_eq!(
            btn,
            theme.resolve(
                Family::BUTTON,
                Variant::PRIMARY,
                Part::CONTAINER,
                h.state_of(BUTTON),
                termrock::Surface::Canvas
            )
        );
        assert_ne!(
            btn,
            theme.resolve(
                Family::BUTTON,
                Variant::DEFAULT,
                Part::CONTAINER,
                h.state_of(BUTTON),
                termrock::Surface::Canvas
            ),
            "the recorded variant must be the one the component used"
        );

        assert_ne!(list_label.style, btn.style);

        assert_eq!(
            h.resolved_in(Family::TABS, Variant::DEFAULT, LIST, Part::TAB),
            theme.resolve(
                Family::TABS,
                Variant::DEFAULT,
                Part::TAB,
                h.state_of(LIST),
                termrock::Surface::Canvas
            )
        );
    }
}
