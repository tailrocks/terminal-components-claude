//! Initial layer focus must be judged against its new published placement.
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use junie_tui::{
    ActionKey, Anchor, App, Cx, Dismiss, Focusability, Id, Input, LayerSpec, Menu, MenuBar,
    MenuItem, MenuState, Position, Rect, Response, Runtime, Theme, Ui, UpdateCause,
};
use ratatui_core::buffer::Buffer;
const OWNER: Id = Id::root("reparent.owner");
const OTHER: Id = Id::root("reparent.other");
const INNER: Id = Id::root("reparent.inner");
const AREA: Rect = Rect::new(0, 0, 40, 12);
const ITEMS: &[MenuItem<'static>] = &[MenuItem::new(ActionKey::custom("reparent.open"), "Open")];
const MENUS: &[Menu<'static>] = &[Menu::new("File", ITEMS)];
#[derive(Clone, Copy, Default)]
enum Placement {
    #[default]
    Inside,
    Outside,
    Absent,
    Disabled,
}
#[derive(Clone, Copy)]
enum Request {
    Open,
    Outside,
    Nested,
    CloseReopen,
}
#[derive(Default)]
struct Model {
    state: MenuState,
    request: Option<Request>,
    placement: Placement,
    custom: bool,
    updates: usize,
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        for id in [OTHER, INNER] {
            for _ in cx.intents(id) {}
        }
        let _ = MenuBar::new(OWNER, MENUS).update(cx, &mut self.state);
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(OTHER);
        }
        if let Some(request) = self.request.take() {
            match request {
                Request::Open => {
                    let _ = MenuBar::new(OWNER, MENUS).open_menu(cx, &mut self.state, 0);
                }
                Request::Outside => cx.focus(OTHER),
                Request::Nested => {
                    let _ = MenuBar::new(OWNER, MENUS).open_menu(cx, &mut self.state, 0);
                    cx.open_layer(INNER, LayerSpec::modal(INNER).initial_focus(INNER));
                }
                Request::CloseReopen => {
                    let _ = MenuBar::new(OWNER, MENUS).open_menu(cx, &mut self.state, 0);
                    cx.close_layer(OWNER, None);
                    cx.open_layer(
                        OWNER,
                        LayerSpec::popover(OWNER, Anchor::Point(Position::new(0, 1)))
                            .dismiss(Dismiss::ALL),
                    );
                    cx.focus(OTHER);
                }
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(OTHER, Rect::new(0, 10, 5, 1), Focusability::Focusable);
        if !self.custom || self.state.open_menu().is_none() {
            MenuBar::new(OWNER, MENUS).draw(ui, Rect::new(0, 0, 40, 1), &self.state);
        } else {
            if matches!(self.placement, Placement::Outside) {
                ui.register_control(OWNER, Rect::new(0, 0, 4, 1), Focusability::Focusable);
            }
            ui.layer(OWNER, |ui, area| {
                if matches!(self.placement, Placement::Inside | Placement::Disabled) {
                    let f = if matches!(self.placement, Placement::Disabled) {
                        Focusability::Disabled
                    } else {
                        Focusability::Focusable
                    };
                    ui.register_control(OWNER, area, f);
                }
            });
        }
        ui.layer(INNER, |ui, area| {
            ui.register_control(INNER, area, Focusability::Focusable);
        });
    }
}
fn present(rt: &mut Runtime<Model>) {
    let mut buffer = Buffer::empty(AREA);
    for _ in 0..8 {
        rt.draw_buffer(AREA, &mut buffer).commit_presented();
        if !rt.needs_settle() {
            return;
        }
        let _ = rt.settle();
    }
    assert!(!rt.needs_settle(), "settlement failed to converge");
}
fn runtime() -> Runtime<Model> {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    assert_eq!(rt.focus(), Some(OTHER));
    rt
}
fn request(rt: &mut Runtime<Model>, request: Request) {
    rt.app_mut().request = Some(request);
    present(rt);
    let _ = rt.handle(Input::Tick).unwrap();
}
#[test]
fn menu_programmatic_open_reparents_existing_owner_without_premature_dismissal() {
    let mut rt = runtime();
    request(&mut rt, Request::Open);
    assert!(rt.is_open(OWNER));
    assert_eq!(rt.focus(), Some(OWNER));
    assert_eq!(rt.app().state.open_menu(), Some(0));
    let updates = rt.app().updates;
    drop(rt.draw_buffer(AREA, &mut Buffer::empty(AREA)));
    assert!(rt.is_open(OWNER));
    assert_eq!(rt.app().updates, updates);
    let paste = rt.handle(Input::Paste("queued".into())).unwrap_err();
    present(&mut rt);
    assert!(rt.is_open(OWNER));
    assert_eq!(rt.focus(), Some(OWNER));
    let _ = rt.handle(paste.into_input()).unwrap();
}
#[test]
fn actual_wrong_scope_absent_and_disabled_targets_cannot_keep_provisional_focus() {
    for placement in [Placement::Outside, Placement::Absent, Placement::Disabled] {
        let mut rt = runtime();
        rt.app_mut().custom = true;
        rt.app_mut().placement = placement;
        request(&mut rt, Request::Open);
        assert!(rt.is_open(OWNER));
        present(&mut rt);
        assert_ne!(rt.focus(), Some(OWNER));
        assert!(!rt.is_open(OWNER));
    }
}
#[test]
fn superseding_nested_modal_owns_focus_and_blocks_background_input() {
    let mut rt = runtime();
    request(&mut rt, Request::Nested);
    assert!(rt.is_open(OWNER));
    assert!(rt.is_open(INNER));
    assert_eq!(rt.focus(), Some(INNER));
    assert!(rt.handle(Input::Tick).is_err());
    present(&mut rt);
    assert!(rt.is_open(OWNER));
    assert!(rt.is_open(INNER));
    assert_eq!(rt.focus(), Some(INNER));
}
#[test]
fn ordinary_outside_focus_still_dismisses_after_reparenting() {
    let mut rt = runtime();
    request(&mut rt, Request::Open);
    present(&mut rt);
    request(&mut rt, Request::Outside);
    assert!(!rt.is_open(OWNER));
    assert_eq!(rt.focus(), Some(OTHER));
}
#[test]
fn closed_layer_provenance_does_not_authorize_reopened_layer() {
    let mut rt = runtime();
    request(&mut rt, Request::CloseReopen);
    present(&mut rt);
    assert_eq!(rt.focus(), Some(OTHER));
}
