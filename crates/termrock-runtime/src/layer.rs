#![doc = r" The runtime-owned layer stack (`COMPONENT_ARCHITECTURE.md` §9, §21 items 8, 14, 20)."]
#![doc = r""]
#![doc = r" A layer is opened from `update` with [`LayerSpec`], drawn from `draw`"]
#![doc = r" inside `ui.layer(id, …)`, and composited bottom-to-top after `app.draw`"]
#![doc = r" returns, so z-order is the layer order and never the call order."]
#![doc = r" Placement is one resolver: anchor, flip, then clamp (`Rect::clamp`)."]

use ratatui_core::layout::{Constraint, Position, Rect};

pub use termrock_core::layer_types::{DismissReason, LayerEvent, LayerId};

#[cfg(test)]
use crate::action::ActionKey;
use crate::focus::ScopeId;
use crate::id::Id;

#[doc = r" What a layer traps."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerKind {
    #[doc = r" Focus + pointer trap, dim backdrop, inert below."]
    Modal,
    #[doc = r" Pointer barrier only."]
    Popover,
    #[doc = r" Pointer barrier only, no focus scope of its own."]
    Tooltip,
}

#[doc = r" Where the layer's area is placed."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anchor {
    #[doc = r" Relative to the whole screen."]
    Screen(ScreenAlign),
    #[doc = r" Adjacent to a rect, flipping when there is no room."]
    Rect {
        #[doc = r" The anchor rect."]
        rect: Rect,
        #[doc = r" Which side of the rect."]
        side: Side,
        #[doc = r" Alignment along the other axis."]
        align: CrossAlign,
    },
    #[doc = r" At a point (a tooltip or context menu)."]
    Point(Position),
}

#[doc = r" Screen placement for `Anchor::Screen`."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenAlign {
    #[doc = r" Horizontally centred, vertically in the optical centre (upper third)."]
    Center,
    #[doc = r" Horizontally centred, near the top."]
    UpperThird,
    #[doc = r" Horizontally centred, at the bottom."]
    Bottom,
}

#[doc = r" Which side of an anchor rect."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    #[doc = r" Below the rect."]
    Below,
    #[doc = r" Above the rect."]
    Above,
    #[doc = r" Left of the rect."]
    Left,
    #[doc = r" Right of the rect."]
    Right,
}

#[doc = r" Alignment along the axis perpendicular to `Side`."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrossAlign {
    #[doc = r" Align starts."]
    Start,
    #[doc = r" Centre."]
    Center,
    #[doc = r" Align ends."]
    End,
}

#[doc = r" How a layer may be dismissed."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Dismiss {
    #[doc = r" Esc in the bubble phase."]
    pub esc: bool,
    #[doc = r" A click whose hit is below the layer, or nowhere."]
    pub outside_click: bool,
    #[doc = r" Focus leaving the layer (honoured for `Popover`/`Tooltip` only)."]
    pub focus_out: bool,
}

impl Dismiss {
    #[doc = r" Programmatic only."]
    pub const NONE: Dismiss = Dismiss {
        esc: false,
        outside_click: false,
        focus_out: false,
    };
    #[doc = r" Esc only."]
    pub const ESC: Dismiss = Dismiss {
        esc: true,
        outside_click: false,
        focus_out: false,
    };
    #[doc = r" Esc and outside click."]
    pub const ESC_AND_OUTSIDE: Dismiss = Dismiss {
        esc: true,
        outside_click: true,
        focus_out: false,
    };
    #[doc = r" Esc, outside click and focus out."]
    pub const ALL: Dismiss = Dismiss {
        esc: true,
        outside_click: true,
        focus_out: true,
    };
}

#[doc = r" The backdrop behind a layer."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backdrop {
    #[doc = r" No backdrop."]
    None,
    #[doc = r" Dim the page below."]
    Dim {
        #[doc = r" Keep the last row (the hint bar) undimmed."]
        exclude_footer: bool,
    },
}

#[doc = r" How large a layer asks to be (Adjudication N1)."]
#[doc = r""]
#[doc = r" The resolver clamps to the screen; it never grows a layer, so a `Fixed`"]
#[doc = r#" size is a maximum as well as a request ("size, then clamp, then documented"#]
#[doc = r#" degradation", §9.1). The size is the opener's, the placement is the"#]
#[doc = r" runtime's: a component computes a size, never a rect."]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerSize {
    #[doc = r" The whole screen. The content is responsible for its own internal"]
    #[doc = r" layout; `Anchor` is ignored. Help overlays, file browsers, `TooSmall`."]
    Fill,
    #[doc = r" Exactly `w × h` cells before clamping."]
    Fixed(u16, u16),
}

