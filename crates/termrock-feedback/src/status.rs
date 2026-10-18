#![doc = r" `StatusBar` — the merged, priority-ordered item strip"]
#![doc = r" (`COMPONENT_ARCHITECTURE.md` §14.1, §18.2, §18.3 items 9 and 11,"]
#![doc = r" Appendix A 4G)."]
#![doc = r""]
#![doc = r" This one component replaces both the legacy `statusbar` and `segments`:"]
#![doc = r" `Left`/`Center`/`Right` groups of priority-ordered items, one drop order,"]
#![doc = r" one truncation rule, inline meters and clickable item ids. The two"]
#![doc = r" hand-written priority-drop loops in `TablePro`'s identity strip and grid"]
#![doc = r" status line become consumers of this strip in Slice 6."]

use core::fmt;

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;
use ratatui_core::style::Modifier;

use super::{PartStyle, SlotFn, first_row, shift};
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::meter::{Meter, MeterTone};
use crate::progress::{PCT_COLUMNS, spinner_glyph};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_runtime::Status;

#[doc = r" Items per group laid out without allocating."]
#[doc = r""]
#[doc = r" Items beyond this cap are **silently ignored**, exactly as"]
#[doc = r" `RowUi::columns` ignores tracks past `MAX_COLUMNS`: the fixed per-group"]
#[doc = r" buffers are what keep the strip's priority-drop loop allocation-free"]
#[doc = r" (§12.2, R5)."]
pub const MAX_ITEMS: usize = 16;

#[doc = r" The three groups of a status strip."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    #[doc = r" Identity and context; the leading group."]
    Left,
    #[doc = r" Activity; centred in whatever the other two leave."]
    Center,
    #[doc = r" Quotas and runtime facts; the trailing group."]
    Right,
}

impl Group {
    #[doc = r" Every group, in painting order."]
    pub const ALL: [Group; 3] = [Group::Left, Group::Center, Group::Right];

    const fn index(self) -> usize {
        match self {
            Group::Left => 0,
            Group::Center => 1,
            Group::Right => 2,
        }
    }
}

#[doc = r" How much weight an item carries."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum Emphasis {
    #[doc = r" The default reading."]
    #[default]
    Plain,
    #[doc = r" The item that names the surface; bold."]
    Strong,
    #[doc = r" A fact with its own edge — a quota, a runtime id — drawn on the plane"]
    #[doc = r" one step above the strip."]
    Chip,
}

#[doc = r" One fact in a status strip."]
#[doc = r""]
#[doc = r" Built with `StatusItem::new(text)` plus consuming builders, like every"]
#[doc = r" other props type (§13). The strip borrows a `&[StatusItem]` per group and"]
#[doc = r" never holds one."]
#[derive(Clone, Copy, Debug)]
pub struct StatusItem<'a> {
    text: &'a str,
    tone: Option<Role>,
    priority: u8,
    key: Option<ItemKey>,
    emphasis: Emphasis,
    ratio: Option<f64>,
    meter_tone: Option<MeterTone>,
    spinner: Option<usize>,
}

impl<'a> StatusItem<'a> {
    #[doc = r" The default priority; higher survives width pressure longer."]
    pub const DEFAULT_PRIORITY: u8 = 5;

    #[doc = r" An item reading `text`."]
    pub const fn new(text: &'a str) -> Self {
        StatusItem {
            text,
            tone: None,
            priority: Self::DEFAULT_PRIORITY,
            key: None,
            emphasis: Emphasis::Plain,
            ratio: None,
            meter_tone: None,
            spinner: None,
        }
    }

    #[doc = r" Prefix the label with a canonical spinner frame and one separating cell."]
    #[doc = r""]
    #[doc = r" The theme's `motion.spinner_frames` supplies the sequence. Empty"]
    #[doc = r" sequences or empty glyphs add no prefix or gap. The frame is caller-owned;"]
    #[doc = r" measurement, priority dropping and clickable geometry include its width."]
    #[must_use]
    pub const fn spinner(mut self, frame: usize) -> Self {
        self.spinner = Some(frame);
        self
    }

    #[doc = r" The colour role the item reads in; the recipe's `LABEL` tone when"]
    #[doc = r" unset."]
    #[must_use]
    pub const fn tone(mut self, r: Role) -> Self {
        self.tone = Some(r);
        self
    }

