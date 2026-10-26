#![allow(clippy::unwrap_used, reason = "native layer routing assertions")]

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;
use termrock_core::event::{
    Axis, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, RoutedInput,
};
use termrock_core::intent::{Intent, Phase};
use termrock_core::response::Response;
use termrock_core::{Axes, Headroom, Id, Part, PartRef};
use termrock_runtime::{
    Anchor, App, Cx, Dismiss, Focusability, LayerId, LayerSize, LayerSpec, Runtime, Ui,
    UpdateCause, WheelRouteAttempt, WheelRouteResult,
};
use termrock_theme::Theme;

const SCREEN: Rect = Rect::new(0, 0, 32, 12);
const PAGE: Id = Id::root("passive-tip.page");
const TIP: Id = Id::root("passive-tip.tip");
const TIP_TWO: Id = Id::root("passive-tip.tip-two");
const POPUP: Id = Id::root("passive-tip.popup");
const MODAL: Id = Id::root("passive-tip.modal");
const ROUTE_FIRST: Id = Id::root("passive-tip.route-first");
const ROUTE_SECOND: Id = Id::root("passive-tip.route-second");
const PAGE_PART: PartRef = PartRef::of(Part::CONTAINER);
const TIP_HOVER: PartRef = PartRef::of(Part::LABEL);
const PAGE_AREA: Rect = Rect::new(0, 0, 24, 9);
const TIP_AREA: Rect = Rect::new(6, 2, 8, 4);
const ROUTE_SOURCE_REVISION: u64 = 17;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Setup {
    #[default]
    None,
    Tooltip,
    InertTooltip,
    ModalThenTooltip,
    TooltipThenPopover,
    PopoverThenTooltips,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointerSeen {
    owner: Id,
    phase: Phase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WheelSeen {
    owner: Id,
    axis: Axis,
}

struct Probe {
    setup: Setup,
    transparent: bool,
    dismiss: Option<Dismiss>,
    open_tip_on_press: bool,
    capture_on_press: bool,
    open_modal_on_press: bool,
    consume_escape: bool,
    tip_visible: bool,
    tip_two_visible: bool,
    modal_visible: bool,
    popup_visible: bool,
    pointers: Vec<PointerSeen>,
    wheels: Vec<WheelSeen>,
    keys: Vec<(Id, KeyCode)>,
    cancels: Vec<Id>,
    on_esc_calls: usize,
    layer_events: Vec<(Id, termrock_runtime::LayerEvent)>,
}

impl Probe {
    fn new(setup: Setup, transparent: bool, dismiss: Option<Dismiss>) -> Self {
        Self {
            setup,
            transparent,
            dismiss,
            open_tip_on_press: false,
            capture_on_press: false,
            open_modal_on_press: false,
            consume_escape: false,
            tip_visible: false,
            tip_two_visible: false,
            modal_visible: false,
            popup_visible: false,
            pointers: Vec::new(),
            wheels: Vec::new(),
            keys: Vec::new(),
            cancels: Vec::new(),
            on_esc_calls: 0,
            layer_events: Vec::new(),
        }
    }

    fn tooltip_spec(&self, id: Id, inert: bool) -> LayerSpec {
        let mut spec =
            LayerSpec::tooltip(id, Position::new(TIP_AREA.x, TIP_AREA.y.saturating_sub(1)));
        if let Some(dismiss) = self.dismiss {
            spec = spec.dismiss(dismiss);
        }
        spec.size = LayerSize::Fixed(TIP_AREA.width, TIP_AREA.height);
        spec.inert_below = inert;
        if self.transparent {
            spec = spec.tooltip_pointer_transparent(true);
        }
        spec
    }

    fn open_tip(&mut self, cx: &mut Cx<'_>, inert: bool) {
        self.tip_visible = true;
        cx.open_layer(TIP, self.tooltip_spec(TIP, inert));
    }

    fn open_second_tip(&mut self, cx: &mut Cx<'_>) {
        self.tip_two_visible = true;
        cx.open_layer(TIP_TWO, self.tooltip_spec(TIP_TWO, false));
    }

    fn open_popup(&mut self, cx: &mut Cx<'_>) {
        let mut popup = LayerSpec::popover(POPUP, Anchor::Point(Position::new(4, 1)))
            .tooltip_pointer_transparent(true);
        popup.size = LayerSize::Fixed(16, 8);
        self.popup_visible = true;
        cx.open_layer(POPUP, popup);
    }
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut page_pressed = false;
        let mut consumed_escape = false;
        for owner in [PAGE, TIP, TIP_TWO, POPUP, MODAL] {
            for intent in cx.intents(owner) {
                match intent {
                    Intent::Pointer { phase, .. } => {
                        self.pointers.push(PointerSeen { owner, phase });
                        page_pressed |= owner == PAGE && phase == Phase::Press;
                    }
                    Intent::Wheel { axis, .. } => self.wheels.push(WheelSeen { owner, axis }),
                    Intent::Key(key) => {
                        self.keys.push((owner, key.code));
                        consumed_escape |= self.consume_escape && key.code == KeyCode::Esc;
                    }
                    Intent::Layer(event) => self.layer_events.push((owner, event)),
                    Intent::Cancel => self.cancels.push(owner),
                    _ => {}
                }
            }
        }

        for id in [TIP, TIP_TWO, POPUP, MODAL] {
            while let Some(receipt) = cx.layer_event_receipt(id) {
                self.layer_events.push((id, receipt.event()));
            }
        }

        if cx.update_cause() == UpdateCause::Bootstrap {
            match self.setup {
                Setup::None => {}
                Setup::Tooltip => self.open_tip(cx, false),
                Setup::InertTooltip => self.open_tip(cx, true),
                Setup::ModalThenTooltip => {
                    self.modal_visible = true;
                    cx.open_layer(MODAL, LayerSpec::modal(MODAL));
                    self.open_tip(cx, false);
                }
                Setup::TooltipThenPopover => {
                    self.open_tip(cx, false);
                    self.open_popup(cx);
                }
                Setup::PopoverThenTooltips => {
                    self.open_popup(cx);
                    self.open_tip(cx, false);
                    self.open_second_tip(cx);
                }
            }

            // Keep default-policy fixtures focused within their lower layer at first publication.
            if self.dismiss.is_none() {
                match self.setup {
                    Setup::Tooltip => cx.focus(PAGE),
                    Setup::PopoverThenTooltips => cx.focus(POPUP),
                    _ => {}
                }
            }
        }

        if page_pressed && self.capture_on_press {
            assert!(cx.capture(PAGE, PAGE_PART));
            self.capture_on_press = false;
        }
        if page_pressed && self.open_tip_on_press {
            self.open_tip(cx, false);
            self.open_tip_on_press = false;
        }
        if page_pressed && self.open_modal_on_press {
            self.modal_visible = true;
            cx.open_layer(MODAL, LayerSpec::modal(MODAL));
            self.open_modal_on_press = false;
        }
        if consumed_escape {
            Response::consumed()
        } else {
            Response::ignored()
        }
    }

    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        self.on_esc_calls += 1;
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(PAGE, PAGE_AREA, Focusability::Focusable);
        ui.register_part(PAGE, PAGE_PART, PAGE_AREA);
        ui.register_text_drag_autoscroll(
            PAGE,
            PAGE_PART,
            PAGE_AREA,
            Axes::V,
            Headroom {
                down: 2,
                ..Headroom::default()
            },
            1,
        );
        ui.register_scroll(
            PAGE,
            PAGE_AREA,
            Axes::Both,
            Headroom {
                up: 2,
                down: 2,
                left: 2,
                right: 2,
            },
        );

        if self.modal_visible {
            ui.layer(MODAL, |ui, area| {
                ui.register_control(MODAL, area, Focusability::Focusable);
            });
        }
        if self.setup == Setup::PopoverThenTooltips && self.popup_visible {
            ui.layer(POPUP, |ui, area| {
                ui.register_control(POPUP, area, Focusability::Focusable);
            });
        }
        if self.tip_visible {
            ui.layer(TIP, |ui, area| {
                ui.paint_str(
                    Rect::new(area.x, area.y, area.width.min(3), 1),
                    "TIP",
                    Style::default(),
                );
                ui.register_hover_part(TIP, TIP_HOVER, area);
            });
        }
        if self.tip_two_visible {
            ui.layer(TIP_TWO, |ui, area| {
                ui.register_hover_part(TIP_TWO, TIP_HOVER, area);
            });
        }
        if self.popup_visible && self.setup != Setup::PopoverThenTooltips {
            ui.layer(POPUP, |ui, area| {
                ui.register_control(POPUP, area, Focusability::Focusable);
            });
        }
    }
}

