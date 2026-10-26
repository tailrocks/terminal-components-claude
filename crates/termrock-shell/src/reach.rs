//! The reachability harness: does every word a screen shows at a generous
//! size stay reachable at every shipping size?
//!
//! Ported from PR #26 `src/shell/reach.rs` (commit `f4867e9b5`). The parity
//! adaptation drives the [`termrock_runtime::Runtime`] directly over a test
//! buffer instead of the source's `Application` trait, and wheels through
//! [`termrock_core::event::MouseKind::Wheel`] (axis plus notch delta) instead
//! of four wheel kinds. The source's `Words::add_overflowed` and the
//! reveal-driven `Reveal` steps are not ported: they read the source's
//! `ui::reveal` overflow log, which the parity feedback wave owns; a case
//! that hides words only behind truncation reveals stays an allow-file row
//! until that wave lands.

use std::collections::{BTreeMap, BTreeSet};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Position;
use ratatui_core::layout::Rect;
use termrock_core::event::{Input, MouseKind};
use termrock_core::platform::ClientPlatform;
use termrock_runtime::event::Axis;
use termrock_runtime::hit::{Bar, RegionKind, Registry};
use termrock_runtime::{App, Id, PendingInput, Runtime};
use termrock_theme::Theme as ThemeType;

/// The sizes a screen must survive.
pub const SIZES: [(u16, u16); 8] = [
    (40, 12),
    (60, 18),
    (72, 20),
    (80, 24),
    (100, 25),
    (106, 29),
    (120, 40),
    (160, 50),
];

const REFERENCE: (u16, u16) = (300, 100);

const MIN_WORD: usize = 3;

const MAX_PAGES: usize = 200;

/// A clipped hit-area search is bounded by the largest reference frame.
const MAX_ANCHOR_CELLS: usize = REFERENCE.0 as usize * REFERENCE.1 as usize;

/// Pages before an axis gives up; stalls before the walk backs off.
const STALL: (usize, usize) = (3, 12);

/// What one scenario at one size reached.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reach {
    /// The size probed.
    pub size: (u16, u16),
    /// The screen showed its too-small panel instead.
    pub too_small: bool,
    /// Reference words no scroll reached.
    pub missing: BTreeSet<String>,
    /// Reference words reached only as a cut prefix.
    pub cut: BTreeSet<String>,
    /// Wheel pages the exploration turned.
    pub bars: usize,
}

impl Reach {
    pub(crate) fn of(
        size: (u16, u16),
        too_small: bool,
        want: &Words,
        reached: &Words,
        bars: usize,
    ) -> Self {
        if too_small {
            return Self {
                size,
                too_small,
                ..Self::default()
            };
        }
        let (missing, cut) = want.against(reached);
        Self {
            size,
            too_small,
            missing,
            cut,
            bars,
        }
    }
}

/// The words one frame shows: whole words, and prefixes cut by truncation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Words {
    /// Words shown in full.
    pub whole: BTreeSet<String>,
    /// Prefixes a `…` cell cut short.
    pub cut: BTreeSet<String>,
}

impl Words {
    /// Read a whole buffer minus its footer row.
    pub fn read(buf: &Buffer) -> Self {
        let mut out = Self::default();
        out.add(buf);
        out
    }

    /// Add a buffer's words, skipping its footer row.
    pub fn add(&mut self, buf: &Buffer) {
        let area = buf.area();
        self.add_area(
            buf,
            Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1)),
        );
    }

    /// Add the words of one region of a buffer.
    pub fn add_area(&mut self, buf: &Buffer, area: Rect) {
        let area = area.intersection(*buf.area());
        let rows = area.height;
        for y in 0..rows {
            let mut word = String::new();
            for x in 0..area.width {
                let sym = buf[(area.x + x, area.y + y)].symbol();
                let mut chars = sym.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if c.is_alphanumeric() => word.push(c),
                    (Some('…'), None) => {
                        if !word.is_empty() {
                            self.cut.insert(std::mem::take(&mut word));
                        }
                    }
                    (None, _) => {}
                    _ => self.push(&mut word),
                }
            }
            self.push(&mut word);
        }
    }

    fn push(&mut self, word: &mut String) {
        if word.chars().count() >= MIN_WORD {
            self.whole.insert(word.clone());
        }
        word.clear();
    }

    /// Merge another frame's words in.
    pub fn extend(&mut self, other: Self) {
        self.whole.extend(other.whole);
        self.cut.extend(other.cut);
    }

    /// Sort the reference words `reached` never showed whole into missing
    /// and cut.
    pub fn against(&self, reached: &Self) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut missing = BTreeSet::new();
        let mut cut = BTreeSet::new();
        for w in &self.whole {
            if reached.whole.contains(w) {
                continue;
            }
            if reached.whole.iter().any(|r| r.contains(w.as_str())) {
                continue;
            }
            if reached.cut.iter().any(|p| w.starts_with(p.as_str())) {
                cut.insert(w.clone());
            } else {
                missing.insert(w.clone());
            }
        }
        (missing, cut)
    }
}

/// One scenario the reach harness drives: an app plus the setup that puts
/// its screen on show.
pub trait Scenario: Sized {
    /// The app under test.
    type App: App;

    /// The theme the app runs in.
    fn theme() -> ThemeType {
        ThemeType::junie()
    }

    /// A fresh app in its initial screen state.
    fn fresh() -> Self::App;

    /// Is the app showing its too-small panel at this size? Too-small
    /// scenarios are recorded, not failed.
    fn too_small(app: &Self::App) -> bool {
        let _ = app;
        false
    }
}

/// One named screen with its setup.
pub struct Case<S: Scenario + 'static> {
    /// The scenario's name in the allow file.
    pub name: &'static str,
    /// The setup that shows the screen, run before the runtime starts.
    pub base: fn(&mut S::App),
}

impl<S: Scenario + 'static> Case<S> {
    /// A page case.
    pub const fn page(name: &'static str, base: fn(&mut S::App)) -> Self {
        Self { name, base }
    }

    fn build(&self) -> Probe<S> {
        let mut app = S::fresh();
        (self.base)(&mut app);
        Probe::new(app, S::theme())
    }

    /// Run the case at `size` and report what it reached.
    pub fn run(&self, size: (u16, u16)) -> Reach {
        let mut probe = self.build();
        let want = probe.reference(REFERENCE);
        probe.resize(size.0, size.1);
        let small = S::too_small(probe.rt.app());
        let mut reached = Words::default();
        let bars = probe
            .explore(&want, &mut reached)
            .unwrap_or_else(|refusal| panic!("reach exploration refused: {refusal:?}"));
        Reach::of(size, small, &want, &reached, bars)
    }
}

