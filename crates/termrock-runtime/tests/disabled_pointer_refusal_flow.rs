#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_core::event::{Axis, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_core::intent::{FocusVia, Intent, Phase};
use termrock_core::response::{Flow, Invalidate};
use termrock_core::{Id, Part, PartRef, Response};
use termrock_runtime::{
    Anchor, App, Axes, Cx, Focusability, Headroom, LayerSize, LayerSpec, Runtime, Ui,
};
use termrock_theme::{StateFlags, Theme};

const SCREEN: Rect = Rect::new(0, 0, 36, 14);
const TARGET: Id = Id::root("disabled-refusal.target");
const UNDERLAY: Id = Id::root("disabled-refusal.underlay");
const NEIGHBOR: Id = Id::root("disabled-refusal.neighbor");
const POPOVER: Id = Id::root("disabled-refusal.popover");
const RESPONSE_ID: Id = Id::root("disabled-refusal.response");
const TARGET_AREA: Rect = Rect::new(1, 1, 8, 2);
const RELOCATED_TARGET_AREA: Rect = Rect::new(10, 5, 8, 2);
const NEIGHBOR_AREA: Rect = Rect::new(20, 1, 8, 2);
const POPOVER_ANCHOR: Position = Position::new(27, 9);
const TARGET_PART: PartRef = PartRef::of(Part::THUMB);
const RESPONSE_STATE: StateFlags = StateFlags::FOCUSED;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointerIntent {
    owner: Id,
    phase: Phase,
    part: PartRef,
    pos: Position,
    local: Position,
    mods: KeyModifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OtherIntentKind {
    Binding,
    Key {
        code: KeyCode,
        mods: KeyModifiers,
    },
    Paste {
        byte_len: usize,
    },
    Wheel {
        axis: Axis,
        delta: i16,
        part: PartRef,
        pos: Position,
    },
    FocusIn {
        via: FocusVia,
    },
    FocusOut {
        to: Option<Id>,
    },
    Layer,
    Cancel,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OtherIntent {
    owner: Id,
    kind: OtherIntentKind,
}

#[derive(Clone, Copy, Debug)]
enum Registration {
    DisabledControl,
    DisabledPart,
    DisabledOverEnabledUnderlay,
    Enabled,
    ReadOnly,
    ClickOnly,
    StyleOnlyDisabled,
    DecorativeOverDisabled,
    FocusOnlyDisabled,
    Missing,
}

#[derive(Clone, Copy, Debug)]
enum Reply {
    Ignored,
    IgnoredWithPaint,
    IgnoredWithLayout,
    ConsumedWithIdentity,
    ChangedWithIdentity,
    Action,
}

impl Reply {
    fn response(self) -> Response<()> {
        match self {
            Self::Ignored => Response::ignored(),
            Self::IgnoredWithPaint => Response::ignored()
                .for_id(RESPONSE_ID)
                .with_state(RESPONSE_STATE)
                .repaint(),
            Self::IgnoredWithLayout => Response::ignored()
                .for_id(RESPONSE_ID)
                .with_state(RESPONSE_STATE)
                .relayout(),
            Self::ConsumedWithIdentity => Response::consumed()
                .for_id(RESPONSE_ID)
                .with_state(RESPONSE_STATE),
            Self::ChangedWithIdentity => Response::changed()
                .for_id(RESPONSE_ID)
                .with_state(RESPONSE_STATE),
            Self::Action => Response::action(())
                .for_id(RESPONSE_ID)
                .with_state(RESPONSE_STATE),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ResponseSnapshot {
    flow: Flow,
    id: Option<Id>,
    state: StateFlags,
    invalidate: Invalidate,
    action: bool,
}

impl ResponseSnapshot {
    fn capture(response: &Response<()>) -> Self {
        Self {
            flow: response.flow(),
            id: response.id(),
            state: response.state(),
            invalidate: response.invalidate(),
            action: response.action_ref().is_some(),
        }
    }
}

struct Probe {
    registration: Registration,
    target_area: Rect,
    scroll_target: bool,
    reply: Reply,
    updates: usize,
    returned_replies: Vec<ResponseSnapshot>,
    pointers: Vec<PointerIntent>,
    other_intents: Vec<OtherIntent>,
    capture_neighbor: bool,
    capture_succeeded: bool,
    open_popover: bool,
    popover_open: bool,
    popover_control: Option<Focusability>,
}

impl Probe {
    fn new(registration: Registration) -> Self {
        Self {
            registration,
            target_area: TARGET_AREA,
            scroll_target: false,
            reply: Reply::Ignored,
            updates: 0,
            returned_replies: Vec::new(),
            pointers: Vec::new(),
            other_intents: Vec::new(),
            capture_neighbor: false,
            capture_succeeded: false,
            open_popover: false,
            popover_open: false,
            popover_control: None,
        }
    }

    fn with_reply(mut self, reply: Reply) -> Self {
        self.reply = reply;
        self
    }

    fn with_scroll_target(mut self) -> Self {
        self.scroll_target = true;
        self
    }
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        let mut neighbor_pressed = false;
        for owner in [TARGET, UNDERLAY, NEIGHBOR, POPOVER] {
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
                        neighbor_pressed |= owner == NEIGHBOR && phase == Phase::Press;
                    }
                    Intent::Binding(_) => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Binding,
                    }),
                    Intent::Key(key) => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Key {
                            code: key.code,
                            mods: key.mods,
                        },
                    }),
                    Intent::Paste(text) => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Paste {
                            byte_len: text.len(),
                        },
                    }),
                    Intent::Wheel {
                        axis,
                        delta,
                        part,
                        pos,
                    } => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Wheel {
                            axis,
                            delta,
                            part,
                            pos,
                        },
                    }),
                    Intent::FocusIn { via } => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::FocusIn { via },
                    }),
                    Intent::FocusOut { to } => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::FocusOut { to },
                    }),
                    Intent::Layer(_) => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Layer,
                    }),
                    Intent::Cancel => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Cancel,
                    }),
                    _ => self.other_intents.push(OtherIntent {
                        owner,
                        kind: OtherIntentKind::Unknown,
                    }),
                }
            }
        }
        if self.capture_neighbor && neighbor_pressed {
            self.capture_succeeded = cx.capture(NEIGHBOR, TARGET_PART);
        }
        if self.open_popover {
            self.open_popover = false;
            self.popover_open = true;
            let mut spec = LayerSpec::popover(POPOVER, Anchor::Point(POPOVER_ANCHOR));
            spec.size = LayerSize::Fixed(7, 3);
            cx.open_layer(POPOVER, spec);
        }
        let response = self.reply.response();
        self.returned_replies
            .push(ResponseSnapshot::capture(&response));
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(NEIGHBOR, NEIGHBOR_AREA, Focusability::Focusable);
        ui.register_part(NEIGHBOR, TARGET_PART, NEIGHBOR_AREA);

        let target_area = self.target_area;
        if matches!(self.registration, Registration::DisabledOverEnabledUnderlay) {
            ui.register_control(UNDERLAY, target_area, Focusability::Focusable);
        }

        match self.registration {
            Registration::DisabledControl | Registration::DisabledOverEnabledUnderlay => {
                ui.register_control(TARGET, target_area, Focusability::Disabled);
            }
            Registration::DisabledPart => {
                ui.register_control(TARGET, target_area, Focusability::Disabled);
                ui.register_part(TARGET, TARGET_PART, target_area);
            }
            Registration::Enabled => {
                ui.register_control(TARGET, target_area, Focusability::Focusable);
            }
            Registration::ReadOnly => {
                ui.register_control(TARGET, target_area, Focusability::FocusableReadOnly);
            }
            Registration::ClickOnly => {
                ui.register_control(TARGET, target_area, Focusability::ClickOnly);
            }
            Registration::StyleOnlyDisabled => {
                ui.register_control(TARGET, target_area, Focusability::Focusable);
                ui.declare_state(TARGET, StateFlags::DISABLED);
            }
            Registration::DecorativeOverDisabled => {
                ui.register_control(TARGET, target_area, Focusability::Disabled);
                ui.register_decor(TARGET, TARGET_PART, target_area);
            }
            Registration::FocusOnlyDisabled => {
                ui.register_focus_only(TARGET, Focusability::Disabled);
            }
            Registration::Missing => {}
        }

        if self.scroll_target {
            ui.register_scroll(TARGET, target_area, Axes::V, Headroom::default());
        }

        if self.popover_open {
            let _ = ui.layer(POPOVER, |ui, area| {
                if let Some(focusability) = self.popover_control {
                    ui.register_control(POPOVER, area, focusability);
                }
            });
        }
    }
}

