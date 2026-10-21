use ratatui_core::layout::{Constraint, Position, Rect};

pub use termrock_core::layer_types::{DismissReason, LayerEvent, LayerId};

#[cfg(test)]
use crate::action::ActionKey;
use crate::focus::ScopeId;
use crate::id::Id;

/// Identifies one application update callback within a runtime.
///
/// Tokens are intentionally opaque: consumers may compare them to associate a
/// component action with a programmatic layer close from that same callback.
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

/// The lifecycle event for one exact layer opening.
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

/// Receipt for a closed opening after replacement layout and focus have settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerCompletion {
    opening: LayerOpening,
    event: LayerEvent,
    restored_focus: Option<Id>,
    continuation_owner: Option<Id>,
    continuation_acknowledged: bool,
}

/// Settled outcome for a field that continued focus through one exact layer opening.
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

    /// Vertically centred with any odd leftover row placed above the layer,
    /// matching ratatui's `Flex::Center` splitting.
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

/// What a layer shows when a dismissal is refused.
///
/// Ported from the PR #26 reference popup vocabulary
/// (`src/ui/popup.rs` at f4867e9b5f8a08a3bb61ce80d644198a29f51984). Nothing
/// in the runtime renders the feedback yet; the knob exists so overlays can
/// adopt it without another type change. [`RefuseFeedback::default`] keeps the
/// current no-feedback behavior until that adoption lands.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RefuseFeedback {
    /// Flash the layer once to mark the refused dismissal.
    #[default]
    Flash,
    /// Stay silent.
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Dismiss {
    pub esc: bool,

    pub outside_click: bool,

    pub focus_out: bool,

    /// Keys and pointer presses inside the layer still reach the page below.
    ///
    /// Reference `popup::Dismiss::pass_through`: while the layer is open the
    /// runtime keeps routing input through it instead of trapping it. The
    /// runtime does not consult this yet (adoption handoff); it defaults to
    /// `false`, the current trapping behavior.
    pub pass_through: bool,

    /// The layer may show a close affordance; see [`Dismiss::shows_close`].
    pub close_button: bool,

    /// What to show when a dismissal is refused.
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

    /// Whether a dismissal `reason` may close this layer.
    ///
    /// Ported from the reference `Dismiss::allows`. Programmatic dismissal is
    /// never refused: an owner closing its own layer is not a user gesture.
    pub const fn allows(self, reason: DismissReason) -> bool {
        match reason {
            DismissReason::Esc => self.esc,
            DismissReason::OutsideClick => self.outside_click,
            DismissReason::FocusOut => self.focus_out,
            // `DismissReason` is `#[non_exhaustive]`; a reason added later is
            // owner-initiated until this match learns about it, so it is
            // allowed — the same verdict as `Programmatic`.
            _ => true,
        }
    }

    /// A close affordance is only honest when Esc actually dismisses.
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

    /// Total horizontal and vertical viewport space reserved outside this layer.
    pub viewport_margin: (u16, u16),

    /// Keep this live underlying control logically active while this layer owns input.
    pub continues_focus_owner: Option<Id>,

    pub backdrop: Backdrop,

    pub inert_below: bool,
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
        }
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

    #[cfg(test)]
    pub(crate) fn open(
        &mut self,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
    ) -> Option<LayerId> {
        self.open_with_runtime(id, spec, restore_to, 0)
            .map(LayerOpening::layer_id)
    }

    pub(crate) fn open_with_runtime(
        &mut self,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
        runtime: u64,
    ) -> Option<LayerOpening> {
        if self.is_open(id) {
            return None;
        }
        let layer = LayerId(u16::try_from(self.next).ok()?);
        self.next = self.next.saturating_add(1);
        let generation = self.next_generation;
        self.next_generation = self.next_generation.checked_add(1)?;
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
        Some(opening)
    }

    #[cfg(test)]
    pub(crate) fn close(&mut self, id: Id, event: LayerEvent) -> Vec<OpenLayer> {
        self.close_with_update(id, event, None)
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

    #[cfg(test)]
    pub(crate) fn take_pending(&mut self) -> Vec<(Id, LayerEvent)> {
        self.take_pending_receipts()
            .into_iter()
            .map(|pending| (pending.opening.id(), pending.event))
            .collect()
    }

    pub(crate) fn take_pending_receipts(&mut self) -> Vec<PendingLayerEvent> {
        core::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DLG: Id = Id::root("dlg");
    const PICK: Id = Id::root("pick");

    #[test]
    fn push_and_pop_maintain_layer_order() {
        let mut s = LayerStack::default();
        assert_eq!(s.top(), LayerId::PAGE);
        assert_eq!(s.open(DLG, LayerSpec::modal(DLG), None), Some(LayerId(1)));
        assert_eq!(
            s.open(
                PICK,
                LayerSpec::popover(PICK, Anchor::Point(Position::new(1, 1))),
                None
            ),
            Some(LayerId(2))
        );
        assert_eq!(s.open(PICK, LayerSpec::modal(PICK), None), None);
        assert_eq!(s.top(), LayerId(2));
        let closed = s.close(PICK, LayerEvent::Closed(ActionKey::CONFIRM));
        assert_eq!(closed.len(), 1);
        assert_eq!(s.top(), LayerId(1));

        s.open(PICK, LayerSpec::modal(PICK), None);
        let closed = s.close(DLG, LayerEvent::Dismissed(DismissReason::Esc));
        assert_eq!(
            closed.iter().map(|l| l.id).collect::<Vec<_>>(),
            vec![DLG, PICK]
        );
        assert_eq!(s.top(), LayerId::PAGE);
        let ev = s.take_pending();
        assert!(ev.contains(&(DLG, LayerEvent::Dismissed(DismissReason::Esc))));
        assert!(ev.contains(&(PICK, LayerEvent::Dismissed(DismissReason::Programmatic))));
    }

    #[test]
    fn reopened_layers_keep_monotonic_ids() {
        let mut s = LayerStack::default();
        assert_eq!(s.open(DLG, LayerSpec::modal(DLG), None), Some(LayerId(1)));
        assert_eq!(s.close(DLG, LayerEvent::Closed(ActionKey::CANCEL)).len(), 1);
        assert_eq!(s.open(PICK, LayerSpec::modal(PICK), None), Some(LayerId(2)));
    }

    #[test]
    fn receipts_retain_each_reopened_layer_opening_and_close_callback() {
        let mut stack = LayerStack::default();
        let first = stack
            .open_with_runtime(DLG, LayerSpec::modal(DLG), None, 7)
            .expect("first opening");
        let first_update = UpdateToken::new(7, 11);
        stack.close_with_update(
            DLG,
            LayerEvent::Dismissed(DismissReason::Programmatic),
            Some(first_update),
        );
        let second = stack
            .open_with_runtime(DLG, LayerSpec::modal(DLG), None, 7)
            .expect("reopened layer");
        let second_update = UpdateToken::new(7, 12);
        stack.close_with_update(
            DLG,
            LayerEvent::Dismissed(DismissReason::Programmatic),
            Some(second_update),
        );

        let receipts = stack.take_pending_receipts();
        let closed: Vec<_> = receipts
            .iter()
            .filter(|receipt| receipt.event == LayerEvent::Dismissed(DismissReason::Programmatic))
            .collect();
        assert_eq!(closed.len(), 2);
        assert_eq!(closed[0].opening, first);
        assert_eq!(closed[0].update, Some(first_update));
        assert_eq!(closed[1].opening, second);
        assert_eq!(closed[1].update, Some(second_update));
        assert_ne!(closed[0].opening, closed[1].opening);
    }

    #[test]
    fn opening_tokens_identify_runtime_generation_and_concrete_parent() {
        let mut stack = LayerStack::default();
        let parent = stack
            .open_with_runtime(DLG, LayerSpec::modal(DLG), None, 17)
            .expect("parent layer opens");
        let child = stack
            .open_with_runtime(
                PICK,
                LayerSpec::popover(PICK, Anchor::Screen(ScreenAlign::Center)),
                None,
                17,
            )
            .expect("child layer opens");
        assert_eq!(parent.runtime(), 17);
        assert_eq!(parent.generation(), 1);
        assert_eq!(parent.id(), DLG);
        assert_eq!(child.parent(), Some(parent.into()));

        let _ = stack.close(DLG, LayerEvent::Dismissed(DismissReason::Esc));
        let reopened = stack
            .open_with_runtime(DLG, LayerSpec::modal(DLG), None, 17)
            .expect("same layer id can open again");
        assert_ne!(reopened, parent);
        assert_eq!(reopened.generation(), 3);

        let mut other_runtime = LayerStack::default();
        let other = other_runtime
            .open_with_runtime(DLG, LayerSpec::modal(DLG), None, 18)
            .expect("same layer identity in another runtime");
        assert_ne!(other, parent);
    }

    #[test]
    fn modal_pushes_a_trap_and_a_pointer_barrier() {
        let m = LayerSpec::modal(DLG);
        assert_eq!(m.kind, LayerKind::Modal);
        assert!(m.inert_below && m.restore_focus);
        assert!(matches!(
            m.backdrop,
            Backdrop::Dim {
                exclude_footer: true
            }
        ));
        assert!(!m.dismisses_on_focus_out());
        let mut s = LayerStack::default();
        s.open(DLG, m, Some(Id::root("opener")));
        assert_eq!(s.inert_floor(), LayerId(1));
        assert_eq!(
            s.get(DLG).and_then(|l| l.restore_to),
            Some(Id::root("opener"))
        );
    }

    #[test]
    fn popover_pushes_a_pointer_barrier_only() {
        let p = LayerSpec::popover(PICK, Anchor::Point(Position::new(0, 0))).dismiss(Dismiss::ALL);
        assert_eq!(p.kind, LayerKind::Popover);
        assert!(!p.inert_below);
        assert_eq!(p.backdrop, Backdrop::None);
        assert!(p.dismisses_on_focus_out());
        let mut s = LayerStack::default();
        s.open(PICK, p, None);
        assert_eq!(s.inert_floor(), LayerId::PAGE);
        assert_eq!(s.top(), LayerId(1));
    }

    #[test]
    fn anchor_rect_flips_then_clamps() {
        let screen = Rect::new(0, 0, 100, 30);
        let anchor = Rect::new(10, 5, 5, 1);
        let below = Anchor::Rect {
            rect: anchor,
            side: Side::Below,
            align: CrossAlign::Start,
        };
        assert_eq!(
            resolve_anchor(screen, below, LayerSize::Fixed(40, 8)),
            Rect::new(10, 6, 40, 8)
        );
        let low = Anchor::Rect {
            rect: Rect::new(10, 26, 5, 1),
            side: Side::Below,
            align: CrossAlign::Start,
        };
        assert_eq!(
            resolve_anchor(screen, low, LayerSize::Fixed(40, 8)),
            Rect::new(10, 18, 40, 8)
        );
        let right = Anchor::Rect {
            rect: Rect::new(90, 5, 5, 1),
            side: Side::Below,
            align: CrossAlign::Start,
        };
        assert_eq!(
            resolve_anchor(screen, right, LayerSize::Fixed(40, 8)).right(),
            100
        );
        let tall = resolve_anchor(screen, below, LayerSize::Fixed(40, 60));
        assert!(tall.height <= 30);
        let end = Anchor::Rect {
            rect: Rect::new(50, 5, 10, 1),
            side: Side::Above,
            align: CrossAlign::End,
        };
        assert_eq!(
            resolve_anchor(screen, end, LayerSize::Fixed(4, 2)),
            Rect::new(56, 3, 4, 2)
        );
        let side = Anchor::Rect {
            rect: Rect::new(95, 5, 5, 3),
            side: Side::Right,
            align: CrossAlign::Center,
        };
        assert_eq!(
            resolve_anchor(screen, side, LayerSize::Fixed(10, 1)),
            Rect::new(85, 6, 10, 1)
        );
    }

    #[test]
    fn anchor_screen_center_sits_in_the_upper_third() {
        let screen = Rect::new(0, 0, 120, 40);
        let r = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::Center),
            LayerSize::Fixed(60, 12),
        );
        assert_eq!(r, Rect::new(30, 9, 60, 12));
        let b = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::Bottom),
            LayerSize::Fixed(60, 12),
        );
        assert_eq!(b.bottom(), 40);
        let u = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::UpperThird),
            LayerSize::Fixed(60, 12),
        );
        assert!(u.y < r.y);
    }

    #[test]
    fn anchor_screen_geometric_center_centers_inside_offset_viewport() {
        let screen = Rect::new(7, 11, 120, 40);
        let r = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::GeometricCenter),
            LayerSize::Fixed(60, 12),
        );
        assert_eq!(r, Rect::new(37, 25, 60, 12));
    }

    #[test]
    fn anchor_screen_balanced_centers_with_the_odd_row_above() {
        // Ratatui `Flex::Center` splitting: free 29 around an 11-row layer
        // puts 15 rows above and 14 below.
        for (height, expected_y) in [(20u16, 5u16), (24, 7), (30, 10), (40, 15), (50, 20)] {
            let screen = Rect::new(0, 0, 120, height);
            let r = resolve_anchor(
                screen,
                Anchor::Screen(ScreenAlign::Balanced),
                LayerSize::Fixed(54, 11),
            );
            assert_eq!(r, Rect::new(33, expected_y, 54, 11), "height {height}");
        }
        // An even leftover leaves no rounding bias.
        let even = resolve_anchor(
            Rect::new(0, 0, 120, 41),
            Anchor::Screen(ScreenAlign::Balanced),
            LayerSize::Fixed(54, 11),
        );
        assert_eq!(even.y, 15);
    }

    #[test]
    fn anchor_screen_geometric_center_clamps_oversize_and_handles_tiny_viewports() {
        let screen = Rect::new(9, 13, 3, 2);
        assert_eq!(
            resolve_anchor(
                screen,
                Anchor::Screen(ScreenAlign::GeometricCenter),
                LayerSize::Fixed(20, 8),
            ),
            screen
        );
        assert_eq!(
            resolve_anchor(
                screen,
                Anchor::Screen(ScreenAlign::GeometricCenter),
                LayerSize::Fixed(1, 1),
            ),
            Rect::new(10, 13, 1, 1)
        );
    }

    #[test]
    fn fill_resolves_to_the_whole_screen() {
        let screen = Rect::new(0, 0, 120, 40);
        for anchor in [
            Anchor::Screen(ScreenAlign::Center),
            Anchor::Point(Position::new(3, 3)),
            Anchor::Rect {
                rect: Rect::new(1, 1, 2, 2),
                side: Side::Below,
                align: CrossAlign::Start,
            },
        ] {
            assert_eq!(resolve_anchor(screen, anchor, LayerSize::Fill), screen);
        }
        assert_eq!(LayerSpec::modal(DLG).size, LayerSize::Fill);
        assert_eq!(
            LayerSpec::popover(DLG, Anchor::Screen(ScreenAlign::Center)).size,
            LayerSize::Fill
        );
        assert_eq!(
            LayerSpec::tooltip(DLG, Position::new(0, 0)).size,
            LayerSize::Fill
        );
    }

    #[test]
    fn fixed_size_is_clamped_never_grown() {
        let screen = Rect::new(0, 0, 40, 10);
        let r = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::Center),
            LayerSize::Fixed(54, 20),
        );
        assert_eq!(r, screen);
        assert_eq!(
            resolve_anchor(
                screen,
                Anchor::Screen(ScreenAlign::Center),
                LayerSize::Fixed(0, 8)
            ),
            Rect::ZERO
        );
        assert_eq!(
            resolve_anchor(
                screen,
                Anchor::Screen(ScreenAlign::Center),
                LayerSize::Fixed(8, 0)
            ),
            Rect::ZERO
        );

        let small = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::Center),
            LayerSize::Fixed(10, 4),
        );
        assert_eq!((small.width, small.height), (10, 4));
    }

    #[test]
    fn viewport_margin_limits_fixed_size_without_changing_anchor_viewport() {
        let viewport = Rect::new(7, 5, 120, 12);
        let spec = LayerSpec::popover(DLG, Anchor::Screen(ScreenAlign::GeometricCenter))
            .size(LayerSize::Fixed(88, 19))
            .viewport_margin(4, 2);
        assert_eq!(spec.size, LayerSize::Fixed(88, 19));
        assert_eq!(resolve_layer_area(viewport, spec), Rect::new(23, 6, 88, 10));
        assert_eq!(
            resolve_layer_area(Rect::new(7, 5, 120, 40), spec),
            Rect::new(23, 15, 88, 19)
        );
    }

    #[test]
    fn viewport_margin_handles_tiny_and_empty_viewports() {
        let spec = LayerSpec::popover(DLG, Anchor::Screen(ScreenAlign::GeometricCenter))
            .size(LayerSize::Fixed(88, 19))
            .viewport_margin(4, 2);
        assert_eq!(
            resolve_layer_area(Rect::new(9, 13, 1, 1), spec),
            Rect::new(9, 13, 1, 1)
        );
        assert_eq!(resolve_layer_area(Rect::new(9, 13, 0, 5), spec), Rect::ZERO);
        assert_eq!(resolve_layer_area(Rect::new(9, 13, 5, 0), spec), Rect::ZERO);
    }

    #[test]
    fn viewport_margin_does_not_change_fill_layer_semantics() {
        let viewport = Rect::new(7, 5, 120, 40);
        let spec = LayerSpec::modal(DLG).viewport_margin(4, 2);
        assert_eq!(resolve_layer_area(viewport, spec), viewport);
    }

    #[test]
    fn popover_flips_above_when_the_content_does_not_fit_below() {
        let screen = Rect::new(0, 0, 40, 20);
        let below = Anchor::Rect {
            rect: Rect::new(4, 17, 6, 1),
            side: Side::Below,
            align: CrossAlign::Start,
        };
        let r = resolve_anchor(screen, below, LayerSize::Fixed(12, 6));
        assert_eq!(r, Rect::new(4, 11, 12, 6));

        let high = Anchor::Rect {
            rect: Rect::new(4, 2, 6, 1),
            side: Side::Below,
            align: CrossAlign::Start,
        };
        assert_eq!(
            resolve_anchor(screen, high, LayerSize::Fixed(12, 6)),
            Rect::new(4, 3, 12, 6)
        );
    }

    #[test]
    fn point_anchor_flips_instead_of_covering_the_pointer() {
        let screen = Rect::new(0, 0, 40, 10);
        let p = resolve_anchor(
            screen,
            Anchor::Point(Position::new(38, 9)),
            LayerSize::Fixed(10, 3),
        );
        assert_eq!(p, Rect::new(28, 6, 10, 3));
        assert!(!p.contains(Position::new(38, 9)));

        let q = resolve_anchor(
            screen,
            Anchor::Point(Position::new(2, 1)),
            LayerSize::Fixed(10, 3),
        );
        assert_eq!(q, Rect::new(2, 2, 10, 3));
        assert!(!q.contains(Position::new(2, 1)));
    }

    #[test]
    fn closed_with_action_key_emits_layer_event_closed() {
        let mut s = LayerStack::default();
        s.open(DLG, LayerSpec::modal(DLG), None);
        let _ = s.take_pending();
        s.close(DLG, LayerEvent::Closed(ActionKey::SAVE));
        assert_eq!(
            s.take_pending(),
            vec![(DLG, LayerEvent::Closed(ActionKey::SAVE))]
        );
    }

    #[test]
    fn dismissed_emits_the_reason() {
        let mut s = LayerStack::default();
        s.open(DLG, LayerSpec::modal(DLG), None);
        assert_eq!(s.take_pending(), vec![(DLG, LayerEvent::Opened)]);
        s.close(DLG, LayerEvent::Dismissed(DismissReason::OutsideClick));
        assert_eq!(
            s.take_pending(),
            vec![(DLG, LayerEvent::Dismissed(DismissReason::OutsideClick))]
        );
    }

    #[test]
    fn backdrop_excludes_the_footer_row() {
        let screen = Rect::new(0, 0, 120, 40);
        assert_eq!(
            backdrop_area(
                screen,
                Backdrop::Dim {
                    exclude_footer: true
                }
            ),
            Rect::new(0, 0, 120, 39)
        );
        assert_eq!(
            backdrop_area(
                screen,
                Backdrop::Dim {
                    exclude_footer: false
                }
            ),
            screen
        );
        assert_eq!(backdrop_area(screen, Backdrop::None), Rect::ZERO);
    }
}

