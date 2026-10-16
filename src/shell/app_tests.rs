use super::*;
use crate::core::event::Key;
use crate::ui::ctx::RenderCtx;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

/// A one-row app: a button per name on page `page`. Focus changes are
/// logged by `on_blur`.
struct Demo {
    shell: Shell,
    page: u64,
    buttons: Vec<&'static str>,
    primary: Option<&'static str>,
    blurs: Vec<FocusChange>,
}

impl Demo {
    fn new(buttons: &[&'static str]) -> Self {
        Self {
            shell: Shell::new(),
            page: 1,
            buttons: buttons.to_vec(),
            primary: None,
            blurs: vec![],
        }
    }
    fn focused(&self) -> Option<WidgetId> {
        self.shell.focus.current()
    }
}

impl App for Demo {
    type Status = ();
    fn shell(&self) -> &Shell {
        &self.shell
    }
    fn shell_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }
    fn theme(&self) -> Theme {
        Theme::junie()
    }
    fn too_small(&self) -> TooSmall<'_> {
        TooSmall::new("demo").min(20, 3)
    }
    fn now_ms(&self) -> u64 {
        0
    }
    fn tick(&self) -> u64 {
        0
    }
    fn tick_interval(&self) -> Duration {
        Duration::from_millis(100)
    }
    fn draw(&mut self, area: Rect, _buf: &mut Buffer, ctx: &mut RenderCtx) {
        for (i, b) in self.buttons.iter().enumerate() {
            let r = Rect::new(area.x + i as u16 * 4, area.y, 3, 1);
            ctx.control(WidgetId::of(b), r, false);
        }
    }
    fn on_input(&mut self, input: Input) -> Outcome {
        match input {
            Input::Key(k) if k.code == KeyCode::Tab => {
                self.shell.focus_next();
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }
    fn on_blur(&mut self, change: FocusChange) -> Outcome {
        self.blurs.push(change);
        Outcome::Changed
    }
    fn page_key(&self) -> PageKey {
        PageKey(self.page)
    }
    fn primary_focus(&self) -> Option<WidgetId> {
        self.primary.map(WidgetId::of)
    }
}

fn draw_at(app: &mut Demo, w: u16, h: u16) {
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| Application::render(app, f)).unwrap();
}

fn draw(app: &mut Demo) {
    draw_at(app, 40, 5);
}

#[test]
fn a_new_page_focuses_its_primary_control() {
    let mut app = Demo::new(&["a", "b", "c"]);
    app.primary = Some("b");
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("b")));
}

#[test]
fn without_a_primary_control_a_new_page_focuses_its_first() {
    let mut app = Demo::new(&["a", "b", "c"]);
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("a")));
}

#[test]
fn a_control_that_vanishes_hands_focus_to_its_nearest_neighbour() {
    // K8: on the same page, focus stays in the same neighbourhood
    let mut app = Demo::new(&["a", "b", "c", "d"]);
    app.primary = Some("a");
    draw(&mut app);
    app.shell.focus.focus(WidgetId::of("c"));
    draw(&mut app);
    app.buttons.retain(|b| *b != "c");
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("b")));
}

#[test]
fn a_page_change_with_lost_focus_goes_to_the_new_pages_primary() {
    let mut app = Demo::new(&["a", "b", "c"]);
    draw(&mut app);
    app.shell.focus.focus(WidgetId::of("c"));
    draw(&mut app);
    app.page = 2;
    app.buttons = vec!["x", "y", "z"];
    app.primary = Some("z");
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("z")));
}

#[test]
fn every_focus_change_reaches_on_blur() {
    let mut app = Demo::new(&["a", "b"]);
    draw(&mut app);
    let out = Application::handle(
        &mut app,
        Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }),
    );
    assert_eq!(out, Outcome::Changed);
    assert_eq!(
        app.blurs,
        vec![FocusChange {
            from: WidgetId::of("a"),
            to: Some(WidgetId::of("b")),
        }]
    );
}

#[test]
fn a_resize_records_the_size() {
    let mut app = Demo::new(&["a"]);
    let _ = Application::handle(&mut app, Input::Resize(33, 7));
    assert_eq!(app.shell.size, (33, 7));
}

#[test]
fn the_too_small_screen_draws_no_page_and_keeps_focus() {
    let mut app = Demo::new(&["a", "b"]);
    draw(&mut app);
    app.shell.focus.focus(WidgetId::of("b"));
    draw(&mut app);
    draw_at(&mut app, 10, 2);
    assert!(app.shell.too_small);
    assert_eq!(app.focused(), Some(WidgetId::of("b")));
}
