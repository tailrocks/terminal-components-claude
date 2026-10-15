//! Routing and hint integration: keymap ownership changes must suppress
//! the matching hint row, and typed default actions must route to commands.
//! Lives beside `HintBar` (the paint side) against the public runtime API.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};

use termrock_core::keys::KeyPhase;
use termrock_core::layer_types::LayerId;
use termrock_runtime::stub::{Control, SCREEN, Stub, deliver, key, mouse, runtime, step};
use termrock_runtime::ui::FrameRead;
use termrock_runtime::{
    ActionKey, App, Cx, Diagnostic, Focusability, Id, Input, Invalidate, KeyCode, KeyMap,
    LayerSpec, MouseKind, Response, Runtime, StateFlags, Theme, Ui,
};

const A: Id = Id::root("a");
const B: Id = Id::root("b");
const C: Id = Id::root("c");
const ROUTE_PUBLISH: u8 = 1 << 0;
const ROUTE_ALTERNATE: u8 = 1 << 1;
const ROUTE_SECOND: u8 = 1 << 2;
const ROUTE_CONSUME_RAW: u8 = 1 << 3;
const ROUTE_TAB: u8 = 1 << 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteCmd {
    Default,
    Latent,
    Alternate,
}

const DEFAULT_ACTION: ActionKey = ActionKey::custom("runtime.route.default");
const LATENT_ACTION: ActionKey = ActionKey::custom("runtime.route.latent");
const DEFAULT_BINDINGS: &[termrock_runtime::Binding<RouteCmd>] = &[
    termrock_runtime::Binding {
        action: DEFAULT_ACTION,
        chord: Some(termrock_runtime::Chord::key(KeyCode::Enter)),
        cmd: RouteCmd::Default,
        label: "Default",
        priority: 80,
        visible: true,
    },
    termrock_runtime::Binding {
        action: LATENT_ACTION,
        chord: None,
        cmd: RouteCmd::Latent,
        label: "Latent",
        priority: 60,
        visible: true,
    },
];
const ALTERNATE_BINDINGS: &[termrock_runtime::Binding<RouteCmd>] = &[termrock_runtime::Binding {
    action: ActionKey::custom("runtime.route.alternate"),
    chord: Some(termrock_runtime::Chord::key(KeyCode::F(2))),
    cmd: RouteCmd::Alternate,
    label: "Alternate",
    priority: 80,
    visible: true,
}];
const TAB_ACTION: ActionKey = ActionKey::custom("runtime.route.tab");
const TAB_BINDINGS: &[termrock_runtime::Binding<RouteCmd>] = &[termrock_runtime::Binding {
    action: TAB_ACTION,
    chord: Some(termrock_runtime::Chord::key(KeyCode::Tab)),
    cmd: RouteCmd::Alternate,
    label: "Tab action",
    priority: 80,
    visible: true,
}];

#[derive(Debug)]
struct RouteApp {
    keymap: KeyMap,
    options: u8,
    typed: Vec<(Id, RouteCmd)>,
    raw: usize,
    raw_keys: Vec<(Id, KeyCode)>,
    app_commands: usize,
}

impl Default for RouteApp {
    fn default() -> Self {
        RouteApp {
            keymap: KeyMap::new(),
            options: ROUTE_PUBLISH,
            typed: Vec::new(),
            raw: 0,
            raw_keys: Vec::new(),
            app_commands: 0,
        }
    }
}

impl RouteApp {
    fn table(&self) -> &'static [termrock_runtime::Binding<RouteCmd>] {
        if self.options & ROUTE_TAB != 0 {
            TAB_BINDINGS
        } else if self.options & ROUTE_ALTERNATE != 0 {
            ALTERNATE_BINDINGS
        } else {
            DEFAULT_BINDINGS
        }
    }
}

impl App for RouteApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = if cx.command().is_some() {
            self.app_commands = self.app_commands.saturating_add(1);
            Response::consumed()
        } else {
            Response::ignored()
        };
        let table = self.table();
        for owner in [A, B] {
            if owner == B && self.options & ROUTE_SECOND == 0 {
                continue;
            }
            for intent in cx.intents(owner) {
                match intent {
                    termrock_runtime::Intent::Binding(action) => {
                        if let Some(command) = termrock_runtime::Binding::command(table, action) {
                            self.typed.push((owner, command));
                            response |= Response::consumed();
                        }
                    }
                    termrock_runtime::Intent::Key(key) => {
                        self.raw = self.raw.saturating_add(1);
                        self.raw_keys.push((owner, key.code));
                        if self.options & ROUTE_CONSUME_RAW != 0 {
                            response |= Response::consumed();
                        }
                    }
                    _ => {}
                }
            }
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let table = self.table();
        ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Focusable);
        if self.options & ROUTE_PUBLISH != 0 {
            ui.publish_bindings(A, ui.state(A), table);
        }
        if self.options & ROUTE_SECOND != 0 {
            ui.register_control(B, Rect::new(12, 0, 10, 1), Focusability::Focusable);
            if self.options & ROUTE_PUBLISH != 0 {
                ui.publish_bindings(B, ui.state(B), table);
            }
        }
        let _ = termrock_feedback::HintBar::derived(Id::root("runtime.hints"))
            .draw(ui, Rect::new(0, 1, 40, 1));
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn route_runtime(app: RouteApp) -> (Runtime<RouteApp>, Buffer) {
    let area = Rect::new(0, 0, 40, 3);
    let mut buffer = Buffer::empty(area);
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    runtime.draw_buffer(area, &mut buffer).commit_presented();
    runtime.draw_buffer(area, &mut buffer).commit_presented();
    (runtime, buffer)
}