fn present<A: App>(rt: &mut Runtime<A>, buffer: &mut Buffer) {
    for _ in 0..16 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        if rt.needs_present() {
            rt.draw_buffer(SCREEN, buffer).commit_presented();
        }
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
    }
    panic!("runtime did not settle and present");
}

fn ready<A: App>(probe: A) -> (Runtime<A>, Buffer) {
    let mut rt = Runtime::new(probe, Theme::junie());
    let _ = rt.initialize();
    let mut buffer = Buffer::empty(SCREEN);
    present(&mut rt, &mut buffer);
    (rt, buffer)
}

fn mouse(kind: MouseKind, x: u16, y: u16) -> Input {
    Input::Mouse(Mouse {
        kind,
        pos: Position::new(x, y),
        mods: KeyModifiers::NONE,
    })
}

fn count_pointer(rt: &Runtime<Probe>, owner: Id, phase: Phase) -> usize {
    rt.app()
        .pointers
        .iter()
        .filter(|seen| seen.owner == owner && seen.phase == phase)
        .count()
}

fn page_focused_tooltip(transparent: bool, consume_escape: bool) -> (Runtime<Probe>, Buffer) {
    let mut probe = Probe::new(Setup::None, transparent, Some(Dismiss::ESC));
    probe.open_tip_on_press = true;
    probe.consume_escape = consume_escape;
    let (mut rt, mut buffer) = ready(probe);
    rt.set_focus(Some(PAGE));
    present(&mut rt, &mut buffer);

    let _ = rt.handle(mouse(MouseKind::Down, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Up, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert!(rt.is_open(TIP));
    assert_eq!(rt.focus(), Some(PAGE));
    (rt, buffer)
}

struct RoutedWheelProbe {
    close_tip_on_initial_attempt: bool,
    tip_visible: bool,
    initial_attempts: Vec<WheelRouteAttempt>,
    service_attempts: Vec<WheelRouteAttempt>,
}

impl RoutedWheelProbe {
    fn new(close_tip_on_initial_attempt: bool) -> Self {
        Self {
            close_tip_on_initial_attempt,
            tip_visible: false,
            initial_attempts: Vec::new(),
            service_attempts: Vec::new(),
        }
    }
}

impl App for RoutedWheelProbe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            let mut spec =
                LayerSpec::tooltip(TIP, Position::new(TIP_AREA.x, TIP_AREA.y.saturating_sub(1)))
                    .dismiss(Dismiss::NONE)
                    .tooltip_pointer_transparent(true);
            spec.size = LayerSize::Fixed(TIP_AREA.width, TIP_AREA.height);
            self.tip_visible = true;
            cx.open_layer(TIP, spec);
        }

        for owner in [ROUTE_FIRST, ROUTE_SECOND] {
            for intent in cx.intents(owner) {
                if let Intent::Wheel { axis, part, .. } = intent
                    && let Some(attempt) = cx.wheel_route_attempt(owner, part, axis)
                {
                    assert_eq!(owner, ROUTE_FIRST);
                    assert_eq!(attempt.owner(), ROUTE_FIRST);
                    assert_eq!(attempt.attempt_index(), 0);
                    assert_eq!(attempt.source_revision(), ROUTE_SOURCE_REVISION);
                    self.initial_attempts.push(attempt);
                    if self.close_tip_on_initial_attempt && self.tip_visible {
                        self.close_tip_on_initial_attempt = false;
                        self.tip_visible = false;
                        cx.close_layer(TIP, None);
                    }
                    assert!(cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Ignored));
                }
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_hit_stack_wheel(
            ROUTE_SECOND,
            PAGE_PART,
            PAGE_AREA,
            Axes::V,
            ROUTE_SOURCE_REVISION,
        );
        ui.register_hit_stack_wheel(
            ROUTE_FIRST,
            PAGE_PART,
            PAGE_AREA,
            Axes::V,
            ROUTE_SOURCE_REVISION,
        );
        if self.tip_visible {
            ui.layer(TIP, |ui, area| {
                ui.paint_str(
                    Rect::new(area.x, area.y, area.width.min(3), 1),
                    "TIP",
                    Style::default(),
                );
                ui.register_hover_part(TIP, TIP_HOVER, area);
            });
        }
    }

    fn route_wheel_attempt(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        assert_eq!(attempt.source_revision(), ROUTE_SOURCE_REVISION);
        assert_eq!(attempt.owner(), ROUTE_SECOND);
        assert_eq!(attempt.attempt_index(), 1);
        self.service_attempts.push(attempt);
        assert!(cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Consumed));
        Response::ignored()
    }
}