/// A scenario probe driving the runtime over a plain test buffer.
pub(crate) struct Probe<S: Scenario> {
    rt: Runtime<S::App>,
    buf: Buffer,
    area: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScrollTarget {
    owner: Id,
    axis: Axis,
    bar: Option<Bar>,
    track: Option<Rect>,
}

impl ScrollTarget {
    const fn has_bar_route(self) -> bool {
        self.bar.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BodyAnchor {
    at: Position,
    headroom: termrock_runtime::hit::Headroom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExploreRefusal {
    InteriorStart {
        owner: Id,
        axis: Axis,
        negative: u16,
        positive: u16,
    },
    LostAnchor {
        owner: Id,
        axis: Axis,
    },
    BodySweepNotReached {
        owner: Id,
        axis: Axis,
    },
    ReturnNotReached {
        owner: Id,
        axis: Axis,
    },
}

impl<S: Scenario> Probe<S> {
    fn new(app: S::App, theme: ThemeType) -> Self {
        let area = Rect::new(0, 0, 80, 24);
        let mut rt = Runtime::new(app, theme);
        let buf = Buffer::empty(area);
        let _ = rt.initialize();
        let mut probe = Self { rt, buf, area };
        probe.draw();
        probe
    }

    /// Settle pending work, then draw one committed frame.
    fn draw(&mut self) {
        for _ in 0..16 {
            if self.rt.needs_settle() {
                let _ = self.rt.settle();
            }
            if !self.rt.needs_present() && !self.rt.needs_settle() {
                return;
            }
            self.rt
                .draw_buffer(self.area, &mut self.buf)
                .commit_presented();
        }
    }

    fn handle(&mut self, input: Input) -> Result<(), PendingInput> {
        // Keep a rejected input available to the caller. The harness must not
        // retry it until a frame is presented or silently discard it.
        let _ = self.rt.handle(input)?;
        self.draw();
        Ok(())
    }

    fn resize(&mut self, w: u16, h: u16) {
        self.area = Rect::new(0, 0, w, h);
        self.buf = Buffer::empty(self.area);
        self.handle(Input::Resize(w, h))
            .expect("a published probe must accept input");
    }

    fn buffer(&self) -> &Buffer {
        &self.buf
    }
}

impl<S: Scenario> Probe<S> {
    fn registry(&self) -> Option<&Registry> {
        Some(self.rt.registry())
    }

    fn bars(&self) -> Vec<Bar> {
        let Some(hits) = self.registry() else {
            return Vec::new();
        };
        let top = self.rt.top_layer();
        hits.bars()
            .iter()
            .copied()
            .filter(|bar| Self::bar_is_on_layer(hits, *bar, top))
            .collect()
    }

    fn bar_is_on_layer(hits: &Registry, bar: Bar, layer: termrock_runtime::layer::LayerId) -> bool {
        hits.regions().iter().any(|region| {
            region.owner == bar.owner
                && region.kind == RegionKind::Part
                && region.part.part == termrock_core::id::Part::TRACK
                && region.area == bar.track
                && region.layer == layer
        })
    }

    /// Discover the native scroll registrations in the active layer, then
    /// retain a visible-bar route where one is registered for that owner/axis.
    fn scroll_targets(&self) -> Vec<ScrollTarget> {
        let Some(hits) = self.registry() else {
            return Vec::new();
        };
        let top = self.rt.top_layer();
        let bars = self.bars();
        let mut targets = Vec::new();
        for bar in bars.iter().copied() {
            if !targets
                .iter()
                .any(|target: &ScrollTarget| target.owner == bar.owner && target.axis == bar.axis)
            {
                targets.push(ScrollTarget {
                    owner: bar.owner,
                    axis: bar.axis,
                    bar: Some(bar),
                    track: None,
                });
            }
        }
        for region in hits.regions() {
            if region.layer != top || region.kind != RegionKind::Scroll || !region.pointer_enabled()
            {
                continue;
            }
            let Some((axes, _)) = region.scroll() else {
                continue;
            };
            for axis in [Axis::V, Axis::H] {
                if !axes.handles(axis)
                    || targets
                        .iter()
                        .any(|target| target.owner == region.owner && target.axis == axis)
                {
                    continue;
                }
                let bar = bars
                    .iter()
                    .copied()
                    .find(|bar| bar.owner == region.owner && bar.axis == axis);
                let track = bar
                    .is_none()
                    .then(|| Self::track_of(hits, region.owner, axis, top))
                    .flatten();
                if bar.is_none()
                    && track.is_none()
                    && self.body_anchor(region.owner, axis).is_none()
                {
                    continue;
                }
                targets.push(ScrollTarget {
                    owner: region.owner,
                    axis,
                    bar,
                    track,
                });
            }
        }
        targets
    }

    fn track_of(
        hits: &Registry,
        owner: Id,
        axis: Axis,
        layer: termrock_runtime::layer::LayerId,
    ) -> Option<Rect> {
        hits.regions()
            .iter()
            .rev()
            .find(|region| {
                let vertical_track = region.area.width == 1 && region.area.height > 1;
                let horizontal_track = region.area.height == 1 && region.area.width > 1;
                region.owner == owner
                    && region.layer == layer
                    && region.kind == RegionKind::Part
                    && region.pointer_enabled()
                    && region.part.part == termrock_core::id::Part::TRACK
                    && match axis {
                        Axis::V => vertical_track,
                        Axis::H => horizontal_track,
                    }
            })
            .map(|region| region.area)
    }

    /// The newest matching active-layer registration gets first choice of a
    /// clipped body anchor. Every candidate is checked against the registry's
    /// actual hit resolution, so an overlapping owner cannot steal a wheel.
    fn body_anchor(&self, owner: Id, axis: Axis) -> Option<BodyAnchor> {
        let hits = self.registry()?;
        let top = self.rt.top_layer();
        for region in hits.regions().iter().rev() {
            if region.layer != top
                || region.kind != RegionKind::Scroll
                || !region.pointer_enabled()
                || region.owner != owner
                || !region.scroll().is_some_and(|(axes, _)| axes.handles(axis))
            {
                continue;
            }
            if let Some(anchor) = find_body_anchor(hits, region.area, owner, axis, top) {
                return Some(anchor);
            }
        }
        None
    }

    fn wheel_body(
        &mut self,
        owner: Id,
        axis: Axis,
        positive: bool,
    ) -> Result<BodyAnchor, ExploreRefusal> {
        let anchor = self
            .body_anchor(owner, axis)
            .ok_or(ExploreRefusal::LostAnchor { owner, axis })?;
        let kind = MouseKind::Wheel(axis, if positive { 1 } else { -1 });
        self.handle(Input::Mouse(termrock_core::event::Mouse {
            kind,
            pos: anchor.at,
            mods: termrock_core::keys::KeyModifiers::NONE,
        }))
        .expect("a published probe must accept input");
        self.body_anchor(owner, axis)
            .ok_or(ExploreRefusal::LostAnchor { owner, axis })
    }

    /// The position a wheel event at `bar` lands on: the owner's centre when
    /// it is under the scroll stack, else beside the track.
    fn wheel_spot(&self, bar: Bar) -> Position {
        self.visible_track_spot(bar.owner, bar.axis, bar.track)
    }

    fn visible_track_spot(&self, owner: Id, axis: Axis, track: Rect) -> Position {
        let Some(hits) = self.registry() else {
            return Position::ORIGIN;
        };
        let reaches = |p: Position| hits.hit_scroll(p, axis).is_some();
        let middle = hits
            .regions()
            .iter()
            .rev()
            .find(|r| r.owner == owner && r.kind == RegionKind::Control)
            .map(|r| Position::new(r.area.x + r.area.width / 2, r.area.y + r.area.height / 2));
        if let Some(p) = middle.filter(|&p| reaches(p)) {
            return p;
        }
        match axis {
            Axis::V => Position::new(track.x.saturating_sub(1), track.y + track.height / 2),
            Axis::H => Position::new(track.x + track.width / 2, track.y.saturating_sub(1)),
        }
    }

    fn wheel(&mut self, bar: Bar, forward: bool) {
        let delta = (notches(bar) as i16) * if forward { 1 } else { -1 };
        let kind = MouseKind::Wheel(bar.axis, delta);
        let at = self.wheel_spot(bar);
        self.handle(Input::Mouse(termrock_core::event::Mouse {
            kind,
            pos: at,
            mods: termrock_core::keys::KeyModifiers::NONE,
        }))
        .expect("a published probe must accept input");
    }

    fn wheel_target(&mut self, target: ScrollTarget, forward: bool) {
        let Some(bar) = target.bar else {
            return;
        };
        self.wheel(bar, forward);
    }

    /// One wheel page. `true` when the frame changed.
    fn page(&mut self, owner: Id, axis: Axis, forward: bool) -> bool {
        let Some(target) = self.scroll_target(owner, axis) else {
            return false;
        };
        if !target.has_bar_route() {
            return false;
        }
        let before = self.buffer().clone();
        self.wheel_target(target, forward);
        *self.buffer() != before
    }

    fn page_to_end(&mut self, owner: Id, axis: Axis, forward: bool, acc: &mut Acc) -> usize {
        let limit = match axis {
            Axis::V => STALL.0,
            Axis::H => STALL.1,
        };
        let mut pages = 0;
        let mut stalled = 0;
        while pages < MAX_PAGES && stalled < limit && !acc.done() && self.page(owner, axis, forward)
        {
            pages += 1;
            stalled = if acc.add(self.buffer()) {
                0
            } else {
                stalled + 1
            };
        }
        pages
    }

    fn page_back(&mut self, owner: Id, axis: Axis, forward: bool, pages: usize) {
        if pages == 0 {
            return;
        }
        let Some(target) = self.scroll_target(owner, axis) else {
            return;
        };
        if !target.has_bar_route() {
            return;
        }
        let Some(bar) = target.bar else {
            return;
        };
        for _ in 0..pages * usize::from(notches(bar)) {
            self.wheel_target(target, !forward);
        }
    }

    /// The reference words at [`REFERENCE`], minus the reference size's own
    /// dimensions so the size itself never counts as missing.
    fn reference(&mut self, reference: (u16, u16)) -> Words {
        self.resize(reference.0, reference.1);
        let mut want = Words::read(self.buffer());
        let dims = [reference.0.to_string(), reference.1.to_string()];
        want.whole.retain(|w| !dims.contains(w));
        want.cut.clear();
        want
    }

    /// Wheel every active-layer scroll owner, collecting the words it shows.
    /// Barless bodies are explored only from a known boundary and are returned
    /// to that same boundary by native opposite-direction wheels.
    fn explore(&mut self, want: &Words, reached: &mut Words) -> Result<usize, ExploreRefusal> {
        self.preflight_body_starts()?;
        let left = want.against(reached).0;
        let mut acc = Acc {
            reached: std::mem::take(reached),
            left,
            want: want.whole.clone(),
            last: BTreeSet::new(),
        };
        acc.add(self.buffer());
        let mut done: Vec<Id> = Vec::new();
        self.explore_owners(&mut acc, &mut done, 0)?;
        *reached = acc.reached;
        Ok(done.len())
    }

    fn preflight_body_starts(&self) -> Result<(), ExploreRefusal> {
        for target in self.scroll_targets() {
            if target.has_bar_route() {
                continue;
            }
            let anchor =
                self.body_anchor(target.owner, target.axis)
                    .ok_or(ExploreRefusal::LostAnchor {
                        owner: target.owner,
                        axis: target.axis,
                    })?;
            let (negative, positive) = axis_headroom(anchor.headroom, target.axis);
            if negative > 0 && positive > 0 {
                return Err(ExploreRefusal::InteriorStart {
                    owner: target.owner,
                    axis: target.axis,
                    negative,
                    positive,
                });
            }
        }
        Ok(())
    }

    fn explore_owners(
        &mut self,
        acc: &mut Acc,
        done: &mut Vec<Id>,
        depth: u8,
    ) -> Result<(), ExploreRefusal> {
        let targets = self.scroll_targets();
        let mut owners = Vec::new();
        for target in targets {
            if !owners.contains(&target.owner) {
                owners.push(target.owner);
            }
        }
        for owner in owners {
            if done.contains(&owner) || acc.done() {
                continue;
            }
            done.push(owner);
            let v = self.scroll_target(owner, Axis::V);
            let h = self.scroll_target(owner, Axis::H);

            if let Some(target) = h {
                self.sweep_horizontal(target, acc)?;
            }

            let mut v_pages = 0;
            let mut v_body_start_positive = None;
            if let Some(target) = v {
                if target.has_bar_route() {
                    let mut stalled = 0;
                    loop {
                        if let Some(horizontal) = h {
                            self.sweep_horizontal(horizontal, acc)?;
                        }
                        if acc.done() || v_pages >= MAX_PAGES || stalled >= STALL.0 {
                            break;
                        }
                        if !self.page(owner, Axis::V, true) {
                            break;
                        }
                        v_pages += 1;
                        stalled = if acc.add(self.buffer()) {
                            0
                        } else {
                            stalled + 1
                        };
                    }
                } else {
                    v_body_start_positive = Some(self.sweep_body_vertical(target, h, acc)?);
                }
            }
            if depth == 0 {
                self.explore_owners(acc, done, depth + 1)?;
            }
            if let Some(start_positive) = v_body_start_positive {
                self.return_body_to_start(owner, Axis::V, start_positive)?;
            } else if v.is_some_and(ScrollTarget::has_bar_route) {
                self.page_back(owner, Axis::V, true, v_pages);
            }

            for axis in [Axis::V, Axis::H] {
                if self
                    .scroll_target(owner, axis)
                    .is_some_and(ScrollTarget::has_bar_route)
                    && !acc.done()
                {
                    let pages = self.page_to_end(owner, axis, false, acc);
                    self.page_back(owner, axis, false, pages);
                }
            }
        }
        Ok(())
    }

    fn scroll_target(&self, owner: Id, axis: Axis) -> Option<ScrollTarget> {
        self.scroll_targets()
            .into_iter()
            .find(|target| target.owner == owner && target.axis == axis)
    }

    fn sweep_horizontal(
        &mut self,
        target: ScrollTarget,
        acc: &mut Acc,
    ) -> Result<(), ExploreRefusal> {
        if target.axis != Axis::H || acc.done() {
            return Ok(());
        }
        if target.has_bar_route() {
            let pages = self.page_to_end(target.owner, Axis::H, true, acc);
            self.page_back(target.owner, Axis::H, true, pages);
        } else {
            self.sweep_body_axis(target.owner, target.axis, acc)?;
        }
        Ok(())
    }

    fn sweep_body_vertical(
        &mut self,
        target: ScrollTarget,
        horizontal: Option<ScrollTarget>,
        acc: &mut Acc,
    ) -> Result<bool, ExploreRefusal> {
        let anchor = self
            .body_anchor(target.owner, Axis::V)
            .ok_or(ExploreRefusal::LostAnchor {
                owner: target.owner,
                axis: Axis::V,
            })?;
        let (_, positive_headroom) = axis_headroom(anchor.headroom, Axis::V);
        let (negative_headroom, _) = axis_headroom(anchor.headroom, Axis::V);
        let start_positive = if positive_headroom == 0 {
            true
        } else if negative_headroom == 0 {
            false
        } else {
            return Err(ExploreRefusal::InteriorStart {
                owner: target.owner,
                axis: Axis::V,
                negative: negative_headroom,
                positive: positive_headroom,
            });
        };
        if negative_headroom == 0 && positive_headroom == 0 {
            return Ok(start_positive);
        }

        let mut pages = 0;
        let mut stalled = 0;
        while pages < MAX_PAGES && stalled < STALL.0 && !acc.done() {
            let before =
                self.body_anchor(target.owner, Axis::V)
                    .ok_or(ExploreRefusal::LostAnchor {
                        owner: target.owner,
                        axis: Axis::V,
                    })?;
            let remaining = axis_edge(before.headroom, Axis::V, !start_positive);
            if remaining == 0 {
                break;
            }
            let next = self.wheel_body(target.owner, Axis::V, !start_positive)?;
            pages += 1;
            let next_remaining = axis_edge(next.headroom, Axis::V, !start_positive);
            let words_progressed = acc.add(self.buffer());
            stalled = if words_progressed || next_remaining < remaining {
                0
            } else {
                stalled + 1
            };
            if let Some(horizontal) = horizontal {
                self.sweep_horizontal(horizontal, acc)?;
            }
            if next_remaining == 0 {
                break;
            }
        }
        let remaining = axis_edge(
            self.body_anchor(target.owner, Axis::V)
                .ok_or(ExploreRefusal::LostAnchor {
                    owner: target.owner,
                    axis: Axis::V,
                })?
                .headroom,
            Axis::V,
            !start_positive,
        );
        if !acc.done() && remaining != 0 && (pages >= MAX_PAGES || stalled >= STALL.0) {
            return Err(ExploreRefusal::BodySweepNotReached {
                owner: target.owner,
                axis: Axis::V,
            });
        }
        Ok(start_positive)
    }

    fn sweep_body_axis(
        &mut self,
        owner: Id,
        axis: Axis,
        acc: &mut Acc,
    ) -> Result<(), ExploreRefusal> {
        let anchor = self
            .body_anchor(owner, axis)
            .ok_or(ExploreRefusal::LostAnchor { owner, axis })?;
        let (negative, positive) = axis_headroom(anchor.headroom, axis);
        if negative > 0 && positive > 0 {
            return Err(ExploreRefusal::InteriorStart {
                owner,
                axis,
                negative,
                positive,
            });
        }
        let start_positive = positive == 0;
        if negative == 0 && positive == 0 {
            return Ok(());
        }
        let limit = axis_stall_limit(axis);
        let mut pages = 0;
        let mut stalled = 0;
        while pages < MAX_PAGES && stalled < limit && !acc.done() {
            let before = self
                .body_anchor(owner, axis)
                .ok_or(ExploreRefusal::LostAnchor { owner, axis })?;
            let remaining = axis_edge(before.headroom, axis, !start_positive);
            if remaining == 0 {
                break;
            }
            let next = self.wheel_body(owner, axis, !start_positive)?;
            pages += 1;
            let next_remaining = axis_edge(next.headroom, axis, !start_positive);
            let words_progressed = acc.add(self.buffer());
            stalled = if words_progressed || next_remaining < remaining {
                0
            } else {
                stalled + 1
            };
            if next_remaining == 0 {
                break;
            }
        }
        let remaining = axis_edge(
            self.body_anchor(owner, axis)
                .ok_or(ExploreRefusal::LostAnchor { owner, axis })?
                .headroom,
            axis,
            !start_positive,
        );
        if !acc.done() && remaining != 0 && (pages >= MAX_PAGES || stalled >= limit) {
            return Err(ExploreRefusal::BodySweepNotReached { owner, axis });
        }
        self.return_body_to_start(owner, axis, start_positive)
    }

    fn return_body_to_start(
        &mut self,
        owner: Id,
        axis: Axis,
        start_positive: bool,
    ) -> Result<(), ExploreRefusal> {
        let limit = axis_stall_limit(axis);
        let mut stalled = 0;
        for _ in 0..MAX_PAGES {
            let before = self
                .body_anchor(owner, axis)
                .ok_or(ExploreRefusal::LostAnchor { owner, axis })?;
            let remaining = axis_edge(before.headroom, axis, start_positive);
            if remaining == 0 {
                return Ok(());
            }
            let after = self.wheel_body(owner, axis, start_positive)?;
            let next = axis_edge(after.headroom, axis, start_positive);
            stalled = if next < remaining { 0 } else { stalled + 1 };
            if stalled >= limit {
                return Err(ExploreRefusal::ReturnNotReached { owner, axis });
            }
        }
        if self
            .body_anchor(owner, axis)
            .is_some_and(|anchor| axis_edge(anchor.headroom, axis, start_positive) == 0)
        {
            Ok(())
        } else {
            Err(ExploreRefusal::ReturnNotReached { owner, axis })
        }
    }
}

fn axis_headroom(headroom: termrock_runtime::hit::Headroom, axis: Axis) -> (u16, u16) {
    match axis {
        Axis::V => (headroom.up, headroom.down),
        Axis::H => (headroom.left, headroom.right),
    }
}

fn axis_edge(headroom: termrock_runtime::hit::Headroom, axis: Axis, positive: bool) -> u16 {
    let (negative, positive_headroom) = axis_headroom(headroom, axis);
    if positive {
        positive_headroom
    } else {
        negative
    }
}

fn axis_stall_limit(axis: Axis) -> usize {
    match axis {
        Axis::V => STALL.0,
        Axis::H => STALL.1,
    }
}

fn find_body_anchor(
    hits: &Registry,
    area: Rect,
    owner: Id,
    axis: Axis,
    top: termrock_runtime::layer::LayerId,
) -> Option<BodyAnchor> {
    if area.is_empty() {
        return None;
    }
    let center = Position::new(
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    if let Some(anchor) = body_anchor_at(hits, center, owner, axis, top) {
        return Some(anchor);
    }

    let width = usize::from(area.width);
    let cells = width
        .saturating_mul(usize::from(area.height))
        .min(MAX_ANCHOR_CELLS);
    for index in 0..cells {
        let at = Position::new(
            area.x.saturating_add((index % width) as u16),
            area.y.saturating_add((index / width) as u16),
        );
        if at != center
            && let Some(anchor) = body_anchor_at(hits, at, owner, axis, top)
        {
            return Some(anchor);
        }
    }
    None
}

fn body_anchor_at(
    hits: &Registry,
    at: Position,
    owner: Id,
    axis: Axis,
    top: termrock_runtime::layer::LayerId,
) -> Option<BodyAnchor> {
    let hit = hits.hit_scroll(at, axis)?;
    if hit.owner != owner || hit.layer != top {
        return None;
    }
    let region = hits.regions().iter().rev().find(|region| {
        region.owner == owner
            && region.part == hit.part
            && region.layer == top
            && region.kind == RegionKind::Scroll
            && region.pointer_enabled()
            && region.area.contains(at)
            && region.scroll().is_some_and(|(axes, _)| axes.handles(axis))
    })?;
    let (_, headroom) = region.scroll()?;
    Some(BodyAnchor { at, headroom })
}

fn notches(bar: Bar) -> u16 {
    track_notches(bar.axis, bar.track)
}

fn track_notches(axis: Axis, track: Rect) -> u16 {
    match axis {
        Axis::V if track.height < 9 => 1,
        Axis::V => (track.height / 3 - 1).max(1),
        Axis::H => 1,
    }
}

struct Acc {
    reached: Words,
    left: BTreeSet<String>,
    want: BTreeSet<String>,
    last: BTreeSet<String>,
}

impl Acc {
    fn add(&mut self, buf: &Buffer) -> bool {
        let frame = Words::read(buf);
        let fresh = frame
            .whole
            .iter()
            .any(|w| self.want.contains(w) && !self.last.contains(w));
        self.last.clone_from(&frame.whole);
        if !self.left.is_empty() {
            let probe = Words {
                whole: std::mem::take(&mut self.left),
                cut: BTreeSet::new(),
            };
            self.left = probe.against(&frame).0;
        }
        self.reached.extend(frame);
        fresh
    }

    fn done(&self) -> bool {
        self.left.is_empty()
    }
}

/// Check the found reach against the allow file. Every scenario line reads
/// `<app> <scenario> <WxH> <word...> # <reason>`; a fixed row must drop its
/// line and a new row must carry a reason.
pub fn gate(app: &str, found: &[(String, Reach)], allow: &str) -> Result<(), String> {
    let mut allowed: BTreeMap<(String, String), (BTreeSet<String>, String)> = BTreeMap::new();
    for line in allow.lines() {
        let (body, note) = line.split_once('#').unwrap_or((line, ""));
        let mut it = body.split_whitespace();
        if it.next() != Some(app) {
            continue;
        }
        let (Some(scenario), Some(size)) = (it.next(), it.next()) else {
            continue;
        };
        let (scenario, only) = match scenario.split_once('@') {
            Some((name, plat)) => (name, Some(plat)),
            None => (scenario, None),
        };
        let here = match ClientPlatform::current() {
            ClientPlatform::MacOs => "macos",
            ClientPlatform::Other => "other",
        };
        if only.is_some_and(|p| p != here) {
            continue;
        }
        if only.is_none() && allowed.contains_key(&(scenario.to_string(), size.to_string())) {
            continue;
        }
        let shown = |r: &Reach| format!("{}x{}", r.size.0, r.size.1) == size;
        if !found.iter().any(|(s, r)| s == scenario && shown(r)) {
            continue;
        }
        allowed.insert(
            (scenario.to_string(), size.to_string()),
            (it.map(str::to_string).collect(), note.trim().to_string()),
        );
    }
    let mut want = String::new();
    let mut bad = Vec::new();
    for (scenario, r) in found {
        let size = format!("{}x{}", r.size.0, r.size.1);
        let key = (scenario.clone(), size.clone());
        let (ok, note) = allowed.remove(&key).unwrap_or_default();
        if r.missing.is_empty() {
            if !ok.is_empty() {
                bad.push(format!("{scenario} {size}: fixed, drop its line"));
            }
            continue;
        }
        let words: Vec<&str> = r.missing.iter().map(String::as_str).collect();
        let note = if note.is_empty() {
            "R-apps-? <reason>"
        } else {
            &note
        };
        want.push_str(&format!(
            "{app} {scenario} {size} {} # {note}\n",
            words.join(" ")
        ));
        if ok != r.missing {
            let new: Vec<_> = r.missing.difference(&ok).collect();
            let gone: Vec<_> = ok.difference(&r.missing).collect();
            bad.push(format!(
                "{scenario} {size}: newly hidden {new:?}, now reached {gone:?}"
            ));
        }
    }
    for (scenario, size) in allowed.keys() {
        bad.push(format!("{scenario} {size}: no such row, drop its line"));
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{app}: reachability differs from tests/reach_allow.txt\n  {}\n\
             {app}'s lines should read:\n{want}",
            bad.join("\n  ")
        ))
    }
}

/// Check one case at one size against the allow file.
pub fn check<S: Scenario>(app: &str, cases: &[Case<S>], name: &str, size: (u16, u16), allow: &str) {
    let case = cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"));
    let found = [(name.to_string(), case.run(size))];
    if let Err(e) = gate(app, &found, allow) {
        panic!("{e}");
    }
}

/// Dump one case's frame and reach report under `REACH_DUMP=name[:w[:h]]`.
pub fn dump<S: Scenario>(cases: &[Case<S>]) {
    let Ok(spec) = std::env::var("REACH_DUMP") else {
        return;
    };
    let mut it = spec.split(':');
    let name = it.next().unwrap_or_default();
    let mut num = || it.next().and_then(|n| n.parse().ok()).unwrap_or(80);
    let size = (num(), num());
    let case = cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"));
    let mut probe = case.build();
    probe.resize(size.0, size.1);
    let buf = probe.buffer();
    let mut text = String::new();
    for y in 0..buf.area().height {
        for x in 0..buf.area().width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    let want = probe.reference(size);
    let mut reached = Words::default();
    probe
        .explore(&want, &mut reached)
        .unwrap_or_else(|refusal| panic!("reach exploration refused: {refusal:?}"));
    let r = Reach::of(size, false, &want, &reached, 0);
    eprintln!("{text}\nmissing {:?}\ncut {:?}", r.missing, r.cut);
}

/// Declare the per-size reach tests for a scenario's cases.
#[macro_export]
macro_rules! reach_tests {
    ($app:literal, $s:ty, $cases:expr; $($test:ident = $name:literal,)*) => {
        const REACH_ALLOW: &str = "";

        $(
            mod $test {
                use super::*;

                fn at(size: (u16, u16)) {
                    $crate::reach::check::<$s>($app, &$cases, $name, size, REACH_ALLOW);
                }

                #[test]
                fn s40x12() {
                    at((40, 12));
                }

                #[test]
                fn s60x18() {
                    at((60, 18));
                }

                #[test]
                fn s72x20() {
                    at((72, 20));
                }

                #[test]
                fn s80x24() {
                    at((80, 24));
                }

                #[test]
                fn s100x25() {
                    at((100, 25));
                }

                #[test]
                fn s106x29() {
                    at((106, 29));
                }

                #[test]
                fn s120x40() {
                    at((120, 40));
                }

                #[test]
                fn s160x50() {
                    at((160, 50));
                }
            }
        )*

        #[test]
        fn every_case_has_its_test() {
            assert_eq!(
                $crate::reach::SIZES,
                [(40, 12), (60, 18), (72, 20), (80, 24), (100, 25), (106, 29), (120, 40), (160, 50)]
            );
            let tested = [$($name),*];
            for case in $cases {
                assert!(tested.contains(&case.name), "{} has no reach test", case.name);
            }
        }

        #[test]
        fn reach_dump() {
            $crate::reach::dump::<$s>(&$cases);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use termrock_core::intent::Intent;
    use termrock_runtime::{Cx, LayerSpec, Response, ScrollRegion, Ui, UpdateCause};

    struct CountingApp {
        updates: Rc<Cell<usize>>,
    }

    impl App for CountingApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            self.updates.set(self.updates.get() + 1);
            Response::ignored()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    struct CountingScenario;

    impl Scenario for CountingScenario {
        type App = CountingApp;

        fn fresh() -> Self::App {
            CountingApp {
                updates: Rc::new(Cell::new(0)),
            }
        }
    }

    const BODY: Id = Id::root("reach.body");
    const OVERLAP_A: Id = Id::root("reach.overlap.a");
    const OVERLAP_B: Id = Id::root("reach.overlap.b");
    const DUPLICATE: Id = Id::root("reach.duplicate");
    const MODAL_BODY: Id = Id::root("reach.modal.body");
    const MODAL: Id = Id::root("reach.modal");
    const ROUTED: Id = Id::root("reach.routed");
    const VIEWPORT: Id = Id::root("reach.viewport");

    struct ScrollFixture {
        id: Id,
        state: termrock::ScrollState,
        content_len: usize,
        visible_bar: bool,
        moving_area: bool,
        hide_after_wheel: bool,
        reserve_footer: bool,
        ignore_wheels: bool,
        wheel_positions: Rc<RefCell<Vec<Position>>>,
    }

    impl ScrollFixture {
        fn new(id: Id, content_len: usize, visible_bar: bool) -> Self {
            ScrollFixture {
                id,
                state: termrock::ScrollState::new(content_len),
                content_len,
                visible_bar,
                moving_area: false,
                hide_after_wheel: false,
                reserve_footer: false,
                ignore_wheels: false,
                wheel_positions: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl App for ScrollFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            for intent in cx.intents(self.id) {
                if let Intent::Wheel { pos, .. } = intent {
                    self.wheel_positions.borrow_mut().push(pos);
                }
            }
            if self.ignore_wheels {
                return Response::ignored().for_id(self.id);
            }
            ScrollRegion::new(self.id)
                .scrollbar_visible(self.visible_bar)
                .update(cx, &mut self.state, self.content_len)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            if self.hide_after_wheel && !self.wheel_positions.borrow().is_empty() {
                return;
            }
            let full = ui.full();
            let area = if self.moving_area {
                Rect::new(
                    (self.state.offset() % 8) as u16,
                    full.y,
                    20.min(full.width),
                    full.height,
                )
            } else {
                full
            };
            let area = if self.reserve_footer {
                Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1))
            } else {
                area
            };
            let content = ScrollRegion::new(self.id)
                .scrollbar_visible(self.visible_bar)
                .draw(ui, area, &self.state, self.content_len);
            paint_scroll_rows(ui, content, self.state.offset(), self.content_len);
        }
    }

    struct ScrollScenario;

    impl Scenario for ScrollScenario {
        type App = ScrollFixture;

        fn fresh() -> Self::App {
            ScrollFixture::new(BODY, 140, false)
        }
    }

    struct OverlapFixture {
        a: termrock::ScrollState,
        b: termrock::ScrollState,
    }

    impl App for OverlapFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            ScrollRegion::new(OVERLAP_A)
                .scrollbar_visible(false)
                .update(cx, &mut self.a, 80)
                | ScrollRegion::new(OVERLAP_B)
                    .scrollbar_visible(false)
                    .update(cx, &mut self.b, 80)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            ScrollRegion::new(OVERLAP_A)
                .scrollbar_visible(false)
                .draw(ui, area, &self.a, 80);
            ScrollRegion::new(OVERLAP_B)
                .scrollbar_visible(false)
                .draw(ui, area, &self.b, 80);
        }
    }

    struct OverlapScenario;

    impl Scenario for OverlapScenario {
        type App = OverlapFixture;

        fn fresh() -> Self::App {
            OverlapFixture {
                a: termrock::ScrollState::new(80),
                b: termrock::ScrollState::new(80),
            }
        }
    }

    struct DuplicateFixture {
        state: termrock::ScrollState,
    }

    impl App for DuplicateFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            ScrollRegion::new(DUPLICATE)
                .scrollbar_visible(false)
                .update(cx, &mut self.state, 80)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            let left = Rect::new(full.x, full.y, full.width / 2, full.height);
            let right = Rect::new(
                full.x.saturating_add(full.width / 2),
                full.y,
                full.width.saturating_sub(full.width / 2),
                full.height,
            );
            let region = ScrollRegion::new(DUPLICATE).scrollbar_visible(false);
            region.draw(ui, left, &self.state, 80);
            region.draw(ui, right, &self.state, 80);
        }
    }