fn present(runtime: &mut Runtime<Probe>) {
    let mut buffer = Buffer::empty(SCREEN);
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!("probe runtime did not settle and present");
}

fn ready(probe: Probe) -> Runtime<Probe> {
    let mut runtime = Runtime::new(probe, Theme::junie());
    let _ = runtime.initialize();
    present(&mut runtime);
    runtime
}

fn assert_ready_neighbor_focus(runtime: &Runtime<Probe>) -> Vec<OtherIntent> {
    assert_eq!(runtime.focus(), Some(NEIGHBOR));
    assert!(runtime.app().pointers.is_empty());
    let setup = runtime.app().other_intents.clone();
    assert_eq!(
        setup,
        vec![OtherIntent {
            owner: NEIGHBOR,
            kind: OtherIntentKind::FocusIn {
                via: FocusVia::Programmatic,
            },
        }]
    );
    setup
}

fn deliver(runtime: &mut Runtime<Probe>, input: Input) -> Response<()> {
    present(runtime);
    runtime
        .handle(input)
        .expect("the presented runtime accepts input")
}

struct DownObservation {
    setup_focus: Option<Id>,
    setup_other_intents: Vec<OtherIntent>,
    app_returns: Vec<ResponseSnapshot>,
    event_updates: usize,
    pointer_intents: Vec<PointerIntent>,
    other_intents: Vec<OtherIntent>,
    public_response: Response<()>,
}

