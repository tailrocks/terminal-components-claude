//! `StatusBar` — the merged, priority-ordered item strip
//! (`COMPONENT_ARCHITECTURE.md` §14.1, §18.2, §18.3 items 9 and 11,
//! Appendix A 4G).
//!
//! This one component replaces both the legacy `statusbar` and `segments`:
//! `Left`/`Center`/`Right` groups of priority-ordered items, one drop order,
//! one truncation rule, inline meters and clickable item ids. The two
//! hand-written priority-drop loops in `TablePro`'s identity strip and grid
//! status line become consumers of this strip in Slice 6.

use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::{Modifier, Style};

use super::meter::{Meter, MeterTone};
use super::progress::PCT_COLUMNS;
use super::{Overrides, SlotFn, first_row, shift};
use crate::collection::Status;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// Items per group laid out without allocating.
///
/// Items beyond this cap are **silently ignored**, exactly as
/// `RowUi::columns` ignores tracks past `MAX_COLUMNS`: the fixed per-group
/// buffers are what keep the strip's priority-drop loop allocation-free
/// (§12.2, R5).
pub const MAX_ITEMS: usize = 16;

/// The three groups of a status strip.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    /// Identity and context; the leading group.
    Left,
    /// Activity; centred in whatever the other two leave.
    Center,
    /// Quotas and runtime facts; the trailing group.
    Right,
}

impl Group {
    /// Every group, in painting order.
    pub const ALL: [Group; 3] = [Group::Left, Group::Center, Group::Right];

    const fn index(self) -> usize {
        match self {
            Group::Left => 0,
            Group::Center => 1,
            Group::Right => 2,
        }
    }
}

/// How much weight an item carries.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum Emphasis {
    /// The default reading.
    #[default]
    Plain,
    /// The item that names the surface; bold.
    Strong,
    /// A fact with its own edge — a quota, a runtime id — drawn on the plane
    /// one step above the strip.
    Chip,
}

/// One fact in a status strip.
///
/// Built with `StatusItem::new(text)` plus consuming builders, like every
/// other props type (§13). The strip borrows a `&[StatusItem]` per group and
/// never holds one.
#[derive(Clone, Copy, Debug)]
pub struct StatusItem<'a> {
    text: &'a str,
    tone: Option<Role>,
    priority: u8,
    key: Option<ItemKey>,
    emphasis: Emphasis,
    ratio: Option<f64>,
    meter_tone: Option<MeterTone>,
}

impl<'a> StatusItem<'a> {
    /// The default priority; higher survives width pressure longer.
    pub const DEFAULT_PRIORITY: u8 = 5;

    /// An item reading `text`.
    pub const fn new(text: &'a str) -> Self {
        StatusItem {
            text,
            tone: None,
            priority: Self::DEFAULT_PRIORITY,
            key: None,
            emphasis: Emphasis::Plain,
            ratio: None,
            meter_tone: None,
        }
    }

    /// The colour role the item reads in; the recipe's `LABEL` tone when
    /// unset.
    #[must_use]
    pub const fn tone(mut self, r: Role) -> Self {
        self.tone = Some(r);
        self
    }

    /// Higher survives longer when the row is narrow.
    #[must_use]
    pub const fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    /// Make the item clickable and addressable; the strip emits
    /// [`StatusAction::Chose`] with this key.
    #[must_use]
    pub const fn key(mut self, k: ItemKey) -> Self {
        self.key = Some(k);
        self
    }

    /// Bold: the item that names the surface.
    #[must_use]
    pub const fn strong(mut self) -> Self {
        self.emphasis = Emphasis::Strong;
        self
    }

    /// Draw the item as a chip on the raised plane.
    #[must_use]
    pub const fn chip(mut self) -> Self {
        self.emphasis = Emphasis::Chip;
        self
    }

    /// Append a compact inline meter reporting `ratio`, clamped to
    /// `0.0..=1.0`. Its tone is derived from `design.meter` unless
    /// [`StatusItem::meter_tone`] forces one (J12).
    #[must_use]
    pub fn meter(mut self, ratio: f64) -> Self {
        self.ratio = Some(ratio.clamp(0.0, 1.0));
        self
    }

    /// Force the inline meter's tone.
    #[must_use]
    pub const fn meter_tone(mut self, t: MeterTone) -> Self {
        self.meter_tone = Some(t);
        self
    }

