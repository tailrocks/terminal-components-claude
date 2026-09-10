//! Held pointer targets must remain eligible across live publication.
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use junie_tui::*;
use ratatui_core::buffer::Buffer;
const A: Id = Id::root("audit.a");
const B: Id = Id::root("audit.b");
const POP: Id = Id::root("audit.pop");
const AREA: Rect = Rect::new(0, 0, 30, 8);
const THUMB: PartRef = PartRef::of(Part::THUMB);
#[derive(Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent regression fixture axes"
)]
struct Model {
    moved: bool,
    disabled: bool,
    capture: bool,
    remove_part: bool,
    decor_part: bool,
    empty_part: bool,
    open: bool,
    close: bool,
    claim: Option<(Id, PartRef)>,
    claim_result: Option<bool>,
    flash: bool,
    origin: Option<Position>,
    capture_area: Option<Rect>,
    phases: Vec<Phase>,
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut press = false;
        for intent in cx.intents(A) {
            if let Intent::Pointer { phase, .. } = intent {
                self.phases.push(phase);
                press |= phase == Phase::Press;
            }
        }
        for id in [B, POP] {
            for _ in cx.intents(id) {}
        }
        if press && self.capture {
            assert!(cx.capture(A, THUMB));
        }
        if self.open {
            self.open = false;
            cx.open_layer(
                POP,
                LayerSpec::popover(POP, Anchor::Screen(ScreenAlign::Center)),
            );
        }
        if self.close {
            self.close = false;
            cx.close_layer(POP, None);
        }
        if let Some((owner, part)) = self.claim.take() {
            self.claim_result = Some(cx.capture(owner, part));
        }
        if self.flash {
            self.flash = false;
            cx.flash_activation(B);
        }
        self.origin = cx.capture_origin();
        self.capture_area = cx.capture_area();
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let a = Rect::new(if self.moved { 10 } else { 0 }, 0, 8, 2);
        let b = Rect::new(if self.moved { 0 } else { 10 }, 0, 8, 2);
        ui.register_control(
            A,
            a,
            if self.disabled {
                Focusability::Disabled
            } else {
                Focusability::Focusable
            },
        );
        if !self.remove_part {
            let area = if self.empty_part {
                Rect::ZERO
            } else {
                Rect::new(a.x, 0, 4, 2)
            };
            if self.decor_part {
                ui.register_decor(A, THUMB, area);
            } else {
                ui.register_part(A, THUMB, area);
            }
        }
        ui.register_control(B, b, Focusability::Focusable);
        ui.layer(POP, |ui, area| {
            ui.register_control(POP, area, Focusability::Focusable);
        });
    }
}
fn present(rt: &mut Runtime<Model>) {
    for _ in 0..8 {
        rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
        let _ = rt.settle();
    }
    assert!(!rt.needs_settle() && !rt.needs_present(), "nonconvergent");
}
fn runtime(capture: bool) -> Runtime<Model> {
    let mut rt = Runtime::new(
        Model {
            capture,
            ..Model::default()
        },
        Theme::junie(),
    );
    let _ = rt.initialize();
    present(&mut rt);
    rt
}
fn mouse(rt: &mut Runtime<Model>, kind: MouseKind) {
    mouse_at(rt, kind, Position::new(1, 1));
}
fn mouse_at(rt: &mut Runtime<Model>, kind: MouseKind, pos: Position) {
    let _ = rt
        .handle(Input::Mouse(Mouse {
            kind,
            pos,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(rt);
}
#[test]
fn stationary_pointer_tracks_replaced_geometry() {
    let mut rt = runtime(false);
    mouse(&mut rt, MouseKind::Move);
    assert!(rt.state_of(A).contains(StateFlags::HOVERED));
    rt.app_mut().moved = true;
    present(&mut rt);
    assert!(!rt.state_of(A).contains(StateFlags::HOVERED));
    assert!(rt.state_of(B).contains(StateFlags::HOVERED));
}
#[test]
fn disabled_capture_cannot_receive_drag_or_click() {
    let mut rt = runtime(true);
    mouse(&mut rt, MouseKind::Down);
    rt.app_mut().disabled = true;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse(&mut rt, MouseKind::Drag);
    mouse(&mut rt, MouseKind::Up);
    assert!(
        !rt.app()
            .phases
            .get(n..)
            .unwrap()
            .iter()
            .any(|p| matches!(p, Phase::Drag | Phase::Click))
    );
}
#[test]
fn vanished_part_cancels_capture_even_while_owner_remains() {
    let mut rt = runtime(true);
    mouse(&mut rt, MouseKind::Down);
    rt.app_mut().remove_part = true;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse(&mut rt, MouseKind::Drag);
    assert!(!rt.app().phases.get(n..).unwrap().contains(&Phase::Drag));
}
#[test]
fn noninert_popover_blocks_prior_background_capture() {
    let mut rt = runtime(true);
    mouse(&mut rt, MouseKind::Down);
    rt.app_mut().open = true;
    present(&mut rt);
    let _ = rt.handle(Input::Tick).unwrap();
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse(&mut rt, MouseKind::Drag);
    assert!(!rt.app().phases.get(n..).unwrap().contains(&Phase::Drag));
}
#[test]
fn disabled_uncaptured_press_cannot_activate_on_release() {
    let mut rt = runtime(false);
    mouse(&mut rt, MouseKind::Down);
    rt.app_mut().disabled = true;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse(&mut rt, MouseKind::Up);
    assert!(!rt.app().phases.get(n..).unwrap().contains(&Phase::Click));
}

fn tick(rt: &mut Runtime<Model>) {
    present(rt);
    let _ = rt.handle(Input::Tick).unwrap();
    present(rt);
}

#[test]
fn invalid_claims_do_not_fabricate_target_or_origin() {
    for target in [
        (Id::root("absent"), THUMB),
        (A, PartRef::of(Part::LABEL)),
        (A, THUMB),
    ] {
        let mut rt = runtime(false);
        rt.app_mut().claim = Some(target);
        tick(&mut rt);
        assert_eq!(rt.app().claim_result, Some(false), "no press: {target:?}");
        assert_eq!(rt.capture_owner(), None);
    }
    for target in [(Id::root("absent"), THUMB), (A, PartRef::of(Part::LABEL))] {
        let mut rt = runtime(false);
        mouse(&mut rt, MouseKind::Down);
        rt.app_mut().claim = Some(target);
        tick(&mut rt);
        assert_eq!(rt.app().claim_result, Some(false));
        assert_eq!(rt.capture_owner(), None);
    }
}

#[test]
fn disabling_then_reenabling_does_not_resurrect_gesture_or_cancel_feedback() {
    for capture in [false, true] {
        let mut rt = runtime(capture);
        mouse(&mut rt, MouseKind::Down);
        rt.app_mut().flash = true;
        tick(&mut rt);
        let feedback = rt.activation_feedback();
        rt.app_mut().disabled = true;
        present(&mut rt);
        assert_eq!(rt.capture_owner(), None);
        assert_eq!(rt.activation_feedback(), feedback);
        rt.app_mut().disabled = false;
        present(&mut rt);
        let n = rt.app().phases.len();
        mouse(&mut rt, MouseKind::Up);
        assert!(
            !rt.app()
                .phases
                .get(n..)
                .unwrap()
                .iter()
                .any(|phase| matches!(phase, Phase::Release | Phase::Click))
        );
    }
}

#[test]
fn moving_a_valid_part_preserves_capture_area_and_press_origin() {
    let mut rt = runtime(true);
    mouse(&mut rt, MouseKind::Down);
    let origin = rt.app().origin;
    let area = rt.app().capture_area;
    rt.app_mut().moved = true;
    present(&mut rt);
    mouse(&mut rt, MouseKind::Drag);
    assert_eq!(rt.capture_owner(), Some(A));
    assert_eq!(rt.app().origin, origin);
    assert_eq!(rt.app().capture_area, area);
    assert_eq!(origin, Some(Position::new(1, 1)));
    assert_eq!(area, Some(Rect::new(0, 0, 4, 2)));
    mouse(&mut rt, MouseKind::Up);
    assert!(rt.app().phases.contains(&Phase::Click));
}

#[test]
fn resize_and_layer_roundtrip_cancel_held_gesture_before_reenable() {
    for resize in [false, true] {
        let mut rt = runtime(true);
        mouse(&mut rt, MouseKind::Down);
        if resize {
            let _ = rt.handle(Input::Resize(31, 9)).unwrap();
            present(&mut rt);
        } else {
            rt.app_mut().open = true;
            tick(&mut rt);
            assert_eq!(rt.capture_owner(), None);
            rt.app_mut().close = true;
            tick(&mut rt);
        }
        rt.app_mut().claim = Some((A, THUMB));
        tick(&mut rt);
        assert_eq!(rt.app().claim_result, Some(false));
        let n = rt.app().phases.len();
        mouse(&mut rt, MouseKind::Up);
        assert!(!rt.app().phases.get(n..).unwrap().contains(&Phase::Click));
    }
}

#[test]
fn dropped_disabled_frame_does_not_cancel_until_successful_publication() {
    let mut rt = runtime(true);
    mouse(&mut rt, MouseKind::Down);
    rt.app_mut().disabled = true;
    drop(rt.draw_buffer(AREA, &mut Buffer::empty(AREA)));
    assert_eq!(rt.capture_owner(), Some(A));
    present(&mut rt);
    assert_eq!(rt.capture_owner(), None);
}

#[test]
fn decorative_or_empty_replacement_cancels_actual_part_capture() {
    for decor in [false, true] {
        let mut rt = runtime(true);
        mouse(&mut rt, MouseKind::Down);
        if decor {
            rt.app_mut().decor_part = true;
        } else {
            rt.app_mut().empty_part = true;
        }
        present(&mut rt);
        assert_eq!(rt.capture_owner(), None);
        let n = rt.app().phases.len();
        mouse(&mut rt, MouseKind::Drag);
        mouse(&mut rt, MouseKind::Up);
        assert!(
            !rt.app()
                .phases
                .get(n..)
                .unwrap()
                .iter()
                .any(|phase| matches!(phase, Phase::Drag | Phase::Click))
        );
    }
}

#[test]
fn live_press_cannot_claim_disabled_decorative_empty_or_lower_layer_target() {
    for case in 0..4 {
        let mut rt = runtime(false);
        match case {
            0 => rt.app_mut().disabled = true,
            1 => rt.app_mut().decor_part = true,
            2 => rt.app_mut().empty_part = true,
            _ => {}
        }
        present(&mut rt);
        mouse_at(&mut rt, MouseKind::Down, Position::new(11, 1));
        rt.app_mut().claim = Some((A, THUMB));
        if case == 3 {
            rt.app_mut().open = true;
        }
        tick(&mut rt);
        assert_eq!(rt.app().claim_result, Some(false), "case {case}");
        assert_eq!(rt.capture_owner(), None);
    }
}