fn observe_down(registration: Registration, reply: Reply) -> DownObservation {
    let mut runtime = ready(Probe::new(registration).with_reply(reply));
    let setup_focus = runtime.focus();
    let setup_other_intents = assert_ready_neighbor_focus(&runtime);
    let prior_reply_count = runtime.app().returned_replies.len();
    let prior_update_count = runtime.app().updates;
    let prior_pointer_count = runtime.app().pointers.len();
    let prior_other_intent_count = runtime.app().other_intents.len();
    let public_response = deliver(
        &mut runtime,
        mouse(MouseKind::Down, TARGET_AREA.x, TARGET_AREA.y),
    );
    DownObservation {
        setup_focus,
        setup_other_intents,
        app_returns: runtime.app().returned_replies[prior_reply_count..].to_vec(),
        event_updates: runtime.app().updates.saturating_sub(prior_update_count),
        pointer_intents: runtime.app().pointers[prior_pointer_count..].to_vec(),
        other_intents: runtime.app().other_intents[prior_other_intent_count..].to_vec(),
        public_response,
    }
}

fn mouse(kind: MouseKind, x: u16, y: u16) -> Input {
    Input::Mouse(Mouse {
        kind,
        pos: Position::new(x, y),
        mods: KeyModifiers::NONE,
    })
}

#[test]
fn disabled_control_and_part_down_are_consumed_without_a_press_or_stale_flow() {
    for registration in [Registration::DisabledControl, Registration::DisabledPart] {
        let mut runtime = ready(Probe::new(registration));
        let hit = runtime
            .registry()
            .hit(Position::new(TARGET_AREA.x, TARGET_AREA.y))
            .expect("the disabled target is a real presented hit");
        assert_eq!(hit.owner, TARGET);
        assert_eq!(
            hit.kind,
            match registration {
                Registration::DisabledPart => termrock_runtime::RegionKind::Part,
                _ => termrock_runtime::RegionKind::Control,
            }
        );
        assert!(
            runtime
                .ring()
                .entry(TARGET)
                .is_some_and(|entry| entry.disabled)
        );
        let focus_before = runtime.focus();
        assert_eq!(focus_before, Some(NEIGHBOR));
        let setup_other_intents = assert_ready_neighbor_focus(&runtime);

        let before = runtime.app().updates;
        let other_intents_before = runtime.app().other_intents.len();
        let down = deliver(
            &mut runtime,
            mouse(MouseKind::Down, TARGET_AREA.x, TARGET_AREA.y),
        );
        assert!(down.is_consumed(), "disabled primary Down: {down:?}");
        assert_eq!(down.flow(), Flow::Consumed);
        assert_eq!(down.invalidate(), Invalidate::None);
        assert_eq!(runtime.app().updates, before + 1);
        assert!(runtime.app().pointers.is_empty());
        assert!(runtime.app().other_intents[other_intents_before..].is_empty());
        assert_eq!(runtime.focus(), focus_before);
        assert_eq!(runtime.capture_owner(), None);
        assert!(runtime.diagnostics().is_empty());

        let up = deliver(
            &mut runtime,
            mouse(MouseKind::Up, TARGET_AREA.x, TARGET_AREA.y),
        );
        assert!(!up.is_consumed(), "no stored press means Up stays ignored");
        assert!(
            runtime.app().pointers.is_empty(),
            "no synthetic release/click"
        );
        assert_eq!(runtime.focus(), focus_before);

        let empty_down = deliver(&mut runtime, mouse(MouseKind::Down, 34, 12));
        assert!(
            !empty_down.is_consumed(),
            "refusal belongs to one Down only"
        );
        assert_eq!(runtime.focus(), focus_before);
        let other_intents_before_key = runtime.app().other_intents.len();
        let key = deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('x'),
                mods: KeyModifiers::NONE,
            }),
        );
        assert!(!key.is_consumed(), "a later key is not forced consumed");
        assert!(
            runtime.app().other_intents[other_intents_before_key..]
                .iter()
                .all(|intent| intent.owner != TARGET)
        );
        assert_eq!(runtime.focus(), focus_before);
        let tick = deliver(&mut runtime, Input::Tick);
        assert!(!tick.is_consumed(), "a later Tick is not forced consumed");
        assert!(runtime.app().pointers.is_empty());
        assert_eq!(
            &runtime.app().other_intents[..setup_other_intents.len()],
            setup_other_intents.as_slice()
        );
        assert_eq!(runtime.focus(), focus_before);
        assert_eq!(runtime.capture_owner(), None);
        assert!(runtime.diagnostics().is_empty());
    }
}