fn hint_row(buffer: &Buffer) -> String {
    (0..40)
        .filter_map(|x| buffer.cell(Position::new(x, 1)))
        .map(ratatui_core::buffer::Cell::symbol)
        .collect()
}

#[test]
fn default_action_routes_to_typed_command() {
    let (mut runtime, _) = route_runtime(RouteApp::default());
    let response = deliver(&mut runtime, key(KeyCode::Enter));
    assert!(response.is_consumed());
    assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Default)]);
    assert_eq!(runtime.app().raw, 0);
}

#[test]
fn owner_remove_suppresses_handling_and_hint() {
    let mut app = RouteApp::default();
    app.keymap.remove_component(A, DEFAULT_ACTION);
    let (mut runtime, mut buffer) = route_runtime(app);
    assert!(!hint_row(&buffer).contains("Default"));
    let response = deliver(&mut runtime, key(KeyCode::Enter));
    runtime
        .draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer)
        .commit_presented();
    assert!(!response.is_consumed());
    assert!(runtime.app().typed.is_empty());
    assert_eq!(runtime.app().raw, 1);
}

#[test]
fn remap_old_is_raw_and_new_is_typed_and_shown() {
    let mut app = RouteApp::default();
    app.keymap.remap_component(
        A,
        DEFAULT_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(3)),
    );
    let (mut runtime, mut buffer) = route_runtime(app);
    assert!(hint_row(&buffer).contains("F3"));
    let old = deliver(&mut runtime, key(KeyCode::Enter));
    assert!(!old.is_consumed());
    assert_eq!(runtime.app().raw, 1);
    runtime
        .draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer)
        .commit_presented();
    let new = deliver(&mut runtime, key(KeyCode::F(3)));
    assert!(new.is_consumed());
    assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Default)]);
}

#[test]
fn latent_action_activates_when_component_bound() {
    let mut app = RouteApp::default();
    app.keymap = app.keymap.bind_component(
        A,
        LATENT_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(4)),
    );
    let (mut runtime, buffer) = route_runtime(app);
    assert!(hint_row(&buffer).contains("F4"));
    assert!(deliver(&mut runtime, key(KeyCode::F(4))).is_consumed());
    assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Latent)]);
}

#[test]
fn component_override_is_owner_scoped() {
    let mut app = RouteApp {
        options: ROUTE_PUBLISH | ROUTE_SECOND,
        ..RouteApp::default()
    };
    app.keymap.remap_component(
        A,
        DEFAULT_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(5)),
    );
    let (mut runtime, mut buffer) = route_runtime(app);
    let _ = deliver(&mut runtime, key(KeyCode::Tab));
    runtime
        .draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer)
        .commit_presented();
    assert_eq!(runtime.focus(), Some(B));
    assert!(deliver(&mut runtime, key(KeyCode::Enter)).is_consumed());
    assert_eq!(runtime.app().typed, vec![(B, RouteCmd::Default)]);
}

#[test]
fn capture_keymap_wins_over_component_binding() {
    let mut app = RouteApp::default();
    app.keymap.add(
        KeyPhase::Capture,
        termrock_runtime::Chord::key(KeyCode::Enter),
        ActionKey::SAVE,
    );
    let (mut runtime, _) = route_runtime(app);
    assert!(deliver(&mut runtime, key(KeyCode::Enter)).is_consumed());
    assert_eq!(runtime.app().app_commands, 1);
    assert!(runtime.app().typed.is_empty());
}

#[test]
fn bubble_runs_only_when_raw_key_is_ignored() {
    let mut app = RouteApp::default();
    app.keymap.add(
        KeyPhase::Bubble,
        termrock_runtime::Chord::key(KeyCode::F(6)),
        ActionKey::SAVE,
    );
    let (mut runtime, _) = route_runtime(app);
    assert!(deliver(&mut runtime, key(KeyCode::F(6))).is_consumed());
    assert_eq!(runtime.app().app_commands, 1);

    runtime.app_mut().options |= ROUTE_CONSUME_RAW;
    assert!(deliver(&mut runtime, key(KeyCode::F(6))).is_consumed());
    assert_eq!(runtime.app().app_commands, 1);
}

