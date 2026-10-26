#![allow(clippy::unwrap_used, reason = "native tooltip lifecycle assertions")]

use std::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;
use termrock_core::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_core::intent::{Intent, Phase};
use termrock_core::response::Response;
use termrock_core::{Id, Part, PartRef};
use termrock_runtime::{
    Anchor, App, Cx, DismissReason, Focusability, LayerEvent, LayerId, LayerOpening, LayerSize,
    LayerSpec, Runtime, Ui, UpdateCause,
};
use termrock_theme::Theme;

const SCREEN: Rect = Rect::new(0, 0, 32, 12);
const SOURCE: Id = Id::root("hover-tip.source");
const TIP: Id = Id::root("hover-tip.tip");
const NEXT_TIP: Id = Id::root("hover-tip.next-tip");
const UNDERLAY: Id = Id::root("hover-tip.ordinary-underlay");
const PARENT: Id = Id::root("hover-tip.opaque-parent");
const MODAL: Id = Id::root("hover-tip.modal");
const LOWER_SOURCE: Id = Id::root("hover-tip.lower-source");
const UNRELATED: Id = Id::root("hover-tip.unrelated");
const SOURCE_PART: PartRef = PartRef::of(Part::HELP);
const SIBLING_PART: PartRef = PartRef::of(Part::LABEL);
const MISSING_PART: PartRef = PartRef::of(Part::ACTION);
const LOWER_SOURCE_PART: PartRef = PartRef::of(Part::HELP);
const UNRELATED_PART: PartRef = PartRef::of(Part::HELP);
const SOURCE_AREA: Rect = Rect::new(3, 2, 8, 2);
const SIBLING_AREA: Rect = Rect::new(13, 2, 8, 2);
const LOWER_SOURCE_AREA: Rect = Rect::new(22, 2, 9, 2);
const UNRELATED_AREA: Rect = Rect::new(22, 6, 9, 2);
const TIP_ANCHOR: Anchor = Anchor::Rect {
    rect: SOURCE_AREA,
    side: termrock_runtime::Side::Below,
    align: termrock_runtime::CrossAlign::Start,
};
const TIP_SIZE: LayerSize = LayerSize::Fixed(8, 2);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Drawer {
    #[default]
    ExactToken,
    RawId,
    Omitted,
}

struct Probe {
    source_visible: bool,
    source_part_visible: bool,
    sibling_part_visible: bool,
    source_disabled: bool,
    lower_source_visible: bool,
    unrelated_visible: bool,
    open_underlay: bool,
    underlay_opening: Option<LayerOpening>,
    draw_underlay_after_tip: bool,
    open_parent: bool,
    parent_opening: Option<LayerOpening>,
    open_on_hover: bool,
    open_on_tick: bool,
    drawer: Drawer,
    tip_id: Id,
    opening: Option<LayerOpening>,
    old_opening: Option<LayerOpening>,
    reopen_same_id: bool,
    reject_open_attempts: bool,
    try_wrong_parent: bool,
    wrong_parent_rejected: Option<bool>,
    sibling_source_rejected: Option<bool>,
    missing_part_rejected: Option<bool>,
    refresh_after_bad_sources: Option<bool>,
    close_ancestor_and_open_modal: bool,
    ancestor_modal_opening: Option<LayerOpening>,
    ancestor_stale_close: Option<bool>,
    open_after_escape: bool,
    escape_receipt_seen: bool,
    escape_replacement: Option<LayerOpening>,
    escape_stale_close: Option<bool>,
    open_modal: bool,
    foreign_opening: Option<LayerOpening>,
    probe_foreign_token: bool,
    stale_refresh: Option<bool>,
    stale_close: Option<bool>,
    current_refresh: Option<bool>,
    collision_rejected: Option<bool>,
    missing_source_rejected: Option<bool>,
    empty_size_rejected: Option<bool>,
    current_open_after_rejections: Option<bool>,
    foreign_refresh: Option<bool>,
    foreign_close: Option<bool>,
    modal_opening: Option<LayerOpening>,
    presses: Vec<Phase>,
    sibling_presses: Vec<Phase>,
    unrelated_presses: Vec<Phase>,
    layer_receipts: Vec<(LayerOpening, LayerEvent)>,
    hover_suppressed_samples: Vec<bool>,
    raw_draw_calls: Cell<usize>,
    exact_draw_calls: Cell<usize>,
    stale_draw_calls: Cell<usize>,
    foreign_draw_calls: Cell<usize>,
    reference_draw_calls: Cell<usize>,
    reference_layer_for: bool,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            source_visible: true,
            source_part_visible: true,
            sibling_part_visible: true,
            source_disabled: false,
            lower_source_visible: false,
            unrelated_visible: false,
            open_underlay: false,
            underlay_opening: None,
            draw_underlay_after_tip: false,
            open_parent: false,
            parent_opening: None,
            open_on_hover: false,
            open_on_tick: false,
            drawer: Drawer::ExactToken,
            tip_id: TIP,
            opening: None,
            old_opening: None,
            reopen_same_id: false,
            reject_open_attempts: false,
            try_wrong_parent: false,
            wrong_parent_rejected: None,
            sibling_source_rejected: None,
            missing_part_rejected: None,
            refresh_after_bad_sources: None,
            close_ancestor_and_open_modal: false,
            ancestor_modal_opening: None,
            ancestor_stale_close: None,
            open_after_escape: false,
            escape_receipt_seen: false,
            escape_replacement: None,
            escape_stale_close: None,
            open_modal: false,
            foreign_opening: None,
            probe_foreign_token: false,
            stale_refresh: None,
            stale_close: None,
            current_refresh: None,
            collision_rejected: None,
            missing_source_rejected: None,
            empty_size_rejected: None,
            current_open_after_rejections: None,
            foreign_refresh: None,
            foreign_close: None,
            modal_opening: None,
            presses: Vec::new(),
            sibling_presses: Vec::new(),
            unrelated_presses: Vec::new(),
            layer_receipts: Vec::new(),
            hover_suppressed_samples: Vec::new(),
            raw_draw_calls: Cell::new(0),
            exact_draw_calls: Cell::new(0),
            stale_draw_calls: Cell::new(0),
            foreign_draw_calls: Cell::new(0),
            reference_draw_calls: Cell::new(0),
            reference_layer_for: false,
        }
    }
}