    /// The text.
    pub const fn text(&self) -> &'a str {
        self.text
    }

    /// The priority.
    pub const fn priority_of(&self) -> u8 {
        self.priority
    }

    /// The key, when the item is clickable.
    pub const fn key_of(&self) -> Option<ItemKey> {
        self.key
    }

    /// The emphasis.
    pub const fn emphasis(&self) -> Emphasis {
        self.emphasis
    }

    /// Columns the label alone occupies (chips carry their own padding).
    fn label_columns(&self) -> u16 {
        let w = width(self.text);
        match self.emphasis {
            Emphasis::Chip => w.saturating_add(2),
            Emphasis::Plain | Emphasis::Strong => w,
        }
    }

    /// Columns the whole item occupies, inline meter included.
    fn columns(&self, meter_columns: u16) -> u16 {
        match self.ratio {
            Some(_) => self
                .label_columns()
                .saturating_add(1)
                .saturating_add(meter_columns),
            None => self.label_columns(),
        }
    }
}

/// What a status strip reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum StatusAction {
    /// A clickable item was clicked; carries the item's own key.
    Chose(ItemKey),
}

/// A full-width strip of priority-ordered facts in three groups.
///
/// ## Construction
/// `StatusBar::new(id)`, then `.left(items)` / `.center(items)` /
/// `.right(items)` with a borrowed slice per group. Items are passed per
/// phase like every other collection's data (§12.2), so the props never
/// borrow the field an action closure mutates.
///
/// ## Ownership
/// Stateless — there is no `StatusBarState`. The caller owns the item
/// slices and the animation frame; the runtime owns hover and press, including
/// the hovered keyed item.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.left/.center/
/// .right(&[StatusItem])` (empty), `.status(Status)` (`Ready`),
/// `.frame(usize)` (`0`), `.patch`, `.patch_part`, `.slot`,
/// `.state_override`.
///
/// ## Variants
/// `Family::STATUSBAR`; `DEFAULT` only.
///
/// ## States
/// Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)` and paints a
/// leading readiness affordance for them; wears `HOVERED` and `PRESSED` from
/// the runtime. A matching keyed item alone wears `HOVERED`; per-item tone and
/// emphasis remain the item's own configuration.
///
/// ## Actions
/// `StatusAction::Chose(ItemKey)` — a click on an item that declared a
/// `.key(…)`. Items without a key register nothing and cannot be clicked.
///
/// ## Focus
/// Never a focus stop: the strip registers `Part` regions for its clickable
/// items and no ring entry, because a status strip is a report, not a
/// control an operator tabs through. `swallows_typing` is false.
///
/// ## Keyboard
/// None. A product that wants a chord for one of these facts binds it
/// through its own `KeyMap` and maps it onto the same action.
///
/// ## Mouse
/// `PartRef::item(Part::LABEL, key)` per clickable item: a click emits
/// `StatusAction::Chose(key)`.
///
/// ## Layout
/// `measure` returns `(every item at its natural width, 1)`. `draw` uses the
/// first row of `area`; when the row is too narrow items leave by ascending
/// priority — **centre first, then right, then left** — and the strongest
/// left item never leaves, truncating with `GlyphRole::Ellipsis` instead.
/// Returns the rect it painted; a degenerate rect registers nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the row fill), `LABEL` (each item), `MARKER` (the readiness
/// glyph), `ICON` (the readiness spinner), plus `TRACK`, `THUMB` and
/// `OVERFLOW` for an item's inline meter and the truncation marker.
///
/// ## Overrides
/// `.patch` and `.patch_part` on any part, forwarded to an item's inline
/// [`Meter`]. `.slot` on exactly `LABEL`, `MARKER`, `ICON`, `TRACK` and
/// `OVERFLOW`. `LABEL` answers for every item the strip paints and
/// `OVERFLOW` for every cut it marks; `TRACK` is forwarded to the inline
/// meter. `CONTAINER` is not slot-addressable, because its fill *is* the
/// strip, and neither is `THUMB`: a slot on `TRACK` replaces the whole
/// inline run, used share included, because the split between the two is the
/// meter's own arithmetic.
///
/// ## Identity
/// Items are keyed by the caller through `StatusItem::key`; there is no
/// `ByIndex` default, because an item without a key is not addressable at
/// all — it is a label, and no action can name it.
///
/// ## Testing
/// `StatusBarCase` in `crates/tui/tests/conformance.rs`, declaring
/// `Caps::empty()`, so twelve of its twenty-one `status_bar::*` cases are
/// capability-gated and return immediately, and
/// `mono_states_are_distinguishable` is narrowed to the single default
/// state. The default fixture rect is 30 columns and the fixture strip
/// needs 35, so the drop loop does run under the cases that remain — but
/// they assert byte-identity, containment, theme isolation and tiny-rect
/// survival, never which items survived.
///
/// The render matrix in `crates/tui/tests/render_components.rs` generates
/// exactly eight cells per component, one per `St` variant: there is no
/// `render::components::status_bar::busy`, no `::error` and no
/// `::overflow`. Readiness arrives through the matrix's `status_for`
/// mapping — `::pressed` is `Status::Busy`, `::editing` `Status::Loading`,
/// `::disabled` `Status::Error` — and `draw` **ors** the status-derived
/// flags into the forced ones instead of replacing them, so the `MARKER`
/// error glyph as well as the `ICON` spinner is genuinely painted, and
/// pinned as a digest, by those three cells. At the matrix's two widths the
/// strip needs 35 columns (37 with a readiness affordance) and is given 40
/// or 120, so no matrix cell drops an item.
///
/// The drop order is covered by the unit tests in this module, which call
/// `survivors` directly: `a_wide_row_keeps_every_item`,
/// `narrow_rows_drop_centre_then_right_then_left_and_keep_the_name`,
/// `ties_take_the_later_item_first` and `items_past_the_cap_are_ignored`.
/// Those masks say which items survive and nothing about where they land, so
/// the placement itself is covered by
/// `the_left_group_starts_at_the_gutter_the_right_is_flush_and_the_centre_sits_between`,
/// which reads the three groups' columns back out of the painted buffer
/// rather than out of any of the helpers above.
///
/// Exercised by no test: the painted `Part::OVERFLOW` ellipsis — the drop
/// loop is asserted, the truncation marker it leaves behind is not — and
/// `StatusAction::Chose`, because the case declares no `Caps::ACTIVATES`
/// and `status_bar::keyboard_and_mouse_activation_are_equivalent`
/// therefore returns before it clicks anything.
///
/// ## Invariants
/// The drop order is exactly centre → right → left with the strongest left
/// item retained, so a narrowing terminal loses activity before context and
/// context before identity. At most [`MAX_ITEMS`] items per group are laid
/// out. Never allocates. Only the keyed item matching the frame's hovered
/// `PartRef` receives `HOVERED`; keyboard suppression makes that lookup return
/// `None`.
pub struct StatusBar<'a> {
    id: Id,
    left: &'a [StatusItem<'a>],
    center: &'a [StatusItem<'a>],
    right: &'a [StatusItem<'a>],
    variant: Variant,
    status: Status,
    frame: usize,
    /// Kept beside `ov` so an item's inline [`Meter`] can be built with the
    /// caller's own overrides: it paints `TRACK` and `THUMB` under *this*
    /// strip's `Id`, so a bare construction dropped the caller's `.patch`
    /// and `.patch_part` on those parts where `Invariant P` could not see it
    /// (§45.7 obligation 2).
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: Overrides<'a>,
}