#[test]
fn bubble_followup_does_not_redeliver_raw_key() {
    let mut app = RouteApp::default();
    app.keymap.add(
        KeyPhase::Bubble,
        termrock_runtime::Chord::key(KeyCode::F(6)),
        ActionKey::SAVE,
    );
    let (mut runtime, _) = route_runtime(app);
    assert_eq!(runtime.focus(), Some(A));
    assert!(deliver(&mut runtime, key(KeyCode::F(6))).is_consumed());
    assert_eq!(runtime.app().app_commands, 1);
    assert_eq!(runtime.app().raw_keys, vec![(A, KeyCode::F(6))]);
}

#[test]
fn map_replacement_invalidates_focused_hints() {
    let (mut runtime, mut buffer) = route_runtime(RouteApp::default());
    runtime.app_mut().keymap.remap_component(
        A,
        DEFAULT_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(7)),
    );
    let _ = deliver(&mut runtime, Input::Tick);
    runtime
        .draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer)
        .commit_presented();
    assert!(hint_row(&buffer).contains("F7"));
}

#[test]
fn unpublished_legacy_control_receives_raw_key() {
    let app = RouteApp {
        options: 0,
        ..RouteApp::default()
    };
    let (mut runtime, _) = route_runtime(app);
    assert!(!deliver(&mut runtime, key(KeyCode::Enter)).is_consumed());
    assert_eq!(runtime.app().raw, 1);
    assert!(runtime.app().typed.is_empty());
}

#[test]
fn component_tab_binding_precedes_focus_traversal() {
    let app = RouteApp {
        options: ROUTE_PUBLISH | ROUTE_SECOND | ROUTE_TAB,
        ..RouteApp::default()
    };
    let (mut runtime, _) = route_runtime(app);
    assert!(deliver(&mut runtime, key(KeyCode::Tab)).is_consumed());
    assert_eq!(runtime.focus(), Some(A));
    assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Alternate)]);
}

#[test]
fn removed_tab_binding_falls_back_to_focus_traversal() {
    let mut app = RouteApp {
        options: ROUTE_PUBLISH | ROUTE_SECOND | ROUTE_TAB,
        ..RouteApp::default()
    };
    app.keymap.remove_component(A, TAB_ACTION);
    let (mut runtime, _) = route_runtime(app);
    let _ = deliver(&mut runtime, key(KeyCode::Tab));
    assert_eq!(runtime.focus(), Some(B));
    assert!(runtime.app().typed.is_empty());
}

fn three() -> Stub {
    Stub {
        page: vec![
            Control::new(A, Rect::new(0, 0, 10, 1)),
            Control::new(B, Rect::new(0, 2, 10, 1)),
            Control::new(C, Rect::new(0, 4, 10, 1)),
        ],
        consume_keys: true,
        ..Stub::default()
    }
}

#[test]
fn derived_cache_is_fresh_when_component_reappears_after_a_frame_gap() {
    #[derive(Default)]
    struct GapCache(u32);
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let area = Rect::new(0, 0, 4, 1);
    let mut buffer = Buffer::empty(area);
    let component = Id::root("cache-gap");

    runtime
        .draw_scene(area, &mut buffer, |ui, _| {
            ui.cache::<GapCache>(component).0 = 7;
        })
        .commit_presented();
    runtime
        .draw_scene(area, &mut buffer, |_ui, _| {})
        .commit_presented();

    let observed = core::cell::Cell::new(u32::MAX);
    runtime
        .draw_scene(area, &mut buffer, |ui, _| {
            observed.set(ui.cache::<GapCache>(component).0);
        })
        .commit_presented();
    assert_eq!(observed.get(), 0);
}

#[test]
fn first_draw_focuses_the_first_control_and_tab_walks_the_ring() {
    let (mut rt, mut buf) = runtime(three());
    assert_eq!(rt.focus(), Some(A));
    let r = step(&mut rt, &mut buf, key(KeyCode::Tab));
    assert_eq!(rt.focus(), Some(B));
    assert!(rt.app().saw(A, "FocusOut") && rt.app().saw(B, "FocusIn { via: Keyboard }"));
    assert!(rt.focus_visible());
    let _ = r;
    let _ = step(&mut rt, &mut buf, key(KeyCode::BackTab));
    assert_eq!(rt.focus(), Some(A));
    let _ = step(&mut rt, &mut buf, key(KeyCode::BackTab));
    assert_eq!(rt.focus(), Some(C));
}