#[test]
fn disabled_hit_blocks_an_enabled_overlap_but_does_not_block_a_neighbor() {
    let mut runtime = ready(Probe::new(Registration::DisabledOverEnabledUnderlay));
    let hit = runtime
        .registry()
        .hit(Position::new(TARGET_AREA.x, TARGET_AREA.y))
        .expect("the overlap has a winning hit");
    assert_eq!(hit.owner, TARGET, "the top registration owns the hit");

    let response = deliver(
        &mut runtime,
        mouse(MouseKind::Down, TARGET_AREA.x, TARGET_AREA.y),
    );
    assert!(response.is_consumed());
    assert!(
        runtime.app().pointers.is_empty(),
        "do not retry the underlay"
    );

    let up = deliver(
        &mut runtime,
        mouse(MouseKind::Up, TARGET_AREA.x, TARGET_AREA.y),
    );
    assert!(!up.is_consumed());
    assert!(runtime.app().pointers.is_empty());

    let _ = deliver(
        &mut runtime,
        mouse(MouseKind::Down, NEIGHBOR_AREA.x, NEIGHBOR_AREA.y),
    );
    let _ = deliver(
        &mut runtime,
        mouse(MouseKind::Up, NEIGHBOR_AREA.x, NEIGHBOR_AREA.y),
    );
    assert!(
        runtime
            .app()
            .pointers
            .iter()
            .any(|intent| intent.owner == NEIGHBOR && intent.phase == Phase::Press)
    );
    assert!(
        runtime
            .app()
            .pointers
            .iter()
            .any(|intent| intent.owner == NEIGHBOR && intent.phase == Phase::Release)
    );
    assert!(
        runtime
            .app()
            .pointers
            .iter()
            .any(|intent| intent.owner == NEIGHBOR && intent.phase == Phase::Click)
    );
    assert!(
        !runtime
            .app()
            .pointers
            .iter()
            .any(|intent| intent.owner == UNDERLAY)
    );
}

#[test]
fn only_presented_disabled_control_or_part_metadata_triggers_refusal() {
    for registration in [
        Registration::Enabled,
        Registration::ReadOnly,
        Registration::ClickOnly,
        Registration::StyleOnlyDisabled,
        Registration::DecorativeOverDisabled,
        Registration::FocusOnlyDisabled,
        Registration::Missing,
    ] {
        let mut runtime = ready(Probe::new(registration));
        let hit = runtime
            .registry()
            .hit(Position::new(TARGET_AREA.x, TARGET_AREA.y));
        let response = deliver(
            &mut runtime,
            mouse(MouseKind::Down, TARGET_AREA.x, TARGET_AREA.y),
        );
        assert!(
            !response.is_consumed(),
            "{registration:?} was not a disabled hit"
        );
        match registration {
            Registration::Enabled
            | Registration::ReadOnly
            | Registration::ClickOnly
            | Registration::StyleOnlyDisabled => {
                assert_eq!(hit.expect("interactive target is registered").owner, TARGET);
                assert!(
                    runtime
                        .app()
                        .pointers
                        .iter()
                        .any(|intent| intent.owner == TARGET && intent.phase == Phase::Press)
                );
            }
            Registration::DecorativeOverDisabled => {
                assert_eq!(
                    hit.expect("decorative region is registered").kind,
                    termrock_runtime::RegionKind::Decorative
                );
                assert!(runtime.app().pointers.is_empty());
            }
            Registration::FocusOnlyDisabled | Registration::Missing => {
                assert!(hit.is_none());
                assert!(runtime.app().pointers.is_empty());
            }
            Registration::DisabledControl
            | Registration::DisabledPart
            | Registration::DisabledOverEnabledUnderlay => unreachable!(),
        }
    }
}