#[test]
fn transparent_tooltip_keeps_visual_top_and_routes_native_pointer_hover_and_wheel_to_page() {
    let probe = Probe::new(Setup::Tooltip, true, Some(Dismiss::NONE));
    let (mut rt, mut buffer) = ready(probe);
    let visual_top = rt.top_layer();
    assert_ne!(visual_top, LayerId::PAGE);
    assert_eq!(buffer[(TIP_AREA.x, TIP_AREA.y)].symbol(), "T");

    let _ = rt.handle(mouse(MouseKind::Move, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.hover(), Some(TIP));
    assert_eq!(count_pointer(&rt, TIP, Phase::Move), 0);

    let _ = rt.handle(mouse(MouseKind::Move, 2, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.hover(), Some(PAGE));

    let _ = rt.handle(mouse(MouseKind::Down, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Up, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Click), 1);
    assert_eq!(count_pointer(&rt, TIP, Phase::Press), 0);
    assert_eq!(rt.top_layer(), visual_top);

    let _ = rt
        .handle_routed(RoutedInput::MiddleClick {
            pos: Position::new(7, 3),
            mods: KeyModifiers::NONE,
        })
        .unwrap();
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Secondary, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);
    let _ = rt
        .handle(mouse(MouseKind::Wheel(Axis::H, 1), 7, 3))
        .unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Middle), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Secondary), 1);
    assert!(rt.app().wheels.contains(&WheelSeen {
        owner: PAGE,
        axis: Axis::H,
    }));
    assert!(rt.app().wheels.iter().all(|seen| seen.owner != TIP));
}