#[cfg(test)]
mod runtime_tests {
    use ratatui_core::layout::{Position, Rect};

    use super::*;
    use crate::diagnostics::Diagnostic;
    use crate::event::{KeyCode, MouseKind};
    use crate::focus::ScopeId;
    use crate::intent::Intent;
    use crate::runtime::stub::{Control, Stub, key, mouse, runtime, step};

    const PAGE_A: Id = Id::root("page.a");
    const PAGE_B: Id = Id::root("page.b");
    const DLG: Id = Id::root("dlg");
    const OK: Id = Id::root("dlg.ok");
    const INPUT: Id = Id::root("dlg.input");
    const PICK: Id = Id::root("pick");
    const ROW: Id = Id::root("pick.row");

    #[derive(Default)]
    struct BootstrapLayer {
        opened: bool,
        bootstrap_viewport: Option<Rect>,
    }

    impl crate::runtime::App for BootstrapLayer {
        fn update(&mut self, cx: &mut crate::ui::Cx<'_>) -> crate::response::Response<()> {
            if !self.opened {
                self.bootstrap_viewport = Some(cx.viewport());
                self.opened = true;
                cx.open_layer(
                    PICK,
                    LayerSpec::popover(PICK, Anchor::Screen(ScreenAlign::GeometricCenter))
                        .size(LayerSize::Fixed(88, 19))
                        .viewport_margin(4, 2),
                );
            }
            crate::response::Response::ignored()
        }