#[test]
fn disabled_down_changes_only_ignored_response_flow_after_the_existing_fold() {
    struct Case {
        name: &'static str,
        reply: Reply,
        app_reply: ResponseSnapshot,
        baseline: ResponseSnapshot,
        disabled_ignored: Option<ResponseSnapshot>,
    }

    let cases = [
        Case {
            name: "ignored without an id or state",
            reply: Reply::Ignored,
            app_reply: ResponseSnapshot {
                flow: Flow::Ignored,
                id: None,
                state: StateFlags::empty(),
                invalidate: Invalidate::None,
                action: false,
            },
            baseline: ResponseSnapshot {
                flow: Flow::Ignored,
                id: None,
                state: StateFlags::empty(),
                invalidate: Invalidate::None,
                action: false,
            },
            disabled_ignored: Some(ResponseSnapshot {
                flow: Flow::Consumed,
                id: None,
                state: StateFlags::empty(),
                invalidate: Invalidate::None,
                action: false,
            }),
        },
        Case {
            name: "ignored with app state and paint",
            reply: Reply::IgnoredWithPaint,
            app_reply: ResponseSnapshot {
                flow: Flow::Ignored,
                id: Some(RESPONSE_ID),
                state: RESPONSE_STATE,
                invalidate: Invalidate::Paint,
                action: false,
            },
            baseline: ResponseSnapshot {
                flow: Flow::Ignored,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::Paint,
                action: false,
            },
            disabled_ignored: Some(ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::Paint,
                action: false,
            }),
        },
        Case {
            name: "ignored with app state and layout",
            reply: Reply::IgnoredWithLayout,
            app_reply: ResponseSnapshot {
                flow: Flow::Ignored,
                id: Some(RESPONSE_ID),
                state: RESPONSE_STATE,
                invalidate: Invalidate::Layout,
                action: false,
            },
            baseline: ResponseSnapshot {
                flow: Flow::Ignored,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::Layout,
                action: false,
            },
            disabled_ignored: Some(ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::Layout,
                action: false,
            }),
        },
        Case {
            name: "already consumed with app state",
            reply: Reply::ConsumedWithIdentity,
            app_reply: ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: RESPONSE_STATE,
                invalidate: Invalidate::None,
                action: false,
            },
            baseline: ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::None,
                action: false,
            },
            disabled_ignored: None,
        },
        Case {
            name: "changed app response",
            reply: Reply::ChangedWithIdentity,
            app_reply: ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: RESPONSE_STATE,
                invalidate: Invalidate::Paint,
                action: false,
            },
            baseline: ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::Paint,
                action: false,
            },
            disabled_ignored: None,
        },
        Case {
            name: "action app response",
            reply: Reply::Action,
            app_reply: ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: RESPONSE_STATE,
                invalidate: Invalidate::Paint,
                action: true,
            },
            baseline: ResponseSnapshot {
                flow: Flow::Consumed,
                id: Some(RESPONSE_ID),
                state: StateFlags::empty(),
                invalidate: Invalidate::Paint,
                action: false,
            },
            disabled_ignored: None,
        },
    ];

    for case in cases {
        // Missing leaves the tested point unhit; otherwise both fresh Runtime
        // instances see the same event and the same App-returned reply.
        let baseline = observe_down(Registration::Missing, case.reply);
        let disabled = observe_down(Registration::DisabledControl, case.reply);

        let expected_setup = vec![OtherIntent {
            owner: NEIGHBOR,
            kind: OtherIntentKind::FocusIn {
                via: FocusVia::Programmatic,
            },
        }];
        for observation in [&baseline, &disabled] {
            assert_eq!(observation.setup_focus, Some(NEIGHBOR), "{}", case.name);
            assert_eq!(
                observation.setup_other_intents, expected_setup,
                "{}",
                case.name
            );
            assert_eq!(observation.event_updates, 1, "{}", case.name);
            assert_eq!(
                observation.app_returns.as_slice(),
                &[case.app_reply],
                "{}",
                case.name
            );
            assert!(observation.pointer_intents.is_empty(), "{}", case.name);
            assert!(observation.other_intents.is_empty(), "{}", case.name);
        }

        assert_eq!(
            ResponseSnapshot::capture(&baseline.public_response),
            case.baseline,
            "literal existing-fold output for {}",
            case.name
        );

        match case.disabled_ignored {
            Some(expected_disabled) => assert_eq!(
                ResponseSnapshot::capture(&disabled.public_response),
                expected_disabled,
                "disabled refusal changes only Ignored flow for {}",
                case.name
            ),
            None => assert_eq!(
                disabled.public_response, baseline.public_response,
                "already-consumed Runtime responses remain fully equal for {}",
                case.name
            ),
        }
    }
}

#[test]
fn a_disabled_presented_popover_control_consumes_but_a_lower_disabled_hit_dismisses_normally() {
    let mut top_disabled = Probe::new(Registration::Enabled);
    top_disabled.open_popover = true;
    top_disabled.popover_control = Some(Focusability::Disabled);
    let mut runtime = ready(top_disabled);
    assert_ne!(runtime.top_layer(), termrock_runtime::LayerId::PAGE);
    let area = runtime
        .area_of(POPOVER)
        .expect("the disabled popover control was presented");
    let hit = runtime
        .registry()
        .hit(Position::new(area.x, area.y))
        .expect("top-layer control has a real hit");
    assert_eq!(hit.owner, POPOVER);
    assert_eq!(hit.layer, runtime.top_layer());
    let response = deliver(&mut runtime, mouse(MouseKind::Down, area.x, area.y));
    assert!(response.is_consumed());
    assert!(runtime.app().pointers.is_empty());

    let mut lower_disabled = Probe::new(Registration::DisabledControl);
    lower_disabled.open_popover = true;
    lower_disabled.popover_control = Some(Focusability::Focusable);
    let mut runtime = ready(lower_disabled);
    let hit = runtime
        .registry()
        .hit(Position::new(TARGET_AREA.x, TARGET_AREA.y))
        .expect("the underlying disabled control remains the presented lower hit");
    assert_eq!(hit.owner, TARGET);
    assert_eq!(hit.layer, termrock_runtime::LayerId::PAGE);
    assert_ne!(runtime.top_layer(), hit.layer);
    let response = deliver(
        &mut runtime,
        mouse(MouseKind::Down, TARGET_AREA.x, TARGET_AREA.y),
    );
    assert!(
        !response.is_consumed(),
        "lower-layer metadata is not refusal authority"
    );
    assert!(
        !runtime.is_open(POPOVER),
        "outside routing still dismisses the popover"
    );
    assert!(runtime.app().pointers.is_empty());
}