#[doc = r" A layer's configuration. Construct through the builders."]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LayerSpec {
    #[doc = r" The kind."]
    pub kind: LayerKind,
    #[doc = r" Anchor owner and focus-restore target."]
    pub owner: Id,
    #[doc = r" Placement."]
    pub anchor: Anchor,
    #[doc = r" Dismissal."]
    pub dismiss: Dismiss,
    #[doc = r" Restore focus to the opener on close."]
    pub restore_focus: bool,
    #[doc = r" Initial focus inside the layer."]
    pub initial_focus: Option<Id>,
    #[doc = r" The requested content size (§9.1)."]
    pub size: LayerSize,
    #[doc = r" The backdrop."]
    pub backdrop: Backdrop,
    #[doc = r" No registrations from the layers below."]
    pub inert_below: bool,
}

impl LayerSpec {
    #[doc = r" Modal: `Screen(Center)`, esc + outside click, dim, inert below."]
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

    #[doc = r" Popover: pointer barrier only, no dim."]
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

    #[doc = r" Tooltip at a point: no focus, dismissed by anything."]
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

    #[doc = r" Set the anchor."]
    #[must_use]
    pub const fn anchor(mut self, a: Anchor) -> Self {
        self.anchor = a;
        self
    }

    #[doc = r" Set dismissal."]
    #[must_use]
    pub const fn dismiss(mut self, d: Dismiss) -> Self {
        self.dismiss = d;
        self
    }

    #[doc = r" Set the backdrop."]
    #[must_use]
    pub const fn backdrop(mut self, b: Backdrop) -> Self {
        self.backdrop = b;
        self
    }

    #[doc = r" Set the initial focus."]
    #[must_use]
    pub const fn initial_focus(mut self, id: Id) -> Self {
        self.initial_focus = Some(id);
        self
    }

    #[doc = r" Set the requested size. `LayerSize::Fixed(w, h)` is a request *and* a"]
    #[doc = r" maximum: the resolver clamps to the screen and never grows a layer."]
    #[must_use]
    pub const fn size(mut self, s: LayerSize) -> Self {
        self.size = s;
        self
    }

    #[doc = r" Set inertness of the layers below."]
    #[must_use]
    pub const fn inert_below(mut self, yes: bool) -> Self {
        self.inert_below = yes;
        self
    }

    #[doc = r" Set focus restoration."]
    #[must_use]
    pub const fn restore_focus(mut self, yes: bool) -> Self {
        self.restore_focus = yes;
        self
    }

    #[doc = r" Whether `focus_out` dismissal applies (never for a modal, §21 item 30 A10)."]
    pub const fn dismisses_on_focus_out(&self) -> bool {
        self.dismiss.focus_out && !matches!(self.kind, LayerKind::Modal)
    }
}

#[doc = r" Resolve a layer's area: anchor, flip, then clamp (§9.1)."]
#[doc = r""]
#[doc = r" [`LayerSize::Fill`] yields the whole screen. A `Fixed` size is clipped to"]
#[doc = r" the screen first, then anchored, then flipped if the chosen side has no"]
#[doc = r" room, then `Rect::clamp`ed. A layer is **never grown** to meet its"]
#[doc = r#" request ("size, then clamp, then documented degradation"). A zero"#]
#[doc = r" dimension is an empty layer, not the screen."]
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

#[doc = r" The area a backdrop dims: the screen, minus the last row when the footer"]
#[doc = r" is excluded (`DESIGN.md:537`)."]
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

#[doc = r" One open layer."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpenLayer {
    pub(crate) id: Id,
    pub(crate) layer: LayerId,
    pub(crate) spec: LayerSpec,
    #[doc = r" Focus to restore when this layer closes."]
    pub(crate) restore_to: Option<Id>,
}

impl OpenLayer {
    pub(crate) const fn scope(&self) -> ScopeId {
        ScopeId::new(self.id)
    }
}

