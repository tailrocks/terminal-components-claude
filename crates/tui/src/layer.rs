//! The runtime-owned layer stack (`COMPONENT_ARCHITECTURE.md` §9, §21 items 8, 14, 20).
//!
//! A layer is opened from `update` with [`LayerSpec`], drawn from `draw`
//! inside `ui.layer(id, …)`, and composited bottom-to-top after `app.draw`
//! returns, so z-order is the layer order and never the call order.
//! Placement is one resolver: anchor, flip, then clamp (`Rect::clamp`).

use ratatui_core::layout::{Constraint, Position, Rect};

use crate::action::ActionKey;
use crate::focus::ScopeId;
use crate::id::Id;

/// A stack position; `0` is the page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LayerId(pub(crate) u16);

impl LayerId {
    /// The page.
    pub const PAGE: LayerId = LayerId(0);

    /// The stack position.
    pub const fn index(self) -> u16 {
        self.0
    }
}

/// What a layer traps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerKind {
    /// Focus + pointer trap, dim backdrop, inert below.
    Modal,
    /// Pointer barrier only.
    Popover,
    /// Pointer barrier only, no focus scope of its own.
    Tooltip,
}

/// Where the layer's area is placed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anchor {
    /// Relative to the whole screen.
    Screen(ScreenAlign),
    /// Adjacent to a rect, flipping when there is no room.
    Rect {
        /// The anchor rect.
        rect: Rect,
        /// Which side of the rect.
        side: Side,
        /// Alignment along the other axis.
        align: CrossAlign,
    },
    /// At a point (a tooltip or context menu).
    Point(Position),
}

/// Screen placement for `Anchor::Screen`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenAlign {
    /// Horizontally centred, vertically in the optical centre (upper third).
    Center,
    /// Horizontally centred, near the top.
    UpperThird,
    /// Horizontally centred, at the bottom.
    Bottom,
}

/// Which side of an anchor rect.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// Below the rect.
    Below,
    /// Above the rect.
    Above,
    /// Left of the rect.
    Left,
    /// Right of the rect.
    Right,
}

/// Alignment along the axis perpendicular to `Side`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrossAlign {
    /// Align starts.
    Start,
    /// Centre.
    Center,
    /// Align ends.
    End,
}

/// How a layer may be dismissed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Dismiss {
    /// Esc in the bubble phase.
    pub esc: bool,
    /// A click whose hit is below the layer, or nowhere.
    pub outside_click: bool,
    /// Focus leaving the layer (honoured for `Popover`/`Tooltip` only).
    pub focus_out: bool,
}

impl Dismiss {
    /// Programmatic only.
    pub const NONE: Dismiss = Dismiss {
        esc: false,
        outside_click: false,
        focus_out: false,
    };
    /// Esc only.
    pub const ESC: Dismiss = Dismiss {
        esc: true,
        outside_click: false,
        focus_out: false,
    };
    /// Esc and outside click.
    pub const ESC_AND_OUTSIDE: Dismiss = Dismiss {
        esc: true,
        outside_click: true,
        focus_out: false,
    };
    /// Esc, outside click and focus out.
    pub const ALL: Dismiss = Dismiss {
        esc: true,
        outside_click: true,
        focus_out: true,
    };
}

/// The backdrop behind a layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backdrop {
    /// No backdrop.
    None,
    /// Dim the page below.
    Dim {
        /// Keep the last row (the hint bar) undimmed.
        exclude_footer: bool,
    },
}

/// How large a layer asks to be (Adjudication N1).
///
/// The resolver clamps to the screen; it never grows a layer, so a `Fixed`
/// size is a maximum as well as a request ("size, then clamp, then documented
/// degradation", §9.1). The size is the opener's, the placement is the
/// runtime's: a component computes a size, never a rect.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerSize {
    /// The whole screen. The content is responsible for its own internal
    /// layout; `Anchor` is ignored. Help overlays, file browsers, `TooSmall`.
    Fill,
    /// Exactly `w × h` cells before clamping.
    Fixed(u16, u16),
}