#[test]
fn focus_move_invalidates_paint() {
    let (mut rt, mut buf) = runtime(three());
    assert_eq!(rt.focus(), Some(A));
    rt.draw_buffer(SCREEN, &mut buf).commit_presented();

    let response = deliver(&mut rt, key(KeyCode::Tab));

    assert_eq!(rt.focus(), Some(B));
    assert_eq!(response.invalidate(), Invalidate::Paint);
}

#[test]
fn keys_go_to_the_focused_owner_and_are_consumed_or_bubbled() {
    let (mut rt, mut buf) = runtime(three());
    let r = step(&mut rt, &mut buf, key(KeyCode::Enter));
    assert!(r.is_consumed());
    assert_eq!(rt.app().count(A, "Key("), 1);
    assert_eq!(rt.app().count(B, "Key("), 0);
    rt.app_mut().consume_keys = false;
    let r = step(&mut rt, &mut buf, key(KeyCode::Esc));
    assert!(r.is_consumed(), "the on_esc ladder consumed it");
    assert_eq!(rt.app().esc_hits, 1);
}

#[test]
fn press_focuses_the_owner_and_click_is_delivered_with_local_coordinates() {
    let (mut rt, mut buf) = runtime(three());
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 2));
    assert_eq!(rt.focus(), Some(B));
    assert!(!rt.focus_visible());
    assert!(rt.state_of(B).contains(StateFlags::PRESSED));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 2));
    assert!(rt.app().saw(B, "phase: Press"));
    assert!(rt.app().saw(B, "phase: Click"));
    assert!(rt.app().saw(B, "local: Position { x: 3, y: 0 }"));
    // hover never focuses
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 4));
    assert_eq!(rt.hover(), Some(C));
    assert_eq!(rt.focus(), Some(B));
    assert!(rt.state_of(C).contains(StateFlags::HOVERED));
    // a key suppresses hover until the pointer moves
    let _ = step(&mut rt, &mut buf, key(KeyCode::Down));
    assert!(!rt.state_of(C).contains(StateFlags::HOVERED));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 4, 4));
    assert!(rt.state_of(C).contains(StateFlags::HOVERED));
}

#[test]
fn pointer_move_delivers_move_to_topmost_live_part() {
    let layer = Id::root("top");
    let decor = Id::root("decor");
    let mut s = Stub {
        page: vec![Control::new(A, Rect::new(0, 0, 10, 1))],
        ..Stub::default()
    };
    s.layers.push((
        layer,
        vec![
            Control::new(B, Rect::new(0, 0, 10, 1)),
            Control {
                decor: true,
                ..Control::new(decor, Rect::new(0, 0, 10, 1))
            },
        ],
    ));
    let (mut rt, mut buf) = runtime(s);
    rt.app_mut().open_request = Some((layer, LayerSpec::modal(layer)));
    let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
    rt.app_mut().log.clear();

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 0));

    assert_eq!(rt.app().count(B, "phase: Move"), 1);
    assert_eq!(rt.app().count(A, "phase: Move"), 0);
    assert_eq!(rt.app().count(decor, "Pointer"), 0);
    assert!(rt.app().saw(B, "part: PartRef { part: Part::LABEL"));
}

#[test]
fn pointer_move_over_decorative_delivers_nothing() {
    let decor = Id::root("decor");
    let s = Stub {
        page: vec![Control {
            decor: true,
            ..Control::new(decor, Rect::new(0, 0, 10, 1))
        }],
        ..Stub::default()
    };
    let (mut rt, mut buf) = runtime(s);

    let response = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 0));

    assert_eq!(rt.hover(), None);
    assert_eq!(response.invalidate(), Invalidate::None);
    assert_eq!(rt.app().count(decor, "Pointer"), 0);
}

#[test]
fn captured_move_is_drag_not_move() {
    let mut s = three();
    s.page[0].captures = true;
    let (mut rt, mut buf) = runtime(s);
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 0));
    assert_eq!(rt.capture_owner(), Some(A));
    rt.app_mut().log.clear();

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 2));

    assert_eq!(rt.app().count(A, "phase: Drag"), 1);
    assert_eq!(rt.app().count(A, "phase: Move"), 0);
    assert_eq!(rt.app().count(B, "Pointer"), 0);
}

#[test]
fn pointer_move_never_focuses() {
    let (mut rt, mut buf) = runtime(three());
    assert_eq!(rt.focus(), Some(A));

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 2));

    assert_eq!(rt.focus(), Some(A));
    assert_eq!(rt.hover(), Some(B));
    assert_eq!(rt.app().count(B, "phase: Move"), 1);
    assert_eq!(rt.app().count(B, "phase: Press"), 0);
    assert_eq!(rt.app().count(B, "phase: Click"), 0);
}