impl Probe {
    fn open_tip(&mut self, cx: &mut Cx<'_>) {
        self.tip_id = TIP;
        self.opening = cx.open_hover_tooltip(TIP, SOURCE, SOURCE_PART, TIP_ANCHOR, TIP_SIZE);
    }

    fn register_source_owner(&self, ui: &mut Ui<'_>) {
        if !self.source_visible {
            return;
        }
        let focusability = if self.source_disabled {
            Focusability::Disabled
        } else {
            Focusability::Focusable
        };
        ui.register_control(SOURCE, SOURCE_AREA, focusability);
        if self.source_part_visible {
            ui.register_part(SOURCE, SOURCE_PART, SOURCE_AREA);
        }
        if self.sibling_part_visible {
            ui.register_part(SOURCE, SIBLING_PART, SIBLING_AREA);
        }
    }

    fn register_page_auxiliaries(&self, ui: &mut Ui<'_>) {
        if self.lower_source_visible {
            ui.register_control(LOWER_SOURCE, LOWER_SOURCE_AREA, Focusability::Focusable);
            ui.register_part(LOWER_SOURCE, LOWER_SOURCE_PART, LOWER_SOURCE_AREA);
        }
        if self.unrelated_visible {
            ui.register_control(UNRELATED, UNRELATED_AREA, Focusability::Focusable);
            ui.register_part(UNRELATED, UNRELATED_PART, UNRELATED_AREA);
        }
    }

    fn draw_underlay(&self, ui: &mut Ui<'_>) {
        if let Some(opening) = self.underlay_opening {
            let _ = ui.layer_for(opening, |ui, area| {
                ui.paint_str(area, "BASE", Style::default());
            });
        }
    }