/// A layer's configuration. Construct through the builders.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LayerSpec {
    /// The kind.
    pub kind: LayerKind,
    /// Anchor owner and focus-restore target.
    pub owner: Id,
    /// Placement.
    pub anchor: Anchor,
    /// Dismissal.
    pub dismiss: Dismiss,
    /// Restore focus to the opener on close.
    pub restore_focus: bool,
    /// Initial focus inside the layer.
    pub initial_focus: Option<Id>,
    /// The requested content size (§9.1).
    pub size: LayerSize,
    /// The backdrop.
    pub backdrop: Backdrop,
    /// No registrations from the layers below.
    pub inert_below: bool,
}

impl LayerSpec {
    /// Modal: `Screen(Center)`, esc + outside click, dim, inert below.
    pub const fn modal(owner: Id) -> LayerSpec {
        LayerSpec {
            kind: LayerKind::Modal,
            owner,
            anchor: Anchor::Screen(ScreenAlign::Center),
            dismiss: Dismiss::ESC_AND_OUTSIDE,
            restore_focus: true,
            initial_focus: None,
            size: LayerSize::Fill,
            backdrop: Backdrop::Dim {
                exclude_footer: true,
            },
            inert_below: true,
        }
    }

    /// Popover: pointer barrier only, no dim.
    pub const fn popover(owner: Id, anchor: Anchor) -> LayerSpec {
        LayerSpec {
            kind: LayerKind::Popover,
            owner,
            anchor,
            dismiss: Dismiss::ESC_AND_OUTSIDE,
            restore_focus: true,
            initial_focus: None,
            size: LayerSize::Fill,
            backdrop: Backdrop::None,
            inert_below: false,
        }
    }

    /// Tooltip at a point: no focus, dismissed by anything.
    pub const fn tooltip(owner: Id, at: Position) -> LayerSpec {
        LayerSpec {
            kind: LayerKind::Tooltip,
            owner,
            anchor: Anchor::Point(at),
            dismiss: Dismiss::ALL,
            restore_focus: false,
            initial_focus: None,
            size: LayerSize::Fill,
            backdrop: Backdrop::None,
            inert_below: false,
        }
    }

    /// Set the anchor.
    #[must_use]
    pub const fn anchor(mut self, a: Anchor) -> Self {
        self.anchor = a;
        self
    }

    /// Set dismissal.
    #[must_use]
    pub const fn dismiss(mut self, d: Dismiss) -> Self {
        self.dismiss = d;
        self
    }

    /// Set the backdrop.
    #[must_use]
    pub const fn backdrop(mut self, b: Backdrop) -> Self {
        self.backdrop = b;
        self
    }

    /// Set the initial focus.
    #[must_use]
    pub const fn initial_focus(mut self, id: Id) -> Self {
        self.initial_focus = Some(id);
        self
    }

    /// Set the requested size. `LayerSize::Fixed(w, h)` is a request *and* a
    /// maximum: the resolver clamps to the screen and never grows a layer.
    #[must_use]
    pub const fn size(mut self, s: LayerSize) -> Self {
        self.size = s;
        self
    }

    /// Set inertness of the layers below.
    #[must_use]
    pub const fn inert_below(mut self, yes: bool) -> Self {
        self.inert_below = yes;
        self
    }

    /// Set focus restoration.
    #[must_use]
    pub const fn restore_focus(mut self, yes: bool) -> Self {
        self.restore_focus = yes;
        self
    }

    /// Whether `focus_out` dismissal applies (never for a modal, §21 item 30 A10).
    pub const fn dismisses_on_focus_out(&self) -> bool {
        self.dismiss.focus_out && !matches!(self.kind, LayerKind::Modal)
    }
}

/// A layer lifecycle event, delivered to the layer's owner.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerEvent {
    /// The layer opened.
    Opened,
    /// The layer was dismissed.
    Dismissed(DismissReason),
    /// The layer was closed with an action.
    Closed(ActionKey),
}