/// §73 AC-1. A control the last frame registered as
/// `Focusability::Disabled` is registered and never reachable — by `Tab`
/// through `focus_target_admissible`, and now by the pointer through
/// `deliverable`. No `Press`, no `Release`, no `Click`, no `Secondary`
/// reaches it, no press is recorded for it, and a refused press moves no
/// focus. Before §73 the two disagreed: keyboard refused the control
/// while a click still activated it.
#[test]
fn ac1_a_disabled_control_refuses_every_activating_pointer_intent() {
    let mut s = three();
    s.page[1].focus = Focusability::Disabled;
    let (mut rt, mut buf) = runtime(s);
    assert_eq!(rt.focus(), Some(A));

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 2));
    assert_eq!(rt.app().count(B, "phase: Press"), 0);
    assert!(!rt.state_of(B).contains(StateFlags::PRESSED));
    assert_eq!(rt.focus(), Some(A), "a refused press moved focus");

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 2));
    assert_eq!(rt.app().count(B, "phase: Release"), 0);
    assert_eq!(rt.app().count(B, "phase: Click"), 0);

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Secondary, 3, 2));
    assert_eq!(rt.app().count(B, "phase: Secondary"), 0);
    assert_eq!(rt.app().count(B, "Pointer"), 0);

    // the enabled neighbour is untouched by the refusal
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 4));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 4));
    assert_eq!(rt.focus(), Some(C));
    assert_eq!(rt.app().count(C, "phase: Click"), 1);
}

/// §73 AC-2. The refusal is scoped to activation and to the ring's own
/// proof. Hover and `Phase::Move` come from `hit_live`, so a disabled
/// control still hovers; `Intent::Wheel` comes from `hit_scroll`, so its
/// scroll region still scrolls; a `Focusability::ClickOnly` owner takes
/// no ring entry at all, so it is never proven disabled and still
/// presses and clicks; and a hit below the top layer is `outside`
/// before deliverability is consulted, so the disabled control does not
/// swallow an outside-click dismissal.
#[test]
fn ac2_the_refusal_leaves_move_wheel_click_only_and_outside_clicks_intact() {
    let disabled = Id::root("disabled-scroll");
    let click_only = Id::root("click-only");
    let popover = Id::root("popover");
    let mut s = Stub {
        page: vec![
            Control::new(A, Rect::new(0, 0, 10, 1)),
            Control {
                focus: Focusability::Disabled,
                scroll: true,
                ..Control::new(disabled, Rect::new(20, 0, 10, 10))
            },
            Control {
                focus: Focusability::ClickOnly,
                ..Control::new(click_only, Rect::new(0, 6, 10, 1))
            },
        ],
        consume_keys: true,
        ..Stub::default()
    };
    s.layers
        .push((popover, vec![Control::new(B, Rect::new(0, 9, 10, 1))]));
    let (mut rt, mut buf) = runtime(s);

    // hover and Move still reach a disabled control
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 22, 3));
    assert_eq!(rt.hover(), Some(disabled));
    assert_eq!(rt.app().count(disabled, "phase: Move"), 1);

    // and its scroll region still scrolls
    let _ = step(
        &mut rt,
        &mut buf,
        mouse(MouseKind::Wheel(termrock_runtime::event::Axis::V, 1), 22, 3),
    );
    assert!(rt.app().saw(disabled, "Wheel { axis: V, delta: 3"));

    // a ClickOnly owner has no ring entry, so nothing proves it disabled
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 6));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 6));
    assert_eq!(rt.app().count(click_only, "phase: Press"), 1);
    assert_eq!(rt.app().count(click_only, "phase: Click"), 1);
    assert_eq!(rt.focus(), Some(A), "a ClickOnly owner took focus");

    // a press on the disabled control below an open popover is still an
    // outside click
    rt.app_mut().open_request = Some((
        popover,
        LayerSpec::popover(
            popover,
            termrock_runtime::layer::Anchor::Point(Position::new(0, 9)),
        ),
    ));
    let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
    assert!(rt.is_open(popover));

    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 22, 3));

    assert!(!rt.is_open(popover));
    assert!(rt.app().saw(popover, "Dismissed(OutsideClick)"));
    assert_eq!(rt.app().count(disabled, "phase: Press"), 0);
}

#[test]
fn hover_transition_requests_paint() {
    let (mut rt, mut buf) = runtime(three());

    let entered = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 2));
    let stayed = step(&mut rt, &mut buf, mouse(MouseKind::Move, 4, 2));
    let left = step(&mut rt, &mut buf, mouse(MouseKind::Move, 30, 10));
    let stayed_out = step(&mut rt, &mut buf, mouse(MouseKind::Move, 31, 10));

    assert_eq!(entered.invalidate(), Invalidate::Paint);
    assert_eq!(stayed.invalidate(), Invalidate::None);
    assert_eq!(left.invalidate(), Invalidate::Paint);
    assert_eq!(stayed_out.invalidate(), Invalidate::None);

    let _ = step(&mut rt, &mut buf, key(KeyCode::Down));
    let suppressed_out = step(&mut rt, &mut buf, mouse(MouseKind::Move, 32, 10));
    assert_eq!(suppressed_out.invalidate(), Invalidate::None);
}