    #[doc = r" Higher survives longer when the row is narrow."]
    #[must_use]
    pub const fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    #[doc = r" Make the item clickable and addressable; the strip emits"]
    #[doc = r" [`StatusAction::Chose`] with this key."]
    #[must_use]
    pub const fn key(mut self, k: ItemKey) -> Self {
        self.key = Some(k);
        self
    }

    #[doc = r" Bold: the item that names the surface."]
    #[must_use]
    pub const fn strong(mut self) -> Self {
        self.emphasis = Emphasis::Strong;
        self
    }

    #[doc = r" Draw the item as a chip on the raised plane."]
    #[must_use]
    pub const fn chip(mut self) -> Self {
        self.emphasis = Emphasis::Chip;
        self
    }

    #[doc = r" Append a compact inline meter reporting `ratio`, clamped to"]
    #[doc = r" `0.0..=1.0`. Its tone is derived from `design.meter` unless"]
    #[doc = r" [`StatusItem::meter_tone`] forces one (J12)."]
    #[must_use]
    pub fn meter(mut self, ratio: f64) -> Self {
        self.ratio = Some(ratio.clamp(0.0, 1.0));
        self
    }

    #[doc = r" Force the inline meter's tone."]
    #[must_use]
    pub const fn meter_tone(mut self, t: MeterTone) -> Self {
        self.meter_tone = Some(t);
        self
    }

    #[doc = r" The text."]
    pub const fn text(&self) -> &'a str {
        self.text
    }

    #[doc = r" The priority."]
    pub const fn priority_of(&self) -> u8 {
        self.priority
    }

    #[doc = r" The key, when the item is clickable."]
    pub const fn key_of(&self) -> Option<ItemKey> {
        self.key
    }

    #[doc = r" The emphasis."]
    pub const fn emphasis(&self) -> Emphasis {
        self.emphasis
    }

    fn spinner_glyph(&self, metrics: ItemMetrics) -> &'static str {
        self.spinner
            .and_then(|frame| spinner_glyph(metrics.spinner_frames, frame))
            .unwrap_or("")
    }

    fn spinner_columns(&self, metrics: ItemMetrics) -> u16 {
        let glyph = self.spinner_glyph(metrics);
        if glyph.is_empty() {
            0
        } else {
            width(glyph).saturating_add(u16::from(!self.text.is_empty()))
        }
    }

    #[doc = r" Columns the label alone occupies (chips carry their own padding)."]
    fn label_columns(&self, metrics: ItemMetrics) -> u16 {
        let w = width(self.text).saturating_add(self.spinner_columns(metrics));
        match self.emphasis {
            Emphasis::Chip => w.saturating_add(2),
            Emphasis::Plain | Emphasis::Strong => w,
        }
    }

    #[doc = r" Columns the whole item occupies, inline meter included."]
    fn columns(&self, metrics: ItemMetrics) -> u16 {
        match self.ratio {
            Some(_) => self
                .label_columns(metrics)
                .saturating_add(1)
                .saturating_add(metrics.meter_columns),
            None => self.label_columns(metrics),
        }
    }
}

#[derive(Clone, Copy)]
struct ItemMetrics {
    meter_columns: u16,
    spinner_frames: &'static [&'static str],
}

#[doc = r" What a status strip reports."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum StatusAction {
    #[doc = r" A clickable item was clicked; carries the item's own key."]
    Chose(ItemKey),
}

