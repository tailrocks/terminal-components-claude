//! Historical focus restoration waits for authoritative live geometry.
#![cfg(test)]
use junie_tui::{
    App, Cx, Focusability, Id, Intent, Item, ItemKey, KeyCode, Picker, PickerState, Rect, Response,
    Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;
const A: Id = Id::root("opener");
const B: Id = Id::root("survivor");
const MODAL: Id = Id::root("modal");
const ITEMS: &[Item<'static>] = &[Item::new(ItemKey::num(1), "one")];
struct Page {
    show: bool,
    state: PickerState,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(A);
        }
        if self.show {
            for intent in cx.intents(A) {
                if matches!(intent, Intent::Key(_)) {
                    cx.open_layer(MODAL, Picker::<Item<'_>>::new(MODAL).layer(cx, ITEMS));
                }
            }
        }
        for _ in cx.intents(B) {}
        Picker::new(MODAL)
            .update(cx, &mut self.state, ITEMS)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        if self.show {
            ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Focusable);
        }
        ui.register_control(B, Rect::new(0, 1, 10, 1), Focusability::Focusable);
        ui.layer(MODAL, |ui, area| {
            Picker::new(MODAL).draw(ui, area, &self.state, ITEMS)
        });
    }
}
#[test]
fn escape_does_not_restore_removed_opener() {
    let mut h = Harness::new(
        Page {
            show: true,
            state: PickerState::default(),
        },
        Theme::junie(),
        50,
        12,
    );
    let _ = h.key(KeyCode::Char('o'));
    assert!(h.layer_area(MODAL).is_some());
    h.app_mut().show = false;
    h.draw();
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(MODAL).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    assert_ne!(h.focus(), Some(A));
}

const INNER: Id = Id::root("restore.inner");
const FRESH: Id = Id::root("restore.fresh");
const AREA: Rect = Rect::new(0, 0, 50, 12);
#[derive(Clone, Copy, Default)]
enum Opener {
    #[default]
    Present,
    Removed,
    Disabled,
    Reparented,
}
#[derive(Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent lifecycle failure-injection switches"
)]
struct RestoreModel {
    opener: Opener,
    parent_disabled: bool,
    remove_on_close: bool,
    parent_popover: bool,
    fail_paint: bool,
    override_focus: bool,
    traversal: bool,
    fresh: bool,
    ins: Vec<Id>,
    outs: Vec<Id>,
}
impl App for RestoreModel {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(A);
        }
        for owner in [A, B, MODAL, MODAL.sub("fallback"), INNER, FRESH] {
            for intent in cx.intents(owner) {
                match intent {
                    Intent::FocusIn { .. } => self.ins.push(owner),
                    Intent::FocusOut { .. } => self.outs.push(owner),
                    Intent::Key(key) if key.code == KeyCode::Char('o') => {
                        let spec = if self.parent_popover {
                            junie_tui::LayerSpec::popover(
                                MODAL,
                                junie_tui::Anchor::Point(junie_tui::Position::new(0, 3)),
                            )
                        } else {
                            junie_tui::LayerSpec::modal(MODAL)
                        };
                        cx.open_layer(MODAL, spec.initial_focus(MODAL));
                    }
                    Intent::Key(key) if key.code == KeyCode::Char('n') => {
                        cx.open_layer(
                            INNER,
                            junie_tui::LayerSpec::modal(INNER).initial_focus(INNER),
                        );
                    }
                    Intent::Key(key) if key.code == KeyCode::Char('c') => {
                        if self.remove_on_close {
                            self.opener = Opener::Removed;
                        }
                        cx.close_layer(if owner == INNER { INNER } else { MODAL }, None);
                        if self.override_focus {
                            self.fresh = true;
                            cx.focus(FRESH);
                        }
                        if self.traversal {
                            cx.focus_next();
                        }
                    }
                    _ => {}
                }
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        assert!(!self.fail_paint, "injected painter failure");
        match self.opener {
            Opener::Present => {
                ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Focusable);
            }
            Opener::Disabled => {
                ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Disabled);
            }
            Opener::Reparented => {
                ui.focus_scope(A.sub("new.scope"), junie_tui::ScopeMode::Normal, |ui| {
                    ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Focusable);
                });
            }
            Opener::Removed => {}
        }
        ui.register_control(B, Rect::new(0, 1, 10, 1), Focusability::Focusable);
        if self.fresh {
            ui.register_control(FRESH, Rect::new(0, 2, 10, 1), Focusability::Focusable);
        }
        ui.layer(MODAL, |ui, area| {
            ui.register_control(
                MODAL,
                area,
                if self.parent_disabled {
                    Focusability::Disabled
                } else {
                    Focusability::Focusable
                },
            );
            ui.register_control(MODAL.sub("fallback"), area, Focusability::Focusable);
        });
        ui.layer(INNER, |ui, area| {
            ui.register_control(INNER, area, Focusability::Focusable);
        });
    }
}
fn publish(rt: &mut junie_tui::Runtime<RestoreModel>, buf: &mut ratatui_core::buffer::Buffer) {
    for _ in 0..12 {
        if rt.needs_present() {
            rt.draw_buffer(AREA, buf).commit_presented();
        }
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        if !rt.needs_present() && !rt.needs_settle() {
            return;
        }
    }
    assert!(
        !rt.needs_present(),
        "restoration never reached compatible publication"
    );
    assert!(!rt.needs_settle(), "restoration never settled");
}
fn key(rt: &mut junie_tui::Runtime<RestoreModel>, code: char) {
    assert!(
        rt.handle(junie_tui::Input::Key(junie_tui::Key {
            code: KeyCode::Char(code),
            mods: junie_tui::KeyModifiers::NONE
        }))
        .is_ok()
    );
}
fn opened() -> (
    junie_tui::Runtime<RestoreModel>,
    ratatui_core::buffer::Buffer,
) {
    let mut rt = junie_tui::Runtime::new(RestoreModel::default(), Theme::junie());
    let mut buf = ratatui_core::buffer::Buffer::empty(AREA);
    let _ = rt.initialize();
    publish(&mut rt, &mut buf);
    key(&mut rt, 'o');
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(MODAL));
    (rt, buf)
}
#[test]
fn removed_disabled_and_reparented_openers_use_published_eligibility() {
    for (opener, expected) in [
        (Opener::Removed, B),
        (Opener::Disabled, B),
        (Opener::Reparented, A),
    ] {
        let (mut rt, mut buf) = opened();
        rt.app_mut().opener = opener;
        publish(&mut rt, &mut buf);
        let before = rt.app().ins.len();
        key(&mut rt, 'c');
        assert_eq!(rt.focus(), None);
        assert_eq!(
            rt.app().outs.last(),
            Some(&MODAL),
            "closed owner must receive immediate FocusOut"
        );
        assert_eq!(
            rt.app().ins.len(),
            before,
            "historical opener received premature FocusIn"
        );
        publish(&mut rt, &mut buf);
        assert_eq!(rt.focus(), Some(expected));
        assert_eq!(rt.app().ins.get(before..), Some([expected].as_slice()));
        assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
    }
}
#[test]
fn fresh_programmatic_focus_and_traversal_supersede_restoration() {
    for traversal in [false, true] {
        let (mut rt, mut buf) = opened();
        rt.app_mut().override_focus = !traversal;
        rt.app_mut().traversal = traversal;
        rt.app_mut().opener = Opener::Removed;
        publish(&mut rt, &mut buf);
        key(&mut rt, 'c');
        publish(&mut rt, &mut buf);
        assert_eq!(rt.focus(), Some(if traversal { B } else { FRESH }));
        assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
    }
}
#[test]
fn nested_close_restores_only_inside_surviving_modal() {
    let (mut rt, mut buf) = opened();
    key(&mut rt, 'n');
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(INNER));
    key(&mut rt, 'c');
    assert_eq!(rt.app().outs.last(), Some(&INNER));
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(MODAL));
    assert!(rt.layer_area(MODAL).is_some());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}