#[test]
fn default_tooltip_remains_opaque_and_inert_tooltip_cannot_be_skipped() {
    let (mut opaque, mut buffer) = ready(Probe::new(Setup::Tooltip, false, None));
    assert!(opaque.is_open(TIP));
    let _ = opaque.handle(mouse(MouseKind::Down, 7, 3)).unwrap();
    present(&mut opaque, &mut buffer);
    let _ = opaque.handle(mouse(MouseKind::Up, 7, 3)).unwrap();
    assert_eq!(count_pointer(&opaque, PAGE, Phase::Press), 0);
    assert!(
        !opaque.is_open(TIP),
        "the default outside policy dismisses its opaque layer"
    );

    let (mut inert, mut buffer) = ready(Probe::new(Setup::InertTooltip, true, Some(Dismiss::NONE)));
    let _ = inert.handle(mouse(MouseKind::Down, 2, 3)).unwrap();
    present(&mut inert, &mut buffer);
    let _ = inert.handle(mouse(MouseKind::Up, 2, 3)).unwrap();
    assert_eq!(count_pointer(&inert, PAGE, Phase::Press), 0);
    assert!(inert.is_open(TIP));
}

#[test]
fn transparent_tooltip_outside_dismissal_keeps_the_original_page_press() {
    let (mut rt, mut buffer) = ready(Probe::new(Setup::Tooltip, true, None));
    assert!(rt.is_open(TIP));
    let _ = rt.handle(mouse(MouseKind::Down, 2, 2)).unwrap();
    assert!(!rt.is_open(TIP));
    assert!(rt.app().layer_events.contains(&(
        TIP,
        termrock_runtime::LayerEvent::Dismissed(termrock_runtime::DismissReason::OutsideClick),
    )));
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 1);
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Up, 2, 2)).unwrap();
    assert_eq!(count_pointer(&rt, PAGE, Phase::Click), 1);
    assert_eq!(count_pointer(&rt, TIP, Phase::Press), 0);
}