#[doc = r" A full-width strip of priority-ordered facts in three groups."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `StatusBar::new(id)`, then `.left(items)` / `.center(items)` /"]
#[doc = r" `.right(items)` with a borrowed slice per group. Items are passed per"]
#[doc = r" phase like every other collection's data (§12.2), so the props never"]
#[doc = r" borrow the field an action closure mutates."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless — there is no `StatusBarState`. The caller owns the item"]
#[doc = r" slices and the animation frame; the runtime owns hover and press, including"]
#[doc = r" the hovered keyed item."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.variant(Variant)` (default `Recipe.default_variant`), `.left/.center/"]
#[doc = r" .right(&[StatusItem])` (empty), `.status(Status)` (`Ready`),"]
#[doc = r" `.frame(usize)` (`0`), `.patch`, `.patch_part`, `.slot`,"]
#[doc = r" runtime state."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::STATUSBAR`; `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)` and paints a"]
#[doc = r" leading readiness affordance for them; wears `HOVERED` and `PRESSED` from"]
#[doc = r" the runtime. A matching keyed item alone wears `HOVERED`; per-item tone and"]
#[doc = r" emphasis remain the item's own configuration."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" `StatusAction::Chose(ItemKey)` — a click on an item that declared a"]
#[doc = r" `.key(…)`. Items without a key register nothing and cannot be clicked."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop: the strip registers `Part` regions for its clickable"]
#[doc = r" items and no ring entry, because a status strip is a report, not a"]
#[doc = r" control an operator tabs through. `swallows_typing` is false."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None. A product that wants a chord for one of these facts binds it"]
#[doc = r" through its own `KeyMap` and maps it onto the same action."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::item(Part::LABEL, key)` per clickable item: a click emits"]
#[doc = r" `StatusAction::Chose(key)`."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" `measure` returns `(every item at its natural width, 1)`. `draw` uses the"]
#[doc = r" first row of `area`; when the row is too narrow items leave by ascending"]
#[doc = r" priority — **centre first, then right, then left** — and the strongest"]
#[doc = r" left item never leaves, truncating with `GlyphRole::Ellipsis` instead."]
#[doc = r" Returns the rect it painted; a degenerate rect registers nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER` (the row fill), `LABEL` (each item), `MARKER` (the readiness"]
#[doc = r" glyph), `ICON` (the readiness spinner), plus `TRACK`, `THUMB` and"]
#[doc = r" `OVERFLOW` for an item's inline meter and the truncation marker."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` on any part, forwarded to an item's inline"]
#[doc = r" [`Meter`]. `.slot` on exactly `LABEL`, `MARKER`, `ICON`, `TRACK` and"]
#[doc = r" `OVERFLOW`. `LABEL` answers for every item the strip paints and"]
#[doc = r" `OVERFLOW` for every cut it marks; `TRACK` is forwarded to the inline"]
#[doc = r" meter. `CONTAINER` is not slot-addressable, because its fill *is* the"]
#[doc = r" strip, and neither is `THUMB`: a slot on `TRACK` replaces the whole"]
#[doc = r" inline run, used share included, because the split between the two is the"]
#[doc = r" meter's own arithmetic."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" Items are keyed by the caller through `StatusItem::key`; there is no"]
#[doc = r" `ByIndex` default, because an item without a key is not addressable at"]
#[doc = r" all — it is a label, and no action can name it."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `StatusBarCase` in `crates/tui/tests/conformance.rs`, declaring"]
#[doc = r" `Caps::REPORTS_STATUS`, must retain `BUSY` and `ERROR` in its mono states."]
#[doc = r" `mono_states_are_distinguishable` covers `StateFlags::empty()`, `PRESSED`,"]
#[doc = r" `ERROR`, `WARNING` and `BUSY`."]
#[doc = r" The default fixture rect is 30 columns and the fixture strip"]
#[doc = r" needs 35, so the drop loop does run under the cases that remain — but"]
#[doc = r" they assert byte-identity, containment, theme isolation and tiny-rect"]
#[doc = r" survival, never which items survived."]
#[doc = r""]
#[doc = r" The render matrix in `crates/tui/tests/render_components.rs` generates"]
#[doc = r" exactly eight cells per component, one per `St` variant: there is no"]
#[doc = r" `render::components::status_bar::busy`, no `::error` and no"]
#[doc = r" `::overflow`. Readiness arrives through the matrix's `status_for`"]
#[doc = r" mapping — `::pressed` is `Status::Busy`, `::editing` `Status::Loading`,"]
#[doc = r" `::disabled` `Status::Error` — and `draw` **ors** the status-derived"]
#[doc = r" flags into runtime state, so the `MARKER`"]
#[doc = r" error glyph as well as the `ICON` spinner is genuinely painted, and"]
#[doc = r" pinned as a digest, by those three cells. At the matrix's two widths the"]
#[doc = r" strip needs 35 columns (37 with a readiness affordance) and is given 40"]
#[doc = r" or 120, so no matrix cell drops an item."]
#[doc = r""]
#[doc = r" The drop order is covered by the unit tests in this module, which call"]
#[doc = r" `survivors` directly: `a_wide_row_keeps_every_item`,"]
#[doc = r" `narrow_rows_drop_centre_then_right_then_left_and_keep_the_name`,"]
#[doc = r" `ties_take_the_later_item_first` and `items_past_the_cap_are_ignored`."]
#[doc = r" Those masks say which items survive and nothing about where they land, so"]
#[doc = r" the placement itself is covered by"]
#[doc = r" `the_left_group_starts_at_the_gutter_the_right_is_flush_and_the_centre_sits_between`,"]
#[doc = r" which reads the three groups' columns back out of the painted buffer"]
#[doc = r" rather than out of any of the helpers above."]
#[doc = r""]
#[doc = r" `a_truncated_item_paints_the_overflow_glyph` covers the painted"]
#[doc = r" `Part::OVERFLOW` marker, and `a_keyed_item_click_reports_chose` covers"]
#[doc = r" [`StatusAction::Chose`] through the runtime's hit and intent path."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" The drop order is exactly centre → right → left with the strongest left"]
#[doc = r" item retained, so a narrowing terminal loses activity before context and"]
#[doc = r" context before identity. At most [`MAX_ITEMS`] items per group are laid"]
#[doc = r" out. Never allocates. Only the keyed item matching the frame's hovered"]
#[doc = r" `PartRef` receives `HOVERED`; keyboard suppression makes that lookup return"]
#[doc = r" `None`."]
pub struct StatusBar<'a> {
    id: Id,
    left: &'a [StatusItem<'a>],
    center: &'a [StatusItem<'a>],
    right: &'a [StatusItem<'a>],
    variant: Variant,
    status: Status,
    frame: usize,
    #[doc = r" Kept beside `ov` so an item's inline [`Meter`] can be built with the"]
    #[doc = r" caller's own overrides: it paints `TRACK` and `THUMB` under *this*"]
    #[doc = r" strip's `Id`, so a bare construction dropped the caller's `.patch`"]
    #[doc = r" and `.patch_part` on those parts where `Invariant P` could not see it"]
    #[doc = r" (§45.7 obligation 2)."]
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
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