    fn paint_tip(&self, ui: &mut Ui<'_>, area: Rect, owner: Id) {
        ui.paint_str(
            Rect::new(area.x, area.y, area.width.min(3), area.height.min(1)),
            "TIP",
            Style::default(),
        );
        ui.register_hover_part(owner, PartRef::of(Part::LABEL), area);
    }
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.hover_suppressed_samples.push(cx.hover_suppressed());
        if cx.update_cause() == UpdateCause::Bootstrap && self.open_underlay {
            let mut spec = LayerSpec::tooltip(UNDERLAY, Position::new(0, 0))
                .dismiss(termrock_runtime::Dismiss::NONE)
                .tooltip_pointer_transparent(true);
            spec.size = LayerSize::Fixed(5, 1);
            self.underlay_opening = cx.open_layer_with_token(UNDERLAY, spec);
        }
        if cx.update_cause() == UpdateCause::Bootstrap && self.open_parent {
            self.parent_opening = cx.open_layer_with_token(
                PARENT,
                LayerSpec::popover(
                    PARENT,
                    Anchor::Screen(termrock_runtime::ScreenAlign::Center),
                )
                .dismiss(termrock_runtime::Dismiss::NONE),
            );
        }
        let mut source_move = false;
        for intent in cx.intents(SOURCE) {
            if let Intent::Pointer { phase, part, .. } = intent {
                if part == SOURCE_PART && phase == Phase::Press {
                    self.presses.push(phase);
                }
                if part == SIBLING_PART && phase == Phase::Press {
                    self.sibling_presses.push(phase);
                }
                source_move |= part == SOURCE_PART && phase == Phase::Move;
            }
        }
        for intent in cx.intents(UNRELATED) {
            if let Intent::Pointer { phase, part, .. } = intent
                && part == UNRELATED_PART
                && phase == Phase::Press
            {
                self.unrelated_presses.push(phase);
            }
        }
        let mut saw_tip_escape = false;
        for id in [TIP, NEXT_TIP, UNDERLAY, PARENT, MODAL] {
            while let Some(receipt) = cx.layer_event_receipt(id) {
                let opening = receipt.opening();
                let event = receipt.event();
                saw_tip_escape |= self.opening == Some(opening)
                    && event == LayerEvent::Dismissed(DismissReason::Esc);
                self.layer_receipts.push((opening, event));
            }
        }