#[test]
fn active_pointer_capture_keeps_ownership_over_a_disabled_underlying_hit() {
    let mut probe = Probe::new(Registration::DisabledControl);
    probe.capture_neighbor = true;
    let mut runtime = ready(probe);
    let setup_other_intents = assert_ready_neighbor_focus(&runtime);
    let capture_area = runtime
        .area_of(NEIGHBOR)
        .expect("capturing control was presented");
    let setup_response = deliver(
        &mut runtime,
        mouse(MouseKind::Down, NEIGHBOR_AREA.x, NEIGHBOR_AREA.y),
    );
    assert!(!setup_response.is_consumed());
    assert!(runtime.app().capture_succeeded);
    assert_eq!(runtime.capture_owner(), Some(NEIGHBOR));
    assert_eq!(runtime.focus(), Some(NEIGHBOR));
    assert_eq!(runtime.app().other_intents, setup_other_intents);
    assert_eq!(
        runtime.app().pointers,
        vec![PointerIntent {
            owner: NEIGHBOR,
            phase: Phase::Press,
            part: TARGET_PART,
            pos: Position::new(NEIGHBOR_AREA.x, NEIGHBOR_AREA.y),
            local: Position::new(
                NEIGHBOR_AREA.x.saturating_sub(capture_area.x),
                NEIGHBOR_AREA.y.saturating_sub(capture_area.y),
            ),
            mods: KeyModifiers::NONE,
        }]
    );
    present(&mut runtime);

    let pointer_count_before = runtime.app().pointers.len();
    let other_intent_count_before = runtime.app().other_intents.len();
    let focus_before = runtime.focus();
    let captured_down = Position::new(TARGET_AREA.x, TARGET_AREA.y);
    let response = deliver(
        &mut runtime,
        mouse(MouseKind::Down, captured_down.x, captured_down.y),
    );
    assert!(
        !response.is_consumed(),
        "captured routing is not reclassified by the underlay"
    );
    assert_eq!(response.flow(), Flow::Ignored);
    assert!(response.action_ref().is_none());
    assert_eq!(runtime.capture_owner(), Some(NEIGHBOR));
    assert_eq!(
        runtime.focus(),
        focus_before,
        "captured press must not transfer focus"
    );
    assert!(runtime.app().other_intents[other_intent_count_before..].is_empty());
    assert_eq!(
        &runtime.app().pointers[pointer_count_before..],
        &[PointerIntent {
            owner: NEIGHBOR,
            phase: Phase::Press,
            part: TARGET_PART,
            pos: captured_down,
            local: Position::new(
                captured_down.x.saturating_sub(capture_area.x),
                captured_down.y.saturating_sub(capture_area.y),
            ),
            mods: KeyModifiers::NONE,
        }],
        "captured Down is delivered exactly once to its original owner and part"
    );
}