#[test]
#[expect(
    clippy::expect_used,
    reason = "fixture must start from a compatible snapshot"
)]
fn dropped_and_projection_frames_cannot_acknowledge_restoration() {
    let (mut rt, mut buf) = opened();
    let snapshot = rt.render_snapshot().expect("compatible modal snapshot");
    key(&mut rt, 'c');
    let count = rt.app().ins.len();
    drop(rt.draw_buffer(AREA, &mut buf));
    rt.app_mut().fail_paint = true;
    let aborted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rt.draw_buffer(AREA, &mut buf).commit_presented();
    }));
    assert!(aborted.is_err());
    rt.app_mut().fail_paint = false;
    assert_eq!(rt.focus(), None);
    assert_eq!(rt.app().ins.len(), count);
    rt.draw_projection(AREA, &mut buf, &snapshot, |ui, area| {
        ui.register_control(FRESH, area, Focusability::Focusable);
    })
    .commit_inspected();
    assert_eq!(rt.focus(), None);
    assert_eq!(rt.app().ins.len(), count);
    assert!(
        rt.handle(junie_tui::Input::Key(junie_tui::Key {
            code: KeyCode::Enter,
            mods: junie_tui::KeyModifiers::NONE
        }))
        .is_err()
    );
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(A));
    assert_eq!(rt.app().ins.get(count..), Some([A].as_slice()));
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn nested_disabled_opener_falls_back_inside_parent_barrier() {
    let (mut rt, mut buf) = opened();
    key(&mut rt, 'n');
    publish(&mut rt, &mut buf);
    rt.app_mut().parent_disabled = true;
    publish(&mut rt, &mut buf);
    let count = rt.app().ins.len();
    key(&mut rt, 'c');
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(MODAL.sub("fallback")));
    assert_eq!(
        rt.app().ins.get(count..),
        Some([MODAL.sub("fallback")].as_slice())
    );
    assert!(rt.layer_area(MODAL).is_some());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn close_and_remove_in_one_update_never_notifies_old_opener() {
    let (mut rt, mut buf) = opened();
    rt.app_mut().remove_on_close = true;
    publish(&mut rt, &mut buf);
    let count = rt.app().ins.len();
    key(&mut rt, 'c');
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(B));
    assert_eq!(rt.app().ins.get(count..), Some([B].as_slice()));
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn temporary_restore_gap_does_not_dismiss_surviving_popover() {
    let mut rt = junie_tui::Runtime::new(
        RestoreModel {
            parent_popover: true,
            ..RestoreModel::default()
        },
        Theme::junie(),
    );
    let mut buf = ratatui_core::buffer::Buffer::empty(AREA);
    let _ = rt.initialize();
    publish(&mut rt, &mut buf);
    key(&mut rt, 'o');
    publish(&mut rt, &mut buf);
    key(&mut rt, 'n');
    publish(&mut rt, &mut buf);
    key(&mut rt, 'c');
    assert!(rt.layer_area(MODAL).is_some());
    publish(&mut rt, &mut buf);
    assert_eq!(rt.focus(), Some(MODAL));
    assert!(rt.layer_area(MODAL).is_some());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}
