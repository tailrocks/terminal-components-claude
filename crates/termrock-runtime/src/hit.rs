pub use termrock_core::geometry::{Axes, Headroom};

use ratatui_core::layout::{Position, Rect};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::diagnostics::Diagnostic;
use crate::event::Axis;
use crate::id::{Id, Part, PartRef};
use crate::layer::LayerId;

static NEXT_SCROLL_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

fn next_scroll_registry_id() -> u64 {
    NEXT_SCROLL_REGISTRY_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .expect("scroll registry identity space exhausted")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum RegionKind {
    Control,
    Part,
    Scroll,

    HoverOnly,
    Decorative,

    Backdrop,
}

/// A scrollbar bar registered by its owning widget (ported from PR #26
/// `src/core/hit.rs::Bar`, commit f4867e9b5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bar {
    pub id: Id,

    pub owner: Id,

    pub axis: Axis,

    pub track: Rect,
}

/// Whose release a widget accepts (ported from PR #26 `src/core/hit.rs::ReleaseFrom`,
/// commit f4867e9b5): a drag that started on a widget keeps receiving
/// releases, and a secondary-button release may be accepted while pressed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReleaseFrom {
    Press(Id),

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
    registry_id: u64,
    generation: u32,
    route_epoch: u64,
    route_epoch_exhausted: bool,
    has_pointer_exclusions: bool,
    bars: Vec<Bar>,
    releases: Vec<(Id, ReleaseFrom)>,
}

impl Clone for Registry {
    fn clone(&self) -> Self {
        Self {
            regions: self.regions.clone(),
            pointer_order: self.pointer_order.clone(),
            routed_scrolls: self.routed_scrolls.clone(),
            text_drag_regions: self.text_drag_regions.clone(),
            registry_id: next_scroll_registry_id(),
            generation: self.generation,
            route_epoch: self.route_epoch,
            route_epoch_exhausted: self.route_epoch_exhausted,
            has_pointer_exclusions: self.has_pointer_exclusions,
            bars: self.bars.clone(),
            releases: self.releases.clone(),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Registry {
    pub fn new(generation: u32) -> Self {
        Registry {
            regions: Vec::new(),
            pointer_order: None,
            routed_scrolls: Vec::new(),
            text_drag_regions: Vec::new(),
            registry_id: next_scroll_registry_id(),
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
            registry_id: self.registry_id,
            registry_epoch: self.route_epoch,
            generation: self.generation,
            registration_index,
        })
    }

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
        (key.registry_epoch == self.route_epoch
            && key.registry_id == self.registry_id
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
                registry_id: self.registry_id,
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

    pub fn has_owner(&self, id: Id) -> bool {
        self.regions.iter().any(|r| r.owner == id || r.id == id)
    }

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

    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    pub fn bar(&self, id: Id) -> Option<Bar> {
        self.bars.iter().rev().find(|b| b.id == id).copied()
    }

    /// Accept a release for `id` from `from` this frame (ported from PR #26
    /// `src/core/hit.rs::accept_release`, commit f4867e9b5).
    pub fn accept_release(&mut self, id: Id, from: ReleaseFrom) {
        self.releases.push((id, from));
    }

    pub fn accepts_release(&self, id: Id, from: ReleaseFrom) -> bool {
        self.releases.contains(&(id, from))
    }

    /// Register a light-dismiss backdrop owned by `id` (ported from PR #26
    /// `src/core/hit.rs::register_backdrop`, commit f4867e9b5).

    pub fn register_backdrop(&mut self, id: Id, area: Rect, layer: LayerId, pass_through: bool) {
        self.register_backdrop_of(id, id, area, layer, pass_through);
    }

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
