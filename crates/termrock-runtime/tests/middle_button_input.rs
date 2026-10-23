#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use termrock_core::event::{Input, KeyModifiers, RoutedInput};
use termrock_core::{Id, Intent, Part, PartRef, Phase, Response};
use termrock_runtime::{App, Cx, Focusability, LayerSpec, Runtime, Ui};
use termrock_theme::{StateFlags, Theme};

const AREA: Rect = Rect::new(0, 0, 40, 14);
const FOCUS: Id = Id::root("middle.focus");
const TARGET: Id = Id::root("middle.target");
const MODAL: Id = Id::root("middle.modal");
const HOVER: Id = Id::root("middle.hover");
const ITEM: PartRef = PartRef::of(Part::THUMB);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointerIntent {
    owner: Id,
    phase: Phase,
    part: PartRef,
    pos: Position,
    local: Position,
    mods: KeyModifiers,
}

#[derive(Default)]
struct Probe {
    updates: usize,
    pointers: Vec<PointerIntent>,
    unexpected: usize,
    capture_on_press: bool,
    open_modal: bool,
    modal_visible: bool,
    disabled_target: bool,
    decor_target: bool,
    hover_target: bool,
    reference_target: bool,
    remove_target_on_middle: bool,
    hide_target: bool,
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        let mut capture_requested = false;
        for owner in [FOCUS, TARGET, MODAL, HOVER] {
            for intent in cx.intents(owner) {
                match intent {
                    Intent::Pointer {
                        phase,
                        part,
                        pos,
                        local,
                        mods,
                    } => {
                        self.pointers.push(PointerIntent {
                            owner,
                            phase,
                            part,
                            pos,
                            local,
                            mods,
                        });
                        capture_requested |= owner == TARGET && phase == Phase::Press;
                        if owner == FOCUS && phase == Phase::Middle && self.remove_target_on_middle
                        {
                            self.remove_target_on_middle = false;
                            self.hide_target = true;
                        }
                    }
                    _ => self.unexpected = self.unexpected.saturating_add(1),
                }
            }
        }
        if capture_requested && self.capture_on_press {
            assert!(cx.capture(TARGET, ITEM));
        }
        if self.open_modal {
            self.open_modal = false;
            self.modal_visible = true;
            let mut spec = LayerSpec::modal(MODAL);
            spec.size = termrock_runtime::LayerSize::Fixed(8, 4);
            cx.open_layer(MODAL, spec);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(FOCUS, Rect::new(1, 1, 5, 2), Focusability::Focusable);
        if self.reference_target {
            ui.reference(None, |ui| {
                ui.register_control(TARGET, Rect::new(10, 5, 10, 4), Focusability::Focusable);
                ui.register_part(TARGET, ITEM, Rect::new(12, 6, 4, 2));
            });
        } else if !self.hide_target {
            ui.register_control(
                TARGET,
                Rect::new(10, 5, 10, 4),
                if self.disabled_target {
                    Focusability::Disabled
                } else {
                    Focusability::Focusable
                },
            );
            if self.hover_target {
                ui.register_part(TARGET, ITEM, Rect::new(12, 6, 4, 2));
                ui.register_hover_part(HOVER, ITEM, Rect::new(12, 6, 4, 2));
            } else if self.decor_target {
                ui.register_decor(TARGET, ITEM, Rect::new(12, 6, 4, 2));
            } else {
                ui.register_part(TARGET, ITEM, Rect::new(12, 6, 4, 2));
            }
        }
        if self.modal_visible {
            ui.layer(MODAL, |ui, area| {
                ui.register_control(MODAL, area, Focusability::Focusable);
            });
        }
    }
}

fn present(rt: &mut Runtime<Probe>) {
    for _ in 0..16 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        if rt.needs_present() {
            rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
                .commit_presented();
        }
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
    }
    panic!("runtime did not settle and present");
}

fn mouse(kind: MouseEventKind, x: u16, y: u16, mods: KeyModifiers) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: x,
        row: y,
        modifiers: mods.into(),
    })
}