impl fmt::Debug for StatusBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StatusBar")
            .field("id", &self.id)
            .field("left", &self.left.len())
            .field("center", &self.center.len())
            .field("right", &self.right.len())
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

/// Which items survive, one bit per item, one mask per group.
type Keep = [u32; 3];

/// The mask with the low `n` bits set: every item of an `n`-item group alive.
///
/// A group with more items than the mask has bits keeps every bit set, which
/// is what the drop loop wants — the strip is full and items start leaving.
fn all_alive(n: usize) -> u32 {
    let n = u32::try_from(n).unwrap_or(u32::BITS);
    // `u32::MAX >> (32 - n)`. `checked_shr` answers `None` for a shift of 32
    // or more, which is exactly `n == 0`: the empty group, no bits set.
    u32::MAX
        .checked_shr(u32::BITS.saturating_sub(n))
        .unwrap_or(0)
}

impl<'a> StatusBar<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::LABEL,
        Part::MARKER,
        Part::ICON,
        Part::TRACK,
        Part::THUMB,
        Part::OVERFLOW,
    ];

    /// An empty strip.
    pub const fn new(id: Id) -> Self {
        StatusBar {
            id,
            left: &[],
            center: &[],
            right: &[],
            variant: Variant::DEFAULT,
            status: Status::Ready,
            frame: 0,
            patch: None,
            parts: &[],
            ov: Overrides::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The leading group: identity and context.
    #[must_use]
    pub const fn left(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.left = items;
        self
    }

    /// The centre group: activity.
    #[must_use]
    pub const fn center(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.center = items;
        self
    }

    /// The trailing group: quotas and runtime facts.
    #[must_use]
    pub const fn right(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.right = items;
        self
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Data readiness of the surface the strip reports on.
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// The animation frame the readiness spinner reads.
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting; layout and hit regions stay.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11): render a forced state. Such a
    /// strip registers nothing.
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    /// The items of one group, capped at [`MAX_ITEMS`].
    fn group(&self, g: Group) -> &'a [StatusItem<'a>] {
        let items = match g {
            Group::Left => self.left,
            Group::Center => self.center,
            Group::Right => self.right,
        };
        items.get(..items.len().min(MAX_ITEMS)).unwrap_or(items)
    }

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    /// Columns an inline meter occupies.
    fn meter_columns(ui: &Ui<'_>) -> u16 {
        ui.design()
            .size
            .meter_track
            .saturating_add(PCT_COLUMNS)
            .saturating_add(2)
    }

    /// The glyph slot `Part::MARKER` resolves under for **this instance**.
    ///
    /// [`Ui::resolve`] is §26 N2's `&self` measuring path: it stops at
    /// precedence 5, so an instance `.patch` or `.patch_part` reached this
    /// cell's *colour* — resolved through [`Overrides::style`] — and could
    /// not reach its *glyph* (§45.5). Precedence 6 is applied here exactly
    /// as `theme::resolve::bind` applies it on the painting path, so the two
    /// halves of one cell cannot answer to different override chains.
    fn marker_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Slot<GlyphRole> {
        let base = ui
            .resolve(Family::STATUSBAR, self.variant, Part::MARKER, live)
            .glyph;
        self.ov
            .part_patch(Part::MARKER)
            .map_or(base, |p| p.glyph.over(base))
    }

    /// The readiness affordance: the spinner while busy, the recipe's marker
    /// (or the error glyph) while in error.
    fn readiness(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if self.busy() {
            let frames = ui.design().motion.spinner_frames;
            return frames
                .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                .copied();
        }
        if live.contains(StateFlags::ERROR) {
            let g = match self.marker_glyph(ui, live) {
                Slot::Set(g) => g,
                Slot::Inherit => GlyphRole::Error,
                Slot::Clear => return None,
            };
            return Some(ui.glyph_str(g));
        }
        if live.contains(StateFlags::WARNING) {
            let g = match self.marker_glyph(ui, live) {
                Slot::Set(g) => g,
                Slot::Inherit => GlyphRole::Dirty,
                Slot::Clear => return None,
            };
            return Some(ui.glyph_str(g));
        }
        None
    }

    /// Columns a group occupies under `keep`.
    fn group_columns(&self, g: Group, keep: Keep, mw: u16, gap: u16) -> u16 {
        let items = self.group(g);
        let mask = keep.get(g.index()).copied().unwrap_or(0);
        let mut w = 0u16;
        let mut n = 0u16;
        for (i, it) in items.iter().enumerate() {
            if mask & (1 << i) != 0 {
                w = w.saturating_add(it.columns(mw));
                n = n.saturating_add(1);
            }
        }
        if n > 0 {
            w.saturating_add(n.saturating_sub(1).saturating_mul(gap))
        } else {
            0
        }
    }

    /// The keep mask with every item of every group alive.
    fn all_alive_keep(&self) -> Keep {
        Group::ALL.map(|g| all_alive(self.group(g).len()))
    }

    /// Columns the whole strip needs under `keep`.
    fn needed(&self, keep: Keep, mw: u16, gap: u16, edge: u16, lead: u16) -> u16 {
        let ws = Group::ALL.map(|g| self.group_columns(g, keep, mw, gap));
        let present = ws.iter().filter(|w| **w > 0).count().min(3) as u16;
        ws.iter()
            .fold(0u16, |a, w| a.saturating_add(*w))
            .saturating_add(present.saturating_sub(1).saturating_mul(gap))
            .saturating_add(edge.saturating_mul(2))
            .saturating_add(lead)
    }

    /// Which items survive at `total` columns.
    ///
    /// Items leave by ascending priority; on a tie the centre group loses
    /// before the right and the right before the left, and the strongest
    /// left item never leaves — it truncates instead. This is the legacy
    /// strip's order, kept verbatim so the two hand-written copies §18.3
    /// items 9 and 11 delete can be replaced without a visual review of
    /// every width.
    fn survivors(&self, total: u16, mw: u16, gap: u16, edge: u16, lead: u16) -> Keep {
        let mut keep: Keep = self.all_alive_keep();
        while self.needed(keep, mw, gap, edge, lead) > total {
            let mut victim: Option<(u8, usize, usize)> = None;
            for g in [Group::Center, Group::Right, Group::Left] {
                let gi = g.index();
                let items = self.group(g);
                let mask = keep.get(gi).copied().unwrap_or(0);
                let alive = mask.count_ones();
                if alive == 0 || (g == Group::Left && alive == 1) {
                    continue;
                }
                // the lowest priority; on a tie the later item leaves first
                let mut worst: Option<(u8, usize)> = None;
                for (i, it) in items.iter().enumerate() {
                    if mask & (1 << i) == 0 {
                        continue;
                    }
                    if worst.is_none_or(|(p, _)| it.priority <= p) {
                        worst = Some((it.priority, i));
                    }
                }
                if let Some((p, i)) = worst
                    && victim.is_none_or(|(vp, _, _)| p < vp)
                {
                    victim = Some((p, gi, i));
                }
            }
            match victim {
                Some((_, gi, i)) => {
                    if let Some(m) = keep.get_mut(gi) {
                        *m &= !(1u32 << i);
                    }
                }
                None => break,
            }
        }
        keep
    }

    /// The update phase: a click on a keyed item.
    pub fn update(&self, cx: &mut Cx<'_>) -> Response<StatusAction> {
        let mut r: Response<StatusAction> = Response::ignored();
        for it in cx.intents(self.id) {
            match it {
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part,
                    ..
                } => {
                    if let Some(k) = part.item {
                        r = Response::action(StatusAction::Chose(k));
                    }
                }
                Intent::Pointer { .. } if r.action_ref().is_none() => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    /// The style an item paints with: the `LABEL` recipe plus the item's own
    /// hover, emphasis and tone, layered as a role delta (the `CellUi::tone`
    /// shape, never a colour).
    fn item_style(&self, ui: &mut Ui<'_>, it: &StatusItem<'_>, live: StateFlags) -> Style {
        let hovered = it.key.is_some_and(|key| {
            FrameRead::hovered_part(ui, self.id) == Some(PartRef::item(Part::LABEL, key))
        });
        let live = if self.ov.is_forced() || hovered {
            live
        } else {
            live.difference(StateFlags::HOVERED)
        };
        let base = self.ov.style(
            ui,
            self.id,
            Family::STATUSBAR,
            self.variant,
            Part::LABEL,
            live,
        );
        let mut delta = StylePatch::new();
        match it.emphasis {
            // caller-declared emphasis, the `CellUi::italic` shape: the
            // component invents no colour, only the weight the caller asked
            // for
            Emphasis::Strong => delta = delta.add(Modifier::BOLD),
            Emphasis::Chip => delta = delta.set_bg(Role::RaisedSurface),
            Emphasis::Plain => {}
        }
        if let Some(r) = it.tone {
            delta = delta.set_fg(r);
        }
        if delta.is_empty() {
            return base.style;
        }
        let top = crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style;
        base.style.patch(top)
    }

    /// Paint one item into `cell`, truncating with the overflow marker when
    /// it does not fit. Returns the columns used.
    fn paint_item(
        &self,
        ui: &mut Ui<'_>,
        it: &StatusItem<'_>,
        cell: Rect,
        live: StateFlags,
        mw: u16,
    ) -> u16 {
        if cell.is_empty() {
            return 0;
        }
        let label_w = it.label_columns().min(cell.width);
        let label = Rect {
            width: label_w,
            ..cell
        };
        if let Some(f) = self.ov.slot_for(Part::LABEL) {
            // substitution, not suppression: the item keeps its columns, its
            // hit registration and its inline meter (§45.3, Invariant R)
            f(ui, label);
        } else {
            let style = self.item_style(ui, it, live);
            if it.emphasis == Emphasis::Chip {
                ui.fill(label, style);
                ui.paint_str(shift(label, 1), it.text, style);
            } else {
                ui.paint_str(label, it.text, style);
            }
        }
        if it.label_columns() > cell.width && cell.width > 0 {
            // R-2: paint clipped, then mark the cut — never pre-truncate
            let last = Rect {
                x: cell.right().saturating_sub(1),
                width: 1,
                ..cell
            };
            if let Some(f) = self.ov.slot_for(Part::OVERFLOW) {
                f(ui, last);
            } else {
                let s = self.ov.style(
                    ui,
                    self.id,
                    Family::STATUSBAR,
                    self.variant,
                    Part::OVERFLOW,
                    live,
                );
                match s.glyph {
                    Slot::Set(g) => {
                        ui.glyph(last, g, s.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(last, GlyphRole::Ellipsis, s.style);
                    }
                    Slot::Clear => {
                        ui.fill(last, s.style);
                    }
                }
            }
        }
        if let Some(k) = it.key
            && !self.ov.is_forced()
        {
            ui.register_part(self.id, PartRef::item(Part::LABEL, k), label);
        }
        let mut used = label_w;
        if it.ratio.is_some() {
            let m = Rect {
                x: cell.x.saturating_add(label_w).saturating_add(1),
                width: cell.width.saturating_sub(label_w).saturating_sub(1).min(mw),
                ..cell
            };
            if !m.is_empty() {
                // the inline meter paints `TRACK` under the strip's own `Id`,
                // so the strip's `.patch`, `.patch_part` and `TRACK` slot are
                // the strip's to forward; a bare `Meter::new` dropped all
                // three (§45.1, §45.7 obligation 2)
                let mut meter = Meter::new(self.id).value("").patch_part(self.parts);
                if let Some(p) = self.patch {
                    meter = meter.patch(p);
                }
                if let Some(f) = self.ov.slot_for(Part::TRACK) {
                    meter = meter.slot(Part::TRACK, f);
                }
                if let Some(r) = it.ratio {
                    meter = meter.ratio(r);
                }
                if let Some(t) = it.meter_tone {
                    meter = meter.tone(t);
                }
                meter.draw(ui, m);
                used = used.saturating_add(1).saturating_add(m.width);
            }
        }
        used
    }

    /// The draw phase; returns the rect painted.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the readiness affordance and the three groups"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        let live = self.ov.flags(ui.state(self.id), self.status.flags());
        let ov = self.ov;
        let id = self.id;
        let d = ui.design();
        let gap = d.space.gap.max(1);
        let edge = d.space.gutter.max(1);
        let mw = Self::meter_columns(ui);
        let container = ov.style(
            ui,
            id,
            Family::STATUSBAR,
            self.variant,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);

        // the readiness affordance leads the strip
        let ready = self.readiness(ui, live);
        let lead = ready.map_or(0, |g| width(g).saturating_add(1));
        if let Some(g) = ready {
            // one cell, two parts: the spinner is `ICON` and the error or
            // warning marker is `MARKER`. The slot is consulted before
            // `spinner_frames` (§45.4) — a slot is substitution, not
            // suppression, so `lead` reserves the same columns either way.
            let part = if self.busy() {
                Part::ICON
            } else {
                Part::MARKER
            };
            let cell = Rect {
                x: area.x.saturating_add(edge),
                width: area.width.saturating_sub(edge),
                ..area
            };
            if let Some(f) = ov.slot_for(part) {
                f(ui, cell);
            } else {
                let s = ov.style(ui, id, Family::STATUSBAR, self.variant, part, live);
                ui.paint_str(cell, g, s.style);
            }
        }

        let keep = self.survivors(area.width, mw, gap, edge, lead);
        let inner_left = area.x.saturating_add(edge).saturating_add(lead);
        let inner_right = area.right().saturating_sub(edge);

        // left, from the leading edge
        let mut x = inner_left;
        let left_mask = keep.first().copied().unwrap_or(0);
        for (i, it) in self.group(Group::Left).iter().enumerate() {
            if left_mask & (1 << i) == 0 {
                continue;
            }
            let room = inner_right.saturating_sub(x);
            if room == 0 {
                break;
            }
            let cell = Rect {
                x,
                width: it.columns(mw).min(room),
                ..area
            };
            let used = self.paint_item(ui, it, cell, live, mw);
            x = x.saturating_add(used).saturating_add(gap);
        }
        let left_end = x.saturating_sub(gap);

        // right, from the trailing edge backwards
        let right_mask = keep.get(2).copied().unwrap_or(0);
        let mut rx = inner_right;
        let right_items = self.group(Group::Right);
        for (i, it) in right_items.iter().enumerate().rev() {
            if right_mask & (1 << i) == 0 {
                continue;
            }
            let w = it.columns(mw);
            if rx.saturating_sub(w) <= left_end {
                break;
            }
            rx = rx.saturating_sub(w);
            let cell = Rect {
                x: rx,
                width: w,
                ..area
            };
            self.paint_item(ui, it, cell, live, mw);
            rx = rx.saturating_sub(gap);
        }
        let right_start = if right_mask == 0 {
            inner_right
        } else {
            rx.saturating_add(gap)
        };

        // centre, in the free span between the two
        let cw = self.group_columns(Group::Center, keep, mw, gap);
        if cw > 0 {
            let lo = left_end.saturating_add(gap);
            let hi = right_start.saturating_sub(gap);
            let free = hi.saturating_sub(lo);
            let mut cx = lo.saturating_add(free.saturating_sub(cw) / 2);
            let center_mask = keep.get(1).copied().unwrap_or(0);
            for (i, it) in self.group(Group::Center).iter().enumerate() {
                if center_mask & (1 << i) == 0 {
                    continue;
                }
                let w = it.columns(mw);
                let cell = Rect {
                    x: cx,
                    width: hi.saturating_sub(cx).min(w),
                    ..area
                };
                self.paint_item(ui, it, cell, live, mw);
                cx = cx.saturating_add(w).saturating_add(gap);
            }
        }
        area
    }

    /// The natural size: one row wide enough for every item.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let mw = Self::meter_columns(ui);
        let gap = ui.design().space.gap.max(1);
        let edge = ui.design().space.gutter.max(1);
        let full = self.all_alive_keep();
        let preferred = self.needed(full, mw, gap, edge, 0);
        let strongest = self
            .group(Group::Left)
            .iter()
            .map(|it| it.columns(mw))
            .next()
            .unwrap_or(0);
        Size {
            min: (strongest.saturating_add(edge.saturating_mul(2)), 1),
            preferred: (preferred, 1),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::Theme;

    const LEFT: [StatusItem<'static>; 2] = [
        StatusItem::new("payments-platform").strong().priority(9),
        StatusItem::new("PR #482 settlement backoff").priority(7),
    ];
    const CENTER: [StatusItem<'static>; 1] = [StatusItem::new("agent working").priority(4)];
    const RIGHT: [StatusItem<'static>; 3] = [
        StatusItem::new("Weekly 59%").chip().priority(6),
        StatusItem::new("run-7f3a").chip().priority(3),
        StatusItem::new("run 9c41").priority(2),
    ];

    fn bar() -> StatusBar<'static> {
        StatusBar::new(Id::root("t"))
            .left(&LEFT)
            .center(&CENTER)
            .right(&RIGHT)
    }

    #[test]
    fn a_wide_row_keeps_every_item() {
        let keep = bar().survivors(200, 16, 2, 1, 0);
        assert_eq!(keep, [0b11, 0b1, 0b111]);
    }

    #[test]
    fn narrow_rows_drop_centre_then_right_then_left_and_keep_the_name() {
        // the centre is the first to leave
        let keep = bar().survivors(60, 16, 2, 1, 0);
        assert_eq!(keep.get(1).copied(), Some(0), "the centre leaves first");
        assert_ne!(keep.first().copied(), Some(0), "identity stays");
        // the strongest left item never leaves
        let keep = bar().survivors(10, 16, 2, 1, 0);
        assert_eq!(keep.get(1).copied(), Some(0));
        assert_eq!(keep.get(2).copied(), Some(0));
        assert_eq!(
            keep.first().copied().map(u32::count_ones),
            Some(1),
            "exactly the strongest left item survives"
        );
        assert_eq!(keep.first().copied(), Some(0b1));
    }

    #[test]
    fn ties_take_the_later_item_first() {
        const TIED: [StatusItem<'static>; 3] = [
            StatusItem::new("aaaaaaaa").priority(5),
            StatusItem::new("bbbbbbbb").priority(5),
            StatusItem::new("cccccccc").priority(5),
        ];
        let bar = StatusBar::new(Id::root("t")).right(&TIED);
        let keep = bar.survivors(22, 16, 2, 1, 0);
        assert_eq!(keep.get(2).copied(), Some(0b011), "the last item leaves");
    }

    #[test]
    fn items_past_the_cap_are_ignored() {
        const MANY: [StatusItem<'static>; 20] = [StatusItem::new("x"); 20];
        let bar = StatusBar::new(Id::root("t")).left(&MANY);
        assert_eq!(bar.group(Group::Left).len(), MAX_ITEMS);
    }

    #[test]
    fn an_item_reports_its_own_columns() {
        let plain = StatusItem::new("abc");
        assert_eq!(plain.columns(16), 3);
        assert_eq!(StatusItem::new("abc").chip().columns(16), 5);
        assert_eq!(StatusItem::new("abc").meter(0.5).columns(16), 3 + 1 + 16);
    }

    /// The painted row of `buf`, one `char` per column.
    ///
    /// Every fixture label below is ASCII, so a byte offset into this string
    /// is the column the label starts at.
    fn painted_row(buf: &Buffer, w: u16) -> String {
        let mut row = String::new();
        for x in 0..w {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                row.push_str(cell.symbol());
            }
        }
        row
    }

    /// GAP-8: where the three groups actually land, read back out of the
    /// painted buffer.
    ///
    /// Successor to the legacy
    /// `widgets::statusbar::tests::groups_keep_their_order_and_sides`. The
    /// replacement `survivors` returns keep-masks and never geometry, so a
    /// strip that right-aligned its left group would keep every item and pass
    /// every other test in this module. Asserting this against `survivors`,
    /// or against any other accessor of the strip's own arithmetic, would
    /// reproduce that defect one layer down; the buffer is the only witness
    /// that cannot agree with a wrong implementation.
    #[test]
    fn the_left_group_starts_at_the_gutter_the_right_is_flush_and_the_centre_sits_between() {
        const ROW: Rect = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 1,
        };
        const L: [StatusItem<'static>; 2] = [
            StatusItem::new("alpha").priority(9),
            StatusItem::new("bravo").priority(8),
        ];
        const C: [StatusItem<'static>; 1] = [StatusItem::new("charlie").priority(7)];
        const R: [StatusItem<'static>; 2] = [
            StatusItem::new("delta").priority(6),
            StatusItem::new("echo").priority(5),
        ];

        let theme = Theme::junie();
        let edge = theme.design.space.gutter.max(1);
        let bar = StatusBar::new(Id::root("status.geometry"))
            .left(&L)
            .center(&C)
            .right(&R);
        let mut rt = Runtime::new(Stub::default(), theme);
        let mut buf = Buffer::empty(ROW);
        rt.draw_scene(ROW, &mut buf, |ui, area| {
            bar.draw(ui, area);
        });
        let row = painted_row(&buf, ROW.width);

        let at = |needle: &str| -> u16 {
            let found = row.find(needle);
            assert!(found.is_some(), "{needle:?} is not painted in {row:?}");
            found
                .and_then(|i| u16::try_from(i).ok())
                .unwrap_or_default()
        };
        let end = |needle: &str| -> u16 { at(needle).saturating_add(width(needle)) };

        // `at` panics on a label the strip did not paint, so reaching the
        // assertions below is itself the statement that this row is wide
        // enough for all five items and nothing dropped

        // left: the leading group starts one gutter in from the left edge
        assert_eq!(
            at("alpha"),
            ROW.x.saturating_add(edge),
            "the left group starts at the gutter, in {row:?}"
        );
        assert!(
            end("alpha") < at("bravo"),
            "left items run left-to-right in declaration order, in {row:?}"
        );

        // right: the trailing group ends flush one gutter in from the right
        assert_eq!(
            end("echo"),
            ROW.right().saturating_sub(edge),
            "the right group is flush against the trailing gutter, in {row:?}"
        );
        assert!(
            end("delta") < at("echo"),
            "right items run left-to-right in declaration order, in {row:?}"
        );

        // centre: strictly between the two, and centred in what they leave
        assert!(
            end("bravo") < at("charlie") && end("charlie") < at("delta"),
            "the centre group sits strictly between left and right, in {row:?}"
        );
        let before = at("charlie").saturating_sub(end("bravo"));
        let after = at("delta").saturating_sub(end("charlie"));
        assert!(
            before.abs_diff(after) <= 1,
            "the centre group is centred in the free span: {before} before, {after} after, in {row:?}"
        );
    }
}