#[test]
fn presented_disabled_registration_lifecycle_tracks_enablement_removal_and_moved_geometry() {
    let old_point = Position::new(TARGET_AREA.x, TARGET_AREA.y);
    let moved_point = Position::new(RELOCATED_TARGET_AREA.x, RELOCATED_TARGET_AREA.y);
    let mut runtime = ready(Probe::new(Registration::DisabledControl));
    let initial_focus = Some(NEIGHBOR);

    let disabled_hit = runtime
        .registry()
        .hit(old_point)
        .expect("the initial disabled control is presented at its old area");
    assert_eq!(disabled_hit.owner, TARGET);
    assert_eq!(disabled_hit.kind, termrock_runtime::RegionKind::Control);
    assert_eq!(runtime.area_of(TARGET), Some(TARGET_AREA));
    assert!(
        runtime
            .ring()
            .entry(TARGET)
            .is_some_and(|entry| entry.disabled)
    );
    assert_eq!(runtime.focus(), initial_focus);

    let disabled_intent_start = runtime.app().other_intents.len();
    let first_disabled_down = deliver(
        &mut runtime,
        mouse(MouseKind::Down, old_point.x, old_point.y),
    );
    assert_eq!(first_disabled_down.flow(), Flow::Consumed);
    assert!(runtime.app().pointers.is_empty());
    assert!(runtime.app().other_intents[disabled_intent_start..].is_empty());
    let first_disabled_up = deliver(&mut runtime, mouse(MouseKind::Up, old_point.x, old_point.y));
    assert_eq!(first_disabled_up.flow(), Flow::Ignored);
    assert!(runtime.app().pointers.is_empty());
    assert_eq!(runtime.focus(), initial_focus);
    assert_eq!(runtime.capture_owner(), None);

    // Publish a fresh enabled registration before testing its real click path.
    runtime.app_mut().registration = Registration::Enabled;
    present(&mut runtime);
    let enabled_hit = runtime
        .registry()
        .hit(old_point)
        .expect("the enabled target is in the newly presented registry");
    assert_eq!(enabled_hit.owner, TARGET);
    assert_eq!(enabled_hit.kind, termrock_runtime::RegionKind::Control);
    assert_eq!(runtime.area_of(TARGET), Some(TARGET_AREA));
    assert!(
        !runtime
            .ring()
            .entry(TARGET)
            .is_some_and(|entry| entry.disabled)
    );

    let pointer_start = runtime.app().pointers.len();
    let focus_intent_start = runtime.app().other_intents.len();
    let enabled_down = deliver(
        &mut runtime,
        mouse(MouseKind::Down, old_point.x, old_point.y),
    );
    assert_eq!(enabled_down.flow(), Flow::Ignored);
    assert!(
        !enabled_down.is_consumed(),
        "the enabled Down keeps the Probe's ignored flow"
    );
    let enabled_up = deliver(&mut runtime, mouse(MouseKind::Up, old_point.x, old_point.y));
    assert_eq!(enabled_up.flow(), Flow::Ignored);
    assert!(!enabled_up.is_consumed());
    assert_eq!(
        &runtime.app().pointers[pointer_start..],
        &[
            PointerIntent {
                owner: TARGET,
                phase: Phase::Press,
                part: PartRef::of(Part::CONTAINER),
                pos: old_point,
                local: Position::new(0, 0),
                mods: KeyModifiers::NONE,
            },
            PointerIntent {
                owner: TARGET,
                phase: Phase::Release,
                part: PartRef::of(Part::CONTAINER),
                pos: old_point,
                local: Position::new(0, 0),
                mods: KeyModifiers::NONE,
            },
            PointerIntent {
                owner: TARGET,
                phase: Phase::Click,
                part: PartRef::of(Part::CONTAINER),
                pos: old_point,
                local: Position::new(0, 0),
                mods: KeyModifiers::NONE,
            },
        ],
        "a freshly presented enabled control receives its actual Press/Release/Click"
    );
    let focus_changes = &runtime.app().other_intents[focus_intent_start..];
    assert!(focus_changes.contains(&OtherIntent {
        owner: NEIGHBOR,
        kind: OtherIntentKind::FocusOut { to: Some(TARGET) },
    }));
    assert!(focus_changes.contains(&OtherIntent {
        owner: TARGET,
        kind: OtherIntentKind::FocusIn {
            via: FocusVia::Pointer,
        },
    }));
    assert_eq!(runtime.focus(), Some(TARGET));
    assert_eq!(runtime.capture_owner(), None);

    // Removing the registration and presenting again reconciles focus to the
    // remaining real control. Input at the old point cannot target stale data.
    runtime.app_mut().registration = Registration::Missing;
    present(&mut runtime);
    assert_eq!(runtime.area_of(TARGET), None);
    assert!(runtime.registry().hit(old_point).is_none());
    assert_eq!(runtime.focus(), initial_focus);
    let missing_pointer_start = runtime.app().pointers.len();
    let missing_intent_start = runtime.app().other_intents.len();
    let missing_down = deliver(
        &mut runtime,
        mouse(MouseKind::Down, old_point.x, old_point.y),
    );
    let missing_up = deliver(&mut runtime, mouse(MouseKind::Up, old_point.x, old_point.y));
    assert_eq!(missing_down.flow(), Flow::Ignored);
    assert_eq!(missing_up.flow(), Flow::Ignored);
    assert!(
        runtime.app().pointers[missing_pointer_start..]
            .iter()
            .all(|intent| intent.owner != TARGET)
    );
    assert!(runtime.app().other_intents[missing_intent_start..].is_empty());
    assert_eq!(runtime.focus(), initial_focus);
    assert_eq!(runtime.capture_owner(), None);

    // Move the same disabled target. A normal new presentation removes the old
    // hit and makes only the new rectangle eligible for refusal.
    {
        let probe = runtime.app_mut();
        probe.registration = Registration::DisabledControl;
        probe.target_area = RELOCATED_TARGET_AREA;
    }
    present(&mut runtime);
    let moved_hit = runtime
        .registry()
        .hit(moved_point)
        .expect("the relocated disabled control is presented at its new area");
    assert_eq!(moved_hit.owner, TARGET);
    assert_eq!(moved_hit.kind, termrock_runtime::RegionKind::Control);
    assert_eq!(runtime.area_of(TARGET), Some(RELOCATED_TARGET_AREA));
    assert!(
        runtime
            .ring()
            .entry(TARGET)
            .is_some_and(|entry| entry.disabled)
    );
    assert!(runtime.registry().hit(old_point).is_none());

    let moved_pointer_start = runtime.app().pointers.len();
    let moved_old_down = deliver(
        &mut runtime,
        mouse(MouseKind::Down, old_point.x, old_point.y),
    );
    let moved_old_up = deliver(&mut runtime, mouse(MouseKind::Up, old_point.x, old_point.y));
    assert_eq!(moved_old_down.flow(), Flow::Ignored);
    assert_eq!(moved_old_up.flow(), Flow::Ignored);
    assert!(
        runtime.app().pointers[moved_pointer_start..]
            .iter()
            .all(|intent| intent.owner != TARGET)
    );

    let moved_down = deliver(
        &mut runtime,
        mouse(MouseKind::Down, moved_point.x, moved_point.y),
    );
    assert_eq!(moved_down.flow(), Flow::Consumed);
    let moved_up = deliver(
        &mut runtime,
        mouse(MouseKind::Up, moved_point.x, moved_point.y),
    );
    assert_eq!(moved_up.flow(), Flow::Ignored);
    assert!(
        runtime.app().pointers[moved_pointer_start..]
            .iter()
            .all(|intent| intent.owner != TARGET)
    );
    assert_eq!(runtime.focus(), initial_focus);
    assert_eq!(runtime.capture_owner(), None);
    assert!(runtime.diagnostics().is_empty());
}

