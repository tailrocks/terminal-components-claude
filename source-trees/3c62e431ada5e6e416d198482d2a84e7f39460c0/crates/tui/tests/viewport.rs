//! Current viewport and successfully published control geometry have distinct lifetimes.
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use junie_tui::{App, Cx, Focusability, Id, Input, Rect, Response, Runtime, Theme, Ui};
use ratatui_core::buffer::Buffer;
const OWNER: Id = Id::root("viewport.owner");
#[derive(Default)]
struct Model {
    seen: Vec<Rect>,
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.seen.push(cx.viewport());
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(OWNER, ui.full(), Focusability::Focusable);
    }
}
fn present(rt: &mut Runtime<Model>, area: Rect) {
    let mut buffer = Buffer::empty(area);
    for _ in 0..8 {
        rt.draw_buffer(area, &mut buffer).commit_presented();
        if !rt.needs_settle() {
            return;
        }
        let _ = rt.settle();
    }
    assert!(!rt.needs_settle(), "settlement did not converge");
}
#[test]
fn bootstrap_and_aborted_first_paint_have_no_viewport() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    assert_eq!(rt.screen(), Rect::ZERO);
    let _ = rt.initialize();
    assert_eq!(rt.app().seen, [Rect::ZERO]);
    let area = Rect::new(0, 0, 20, 6);
    drop(rt.draw_buffer(area, &mut Buffer::empty(area)));
    assert_eq!(rt.screen(), Rect::ZERO);
    assert_eq!(rt.app().seen, [Rect::ZERO]);
    present(&mut rt, area);
    assert_eq!(rt.screen(), area);
    assert_eq!(rt.app().seen.last(), Some(&area));
}
#[test]
fn resize_updates_current_viewport_before_replacing_published_geometry() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let _ = rt.initialize();
    let old = Rect::new(0, 0, 20, 6);
    present(&mut rt, old);
    let _ = rt.handle(Input::Resize(80, 12)).unwrap();
    let current = Rect::new(0, 0, 80, 12);
    assert_eq!(rt.app().seen.last(), Some(&current));
    assert_eq!(rt.screen(), current);
    assert_eq!(rt.area_of(OWNER), Some(old));
    assert!(rt.render_snapshot().is_err());
    let pending = rt.handle(Input::Paste("retained".into())).unwrap_err();
    drop(rt.draw_buffer(current, &mut Buffer::empty(current)));
    assert_eq!(rt.screen(), current);
    assert_eq!(rt.area_of(OWNER), Some(old));
    present(&mut rt, current);
    assert_eq!(rt.area_of(OWNER), Some(current));
    let _ = rt.handle(pending.into_input()).unwrap();
    assert_eq!(rt.app().seen.last(), Some(&current));
}