#[doc = r" The runtime's layer stack."]
#[derive(Clone, Debug)]
pub(crate) struct LayerStack {
    open: Vec<OpenLayer>,
    #[doc = r" Events staged for delivery on the next `handle`."]
    pending: Vec<(Id, LayerEvent)>,
    #[doc = r" The next monotonic layer number. It is not reduced when layers close."]
    next: u32,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self {
            open: Vec::new(),
            pending: Vec::new(),
            next: 1,
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

    #[doc = r" The spec of an open layer, mutable. Only the geometry (`size`,"]
    #[doc = r" `anchor`) may change while a layer is open: `kind`, `inert_below`,"]
    #[doc = r" `restore_focus` and `initial_focus` were armed by the runtime when the"]
    #[doc = r" layer was pushed and re-deriving them would desync the focus scope and"]
    #[doc = r" the inert floor (§21 item 14)."]
    pub(crate) fn spec_mut(&mut self, id: Id) -> Option<&mut LayerSpec> {
        self.open
            .iter_mut()
            .find(|l| l.id == id)
            .map(|l| &mut l.spec)
    }

    pub(crate) fn top_layer(&self) -> Option<&OpenLayer> {
        self.open.last()
    }

    #[doc = r" The lowest layer with `inert_below`; everything beneath it is inert."]
    pub(crate) fn inert_floor(&self) -> LayerId {
        self.open
            .iter()
            .rev()
            .find(|l| l.spec.inert_below)
            .map_or(LayerId::PAGE, |l| l.layer)
    }

    #[doc = r" Open a layer; its `LayerId` is assigned monotonically. Re-opening an"]
    #[doc = r" open id is a no-op."]
    pub(crate) fn open(
        &mut self,
        id: Id,
        spec: LayerSpec,
        restore_to: Option<Id>,
    ) -> Option<LayerId> {
        if self.is_open(id) {
            return None;
        }
        let layer = LayerId(u16::try_from(self.next).ok()?);
        self.next = self.next.saturating_add(1);
        self.open.push(OpenLayer {
            id,
            layer,
            spec,
            restore_to,
        });
        self.pending.push((id, LayerEvent::Opened));
        Some(layer)
    }

    #[doc = r" Close a layer and everything above it. Returns the closed layers,"]
    #[doc = r" bottom-most first."]
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

    #[doc = r" The field was never a minimum: the resolver clamps down and never"]
    #[doc = r" grows (Adjudication N1). A zero dimension is an *empty* layer, which"]
    #[doc = r" is what §16.2 case 19 and `draw_registers_nothing_when_it_cannot_draw`"]
    #[doc = r" assume — not the whole screen."]
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

    #[doc = r" Now reachable: while every spec asked for the whole screen the flip"]
    #[doc = r" arms of `Anchor::Rect` could never run."]
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

    #[doc = r" A context menu near the bottom-right must not slide up *over* the"]
    #[doc = r" pointer; it flips above and left of it (Adjudication N1, change 2)."]
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

    #[doc = r" Two page controls and a popover that registers one control of its"]
    #[doc = r" own, so that `Tab` off the opener lands on a control which is"]
    #[doc = r" genuinely outside the popover's scope."]
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

    #[doc = r" The size a component asserts from `update` takes effect in the very"]
    #[doc = r" next draw — the same frame, no flash (Adjudication N1)."]
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

    #[doc = r" Geometry is the only part of a spec that may change while a layer is"]
    #[doc = r" open: `kind`, `inert_below`, `restore_focus` and `initial_focus` were"]
    #[doc = r" armed when the layer was pushed (§21 item 14)."]
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

    #[doc = r" §29.8 D2: `Dismiss.focus_out` has a producer."]
    #[doc = r""]
    #[doc = r" A `Popover` is a pointer barrier, not a focus trap, so nothing stops"]
    #[doc = r" `Tab` — runtime focus policy, intercepted before any intent is"]
    #[doc = r" enqueued — from parking focus on a control *behind* an open popup."]
    #[doc = r" The runtime closes the layer instead, with `DismissReason::FocusOut`,"]
    #[doc = r" and the focus restore that `LayerSpec::popover` arms must **not**"]
    #[doc = r" fire: restoring to the opener would make `Tab` a no-op and re-create"]
    #[doc = r" the legacy key swallow by accident. A `Modal` is excluded by"]
    #[doc = r" `dismisses_on_focus_out` itself, whatever its `Dismiss` says."]
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