#[test]
fn opaque_popup_and_modal_remain_the_input_boundary_around_transparent_tooltips() {
    let (mut popup, mut popup_buffer) = ready(Probe::new(
        Setup::TooltipThenPopover,
        true,
        Some(Dismiss::NONE),
    ));
    let _ = popup.handle(mouse(MouseKind::Move, 7, 3)).unwrap();
    present(&mut popup, &mut popup_buffer);
    assert_eq!(popup.hover(), Some(POPUP));
    let _ = popup
        .handle(mouse(MouseKind::Wheel(Axis::V, 1), 7, 3))
        .unwrap();
    present(&mut popup, &mut popup_buffer);
    assert!(popup.app().wheels.iter().all(|seen| seen.owner != PAGE));
    let _ = popup.handle(mouse(MouseKind::Down, 7, 3)).unwrap();
    present(&mut popup, &mut popup_buffer);
    assert_eq!(count_pointer(&popup, POPUP, Phase::Press), 1);
    assert_eq!(count_pointer(&popup, PAGE, Phase::Press), 0);

    let (mut modal, mut modal_buffer) = ready(Probe::new(
        Setup::ModalThenTooltip,
        true,
        Some(Dismiss::NONE),
    ));
    let _ = modal.handle(mouse(MouseKind::Down, 7, 3)).unwrap();
    present(&mut modal, &mut modal_buffer);
    assert_eq!(count_pointer(&modal, MODAL, Phase::Press), 1);
    assert_eq!(count_pointer(&modal, PAGE, Phase::Press), 0);
    let _ = modal
        .handle(mouse(MouseKind::Wheel(Axis::V, 1), 7, 3))
        .unwrap();
    present(&mut modal, &mut modal_buffer);
    assert!(modal.app().wheels.iter().all(|seen| seen.owner != PAGE));
}

#[test]
fn multiple_transparent_tips_close_with_native_receipts_without_replaying_outside_to_page() {
    let (mut rt, mut buffer) = ready(Probe::new(Setup::PopoverThenTooltips, true, None));
    assert!(rt.is_open(TIP));
    assert!(rt.is_open(TIP_TWO));
    assert!(rt.is_open(POPUP));
    let _ = rt.handle(mouse(MouseKind::Move, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.hover(), Some(TIP_TWO));

    let _ = rt.handle(mouse(MouseKind::Down, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(count_pointer(&rt, POPUP, Phase::Press), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 0);
    let _ = rt.handle(mouse(MouseKind::Up, 7, 3)).unwrap();
    present(&mut rt, &mut buffer);

    let _ = rt.handle(mouse(MouseKind::Down, 2, 2)).unwrap();
    assert!(!rt.is_open(TIP_TWO));
    assert!(!rt.is_open(TIP));
    assert!(!rt.is_open(POPUP));
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 0);
    present(&mut rt, &mut buffer);
    assert!(rt.app().layer_events.contains(&(
        TIP_TWO,
        termrock_runtime::LayerEvent::Dismissed(termrock_runtime::DismissReason::OutsideClick,),
    )));
    assert!(rt.app().layer_events.contains(&(
        TIP,
        termrock_runtime::LayerEvent::Dismissed(termrock_runtime::DismissReason::OutsideClick,),
    )));
    assert!(rt.app().layer_events.contains(&(
        POPUP,
        termrock_runtime::LayerEvent::Dismissed(termrock_runtime::DismissReason::OutsideClick,),
    )));
}

#[test]
fn focused_page_keeps_keys_and_escape_still_dismisses_the_actual_visual_top() {
    let (mut rt, mut buffer) = ready(Probe::new(Setup::Tooltip, true, Some(Dismiss::ESC)));
    rt.set_focus(Some(PAGE));
    present(&mut rt, &mut buffer);
    let visual_top = rt.top_layer();

    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut rt, &mut buffer);
    assert!(rt.app().keys.contains(&(PAGE, KeyCode::Char('x'))));

    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    assert!(!rt.is_open(TIP));
    assert_eq!(rt.focus(), Some(PAGE));
    assert_eq!(rt.top_layer(), LayerId::PAGE);
    assert_ne!(visual_top, LayerId::PAGE);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 0);
}