/// Why a layer was dismissed.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DismissReason {
    /// Esc.
    Esc,
    /// A click outside.
    OutsideClick,
    /// Focus left.
    FocusOut,
    /// `cx.close_layer(id, None)`.
    Programmatic,
}

/// Resolve a layer's area: anchor, flip, then clamp (§9.1).
///
/// [`LayerSize::Fill`] yields the whole screen. A `Fixed` size is clipped to
/// the screen first, then anchored, then flipped if the chosen side has no
/// room, then `Rect::clamp`ed. A layer is **never grown** to meet its
/// request ("size, then clamp, then documented degradation"). A zero
/// dimension is an empty layer, not the screen.
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
        // a tooltip or a context menu near an edge is placed above/left of
        // the pointer rather than sliding over it (Adjudication N1, change 2)
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

/// The area a backdrop dims: the screen, minus the last row when the footer
/// is excluded (`DESIGN.md:537`).
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

/// One open layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpenLayer {
    pub(crate) id: Id,
    pub(crate) layer: LayerId,
    pub(crate) spec: LayerSpec,
    /// Focus to restore when this layer closes.
    pub(crate) restore_to: Option<Id>,
}

impl OpenLayer {
    pub(crate) const fn scope(&self) -> ScopeId {
        ScopeId::new(self.id)
    }
}

/// The runtime's layer stack.
#[derive(Clone, Debug, Default)]
pub(crate) struct LayerStack {
    open: Vec<OpenLayer>,
    /// Events staged for delivery on the next `handle`.
    pending: Vec<(Id, LayerEvent)>,
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

    /// The spec of an open layer, mutable. Only the geometry (`size`,
    /// `anchor`) may change while a layer is open: `kind`, `inert_below`,
    /// `restore_focus` and `initial_focus` were armed by the runtime when the
    /// layer was pushed and re-deriving them would desync the focus scope and
    /// the inert floor (§21 item 14).
    pub(crate) fn spec_mut(&mut self, id: Id) -> Option<&mut LayerSpec> {
        self.open
            .iter_mut()
            .find(|l| l.id == id)
            .map(|l| &mut l.spec)
    }

    pub(crate) fn top_layer(&self) -> Option<&OpenLayer> {
        self.open.last()
    }

    /// The lowest layer with `inert_below`; everything beneath it is inert.
    pub(crate) fn inert_floor(&self) -> LayerId {
        self.open
            .iter()
            .rev()
            .find(|l| l.spec.inert_below)
            .map_or(LayerId::PAGE, |l| l.layer)
    }

    /// Open a layer; the `LayerId` is its stack position. Re-opening an open
    /// id is a no-op.
    pub(crate) fn open(
        &mut self,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
    ) -> Option<LayerId> {
        if self.is_open(id) {
            return None;
        }
        let layer = LayerId((self.open.len() as u16).saturating_add(1));
        self.open.push(OpenLayer {
            id,
            layer,
            spec,
            restore_to,
        });
        self.pending.push((id, LayerEvent::Opened));
        Some(layer)
    }

    /// Close a layer and everything above it. Returns the closed layers,
    /// bottom-most first.
    pub(crate) fn close(&mut self, id: Id, event: LayerEvent) -> Vec<OpenLayer> {
        let Some(pos) = self.open.iter().position(|l| l.id == id) else {
            return Vec::new();
        };
        let closed: Vec<OpenLayer> = self.open.drain(pos..).collect();
        for (i, l) in closed.iter().enumerate() {
            let ev = if i == 0 {
                event
            } else {
                LayerEvent::Dismissed(DismissReason::Programmatic)
            };
            self.pending.push((l.id, ev));
        }
        closed
    }