#[test]
fn focus_transition_requests_paint_for_terminal_sessions() {
    let (mut rt, mut buf) = runtime(three());

    let response = deliver(&mut rt, key(KeyCode::Tab));

    assert_eq!(rt.focus(), Some(B));
    assert_eq!(response.invalidate(), Invalidate::Paint);
    rt.draw_buffer(SCREEN, &mut buf).commit_presented();
}

#[test]
fn double_click_within_the_window_and_wheel_routing() {
    let mut s = three();
    s.page.push(Control {
        scroll: true,
        ..Control::new(Id::root("list"), Rect::new(20, 0, 10, 10))
    });
    let (mut rt, mut buf) = runtime(s);
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 1, 0));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 1, 0));
    assert_eq!(rt.app().count(A, "DoubleClick"), 1);
    let _ = step(
        &mut rt,
        &mut buf,
        mouse(MouseKind::Wheel(termrock_runtime::event::Axis::V, 1), 25, 5),
    );
    assert!(rt.app().saw(Id::root("list"), "Wheel { axis: V, delta: 3"));
    // a wheel over a non-scroll region is dropped, never chained
    let r = step(
        &mut rt,
        &mut buf,
        mouse(MouseKind::Wheel(termrock_runtime::event::Axis::V, 1), 1, 0),
    );
    assert!(!r.is_consumed());
}

#[test]
fn resize_releases_capture_and_relayouts() {
    let (mut rt, buf) = runtime(three());
    let r = deliver(&mut rt, Input::Resize(50, 20));
    assert_eq!(r.invalidate(), Invalidate::Layout);
    assert_eq!(rt.screen(), Rect::new(0, 0, 50, 20));
    let _ = buf;
}

#[test]
fn focus_transition_settles() {
    // one programmatic hop per pass: A → B → C settles within the budget
    let mut s = three();
    s.chase = vec![C, B];
    let (mut rt, mut buf) = runtime(s);
    let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
    assert_eq!(rt.focus(), Some(C));
    assert!(
        rt.diagnostics()
            .iter()
            .all(|d| !matches!(d, Diagnostic::FocusTransitionDidNotSettle { .. }))
    );
    // FocusOut/FocusIn were enqueued per hop, never an input intent twice
    assert_eq!(rt.app().count(A, "Key("), 1);
    assert!(rt.app().saw(B, "FocusIn { via: Programmatic }"));
    assert!(rt.app().saw(C, "FocusIn { via: Programmatic }"));
}

#[test]
fn a_fifth_focus_pass_is_diagnosed_and_applied() {
    let mut s = three();
    s.chase = vec![C, B, C, B, C, B, C];
    let (mut rt, mut buf) = runtime(s);
    let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
    assert!(
        rt.diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::FocusTransitionDidNotSettle { .. }))
    );
    let settled = rt.focus().expect("the give-up path applies the transition");
    // §21 item 11 says the pair is *applied*: the give-up path used to
    // enqueue `FocusOut`/`FocusIn` and then `intents.clear()` them, so
    // neither ever reached a component (MI-7). They are now delivered by
    // the next `handle`.
    rt.app_mut().chase.clear();
    rt.app_mut().log.clear();
    let _ = step(&mut rt, &mut buf, Input::Tick);
    assert!(
        rt.app().saw(settled, "FocusIn"),
        "the matching FocusIn must reach the new owner: {:?}",
        rt.app().log
    );
    assert!(
        rt.app().log.iter().any(|(_, s)| s.contains("FocusOut")),
        "the pending FocusOut must be delivered too: {:?}",
        rt.app().log
    );
}

#[test]
fn undelivered_intents_are_diagnosed_only_for_control_owners() {
    let mut s = three();
    s.page.clear();
    s.page.push(Control::new(A, Rect::new(0, 0, 10, 1)));
    let (mut rt, mut buf) = runtime(s);
    // Keep A in actual presented geometry while deliberately leaving its bucket undrained.
    rt.app_mut().skip_drain = Some(A);
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
    assert!(
        rt.diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::UndeliveredIntent { owner } if *owner == A))
    );
}

#[test]
fn paste_reaches_only_an_editing_owner() {
    let mut s = three();
    s.page[0].editor = true;
    let (mut rt, mut buf) = runtime(s);
    let _ = step(&mut rt, &mut buf, Input::Paste("hi".to_owned()));
    assert!(rt.app().saw(A, "Paste"));
    assert!(
        rt.app()
            .log
            .iter()
            .all(|(_, message)| !message.contains("hi")),
        "paste payload reached the runtime debug log: {:?}",
        rt.app().log
    );
    let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
    assert_eq!(rt.focus(), Some(B));
    let _ = step(&mut rt, &mut buf, Input::Paste("no".to_owned()));
    assert!(!rt.app().saw(B, "Paste"));
}

