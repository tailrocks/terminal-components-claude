use ratatui_core::layout::{Position, Rect};

use crate::focus::FocusRing;
use crate::hit::{Hit, RegionKind, Registry, ReleaseFrom};
use crate::id::{Id, PartRef};
use crate::layer::LayerId;
use termrock_core::outcome::Outcome;

pub(crate) fn target_eligible(
    ring: &FocusRing,
    top: LayerId,
    owner: Id,
    layer: LayerId,
    kind: RegionKind,
) -> bool {
    layer == top
        && matches!(kind, RegionKind::Control | RegionKind::Part)
        && !ring.entry(owner).is_some_and(|entry| entry.disabled)
}

pub(crate) fn target_area(
    registry: &Registry,
    ring: &FocusRing,
    top: LayerId,
    owner: Id,
    part: PartRef,
) -> Option<Rect> {
    registry.regions().iter().rev().find_map(|region| {
        (region.owner == owner
            && region.pointer_enabled()
            && region.part == part
            && !region.area.is_empty()
            && target_eligible(ring, top, owner, region.layer, region.kind))
        .then_some(region.area)
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capture {
    pub owner: Id,
    pub part: PartRef,
    pub origin: Position,
    pub area: Rect,
    pub generation: u32,
}

impl Capture {
    pub const fn local(&self, pos: Position) -> Position {
        Position {
            x: pos.x.saturating_sub(self.area.x),
            y: pos.y.saturating_sub(self.area.y),
        }
    }

    pub const fn contains(&self, pos: Position) -> bool {
        self.area.contains(pos)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CaptureSlot {
    live: Option<Capture>,
}

impl CaptureSlot {
    pub(crate) fn claim(&mut self, c: Capture) -> bool {
        if self.live.is_some() {
            return false;
        }
        self.live = Some(c);
        true
    }

    pub(crate) const fn get(&self) -> Option<Capture> {
        self.live
    }

    pub(crate) fn release(&mut self) -> Option<Capture> {
        self.live.take()
    }

    pub(crate) fn release_if_stale(&mut self, reg: &Registry) {
        if let Some(c) = self.live
            && (!reg.has_owner(c.owner) || reg.generation() < c.generation)
        {
            self.live = None;
        }
    }
}

/// What a right-button click outside a popup does (ported from PR #26
/// `src/core/pointer.rs::SecondaryOutside`, commit f4867e9b5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecondaryOutside {
    #[default]
    Reopen,

    Close,

    Ignore,
}

/// Where a secondary (right button) press is routed (ported from PR #26
/// `src/core/pointer.rs::SecondaryRoute`, commit f4867e9b5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryRoute {
    Nothing,

    Target(Hit),

    Outside { backdrop: Id, reopen: Option<Hit> },

    Swallowed,
}

/// Tracks the secondary button across a press-drag-release so a drag that
/// leaves a row can still deliver its release (ported from PR #26
/// `src/core/pointer.rs` secondary handling, commit f4867e9b5).
#[derive(Debug, Clone, Copy, Default)]
pub struct SecondaryTracker {
    hover: Option<Hit>,
    secondary_at: Option<Position>,
}

impl SecondaryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn hover(&self) -> Option<Hit> {
        self.hover
    }

    pub fn on_secondary(&mut self, hits: &Registry, pos: Position) -> Option<Hit> {
        self.secondary_at = Some(pos);
        let hit = hits.hit(pos);
        self.hover = hit;
        hit
    }

    pub fn route_secondary(
        &mut self,
        hits: &Registry,
        pos: Position,
        policy: SecondaryOutside,
    ) -> SecondaryRoute {
        let Some(hit) = self.on_secondary(hits, pos) else {
            return SecondaryRoute::Nothing;
        };
        let Some((backdrop, beneath)) = backdrop_at(hits, pos) else {
            return SecondaryRoute::Target(hit);
        };
        match policy {
            SecondaryOutside::Ignore => SecondaryRoute::Swallowed,
            SecondaryOutside::Close => SecondaryRoute::Outside {
                backdrop,
                reopen: None,
            },
            SecondaryOutside::Reopen => {
                self.hover = beneath;
                SecondaryRoute::Outside {
                    backdrop,
                    reopen: beneath,
                }
            }
        }
    }

    pub fn on_secondary_drag(&mut self, hits: &Registry, pos: Position) -> Outcome {
        let was = self.hover;
        self.hover = hits.hit(pos);
        if self.hover != was {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_secondary_up(&mut self, hits: &Registry, pos: Position) -> Option<Hit> {
        let from = self.secondary_at.take()?;
        if from == pos {
            return None;
        }
        let hit = hits.hit(pos)?;
        hits.accepts_release(hit.owner, ReleaseFrom::Secondary)
            .then_some(hit)
    }
}

/// The topmost backdrop at `pos` and the target beneath it (ported from
/// PR #26 `src/core/hit.rs::backdrop_at`, commit f4867e9b5; adapted to
/// parity's layers instead of a barrier stack).
pub(crate) fn backdrop_at(hits: &Registry, pos: Position) -> Option<(Id, Option<Hit>)> {
    let top = hits.region_at(pos)?;
    if top.kind != RegionKind::Backdrop {
        return None;
    }
    let beneath = hits
        .pointer_regions()
        .rev()
        .find(|(_, r)| {
            r.layer < top.layer
                && r.area.contains(pos)
                && r.pointer_enabled()
                && matches!(r.kind, RegionKind::Control | RegionKind::Part)
        })
        .map(|(_, r)| Hit {
            owner: r.owner,
            part: r.part,
            layer: r.layer,
            kind: r.kind,
            local: Position::new(
                pos.x.saturating_sub(r.area.x),
                pos.y.saturating_sub(r.area.y),
            ),
        });
    Some((top.owner, beneath))
}

/// What a scrollbar bar should do for a pointer phase (ported from PR #26
/// `src/core/pointer.rs::scroll_input`, commit f4867e9b5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarInput {
    Press,

    Drag,

    Release,

    None,
}

pub const fn bar_input_for(phase: termrock_core::intent::Phase) -> BarInput {
    use termrock_core::intent::Phase;
    match phase {
        Phase::Press => BarInput::Press,
        Phase::DragStart | Phase::Drag => BarInput::Drag,
        Phase::Click | Phase::Release | Phase::DragEnd => BarInput::Release,
        _ => BarInput::None,
    }
}