fn ready_runtime(probe: Probe) -> Runtime<Probe> {
    let mut rt = Runtime::new(probe, Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    rt
}

#[test]
fn decoded_middle_routes_one_exact_part_without_primary_side_effects() {
    let mut rt = ready_runtime(Probe::default());
    rt.set_focus(Some(FOCUS));
    present(&mut rt);
    assert_eq!(rt.focus(), Some(FOCUS));
    let focus_visible = rt.focus_visible();
    let baseline_updates = rt.app().updates;
    rt.app_mut().pointers.clear();
    rt.app_mut().unexpected = 0;
    present(&mut rt);

    let mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    let input = RoutedInput::from_crossterm(mouse(
        MouseEventKind::Down(MouseButton::Middle),
        13,
        6,
        mods,
    ))
    .expect("middle Down is a routed input");
    let response = rt
        .handle_routed(input)
        .expect("ready runtime handles input");

    assert!(!response.is_consumed(), "page miss policy stays ignored");
    assert_eq!(rt.app().updates, baseline_updates + 1);
    assert_eq!(
        rt.app().pointers,
        vec![PointerIntent {
            owner: TARGET,
            phase: Phase::Middle,
            part: ITEM,
            pos: Position::new(13, 6),
            local: Position::new(1, 0),
            mods,
        }]
    );
    assert_eq!(rt.app().unexpected, 0);
    assert_eq!(rt.focus(), Some(FOCUS));
    assert_eq!(rt.focus_visible(), focus_visible);
    assert_eq!(rt.capture_owner(), None);
    assert!(!rt.state_of(TARGET).contains(StateFlags::PRESSED));
    assert_eq!(rt.activation_feedback(), None);

    rt.app_mut().pointers.clear();
    present(&mut rt);
    let miss = rt
        .handle_routed(
            RoutedInput::from_crossterm(mouse(
                MouseEventKind::Down(MouseButton::Middle),
                35,
                12,
                mods,
            ))
            .expect("middle down is preserved even when the page hit misses"),
        )
        .expect("ready runtime handles a page miss");
    assert!(!miss.is_consumed());
    assert!(rt.app().pointers.is_empty());
}

#[test]
fn middle_down_drag_up_decodes_as_exactly_one_semantic_event() {
    let mut rt = ready_runtime(Probe::default());
    for kind in [
        MouseEventKind::Down(MouseButton::Middle),
        MouseEventKind::Drag(MouseButton::Middle),
        MouseEventKind::Up(MouseButton::Middle),
    ] {
        if let Some(input) = RoutedInput::from_crossterm(mouse(kind, 13, 6, KeyModifiers::NONE)) {
            let _ = rt.handle_routed(input).expect("middle down is handled");
            present(&mut rt);
        }
    }
    assert_eq!(
        rt.app()
            .pointers
            .iter()
            .filter(|intent| intent.phase == Phase::Middle)
            .count(),
        1
    );
}

#[test]
fn disabled_and_decorative_targets_never_receive_middle() {
    for probe in [
        Probe {
            disabled_target: true,
            ..Probe::default()
        },
        Probe {
            decor_target: true,
            ..Probe::default()
        },
        Probe {
            reference_target: true,
            ..Probe::default()
        },
    ] {
        let mut rt = ready_runtime(probe);
        let input = RoutedInput::from_crossterm(mouse(
            MouseEventKind::Down(MouseButton::Middle),
            13,
            6,
            KeyModifiers::NONE,
        ))
        .expect("middle down is preserved");
        let _ = rt
            .handle_routed(input)
            .expect("ready runtime handles input");
        assert!(rt.app().pointers.is_empty());
    }
}

#[test]
fn hover_only_region_is_skipped_and_does_not_mask_real_control() {
    let mut rt = ready_runtime(Probe {
        hover_target: true,
        ..Probe::default()
    });
    let _ = rt
        .handle_routed(
            RoutedInput::from_crossterm(mouse(
                MouseEventKind::Down(MouseButton::Middle),
                13,
                6,
                KeyModifiers::NONE,
            ))
            .expect("middle down is preserved"),
        )
        .expect("ready runtime handles input");
    assert_eq!(rt.app().pointers.len(), 1);
    assert_eq!(rt.app().pointers[0].owner, TARGET);
    assert_eq!(rt.app().pointers[0].part, ITEM);
}

#[test]
fn open_modal_consumes_hit_and_miss_without_falling_through() {
    let mut rt = ready_runtime(Probe {
        open_modal: true,
        ..Probe::default()
    });
    assert!(rt.is_open(MODAL));
    let modal_rect = rt
        .area_of(MODAL)
        .expect("modal target has published bounds");
    let modal_pos = Position::new(modal_rect.x + 1, modal_rect.y + 1);

    for pos in [modal_pos, Position::new(13, 6)] {
        rt.app_mut().pointers.clear();
        rt.app_mut().unexpected = 0;
        present(&mut rt);
        let response = rt
            .handle_routed(
                RoutedInput::from_crossterm(mouse(
                    MouseEventKind::Down(MouseButton::Middle),
                    pos.x,
                    pos.y,
                    KeyModifiers::ALT,
                ))
                .expect("middle down is preserved"),
            )
            .expect("ready runtime handles input");
        assert!(
            response.is_consumed(),
            "an open non-page layer swallows input"
        );
        assert!(rt.is_open(MODAL), "middle does not dismiss the modal");
        if pos == modal_pos {
            assert_eq!(
                rt.app().pointers,
                vec![PointerIntent {
                    owner: MODAL,
                    phase: Phase::Middle,
                    part: PartRef::of(Part::CONTAINER),
                    pos,
                    local: Position::new(1, 1),
                    mods: KeyModifiers::ALT,
                }]
            );
        } else {
            assert!(
                rt.app().pointers.is_empty(),
                "page target is inert below modal"
            );
        }
        assert_eq!(rt.app().unexpected, 0);
        present(&mut rt);
    }
}

#[test]
fn middle_input_preserves_live_primary_capture_and_following_drag() {
    let mut rt = ready_runtime(Probe {
        capture_on_press: true,
        ..Probe::default()
    });
    let _ = rt
        .handle(Input::Mouse(termrock_core::Mouse {
            kind: termrock_core::MouseKind::Down,
            pos: Position::new(13, 6),
            mods: KeyModifiers::NONE,
        }))
        .expect("legacy primary down remains supported");
    present(&mut rt);
    assert_eq!(rt.capture_owner(), Some(TARGET));
    rt.app_mut().pointers.clear();
    present(&mut rt);

    let middle = RoutedInput::from_crossterm(mouse(
        MouseEventKind::Down(MouseButton::Middle),
        2,
        1,
        KeyModifiers::NONE,
    ))
    .expect("middle down is preserved");
    let _ = rt
        .handle_routed(middle)
        .expect("ready runtime handles input");
    assert_eq!(rt.capture_owner(), Some(TARGET));
    assert_eq!(rt.app().pointers[0].phase, Phase::Middle);
    assert_eq!(rt.app().pointers[0].owner, FOCUS);
    present(&mut rt);

    let _ = rt
        .handle(Input::Mouse(termrock_core::Mouse {
            kind: termrock_core::MouseKind::Drag,
            pos: Position::new(30, 10),
            mods: KeyModifiers::NONE,
        }))
        .expect("ordinary primary drag continues through capture");
    assert!(
        rt.app()
            .pointers
            .iter()
            .any(|intent| { intent.owner == TARGET && intent.phase == Phase::Drag })
    );
    present(&mut rt);
    let _ = rt
        .handle(Input::Mouse(termrock_core::Mouse {
            kind: termrock_core::MouseKind::Up,
            pos: Position::new(30, 10),
            mods: KeyModifiers::NONE,
        }))
        .expect("ordinary primary release follows the captured drag");
    assert!(
        rt.app()
            .pointers
            .iter()
            .any(|intent| { intent.owner == TARGET && intent.phase == Phase::Release })
    );
    assert!(
        rt.app()
            .pointers
            .iter()
            .any(|intent| { intent.owner == TARGET && intent.phase == Phase::DragEnd })
    );
    assert_eq!(rt.capture_owner(), None);
}

#[test]
fn consumer_removal_still_invalidates_capture_after_middle() {
    let mut rt = ready_runtime(Probe {
        capture_on_press: true,
        remove_target_on_middle: true,
        ..Probe::default()
    });
    let _ = rt
        .handle(Input::Mouse(termrock_core::Mouse {
            kind: termrock_core::MouseKind::Down,
            pos: Position::new(13, 6),
            mods: KeyModifiers::NONE,
        }))
        .expect("legacy primary down remains supported");
    present(&mut rt);
    assert_eq!(rt.capture_owner(), Some(TARGET));

    let _ = rt
        .handle_routed(
            RoutedInput::from_crossterm(mouse(
                MouseEventKind::Down(MouseButton::Middle),
                2,
                1,
                KeyModifiers::NONE,
            ))
            .expect("middle down is preserved"),
        )
        .expect("ready runtime handles input");
    present(&mut rt);
    assert_eq!(rt.capture_owner(), None);
}

#[test]
fn middle_does_not_change_the_primary_click_count() {
    let mut rt = ready_runtime(Probe::default());
    let primary = |kind| {
        Input::Mouse(termrock_core::Mouse {
            kind,
            pos: Position::new(13, 6),
            mods: KeyModifiers::NONE,
        })
    };
    let click = |rt: &mut Runtime<Probe>| {
        let _ = rt.handle(primary(termrock_core::MouseKind::Down)).unwrap();
        present(rt);
        let _ = rt.handle(primary(termrock_core::MouseKind::Up)).unwrap();
        present(rt);
    };

    click(&mut rt);
    assert!(
        rt.app()
            .pointers
            .iter()
            .any(|intent| intent.phase == Phase::Click)
    );
    rt.app_mut().pointers.clear();
    present(&mut rt);

    let _ = rt
        .handle_routed(
            RoutedInput::from_crossterm(mouse(
                MouseEventKind::Down(MouseButton::Middle),
                13,
                6,
                KeyModifiers::NONE,
            ))
            .expect("middle down is preserved"),
        )
        .unwrap();
    present(&mut rt);
    assert_eq!(rt.app().pointers.len(), 1);
    assert_eq!(rt.app().pointers[0].phase, Phase::Middle);
    rt.app_mut().pointers.clear();
    present(&mut rt);

    click(&mut rt);
    assert!(
        rt.app()
            .pointers
            .iter()
            .any(|intent| intent.phase == Phase::DoubleClick),
        "a middle press must not reset or advance the primary click sequence"
    );
}

#[test]
fn routed_pending_input_is_lossless_until_presented_and_settled() {
    let event = || RoutedInput::MiddleClick {
        pos: Position::new(13, 6),
        mods: KeyModifiers::ALT,
    };
    let mut unbootstrapped = Runtime::new(Probe::default(), Theme::junie());
    assert_eq!(
        unbootstrapped
            .handle_routed(event())
            .unwrap_err()
            .into_routed_input(),
        event()
    );

    let mut rt = Runtime::new(Probe::default(), Theme::junie());
    let _ = rt.initialize();
    assert_eq!(
        rt.handle_routed(event()).unwrap_err().into_routed_input(),
        event(),
        "unpresented frames retain the original routed input"
    );
    present(&mut rt);
    rt.set_focus(Some(TARGET));
    assert_eq!(
        rt.handle_routed(event()).unwrap_err().into_routed_input(),
        event(),
        "unsettled focus transitions retain the original routed input"
    );
    present(&mut rt);
    let _ = rt
        .handle_routed(event())
        .expect("retried event is dispatched after normal lifecycle");
    assert_eq!(
        rt.app().pointers,
        vec![PointerIntent {
            owner: TARGET,
            phase: Phase::Middle,
            part: ITEM,
            pos: Position::new(13, 6),
            local: Position::new(1, 0),
            mods: KeyModifiers::ALT,
        }]
    );
}

#[test]
fn old_and_routed_standard_input_entry_points_keep_their_contracts() {
    let mut old = ready_runtime(Probe::default());
    let mut routed = ready_runtime(Probe::default());
    let old_response = old
        .handle(Input::Tick)
        .expect("legacy entry point is ready");
    let routed_response = routed
        .handle_routed(RoutedInput::Input(Input::Tick))
        .expect("routed entry point is ready");
    assert_eq!(old_response, routed_response);
    assert_eq!(old.app().updates, routed.app().updates);

    let pending = Runtime::new(Probe::default(), Theme::junie())
        .handle(Input::Paste("pending legacy paste".to_owned()))
        .unwrap_err();
    assert_eq!(
        pending.into_input(),
        Input::Paste("pending legacy paste".to_owned())
    );
    let pending = Runtime::new(Probe::default(), Theme::junie())
        .handle_routed(RoutedInput::Input(Input::Paste(
            "pending routed paste".to_owned(),
        )))
        .unwrap_err();
    assert_eq!(
        pending.into_routed_input(),
        RoutedInput::Input(Input::Paste("pending routed paste".to_owned()))
    );
}