#[test]
fn bare_escape_preempts_a_consuming_lower_owner_only_for_the_passive_top_tooltip() {
    let (mut transparent, mut transparent_buffer) = page_focused_tooltip(true, true);
    let _ = transparent
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut transparent, &mut transparent_buffer);
    assert!(!transparent.is_open(TIP));
    assert!(!transparent.app().keys.contains(&(PAGE, KeyCode::Esc)));
    assert!(!transparent.app().cancels.contains(&PAGE));
    assert!(transparent.app().cancels.contains(&TIP));
    assert_eq!(transparent.focus(), Some(PAGE));
    assert!(transparent.focus_visible());
    assert_eq!(transparent.app().on_esc_calls, 0);
    assert!(transparent.app().layer_events.contains(&(
        TIP,
        termrock_runtime::LayerEvent::Dismissed(termrock_runtime::DismissReason::Esc),
    )));

    let (mut opaque, mut opaque_buffer) = page_focused_tooltip(false, true);
    let _ = opaque
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut opaque, &mut opaque_buffer);
    assert!(opaque.is_open(TIP));
    assert!(opaque.app().keys.contains(&(PAGE, KeyCode::Esc)));
    assert!(!opaque.app().cancels.contains(&TIP));
    assert_eq!(opaque.focus(), Some(PAGE));
    assert_eq!(opaque.app().on_esc_calls, 0);

    let (mut modified, mut modified_buffer) = page_focused_tooltip(true, true);
    let _ = modified
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::SHIFT,
        }))
        .unwrap();
    present(&mut modified, &mut modified_buffer);
    assert!(modified.is_open(TIP));
    assert!(modified.app().keys.contains(&(PAGE, KeyCode::Esc)));
    assert!(!modified.app().cancels.contains(&TIP));
    assert_eq!(modified.focus(), Some(PAGE));
    assert_eq!(modified.app().on_esc_calls, 0);
}

#[test]
fn bare_escape_closes_only_the_passive_top_tip_above_an_opaque_menu() {
    let (mut rt, mut buffer) = ready(Probe::new(
        Setup::PopoverThenTooltips,
        true,
        Some(Dismiss::ESC),
    ));
    assert!(rt.is_open(TIP));
    assert!(rt.is_open(TIP_TWO));
    assert!(rt.is_open(POPUP));
    rt.set_focus(Some(POPUP));
    present(&mut rt, &mut buffer);
    assert_eq!(rt.focus(), Some(POPUP));

    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut rt, &mut buffer);

    assert!(!rt.is_open(TIP_TWO));
    assert!(rt.is_open(TIP));
    assert!(rt.is_open(POPUP));
    assert_eq!(rt.focus(), Some(POPUP));
    assert!(!rt.app().keys.contains(&(POPUP, KeyCode::Esc)));
    assert!(!rt.app().cancels.contains(&POPUP));
    assert!(rt.app().cancels.contains(&TIP_TWO));
    assert_eq!(rt.app().on_esc_calls, 0);
}

#[test]
fn bare_escape_closes_a_passive_tip_above_a_modal_without_replaying_to_the_modal() {
    let (mut rt, mut buffer) = ready(Probe::new(
        Setup::ModalThenTooltip,
        true,
        Some(Dismiss::ESC),
    ));
    assert!(rt.is_open(TIP));
    assert!(rt.is_open(MODAL));
    rt.set_focus(Some(MODAL));
    present(&mut rt, &mut buffer);
    assert_eq!(rt.focus(), Some(MODAL));

    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut rt, &mut buffer);

    assert!(!rt.is_open(TIP));
    assert!(rt.is_open(MODAL));
    assert_eq!(rt.focus(), Some(MODAL));
    assert!(!rt.app().keys.contains(&(MODAL, KeyCode::Esc)));
    assert!(!rt.app().cancels.contains(&MODAL));
    assert!(rt.app().cancels.contains(&TIP));
    assert_eq!(rt.app().on_esc_calls, 0);
}