#[doc = r" Which items survive, one bit per item, one mask per group."]
type Keep = [u32; 3];

#[doc = r" The mask with the low `n` bits set: every item of an `n`-item group alive."]
#[doc = r""]
#[doc = r" A group with more items than the mask has bits keeps every bit set, which"]
#[doc = r" is what the drop loop wants — the strip is full and items start leaving."]
fn all_alive(n: usize) -> u32 {
    let n = u32::try_from(n).unwrap_or(u32::BITS);

    u32::MAX
        .checked_shr(u32::BITS.saturating_sub(n))
        .unwrap_or(0)
}

impl<'a> StatusBar<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::LABEL,
        Part::MARKER,
        Part::ICON,
        Part::TRACK,
        Part::THUMB,
        Part::OVERFLOW,
    ];

    #[doc = r" An empty strip."]
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
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" The leading group: identity and context."]
    #[must_use]
    pub const fn left(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.left = items;
        self
    }

    #[doc = r" The centre group: activity."]
    #[must_use]
    pub const fn center(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.center = items;
        self
    }

    #[doc = r" The trailing group: quotas and runtime facts."]
    #[must_use]
    pub const fn right(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.right = items;
        self
    }

    #[doc = r" Set the variant."]
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[doc = r" Data readiness of the surface the strip reports on."]
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[doc = r" The animation frame the readiness spinner reads."]
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[doc = r" An instance patch over every part (precedence 6)."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting; layout and hit regions stay."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The items of one group, capped at [`MAX_ITEMS`]."]
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

    #[doc = r" Columns an inline meter occupies."]
    fn meter_columns(ui: &Ui<'_>) -> u16 {
        ui.design()
            .size
            .meter_track
            .saturating_add(PCT_COLUMNS)
            .saturating_add(2)
    }

    #[doc = r" The glyph slot `Part::MARKER` resolves under for **this instance**."]
    #[doc = r""]
    #[doc = r" [`Ui::resolve`] is §26 N2's `&self` measuring path: it stops at"]
    #[doc = r" precedence 5, so an instance `.patch` or `.patch_part` reached this"]
    #[doc = r" cell's *colour* — resolved through [`PartStyle::style`] — and could"]
    #[doc = r" not reach its *glyph* (§45.5). Precedence 6 is applied here exactly"]
    #[doc = r" as `theme::resolve::bind` applies it on the painting path, so the two"]
    #[doc = r" halves of one cell cannot answer to different override chains."]
    fn marker_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Slot<GlyphRole> {
        let base = ui
            .resolve(Family::STATUSBAR, self.variant, Part::MARKER, live)
            .glyph;
        self.ov
            .part_patch(Part::MARKER)
            .map_or(base, |p| p.glyph.over(base))
    }

    #[doc = r" The readiness affordance: the spinner while busy, the recipe's marker"]
    #[doc = r" (or the error glyph) while in error."]
    fn readiness(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if self.busy() {
            return spinner_glyph(ui.design().motion.spinner_frames, self.frame);
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

    #[doc = r" Columns a group occupies under `keep`."]
    fn group_columns(&self, g: Group, keep: Keep, metrics: ItemMetrics, gap: u16) -> u16 {
        let items = self.group(g);
        let mask = keep.get(g.index()).copied().unwrap_or(0);
        let mut w = 0u16;
        let mut n = 0u16;
        for (i, it) in items.iter().enumerate() {
            if mask & (1 << i) != 0 {
                w = w.saturating_add(it.columns(metrics));
                n = n.saturating_add(1);
            }
        }
        if n > 0 {
            w.saturating_add(n.saturating_sub(1).saturating_mul(gap))
        } else {
            0
        }
    }

    #[doc = r" The keep mask with every item of every group alive."]
    fn all_alive_keep(&self) -> Keep {
        Group::ALL.map(|g| all_alive(self.group(g).len()))
    }

    #[doc = r" Columns the whole strip needs under `keep`."]
    fn needed(&self, keep: Keep, metrics: ItemMetrics, gap: u16, edge: u16, lead: u16) -> u16 {
        let ws = Group::ALL.map(|g| self.group_columns(g, keep, metrics, gap));
        let present = ws.iter().filter(|w| **w > 0).count().min(3) as u16;
        ws.iter()
            .fold(0u16, |a, w| a.saturating_add(*w))
            .saturating_add(present.saturating_sub(1).saturating_mul(gap))
            .saturating_add(edge.saturating_mul(2))
            .saturating_add(lead)
    }

    #[doc = r" Which items survive at `total` columns."]
    #[doc = r""]
    #[doc = r" Items leave by ascending priority; on a tie the centre group loses"]
    #[doc = r" before the right and the right before the left, and the strongest"]
    #[doc = r" left item never leaves — it truncates instead. This is the legacy"]
    #[doc = r" strip's order, kept verbatim so the two hand-written copies §18.3"]
    #[doc = r" items 9 and 11 delete can be replaced without a visual review of"]
    #[doc = r" every width."]
    fn survivors(&self, total: u16, metrics: ItemMetrics, gap: u16, edge: u16, lead: u16) -> Keep {
        let mut keep: Keep = self.all_alive_keep();
        while self.needed(keep, metrics, gap, edge, lead) > total {
            let mut victim: Option<(u8, usize, usize)> = None;
            for g in [Group::Center, Group::Right, Group::Left] {
                let gi = g.index();
                let items = self.group(g);
                let mask = keep.get(gi).copied().unwrap_or(0);
                let alive = mask.count_ones();
                if alive == 0 || (g == Group::Left && alive == 1) {
                    continue;
                }

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

    #[doc = r" The update phase: a click on a keyed item."]
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

                Intent::Pointer {
                    phase: Phase::Move, ..
                } => {}
                Intent::Pointer { .. } if r.action_ref().is_none() => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    #[doc = r" The style an item paints with: the `LABEL` recipe plus the item's own"]
    #[doc = r" hover, emphasis and tone, layered as a role delta (the `CellUi::tone`"]
    #[doc = r" shape, never a colour)."]
    fn item_style(&self, ui: &mut Ui<'_>, it: &StatusItem<'_>, live: StateFlags) -> PaintStyle {
        let hovered = it.key.is_some_and(|key| {
            FrameRead::hovered_part(ui, self.id) == Some(PartRef::item(Part::LABEL, key))
        });
        let live = if hovered {
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

    #[doc = r" Paint one item into `cell`, truncating with the overflow marker when"]
    #[doc = r" it does not fit. Returns the columns used."]
    fn paint_item(
        &self,
        ui: &mut Ui<'_>,
        it: &StatusItem<'_>,
        cell: Rect,
        live: StateFlags,
        metrics: ItemMetrics,
    ) -> u16 {
        if cell.is_empty() {
            return 0;
        }
        let label_w = it.label_columns(metrics).min(cell.width);
        let label = Rect {
            width: label_w,
            ..cell
        };
        if let Some(f) = self.ov.slot_for(Part::LABEL) {
            f(ui, label);
        } else {
            let style = self.item_style(ui, it, live);
            let text = if it.emphasis == Emphasis::Chip {
                ui.fill(label, style);
                shift(label, 1)
            } else {
                label
            };
            let glyph = it.spinner_glyph(metrics);
            ui.paint_str(text, glyph, style);
            ui.paint_str(shift(text, it.spinner_columns(metrics)), it.text, style);
        }
        if it.label_columns(metrics) > cell.width && cell.width > 0 {
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
            && !ui.is_inert()
        {
            ui.register_part(self.id, PartRef::item(Part::LABEL, k), label);
        }
        let mut used = label_w;
        if it.ratio.is_some() {
            let m = Rect {
                x: cell.x.saturating_add(label_w).saturating_add(1),
                width: cell
                    .width
                    .saturating_sub(label_w)
                    .saturating_sub(1)
                    .min(metrics.meter_columns),
                ..cell
            };
            if !m.is_empty() {
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

    #[doc = r" The draw phase; returns the rect painted."]
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the readiness affordance and the three groups"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        let live = PartStyle::flags(ui.state(self.id), self.status.flags());
        let ov = self.ov;
        let id = self.id;
        let d = ui.design();
        let gap = d.space.gap.max(1);
        let edge = d.space.gutter.max(1);
        let metrics = ItemMetrics {
            meter_columns: Self::meter_columns(ui),
            spinner_frames: ui.design().motion.spinner_frames,
        };
        let container = ov.style(
            ui,
            id,
            Family::STATUSBAR,
            self.variant,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);

        let ready = self.readiness(ui, live);
        let lead = ready.map_or(0, |g| width(g).saturating_add(1));
        if let Some(g) = ready {
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

        let keep = self.survivors(area.width, metrics, gap, edge, lead);
        let inner_left = area.x.saturating_add(edge).saturating_add(lead);
        let inner_right = area.right().saturating_sub(edge);

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
                width: it.columns(metrics).min(room),
                ..area
            };
            let used = self.paint_item(ui, it, cell, live, metrics);
            x = x.saturating_add(used).saturating_add(gap);
        }
        let left_end = x.saturating_sub(gap);

        let right_mask = keep.get(2).copied().unwrap_or(0);
        let mut rx = inner_right;
        let right_items = self.group(Group::Right);
        for (i, it) in right_items.iter().enumerate().rev() {
            if right_mask & (1 << i) == 0 {
                continue;
            }
            let w = it.columns(metrics);
            if rx.saturating_sub(w) <= left_end {
                break;
            }
            rx = rx.saturating_sub(w);
            let cell = Rect {
                x: rx,
                width: w,
                ..area
            };
            self.paint_item(ui, it, cell, live, metrics);
            rx = rx.saturating_sub(gap);
        }
        let right_start = if right_mask == 0 {
            inner_right
        } else {
            rx.saturating_add(gap)
        };

        let cw = self.group_columns(Group::Center, keep, metrics, gap);
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
                let w = it.columns(metrics);
                let cell = Rect {
                    x: cx,
                    width: hi.saturating_sub(cx).min(w),
                    ..area
                };
                self.paint_item(ui, it, cell, live, metrics);
                cx = cx.saturating_add(w).saturating_add(gap);
            }
        }
        area
    }

    #[doc = r" The natural size: one row wide enough for every item."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let metrics = ItemMetrics {
            meter_columns: Self::meter_columns(ui),
            spinner_frames: ui.design().motion.spinner_frames,
        };
        let gap = ui.design().space.gap.max(1);
        let edge = ui.design().space.gutter.max(1);
        let full = self.all_alive_keep();
        let preferred = self.needed(full, metrics, gap, edge, 0);
        let strongest = self
            .group(Group::Left)
            .iter()
            .map(|it| it.columns(metrics))
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
    use crate::event::MouseKind;
    use crate::response::Invalidate;
    use crate::runtime::stub::{Stub, mouse};
    use crate::runtime::{App, Runtime};
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

    const TEST_METRICS: ItemMetrics = ItemMetrics {
        meter_columns: 16,
        spinner_frames: &[],
    };

    fn bar() -> StatusBar<'static> {
        StatusBar::new(Id::root("t"))
            .left(&LEFT)
            .center(&CENTER)
            .right(&RIGHT)
    }

    #[test]
    fn a_wide_row_keeps_every_item() {
        let keep = bar().survivors(200, TEST_METRICS, 2, 1, 0);
        assert_eq!(keep, [0b11, 0b1, 0b111]);
    }

    #[test]
    fn narrow_rows_drop_centre_then_right_then_left_and_keep_the_name() {
        let keep = bar().survivors(60, TEST_METRICS, 2, 1, 0);
        assert_eq!(keep.get(1).copied(), Some(0), "the centre leaves first");
        assert_ne!(keep.first().copied(), Some(0), "identity stays");

        let keep = bar().survivors(10, TEST_METRICS, 2, 1, 0);
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
        let keep = bar.survivors(22, TEST_METRICS, 2, 1, 0);
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
        assert_eq!(plain.columns(TEST_METRICS), 3);
        assert_eq!(StatusItem::new("abc").chip().columns(TEST_METRICS), 5);
        assert_eq!(
            StatusItem::new("abc").meter(0.5).columns(TEST_METRICS),
            3 + 1 + 16
        );
    }

    #[test]
    fn a_truncated_item_paints_the_overflow_glyph() {
        const ROW: Rect = Rect::new(0, 0, 8, 1);
        const ITEMS: [StatusItem<'static>; 1] =
            [StatusItem::new("identity-that-does-not-fit").strong()];
        let theme = Theme::junie();
        let overflow = theme.design.glyphs.get(GlyphRole::Ellipsis);
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(ROW);

        runtime
            .draw_scene(ROW, &mut buffer, |ui, area| {
                StatusBar::new(Id::root("status.overflow"))
                    .left(&ITEMS)
                    .draw(ui, area);
            })
            .commit_presented();

        assert!(
            painted_row(&buffer, ROW.width).contains(overflow),
            "a retained item clipped by the row must paint the overflow glyph"
        );
    }

    const CLICK_ID: Id = Id::root("status.click");
    const CLICK_KEY: ItemKey = ItemKey::num(7);
    const CLICK_ITEMS: [StatusItem<'static>; 1] = [StatusItem::new("Open").key(CLICK_KEY)];

    #[derive(Default)]
    struct ClickApp {
        action: Option<StatusAction>,
    }

    impl App for ClickApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = StatusBar::new(CLICK_ID).left(&CLICK_ITEMS).update(cx);
            self.action = response.action_ref().copied().or(self.action);
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            StatusBar::new(CLICK_ID)
                .left(&CLICK_ITEMS)
                .draw(ui, Rect::new(0, 0, 20, 1));
        }
    }

    #[test]
    fn a_keyed_item_click_reports_chose() {
        const SCREEN: Rect = Rect::new(0, 0, 20, 1);
        let mut runtime = Runtime::new(ClickApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let item = runtime
            .area_of_part(CLICK_ID, PartRef::item(Part::LABEL, CLICK_KEY))
            .unwrap_or(Rect::ZERO);
        assert!(
            !item.is_empty(),
            "the keyed item must register a hit region"
        );
        let x = item.x.saturating_add(item.width / 2);

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, x, item.y));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, x, item.y));

        assert_eq!(runtime.app().action, Some(StatusAction::Chose(CLICK_KEY)));
    }

    const HOVER_ID: Id = Id::root("status.hover");
    const FIRST: ItemKey = ItemKey::num(1);
    const SECOND: ItemKey = ItemKey::num(2);
    const HOVER_ITEMS: [StatusItem<'static>; 2] = [
        StatusItem::new("alpha").key(FIRST),
        StatusItem::new("bravo").key(SECOND),
    ];
    const HOVER_ROW: Rect = Rect::new(0, 0, 40, 1);

    struct HoverApp;

    impl App for HoverApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            StatusBar::new(HOVER_ID)
                .left(&HOVER_ITEMS)
                .update(cx)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            StatusBar::new(HOVER_ID)
                .left(&HOVER_ITEMS)
                .draw(ui, HOVER_ROW);
        }
    }

    #[doc = r" S4: the strip paints per-item hover from `FrameRead::hovered_part`,"]
    #[doc = r" and the runtime repaints exactly when that hovered `(Id, PartRef)`"]
    #[doc = r" changes. A `Move` inside one item has no transition to show, so it"]
    #[doc = r" must not repaint; crossing to the next item must."]
    #[test]
    fn a_pointer_move_without_a_hover_transition_does_not_repaint() {
        let mut runtime = Runtime::new(HoverApp, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(HOVER_ROW);
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();
        let area = |runtime: &Runtime<HoverApp>, k: ItemKey| {
            runtime
                .area_of_part(HOVER_ID, PartRef::item(Part::LABEL, k))
                .unwrap_or(Rect::ZERO)
        };
        let first = area(&runtime, FIRST);
        let second = area(&runtime, SECOND);
        assert!(first.width >= 2 && !second.is_empty());

        let entered =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, first.x, first.y));
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();
        let stayed = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Move, first.x.saturating_add(1), first.y),
        );
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();
        let crossed =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, second.x, second.y));
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();

        assert_eq!(entered.invalidate(), Invalidate::Paint);
        assert_eq!(
            stayed.invalidate(),
            Invalidate::None,
            "moving inside one item repainted the whole strip"
        );
        assert_eq!(crossed.invalidate(), Invalidate::Paint);
    }

    #[doc = r" The painted row of `buf`, one `char` per column."]
    #[doc = r""]
    #[doc = r" Every fixture label below is ASCII, so a byte offset into this string"]
    #[doc = r" is the column the label starts at."]
    fn painted_row(buf: &Buffer, w: u16) -> String {
        let mut row = String::new();
        for x in 0..w {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                row.push_str(cell.symbol());
            }
        }
        row
    }

    #[doc = r" GAP-8: where the three groups actually land, read back out of the"]
    #[doc = r" painted buffer."]
    #[doc = r""]
    #[doc = r" Successor to the legacy"]
    #[doc = r" `widgets::statusbar::tests::groups_keep_their_order_and_sides`. The"]
    #[doc = r" replacement `survivors` returns keep-masks and never geometry, so a"]
    #[doc = r" strip that right-aligned its left group would keep every item and pass"]
    #[doc = r" every other test in this module. Asserting this against `survivors`,"]
    #[doc = r" or against any other accessor of the strip's own arithmetic, would"]
    #[doc = r" reproduce that defect one layer down; the buffer is the only witness"]
    #[doc = r" that cannot agree with a wrong implementation."]
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
        })
        .commit_presented();
        let row = painted_row(&buf, ROW.width);

        let at = |needle: &str| -> u16 {
            let found = row.find(needle);
            assert!(found.is_some(), "{needle:?} is not painted in {row:?}");
            found
                .and_then(|i| u16::try_from(i).ok())
                .unwrap_or_default()
        };
        let end = |needle: &str| -> u16 { at(needle).saturating_add(width(needle)) };

        assert_eq!(
            at("alpha"),
            ROW.x.saturating_add(edge),
            "the left group starts at the gutter, in {row:?}"
        );
        assert!(
            end("alpha") < at("bravo"),
            "left items run left-to-right in declaration order, in {row:?}"
        );

        assert_eq!(
            end("echo"),
            ROW.right().saturating_sub(edge),
            "the right group is flush against the trailing gutter, in {row:?}"
        );
        assert!(
            end("delta") < at("echo"),
            "right items run left-to-right in declaration order, in {row:?}"
        );

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
