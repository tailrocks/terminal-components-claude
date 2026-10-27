pub use termrock_core::geometry::{Axes, Headroom};

use ratatui_core::layout::{Position, Rect};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::diagnostics::Diagnostic;
use crate::event::Axis;
use crate::id::{Id, Part, PartRef};
use crate::layer::LayerId;

static NEXT_SCROLL_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

fn next_scroll_registry_id(counter: &AtomicU64) -> Option<u64> {
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .ok()
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ScrollRegistryIdentity(Option<u64>);

impl core::fmt::Debug for ScrollRegistryIdentity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(identity) => core::fmt::Debug::fmt(&identity, f),
            None => f.write_str("exhausted"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum RegionKind {
    Control,
    Part,
    Scroll,
    /// A non-interactive region that participates only in top-layer hover.
    HoverOnly,
    Decorative,
    /// A light-dismiss backdrop registered behind a popup or modal.
    Backdrop,
}

/// A scrollbar bar registered by its owning widget (ported from PR #26
/// `src/core/hit.rs::Bar`, commit f4867e9b5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bar {
    /// The bar's own id (the track / thumb control).
    pub id: Id,
    /// The widget the bar scrolls.
    pub owner: Id,
    /// Which axis the bar scrolls.
    pub axis: Axis,
    /// The track area, including the thumb.
    pub track: Rect,
}

/// Whose release a widget accepts (ported from PR #26 `src/core/hit.rs::ReleaseFrom`,
/// commit f4867e9b5): a drag that started on a widget keeps receiving
/// releases, and a secondary-button release may be accepted while pressed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReleaseFrom {
    /// The release of a press that started on `id`.
    Press(Id),
    /// A secondary (right button) release.
    Secondary,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hit {
    pub owner: Id,
    pub part: PartRef,
    pub layer: LayerId,
    pub kind: RegionKind,
    pub local: Position,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Region {
    /// The region's own control id (ported from PR #26
    /// `HitRegion::id`, commit f4867e9b5): equal to `owner` except where a
    /// registration splits the two — a thumb keeps its bar's id and a
    /// backdrop keeps the id it was hit-tested as.
    pub id: Id,
    pub owner: Id,
    pub part: PartRef,
    pub area: Rect,
    pub layer: LayerId,
    pub kind: RegionKind,
    pub generation: u32,
    scroll: Option<(Axes, Headroom)>,
    scroll_routing: ScrollRouting,
    pointer_enabled: bool,
    pass_through: bool,
}

/// Opaque identity for one opt-in wheel registration in one presented registry.
///
/// A key can link only registrations created in the same registry generation.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScrollRouteKey {
    registry_id: u64,
    registry_epoch: u64,
    generation: u32,
    registration_index: usize,
}

impl core::fmt::Debug for ScrollRouteKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ScrollRouteKey { .. }")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TextDragRegistration {
    pub(crate) owner: Id,
    pub(crate) part: PartRef,
    pub(crate) area: Rect,
    pub(crate) layer: LayerId,
    pub(crate) axes: Axes,
    pub(crate) headroom: Headroom,
    pub(crate) source_revision: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TextDragRegion {
    owner: Id,
    part: PartRef,
    area: Rect,
    layer: LayerId,
    generation: u32,
    axes: Axes,
    headroom: Headroom,
    source_revision: u32,
}

impl TextDragRegion {
    pub(crate) const fn owner(self) -> Id {
        self.owner
    }

    pub(crate) const fn part(self) -> PartRef {
        self.part
    }

    pub(crate) const fn area(self) -> Rect {
        self.area
    }

    pub(crate) const fn layer(self) -> LayerId {
        self.layer
    }

    pub(crate) const fn generation(self) -> u32 {
        self.generation
    }

    pub(crate) const fn axes(self) -> Axes {
        self.axes
    }

    pub(crate) const fn source_revision(self) -> u32 {
        self.source_revision
    }

    pub(crate) const fn can_scroll_at(self, pos: Position) -> bool {
        (pos.y < self.area.y && self.headroom.up > 0)
            || (pos.y >= self.area.bottom() && self.headroom.down > 0)
            || (self.axes.handles(Axis::H) && pos.x < self.area.x && self.headroom.left > 0)
            || (self.axes.handles(Axis::H) && pos.x >= self.area.right() && self.headroom.right > 0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum ScrollRouting {
    #[default]
    Capture,
    PassiveReveal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrollCandidate {
    hit: Hit,
    headroom: Headroom,
    passive_reveal: bool,
    registration_index: usize,
}

impl ScrollCandidate {
    pub(crate) const fn hit(self) -> Hit {
        self.hit
    }

    pub(crate) const fn headroom(self) -> Headroom {
        self.headroom
    }

    pub(crate) const fn passive_reveal(self) -> bool {
        self.passive_reveal
    }

    pub(crate) const fn registration_index(self) -> usize {
        self.registration_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RoutedScroll {
    source_revision: u64,
    parent_index: Option<usize>,
    policy: ScrollRoutePolicy,
    invalid: bool,
}

enum PointerRegions<'a> {
    Physical(std::iter::Enumerate<std::slice::Iter<'a, Region>>),
    Virtual {
        order: std::iter::Enumerate<std::slice::Iter<'a, usize>>,
        regions: &'a [Region],
    },
}

impl<'a> Iterator for PointerRegions<'a> {
    type Item = (usize, &'a Region);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Physical(regions) => regions.next(),
            Self::Virtual { order, regions } => {
                let regions: &'a [Region] = regions;
                loop {
                    let (rank, physical_index) = order.next()?;
                    if let Some(region) = regions.get(*physical_index) {
                        return Some((rank, region));
                    }
                }
            }
        }
    }
}

impl<'a> DoubleEndedIterator for PointerRegions<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Physical(regions) => regions.next_back(),
            Self::Virtual { order, regions } => {
                let regions: &'a [Region] = regions;
                loop {
                    let (rank, physical_index) = order.next_back()?;
                    if let Some(region) = regions.get(*physical_index) {
                        return Some((rank, region));
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollRoutePolicy {
    LinkedParent,
    HitStack,
    EventLocalHitStack,
}

impl Region {
    pub const fn pointer_enabled(&self) -> bool {
        self.pointer_enabled
    }

    pub const fn scroll(&self) -> Option<(Axes, Headroom)> {
        self.scroll
    }

    /// Whether a click on this backdrop also reaches the layer beneath.
    pub const fn pass_through(&self) -> bool {
        self.pass_through
    }
}

#[derive(Debug)]
pub struct Registry {
    regions: Vec<Region>,
    pointer_order: Option<Vec<usize>>,
    routed_scrolls: Vec<Option<RoutedScroll>>,
    text_drag_regions: Vec<TextDragRegion>,
    registry_id: ScrollRegistryIdentity,
    generation: u32,
    route_epoch: u64,
    route_epoch_exhausted: bool,
    has_pointer_exclusions: bool,
    bars: Vec<Bar>,
    releases: Vec<(Id, ReleaseFrom)>,
}

impl Clone for Registry {
    fn clone(&self) -> Self {
        self.clone_with_registry_id(&NEXT_SCROLL_REGISTRY_ID)
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Registry {
    fn clone_with_registry_id(&self, next: &AtomicU64) -> Self {
        Self {
            regions: self.regions.clone(),
            pointer_order: self.pointer_order.clone(),
            routed_scrolls: self.routed_scrolls.clone(),
            text_drag_regions: self.text_drag_regions.clone(),
            registry_id: ScrollRegistryIdentity(next_scroll_registry_id(next)),
            generation: self.generation,
            route_epoch: self.route_epoch,
            route_epoch_exhausted: self.route_epoch_exhausted,
            has_pointer_exclusions: self.has_pointer_exclusions,
            bars: self.bars.clone(),
            releases: self.releases.clone(),
        }
    }

    pub fn new(generation: u32) -> Self {
        Self::new_with_registry_id(generation, &NEXT_SCROLL_REGISTRY_ID)
    }

    fn new_with_registry_id(generation: u32, next: &AtomicU64) -> Self {
        Registry {
            regions: Vec::new(),
            pointer_order: None,
            routed_scrolls: Vec::new(),
            text_drag_regions: Vec::new(),
            registry_id: ScrollRegistryIdentity(next_scroll_registry_id(next)),
            generation,
            route_epoch: 1,
            route_epoch_exhausted: false,
            has_pointer_exclusions: false,
            bars: Vec::new(),
            releases: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self, generation: u32) {
        self.regions.clear();
        self.pointer_order = None;
        self.routed_scrolls.clear();
        self.text_drag_regions.clear();
        self.generation = generation;
        if let Some(next) = self.route_epoch.checked_add(1) {
            self.route_epoch = next;
        } else {
            self.route_epoch_exhausted = true;
        }
        self.has_pointer_exclusions = false;
        self.bars.clear();
        self.releases.clear();
    }

    pub const fn generation(&self) -> u32 {
        self.generation
    }

    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub(crate) fn pointer_regions(&self) -> impl DoubleEndedIterator<Item = (usize, &Region)> + '_ {
        match self.pointer_order.as_deref() {
            Some(order) => PointerRegions::Virtual {
                order: order.iter().enumerate(),
                regions: &self.regions,
            },
            None => PointerRegions::Physical(self.regions.iter().enumerate()),
        }
    }

    pub(crate) fn promote_part_hit_suffix(&mut self, start: usize, part: PartRef) {
        let len = self.regions.len();
        if start >= len {
            return;
        }

        let mut matching_seen = false;
        let mut needs_partition = false;
        for rank in start..len {
            let physical_index = match self.pointer_order.as_ref() {
                Some(order) => {
                    let Some(physical_index) = order.get(rank) else {
                        return;
                    };
                    *physical_index
                }
                None => rank,
            };
            let Some(region) = self.regions.get(physical_index) else {
                return;
            };
            if Self::is_requested_part(region, part) {
                matching_seen = true;
            } else if matching_seen {
                needs_partition = true;
                break;
            }
        }

        if !needs_partition {
            return;
        }
        if self.pointer_order.is_none() {
            self.pointer_order = Some((0..len).collect());
        }
        let regions = &self.regions;
        if let Some(order) = self.pointer_order.as_mut()
            && let Some(suffix) = order.get_mut(start..)
        {
            suffix.sort_by_key(|physical_index| {
                regions
                    .get(*physical_index)
                    .is_some_and(|region| Self::is_requested_part(region, part))
            });
        }
    }

    fn is_requested_part(region: &Region, part: PartRef) -> bool {
        region.kind == RegionKind::Part && region.part == part
    }

    pub fn register_control(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
    ) -> Option<Diagnostic> {
        if area.is_empty() {
            return None;
        }
        let dup = self
            .regions
            .iter()
            .find(|r| r.owner == owner && r.kind == RegionKind::Control)
            .map(|r| Diagnostic::DuplicateId {
                id: owner,
                first: r.area,
                second: area,
            });
        self.push(
            owner,
            PartRef::of(Part::CONTAINER),
            area,
            layer,
            RegionKind::Control,
            None,
        );
        dup
    }

    pub(crate) fn register_keyboard_control(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
    ) -> Option<Diagnostic> {
        if area.is_empty() {
            return None;
        }
        let diagnostic = self.register_control(owner, area, layer);

        for region in self
            .regions
            .iter_mut()
            .filter(|region| region.owner == owner)
        {
            region.pointer_enabled = false;
        }
        self.has_pointer_exclusions = true;
        diagnostic
    }

    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect, layer: LayerId) {
        self.push(owner, part, area, layer, RegionKind::Part, None);
    }

    pub(crate) fn register_hover_part(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
    ) {
        self.push(owner, part, area, layer, RegionKind::HoverOnly, None);
    }

    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect, layer: LayerId) {
        self.push(owner, part, area, layer, RegionKind::Decorative, None);
    }

    pub fn register_scroll(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
        axes: Axes,
        head: Headroom,
    ) {
        self.push(
            owner,
            PartRef::of(Part::CONTAINER),
            area,
            layer,
            RegionKind::Scroll,
            Some((axes, head)),
        );
    }

    /// Register a scroll target that can explicitly acknowledge a wheel route.
    ///
    /// The returned key is valid only for this registry frame. Existing scroll
    /// registrations remain single-target unless they use this opt-in method.
    pub fn register_routed_scroll(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
        axes: Axes,
        head: Headroom,
        source_revision: u64,
    ) -> Option<ScrollRouteKey> {
        self.register_routed_wheel_target(
            owner,
            PartRef::of(Part::CONTAINER),
            area,
            layer,
            axes,
            head,
            source_revision,
            ScrollRoutePolicy::LinkedParent,
        )
    }

    /// Register a scroll-only, source-qualified wheel target in the opt-in hit stack.
    ///
    /// This creates a `Scroll` region with the supplied part, while leaving any
    /// ordinary `Control` or `Part` registration unchanged. Only other targets
    /// registered with this method can receive fallback offers.
    pub fn register_hit_stack_wheel(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        axes: Axes,
        source_revision: u64,
    ) -> Option<ScrollRouteKey> {
        self.register_routed_wheel_target(
            owner,
            part,
            area,
            layer,
            axes,
            Headroom::default(),
            source_revision,
            ScrollRoutePolicy::HitStack,
        )
    }

    /// Register an event-local, source-qualified target in the opt-in wheel hit stack.
    ///
    /// Repeated owner/part registrations are allowed when they resolve to one
    /// eligible scroll target at the event position and axis. Exact duplicate
    /// registrations coalesce; conflicting eligible registrations stop
    /// fallback. Only pointer-enabled registrations on the same layer that
    /// contain the position and handle the axis participate. The owner must
    /// resolve its live source from the event position and axis before
    /// acknowledging an attempt.
    ///
    /// The returned key is valid only for this registry frame.
    pub fn register_event_local_hit_stack_wheel(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        axes: Axes,
        source_revision: u64,
    ) -> Option<ScrollRouteKey> {
        self.register_routed_wheel_target(
            owner,
            part,
            area,
            layer,
            axes,
            Headroom::default(),
            source_revision,
            ScrollRoutePolicy::EventLocalHitStack,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one registry helper creates an exact source-qualified wheel registration"
    )]
    fn register_routed_wheel_target(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        axes: Axes,
        head: Headroom,
        source_revision: u64,
        policy: ScrollRoutePolicy,
    ) -> Option<ScrollRouteKey> {
        let registry_id = self.registry_id.0?;
        if area.is_empty() || self.route_epoch_exhausted {
            return None;
        }
        let registration_index = self.regions.len();
        self.push(
            owner,
            part,
            area,
            layer,
            RegionKind::Scroll,
            Some((axes, head)),
        );
        if self.regions.len() == registration_index {
            return None;
        }
        let route = self.routed_scrolls.get_mut(registration_index)?;
        *route = Some(RoutedScroll {
            source_revision,
            parent_index: None,
            policy,
            invalid: false,
        });
        Some(ScrollRouteKey {
            registry_id,
            registry_epoch: self.route_epoch,
            generation: self.generation,
            registration_index,
        })
    }

    /// Link an opt-in child target to its immediate containing parent target.
    ///
    /// The parent must be the next eligible registration below the child in
    /// reverse hit order. Invalid links disable routing for the child.
    pub fn link_scroll_parent(&mut self, child: ScrollRouteKey, parent: ScrollRouteKey) -> bool {
        let Some(child_index) = self.valid_route_key(child) else {
            return false;
        };
        let Some(parent_index) = self.valid_route_key(parent) else {
            self.invalidate_routed_scroll(child_index);
            return false;
        };
        let valid_pair = child_index > parent_index
            && self
                .routed_scrolls
                .get(child_index)
                .and_then(Option::as_ref)
                .is_some_and(|route| route.policy == ScrollRoutePolicy::LinkedParent)
            && self
                .routed_scrolls
                .get(parent_index)
                .and_then(Option::as_ref)
                .is_some_and(|route| route.policy == ScrollRoutePolicy::LinkedParent)
            && self.regions.get(child_index).is_some_and(|child_region| {
                self.regions.get(parent_index).is_some_and(|parent_region| {
                    child_region.kind == RegionKind::Scroll
                        && parent_region.kind == RegionKind::Scroll
                        && child_region.layer == parent_region.layer
                        && parent_region.area.intersection(child_region.area) == child_region.area
                        && child_region.scroll_routing == ScrollRouting::Capture
                        && parent_region.scroll_routing == ScrollRouting::Capture
                })
            });
        if !valid_pair || self.would_form_scroll_route_cycle(child_index, parent_index) {
            self.invalidate_routed_scroll(child_index);
            return false;
        }
        let Some(child_route) = self
            .routed_scrolls
            .get_mut(child_index)
            .and_then(Option::as_mut)
        else {
            return false;
        };
        if child_route.invalid {
            return false;
        }
        match child_route.parent_index {
            None => {
                child_route.parent_index = Some(parent_index);
                true
            }
            Some(existing) if existing == parent_index => true,
            Some(_) => {
                child_route.invalid = true;
                false
            }
        }
    }

    fn valid_route_key(&self, key: ScrollRouteKey) -> Option<usize> {
        let registry_id = self.registry_id.0?;
        (key.registry_epoch == self.route_epoch
            && key.registry_id == registry_id
            && key.generation == self.generation
            && !self.route_epoch_exhausted
            && self
                .routed_scrolls
                .get(key.registration_index)
                .is_some_and(Option::is_some))
        .then_some(key.registration_index)
    }

    fn invalidate_routed_scroll(&mut self, index: usize) {
        if let Some(Some(route)) = self.routed_scrolls.get_mut(index) {
            route.invalid = true;
        }
    }

    fn would_form_scroll_route_cycle(&self, child_index: usize, parent_index: usize) -> bool {
        let mut current = Some(parent_index);
        for _ in 0..=self.routed_scrolls.len() {
            let Some(index) = current else {
                return false;
            };
            if index == child_index {
                return true;
            }
            current = self
                .routed_scrolls
                .get(index)
                .and_then(Option::as_ref)
                .filter(|route| !route.invalid)
                .and_then(|route| route.parent_index);
        }
        true
    }

    pub(crate) fn routed_scroll_info(
        &self,
        registration_index: usize,
    ) -> Option<(ScrollRouteKey, u64)> {
        let registry_id = self.registry_id.0?;
        if self.route_epoch_exhausted {
            return None;
        }
        let route = self.routed_scrolls.get(registration_index)?.as_ref()?;
        let region = self.regions.get(registration_index)?;
        if route.invalid || region.kind != RegionKind::Scroll {
            return None;
        }
        Some((
            ScrollRouteKey {
                registry_id,
                registry_epoch: self.route_epoch,
                generation: self.generation,
                registration_index,
            },
            route.source_revision,
        ))
    }

    pub(crate) fn routed_scroll_source_revision(&self, key: ScrollRouteKey) -> Option<u64> {
        let index = self.valid_route_key(key)?;
        let route = self.routed_scrolls.get(index)?.as_ref()?;
        let region = self.regions.get(index)?;
        (!route.invalid && region.kind == RegionKind::Scroll).then_some(route.source_revision)
    }

    pub(crate) fn routed_scroll_policy(&self, key: ScrollRouteKey) -> Option<ScrollRoutePolicy> {
        let index = self.valid_route_key(key)?;
        let route = self.routed_scrolls.get(index)?.as_ref()?;
        let region = self.regions.get(index)?;
        (!route.invalid && region.kind == RegionKind::Scroll).then_some(route.policy)
    }

    pub(crate) fn next_scroll_route_parent(
        &self,
        key: ScrollRouteKey,
        axis: Axis,
        pos: Position,
        layer: LayerId,
    ) -> Option<(ScrollRouteKey, u64)> {
        let child_index = self.valid_route_key(key)?;
        let child_region = self.regions.get(child_index)?;
        let child_route = self.routed_scrolls.get(child_index)?.as_ref()?;
        let parent_index = child_route.parent_index?;
        if child_route.invalid
            || !self.route_region_is_eligible(child_index, axis, pos, layer)
            || self.route_pair_is_ambiguous(child_index, parent_index, axis, pos, layer)
        {
            return None;
        }
        let parent_region = self.regions.get(parent_index)?;
        let parent_route = self.routed_scrolls.get(parent_index)?.as_ref()?;
        if parent_route.invalid
            || !self.route_region_is_eligible(parent_index, axis, pos, layer)
            || parent_region.area.intersection(child_region.area) != child_region.area
        {
            return None;
        }

        let mut saw_child = false;
        for candidate in self.scroll_candidates(pos, axis, layer) {
            let candidate_index = candidate.registration_index();
            if candidate_index == child_index {
                saw_child = true;
                continue;
            }
            if !saw_child {
                continue;
            }
            if candidate_index == parent_index {
                return self.routed_scroll_info(parent_index);
            }
            if self.same_routed_registration(child_index, candidate_index) {
                continue;
            }
            return None;
        }
        None
    }

    pub(crate) fn next_scroll_route_target(
        &self,
        key: ScrollRouteKey,
        axis: Axis,
        pos: Position,
        layer: LayerId,
        offered_owners: &[Id],
    ) -> Option<(ScrollRouteKey, u64)> {
        match self.routed_scroll_policy(key)? {
            ScrollRoutePolicy::LinkedParent => self.next_scroll_route_parent(key, axis, pos, layer),
            policy @ (ScrollRoutePolicy::HitStack | ScrollRoutePolicy::EventLocalHitStack) => {
                self.next_hit_stack_wheel_target(key, axis, pos, layer, policy, offered_owners)
            }
        }
    }

    fn next_hit_stack_wheel_target(
        &self,
        key: ScrollRouteKey,
        axis: Axis,
        pos: Position,
        layer: LayerId,
        policy: ScrollRoutePolicy,
        offered_owners: &[Id],
    ) -> Option<(ScrollRouteKey, u64)> {
        let current_index = self.valid_route_key(key)?;
        let current_route = self.routed_scrolls.get(current_index)?.as_ref()?;
        if current_route.invalid
            || current_route.policy != policy
            || !self.route_region_is_eligible(current_index, axis, pos, layer)
            || !self.hit_stack_route_is_unambiguous(current_index, axis, pos, layer, policy)
            || !self.hit_stack_route_is_clear_above(
                current_index,
                axis,
                pos,
                layer,
                policy,
                offered_owners,
            )
        {
            return None;
        }

        let mut after_current = false;
        for (index, region) in self.regions.iter().enumerate().rev() {
            if index == current_index {
                after_current = true;
                continue;
            }
            if !after_current
                || !region.pointer_enabled
                || region.layer != layer
                || !region.area.contains(pos)
                || region.kind == RegionKind::HoverOnly
            {
                continue;
            }

            if offered_owners.contains(&region.owner) {
                if policy == ScrollRoutePolicy::EventLocalHitStack
                    && region.kind == RegionKind::Scroll
                    && region.scroll.is_some_and(|(axes, _)| axes.handles(axis))
                    && !self.scroll_region_matches_policy(index, policy)
                {
                    return None;
                }
                continue;
            }

            match region.kind {
                RegionKind::Control | RegionKind::Part => {
                    if self.hit_stack_region_is_enrolled_part(index, axis, pos, layer, policy) {
                        continue;
                    }
                    return None;
                }
                RegionKind::Scroll => {
                    let (axes, _) = region.scroll?;
                    if !axes.handles(axis) {
                        continue;
                    }
                    let route = self.routed_scrolls.get(index).and_then(Option::as_ref)?;
                    if route.invalid
                        || route.policy != policy
                        || region.scroll_routing != ScrollRouting::Capture
                        || !self.hit_stack_route_is_unambiguous(index, axis, pos, layer, policy)
                    {
                        return None;
                    }
                    return self.routed_scroll_info(index);
                }
                RegionKind::HoverOnly => {}
                RegionKind::Decorative | RegionKind::Backdrop => return None,
            }
        }
        None
    }

    fn hit_stack_route_is_unambiguous(
        &self,
        selected_index: usize,
        axis: Axis,
        pos: Position,
        layer: LayerId,
        policy: ScrollRoutePolicy,
    ) -> bool {
        let Some(selected) = self.regions.get(selected_index) else {
            return false;
        };
        match policy {
            ScrollRoutePolicy::HitStack => {
                self.regions.iter().enumerate().all(|(index, region)| {
                    index == selected_index
                        || region.kind != RegionKind::Scroll
                        || region.layer != layer
                        || (region.owner, region.part) != (selected.owner, selected.part)
                        || self.same_routed_registration(selected_index, index)
                })
            }
            ScrollRoutePolicy::EventLocalHitStack => {
                for candidate in self.scroll_candidates(pos, axis, layer) {
                    let index = candidate.registration_index();
                    let Some(region) = self.regions.get(index) else {
                        return false;
                    };
                    if (region.owner, region.part) == (selected.owner, selected.part)
                        && !self.same_routed_registration(selected_index, index)
                    {
                        return false;
                    }
                }
                true
            }
            ScrollRoutePolicy::LinkedParent => false,
        }
    }

    fn hit_stack_route_is_clear_above(
        &self,
        selected_index: usize,
        axis: Axis,
        pos: Position,
        layer: LayerId,
        policy: ScrollRoutePolicy,
        offered_owners: &[Id],
    ) -> bool {
        for (index, region) in self.regions.iter().enumerate().rev() {
            if index <= selected_index {
                break;
            }
            if !region.pointer_enabled || region.layer != layer || !region.area.contains(pos) {
                continue;
            }
            if offered_owners.contains(&region.owner)
                && policy == ScrollRoutePolicy::EventLocalHitStack
                && region.kind == RegionKind::Scroll
                && region.scroll.is_some_and(|(axes, _)| axes.handles(axis))
                && !self.scroll_region_matches_policy(index, policy)
            {
                return false;
            }
            match region.kind {
                RegionKind::HoverOnly => continue,
                RegionKind::Control | RegionKind::Part => {
                    if offered_owners.contains(&region.owner)
                        && self.hit_stack_region_is_enrolled_part(index, axis, pos, layer, policy)
                    {
                        continue;
                    }
                    return false;
                }
                RegionKind::Scroll => {
                    if !region.scroll.is_some_and(|(axes, _)| axes.handles(axis)) {
                        continue;
                    }
                    let is_offered_hit_stack_target = offered_owners.contains(&region.owner)
                        && self
                            .routed_scrolls
                            .get(index)
                            .and_then(Option::as_ref)
                            .is_some_and(|route| route.policy == policy && !route.invalid);
                    if is_offered_hit_stack_target
                        || self.same_routed_registration(selected_index, index)
                    {
                        continue;
                    }
                    return false;
                }
                RegionKind::Decorative | RegionKind::Backdrop => return false,
            }
        }
        true
    }

    fn hit_stack_region_is_enrolled_part(
        &self,
        region_index: usize,
        axis: Axis,
        pos: Position,
        layer: LayerId,
        policy: ScrollRoutePolicy,
    ) -> bool {
        let Some(region) = self.regions.get(region_index) else {
            return false;
        };
        if !matches!(region.kind, RegionKind::Control | RegionKind::Part) {
            return false;
        }
        self.regions
            .iter()
            .enumerate()
            .any(|(target_index, target)| {
                target.owner == region.owner
                    && target.part == region.part
                    && target.area == region.area
                    && target.layer == region.layer
                    && target.generation == region.generation
                    && target.kind == RegionKind::Scroll
                    && self.routed_scrolls[target_index]
                        .is_some_and(|route| route.policy == policy && !route.invalid)
                    && self.route_region_is_eligible(target_index, axis, pos, layer)
            })
    }

    fn scroll_region_matches_policy(&self, index: usize, policy: ScrollRoutePolicy) -> bool {
        self.routed_scrolls
            .get(index)
            .and_then(Option::as_ref)
            .is_some_and(|route| route.policy == policy && !route.invalid)
    }

    pub(crate) fn route_key_is_eligible(
        &self,
        key: ScrollRouteKey,
        axis: Axis,
        pos: Position,
        layer: LayerId,
    ) -> bool {
        let Some(index) = self.valid_route_key(key) else {
            return false;
        };
        let Some(route) = self.routed_scrolls.get(index).and_then(Option::as_ref) else {
            return false;
        };
        self.route_region_is_eligible(index, axis, pos, layer) && !route.invalid
    }

    pub(crate) fn routed_scroll_hit(
        &self,
        key: ScrollRouteKey,
        axis: Axis,
        pos: Position,
        layer: LayerId,
    ) -> Option<Hit> {
        let index = self.valid_route_key(key)?;
        if !self.route_region_is_eligible(index, axis, pos, layer) {
            return None;
        }
        Some(Self::hit_of(self.regions.get(index)?, pos))
    }

    fn route_region_is_eligible(
        &self,
        index: usize,
        axis: Axis,
        pos: Position,
        layer: LayerId,
    ) -> bool {
        self.regions.get(index).is_some_and(|region| {
            region.pointer_enabled
                && region.kind == RegionKind::Scroll
                && region.scroll_routing == ScrollRouting::Capture
                && region.layer == layer
                && region.area.contains(pos)
                && region.scroll.is_some_and(|(axes, _)| axes.handles(axis))
        })
    }

    fn route_pair_is_ambiguous(
        &self,
        child_index: usize,
        parent_index: usize,
        axis: Axis,
        pos: Position,
        layer: LayerId,
    ) -> bool {
        let Some(child) = self.regions.get(child_index) else {
            return true;
        };
        let Some(parent) = self.regions.get(parent_index) else {
            return true;
        };
        self.scroll_candidates(pos, axis, layer).any(|candidate| {
            let index = candidate.registration_index();
            let Some(region) = self.regions.get(index) else {
                return true;
            };
            ((region.owner, region.part) == (child.owner, child.part)
                && !self.same_routed_registration(child_index, index))
                || ((region.owner, region.part) == (parent.owner, parent.part)
                    && !self.same_routed_registration(parent_index, index))
        })
    }

    fn same_routed_registration(&self, left_index: usize, right_index: usize) -> bool {
        let Some((left, right)) = self
            .regions
            .get(left_index)
            .zip(self.regions.get(right_index))
        else {
            return false;
        };
        let Some((Some(left_route), Some(right_route))) = self
            .routed_scrolls
            .get(left_index)
            .zip(self.routed_scrolls.get(right_index))
        else {
            return false;
        };
        left.owner == right.owner
            && left.part == right.part
            && left.area == right.area
            && left.layer == right.layer
            && left.kind == right.kind
            && left.generation == right.generation
            && left.scroll == right.scroll
            && left.scroll_routing == right.scroll_routing
            && left.pointer_enabled == right.pointer_enabled
            && left.pass_through == right.pass_through
            && left_route.source_revision == right_route.source_revision
            && left_route.parent_index == right_route.parent_index
            && left_route.policy == right_route.policy
            && !left_route.invalid
            && !right_route.invalid
    }

    pub(crate) fn register_passive_reveal_scroll(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
        head: Headroom,
    ) {
        if area.is_empty() {
            return;
        }
        self.push(
            owner,
            PartRef::of(Part::CONTAINER),
            area,
            layer,
            RegionKind::Scroll,
            Some((Axes::V, head)),
        );
        if let Some(region) = self.regions.last_mut() {
            region.scroll_routing = ScrollRouting::PassiveReveal;
        }
    }

    pub(crate) fn register_text_drag_autoscroll(&mut self, registration: TextDragRegistration) {
        let TextDragRegistration {
            owner,
            part,
            area,
            layer,
            axes,
            headroom,
            source_revision,
        } = registration;
        if area.is_empty() || !axes.handles(Axis::V) {
            return;
        }
        let is_registered_text_part = self.regions.iter().rev().any(|region| {
            region.owner == owner
                && region.part == part
                && region.area == area
                && region.layer == layer
                && region.kind == RegionKind::Part
                && region.pointer_enabled
        });
        if !is_registered_text_part {
            return;
        }
        self.text_drag_regions.push(TextDragRegion {
            owner,
            part,
            area,
            layer,
            generation: self.generation,
            axes,
            headroom,
            source_revision,
        });
    }

    pub(crate) fn text_drag_region(
        &self,
        owner: Id,
        part: PartRef,
        layer: LayerId,
    ) -> Option<TextDragRegion> {
        self.text_drag_regions
            .iter()
            .rev()
            .find(|region| {
                region.owner == owner
                    && region.part == part
                    && region.layer == layer
                    && region.generation == self.generation
            })
            .copied()
    }

    fn push(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        kind: RegionKind,
        scroll: Option<(Axes, Headroom)>,
    ) {
        if area.is_empty() {
            return;
        }
        self.push_kind(owner, owner, part, area, layer, kind, false);
        // `push_kind` skips an empty area, so a region is present here.
        if let Some(scroll) = scroll
            && let Some(region) = self.regions.last_mut()
        {
            region.scroll = Some(scroll);
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "internal helper for region registration with explicit owner and layer dispatch"
    )]
    fn push_kind(
        &mut self,
        id: Id,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        kind: RegionKind,
        pass_through: bool,
    ) {
        if area.is_empty() {
            return;
        }
        let pointer_enabled = !self.has_pointer_exclusions
            || !self
                .regions
                .iter()
                .any(|region| region.owner == owner && !region.pointer_enabled);
        self.regions.push(Region {
            id,
            owner,
            part,
            area,
            layer,
            kind,
            generation: self.generation,
            scroll: None,
            scroll_routing: ScrollRouting::Capture,
            pointer_enabled,
            pass_through,
        });
        if let Some(pointer_order) = self.pointer_order.as_mut() {
            pointer_order.push(self.regions.len() - 1);
        }
        self.routed_scrolls.push(None);
    }

    const fn hit_of(r: &Region, pos: Position) -> Hit {
        Hit {
            owner: r.owner,
            part: r.part,
            layer: r.layer,
            kind: r.kind,
            local: Position {
                x: pos.x.saturating_sub(r.area.x),
                y: pos.y.saturating_sub(r.area.y),
            },
        }
    }

    pub fn hit(&self, pos: Position) -> Option<Hit> {
        self.pointer_regions()
            .filter(|(_, r)| {
                r.pointer_enabled
                    && !matches!(r.kind, RegionKind::Scroll | RegionKind::HoverOnly)
                    && r.area.contains(pos)
            })
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    /// Resolve an activation target on the shared pointer input boundary.
    /// Public `hit` keeps its established all-layer query semantics.
    pub(crate) fn hit_on_layer(&self, pos: Position, layer: LayerId) -> Option<Hit> {
        self.pointer_regions()
            .filter(|(_, region)| {
                region.pointer_enabled
                    && region.layer == layer
                    && !matches!(region.kind, RegionKind::Scroll | RegionKind::HoverOnly)
                    && region.area.contains(pos)
            })
            .max_by_key(|(index, _)| *index)
            .map(|(_, region)| Self::hit_of(region, pos))
    }

    /// Observe only passive hover registrations from an explicitly supplied
    /// transparent suffix. Activation regions in those layers stay ignored.
    pub(crate) fn hit_passive_hover(
        &self,
        pos: Position,
        mut layers_high_to_low: impl Iterator<Item = LayerId>,
    ) -> Option<Hit> {
        layers_high_to_low.find_map(|layer| {
            self.pointer_regions()
                .filter(|(_, region)| {
                    region.pointer_enabled
                        && region.layer == layer
                        && region.kind == RegionKind::HoverOnly
                        && region.area.contains(pos)
                })
                .max_by_key(|(index, _)| *index)
                .map(|(_, region)| Self::hit_of(region, pos))
        })
    }

    pub(crate) fn hit_live(&self, pos: Position, layer: LayerId) -> Option<Hit> {
        self.pointer_regions()
            .filter(|(_, r)| {
                r.pointer_enabled
                    && r.layer == layer
                    && matches!(
                        r.kind,
                        RegionKind::Control | RegionKind::Part | RegionKind::HoverOnly
                    )
                    && r.area.contains(pos)
            })
            .max_by_key(|(i, _)| *i)
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    pub fn hit_scroll(&self, pos: Position, axis: Axis) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.pointer_enabled
                    && r.kind == RegionKind::Scroll
                    && r.area.contains(pos)
                    && r.scroll.is_some_and(|(axes, _)| axes.handles(axis))
            })
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    pub(crate) fn scroll_candidates(
        &self,
        pos: Position,
        axis: Axis,
        layer: LayerId,
    ) -> impl Iterator<Item = ScrollCandidate> + '_ {
        self.regions
            .iter()
            .enumerate()
            .rev()
            .filter_map(move |(registration_index, region)| {
                if !region.pointer_enabled
                    || region.kind != RegionKind::Scroll
                    || region.layer != layer
                    || !region.area.contains(pos)
                {
                    return None;
                }
                let (axes, headroom) = region.scroll?;
                axes.handles(axis).then_some(ScrollCandidate {
                    hit: Self::hit_of(region, pos),
                    headroom,
                    passive_reveal: region.scroll_routing == ScrollRouting::PassiveReveal,
                    registration_index,
                })
            })
    }

    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id && r.kind == RegionKind::Control)
            .or_else(|| {
                self.regions
                    .iter()
                    .rev()
                    .find(|r| r.owner == id && r.kind == RegionKind::Decorative)
            })
            .map(|r| r.area)
    }

    pub fn area_of_part(&self, id: Id, part: PartRef) -> Option<Rect> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id && r.part == part)
            .map(|r| r.area)
    }

    pub fn layer_of(&self, id: Id) -> Option<LayerId> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id)
            .map(|r| r.layer)
    }

    pub fn delivers_to(&self, id: Id) -> bool {
        self.regions
            .iter()
            .any(|r| r.owner == id && matches!(r.kind, RegionKind::Control | RegionKind::Part))
    }

    /// Whether `id` owns a region or is a region's control id (a backdrop
    /// hit-tested under a derived id, as in PR #26).
    pub fn has_owner(&self, id: Id) -> bool {
        self.regions.iter().any(|r| r.owner == id || r.id == id)
    }

    /// The topmost region at `pos` of any kind, if any.
    pub fn region_at(&self, pos: Position) -> Option<&Region> {
        self.pointer_regions()
            .filter(|(_, r)| r.area.contains(pos))
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| r)
    }

    /// Register a scrollbar bar: the track becomes a part region of the
    /// owning widget (ported from PR #26 `src/core/hit.rs::register_bar`,
    /// commit f4867e9b5).
    pub fn register_bar(&mut self, bar: Bar, layer: LayerId) {
        if bar.track.is_empty() {
            return;
        }
        self.register_part(bar.owner, PartRef::of(Part::TRACK), bar.track, layer);
        self.bars.retain(|b| b.id != bar.id);
        self.bars.push(bar);
    }

    /// Register the draggable thumb of an already-registered bar (ported
    /// from PR #26 `src/core/hit.rs::register_thumb`, commit f4867e9b5).
    /// Hits on the thumb resolve to `owner`, the widget the bar scrolls,
    /// while the region keeps `bar_id` as its control id — PR #26's
    /// id/owner split. A thumb without its bar is ignored.
    pub fn register_thumb(&mut self, bar_id: Id, owner: Id, thumb: Rect, layer: LayerId) {
        if self.bar(bar_id).is_none() {
            return;
        }
        self.push_kind(
            bar_id,
            owner,
            PartRef::of(Part::THUMB),
            thumb,
            layer,
            RegionKind::Part,
            false,
        );
    }

    /// The registered bars, oldest first.
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    /// The bar with the given id, if registered this frame.
    pub fn bar(&self, id: Id) -> Option<Bar> {
        self.bars.iter().rev().find(|b| b.id == id).copied()
    }

    /// Accept a release for `id` from `from` this frame (ported from PR #26
    /// `src/core/hit.rs::accept_release`, commit f4867e9b5).
    pub fn accept_release(&mut self, id: Id, from: ReleaseFrom) {
        self.releases.push((id, from));
    }

    /// Whether `id` accepted a release from `from` this frame. Parity keys
    /// acceptance by the hit's owner (the id a widget registers), where
    /// PR #26 keyed it by the hit's control id (commit f4867e9b5) — a
    /// documented adaptation, not a coarsening.
    pub fn accepts_release(&self, id: Id, from: ReleaseFrom) -> bool {
        self.releases.contains(&(id, from))
    }

    /// Register a light-dismiss backdrop owned by `id` (ported from PR #26
    /// `src/core/hit.rs::register_backdrop`, commit f4867e9b5). The backdrop
    /// catches clicks outside the popup so they reach the owner; with
    /// `pass_through` it lets the click also reach the layer beneath.
    pub fn register_backdrop(&mut self, id: Id, area: Rect, layer: LayerId, pass_through: bool) {
        self.register_backdrop_of(id, id, area, layer, pass_through);
    }

    /// Register a light-dismiss backdrop owned by `owner`, hit-tested as `id`.
    /// The registry answers hits with `owner`; `id` names the backdrop
    /// control for `has_owner` style queries (stored as the region's id, as
    /// PR #26's `HitRegion::plain(id, area)` does).
    pub fn register_backdrop_of(
        &mut self,
        id: Id,
        owner: Id,
        area: Rect,
        layer: LayerId,
        pass_through: bool,
    ) {
        self.push_kind(
            id,
            owner,
            PartRef::of(Part::BACKDROP),
            area,
            layer,
            RegionKind::Backdrop,
            pass_through,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Id = Id::root("a");
    const B: Id = Id::root("b");
    const C: Id = Id::root("c");

    #[test]
    fn scroll_registry_identity_allocation_stops_permanently_at_max() {
        let counter = AtomicU64::new(1);
        assert_eq!(next_scroll_registry_id(&counter), Some(1));
        assert_eq!(next_scroll_registry_id(&counter), Some(2));

        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(next_scroll_registry_id(&counter), Some(u64::MAX - 1));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
        assert_eq!(next_scroll_registry_id(&counter), None);
        assert_eq!(next_scroll_registry_id(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }

    #[test]
    fn concurrent_scroll_registry_identity_exhaustion_keeps_unique_successes() {
        let counter = AtomicU64::new(u64::MAX - 2);
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| next_scroll_registry_id(&counter)))
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("identity allocation worker completes"))
                .collect::<Vec<_>>()
        });
        assert_eq!(results.len(), 8);
        assert_eq!(results.iter().filter(|result| result.is_none()).count(), 6);
        let mut issued: Vec<_> = results.into_iter().flatten().collect();
        issued.sort_unstable();

        assert_eq!(issued, [u64::MAX - 2, u64::MAX - 1]);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
        assert_eq!(next_scroll_registry_id(&counter), None);
    }

    #[test]
    fn exhausted_scroll_registry_identity_disables_only_routed_wheel_capability() {
        let counter = AtomicU64::new(u64::MAX - 2);
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let pos = Position::new(4, 4);
        let mut previous = Registry::new_with_registry_id(7, &counter);
        let previous_key = previous
            .register_hit_stack_wheel(
                A,
                PartRef::of(Part::LABEL),
                outer,
                LayerId::PAGE,
                Axes::V,
                3,
            )
            .expect("the last two allocated registries remain usable");
        let mut source = Registry::new_with_registry_id(7, &counter);
        let parent = source
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the final allocated registry remains usable");
        let child = source
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the final allocated registry remains usable");
        assert!(source.link_scroll_parent(child, parent));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
        assert!(previous.route_key_is_eligible(previous_key, Axis::V, pos, LayerId::PAGE));
        assert!(source.route_key_is_eligible(child, Axis::V, pos, LayerId::PAGE));
        assert_eq!(source.routed_scroll_source_revision(child), Some(11));
        assert_eq!(
            source.next_scroll_route_parent(child, Axis::V, pos, LayerId::PAGE),
            Some((parent, 7))
        );

        let mut clone = source.clone_with_registry_id(&counter);
        assert_eq!(clone.registry_id.0, None);
        assert_eq!(clone.regions, source.regions);
        assert_eq!(clone.routed_scrolls, source.routed_scrolls);
        assert_eq!(clone.generation, source.generation);
        assert_eq!(clone.route_epoch, source.route_epoch);
        assert_eq!(clone.route_epoch_exhausted, source.route_epoch_exhausted);
        assert_eq!(clone.routed_scroll_info(0), None);
        assert_eq!(clone.routed_scroll_source_revision(child), None);
        assert_eq!(clone.routed_scroll_policy(child), None);
        assert!(!clone.route_key_is_eligible(child, Axis::V, pos, LayerId::PAGE));
        assert!(!clone.link_scroll_parent(child, parent));
        assert_eq!(
            clone.next_scroll_route_target(child, Axis::V, pos, LayerId::PAGE, &[]),
            None
        );
        for key in [previous_key, parent, child] {
            assert_eq!(clone.valid_route_key(key), None);
        }

        let before_routed_rejection = (clone.regions.len(), clone.routed_scrolls.len());
        assert_eq!(
            clone
                .register_routed_scroll(C, inner, LayerId::PAGE, Axes::V, Headroom::default(), 19,),
            None
        );
        assert_eq!(
            (clone.regions.len(), clone.routed_scrolls.len()),
            before_routed_rejection,
            "routed rejection does not add an ordinary scroll registration"
        );

        let control_area = Rect::new(30, 1, 4, 4);
        let part_area = Rect::new(40, 1, 4, 4);
        let scroll_area = Rect::new(50, 1, 4, 4);
        let ordinary_control = Id::root("exhausted.ordinary-control");
        let ordinary_part = Id::root("exhausted.ordinary-part");
        let ordinary_scroll = Id::root("exhausted.ordinary-scroll");
        let part = PartRef::of(Part::SEAM);
        clone.register_control(ordinary_control, control_area, LayerId::PAGE);
        clone.register_part(ordinary_part, part, part_area, LayerId::PAGE);
        clone.register_scroll(
            ordinary_scroll,
            scroll_area,
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert_eq!(
            clone.hit(Position::new(31, 2)).map(|hit| hit.owner),
            Some(ordinary_control)
        );
        assert_eq!(
            clone
                .hit(Position::new(41, 2))
                .map(|hit| (hit.owner, hit.kind)),
            Some((ordinary_part, RegionKind::Part))
        );
        assert_eq!(
            clone
                .hit_scroll(Position::new(51, 2), Axis::V)
                .map(|hit| hit.owner),
            Some(ordinary_scroll)
        );

        let mut other = Registry::new_with_registry_id(7, &counter);
        assert_eq!(other.registry_id.0, None);
        assert_eq!(
            other
                .register_routed_scroll(C, inner, LayerId::PAGE, Axes::V, Headroom::default(), 19,),
            None
        );
        assert!(other.regions.is_empty());
        for key in [previous_key, parent, child] {
            assert_eq!(other.valid_route_key(key), None);
        }

        clone.reset(8);
        assert_eq!(clone.registry_id.0, None);
        assert_eq!(
            clone
                .register_routed_scroll(C, inner, LayerId::PAGE, Axes::V, Headroom::default(), 19,),
            None
        );
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);

        source.reset(8);
        assert_eq!(source.registry_id.0, Some(u64::MAX - 1));
        assert_eq!(source.valid_route_key(parent), None);
        let fresh = source
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 23)
            .expect("reset preserves an already allocated identity");
        assert_eq!(fresh.registry_id, u64::MAX - 1);
        assert!(source.route_key_is_eligible(fresh, Axis::V, pos, LayerId::PAGE));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }

    #[test]
    fn scroll_registry_identity_exhaustion_does_not_change_epoch_exhaustion() {
        let counter = AtomicU64::new(100);
        let mut registry = Registry::new_with_registry_id(3, &counter);
        registry.route_epoch = u64::MAX;
        registry.reset(4);
        assert!(registry.route_epoch_exhausted);
        assert_eq!(registry.route_epoch, u64::MAX);
        assert!(
            registry
                .register_hit_stack_wheel(
                    A,
                    PartRef::of(Part::LABEL),
                    Rect::new(0, 0, 2, 2),
                    LayerId::PAGE,
                    Axes::V,
                    7,
                )
                .is_none()
        );

        let mut clone = registry.clone_with_registry_id(&counter);
        assert_eq!(clone.registry_id.0, Some(101));
        assert!(clone.route_epoch_exhausted);
        assert_eq!(clone.route_epoch, u64::MAX);
        assert!(
            clone
                .register_hit_stack_wheel(
                    A,
                    PartRef::of(Part::LABEL),
                    Rect::new(0, 0, 2, 2),
                    LayerId::PAGE,
                    Axes::V,
                    7,
                )
                .is_none()
        );
    }

    #[test]
    fn allocated_scroll_route_keys_keep_numeric_debug_and_hash_identity() {
        let counter = AtomicU64::new(1);
        let area = Rect::new(0, 0, 8, 6);
        let mut source = Registry::new_with_registry_id(3, &counter);
        let key = source
            .register_hit_stack_wheel(A, PartRef::of(Part::LABEL), area, LayerId::PAGE, Axes::V, 7)
            .expect("the allocated registry creates a key");
        let same_key = source
            .routed_scroll_info(0)
            .expect("the route remains live")
            .0;
        assert_eq!(key, same_key);
        assert!(format!("{source:?}").contains("registry_id: 1,"));
        assert_eq!(format!("{key:?}"), "ScrollRouteKey { .. }");

        let clone = source.clone_with_registry_id(&counter);
        let clone_key = clone
            .routed_scroll_info(0)
            .expect("the clone has a fresh identity")
            .0;
        assert_ne!(key, clone_key);
        assert!(format!("{clone:?}").contains("registry_id: 2,"));
        let keys = std::collections::HashSet::from([key, same_key, clone_key]);
        assert_eq!(keys.len(), 2);

        let exhausted_counter = AtomicU64::new(u64::MAX);
        let exhausted = Registry::new_with_registry_id(3, &exhausted_counter);
        assert!(format!("{exhausted:?}").contains("registry_id: exhausted,"));
    }

    #[test]
    fn last_registration_wins() {
        let mut h = Registry::new(1);
        assert!(
            h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE)
                .is_none()
        );
        assert!(
            h.register_control(B, Rect::new(2, 2, 3, 3), LayerId::PAGE)
                .is_none()
        );
        assert_eq!(h.hit(Position::new(1, 1)).map(|h| h.owner), Some(A));
        assert_eq!(h.hit(Position::new(3, 3)).map(|h| h.owner), Some(B));
        assert_eq!(h.hit(Position::new(20, 20)), None);
    }

    #[test]
    fn higher_layer_shadows_lower() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));
        let hit = h.hit(Position::new(3, 3));
        assert_eq!(hit.map(|h| (h.owner, h.layer)), Some((B, LayerId(1))));

        h.register_control(C, Rect::new(2, 2, 3, 3), LayerId(1));
        assert_eq!(h.hit(Position::new(3, 3)).map(|h| h.owner), Some(C));
    }

    #[test]
    fn scoped_part_precedence_is_exact_local_and_shared_by_pointer_queries() {
        let area = Rect::new(0, 0, 8, 6);
        let pos = Position::new(3, 2);
        let requested = PartRef::item(Part::SEAM, crate::id::ItemKey::num(1));
        let other_item = PartRef::item(Part::SEAM, crate::id::ItemKey::num(2));
        let mut registry = Registry::new(1);
        registry.register_control(A, area, LayerId::PAGE);

        let scope_start = registry.len();
        registry.register_part(B, requested, area, LayerId::PAGE);
        registry.register_control(C, area, LayerId::PAGE);
        registry.register_part(C, other_item, area, LayerId::PAGE);
        registry.register_decor(A, requested, area, LayerId::PAGE);
        registry.register_hover_part(C, PartRef::of(Part::LABEL), area, LayerId::PAGE);
        let physical_regions = registry.regions().to_vec();

        registry.promote_part_hit_suffix(scope_start, requested);

        assert_eq!(
            registry.pointer_order.as_deref(),
            Some(&[0, 2, 3, 4, 5, 1][..])
        );
        assert_eq!(registry.regions(), physical_regions);
        let hit = registry.hit(pos).expect("the exact requested part wins");
        assert_eq!(
            (hit.owner, hit.part, hit.kind),
            (B, requested, RegionKind::Part)
        );
        assert_eq!(registry.hit_on_layer(pos, LayerId::PAGE), Some(hit));
        assert_eq!(registry.hit_live(pos, LayerId::PAGE), Some(hit));
        assert_eq!(
            registry
                .region_at(pos)
                .map(|region| (region.owner, region.part, region.kind)),
            Some((B, requested, RegionKind::Part))
        );
        let hover = registry
            .hit_passive_hover(pos, [LayerId::PAGE].into_iter())
            .expect("passive hover still filters to hover-only entries");
        assert_eq!((hover.owner, hover.kind), (C, RegionKind::HoverOnly));

        registry.register_control(A, area, LayerId::PAGE);
        assert_eq!(
            registry.hit(pos).map(|hit| hit.owner),
            Some(A),
            "registrations after the scope retain ordinary last-wins behavior"
        );
        registry.register_control(C, area, LayerId(1));
        assert_eq!(
            registry.hit(pos).map(|hit| (hit.owner, hit.layer)),
            Some((C, LayerId(1))),
            "layer priority remains above scoped same-layer order"
        );
        assert_eq!(
            registry
                .hit_on_layer(pos, LayerId::PAGE)
                .map(|hit| hit.owner),
            Some(A)
        );
    }

    #[test]
    fn nested_part_scopes_keep_matching_registration_order_and_later_suffixes() {
        let area = Rect::new(0, 0, 8, 6);
        let pos = Position::new(3, 2);
        let seam = PartRef::of(Part::SEAM);
        let mut registry = Registry::new(1);

        let outer_start = registry.len();
        registry.register_part(A, seam, area, LayerId::PAGE);
        registry.register_control(B, area, LayerId::PAGE);
        let nested_start = registry.len();
        registry.register_part(C, seam, area, LayerId::PAGE);
        registry.register_control(Id::root("after-child"), area, LayerId::PAGE);
        registry.promote_part_hit_suffix(nested_start, seam);
        registry.register_control(Id::root("after-parent"), area, LayerId::PAGE);
        registry.promote_part_hit_suffix(outer_start, seam);

        assert_eq!(
            registry.pointer_order.as_deref(),
            Some(&[1, 3, 4, 0, 2][..])
        );
        assert_eq!(registry.hit(pos).map(|hit| hit.owner), Some(C));
        registry.register_control(Id::root("outside-scope"), area, LayerId::PAGE);
        assert_eq!(
            registry.hit(pos).map(|hit| hit.owner),
            Some(Id::root("outside-scope")),
            "later registrations remain after the completed parent scope"
        );
    }

    #[test]
    fn scoped_order_clones_resets_and_keeps_wheel_keys_physical() {
        let area = Rect::new(1, 1, 8, 6);
        let pos = Position::new(3, 2);
        let requested = PartRef::of(Part::SEAM);
        let mut registry = Registry::new(3);
        let start = registry.len();
        registry.register_part(A, requested, area, LayerId::PAGE);
        let wheel = registry
            .register_hit_stack_wheel(B, PartRef::of(Part::TRACK), area, LayerId::PAGE, Axes::V, 7)
            .expect("the physical wheel registration");
        registry.promote_part_hit_suffix(start, requested);

        assert_eq!(wheel.registration_index, 1);
        assert_eq!(registry.pointer_order.as_deref(), Some(&[1, 0][..]));
        assert!(registry.route_key_is_eligible(wheel, Axis::V, pos, LayerId::PAGE));
        assert_eq!(registry.hit(pos).map(|hit| hit.owner), Some(A));

        let mut cloned = registry.clone();
        assert_eq!(cloned.pointer_order, registry.pointer_order);
        assert_eq!(cloned.hit(pos).map(|hit| hit.owner), Some(A));
        assert!(
            !cloned.route_key_is_eligible(wheel, Axis::V, pos, LayerId::PAGE),
            "clone registry identities keep physical wheel keys instance-local"
        );

        cloned.reset(4);
        assert!(cloned.pointer_order.is_none());
        cloned.register_control(A, area, LayerId::PAGE);
        cloned.register_control(B, area, LayerId::PAGE);
        assert_eq!(cloned.hit(pos).map(|hit| hit.owner), Some(B));
    }

    #[test]
    fn an_already_partitioned_scope_keeps_the_lazy_default_order() {
        let area = Rect::new(0, 0, 8, 6);
        let requested = PartRef::of(Part::SEAM);
        let mut registry = Registry::new(1);
        registry.register_control(A, area, LayerId::PAGE);
        let start = registry.len();
        registry.register_control(B, area, LayerId::PAGE);
        registry.register_part(C, requested, area, LayerId::PAGE);

        registry.promote_part_hit_suffix(start, requested);

        assert!(registry.pointer_order.is_none());
        assert_eq!(
            registry.hit(Position::new(1, 1)).map(|hit| hit.owner),
            Some(C)
        );
    }

    #[test]
    fn a_lower_layer_region_registered_later_does_not_shadow_a_higher_one() {
        let mut h = Registry::new(1);

        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        let hit = h.hit(Position::new(3, 3)).expect("a covering region");
        assert_eq!((hit.owner, hit.layer), (B, LayerId(1)));

        h.register_scroll(
            C,
            Rect::new(2, 2, 3, 3),
            LayerId(1),
            Axes::V,
            Headroom::default(),
        );
        h.register_scroll(
            A,
            Rect::new(0, 0, 10, 10),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert_eq!(
            h.hit_scroll(Position::new(3, 3), Axis::V).map(|x| x.owner),
            Some(C)
        );
    }

    #[test]
    fn hit_returns_a_lower_layer_region_for_the_outside_click_test() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));

        let hit = h.hit(Position::new(8, 8));
        assert_eq!(hit.map(|h| (h.owner, h.layer)), Some((A, LayerId::PAGE)));
    }

    #[test]
    fn hit_returns_part_ref_not_a_derived_id() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        let part = PartRef::item(Part::ROW, crate::id::ItemKey::num(7));
        h.register_part(A, part, Rect::new(0, 3, 10, 1), LayerId::PAGE);
        let hit = h
            .hit(Position::new(4, 3))
            .map(|h| (h.owner, h.part, h.kind, h.local));
        assert_eq!(hit, Some((A, part, RegionKind::Part, Position::new(4, 0))));
        assert_eq!(h.area_of_part(A, part), Some(Rect::new(0, 3, 10, 1)));
        assert_eq!(h.area_of(A), Some(Rect::new(0, 0, 10, 10)));
    }

    #[test]
    fn hover_only_is_observable_for_top_layer_hover_but_not_click_delivery() {
        let mut registry = Registry::new(1);
        let area = Rect::new(1, 2, 8, 3);
        let hover_part = PartRef::of(Part::LABEL);
        registry.register_control(A, area, LayerId::PAGE);
        registry.register_hover_part(B, hover_part, area, LayerId::PAGE);

        let hover = registry
            .hit_live(Position::new(3, 3), LayerId::PAGE)
            .expect("the exact top-layer hover-only part");
        assert_eq!(
            (hover.owner, hover.part, hover.kind),
            (B, hover_part, RegionKind::HoverOnly)
        );
        assert_eq!(
            registry
                .hit(Position::new(3, 3))
                .map(|hit| (hit.owner, hit.kind)),
            Some((A, RegionKind::Control)),
            "ordinary click hit testing skips hover-only registrations"
        );
        assert_eq!(registry.hit_live(Position::new(3, 3), LayerId(1)), None);
        assert_eq!(registry.hit_scroll(Position::new(3, 3), Axis::V), None);
        assert!(!registry.delivers_to(B));
        assert!(
            registry
                .regions()
                .iter()
                .filter(|region| region.owner == B)
                .all(|region| region.kind == RegionKind::HoverOnly && region.scroll().is_none())
        );
    }

    #[test]
    fn passive_scroll_policy_is_attached_to_each_registration_and_top_layer() {
        let mut registry = Registry::new(1);
        let area = Rect::new(1, 1, 12, 6);
        let normal_headroom = Headroom {
            down: 8,
            ..Headroom::default()
        };
        let passive_headroom = Headroom {
            up: 2,
            down: 3,
            ..Headroom::default()
        };
        registry.register_scroll(A, area, LayerId::PAGE, Axes::V, normal_headroom);
        registry.register_passive_reveal_scroll(A, area, LayerId::PAGE, passive_headroom);
        registry.register_scroll(B, area, LayerId(1), Axes::V, Headroom::default());

        let page_candidates = registry
            .scroll_candidates(Position::new(4, 3), Axis::V, LayerId::PAGE)
            .map(|candidate| (candidate.passive_reveal(), candidate.headroom()))
            .collect::<Vec<_>>();
        assert_eq!(
            page_candidates,
            [(true, passive_headroom), (false, normal_headroom),],
            "reverse registration order keeps policy on the exact duplicate owner registration"
        );
        let modal_candidates = registry
            .scroll_candidates(Position::new(4, 3), Axis::V, LayerId(1))
            .map(|candidate| (candidate.hit().owner, candidate.passive_reveal()))
            .collect::<Vec<_>>();
        assert_eq!(modal_candidates, [(B, false)]);

        assert_eq!(
            registry
                .hit_scroll(Position::new(4, 3), Axis::V)
                .map(|hit| hit.owner),
            Some(B),
            "legacy generic hit_scroll ordering and endpoint behavior remain unchanged"
        );
    }

    #[test]
    fn explicit_scroll_parent_skips_only_exact_duplicate_child_registrations() {
        let mut registry = Registry::new(3);
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the parent registration");
        let child = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the child registration");
        let duplicate = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the duplicate child registration");
        assert!(registry.link_scroll_parent(child, parent));
        assert!(registry.link_scroll_parent(duplicate, parent));

        assert_eq!(
            registry.next_scroll_route_parent(
                duplicate,
                Axis::V,
                Position::new(4, 4),
                LayerId::PAGE,
            ),
            Some((parent, 7)),
            "identical registration metadata coalesces to one target"
        );
    }

    #[test]
    fn hit_stack_keys_are_registry_scoped_and_epoch_exhaustion_fails_closed() {
        let area = Rect::new(1, 1, 12, 6);
        let mut registry = Registry::new(3);
        let key = registry
            .register_hit_stack_wheel(
                A,
                PartRef::of(Part::LABEL),
                area,
                LayerId::PAGE,
                Axes::Both,
                7,
            )
            .expect("the hit-stack registration");
        assert!(registry.route_key_is_eligible(key, Axis::H, Position::new(4, 3), LayerId::PAGE,));

        let mut clone = registry.clone();
        assert!(!clone.route_key_is_eligible(key, Axis::H, Position::new(4, 3), LayerId::PAGE,));
        let clone_key = clone
            .register_hit_stack_wheel(
                A,
                PartRef::of(Part::LABEL),
                area,
                LayerId::PAGE,
                Axes::Both,
                7,
            )
            .expect("the clone creates a registry-local key");
        assert_ne!(key, clone_key);
        assert!(clone.route_key_is_eligible(
            clone_key,
            Axis::H,
            Position::new(4, 3),
            LayerId::PAGE,
        ));

        registry.route_epoch = u64::MAX;
        registry.reset(4);
        assert!(registry.route_epoch_exhausted);
        assert!(
            registry
                .register_hit_stack_wheel(
                    A,
                    PartRef::of(Part::LABEL),
                    area,
                    LayerId::PAGE,
                    Axes::Both,
                    7,
                )
                .is_none()
        );
    }

    #[test]
    fn event_local_ambiguity_ignores_pointer_disabled_sibling_targets() {
        let mut registry = Registry::new(3);
        let left = Rect::new(0, 0, 8, 8);
        let overlapping = Rect::new(0, 0, 8, 8);
        let under = registry
            .register_event_local_hit_stack_wheel(
                C,
                PartRef::of(Part::CONTAINER),
                Rect::new(0, 0, 20, 8),
                LayerId::PAGE,
                Axes::Both,
                31,
            )
            .expect("the underlying route target");
        let selected = registry
            .register_event_local_hit_stack_wheel(
                A,
                PartRef::of(Part::BODY),
                left,
                LayerId::PAGE,
                Axes::Both,
                11,
            )
            .expect("the point-eligible source");
        let disabled = registry
            .register_event_local_hit_stack_wheel(
                A,
                PartRef::of(Part::BODY),
                overlapping,
                LayerId::PAGE,
                Axes::Both,
                22,
            )
            .expect("the pointer-disabled sibling");
        assert_eq!(
            registry.next_scroll_route_target(
                selected,
                Axis::V,
                Position::new(2, 2),
                LayerId::PAGE,
                &[A],
            ),
            None,
            "an enabled overlapping sibling with another source is ambiguous"
        );

        registry.regions[disabled.registration_index].pointer_enabled = false;

        assert_eq!(
            registry.next_scroll_route_target(
                selected,
                Axis::V,
                Position::new(2, 2),
                LayerId::PAGE,
                &[A],
            ),
            Some((under, 31))
        );
    }

    #[test]
    fn hit_stack_keys_cannot_be_mixed_into_parent_links() {
        let mut registry = Registry::new(3);
        let outer = Rect::new(0, 0, 20, 12);
        let inner = Rect::new(2, 2, 12, 6);
        let parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::Both, Headroom::default(), 7)
            .expect("the linked parent");
        let child = registry
            .register_hit_stack_wheel(
                B,
                PartRef::of(Part::LABEL),
                inner,
                LayerId::PAGE,
                Axes::Both,
                11,
            )
            .expect("the hit-stack child");

        assert!(!registry.link_scroll_parent(child, parent));
        assert_eq!(registry.routed_scroll_source_revision(child), None);
    }

    #[test]
    fn conflicting_duplicate_or_unrelated_overlap_blocks_scroll_fallback() {
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let mut conflict = Registry::new(3);
        let parent = conflict
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the parent registration");
        let first = conflict
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the first child registration");
        let second = conflict
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 12)
            .expect("the conflicting child registration");
        assert!(conflict.link_scroll_parent(first, parent));
        assert!(conflict.link_scroll_parent(second, parent));
        assert_eq!(
            conflict.next_scroll_route_parent(second, Axis::V, Position::new(4, 4), LayerId::PAGE,),
            None,
            "same-owner geometry with a changed source revision is ambiguous"
        );

        let mut overlap = Registry::new(3);
        let parent = overlap
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the parent registration");
        overlap.register_scroll(
            C,
            Rect::new(1, 1, 15, 15),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        let child = overlap
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the child registration");
        assert!(overlap.link_scroll_parent(child, parent));
        assert_eq!(
            overlap.next_scroll_route_parent(child, Axis::V, Position::new(4, 4), LayerId::PAGE),
            None,
            "an unrelated eligible registration is not skipped"
        );
    }

    #[test]
    fn older_same_owner_part_scroll_conflicts_block_parent_offer() {
        let outer = Rect::new(0, 0, 30, 30);
        let parent_area = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let point = Position::new(4, 4);

        let mut older_child_conflict = Registry::new(3);
        older_child_conflict
            .register_routed_scroll(B, outer, LayerId::PAGE, Axes::V, Headroom::default(), 99)
            .expect("the older conflicting child registration");
        let parent = older_child_conflict
            .register_routed_scroll(
                A,
                parent_area,
                LayerId::PAGE,
                Axes::V,
                Headroom::default(),
                7,
            )
            .expect("the selected parent");
        let child = older_child_conflict
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the selected child with conflicting owner metadata");
        assert!(older_child_conflict.link_scroll_parent(child, parent));
        assert_eq!(
            older_child_conflict.next_scroll_route_parent(child, Axis::V, point, LayerId::PAGE),
            None,
            "an older event-eligible same-owner/part child conflict blocks the parent"
        );

        let mut older_parent_conflict = Registry::new(3);
        older_parent_conflict
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 99)
            .expect("the older conflicting parent registration");
        let parent = older_parent_conflict
            .register_routed_scroll(
                A,
                parent_area,
                LayerId::PAGE,
                Axes::V,
                Headroom::default(),
                7,
            )
            .expect("the selected parent with conflicting owner metadata");
        let child = older_parent_conflict
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the selected child");
        assert!(older_parent_conflict.link_scroll_parent(child, parent));
        assert_eq!(
            older_parent_conflict.next_scroll_route_parent(child, Axis::V, point, LayerId::PAGE),
            None,
            "an older event-eligible same-owner/part parent conflict blocks the offer"
        );
    }

    #[test]
    fn scroll_route_links_are_generation_layer_and_axis_bound() {
        let mut registry = Registry::new(3);
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the parent registration");
        let horizontal = registry
            .register_routed_scroll(C, inner, LayerId::PAGE, Axes::H, Headroom::default(), 8)
            .expect("the horizontal overlap");
        let child = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the child registration");
        assert!(registry.link_scroll_parent(child, parent));
        assert_eq!(
            registry.next_scroll_route_parent(child, Axis::V, Position::new(4, 4), LayerId::PAGE),
            Some((parent, 7)),
            "an axis-ineligible overlapping target is not a route candidate"
        );

        let wrong_layer = registry
            .register_routed_scroll(A, outer, LayerId(1), Axes::V, Headroom::default(), 7)
            .expect("the modal registration");
        let modal_child = registry
            .register_routed_scroll(B, inner, LayerId(1), Axes::V, Headroom::default(), 11)
            .expect("the modal child");
        assert!(registry.link_scroll_parent(modal_child, wrong_layer));
        assert!(!registry.link_scroll_parent(modal_child, child));
        assert_eq!(
            registry.next_scroll_route_parent(
                modal_child,
                Axis::V,
                Position::new(4, 4),
                LayerId(1)
            ),
            None,
            "cross-layer re-linking invalidates the child"
        );
        assert_ne!(horizontal, child);

        let stale = parent;
        registry.reset(4);
        let new_parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the current parent registration");
        let new_child = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the current child registration");
        assert!(!registry.link_scroll_parent(new_child, stale));
        assert_eq!(
            registry.next_scroll_route_parent(
                new_child,
                Axis::V,
                Position::new(4, 4),
                LayerId::PAGE
            ),
            None,
            "a stale key cannot authorize a current route"
        );

        registry.reset(5);
        let fresh_parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the fresh parent registration");
        let fresh_child = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the fresh child registration");
        assert!(registry.link_scroll_parent(fresh_child, fresh_parent));
        assert_eq!(
            registry.next_scroll_route_parent(
                fresh_child,
                Axis::V,
                Position::new(4, 4),
                LayerId::PAGE,
            ),
            Some((fresh_parent, 7))
        );
        assert_ne!(new_parent, fresh_parent);

        let mut foreign_registry = Registry::new(5);
        let foreign_parent = foreign_registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the foreign parent registration");
        assert!(!registry.link_scroll_parent(fresh_child, foreign_parent));
        assert_eq!(
            registry.next_scroll_route_parent(
                fresh_child,
                Axis::V,
                Position::new(4, 4),
                LayerId::PAGE,
            ),
            None,
            "a key from another registry cannot authorize a route"
        );
    }

    #[test]
    fn cloned_registry_preserves_regions_but_gets_a_fresh_route_identity() {
        let mut source = Registry::new(3);
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let parent = source
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the source parent");
        let child = source
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the source child");
        assert!(source.link_scroll_parent(child, parent));

        let mut clone = source.clone();
        assert!(!clone.link_scroll_parent(child, parent));
        assert_eq!(
            clone.next_scroll_route_parent(child, Axis::V, Position::new(4, 4), LayerId::PAGE),
            None,
            "source keys cannot be used in a cloned registry"
        );

        let outer = Rect::new(30, 30, 20, 20);
        let inner = Rect::new(32, 32, 10, 10);
        let clone_parent = clone
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 17)
            .expect("a new parent in the clone");
        let clone_child = clone
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 19)
            .expect("a new child in the clone");
        assert!(clone.link_scroll_parent(clone_child, clone_parent));
        assert_eq!(
            clone.next_scroll_route_parent(
                clone_child,
                Axis::V,
                Position::new(34, 34),
                LayerId::PAGE,
            ),
            Some((clone_parent, 17)),
            "fresh clone registrations remain routable"
        );
    }

    #[test]
    fn disabled_and_empty_routed_regions_do_not_offer_a_target() {
        let mut registry = Registry::new(3);
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the parent registration");
        let child = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the child registration");
        assert!(registry.link_scroll_parent(child, parent));
        assert_eq!(
            registry.register_routed_scroll(
                C,
                Rect::ZERO,
                LayerId::PAGE,
                Axes::V,
                Headroom::default(),
                12,
            ),
            None,
            "empty clipped geometry has no route key"
        );
        registry.register_keyboard_control(B, inner, LayerId::PAGE);
        assert!(!registry.route_key_is_eligible(
            child,
            Axis::V,
            Position::new(4, 4),
            LayerId::PAGE,
        ));
        assert_eq!(
            registry.next_scroll_route_parent(child, Axis::V, Position::new(4, 4), LayerId::PAGE),
            None,
            "a keyboard-only disabled registration cannot route"
        );
    }

    #[test]
    fn same_owner_control_region_does_not_conflict_with_its_routed_scroll() {
        let mut registry = Registry::new(3);
        let outer = Rect::new(0, 0, 20, 20);
        let inner = Rect::new(2, 2, 10, 10);
        let parent = registry
            .register_routed_scroll(A, outer, LayerId::PAGE, Axes::V, Headroom::default(), 7)
            .expect("the parent registration");
        let child = registry
            .register_routed_scroll(B, inner, LayerId::PAGE, Axes::V, Headroom::default(), 11)
            .expect("the child registration");
        assert!(registry.link_scroll_parent(child, parent));
        registry.register_control(B, inner, LayerId::PAGE);
        assert_eq!(
            registry.next_scroll_route_parent(child, Axis::V, Position::new(4, 4), LayerId::PAGE),
            Some((parent, 7)),
            "non-scroll control registration does not create scroll-route ambiguity"
        );
    }

    #[test]
    fn hit_scroll_returns_the_innermost_handler_of_the_axis() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 20, 20),
            LayerId::PAGE,
            Axes::Both,
            Headroom::default(),
        );
        h.register_scroll(
            B,
            Rect::new(5, 5, 5, 5),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert_eq!(
            h.hit_scroll(Position::new(6, 6), Axis::V).map(|h| h.owner),
            Some(B)
        );
        assert_eq!(
            h.hit_scroll(Position::new(6, 6), Axis::H).map(|h| h.owner),
            Some(A)
        );
        assert_eq!(
            h.hit_scroll(Position::new(1, 1), Axis::V).map(|h| h.owner),
            Some(A)
        );

        assert_eq!(h.hit(Position::new(6, 6)), None);
    }

    #[test]
    fn hit_scroll_returns_a_region_at_zero_headroom() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 20, 20),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        let hit = h.hit_scroll(Position::new(1, 1), Axis::V);
        assert_eq!(hit.map(|h| h.owner), Some(A));
    }

    #[test]
    fn hit_scroll_skips_regions_that_do_not_handle_the_axis() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 20, 20),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        h.register_control(B, Rect::new(0, 0, 20, 20), LayerId::PAGE);
        assert_eq!(h.hit_scroll(Position::new(1, 1), Axis::H), None);
    }

    #[test]
    fn duplicate_id_is_reported_as_a_diagnostic_not_a_panic() {
        let mut h = Registry::new(1);
        assert!(
            h.register_control(A, Rect::new(0, 0, 1, 1), LayerId::PAGE)
                .is_none()
        );
        let d = h.register_control(A, Rect::new(5, 5, 1, 1), LayerId::PAGE);
        assert_eq!(
            d,
            Some(Diagnostic::DuplicateId {
                id: A,
                first: Rect::new(0, 0, 1, 1),
                second: Rect::new(5, 5, 1, 1)
            })
        );
    }

    #[test]
    fn empty_rects_are_rejected() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 0, 5), LayerId::PAGE);
        h.register_part(A, PartRef::of(Part::LABEL), Rect::ZERO, LayerId::PAGE);
        h.register_scroll(
            A,
            Rect::new(1, 1, 3, 0),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert!(h.is_empty());
        assert_eq!(h.area_of(A), None);
    }

    // Regression for review finding 2 (2026-10-21): a rejected empty-area
    // registration must never write scroll state onto the previous,
    // unrelated region.
    #[test]
    fn pushing_an_empty_area_never_touches_the_previous_regions_scroll() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 10, 10),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        let scroll = (Axes::V, Headroom::default());
        h.register_scroll(B, Rect::new(0, 0, 10, 0), LayerId::PAGE, scroll.0, scroll.1);
        assert_eq!(h.len(), 1, "the empty scroll area registers nothing");
        let region = h.regions().first().expect("the scrollable region");
        assert_eq!(
            (region.owner, region.scroll()),
            (A, Some(scroll)),
            "the first region keeps its owner and scroll state"
        );
        h.register_part(A, PartRef::of(Part::LABEL), Rect::ZERO, LayerId::PAGE);
        assert_eq!(h.len(), 1);
        assert_eq!(h.regions().first().map(|r| r.scroll()), Some(Some(scroll)));
    }

    #[test]
    fn generation_bump_invalidates_stale_regions() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 1, 1), LayerId::PAGE);
        assert_eq!(h.regions().first().map(|r| r.generation), Some(1));
        h.reset(2);
        assert!(h.is_empty());
        assert_eq!(h.generation(), 2);
        assert_eq!(h.hit(Position::new(0, 0)), None);
    }

    // Ported from PR #26 src/core/hit.rs `parts_resolve_to_their_owner_and_part`
    // (the bar / thumb half), commit f4867e9b5.
    #[test]
    fn a_bar_resolves_to_its_owners_track_and_the_thumb_is_its_own_part() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 20, 5), LayerId::PAGE);
        h.register_bar(
            Bar {
                id: B,
                owner: A,
                axis: Axis::V,
                track: Rect::new(19, 0, 1, 5),
            },
            LayerId::PAGE,
        );
        let on_track = h.hit(Position::new(19, 2)).expect("the track");
        assert_eq!(
            (on_track.owner, on_track.part),
            (A, PartRef::of(Part::TRACK)),
            "a scrollbar resolves to its owner's track"
        );
        assert_eq!(
            h.bars().iter().map(|b| b.owner).collect::<Vec<_>>(),
            vec![A],
            "the frame's scrollbars are listed"
        );
        assert_eq!(h.bar(B).map(|b| b.axis), Some(Axis::V));

        h.register_thumb(B, A, Rect::new(19, 1, 1, 2), LayerId::PAGE);
        let on_thumb = h.hit(Position::new(19, 2)).expect("the thumb");
        assert_eq!(
            (on_thumb.owner, on_thumb.part),
            (A, PartRef::of(Part::THUMB)),
            "the thumb resolves to the bar's owner, its own part"
        );
        let thumb_region = h
            .regions()
            .iter()
            .rev()
            .find(|r| r.part == PartRef::of(Part::THUMB))
            .expect("the thumb region");
        assert_eq!(
            (thumb_region.id, thumb_region.owner),
            (B, A),
            "the thumb keeps the bar's id and the owner's owner"
        );
        assert_eq!(
            h.hit(Position::new(19, 4)).map(|x| x.part),
            Some(PartRef::of(Part::TRACK)),
            "the track below the thumb keeps answering"
        );
        assert_eq!(h.hit(Position::new(25, 2)), None, "outside every region");
    }

    #[test]
    fn a_thumb_without_its_bar_is_ignored_and_an_empty_track_never_registers() {
        let mut h = Registry::new(1);
        h.register_thumb(B, A, Rect::new(0, 0, 1, 1), LayerId::PAGE);
        assert!(h.is_empty(), "a thumb needs its bar first");
        h.register_bar(
            Bar {
                id: B,
                owner: A,
                axis: Axis::V,
                track: Rect::ZERO,
            },
            LayerId::PAGE,
        );
        assert!(
            h.is_empty() && h.bars().is_empty(),
            "an empty track is no bar"
        );
    }

    // Ported from PR #26 `accept_release` / `accepts_release`, commit f4867e9b5.
    #[test]
    fn releases_are_accepted_per_widget_and_source() {
        let mut h = Registry::new(1);
        assert!(!h.accepts_release(A, ReleaseFrom::Press(A)));
        h.accept_release(A, ReleaseFrom::Press(A));
        h.accept_release(B, ReleaseFrom::Secondary);
        assert!(h.accepts_release(A, ReleaseFrom::Press(A)));
        assert!(!h.accepts_release(A, ReleaseFrom::Secondary));
        assert!(h.accepts_release(B, ReleaseFrom::Secondary));
        assert!(!h.accepts_release(B, ReleaseFrom::Press(A)));
        h.reset(2);
        assert!(
            !h.accepts_release(A, ReleaseFrom::Press(A)),
            "a reset drops the frame's releases"
        );
    }

    // Ported from PR #26 `register_backdrop` / `register_backdrop_of`,
    // commit f4867e9b5 (adapted: parity answers hits with the owner).
    #[test]
    fn a_backdrop_catches_clicks_for_its_owner() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        h.register_backdrop_of(B, A, Rect::new(0, 0, 40, 20), LayerId(1), false);
        let on_backdrop = h.hit(Position::new(30, 15)).expect("the backdrop");
        assert_eq!(
            (on_backdrop.owner, on_backdrop.part),
            (A, PartRef::of(Part::BACKDROP))
        );
        assert!(
            h.has_owner(B),
            "the backdrop's own control id stays queryable"
        );
        assert!(h.has_owner(A), "the owner stays queryable");
        let backdrop = h
            .regions()
            .iter()
            .find(|r| r.kind == RegionKind::Backdrop)
            .expect("the backdrop region");
        assert_eq!(
            (backdrop.id, backdrop.owner),
            (B, A),
            "the region keeps the backdrop id alongside its owner"
        );
        assert!(
            !h.region_at(Position::new(30, 15))
                .is_some_and(|r| r.pass_through()),
            "a solid backdrop does not pass through"
        );
        // On the backdrop's layer the backdrop also wins over the page
        // control it covers: the popup is the top layer.
        let over_page = h.hit(Position::new(2, 2)).expect("covered by backdrop");
        assert_eq!(
            (over_page.owner, over_page.part, over_page.layer),
            (A, PartRef::of(Part::BACKDROP), LayerId(1))
        );

        h.register_backdrop(C, Rect::new(0, 0, 5, 5), LayerId(2), true);
        assert!(
            h.region_at(Position::new(2, 2))
                .is_some_and(|r| r.pass_through()),
            "a pass-through backdrop says so"
        );
        h.register_backdrop(C, Rect::ZERO, LayerId(2), false);
    }
}

#[cfg(test)]
mod runtime_tests {
    use ratatui_core::layout::Rect;

    use crate::event::{KeyCode, MouseKind};
    use crate::id::Id;
    use crate::layer::LayerSpec;
    use crate::runtime::stub::{Control, Stub, key, mouse, runtime, step};

    const PAGE_A: Id = Id::root("page.a");
    const DLG: Id = Id::root("dlg");
    const OK: Id = Id::root("dlg.ok");

    #[test]
    fn inert_below_registers_nothing() {
        let s = Stub {
            page: vec![Control::new(PAGE_A, Rect::new(0, 0, 10, 1))],
            layers: vec![(DLG, vec![Control::new(OK, Rect::new(2, 4, 5, 1))])],
            ..Stub::default()
        };
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert!(
            rt.registry().area_of(PAGE_A).is_some(),
            "the page registers"
        );

        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert!(
            rt.registry().area_of(PAGE_A).is_none(),
            "a page control below an inert layer registers nothing"
        );
        assert!(rt.registry().area_of(OK).is_some());

        rt.app_mut().log.clear();
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        assert!(!rt.app().saw(PAGE_A, "Press"), "{:?}", rt.app().log);
    }
}