#[test]
fn passive_escape_does_not_preempt_when_the_tooltip_refuses_escape() {
    let mut probe = Probe::new(Setup::None, true, Some(Dismiss::NONE));
    probe.open_tip_on_press = true;
    probe.consume_escape = true;
    let (mut rt, mut buffer) = ready(probe);
    rt.set_focus(Some(PAGE));
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Down, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Up, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert!(rt.is_open(TIP));
    assert_eq!(rt.focus(), Some(PAGE));

    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut rt, &mut buffer);

    assert!(rt.is_open(TIP));
    assert!(rt.app().keys.contains(&(PAGE, KeyCode::Esc)));
    assert!(!rt.app().cancels.contains(&TIP));
    assert_eq!(rt.focus(), Some(PAGE));
}

#[test]
fn lower_capture_survives_transparent_tooltip_publication_and_releases_once() {
    let mut probe = Probe::new(Setup::None, true, Some(Dismiss::ESC));
    probe.capture_on_press = true;
    probe.open_tip_on_press = true;
    let (mut rt, mut buffer) = ready(probe);

    let _ = rt.handle(mouse(MouseKind::Down, 3, 3)).unwrap();
    assert_eq!(rt.capture_owner(), Some(PAGE));
    present(&mut rt, &mut buffer);
    assert_eq!(rt.capture_owner(), Some(PAGE));
    assert!(rt.is_open(TIP));
    let _ = rt.handle(mouse(MouseKind::Drag, 8, 4)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Drag), 1);
    assert_eq!(rt.capture_owner(), Some(PAGE));

    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut rt, &mut buffer);
    assert!(!rt.is_open(TIP));
    assert_eq!(rt.capture_owner(), Some(PAGE));

    let _ = rt.handle(mouse(MouseKind::Up, 28, 10)).unwrap();
    assert_eq!(rt.capture_owner(), None);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Release), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::DragEnd), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Click), 0);
}

#[test]
fn fresh_capture_starts_below_an_already_open_transparent_tooltip() {
    let mut probe = Probe::new(Setup::Tooltip, true, Some(Dismiss::NONE));
    probe.capture_on_press = true;
    let (mut rt, mut buffer) = ready(probe);
    assert!(rt.is_open(TIP));

    let _ = rt.handle(mouse(MouseKind::Down, 7, 3)).unwrap();
    assert_eq!(rt.capture_owner(), Some(PAGE));
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 1);
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Drag, 8, 4)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.capture_owner(), Some(PAGE));
    assert_eq!(count_pointer(&rt, PAGE, Phase::Drag), 1);

    let _ = rt.handle(mouse(MouseKind::Up, 28, 10)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.capture_owner(), None);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Release), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::DragEnd), 1);
    assert_eq!(count_pointer(&rt, PAGE, Phase::Click), 0);
}

#[test]
fn routed_wheel_fallback_below_a_passive_tip_keeps_the_physical_attempt_identity() {
    let (mut rt, mut buffer) = ready(RoutedWheelProbe::new(false));
    assert!(rt.is_open(TIP));
    let _ = rt
        .handle(mouse(MouseKind::Wheel(Axis::V, 1), 7, 3))
        .unwrap();
    present(&mut rt, &mut buffer);

    let app = rt.app();
    assert_eq!(app.initial_attempts.len(), 1);
    assert_eq!(app.service_attempts.len(), 1);
    let first = app.initial_attempts[0];
    let fallback = app.service_attempts[0];
    assert_eq!(first.owner(), ROUTE_FIRST);
    assert_eq!(first.layer(), LayerId::PAGE);
    assert_eq!(first.attempt_index(), 0);
    assert_eq!(fallback.owner(), ROUTE_SECOND);
    assert_eq!(fallback.attempt_index(), 1);
    assert_eq!(fallback.event_id(), first.event_id());
    assert_eq!(fallback.layer(), first.layer());
    assert_eq!(fallback.axis(), first.axis());
    assert_eq!(fallback.delta(), first.delta());
    assert_eq!(fallback.unscaled_delta(), first.unscaled_delta());
    assert_eq!(fallback.position(), first.position());
    assert_eq!(fallback.modifiers(), first.modifiers());
    assert_eq!(fallback.registry_generation(), first.registry_generation());
    assert_eq!(fallback.source_revision(), first.source_revision());
    assert!(rt.is_open(TIP));
    assert_ne!(rt.top_layer(), LayerId::PAGE);
}