        if self.open_on_hover && self.opening.is_none() && source_move {
            self.open_tip(cx);
        }
        if self.open_on_tick {
            self.open_on_tick = false;
            self.open_tip(cx);
        }
        if self.reopen_same_id {
            self.reopen_same_id = false;
            let old = self.opening.expect("prior managed opening");
            self.current_refresh = Some(cx.refresh_hover_tooltip(old, TIP_ANCHOR, TIP_SIZE));
            assert!(cx.close_hover_tooltip(old));
            let replacement = cx
                .open_hover_tooltip(TIP, SOURCE, SOURCE_PART, TIP_ANCHOR, TIP_SIZE)
                .expect("same ID reopens with a new generation");
            self.tip_id = TIP;
            self.old_opening = Some(old);
            self.opening = Some(replacement);
            self.stale_refresh = Some(cx.refresh_hover_tooltip(old, TIP_ANCHOR, TIP_SIZE));
            self.stale_close = Some(cx.close_hover_tooltip(old));
            self.collision_rejected = Some(
                cx.open_layer_with_token(TIP, LayerSpec::modal(TIP))
                    .is_none(),
            );
            self.current_open_after_rejections = Some(cx.is_open(TIP));
        }
        if self.reject_open_attempts {
            self.reject_open_attempts = false;
            self.missing_source_rejected = Some(
                cx.open_hover_tooltip(
                    Id::root("hover-tip.missing"),
                    Id::root("hover-tip.no-owner"),
                    SOURCE_PART,
                    TIP_ANCHOR,
                    TIP_SIZE,
                )
                .is_none(),
            );
            self.empty_size_rejected = Some(
                cx.open_hover_tooltip(
                    Id::root("hover-tip.empty"),
                    SOURCE,
                    SOURCE_PART,
                    TIP_ANCHOR,
                    LayerSize::Fixed(0, 2),
                )
                .is_none(),
            );
            self.missing_part_rejected = Some(
                cx.open_hover_tooltip(
                    Id::root("hover-tip.missing-part"),
                    SOURCE,
                    MISSING_PART,
                    TIP_ANCHOR,
                    TIP_SIZE,
                )
                .is_none(),
            );
            self.sibling_source_rejected = Some(
                cx.open_hover_tooltip(TIP, SOURCE, SIBLING_PART, TIP_ANCHOR, TIP_SIZE)
                    .is_none(),
            );
            if self.try_wrong_parent {
                self.wrong_parent_rejected = Some(
                    cx.open_hover_tooltip(
                        NEXT_TIP,
                        LOWER_SOURCE,
                        LOWER_SOURCE_PART,
                        TIP_ANCHOR,
                        TIP_SIZE,
                    )
                    .is_none(),
                );
            }
            if let Some(opening) = self.opening {
                self.refresh_after_bad_sources =
                    Some(cx.refresh_hover_tooltip(opening, TIP_ANCHOR, TIP_SIZE));
            }
            self.current_open_after_rejections = Some(cx.is_open(TIP));
        }
        if self.probe_foreign_token {
            self.probe_foreign_token = false;
            let foreign = self.foreign_opening.expect("foreign opening configured");
            self.foreign_refresh = Some(cx.refresh_hover_tooltip(foreign, TIP_ANCHOR, TIP_SIZE));
            self.foreign_close = Some(cx.close_hover_tooltip(foreign));
        }
        if self.open_modal {
            self.open_modal = false;
            self.modal_opening = cx.open_layer_with_token(MODAL, LayerSpec::modal(MODAL));
            if let Some(old) = self.opening {
                self.stale_close = Some(cx.close_hover_tooltip(old));
            }
        }
        if self.close_ancestor_and_open_modal {
            self.close_ancestor_and_open_modal = false;
            let old = self.opening.expect("managed opening below raw ancestor");
            cx.close_layer(UNDERLAY, None);
            self.ancestor_modal_opening = cx.open_layer_with_token(MODAL, LayerSpec::modal(MODAL));
            self.ancestor_stale_close = Some(cx.close_hover_tooltip(old));
        }
        if self.open_after_escape && saw_tip_escape {
            self.open_after_escape = false;
            self.escape_receipt_seen = true;
            let old = self
                .opening
                .expect("Escape closed the current managed opening");
            let replacement = cx
                .open_hover_tooltip(NEXT_TIP, SOURCE, SOURCE_PART, TIP_ANCHOR, TIP_SIZE)
                .expect("Escape cleared the managed slot before this update");
            self.tip_id = NEXT_TIP;
            self.escape_replacement = Some(replacement);
            self.opening = Some(replacement);
            self.escape_stale_close = Some(cx.close_hover_tooltip(old));
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if !self.draw_underlay_after_tip {
            self.draw_underlay(ui);
        }
        self.register_page_auxiliaries(ui);
        if let Some(parent) = self.parent_opening {
            let _ = ui.layer_for(parent, |ui, _| self.register_source_owner(ui));
        } else {
            self.register_source_owner(ui);
        }

        if let Some(stale) = self.old_opening {
            let _ = ui.layer_for(stale, |ui, area| {
                self.stale_draw_calls.set(self.stale_draw_calls.get() + 1);
                self.paint_tip(ui, area, TIP);
            });
        }
        if let Some(foreign) = self.foreign_opening {
            let _ = ui.layer_for(foreign, |ui, area| {
                self.foreign_draw_calls
                    .set(self.foreign_draw_calls.get() + 1);
                self.paint_tip(ui, area, TIP);
            });
        }

        if let Some(opening) = self.opening {
            match self.drawer {
                Drawer::ExactToken => {
                    let _ = ui.layer_for(opening, |ui, area| {
                        self.exact_draw_calls.set(self.exact_draw_calls.get() + 1);
                        self.paint_tip(ui, area, self.tip_id);
                    });
                }
                Drawer::RawId => {
                    let _ = ui.layer(self.tip_id, |ui, area| {
                        self.raw_draw_calls.set(self.raw_draw_calls.get() + 1);
                        self.paint_tip(ui, area, self.tip_id);
                    });
                }
                Drawer::Omitted => {}
            }
        }
        if self.draw_underlay_after_tip {
            self.draw_underlay(ui);
        }
        if self.reference_layer_for
            && let Some(opening) = self.opening
        {
            ui.reference(None, |reference| {
                let _ = reference.layer_for(opening, |ui, area| {
                    self.reference_draw_calls
                        .set(self.reference_draw_calls.get() + 1);
                    self.paint_tip(ui, area, self.tip_id);
                });
            });
        }
        if self.modal_opening.is_some() || self.ancestor_modal_opening.is_some() {
            let _ = ui.layer(MODAL, |ui, area| {
                ui.paint_str(area, "MODAL", Style::default());
            });
        }
    }
}

fn present<A: App>(runtime: &mut Runtime<A>, buffer: &mut Buffer) {
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            runtime.draw_buffer(SCREEN, buffer).commit_presented();
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!("runtime did not settle and present");
}