#[test]
fn capture_keymap_runs_before_dispatch_and_skips_bare_chars_while_typing() {
    use termrock_runtime::action::ActionKey;
    use termrock_runtime::keymap::{KeyMap, KeyPhase};
    struct Mapped(Stub, KeyMap);
    impl App for Mapped {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if let Some(cmd) = cx.command() {
                self.0.log.push((KeyMap::OWNER, format!("cmd:{cmd:?}")));
                return Response::consumed();
            }
            self.0.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            self.0.draw(ui);
        }
        fn keymap(&self) -> &KeyMap {
            &self.1
        }
    }
    let mut s = three();
    s.page[0].editor = true;
    let km = KeyMap::new().bind(
        KeyPhase::Capture,
        termrock_runtime::event::Chord::key(KeyCode::Char('q')),
        ActionKey::CLOSE,
    );
    let mut rt = Runtime::new(Mapped(s, km), Theme::junie());
    let _ = rt.initialize();
    let mut buf = Buffer::empty(SCREEN);
    rt.draw_buffer(SCREEN, &mut buf).commit_presented();
    // the editor swallows typing: `q` reaches the editor, not the keymap
    let _ = deliver(&mut rt, key(KeyCode::Char('q')));
    assert!(rt.app().0.saw(A, "Char('q')"));
    assert!(!rt.app().0.saw(KeyMap::OWNER, "cmd"));
    rt.draw_buffer(SCREEN, &mut buf).commit_presented();
    let _ = deliver(&mut rt, key(KeyCode::Tab));
    rt.draw_buffer(SCREEN, &mut buf).commit_presented();
    let _ = deliver(&mut rt, key(KeyCode::Char('q')));
    assert!(rt.app().0.saw(KeyMap::OWNER, "cmd:ActionKey"));
}

#[test]
fn layer_scope_is_armed_at_open_and_the_cursor_is_kept_on_the_top_layer() {
    let dlg = Id::root("dlg");
    let ok = Id::root("ok");
    let mut s = three();
    s.page[0].editor = true;
    s.layers.push((
        dlg,
        vec![Control {
            editor: true,
            ..Control::new(ok, Rect::new(5, 5, 5, 1))
        }],
    ));
    let (mut rt, mut buf) = runtime(s);
    assert_eq!(rt.cursor(), Some(Position::new(0, 0)));
    rt.app_mut().open_request = Some((dlg, LayerSpec::modal(dlg)));
    let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
    let _ = step(&mut rt, &mut buf, Input::Tick);
    assert!(rt.is_open(dlg));
    assert_eq!(rt.top_layer(), LayerId(1));
    // focus reconciled into the trap; the page editor's cursor is silently discarded (inert)
    assert_eq!(rt.focus(), Some(ok));
    assert_eq!(rt.cursor(), Some(Position::new(5, 5)));
    assert!(
        rt.diagnostics()
            .iter()
            .all(|d| !matches!(d, Diagnostic::CursorRejected { .. }))
    );
    assert_eq!(rt.ring().reachable().count(), 1);
    assert!(rt.app().saw(dlg, "Layer(Opened)"));
}

/// §3.3 step 9: the dismissal of a layer whose owner registers only
/// decoration — every `Dialog` (`components/dialog.rs` registers
/// `Decorative` regions for its own id) — must still be reported when
/// the owner does not drain it. This is the gated
/// `if cx.is_open(id) { dialog.update(cx, …) }` shape: `is_open` is
/// false by the time `update` re-runs, so the `Cancel` and
/// `Layer(Dismissed)` the runtime addressed to the owner are dropped.
/// Before the guard was widened this loss was **silent**, because
/// `Registry::delivers_to` requires a `Control` or `Part` region.
/// The other half of the same rule (§21 item 13): widening the guard for
/// runtime-addressed intents must not start diagnosing a decorative
/// container for a **pointer** intent. It cannot: `deliverable` never
/// routes a pointer intent to a `Decorative` region, so the bucket is
/// never created.
#[test]
fn a_decorative_owner_is_not_diagnosed_for_a_pointer_intent() {
    let decor = Id::root("decor");
    let mut s = Stub {
        page: vec![
            Control::new(A, Rect::new(0, 0, 10, 1)),
            Control {
                decor: true,
                ..Control::new(decor, Rect::new(0, 4, 10, 3))
            },
        ],
        ..Stub::default()
    };
    s.skip_drain = Some(decor);
    let (mut rt, mut buf) = runtime(s);
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 5));
    let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 5));
    assert!(
        rt.diagnostics()
            .iter()
            .all(|d| !matches!(d, Diagnostic::UndeliveredIntent { .. })),
        "{:?}",
        rt.diagnostics()
    );
    assert!(!rt.app().saw(decor, "Pointer"));
}