#[test]
fn routed_wheel_fallback_stops_when_initial_owner_closes_the_passive_tip() {
    let (mut rt, mut buffer) = ready(RoutedWheelProbe::new(true));
    assert!(rt.is_open(TIP));
    assert_ne!(rt.top_layer(), LayerId::PAGE);
    let _ = rt
        .handle(mouse(MouseKind::Wheel(Axis::V, 1), 7, 3))
        .unwrap();
    present(&mut rt, &mut buffer);

    let app = rt.app();
    assert_eq!(app.initial_attempts.len(), 1);
    assert_eq!(app.service_attempts.len(), 0);
    let initial = app.initial_attempts[0];
    assert_eq!(initial.owner(), ROUTE_FIRST);
    assert_eq!(initial.attempt_index(), 0);
    assert_eq!(initial.layer(), LayerId::PAGE);
    assert!(!rt.is_open(TIP));
    assert_eq!(rt.top_layer(), LayerId::PAGE);
}

#[test]
fn held_text_drag_autoscroll_remains_live_below_a_transparent_tooltip() {
    let mut probe = Probe::new(Setup::None, true, Some(Dismiss::NONE));
    probe.capture_on_press = true;
    probe.open_tip_on_press = true;
    let (mut rt, mut buffer) = ready(probe);

    let _ = rt.handle(mouse(MouseKind::Down, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    let _ = rt.handle(mouse(MouseKind::Drag, 3, 9)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.capture_owner(), Some(PAGE));
    let first_drag_count = count_pointer(&rt, PAGE, Phase::Drag);
    assert_eq!(first_drag_count, 1);
    assert!(rt.next_deadline().is_some());

    rt.advance_clock(50);
    let _ = rt.settle();
    present(&mut rt, &mut buffer);
    assert!(count_pointer(&rt, PAGE, Phase::Drag) > first_drag_count);
    assert_eq!(rt.capture_owner(), Some(PAGE));
}

#[test]
fn an_opaque_barrier_cannot_turn_a_page_press_into_a_stale_click() {
    let mut probe = Probe::new(Setup::None, false, Some(Dismiss::NONE));
    probe.open_modal_on_press = true;
    let (mut rt, mut buffer) = ready(probe);

    let _ = rt.handle(mouse(MouseKind::Down, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert!(rt.is_open(MODAL));
    assert_eq!(count_pointer(&rt, PAGE, Phase::Press), 1);

    let _ = rt.handle(mouse(MouseKind::Up, 3, 3)).unwrap();
    assert_eq!(count_pointer(&rt, PAGE, Phase::Click), 0);
}

#[test]
fn an_opaque_modal_opened_during_capture_still_cancels_the_lower_capture() {
    let mut probe = Probe::new(Setup::None, false, Some(Dismiss::NONE));
    probe.capture_on_press = true;
    probe.open_modal_on_press = true;
    let (mut rt, mut buffer) = ready(probe);

    let _ = rt.handle(mouse(MouseKind::Down, 3, 3)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.capture_owner(), None);
    assert!(rt.is_open(MODAL));
    assert_ne!(rt.top_layer(), LayerId::PAGE);
    let _ = rt.handle(mouse(MouseKind::Drag, 8, 4)).unwrap();
    assert_eq!(count_pointer(&rt, PAGE, Phase::Drag), 0);
}