#[test]
fn disabled_registration_keeps_secondary_excluded_and_wheel_routing_available() {
    let mut runtime = ready(Probe::new(Registration::DisabledControl).with_scroll_target());
    let point = Position::new(TARGET_AREA.x, TARGET_AREA.y);
    let focus_before = Some(NEIGHBOR);

    let control_hit = runtime
        .registry()
        .hit(point)
        .expect("the disabled control has a real presented pointer hit");
    assert_eq!(control_hit.owner, TARGET);
    assert_eq!(control_hit.kind, termrock_runtime::RegionKind::Control);
    assert!(
        runtime
            .ring()
            .entry(TARGET)
            .is_some_and(|entry| entry.disabled)
    );
    let scroll_hit = runtime
        .registry()
        .hit_scroll(point, Axis::V)
        .expect("the same presented allocation has a real vertical wheel route");
    assert_eq!(scroll_hit.owner, TARGET);
    assert_eq!(scroll_hit.part, PartRef::of(Part::CONTAINER));
    assert_eq!(scroll_hit.kind, termrock_runtime::RegionKind::Scroll);
    assert_eq!(runtime.focus(), focus_before);

    let pointer_start = runtime.app().pointers.len();
    let other_start = runtime.app().other_intents.len();
    let secondary = deliver(&mut runtime, mouse(MouseKind::Secondary, point.x, point.y));
    let secondary_up = deliver(
        &mut runtime,
        mouse(MouseKind::SecondaryUp, point.x, point.y),
    );
    assert_eq!(secondary.flow(), Flow::Ignored);
    assert_eq!(secondary_up.flow(), Flow::Ignored);
    assert!(runtime.app().pointers[pointer_start..].is_empty());
    assert!(runtime.app().other_intents[other_start..].is_empty());
    assert_eq!(runtime.focus(), focus_before);
    assert_eq!(runtime.capture_owner(), None);

    assert_eq!(Theme::junie().design.motion.wheel_rows, 3);
    let wheel_intent_start = runtime.app().other_intents.len();
    let wheel = deliver(
        &mut runtime,
        mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
    );
    assert_eq!(wheel.flow(), Flow::Ignored);
    assert!(wheel.action_ref().is_none());
    assert_eq!(
        &runtime.app().other_intents[wheel_intent_start..],
        &[OtherIntent {
            owner: TARGET,
            kind: OtherIntentKind::Wheel {
                axis: Axis::V,
                delta: 3,
                part: PartRef::of(Part::CONTAINER),
                pos: point,
            },
        }],
        "the real disabled-control wheel route preserves the Probe's typed delta and owner"
    );
    assert!(runtime.app().pointers[pointer_start..].is_empty());
    assert_eq!(runtime.focus(), focus_before);
    assert_eq!(runtime.capture_owner(), None);

    let refused_down = deliver(&mut runtime, mouse(MouseKind::Down, point.x, point.y));
    assert_eq!(refused_down.flow(), Flow::Consumed);
    let unpaired_up = deliver(&mut runtime, mouse(MouseKind::Up, point.x, point.y));
    assert_eq!(unpaired_up.flow(), Flow::Ignored);
    assert!(runtime.app().pointers[pointer_start..].is_empty());
    assert_eq!(runtime.focus(), focus_before);
    assert_eq!(runtime.capture_owner(), None);
    assert!(runtime.diagnostics().is_empty());
}