    struct DuplicateScenario;

    impl Scenario for DuplicateScenario {
        type App = DuplicateFixture;

        fn fresh() -> Self::App {
            DuplicateFixture {
                state: termrock::ScrollState::new(80),
            }
        }
    }

    struct ModalFixture {
        lower: termrock::ScrollState,
        modal: termrock::ScrollState,
    }

    impl App for ModalFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.open_layer(MODAL, LayerSpec::modal(MODAL));
            }
            ScrollRegion::new(BODY)
                .scrollbar_visible(false)
                .update(cx, &mut self.lower, 80)
                | ScrollRegion::new(MODAL_BODY)
                    .scrollbar_visible(false)
                    .update(cx, &mut self.modal, 80)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            ScrollRegion::new(BODY)
                .scrollbar_visible(false)
                .draw(ui, full, &self.lower, 80);
            let _ = ui.layer(MODAL, |ui, area| {
                ScrollRegion::new(MODAL_BODY).scrollbar_visible(false).draw(
                    ui,
                    area,
                    &self.modal,
                    80,
                );
            });
        }
    }

    struct ModalScenario;

    impl Scenario for ModalScenario {
        type App = ModalFixture;

        fn fresh() -> Self::App {
            ModalFixture {
                lower: termrock::ScrollState::new(80),
                modal: termrock::ScrollState::new(80),
            }
        }
    }

    struct RoutedFixture {
        state: termrock::ScrollState,
        attempts: Rc<Cell<usize>>,
    }

    impl App for RoutedFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx
                .wheel_route_attempt(
                    ROUTED,
                    termrock_core::id::PartRef::of(termrock_core::id::Part::CONTAINER),
                    Axis::V,
                )
                .is_some()
            {
                self.attempts.set(self.attempts.get() + 1);
            }
            ScrollRegion::new(ROUTED)
                .scrollbar_visible(false)
                .update_routed(cx, &mut self.state, 80, 7)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            let _ = ScrollRegion::new(ROUTED)
                .scrollbar_visible(false)
                .draw_routed(ui, area, &self.state, 80, 7);
        }
    }

    struct RoutedScenario;

    impl Scenario for RoutedScenario {
        type App = RoutedFixture;

        fn fresh() -> Self::App {
            RoutedFixture {
                state: termrock::ScrollState::new(80),
                attempts: Rc::new(Cell::new(0)),
            }
        }
    }

    const VIEWPORT_LINES: [termrock::ViewportLine<'static>; 16] = [
        termrock::ViewportLine::Plain(
            "row000 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row001 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row002 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row003 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row004 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row005 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row006 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row007 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row008 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row009 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row010 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row011 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row012 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row013 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row014 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
        termrock::ViewportLine::Plain(
            "row015 aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz farword",
        ),
    ];

    struct ViewportFixture {
        state: termrock::ViewportState,
    }

    impl App for ViewportFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            termrock::TextViewport::new(VIEWPORT)
                .horizontal_pan(true)
                .update(cx, &mut self.state, &VIEWPORT_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            termrock::TextViewport::new(VIEWPORT)
                .horizontal_pan(true)
                .draw(ui, area, &self.state, &VIEWPORT_LINES);
        }
    }

    struct ViewportScenario;

    impl Scenario for ViewportScenario {
        type App = ViewportFixture;

        fn fresh() -> Self::App {
            ViewportFixture {
                state: termrock::ViewportState::default(),
            }
        }
    }

    fn paint_scroll_rows(ui: &mut Ui<'_>, area: Rect, offset: usize, content_len: usize) {
        for row in 0..area.height {
            let index = offset.saturating_add(usize::from(row));
            if index >= content_len {
                break;
            }
            let word = format!("word{index:03}");
            ui.paint_str(
                Rect::new(area.x, area.y + row, area.width, 1),
                &word,
                ui.surface_style(),
            );
        }
    }

    fn ready_probe(updates: Rc<Cell<usize>>) -> Probe<CountingScenario> {
        Probe::new(CountingApp { updates }, ThemeType::junie())
    }

    #[test]
    fn accepted_probe_input_is_delivered_once() {
        let updates = Rc::new(Cell::new(0));
        let mut probe = ready_probe(Rc::clone(&updates));
        updates.set(0);

        probe
            .handle(Input::Resize(80, 24))
            .expect("a ready probe accepts one input");

        assert_eq!(updates.get(), 1);
    }

    #[test]
    fn unready_probe_returns_the_unchanged_pending_input() {
        let updates = Rc::new(Cell::new(0));
        let area = Rect::new(0, 0, 80, 24);
        let mut probe = Probe::<CountingScenario> {
            rt: Runtime::new(
                CountingApp {
                    updates: Rc::clone(&updates),
                },
                ThemeType::junie(),
            ),
            buf: Buffer::empty(area),
            area,
        };

        let pending = probe
            .handle(Input::Resize(31, 17))
            .expect_err("an uninitialized probe cannot consume input");

        assert!(matches!(pending.into_input(), Input::Resize(31, 17)));
        assert_eq!(updates.get(), 0);
    }

    fn buf(lines: &[&str]) -> Buffer {
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let mut b = Buffer::empty(Rect::new(0, 0, w, lines.len() as u16 + 1));
        for (y, l) in lines.iter().enumerate() {
            b.set_string(0, y as u16, l, ratatui_core::style::Style::default());
        }
        b
    }

    #[test]
    fn words_split_cut_and_skip_the_footer() {
        let w = Words::read(&buf(&["alpha be gamma│delt…", "x 1234"]));
        assert!(w.whole.contains("alpha"));
        assert!(w.whole.contains("gamma"));
        assert!(!w.whole.contains("be"));
        assert!(w.cut.contains("delt"));
        assert!(w.whole.contains("1234"));
        // add_area honours its region, so a caller can skip the footer row.
        let b = buf(&["alpha delta", "footer row"]);
        let mut w = Words::default();
        w.add_area(&b, Rect::new(0, 0, 11, 1));
        assert!(
            w.whole.contains("alpha") && !w.whole.contains("footer"),
            "{w:?}"
        );
    }

    #[test]
    fn against_sorts_missing_from_cut() {
        let want = Words::read(&buf(&["alpha delta omega"]));
        let got = Words::read(&buf(&["alpha delt…"]));
        let (missing, cut) = want.against(&got);
        assert_eq!(missing.into_iter().collect::<Vec<_>>(), ["omega"]);
        assert_eq!(cut.into_iter().collect::<Vec<_>>(), ["delta"]);
    }

    #[test]
    fn hidden_scroll_region_body_reaches_reference_words_and_returns_to_boundary() {
        let mut probe =
            Probe::<ScrollScenario>::new(ScrollFixture::new(BODY, 140, false), ThemeType::junie());
        let want = probe.reference(REFERENCE);
        probe.resize(40, 12);

        assert!(probe.bars().is_empty());
        assert_eq!(
            probe.scroll_targets(),
            [ScrollTarget {
                owner: BODY,
                axis: Axis::V,
                bar: None,
                track: None,
            }]
        );
        let initial = probe
            .body_anchor(BODY, Axis::V)
            .expect("body is hit-testable");
        assert_eq!(initial.headroom.up, 0);
        assert!(initial.headroom.down > 0);

        let mut reached = Words::default();
        assert_eq!(probe.explore(&want, &mut reached), Ok(1));
        assert!(reached.whole.contains("word098"));
        assert_eq!(probe.rt.app().state.offset(), 0);
        assert!(!probe.rt.app().wheel_positions.borrow().is_empty());
    }

    #[test]
    fn body_discovery_deduplicates_and_uses_the_newest_overlapping_owner() {
        let mut duplicate = Probe::<DuplicateScenario>::new(
            DuplicateFixture {
                state: termrock::ScrollState::new(80),
            },
            ThemeType::junie(),
        );
        duplicate.resize(40, 12);
        assert_eq!(
            duplicate.scroll_targets(),
            [ScrollTarget {
                owner: DUPLICATE,
                axis: Axis::V,
                bar: None,
                track: None,
            }]
        );

        let mut overlap =
            Probe::<OverlapScenario>::new(OverlapScenario::fresh(), ThemeType::junie());
        overlap.resize(40, 12);
        assert_eq!(
            overlap.scroll_targets(),
            [ScrollTarget {
                owner: OVERLAP_B,
                axis: Axis::V,
                bar: None,
                track: None,
            }]
        );
    }

    #[test]
    fn body_discovery_only_uses_the_modal_top_layer() {
        let mut probe = Probe::<ModalScenario>::new(ModalScenario::fresh(), ThemeType::junie());
        probe.resize(40, 12);
        let targets = probe.scroll_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].owner, MODAL_BODY);
        assert_eq!(targets[0].axis, Axis::V);
        assert_eq!(targets[0].bar, None);
        assert_eq!(targets[0].track, None);
        let anchor = probe.body_anchor(MODAL_BODY, Axis::V).unwrap();
        assert_eq!(
            probe
                .registry()
                .unwrap()
                .hit_scroll(anchor.at, Axis::V)
                .unwrap()
                .layer,
            probe.rt.top_layer()
        );
    }

    #[test]
    fn body_wheels_reanchor_after_each_presented_geometry_change() {
        let mut app = ScrollFixture::new(BODY, 80, false);
        app.moving_area = true;
        let positions = Rc::clone(&app.wheel_positions);
        let mut probe = Probe::<ScrollScenario>::new(app, ThemeType::junie());
        probe.resize(40, 12);
        let first = probe.body_anchor(BODY, Axis::V).unwrap().at;

        let after = probe.wheel_body(BODY, Axis::V, true).unwrap();
        assert_ne!(first, after.at);
        let _ = probe.wheel_body(BODY, Axis::V, true).unwrap();
        let recorded = positions.borrow();
        assert_eq!(recorded.len(), 2);
        assert_ne!(recorded[0], recorded[1]);
    }

    #[test]
    fn barless_interior_start_refuses_before_the_first_wheel() {
        let mut probe =
            Probe::<ScrollScenario>::new(ScrollFixture::new(BODY, 20, false), ThemeType::junie());
        probe.resize(40, 12);
        probe.rt.app_mut().state.set_viewport(12);
        probe.rt.app_mut().state.scroll_to(1);
        probe.draw();
        assert_eq!(probe.rt.app().state.max_offset(), 8);

        let mut walk = probe.rt.app().state;
        for _ in 0..3 {
            let _ = walk.wheel(3);
        }
        assert_eq!(walk.offset(), 8);
        for _ in 0..3 {
            let _ = walk.wheel(-3);
        }
        assert_eq!(
            walk.offset(),
            0,
            "max 8 / stride 3 / offset 1 cannot be reversed exactly"
        );
        assert_ne!(walk.offset(), 1);

        let want = Words {
            whole: ["word019".to_string()].into_iter().collect(),
            cut: BTreeSet::new(),
        };
        let mut reached = Words::default();
        assert_eq!(
            probe.explore(&want, &mut reached),
            Err(ExploreRefusal::InteriorStart {
                owner: BODY,
                axis: Axis::V,
                negative: 1,
                positive: 7,
            })
        );
        assert!(probe.rt.app().wheel_positions.borrow().is_empty());
        assert_eq!(probe.rt.app().state.offset(), 1);
    }

    #[test]
    fn a_lost_body_anchor_is_an_explicit_refusal() {
        let mut app = ScrollFixture::new(BODY, 80, false);
        app.hide_after_wheel = true;
        let positions = Rc::clone(&app.wheel_positions);
        let mut probe = Probe::<ScrollScenario>::new(app, ThemeType::junie());
        probe.resize(40, 12);
        let want = Words {
            whole: ["word079".to_string()].into_iter().collect(),
            cut: BTreeSet::new(),
        };
        let mut reached = Words::default();
        assert_eq!(
            probe.explore(&want, &mut reached),
            Err(ExploreRefusal::LostAnchor {
                owner: BODY,
                axis: Axis::V,
            })
        );
        assert_eq!(positions.borrow().len(), 1);
    }

    #[test]
    fn routed_barless_body_uses_the_runtime_attempt_and_acknowledges_it() {
        let app = RoutedFixture {
            state: termrock::ScrollState::new(80),
            attempts: Rc::new(Cell::new(0)),
        };
        let attempts = Rc::clone(&app.attempts);
        let theme = ThemeType::junie();
        let wheel_rows = usize::from(theme.design.motion.wheel_rows);
        let mut probe = Probe::<RoutedScenario>::new(app, theme);
        probe.resize(40, 12);
        assert_eq!(probe.scroll_targets()[0].bar, None);
        probe.wheel_body(ROUTED, Axis::V, true).unwrap();
        assert_eq!(attempts.get(), 1);
        assert_eq!(probe.rt.app().state.offset(), wheel_rows);
    }

    #[test]
    fn both_axis_native_viewport_reaches_words_through_its_body_registration() {
        let mut probe =
            Probe::<ViewportScenario>::new(ViewportScenario::fresh(), ThemeType::junie());
        let want = probe.reference(REFERENCE);
        assert!(want.whole.contains("farword"));
        probe.resize(40, 12);
        assert!(!Words::read(probe.buffer()).whole.contains("farword"));
        let targets = probe.scroll_targets();
        assert!(targets.iter().any(|target| {
            target.owner == VIEWPORT && target.axis == Axis::V && target.track.is_some()
        }));
        assert!(targets.iter().any(|target| {
            target.owner == VIEWPORT
                && target.axis == Axis::H
                && target.bar.is_none()
                && target.track.is_none()
        }));

        let mut reached = Words::default();
        assert_eq!(probe.explore(&want, &mut reached), Ok(1));
        assert!(reached.whole.contains("farword"));
        assert_eq!(probe.rt.app().state.horizontal_offset(), 0);
    }

    #[test]
    fn visible_track_without_bar_refuses_an_interior_start_before_any_wheel() {
        let mut probe =
            Probe::<ScrollScenario>::new(ScrollFixture::new(BODY, 20, true), ThemeType::junie());
        probe.resize(40, 12);
        probe.rt.app_mut().state.set_viewport(12);
        probe.rt.app_mut().state.scroll_to(1);
        probe.draw();
        let target = probe.scroll_target(BODY, Axis::V).unwrap();
        assert_eq!(target.bar, None, "do not synthesize a Bar record");
        assert!(
            target.track.is_some(),
            "the native track part is registered"
        );

        let want = Words {
            whole: ["word019".to_string()].into_iter().collect(),
            cut: BTreeSet::new(),
        };
        let mut reached = Words::default();
        assert_eq!(
            probe.explore(&want, &mut reached),
            Err(ExploreRefusal::InteriorStart {
                owner: BODY,
                axis: Axis::V,
                negative: 1,
                positive: 7,
            })
        );
        assert!(probe.rt.app().wheel_positions.borrow().is_empty());
        assert_eq!(probe.rt.app().state.offset(), 1);
    }

    #[test]
    fn visible_track_without_bar_reaches_and_returns_from_a_boundary() {
        let mut app = ScrollFixture::new(BODY, 20, true);
        app.reserve_footer = true;
        let mut probe = Probe::<ScrollScenario>::new(app, ThemeType::junie());
        let want = Words {
            whole: ["word019".to_string()].into_iter().collect(),
            cut: BTreeSet::new(),
        };
        assert!(want.whole.contains("word019"));
        probe.resize(40, 12);
        probe.rt.app_mut().state.set_viewport(11);
        probe.rt.app_mut().state.scroll_to(0);
        probe.draw();
        assert!(!Words::read(probe.buffer()).whole.contains("word019"));
        let target = probe.scroll_target(BODY, Axis::V).unwrap();
        assert_eq!(target.bar, None, "do not synthesize a Bar record");
        assert!(
            target.track.is_some(),
            "the native track part is registered"
        );
        let body = probe.body_anchor(BODY, Axis::V).unwrap();
        assert!(body.at.y < 11, "the native scroll body excludes the footer");
        assert_eq!(body.headroom.down, 9);

        let mut reached = Words::default();
        assert_eq!(probe.explore(&want, &mut reached), Ok(1));
        assert!(!probe.rt.app().wheel_positions.borrow().is_empty());
        assert!(reached.whole.contains("word019"));
        assert_eq!(probe.rt.app().state.offset(), 0);
    }

    #[test]
    fn body_sweep_refuses_a_stalled_owner_with_unresolved_words() {
        let mut app = ScrollFixture::new(BODY, 140, false);
        app.ignore_wheels = true;
        let mut probe = Probe::<ScrollScenario>::new(app, ThemeType::junie());
        probe.resize(40, 12);
        probe.rt.app_mut().state.set_viewport(12);
        probe.rt.app_mut().state.scroll_to(0);
        probe.draw();

        let want = Words {
            whole: ["word019".to_string()].into_iter().collect(),
            cut: BTreeSet::new(),
        };
        assert!(!Words::read(probe.buffer()).whole.contains("word019"));
        let before = probe.body_anchor(BODY, Axis::V).unwrap();
        assert_eq!(before.headroom.up, 0);
        assert!(before.headroom.down > 0);

        let mut reached = Words::default();
        assert_eq!(
            probe.explore(&want, &mut reached),
            Err(ExploreRefusal::BodySweepNotReached {
                owner: BODY,
                axis: Axis::V,
            })
        );
        assert_eq!(probe.rt.app().wheel_positions.borrow().len(), STALL.0);
        assert!(reached.whole.is_empty());
        assert_eq!(probe.rt.app().state.offset(), 0);
        assert_eq!(
            probe.body_anchor(BODY, Axis::V).unwrap().headroom,
            before.headroom
        );
    }
}