fn ready(probe: Probe) -> (Runtime<Probe>, Buffer) {
    let mut runtime = Runtime::new(probe, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(SCREEN);
    present(&mut runtime, &mut buffer);
    (runtime, buffer)
}

fn input_tick(runtime: &mut Runtime<Probe>, buffer: &mut Buffer) {
    // app_mut invalidates presentation, and Runtime::handle requires a presented frame.
    present(runtime, buffer);
    let _ = runtime.handle(Input::Tick).unwrap();
    present(runtime, buffer);
}

fn mouse(kind: MouseKind, x: u16, y: u16) -> Input {
    Input::Mouse(Mouse {
        kind,
        pos: Position::new(x, y),
        mods: KeyModifiers::NONE,
    })
}

fn has_dismissal(probe: &Probe, opening: LayerOpening, reason: DismissReason) -> bool {
    probe
        .layer_receipts
        .iter()
        .any(|(receipt, event)| *receipt == opening && *event == LayerEvent::Dismissed(reason))
}

fn open_managed(runtime: &mut Runtime<Probe>, buffer: &mut Buffer) {
    runtime.app_mut().open_on_tick = true;
    input_tick(runtime, buffer);
    let opening = runtime.app().opening.expect("managed opening admitted");
    assert!(runtime.is_open(TIP));
    assert_eq!(runtime.top_layer(), opening.layer_id());
}

#[test]
fn exact_token_open_refresh_and_same_id_replacement_reject_stale_tokens() {
    let (mut runtime, mut buffer) = ready(Probe::default());
    open_managed(&mut runtime, &mut buffer);
    runtime.app_mut().reopen_same_id = true;
    input_tick(&mut runtime, &mut buffer);

    let app = runtime.app();
    let old = app.old_opening.expect("retired opening saved");
    let replacement = app.opening.expect("replacement opening saved");
    assert_ne!(old, replacement);
    assert_eq!(app.stale_refresh, Some(false));
    assert_eq!(app.stale_close, Some(false));
    assert_eq!(app.current_refresh, Some(true));
    assert_eq!(app.collision_rejected, Some(true));
    assert_eq!(app.current_open_after_rejections, Some(true));
    assert!(runtime.is_open(TIP));
    assert_eq!(runtime.top_layer(), replacement.layer_id());

    runtime.app_mut().reject_open_attempts = true;
    input_tick(&mut runtime, &mut buffer);
    assert_eq!(runtime.app().missing_source_rejected, Some(true));
    assert_eq!(runtime.app().empty_size_rejected, Some(true));
    assert_eq!(runtime.app().missing_part_rejected, Some(true));
    assert_eq!(runtime.app().sibling_source_rejected, Some(true));
    assert_eq!(runtime.app().refresh_after_bad_sources, Some(true));
    assert_eq!(runtime.app().current_open_after_rejections, Some(true));
    assert!(runtime.is_open(TIP));
}

#[test]
fn foreign_runtime_tokens_cannot_refresh_or_close_the_current_managed_tip() {
    let (mut source_runtime, mut source_buffer) = ready(Probe::default());
    open_managed(&mut source_runtime, &mut source_buffer);
    let foreign = source_runtime.app().opening.expect("foreign opening");

    let (mut runtime, mut buffer) = ready(Probe::default());
    open_managed(&mut runtime, &mut buffer);
    runtime.app_mut().foreign_opening = Some(foreign);
    runtime.app_mut().probe_foreign_token = true;
    input_tick(&mut runtime, &mut buffer);

    assert_eq!(runtime.app().foreign_refresh, Some(false));
    assert_eq!(runtime.app().foreign_close, Some(false));
    assert!(runtime.is_open(TIP));
    assert_eq!(
        runtime.top_layer(),
        runtime.app().opening.unwrap().layer_id()
    );
}

#[test]
fn publication_retires_omitted_or_raw_draws_but_discarded_draw_does_not() {
    let probe = Probe {
        open_underlay: true,
        ..Probe::default()
    };
    let (mut runtime, mut buffer) = ready(probe);
    open_managed(&mut runtime, &mut buffer);

    runtime.app_mut().drawer = Drawer::Omitted;
    drop(runtime.draw_buffer(SCREEN, &mut buffer));
    assert!(
        runtime.is_open(TIP),
        "discarded drawing cannot retire a live tip"
    );
    present(&mut runtime, &mut buffer);
    assert!(!runtime.is_open(TIP), "published omission retires the tip");
    assert!(runtime.is_open(UNDERLAY));
    assert_eq!(
        runtime.top_layer(),
        runtime.app().underlay_opening.unwrap().layer_id()
    );

    runtime.app_mut().drawer = Drawer::ExactToken;
    open_managed(&mut runtime, &mut buffer);
    runtime.app_mut().drawer = Drawer::RawId;
    present(&mut runtime, &mut buffer);
    assert_eq!(runtime.app().raw_draw_calls.get(), 0);
    assert!(
        !runtime.is_open(TIP),
        "ID-only drawing cannot retain the tip"
    );

    runtime.app_mut().drawer = Drawer::ExactToken;
    open_managed(&mut runtime, &mut buffer);
    runtime.app_mut().source_visible = false;
    present(&mut runtime, &mut buffer);
    assert!(
        !runtime.is_open(TIP),
        "missing source registration retires the tip"
    );
}

#[test]
fn admitted_modal_preempts_tip_and_stale_close_cannot_drain_it() {
    let (mut runtime, mut buffer) = ready(Probe::default());
    open_managed(&mut runtime, &mut buffer);
    let old = runtime.app().opening.unwrap();
    runtime.app_mut().open_modal = true;
    input_tick(&mut runtime, &mut buffer);

    assert_eq!(runtime.app().stale_close, Some(false));
    assert!(!runtime.is_open(TIP));
    assert!(runtime.is_open(MODAL));
    assert_eq!(
        runtime.top_layer(),
        runtime.app().modal_opening.unwrap().layer_id()
    );
    assert_ne!(runtime.app().modal_opening.unwrap(), old);
}

#[test]
fn source_press_retires_before_activation_and_disabled_root_refusal() {
    let (mut enabled, mut enabled_buffer) = ready(Probe::default());
    open_managed(&mut enabled, &mut enabled_buffer);
    let enabled_opening = enabled.app().opening.unwrap();
    let _ = enabled.handle(mouse(MouseKind::Down, 4, 2)).unwrap();
    present(&mut enabled, &mut enabled_buffer);
    assert!(!enabled.is_open(TIP));
    assert_eq!(enabled.app().presses, vec![Phase::Press]);
    assert!(has_dismissal(
        enabled.app(),
        enabled_opening,
        DismissReason::OutsideClick,
    ));

    let disabled_probe = Probe {
        source_disabled: true,
        ..Probe::default()
    };
    let (mut disabled, mut disabled_buffer) = ready(disabled_probe);
    open_managed(&mut disabled, &mut disabled_buffer);
    let disabled_opening = disabled.app().opening.unwrap();
    let _ = disabled.handle(mouse(MouseKind::Down, 4, 2)).unwrap();
    present(&mut disabled, &mut disabled_buffer);
    assert!(!disabled.is_open(TIP));
    assert!(
        disabled.app().presses.is_empty(),
        "disabled source gets no fake Press"
    );
    assert!(has_dismissal(
        disabled.app(),
        disabled_opening,
        DismissReason::OutsideClick,
    ));
}

#[test]
fn layer_for_requires_exact_token_and_reference_cannot_change_live_marker() {
    let probe = Probe {
        open_underlay: true,
        draw_underlay_after_tip: true,
        ..Probe::default()
    };
    let (mut runtime, mut buffer) = ready(probe);
    open_managed(&mut runtime, &mut buffer);
    runtime.app_mut().reopen_same_id = true;
    input_tick(&mut runtime, &mut buffer);
    let current = runtime.app().opening.expect("replacement is current");
    let stale = runtime.app().old_opening.expect("old generation retained");

    let (mut foreign_runtime, mut foreign_buffer) = ready(Probe::default());
    open_managed(&mut foreign_runtime, &mut foreign_buffer);
    let foreign = foreign_runtime
        .app()
        .opening
        .expect("foreign runtime token");

    runtime.app_mut().foreign_opening = Some(foreign);
    runtime.app_mut().reference_layer_for = true;
    present(&mut runtime, &mut buffer);

    let app = runtime.app();
    assert_eq!(app.stale_draw_calls.get(), 0);
    assert_eq!(app.foreign_draw_calls.get(), 0);
    assert_eq!(app.reference_draw_calls.get(), 0);
    assert!(app.exact_draw_calls.get() > 0, "the current token paints");
    assert!(runtime.is_open(TIP));
    assert!(runtime.is_open(UNDERLAY));
    assert_eq!(runtime.app().opening, Some(current));
    assert_eq!(runtime.top_layer(), current.layer_id());
    assert!(
        runtime
            .registry()
            .regions()
            .iter()
            .any(|region| { region.owner == TIP && region.part == PartRef::of(Part::LABEL) })
    );
    assert_ne!(stale, current);
}

#[test]
fn projection_omission_and_stale_draw_do_not_retire_live_managed_tip() {
    let (mut runtime, mut live_buffer) = ready(Probe::default());
    open_managed(&mut runtime, &mut live_buffer);
    runtime.app_mut().reopen_same_id = true;
    input_tick(&mut runtime, &mut live_buffer);
    let current = runtime.app().opening.expect("current managed token");
    let stale = runtime.app().old_opening.expect("stale same-ID token");
    let snapshot = runtime.render_snapshot().expect("live frame is settled");
    let source = runtime
        .registry()
        .regions()
        .iter()
        .find(|region| region.owner == SOURCE && region.part == SOURCE_PART)
        .copied()
        .expect("current exact source registration");

    let mut projection = Buffer::empty(SCREEN);
    runtime
        .draw_projection(SCREEN, &mut projection, &snapshot, |_ui, _area| {})
        .commit_inspected();
    assert!(
        runtime.is_open(TIP),
        "projection omission is not publication"
    );
    assert_eq!(runtime.app().opening, Some(current));
    assert!(runtime.registry().regions().contains(&source));

    let mut projected_tip_draws = 0;
    runtime
        .draw_projection(SCREEN, &mut projection, &snapshot, |ui, _area| {
            let _ = ui.layer_for(current, |ui, area| {
                projected_tip_draws += 1;
                ui.paint_str(area, "STALE", Style::default());
            });
        })
        .commit_inspected();
    assert_eq!(
        projected_tip_draws, 1,
        "the projection can paint its snapshot"
    );
    assert!(runtime.is_open(TIP));
    assert_eq!(runtime.app().opening, Some(current));
    assert!(runtime.registry().regions().contains(&source));

    let mut stale_projection_draws = 0;
    runtime
        .draw_projection(SCREEN, &mut projection, &snapshot, |ui, _area| {
            let _ = ui.layer_for(stale, |_, _| stale_projection_draws += 1);
        })
        .commit_inspected();
    assert_eq!(stale_projection_draws, 0);
    assert!(runtime.is_open(TIP));
    assert_eq!(runtime.top_layer(), current.layer_id());
    assert_eq!(runtime.app().opening, Some(current));
    assert!(runtime.registry().regions().contains(&source));

    present(&mut runtime, &mut live_buffer);
    assert!(
        runtime.is_open(TIP),
        "a later live exact draw still succeeds"
    );
    assert_eq!(runtime.app().opening, Some(current));
    assert_eq!(runtime.top_layer(), current.layer_id());
}

#[test]
fn sibling_and_unrelated_owner_presses_do_not_dismiss_the_source_tip() {
    let probe = Probe {
        unrelated_visible: true,
        ..Probe::default()
    };
    let (mut runtime, mut buffer) = ready(probe);
    open_managed(&mut runtime, &mut buffer);
    let opening = runtime.app().opening.expect("active source tip");

    let _ = runtime.handle(mouse(MouseKind::Down, 14, 2)).unwrap();
    present(&mut runtime, &mut buffer);
    assert_eq!(runtime.app().sibling_presses, vec![Phase::Press]);
    assert!(runtime.app().presses.is_empty());
    assert!(runtime.is_open(TIP));
    assert!(!has_dismissal(
        runtime.app(),
        opening,
        DismissReason::OutsideClick,
    ));
    let _ = runtime.handle(mouse(MouseKind::Up, 14, 2)).unwrap();
    present(&mut runtime, &mut buffer);

    let _ = runtime.handle(mouse(MouseKind::Down, 24, 6)).unwrap();
    present(&mut runtime, &mut buffer);
    assert_eq!(runtime.app().unrelated_presses, vec![Phase::Press]);
    assert!(runtime.app().presses.is_empty());
    assert!(runtime.is_open(TIP));
    assert!(!has_dismissal(
        runtime.app(),
        opening,
        DismissReason::OutsideClick,
    ));
    let _ = runtime.handle(mouse(MouseKind::Up, 24, 6)).unwrap();
    present(&mut runtime, &mut buffer);
}

#[test]
fn removing_only_the_bound_source_part_retires_tip_but_keeps_owner_and_sibling() {
    let (mut runtime, mut buffer) = ready(Probe::default());
    open_managed(&mut runtime, &mut buffer);
    runtime.app_mut().source_part_visible = false;
    present(&mut runtime, &mut buffer);

    assert!(!runtime.is_open(TIP));
    assert!(runtime.registry().has_owner(SOURCE));
    assert!(
        runtime
            .registry()
            .regions()
            .iter()
            .any(|region| { region.owner == SOURCE && region.part == SIBLING_PART })
    );
    assert!(
        !runtime
            .registry()
            .regions()
            .iter()
            .any(|region| { region.owner == SOURCE && region.part == SOURCE_PART })
    );
}

#[test]
fn wrong_parent_and_missing_part_rejections_preserve_valid_opening() {
    let probe = Probe {
        open_parent: true,
        lower_source_visible: true,
        ..Probe::default()
    };
    let (mut runtime, mut buffer) = ready(probe);
    open_managed(&mut runtime, &mut buffer);
    let current = runtime.app().opening.expect("valid parent source tip");
    let parent = runtime.app().parent_opening.expect("opaque parent layer");

    runtime.app_mut().reject_open_attempts = true;
    runtime.app_mut().try_wrong_parent = true;
    input_tick(&mut runtime, &mut buffer);

    assert_eq!(runtime.app().wrong_parent_rejected, Some(true));
    assert_eq!(runtime.app().missing_part_rejected, Some(true));
    assert_eq!(runtime.app().sibling_source_rejected, Some(true));
    assert_eq!(runtime.app().refresh_after_bad_sources, Some(true));
    assert_eq!(runtime.app().current_open_after_rejections, Some(true));
    assert!(runtime.is_open(TIP));
    assert_eq!(runtime.top_layer(), current.layer_id());
    assert!(runtime.registry().regions().iter().any(|region| {
        region.owner == SOURCE && region.part == SOURCE_PART && region.layer == parent.layer_id()
    }));
    assert!(runtime.registry().regions().iter().any(|region| {
        region.owner == LOWER_SOURCE
            && region.part == LOWER_SOURCE_PART
            && region.layer == LayerId::PAGE
    }));
}

#[test]
fn escape_clears_managed_slot_before_new_tip_and_stale_close_is_harmless() {
    let (mut runtime, mut buffer) = ready(Probe::default());
    open_managed(&mut runtime, &mut buffer);
    let old = runtime.app().opening.expect("first tip");
    runtime.app_mut().open_after_escape = true;
    present(&mut runtime, &mut buffer);
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, &mut buffer);

    let next = runtime.app().escape_replacement.expect("new managed tip");
    assert!(runtime.app().escape_receipt_seen);
    assert_eq!(runtime.app().escape_stale_close, Some(false));
    assert_ne!(old, next);
    assert!(!runtime.is_open(TIP));
    assert!(runtime.is_open(NEXT_TIP));
    assert_eq!(runtime.top_layer(), next.layer_id());
    assert!(has_dismissal(runtime.app(), old, DismissReason::Esc));
}