        fn draw(&self, ui: &mut crate::ui::Ui<'_>) {
            let _ = ui.layer(PICK, |_, _| ());
        }
    }

    fn dialog_stub() -> Stub {
        Stub {
            page: vec![Control::new(PAGE_A, Rect::new(0, 0, 10, 1))],
            layers: vec![(
                DLG,
                vec![
                    Control::new(OK, Rect::new(5, 5, 5, 1)),
                    Control {
                        editor: true,
                        ..Control::new(INPUT, Rect::new(5, 7, 10, 1))
                    },
                ],
            )],
            consume_keys: false,
            ..Stub::default()
        }
    }

    fn popover_stub() -> Stub {
        Stub {
            page: vec![
                Control::new(PAGE_A, Rect::new(0, 0, 10, 1)),
                Control::new(PAGE_B, Rect::new(0, 2, 10, 1)),
            ],
            layers: vec![(PICK, vec![Control::new(ROW, Rect::new(20, 4, 5, 1))])],
            consume_keys: false,
            ..Stub::default()
        }
    }

    #[test]
    fn resize_layer_re_resolves_the_anchor_on_the_next_draw() {
        let (mut rt, mut buf) = runtime(dialog_stub());
        rt.app_mut().open_request =
            Some((DLG, LayerSpec::modal(DLG).size(LayerSize::Fixed(20, 4))));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        let first = rt.layer_area(DLG).expect("the layer is open");
        assert_eq!((first.width, first.height), (20, 4));
        rt.app_mut().resize_request = Some((DLG, LayerSize::Fixed(40, 10)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        let grown = rt.layer_area(DLG).expect("the layer is still open");
        assert_eq!((grown.width, grown.height), (40, 10));

        assert_eq!(
            grown,
            resolve_anchor(
                rt.screen(),
                Anchor::Screen(ScreenAlign::Center),
                LayerSize::Fixed(40, 10)
            )
        );

        rt.app_mut().resize_request = Some((DLG, LayerSize::Fixed(40, 10)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert_eq!(rt.layer_area(DLG), Some(grown));
    }

    #[test]
    fn bootstrap_layer_resolves_against_actual_viewport_and_restores_desired_size() {
        use ratatui_core::buffer::Buffer;

        let mut rt =
            crate::runtime::Runtime::new(BootstrapLayer::default(), crate::theme::Theme::junie());
        let _ = rt.initialize();
        assert_eq!(rt.app().bootstrap_viewport, Some(Rect::ZERO));

        let mut short = Buffer::empty(Rect::new(0, 0, 120, 12));
        rt.draw_buffer(Rect::new(0, 0, 120, 12), &mut short)
            .commit_presented();
        assert_eq!(rt.layer_area(PICK), Some(Rect::new(16, 1, 88, 10)));

        let mut tall = Buffer::empty(Rect::new(0, 0, 120, 40));
        rt.draw_buffer(Rect::new(0, 0, 120, 40), &mut tall)
            .commit_presented();
        assert_eq!(rt.layer_area(PICK), Some(Rect::new(16, 10, 88, 19)));
        assert_eq!(
            rt.open_spec(PICK).map(|spec| spec.size),
            Some(LayerSize::Fixed(88, 19))
        );
    }

    #[test]
    fn spec_geometry_is_the_only_mutable_field() {
        let (mut rt, mut buf) = runtime(dialog_stub());
        let spec = LayerSpec::modal(DLG)
            .size(LayerSize::Fixed(20, 4))
            .initial_focus(OK)
            .inert_below(true);
        rt.app_mut().open_request = Some((DLG, spec));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        rt.app_mut().resize_request = Some((DLG, LayerSize::Fixed(24, 6)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

        let open = rt.open_spec(DLG).expect("the layer is open");
        assert_eq!(open.size, LayerSize::Fixed(24, 6));
        assert_eq!(open.kind, spec.kind);
        assert_eq!(open.inert_below, spec.inert_below);
        assert_eq!(open.restore_focus, spec.restore_focus);
        assert_eq!(open.initial_focus, spec.initial_focus);
        assert_eq!(open.dismiss, spec.dismiss);
    }

    #[test]
    fn esc_dismisses_only_the_top_layer() {
        let mut s = dialog_stub();
        s.layers
            .push((PICK, vec![Control::new(ROW, Rect::new(20, 2, 5, 1))]));
        let (mut rt, mut buf) = runtime(s);
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        rt.app_mut().open_request = Some((
            PICK,
            LayerSpec::popover(PICK, Anchor::Point(Position::new(20, 2))),
        ));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert_eq!(rt.top_layer(), LayerId(2));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(!rt.is_open(PICK));
        assert!(rt.is_open(DLG));
        assert!(rt.app().saw(PICK, "Dismissed(Esc)"));
        assert!(rt.app().saw(PICK, "Cancel"));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(!rt.is_open(DLG));
        assert_eq!(rt.focus(), Some(PAGE_A), "focus restored to the opener");
    }

    #[test]
    fn esc_reaches_the_focused_editor_before_the_layer() {
        let (mut rt, mut buf) = runtime(dialog_stub());
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG).initial_focus(INPUT)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert_eq!(rt.focus(), Some(INPUT));

        let r = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(r.is_consumed());
        assert!(rt.is_open(DLG));
        assert!(rt.app().saw(INPUT, "Key("));

        let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
        assert_eq!(rt.focus(), Some(OK));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(!rt.is_open(DLG));
    }

    #[test]
    fn layer_id_is_assigned_at_open_not_at_draw() {
        let mut s = dialog_stub();

        s.layers
            .insert(0, (PICK, vec![Control::new(ROW, Rect::new(20, 2, 5, 1))]));
        let (mut rt, mut buf) = runtime(s);
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        rt.app_mut().open_request = Some((
            PICK,
            LayerSpec::popover(PICK, Anchor::Point(Position::new(20, 2))),
        ));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert_eq!(rt.top_layer(), LayerId(2));
        assert_eq!(rt.registry().layer_of(ROW), Some(LayerId(2)));
        assert_eq!(rt.registry().layer_of(OK), Some(LayerId(1)));

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 21, 2));
        assert!(rt.app().saw(ROW, "Press"));
    }

    #[test]
    fn outside_click_is_layer_less_than_top_or_none() {
        let (mut rt, mut buf) = runtime(dialog_stub());
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 30, 10));
        assert!(!rt.is_open(DLG));
        assert!(rt.app().saw(DLG, "Dismissed(OutsideClick)"));

        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        assert!(!rt.is_open(DLG));
        assert!(!rt.app().saw(PAGE_A, "Press"));
    }

    #[test]
    fn focus_out_dismisses_a_popover_but_never_a_modal() {
        let (mut rt, mut buf) = runtime(popover_stub());
        assert_eq!(rt.focus(), Some(PAGE_A));
        rt.app_mut().open_request = Some((
            PICK,
            LayerSpec::popover(PICK, Anchor::Point(Position::new(20, 4))).dismiss(Dismiss::ALL),
        ));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.is_open(PICK));
        assert_eq!(
            rt.focus(),
            Some(PAGE_A),
            "the opener keeps focus while open"
        );
        let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
        assert!(
            !rt.is_open(PICK),
            "focus left the popover's scope, so the popover is dismissed"
        );
        assert!(
            rt.app().saw(PICK, "Dismissed(FocusOut)"),
            "the owner must be told why: {:?}",
            rt.app().log
        );
        assert_eq!(
            rt.focus(),
            Some(PAGE_B),
            "focus is on the Tab target, not restored to the opener"
        );
        assert!(
            !rt.diagnostics()
                .iter()
                .any(|d| matches!(d, Diagnostic::FocusTransitionDidNotSettle { .. })),
            "the dismissal is one more pass, not an unsettled loop: {:?}",
            rt.diagnostics()
        );

        let (mut rt, mut buf) = runtime(dialog_stub());
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG).dismiss(Dismiss::ALL)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.is_open(DLG));
        assert_eq!(rt.focus(), Some(OK), "the trap moved focus into the modal");
        for _ in 0..3 {
            let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
            assert!(rt.is_open(DLG), "a modal never dismisses on focus-out");
            let focused = rt.focus().expect("the trap is not empty");
            assert_eq!(
                rt.ring().entry(focused).map(|e| e.scope),
                Some(ScopeId::new(DLG)),
                "focus never left the modal's scope"
            );
        }
        assert!(!rt.app().saw(DLG, "Dismissed(FocusOut)"));
    }

    #[test]
    fn nested_layers_each_trap() {
        let mut s = dialog_stub();
        s.layers
            .push((PICK, vec![Control::new(ROW, Rect::new(20, 2, 5, 1))]));
        let (mut rt, mut buf) = runtime(s);
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        let ids: Vec<Id> = rt.ring().reachable().map(|e| e.id).collect();
        assert_eq!(ids, vec![OK, INPUT]);
        rt.app_mut().open_request = Some((PICK, LayerSpec::modal(PICK)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        let ids: Vec<Id> = rt.ring().reachable().map(|e| e.id).collect();
        assert_eq!(ids, vec![ROW]);
        assert_eq!(rt.focus(), Some(ROW));
        rt.app_mut().close_request = Some(PICK);
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert_eq!(
            rt.focus(),
            Some(OK),
            "focus restored to the dialog's control"
        );
        assert!(
            rt.needs_settle(),
            "publication stages restoration callbacks"
        );
        let _ = rt.settle();
        assert!(rt.app().saw(OK, "FocusIn { via: Restore }"));
        let ids: Vec<Id> = rt.ring().reachable().map(|e| e.id).collect();
        assert_eq!(ids, vec![OK, INPUT]);
        let _ = Intent::Cancel;
    }
}
