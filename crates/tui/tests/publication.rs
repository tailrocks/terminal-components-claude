//! Successful output is the sole input-routing publication boundary.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "test assertions prove lifecycle boundaries and injected output failures"
)]
use junie_tui::{
    App, Cx, Focusability, Id, Input, Intent, Key, KeyCode, KeyModifiers, Rect, Response, Runtime,
    StateFlags, Theme, Ui, UpdateCause,
};
use ratatui_core::buffer::Buffer;

const EDITOR: Id = Id::root("publication.editor");
const AREA: Rect = Rect::new(0, 0, 40, 8);

#[derive(Default)]
struct Model {
    updates: usize,
    bootstraps: usize,
    pastes: Vec<String>,
    route: bool,
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates += 1;
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.bootstraps += 1;
        }
        for intent in cx.intents(EDITOR) {
            match intent {
                Intent::Paste(text) => self.pastes.push(text.to_owned()),
                Intent::Key(key) if key.code == KeyCode::Enter => self.route = true,
                _ => {}
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = if self.route {
            Rect::new(10, 2, 10, 1)
        } else {
            Rect::new(0, 0, 10, 1)
        };
        ui.register_editor(EDITOR, area, Focusability::Focusable, StateFlags::EDITING);
    }
}
fn present(rt: &mut Runtime<Model>, buf: &mut Buffer) {
    for _ in 0..8 {
        rt.draw_buffer(AREA, buf).commit_presented();
        if !rt.needs_settle() {
            return;
        }
        let _ = rt.settle();
    }
    panic!("fixture focus did not settle");
}
fn key() -> Input {
    Input::Key(Key {
        code: KeyCode::Enter,
        mods: KeyModifiers::NONE,
    })
}

#[test]
fn first_paste_is_owned_redacted_and_delivered_once_after_initialize_present_settle() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let pending = rt
        .handle(Input::Paste("private-first-paste".into()))
        .unwrap_err();
    assert!(!format!("{pending:?}").contains("private-first-paste"));
    assert_eq!(rt.app().updates, 0);
    let _ = rt.initialize();
    let pending = rt.handle(pending.into_input()).unwrap_err();
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert!(rt.needs_settle());
    let pending = rt.handle(pending.into_input()).unwrap_err();
    let _ = rt.settle();
    let pending = rt.handle(pending.into_input()).unwrap_err();
    present(&mut rt, &mut buf);
    let _ = rt.handle(pending.into_input()).unwrap();
    assert_eq!(rt.app().bootstraps, 1);
    assert_eq!(rt.app().pastes, ["private-first-paste"]);
}

#[test]
fn dropped_candidate_keeps_published_registry_and_blocks_input() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    let old = rt.area_of(EDITOR);
    rt.app_mut().route = true;
    drop(rt.draw_buffer(AREA, &mut buf));
    assert_eq!(rt.area_of(EDITOR), old);
    assert!(rt.handle(key()).is_err());
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert_eq!(rt.area_of(EDITOR), Some(Rect::new(10, 2, 10, 1)));
    let _ = rt.handle(key()).unwrap();
}

#[test]
fn queued_route_then_resize_then_paste_each_require_new_publication() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    let _ = rt.handle(key()).unwrap();
    let resize = rt.handle(Input::Resize(80, 20)).unwrap_err();
    assert_eq!(rt.screen(), AREA);
    present(&mut rt, &mut buf);
    let _ = rt.handle(resize.into_input()).unwrap();
    let paste = rt.handle(Input::Paste("queued".into())).unwrap_err();
    assert!(rt.app().pastes.is_empty());
    let area = Rect::new(0, 0, 80, 20);
    let mut resized = Buffer::empty(area);
    rt.draw_buffer(area, &mut resized).commit_presented();
    let _ = rt.handle(paste.into_input()).unwrap();
    assert_eq!(rt.app().pastes, ["queued"]);
}

#[test]
fn theme_and_model_mutation_require_publication_even_without_changed_response() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    rt.set_theme(Theme::paper());
    assert!(rt.handle(key()).is_err());
    present(&mut rt, &mut buf);
    let _model = rt.app_mut();
    assert!(rt.handle(key()).is_err());
    present(&mut rt, &mut buf);
    assert!(!rt.handle(key()).unwrap().is_changed());
    assert!(rt.handle(key()).is_err());
}

/// A backend that reaches the final output flush, then fails.
struct FailingBackend(ratatui_core::backend::TestBackend);
impl ratatui_core::backend::Backend for FailingBackend {
    type Error = std::io::Error;
    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>,
    {
        self.0.draw(content).unwrap();
        Ok(())
    }
    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_cursor_position(&mut self) -> Result<ratatui_core::layout::Position, Self::Error> {
        Ok(self.0.get_cursor_position().unwrap())
    }
    fn set_cursor_position<P: Into<ratatui_core::layout::Position>>(
        &mut self,
        p: P,
    ) -> Result<(), Self::Error> {
        self.0.set_cursor_position(p).unwrap();
        Ok(())
    }
    fn clear(&mut self) -> Result<(), Self::Error> {
        self.0.clear().unwrap();
        Ok(())
    }
    fn clear_region(
        &mut self,
        region: ratatui_core::backend::ClearType,
    ) -> Result<(), Self::Error> {
        self.0.clear_region(region).unwrap();
        Ok(())
    }
    fn size(&self) -> Result<ratatui_core::layout::Size, Self::Error> {
        Ok(self.0.size().unwrap())
    }
    fn window_size(&mut self) -> Result<ratatui_core::backend::WindowSize, Self::Error> {
        Ok(self.0.window_size().unwrap())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Err(std::io::Error::other("injected flush failure"))
    }
}

