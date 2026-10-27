use ratatui_core::layout::{Constraint, Position, Rect};

pub use termrock_core::layer_types::{DismissReason, LayerEvent, LayerId};

use crate::focus::ScopeId;
use crate::id::Id;
use crate::intent::LayerIntentToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UpdateToken {
    runtime: u64,
    callback: u64,
}

impl UpdateToken {
    pub(crate) const fn new(runtime: u64, callback: u64) -> Self {
        Self { runtime, callback }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerEventReceipt {
    pub(crate) opening: LayerOpening,
    pub(crate) event: LayerEvent,
    pub(crate) update: Option<UpdateToken>,
}

impl LayerEventReceipt {
    pub const fn opening(self) -> LayerOpening {
        self.opening
    }

    pub const fn event(self) -> LayerEvent {
        self.event
    }

    pub const fn update(self) -> Option<UpdateToken> {
        self.update
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PendingLayerEvent {
    pub(crate) opening: LayerOpening,
    pub(crate) event: LayerEvent,
    pub(crate) update: Option<UpdateToken>,
}

#[derive(Debug)]
pub(crate) struct RoutedLayerEvent {
    receipt: LayerEventReceipt,
    token: LayerIntentToken,
}

impl RoutedLayerEvent {
    pub(crate) fn new(receipt: LayerEventReceipt, token: LayerIntentToken) -> Self {
        Self { receipt, token }
    }

    pub(crate) const fn receipt(&self) -> LayerEventReceipt {
        self.receipt
    }

    pub(crate) const fn opening(&self) -> LayerOpening {
        self.receipt.opening
    }

    pub(crate) const fn event(&self) -> LayerEvent {
        self.receipt.event
    }

    pub(crate) fn token(&self) -> &LayerIntentToken {
        &self.token
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerOpening {
    runtime: u64,
    generation: u64,
    id: Id,
    owner: Id,
    layer: LayerId,
    parent: Option<LayerParent>,
}

impl LayerOpening {
    const fn new(
        runtime: u64,
        generation: u64,
        id: Id,
        owner: Id,
        layer: LayerId,
        parent: Option<LayerParent>,
    ) -> Self {
        Self {
            runtime,
            generation,
            id,
            owner,
            layer,
            parent,
        }
    }

    pub const fn owner(self) -> Id {
        self.owner
    }

    pub const fn layer_id(self) -> LayerId {
        self.layer
    }

    pub(crate) const fn runtime(self) -> u64 {
        self.runtime
    }

    pub(crate) const fn generation(self) -> u64 {
        self.generation
    }

    pub(crate) const fn id(self) -> Id {
        self.id
    }

    pub(crate) const fn parent(self) -> Option<LayerParent> {
        self.parent
    }
}

impl core::fmt::Debug for LayerOpening {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LayerOpening")
            .field("id", &self.id)
            .field("owner", &self.owner)
            .field("layer", &self.layer)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerCompletion {
    opening: LayerOpening,
    event: LayerEvent,
    restored_focus: Option<Id>,
    continuation_owner: Option<Id>,
    continuation_acknowledged: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusContinuationOutcome {
    opening: LayerOpening,
    owner: Id,
    restored_focus: Option<Id>,
    continued: bool,
}

impl FocusContinuationOutcome {
    pub(crate) const fn new(
        opening: LayerOpening,
        owner: Id,
        restored_focus: Option<Id>,
        continued: bool,
    ) -> Self {
        Self {
            opening,
            owner,
            restored_focus,
            continued,
        }
    }

    pub const fn opening(self) -> LayerOpening {
        self.opening
    }
    pub const fn owner(self) -> Id {
        self.owner
    }
    pub const fn restored_focus(self) -> Option<Id> {
        self.restored_focus
    }
    pub const fn continued(self) -> bool {
        self.continued
    }

    pub(crate) const fn with_continued(mut self, continued: bool) -> Self {
        self.continued = continued;
        self
    }
}

impl LayerCompletion {
    pub(crate) const fn new(
        opening: LayerOpening,
        event: LayerEvent,
        restored_focus: Option<Id>,
        continuation_owner: Option<Id>,
        continuation_acknowledged: bool,
    ) -> Self {
        Self {
            opening,
            event,
            restored_focus,
            continuation_owner,
            continuation_acknowledged,
        }
    }

    pub const fn opening(self) -> LayerOpening {
        self.opening
    }

    pub const fn event(self) -> LayerEvent {
        self.event
    }

    pub const fn restored_focus(self) -> Option<Id> {
        self.restored_focus
    }

    pub const fn continuation_owner(self) -> Option<Id> {
        self.continuation_owner
    }

    pub const fn continuation_acknowledged(self) -> bool {
        self.continuation_acknowledged
    }

    pub(crate) const fn with_continuation_acknowledged(mut self, acknowledged: bool) -> Self {
        self.continuation_acknowledged = acknowledged;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LayerParent {
    pub(crate) generation: u64,
    pub(crate) id: Id,
    pub(crate) layer: LayerId,
}

impl From<LayerOpening> for LayerParent {
    fn from(opening: LayerOpening) -> Self {
        Self {
            generation: opening.generation,
            id: opening.id,
            layer: opening.layer,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerKind {
    Modal,

    Popover,

    Tooltip,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anchor {
    Screen(ScreenAlign),

    Rect {
        rect: Rect,

        side: Side,

        align: CrossAlign,
    },

    Point(Position),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenAlign {
    Center,

    GeometricCenter,

    Balanced,

    UpperThird,

    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Below,

    Above,

    Left,

    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrossAlign {
    Start,

    Center,

    End,
}

/// Ported from the PR #26 reference popup vocabulary
/// (`src/ui/popup.rs` at f4867e9b5f8a08a3bb61ce80d644198a29f51984).

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RefuseFeedback {
    #[default]
    Flash,

    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Dismiss {
    pub esc: bool,

    pub outside_click: bool,

    pub focus_out: bool,

    pub pass_through: bool,

    pub close_button: bool,

    pub refuse_feedback: RefuseFeedback,
}

impl Dismiss {
    pub const NONE: Dismiss = Dismiss {
        esc: false,
        outside_click: false,
        focus_out: false,
        pass_through: false,
        close_button: false,
        refuse_feedback: RefuseFeedback::Flash,
    };

    pub const ESC: Dismiss = Dismiss {
        esc: true,
        outside_click: false,
        focus_out: false,
        pass_through: false,
        close_button: true,
        refuse_feedback: RefuseFeedback::Flash,
    };

    pub const ESC_AND_OUTSIDE: Dismiss = Dismiss {
        esc: true,
        outside_click: true,
        focus_out: false,
        pass_through: false,
        close_button: true,
        refuse_feedback: RefuseFeedback::Flash,
    };

    pub const ALL: Dismiss = Dismiss {
        esc: true,
        outside_click: true,
        focus_out: true,
        pass_through: false,
        close_button: true,
        refuse_feedback: RefuseFeedback::Flash,
    };

    pub const fn pass_through(mut self, on: bool) -> Self {
        self.pass_through = on;
        self
    }

    pub const fn close_button(mut self, on: bool) -> Self {
        self.close_button = on;
        self
    }

    pub const fn refuse_feedback(mut self, feedback: RefuseFeedback) -> Self {
        self.refuse_feedback = feedback;
        self
    }

    /// Ported from the reference `Dismiss::allows`.
    pub const fn allows(self, reason: DismissReason) -> bool {
        match reason {
            DismissReason::Esc => self.esc,
            DismissReason::OutsideClick => self.outside_click,
            DismissReason::FocusOut => self.focus_out,

            _ => true,
        }
    }

    pub const fn shows_close(self) -> bool {
        self.close_button && self.esc
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backdrop {
    None,

    Dim { exclude_footer: bool },
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerSize {
    Fill,

    Fixed(u16, u16),
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LayerSpec {
    pub kind: LayerKind,

    pub owner: Id,

    pub anchor: Anchor,

    pub dismiss: Dismiss,

    pub restore_focus: bool,

    pub initial_focus: Option<Id>,

    pub size: LayerSize,

    pub viewport_margin: (u16, u16),

    pub continues_focus_owner: Option<Id>,

    pub backdrop: Backdrop,

    pub inert_below: bool,

    pointer_transparent: bool,
}

impl LayerSpec {
    pub const fn modal(owner: Id) -> LayerSpec {
        LayerSpec {
            kind: LayerKind::Modal,
            owner,
            anchor: Anchor::Screen(ScreenAlign::Center),
            dismiss: Dismiss::ESC_AND_OUTSIDE,
            restore_focus: true,
            initial_focus: None,
            size: LayerSize::Fill,
            viewport_margin: (0, 0),
            continues_focus_owner: None,
            backdrop: Backdrop::Dim {
                exclude_footer: true,
            },
            inert_below: true,
            pointer_transparent: false,
        }
    }

    pub const fn popover(owner: Id, anchor: Anchor) -> LayerSpec {
        LayerSpec {
            kind: LayerKind::Popover,
            owner,
            anchor,
            dismiss: Dismiss::ESC_AND_OUTSIDE,
            restore_focus: true,
            initial_focus: None,
            size: LayerSize::Fill,
            viewport_margin: (0, 0),
            continues_focus_owner: None,
            backdrop: Backdrop::None,
            inert_below: false,
            pointer_transparent: false,
        }
    }

    pub const fn tooltip(owner: Id, at: Position) -> LayerSpec {
        LayerSpec {
            kind: LayerKind::Tooltip,
            owner,
            anchor: Anchor::Point(at),
            dismiss: Dismiss::ALL,
            restore_focus: false,
            initial_focus: None,
            size: LayerSize::Fill,
            viewport_margin: (0, 0),
            continues_focus_owner: None,
            backdrop: Backdrop::None,
            inert_below: false,
            pointer_transparent: false,
        }
    }

    #[must_use]
    pub const fn tooltip_pointer_transparent(mut self, on: bool) -> Self {
        self.pointer_transparent = on;
        self
    }

    #[must_use]
    pub const fn anchor(mut self, a: Anchor) -> Self {
        self.anchor = a;
        self
    }

    #[must_use]
    pub const fn continues_focus(mut self, owner: Id) -> Self {
        self.continues_focus_owner = Some(owner);
        self
    }

    #[must_use]
    pub const fn dismiss(mut self, d: Dismiss) -> Self {
        self.dismiss = d;
        self
    }

    #[must_use]
    pub const fn backdrop(mut self, b: Backdrop) -> Self {
        self.backdrop = b;
        self
    }

    #[must_use]
    pub const fn initial_focus(mut self, id: Id) -> Self {
        self.initial_focus = Some(id);
        self
    }

    #[must_use]
    pub const fn size(mut self, s: LayerSize) -> Self {
        self.size = s;
        self
    }

    #[must_use]
    pub const fn viewport_margin(mut self, horizontal_total: u16, vertical_total: u16) -> Self {
        self.viewport_margin = (horizontal_total, vertical_total);
        self
    }

    #[must_use]
    pub const fn inert_below(mut self, yes: bool) -> Self {
        self.inert_below = yes;
        self
    }

    #[must_use]
    pub const fn restore_focus(mut self, yes: bool) -> Self {
        self.restore_focus = yes;
        self
    }

    pub const fn dismisses_on_focus_out(&self) -> bool {
        self.dismiss.focus_out && !matches!(self.kind, LayerKind::Modal)
    }
}

pub(crate) fn resolve_layer_area(viewport: Rect, spec: LayerSpec) -> Rect {
    let LayerSize::Fixed(desired_width, desired_height) = spec.size else {
        return resolve_anchor(viewport, spec.anchor, spec.size);
    };
    if viewport.width == 0 || viewport.height == 0 {
        return Rect::ZERO;
    }
    let max_width = viewport.width.saturating_sub(spec.viewport_margin.0).max(1);
    let max_height = viewport
        .height
        .saturating_sub(spec.viewport_margin.1)
        .max(1);
    let width = desired_width.min(max_width).min(viewport.width);
    let height = desired_height.min(max_height).min(viewport.height);
    resolve_anchor(viewport, spec.anchor, LayerSize::Fixed(width, height))
}

#[expect(
    clippy::many_single_char_names,
    reason = "w/h/x/y/p are the geometry vocabulary of the anchor arms"
)]
pub fn resolve_anchor(screen: Rect, anchor: Anchor, size: LayerSize) -> Rect {
    let (w, h) = match size {
        LayerSize::Fill => return screen,
        LayerSize::Fixed(w, h) if w == 0 || h == 0 => return Rect::ZERO,
        LayerSize::Fixed(w, h) => (w.min(screen.width), h.min(screen.height)),
    };
    let raw = match anchor {
        Anchor::Screen(align) => {
            let x = screen.centered_horizontally(Constraint::Length(w)).x;
            let free = screen.height.saturating_sub(h);
            let y = match align {
                ScreenAlign::Center => screen.y.saturating_add(free / 3),
                ScreenAlign::GeometricCenter => screen.y.saturating_add(free / 2),
                ScreenAlign::Balanced => screen.y.saturating_add(free.div_ceil(2)),
                ScreenAlign::UpperThird => screen.y.saturating_add(free / 6),
                ScreenAlign::Bottom => screen.y.saturating_add(free),
            };
            Rect {
                x,
                y,
                width: w,
                height: h,
            }
        }
        Anchor::Rect { rect, side, align } => {
            let (x, y) = match side {
                Side::Below | Side::Above => {
                    let x = cross(rect.x, rect.width, w, align);
                    let below = rect.bottom();
                    let fits_below = below.saturating_add(h) <= screen.bottom();
                    let fits_above = rect.y >= screen.y.saturating_add(h);
                    let y = match side {
                        Side::Below if fits_below || !fits_above => below,
                        Side::Above if fits_above || !fits_below => rect.y.saturating_sub(h),
                        Side::Below => rect.y.saturating_sub(h),
                        _ => below,
                    };
                    (x, y)
                }
                Side::Left | Side::Right => {
                    let y = cross(rect.y, rect.height, h, align);
                    let right = rect.right();
                    let fits_right = right.saturating_add(w) <= screen.right();
                    let fits_left = rect.x >= screen.x.saturating_add(w);
                    let x = match side {
                        Side::Right if fits_right || !fits_left => right,
                        Side::Left if fits_left || !fits_right => rect.x.saturating_sub(w),
                        Side::Right => rect.x.saturating_sub(w),
                        _ => right,
                    };
                    (x, y)
                }
            };
            Rect {
                x,
                y,
                width: w,
                height: h,
            }
        }

        Anchor::Point(p) => {
            let below = p.y.saturating_add(1);
            let fits_below = below.saturating_add(h) <= screen.bottom();
            let fits_above = p.y >= screen.y.saturating_add(h);
            let y = if fits_below || !fits_above {
                below
            } else {
                p.y.saturating_sub(h)
            };
            let fits_right = p.x.saturating_add(w) <= screen.right();
            let fits_left = p.x >= screen.x.saturating_add(w);
            let x = if fits_right || !fits_left {
                p.x
            } else {
                p.x.saturating_sub(w)
            };
            Rect {
                x,
                y,
                width: w,
                height: h,
            }
        }
    };
    raw.clamp(screen)
}

const fn cross(start: u16, len: u16, size: u16, align: CrossAlign) -> u16 {
    match align {
        CrossAlign::Start => start,
        CrossAlign::Center => start.saturating_add(len.saturating_sub(size) / 2),
        CrossAlign::End => start.saturating_add(len).saturating_sub(size),
    }
}

pub const fn backdrop_area(screen: Rect, backdrop: Backdrop) -> Rect {
    match backdrop {
        Backdrop::None => Rect::ZERO,
        Backdrop::Dim { exclude_footer } => Rect {
            x: screen.x,
            y: screen.y,
            width: screen.width,
            height: if exclude_footer {
                screen.height.saturating_sub(1)
            } else {
                screen.height
            },
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpenLayer {
    pub(crate) id: Id,
    pub(crate) layer: LayerId,
    pub(crate) spec: LayerSpec,
    pub(crate) opening: LayerOpening,
    pub(crate) closed_event: Option<LayerEvent>,

    pub(crate) restore_to: Option<Id>,
}

impl OpenLayer {
    pub(crate) const fn scope(&self) -> ScopeId {
        ScopeId::new(self.id)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LayerStack {
    open: Vec<OpenLayer>,

    pending: Vec<PendingLayerEvent>,

    next: u32,

    next_generation: u64,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self {
            open: Vec::new(),
            pending: Vec::new(),
            next: 1,
            next_generation: 1,
        }
    }
}

impl LayerStack {
    pub(crate) fn top(&self) -> LayerId {
        self.open.last().map_or(LayerId::PAGE, |l| l.layer)
    }

    pub(crate) fn layers(&self) -> &[OpenLayer] {
        &self.open
    }

    pub(crate) fn get(&self, id: Id) -> Option<&OpenLayer> {
        self.open.iter().find(|l| l.id == id)
    }

    pub(crate) fn is_open(&self, id: Id) -> bool {
        self.get(id).is_some()
    }

    fn admission(&self, id: Id) -> Option<(LayerId, u64)> {
        if self.is_open(id) {
            return None;
        }
        let layer = LayerId(u16::try_from(self.next).ok()?);
        self.next_generation.checked_add(1)?;
        Some((layer, self.next_generation))
    }

    pub(crate) fn can_open(&self, id: Id) -> bool {
        self.admission(id).is_some()
    }

    pub(crate) fn spec_mut(&mut self, id: Id) -> Option<&mut LayerSpec> {
        self.open
            .iter_mut()
            .find(|l| l.id == id)
            .map(|l| &mut l.spec)
    }

    pub(crate) fn top_layer(&self) -> Option<&OpenLayer> {
        self.open.last()
    }

    pub(crate) fn inert_floor(&self) -> LayerId {
        self.open
            .iter()
            .rev()
            .find(|l| l.spec.inert_below)
            .map_or(LayerId::PAGE, |l| l.layer)
    }

    pub(crate) fn pointer_input_layer(&self) -> LayerId {
        let inert_floor = self.inert_floor();
        for open in self.open.iter().rev() {
            if open.layer <= inert_floor {
                return inert_floor;
            }
            let transparent_tooltip = open.spec.kind == LayerKind::Tooltip
                && open.spec.pointer_transparent
                && !open.spec.inert_below;
            if !transparent_tooltip {
                return open.layer;
            }
        }
        LayerId::PAGE
    }

    pub(crate) fn pointer_transparent_suffix(&self) -> impl Iterator<Item = LayerId> + '_ {
        let inert_floor = self.inert_floor();
        self.open
            .iter()
            .rev()
            .take_while(move |open| {
                open.layer > inert_floor
                    && open.spec.kind == LayerKind::Tooltip
                    && open.spec.pointer_transparent
                    && !open.spec.inert_below
            })
            .map(|open| open.layer)
    }

    pub(crate) fn open_with_runtime(
        &mut self,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
        runtime: u64,
    ) -> Option<LayerOpening> {
        let (layer, generation) = self.admission(id)?;
        Some(self.open_admitted(id, spec, restore_to, runtime, layer, generation))
    }

    pub(crate) fn replace_top_with_runtime(
        &mut self,
        current: LayerOpening,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
        runtime: u64,
        update: Option<UpdateToken>,
    ) -> Option<(Vec<OpenLayer>, LayerOpening)> {
        if self
            .top_layer()
            .is_none_or(|layer| layer.opening != current)
        {
            return None;
        }
        let (layer, generation) = self.admission(id)?;
        let closed = self.close_with_update(
            current.id(),
            LayerEvent::Dismissed(DismissReason::Programmatic),
            update,
        );
        debug_assert_eq!(closed.len(), 1, "an exact top replacement closes one layer");
        let opening = self.open_admitted(id, spec, restore_to, runtime, layer, generation);
        Some((closed, opening))
    }

    fn open_admitted(
        &mut self,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
        runtime: u64,
        layer: LayerId,
        generation: u64,
    ) -> LayerOpening {
        self.next = self.next.saturating_add(1);
        self.next_generation = generation + 1;
        let parent = self.open.last().map(|open| LayerParent::from(open.opening));
        let opening = LayerOpening::new(runtime, generation, id, spec.owner, layer, parent);
        self.open.push(OpenLayer {
            id,
            layer,
            spec,
            opening,
            closed_event: None,
            restore_to,
        });
        self.pending.push(PendingLayerEvent {
            opening,
            event: LayerEvent::Opened,
            update: None,
        });
        opening
    }

    pub(crate) fn close_with_update(
        &mut self,
        id: Id,
        event: LayerEvent,
        update: Option<UpdateToken>,
    ) -> Vec<OpenLayer> {
        let Some(pos) = self.open.iter().position(|l| l.id == id) else {
            return Vec::new();
        };
        let mut closed: Vec<OpenLayer> = self.open.drain(pos..).collect();
        for (i, l) in closed.iter_mut().enumerate() {
            let ev = if i == 0 {
                event
            } else {
                LayerEvent::Dismissed(DismissReason::Programmatic)
            };
            l.closed_event = Some(ev);
            self.pending.push(PendingLayerEvent {
                opening: l.opening,
                event: ev,
                update: if i == 0 { update } else { None },
            });
        }
        closed
    }

    pub(crate) fn take_pending_receipts(&mut self) -> Vec<PendingLayerEvent> {
        core::mem::take(&mut self.pending)
    }
}
