//! Deferred navigation resolves against the next frame, not the previous route.

use junie_tui::{
    App, Cx, Focusability, Id, Input, Intent, Key, KeyCode, KeyModifiers, LayerSpec, Rect,
    Response, Runtime, Theme, Ui,
};
use ratatui_core::buffer::Buffer;

const NAV: Id = Id::root("traversal.nav");
const OLD: Id = Id::root("traversal.old");
const NEW: Id = Id::root("traversal.new");
const LAST: Id = Id::root("traversal.last");
const MODAL: Id = Id::root("traversal.modal");
const MODAL_FIRST: Id = Id::root("traversal.modal.first");
const MODAL_LAST: Id = Id::root("traversal.modal.last");
const AREA: Rect = Rect::new(0, 0, 40, 12);

#[derive(Clone, Copy)]
enum Request {
    Next,
    NextThenDirect,
    DirectThenNext,
    OpenModal,
    OpenPopover,
}

#[derive(Default)]
struct Page {
    changed_route: bool,
    remove_nav: bool,
    disable_first: bool,
    request: Option<Request>,
    updates: usize,
    focus_ins: usize,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        for owner in [NAV, OLD, NEW, LAST, MODAL, MODAL_FIRST, MODAL_LAST] {
            for intent in cx.intents(owner) {
                if matches!(intent, Intent::FocusIn { .. }) {
                    self.focus_ins = self.focus_ins.saturating_add(1);
                }
            }
        }
        match self.request.take() {
            Some(Request::Next) => cx.focus_next(),
            Some(Request::NextThenDirect) => {
                cx.focus_next();
                cx.focus(LAST);
            }
            Some(Request::DirectThenNext) => {
                cx.focus(LAST);
                cx.focus_next();
            }
            Some(Request::OpenModal) => {
                cx.focus_next();
                cx.open_layer(MODAL, LayerSpec::modal(MODAL));
            }
            Some(Request::OpenPopover) => {
                cx.open_layer(
                    MODAL,
                    LayerSpec::popover(
                        MODAL,
                        junie_tui::Anchor::Screen(junie_tui::ScreenAlign::Center),
                    )
                    .initial_focus(MODAL_LAST)
                    .dismiss(junie_tui::Dismiss::ALL),
                );
            }
            None => return Response::ignored(),
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if !self.remove_nav {
            ui.register_control(NAV, Rect::new(0, 0, 8, 1), Focusability::Focusable);
        }
        let first = if self.changed_route { NEW } else { OLD };
        let enabled = if self.disable_first {
            Focusability::Disabled
        } else {
            Focusability::Focusable
        };
        ui.register_control(first, Rect::new(0, 2, 8, 1), enabled);
        ui.register_control(LAST, Rect::new(0, 4, 8, 1), Focusability::Focusable);
        ui.layer(MODAL, |ui, _| {
            ui.register_control(MODAL_FIRST, Rect::new(0, 6, 8, 1), Focusability::Focusable);
            ui.register_control(MODAL_LAST, Rect::new(0, 8, 8, 1), Focusability::Focusable);
        });
    }
}

fn input() -> Input {
    Input::Key(Key {
        code: KeyCode::Char('x'),
        mods: KeyModifiers::NONE,
    })
}

fn runtime() -> (Runtime<Page>, Buffer) {
    let mut rt = Runtime::new(Page::default(), Theme::junie());
    let _ = rt.initialize();
    let mut buffer = Buffer::empty(AREA);
    rt.draw_buffer(AREA, &mut buffer);
    let _ = rt.handle(input());
    (rt, buffer)
}

#[test]
fn enter_content_uses_new_route_and_does_not_update_during_draw() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().changed_route = true;
    rt.app_mut().request = Some(Request::Next);
    let response = rt.handle(input());
    assert!(response.invalidate() >= junie_tui::Invalidate::Paint);
    assert_eq!(rt.focus(), Some(NAV));
    let updates = rt.app().updates;
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(NEW));
    assert_eq!(rt.app().updates, updates);
    let before = rt.app().focus_ins;
    let _ = rt.handle(input());
    assert_eq!(rt.app().focus_ins, before + 1);
    rt.draw_buffer(AREA, &mut buffer);
    rt.draw_buffer(AREA, &mut buffer);
    let _ = rt.handle(input());
    assert_eq!(rt.app().focus_ins, before + 1);
}

#[test]
fn deferred_next_skips_disabled_new_content() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().changed_route = true;
    rt.app_mut().disable_first = true;
    rt.app_mut().request = Some(Request::Next);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(LAST));
}

#[test]
fn removed_anchor_reconciles_to_nearest_surviving_control() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().remove_nav = true;
    rt.app_mut().request = Some(Request::Next);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(OLD));
}

#[test]
fn new_modal_traps_a_deferred_content_request() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().request = Some(Request::OpenModal);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(MODAL_FIRST));
    rt.app_mut().request = Some(Request::Next);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(MODAL_LAST));
}

#[test]
fn later_direct_focus_supersedes_deferred_traversal() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().request = Some(Request::NextThenDirect);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(LAST));
}

#[test]
fn later_traversal_supersedes_direct_focus() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().request = Some(Request::DirectThenNext);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(OLD));
}

#[test]
fn deferred_focus_out_dismisses_popover_in_update_only() {
    let (mut rt, mut buffer) = runtime();
    rt.app_mut().request = Some(Request::OpenPopover);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(MODAL_LAST));
    rt.app_mut().request = Some(Request::Next);
    let _ = rt.handle(input());
    rt.draw_buffer(AREA, &mut buffer);
    assert_eq!(rt.focus(), Some(NAV));
    assert!(rt.is_open(MODAL), "painting must not dismiss a layer");
    let _ = rt.handle(input());
    assert!(!rt.is_open(MODAL));
    assert_eq!(rt.focus(), Some(NAV));
}