#[test]
fn failed_terminal_flush_does_not_publish_candidate_geometry() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    let old = rt.area_of(EDITOR);
    rt.app_mut().route = true;
    let mut terminal = ratatui_core::terminal::Terminal::new(FailingBackend(
        ratatui_core::backend::TestBackend::new(40, 8),
    ))
    .unwrap();
    let mut painted = None;
    let output = terminal.draw(|f| {
        painted = Some(rt.draw(f));
    });
    assert!(output.is_err());
    drop(painted);
    assert_eq!(rt.area_of(EDITOR), old);
    assert!(rt.needs_present());
    assert!(rt.handle(key()).is_err());
}

#[derive(Default)]
struct Nested {
    focus_ins: usize,
}
const OUTER: Id = Id::root("publication.outer");
const INNER: Id = Id::root("publication.inner");
const DUPLICATE: Id = Id::root("publication.ambiguous");
impl App for Nested {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(OUTER, junie_tui::LayerSpec::modal(OUTER));
            cx.open_layer(INNER, junie_tui::LayerSpec::modal(INNER));
        }
        for owner in [OUTER, INNER, EDITOR, DUPLICATE] {
            for intent in cx.intents(owner) {
                if matches!(intent, Intent::FocusIn { .. }) {
                    self.focus_ins += 1;
                }
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(OUTER, AREA, Focusability::Focusable);
        ui.layer(OUTER, |ui, area| {
            ui.register_control(OUTER.sub("control"), area, Focusability::Focusable);
        });
        ui.layer(INNER, |ui, area| {
            ui.register_control(DUPLICATE, area, Focusability::Disabled);
            ui.register_control(DUPLICATE, area, Focusability::Focusable);
            ui.register_editor(EDITOR, area, Focusability::Focusable, StateFlags::EDITING);
        });
    }
}

#[test]
fn nested_modal_skips_ambiguous_first_candidate_and_settles_once_outside_paint() {
    let mut rt = Runtime::new(Nested::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert_eq!(rt.focus(), Some(EDITOR));
    assert_eq!(rt.app().focus_ins, 0);
    assert!(rt.handle(key()).is_err());
    let _ = rt.settle();
    assert_eq!(rt.app().focus_ins, 1);
    assert!(rt.needs_present());
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert!(!rt.needs_settle());
    let _ = rt.settle();
    assert_eq!(rt.app().focus_ins, 1);
    let _ = rt.handle(key()).unwrap();
}

#[derive(Default)]
struct FocusChanges {
    second: bool,
    seen: Vec<(Id, &'static str)>,
}
const SECOND: Id = Id::root("publication.second");
impl App for FocusChanges {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for owner in [EDITOR, SECOND] {
            for intent in cx.intents(owner) {
                match intent {
                    Intent::FocusIn { .. } => self.seen.push((owner, "in")),
                    Intent::FocusOut { .. } => self.seen.push((owner, "out")),
                    _ => {}
                }
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(
            if self.second { SECOND } else { EDITOR },
            AREA,
            Focusability::Focusable,
        );
    }
}

#[test]
fn consecutive_publications_coalesce_only_unobserved_focus_transitions() {
    let mut rt = Runtime::new(FocusChanges::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    rt.app_mut().second = true;
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert!(rt.app().seen.is_empty());
    let _ = rt.settle();
    assert_eq!(rt.app().seen, [(SECOND, "in")]);
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    rt.app_mut().second = false;
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    rt.app_mut().second = true;
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert!(!rt.needs_settle());
    assert!(
        rt.needs_present(),
        "focus round trip still needs matching output"
    );
    assert!(rt.handle(key()).is_err());
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    let _ = rt.handle(key()).unwrap();
    assert_eq!(rt.app().seen, [(SECOND, "in")]);
}

#[test]
fn committing_repeated_frames_never_runs_application_updates_or_advances_time() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    let before = rt.app().updates;
    for _ in 0..1_000 {
        rt.draw_buffer(AREA, &mut buf).commit_presented();
    }
    assert_eq!(rt.app().updates, before);
    assert!(rt.app().pastes.is_empty());
    assert_eq!(rt.clock_ms(), 0);
    assert!(rt.needs_settle());
}

#[test]
#[expect(
    clippy::mem_forget,
    reason = "prove pre-paint invalidation survives a skipped destructor"
)]
fn aborting_or_forgetting_an_unchanged_candidate_still_blocks_input() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    drop(rt.draw_buffer(AREA, &mut buf));
    assert!(rt.handle(key()).is_err());
    present(&mut rt, &mut buf);
    std::mem::forget(rt.draw_buffer(AREA, &mut buf));
    assert!(rt.handle(key()).is_err());
}

#[derive(Default)]
struct FailingPainter {
    fail: std::cell::Cell<bool>,
}
impl App for FailingPainter {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, _ui: &mut Ui<'_>) {
        assert!(!self.fail.get(), "injected painter failure");
    }
}

#[test]
fn unwind_before_guard_construction_cannot_leave_old_geometry_compatible() {
    let mut rt = Runtime::new(FailingPainter::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    rt.app().fail.set(true);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        drop(rt.draw_buffer(AREA, &mut buf));
    }));
    assert!(result.is_err());
    assert!(rt.handle(key()).is_err());
}