#[test]
fn raw_ancestor_close_clears_tip_slot_before_admitting_modal() {
    let probe = Probe {
        open_underlay: true,
        ..Probe::default()
    };
    let (mut runtime, mut buffer) = ready(probe);
    open_managed(&mut runtime, &mut buffer);
    let old = runtime.app().opening.expect("managed child tip");
    runtime.app_mut().close_ancestor_and_open_modal = true;
    input_tick(&mut runtime, &mut buffer);

    let modal = runtime
        .app()
        .ancestor_modal_opening
        .expect("modal admitted after ancestor close");
    assert_eq!(runtime.app().ancestor_stale_close, Some(false));
    assert!(!runtime.is_open(TIP));
    assert!(!runtime.is_open(UNDERLAY));
    assert!(runtime.is_open(MODAL));
    assert_eq!(runtime.top_layer(), modal.layer_id());
    assert_ne!(old, modal);
}

#[test]
fn hover_suppression_reports_the_native_snapshot_bit() {
    let probe = Probe {
        open_on_hover: true,
        ..Probe::default()
    };
    let (mut runtime, mut buffer) = ready(probe);
    let _ = runtime.handle(mouse(MouseKind::Move, 4, 2)).unwrap();
    present(&mut runtime, &mut buffer);
    assert!(runtime.is_open(TIP), "native hover opens the managed tip");

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, &mut buffer);
    assert!(
        runtime
            .app()
            .hover_suppressed_samples
            .iter()
            .any(|suppressed| *suppressed)
    );
}