#[test]
fn disabled_controls_are_registered_but_never_focused() {
    let mut s = three();
    s.page[1].focus = Focusability::Disabled;
    let (mut rt, mut buf) = runtime(s);
    assert!(rt.ring().is_registered(B));
    assert!(rt.state_of(B).contains(StateFlags::DISABLED));
    let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
    assert_eq!(rt.focus(), Some(C));
}

/// S1: a `Cx::focus` naming a `Disabled` entry is refused outright — not
/// applied for one frame and then undone by step 14 reconciliation,
/// which would deliver `FocusIn` to a control that can never be focused.
#[test]
fn a_programmatic_focus_request_for_a_disabled_control_is_refused() {
    let mut s = three();
    s.page[1].focus = Focusability::Disabled;
    s.focus_request = Some(B);
    let (mut rt, mut buf) = runtime(s);
    assert_eq!(rt.focus(), Some(A));

    let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

    assert_eq!(rt.focus(), Some(A), "focus must not move to a disabled id");
    assert!(!rt.app().saw(B, "FocusIn"), "{:?}", rt.app().log);
    assert!(!rt.app().saw(A, "FocusOut"), "{:?}", rt.app().log);
}

/// S1: `ClickOnly` is "a hit target, never in the ring". Such an id is
/// registered in the *registry* but has no ring entry, and that pair is
/// exactly the proof a focus request naming it is wrong.
#[test]
fn a_programmatic_focus_request_for_a_click_only_control_is_refused() {
    let mut s = three();
    s.page[1].focus = Focusability::ClickOnly;
    s.focus_request = Some(B);
    let (mut rt, mut buf) = runtime(s);
    assert!(!rt.ring().is_registered(B));
    assert!(rt.registry().has_owner(B));

    let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

    assert_eq!(rt.focus(), Some(A));
    assert!(!rt.app().saw(B, "FocusIn"), "{:?}", rt.app().log);
}

/// S1: opening a modal makes the page inert in the very same pass, while
/// the page controls are still in *last* frame's ring. A focus request
/// naming one of them is refused, so nothing behind the modal can take
/// focus out of it.
#[test]
fn a_focus_request_for_a_control_the_open_modal_made_inert_is_refused() {
    const DLG: Id = Id::root("runtime.inert-dialog");
    const OK: Id = Id::root("runtime.inert-dialog.ok");
    let mut s = three();
    s.layers = vec![(DLG, vec![Control::new(OK, Rect::new(2, 2, 6, 1))])];
    let (mut rt, mut buf) = runtime(s);
    assert_eq!(rt.focus(), Some(A));

    rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
    rt.app_mut().focus_request = Some(B);
    let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

    assert!(rt.open_spec(DLG).is_some_and(|s| s.inert_below));
    assert_ne!(rt.focus(), Some(B), "an inert page control took focus");
    assert!(!rt.app().saw(B, "FocusIn"), "{:?}", rt.app().log);
    assert_eq!(
        rt.focus(),
        Some(OK),
        "step 14 reconciliation puts focus inside the modal instead"
    );
}

/// S1's counterpart: an id the last frame never saw is *unknown*, not
/// proven unreachable, and must still be admitted — a layer's own
/// controls are absent from the ring until they first draw, so
/// `LayerSpec::initial_focus` names an unknown id by construction.
#[test]
fn initial_focus_into_a_layer_that_has_never_drawn_is_still_admitted() {
    const DLG: Id = Id::root("runtime.initial-focus-dialog");
    const OK: Id = Id::root("runtime.initial-focus-dialog.ok");
    let mut s = three();
    s.layers = vec![(DLG, vec![Control::new(OK, Rect::new(2, 2, 6, 1))])];
    let (mut rt, _buf) = runtime(s);
    assert!(!rt.ring().is_registered(OK));
    assert!(!rt.registry().has_owner(OK));

    rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG).initial_focus(OK)));
    let _ = deliver(&mut rt, key(KeyCode::Char('x')));

    assert_eq!(
        rt.focus(),
        Some(OK),
        "the named target holds focus before the layer has ever drawn"
    );
}

#[test]
fn same_flags_table_switch_invalidates_focused_hints() {
    let (mut runtime, mut buffer) = route_runtime(RouteApp::default());
    assert!(hint_row(&buffer).contains("Default"));
    let capacity = runtime.focused_hints_capacity();
    runtime.app_mut().options |= ROUTE_ALTERNATE;
    runtime
        .draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer)
        .commit_presented();
    assert!(hint_row(&buffer).contains("Alternate"));
    assert_eq!(runtime.focused_hints_capacity(), capacity);
}