    pub(crate) fn take_pending(&mut self) -> Vec<(Id, LayerEvent)> {
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
        // closing a lower layer closes everything above it
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

    /// The field was never a minimum: the resolver clamps down and never
    /// grows (Adjudication N1). A zero dimension is an *empty* layer, which
    /// is what §16.2 case 19 and `draw_registers_nothing_when_it_cannot_draw`
    /// assume — not the whole screen.
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
        // a request smaller than the screen is honoured exactly
        let small = resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::Center),
            LayerSize::Fixed(10, 4),
        );
        assert_eq!((small.width, small.height), (10, 4));
    }

    /// Now reachable: while every spec asked for the whole screen the flip
    /// arms of `Anchor::Rect` could never run.
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
        // there is room below, so no flip
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

    /// A context menu near the bottom-right must not slide up *over* the
    /// pointer; it flips above and left of it (Adjudication N1, change 2).
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
        // room below and to the right: placed one row under the pointer
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

    /// Two page controls and a popover that registers one control of its
    /// own, so that `Tab` off the opener lands on a control which is
    /// genuinely outside the popover's scope.
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

    /// The size a component asserts from `update` takes effect in the very
    /// next draw — the same frame, no flash (Adjudication N1).
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
        // the resolver still centres it; the component computed no rect
        assert_eq!(
            grown,
            resolve_anchor(
                rt.screen(),
                Anchor::Screen(ScreenAlign::Center),
                LayerSize::Fixed(40, 10)
            )
        );
        // a no-op resize does not request a repaint
        rt.app_mut().resize_request = Some((DLG, LayerSize::Fixed(40, 10)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert_eq!(rt.layer_area(DLG), Some(grown));
    }

    /// Geometry is the only part of a spec that may change while a layer is
    /// open: `kind`, `inert_below`, `restore_focus` and `initial_focus` were
    /// armed when the layer was pushed (§21 item 14).
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
        // `Cx` exposes exactly two spec mutators, both geometric; every other
        // field still reads back as it was armed
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
        // the editor consumes Esc (cancel); the layer stays open
        let r = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(r.is_consumed());
        assert!(rt.is_open(DLG));
        assert!(rt.app().saw(INPUT, "Key("));
        // move to the button: Esc now bubbles to the layer
        let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
        assert_eq!(rt.focus(), Some(OK));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(!rt.is_open(DLG));
    }

    #[test]
    fn layer_id_is_assigned_at_open_not_at_draw() {
        let mut s = dialog_stub();
        // drawn first in `draw`, but opened second
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
        // the popover's row is the pointer target even though it was drawn first
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 21, 2));
        assert!(rt.app().saw(ROW, "Press"));
    }

    #[test]
    fn outside_click_is_layer_less_than_top_or_none() {
        let (mut rt, mut buf) = runtime(dialog_stub());
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        // a click on nothing: outside
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 30, 10));
        assert!(!rt.is_open(DLG));
        assert!(rt.app().saw(DLG, "Dismissed(OutsideClick)"));
        // reopen; a click on a page control (inert, unregistered) is also outside
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        assert!(!rt.is_open(DLG));
        assert!(!rt.app().saw(PAGE_A, "Press"));
    }

    /// §29.8 D2: `Dismiss.focus_out` has a producer.
    ///
    /// A `Popover` is a pointer barrier, not a focus trap, so nothing stops
    /// `Tab` — runtime focus policy, intercepted before any intent is
    /// enqueued — from parking focus on a control *behind* an open popup.
    /// The runtime closes the layer instead, with `DismissReason::FocusOut`,
    /// and the focus restore that `LayerSpec::popover` arms must **not**
    /// fire: restoring to the opener would make `Tab` a no-op and re-create
    /// the legacy key swallow by accident. A `Modal` is excluded by
    /// `dismisses_on_focus_out` itself, whatever its `Dismiss` says.
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

        // the other direction: a modal asking for `Dismiss::ALL` still never
        // dismisses on focus-out, and its trap keeps focus in its own scope
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
        assert!(rt.app().saw(OK, "FocusIn { via: Restore }"));
        let ids: Vec<Id> = rt.ring().reachable().map(|e| e.id).collect();
        assert_eq!(ids, vec![OK, INPUT]);
        let _ = Intent::Cancel;
    }
}
