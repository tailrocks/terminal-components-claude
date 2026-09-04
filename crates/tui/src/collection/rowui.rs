//! Per-row and per-cell painting (`COMPONENT_ARCHITECTURE.md` §12.2, §20.9-6, §21 item 21).
//!
//! Parts come pre-styled for the row's resolved state. Every write goes
//! through the clipping writer in one grapheme walk with no intermediate
//! `String`; `label_fmt`, `num` and `money` format in place.

use core::fmt::{self, Write as _};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Modifier, Style};

use crate::id::{Id, ItemKey, Part};
use crate::layout::{Track, distribute_into};
use crate::response::StateFlags;
use crate::text::{Span, width};
use crate::theme::{Align, Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

/// Maximum columns `RowUi::columns` lays out without allocating.
///
/// Tracks beyond this cap are **silently ignored** — the fixed `[u16; 16]`
/// track buffer is what makes the row painter allocation-free (§12.2, R5,
/// MI-8).
pub const MAX_COLUMNS: usize = 16;

/// A painter for one collection row, its parts pre-styled.
pub struct RowUi<'u> {
    ui: Ui<'u>,
    owner: Id,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    key: ItemKey,
    row: Rect,
    label_patch: Option<StylePatch>,
    /// Next free column from the left.
    left: u16,
    /// Columns reserved from the right (already consumed).
    right: u16,
}

impl fmt::Debug for RowUi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RowUi")
            .field("owner", &self.owner)
            .field("key", &self.key)
            .field("row", &self.row)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish_non_exhaustive()
    }
}

impl<'u> RowUi<'u> {
    /// Begin painting `row` for item `key` of `owner`, wearing `flags`.
    /// The row is filled with the family's `CONTAINER` style first.
    pub fn new<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
    ) -> RowUi<'u> {
        Self::new_with_patches(ui, owner, family, variant, flags, key, row, None, None)
    }

    /// Begin a row with component-owned patches forwarded only to the
    /// automatic container fill and label painters.
    #[expect(
        clippy::too_many_arguments,
        reason = "the crate-private constructor extends RowUi's public phase context with two scoped patches"
    )]
    pub(crate) fn new_with_patches<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        container_patch: Option<StylePatch>,
        label_patch: Option<StylePatch>,
    ) -> RowUi<'u> {
        let mut ui = ui.reborrow();
        let container = match container_patch {
            Some(patch) => {
                ui.style_patched(family, variant, Part::CONTAINER, flags, &patch)
                    .style
            }
            None => ui.style(family, variant, Part::CONTAINER, flags).style,
        };
        ui.fill(row, container);
        RowUi {
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            label_patch,
            left: 0,
            right: 0,
        }
    }

    /// The row's state flags.
    pub const fn flags(&self) -> StateFlags {
        self.flags
    }

    /// The item key.
    pub const fn key(&self) -> ItemKey {
        self.key
    }

    /// The owner id.
    pub const fn owner(&self) -> Id {
        self.owner
    }

    /// The whole row rect.
    pub const fn area(&self) -> Rect {
        self.row
    }

    fn remaining(&self) -> Rect {
        let used = self.left.saturating_add(self.right);
        Rect {
            x: self.row.x.saturating_add(self.left),
            y: self.row.y,
            width: self.row.width.saturating_sub(used),
            height: 1,
        }
    }

    fn style_of(&mut self, part: Part) -> Style {
        let r = match (part, self.label_patch) {
            (Part::LABEL, Some(patch)) => {
                self.ui
                    .style_patched(self.family, self.variant, part, self.flags, &patch)
            }
            _ => self.ui.style(self.family, self.variant, part, self.flags),
        };
        #[cfg(feature = "testing")]
        self.ui
            .note_styled(self.owner, self.family, self.variant, part, r);
        r.style
    }

    /// Paint a marker glyph at the left, then a gap. A resolved `Set`
    /// overrides `g`; `Clear` suppresses the marker while keeping its cell.
    pub fn marker(&mut self, g: GlyphRole) {
        let r = self
            .ui
            .style(self.family, self.variant, Part::MARKER, self.flags);
        #[cfg(feature = "testing")]
        self.ui
            .note_styled(self.owner, self.family, self.variant, Part::MARKER, r);
        let area = self.remaining();
        let cell = Rect {
            width: area.width.min(1),
            ..area
        };
        match r.glyph {
            Slot::Set(glyph) => {
                let used = self.ui.glyph(area, glyph, r.style);
                self.left = self.left.saturating_add(used).saturating_add(1);
            }
            Slot::Inherit => {
                let used = self.ui.glyph(area, g, r.style);
                self.left = self.left.saturating_add(used).saturating_add(1);
            }
            Slot::Clear => {
                self.ui.fill(cell, r.style);
                self.left = self.left.saturating_add(2);
            }
        }
    }

    /// Paint the focus gutter (`GlyphRole::FocusBar` when the recipe says so,
    /// else a blank gutter cell).
    pub fn gutter(&mut self) {
        let r = self
            .ui
            .style(self.family, self.variant, Part::GUTTER, self.flags);
        let area = self.remaining();
        let cell = Rect {
            width: area.width.min(1),
            ..area
        };
        match r.glyph {
            Slot::Set(g) => {
                self.ui.glyph(cell, g, r.style);
            }
            Slot::Inherit | Slot::Clear => self.ui.fill(cell, r.style),
        }
        self.left = self.left.saturating_add(1);
    }

    /// Indent by `depth` tree levels.
    pub fn indent(&mut self, depth: u16) {
        let step = self.ui.design().space.tree_indent;
        self.left = self.left.saturating_add(depth.saturating_mul(step));
    }

    /// Paint the label into what is left, ending with the ellipsis glyph
    /// when it does not fit (the legacy `fit` contract, now allocation-free).
    pub fn label(&mut self, s: &str) {
        let st = self.style_of(Part::LABEL);
        self.label_in(s, st);
    }

    /// Paint the label with an instance patch.
    pub fn label_patched(&mut self, s: &str, p: &StylePatch) {
        let patch = self.label_patch.map_or(*p, |forwarded| forwarded.merge(*p));
        let st = self
            .ui
            .style_patched(self.family, self.variant, Part::LABEL, self.flags, &patch)
            .style;
        self.label_in(s, st);
    }

    fn label_in(&mut self, s: &str, st: Style) {
        let area = self.remaining();
        let used = if width(s) <= area.width {
            self.ui.paint_str(area, s, st)
        } else {
            let head = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            let used = self.ui.paint_str(head, s, st);
            let tail = Rect {
                x: area.x.saturating_add(used),
                width: area.width.saturating_sub(used),
                ..area
            };
            used.saturating_add(self.ui.glyph(tail, GlyphRole::Ellipsis, st))
        };
        self.left = self.left.saturating_add(used);
    }

    /// Paint role-carrying spans as the label (`Buffer::set_line`).
    pub fn label_spans(&mut self, spans: &[Span<'_>]) {
        let st = self.style_of(Part::LABEL);
        let area = self.remaining();
        let used = self.ui.paint_spans(area, spans, st);
        self.left = self.left.saturating_add(used);
    }

    /// Format the label in place (0 allocations).
    pub fn label_fmt(&mut self, args: fmt::Arguments<'_>) {
        let st = self.style_of(Part::LABEL);
        let area = self.remaining();
        let mut w = CellWriter {
            ui: &mut self.ui,
            area,
            x: area.x,
            style: st,
        };
        let _ = w.write_fmt(args);
        let used = w.x.saturating_sub(area.x);
        self.left = self.left.saturating_add(used);
    }

    /// Right-aligned meta text, dropped all-or-none when it does not fit
    /// after a two-cell gap (`DESIGN.md:478`).
    pub fn meta(&mut self, s: &str) {
        let need = width(s);
        let area = self.remaining();
        if need == 0 || need.saturating_add(2) > area.width {
            return;
        }
        let st = self.style_of(Part::META);
        let cell = Rect {
            x: area.right().saturating_sub(need),
            y: area.y,
            width: need,
            height: 1,
        };
        self.ui.paint_str(cell, s, st);
        self.right = self.right.saturating_add(need).saturating_add(1);
    }

    /// Right-aligned trailing text with an instance patch (dropped when it
    /// does not fit).
    pub fn trailing(&mut self, s: &str, p: &StylePatch) {
        let need = width(s);
        let area = self.remaining();
        if need == 0 || need > area.width {
            return;
        }
        let st = self
            .ui
            .style_patched(self.family, self.variant, Part::META, self.flags, p)
            .style;
        let cell = Rect {
            x: area.right().saturating_sub(need),
            y: area.y,
            width: need,
            height: 1,
        };
        self.ui.paint_str(cell, s, st);
        self.right = self.right.saturating_add(need).saturating_add(1);
    }

    /// Reserve `width` columns from the right for `p`; `label` fills what
    /// is left.
    pub fn part(&mut self, p: Part, width: u16) -> CellUi<'_> {
        let area = self.remaining();
        let w = width.min(area.width);
        let cell = Rect {
            x: area.right().saturating_sub(w),
            y: area.y,
            width: w,
            height: 1,
        };
        self.right = self.right.saturating_add(w).saturating_add(1);
        let r = self.ui.style(self.family, self.variant, p, self.flags);
        #[cfg(feature = "testing")]
        self.ui
            .note_styled(self.owner, self.family, self.variant, p, r);
        CellUi::with_resolved_glyph(self.ui.reborrow(), cell, r.style, r.glyph)
    }

    /// Split what is left into columns.
    ///
    /// At most [`MAX_COLUMNS`] tracks are laid out; further tracks are
    /// **silently ignored** so the row can never allocate (§12.2, MI-8). A
    /// component that needs more columns than the cap is a design error, not
    /// a runtime one: split the row.
    pub fn columns(&mut self, widths: &[Track]) -> ColumnsUi<'_> {
        let area = self.remaining();
        let gap = self.ui.design().space.column_gap;
        let mut sizes = [0u16; MAX_COLUMNS];
        let n = widths.len().min(MAX_COLUMNS);
        distribute_into(area.width, widths.get(..n).unwrap_or(&[]), gap, &mut sizes);
        let style = self.style_of(Part::CELL);
        self.left = self.left.saturating_add(area.width);
        ColumnsUi {
            ui: self.ui.reborrow(),
            area,
            sizes,
            n,
            gap,
            style,
        }
    }

    /// The buffer and the row rect; marks **the row rect** written, not the
    /// whole clip (BL-3): a right-aligned cell inside a layer must not make
    /// the layer's written-cell bitset all-true.
    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let row = self.row;
        let (buf, _) = self.ui.buffer_in(row);
        (buf, row)
    }
}

/// A `fmt::Write` that paints straight into cells.
struct CellWriter<'a, 'u> {
    ui: &'a mut Ui<'u>,
    area: Rect,
    x: u16,
    style: Style,
}

impl fmt::Write for CellWriter<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let right = self.area.right();
        if self.x >= right {
            return Ok(());
        }
        let cell = Rect {
            x: self.x,
            y: self.area.y,
            width: right.saturating_sub(self.x),
            height: 1,
        };
        let used = self.ui.paint_str(cell, s, self.style);
        self.x = self.x.saturating_add(used);
        Ok(())
    }
}

/// A painter for one cell; content is laid out and styled on drop.
pub struct CellUi<'u> {
    ui: Ui<'u>,
    area: Rect,
    style: Style,
    /// Columns painted so far, from `area.x`.
    used: u16,
    align: Align,
    tone: Option<Role>,
    add: Modifier,
    patch: Option<StylePatch>,
    resolved_glyph: Slot<GlyphRole>,
}

impl fmt::Debug for CellUi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CellUi")
            .field("area", &self.area)
            .field("used", &self.used)
            .field("align", &self.align)
            .finish_non_exhaustive()
    }
}

impl<'u> CellUi<'u> {
    pub(crate) fn new(ui: Ui<'u>, area: Rect, style: Style) -> Self {
        Self::with_resolved_glyph(ui, area, style, Slot::Inherit)
    }

    fn with_resolved_glyph(
        ui: Ui<'u>,
        area: Rect,
        style: Style,
        resolved_glyph: Slot<GlyphRole>,
    ) -> Self {
        CellUi {
            ui,
            area,
            style,
            used: 0,
            align: Align::Left,
            tone: None,
            add: Modifier::empty(),
            patch: None,
            resolved_glyph,
        }
    }

    /// Columns the resolved glyph slot reserves at the end of the cell.
    ///
    /// `Set` reserves the resolved glyph's own width. `Clear` reserves one
    /// blank cell: an explicitly cleared glyph keeps the geometry it would
    /// have had, exactly as `RowUi::marker` keeps the marker cell and its gap
    /// (§12.2; §29.1 requires `Clear` to stay distinguishable from
    /// `Inherit`). `Inherit` reserves nothing, leaving caller content or a
    /// `suffix` authoritative.
    fn resolved_glyph_width(&self) -> u16 {
        match self.resolved_glyph {
            Slot::Set(g) => width(self.ui.design().glyphs.get(g)),
            Slot::Clear => 1,
            Slot::Inherit => 0,
        }
    }

    fn free(&self) -> Rect {
        let reserved = self.resolved_glyph_width();
        Rect {
            x: self.area.x.saturating_add(self.used),
            y: self.area.y,
            width: self
                .area
                .width
                .saturating_sub(self.used)
                .saturating_sub(reserved),
            height: 1,
        }
    }

    fn paint_resolved_glyph(&mut self) {
        let reserved = self.resolved_glyph_width();
        let free = self.area.width.saturating_sub(self.used);
        if reserved == 0 || free == 0 {
            return;
        }
        let cell = Rect {
            x: self.area.x.saturating_add(self.used),
            y: self.area.y,
            width: reserved.min(free),
            height: 1,
        };
        let used = match self.resolved_glyph {
            Slot::Set(g) => self.ui.glyph(cell, g, self.style),
            // The reservation is kept and shown blank, so a cleared glyph is
            // not the same picture as an inherited one (§29.1).
            Slot::Clear => {
                self.ui.fill(cell, self.style);
                cell.width
            }
            Slot::Inherit => 0,
        };
        self.used = self.used.saturating_add(used);
    }

    /// Paint text.
    pub fn text(&mut self, s: &str) -> &mut Self {
        let area = self.free();
        let used = self.ui.paint_str(area, s, self.style);
        self.used = self.used.saturating_add(used);
        self
    }

    /// Paint a glyph role `n` times (masks).
    pub fn glyphs(&mut self, g: GlyphRole, n: usize) -> &mut Self {
        let sym = self.ui.design().glyphs.get(g);
        for _ in 0..n {
            let area = self.free();
            if area.is_empty() {
                break;
            }
            let used = self.ui.paint_str(area, sym, self.style);
            if used == 0 {
                break;
            }
            self.used = self.used.saturating_add(used);
        }
        self
    }

    /// Format a number in place (0 allocations).
    pub fn num(&mut self, n: i64) -> &mut Self {
        let mut buf = [0u8; 24];
        let s = format_i64(n, &mut buf);
        self.text(s)
    }

    /// Format cents as `-1,234.56` in place (0 allocations).
    pub fn money(&mut self, cents: i64) -> &mut Self {
        let mut buf = [0u8; 32];
        let s = format_money(cents, &mut buf);
        self.text(s)
    }

    /// Set the alignment (applied on drop).
    pub const fn align(&mut self, a: Align) -> &mut Self {
        self.align = a;
        self
    }

    /// Set the foreground role (applied on drop).
    pub const fn tone(&mut self, r: Role) -> &mut Self {
        self.tone = Some(r);
        self
    }

    /// Italic (applied on drop).
    pub fn italic(&mut self, yes: bool) -> &mut Self {
        self.add = if yes {
            self.add.union(Modifier::ITALIC)
        } else {
            self.add.difference(Modifier::ITALIC)
        };
        self
    }

    /// A trailing glyph.
    pub fn suffix(&mut self, g: GlyphRole) -> &mut Self {
        let area = self.free();
        let used = self.ui.glyph(area, g, self.style);
        self.used = self.used.saturating_add(used);
        self
    }

    /// An instance patch over the cell's style (applied on drop).
    pub fn patch(&mut self, p: &StylePatch) -> &mut Self {
        self.patch = Some(match self.patch {
            Some(cur) => cur.merge(*p),
            None => *p,
        });
        self
    }
}

impl Drop for CellUi<'_> {
    fn drop(&mut self) {
        self.paint_resolved_glyph();
        let used = self.used.min(self.area.width);
        if used == 0 {
            return;
        }
        // shift painted cells for alignment
        let free = self.area.width.saturating_sub(used);
        let shift = match self.align {
            Align::Left => 0,
            Align::Center => free / 2,
            Align::Right => free,
        };
        let y = self.area.y;
        if shift > 0 {
            let (buf, _) = self.ui.buffer_in(self.area);
            let mut x = self.area.x.saturating_add(used);
            while x > self.area.x {
                x = x.saturating_sub(1);
                let src = Position::new(x, y);
                let dst = Position::new(x.saturating_add(shift), y);
                if let Some(c) = buf.cell(src).cloned()
                    && let Some(d) = buf.cell_mut(dst)
                {
                    *d = c;
                }
                if let Some(c) = buf.cell_mut(src) {
                    c.reset();
                    c.set_style(self.style);
                }
            }
        }
        // final style over the painted range
        let painted = Rect {
            x: self.area.x.saturating_add(shift),
            y,
            width: used,
            height: 1,
        };
        let theme = self.ui.theme_ref();
        let surface = self.ui.surface();
        let mut delta = StylePatch::new().add(self.add);
        if let Some(r) = self.tone {
            delta = delta.set_fg(r);
        }
        let st = crate::theme::resolve::bind(theme, delta, self.patch.as_ref(), surface).style;
        if st != Style::new() {
            self.ui.paint_style(painted, st);
        }
    }
}

/// Column cells over the remainder of a row.
pub struct ColumnsUi<'u> {
    ui: Ui<'u>,
    area: Rect,
    sizes: [u16; MAX_COLUMNS],
    n: usize,
    gap: u16,
    style: Style,
}

impl fmt::Debug for ColumnsUi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColumnsUi")
            .field("area", &self.area)
            .field("n", &self.n)
            .finish_non_exhaustive()
    }
}

impl ColumnsUi<'_> {
    /// The number of columns.
    pub const fn len(&self) -> usize {
        self.n
    }

    /// Whether there are no columns.
    pub const fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// The rect of column `i`, clipped to the row.
    pub fn rect(&self, i: usize) -> Rect {
        if i >= self.n {
            return Rect::ZERO;
        }
        let mut x = self.area.x;
        for (k, w) in self.sizes.iter().enumerate().take(i) {
            let _ = k;
            x = x.saturating_add(*w).saturating_add(self.gap);
        }
        let w = self.sizes.get(i).copied().unwrap_or(0);
        Rect {
            x,
            y: self.area.y,
            width: w.min(self.area.right().saturating_sub(x)),
            height: 1,
        }
        .intersection(self.area)
    }

    /// A painter for column `i` (an empty painter beyond the last column).
    pub fn cell(&mut self, i: usize) -> CellUi<'_> {
        let rect = self.rect(i);
        let style = self.style;
        CellUi::new(self.ui.reborrow(), rect, style)
    }
}

/// Format an `i64` into `buf`; returns the written slice.
fn format_i64(n: i64, buf: &mut [u8; 24]) -> &str {
    let mut i = buf.len();
    let neg = n < 0;
    let mut v = n.unsigned_abs();
    if v == 0 {
        i = i.saturating_sub(1);
        if let Some(b) = buf.get_mut(i) {
            *b = b'0';
        }
    }
    while v > 0 {
        i = i.saturating_sub(1);
        if let Some(b) = buf.get_mut(i) {
            *b = b'0'.saturating_add((v % 10) as u8);
        }
        v /= 10;
    }
    if neg {
        i = i.saturating_sub(1);
        if let Some(b) = buf.get_mut(i) {
            *b = b'-';
        }
    }
    core::str::from_utf8(buf.get(i..).unwrap_or(&[])).unwrap_or("")
}

/// Format cents as `-1,234.56` into `buf`; returns the written slice.
fn format_money(cents: i64, buf: &mut [u8; 32]) -> &str {
    let mut i = buf.len();
    let neg = cents < 0;
    let v = cents.unsigned_abs();
    let (whole, frac) = (v / 100, v % 100);
    let mut put = |b: u8| {
        i = i.saturating_sub(1);
        if let Some(slot) = buf.get_mut(i) {
            *slot = b;
        }
    };
    put(b'0'.saturating_add((frac % 10) as u8));
    put(b'0'.saturating_add((frac / 10) as u8));
    put(b'.');
    let mut w = whole;
    let mut digits = 0u32;
    loop {
        put(b'0'.saturating_add((w % 10) as u8));
        w /= 10;
        digits = digits.saturating_add(1);
        if w == 0 {
            break;
        }
        if digits.is_multiple_of(3) {
            put(b',');
        }
    }
    if neg {
        put(b'-');
    }
    core::str::from_utf8(buf.get(i..).unwrap_or(&[])).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, UiCore};

    const OWNER: Id = Id::root("rowui.owner");
    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 30,
        height: 2,
    };

    /// Paint one row and return the page buffer.
    fn paint(row: Rect, f: impl FnOnce(&mut RowUi<'_>)) -> Buffer {
        let theme = Theme::junie();
        paint_with(&theme, Family::LIST, StateFlags::empty(), row, f)
    }

    fn paint_with(
        theme: &Theme,
        family: Family,
        flags: StateFlags,
        row: Rect,
        f: impl FnOnce(&mut RowUi<'_>),
    ) -> Buffer {
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            let mut r = RowUi::new(
                &mut ui,
                OWNER,
                family,
                Variant::DEFAULT,
                flags,
                ItemKey::index(0),
                row,
            );
            f(&mut r);
        }
        page
    }

    fn paint_with_patches(
        container_patch: Option<StylePatch>,
        label_patch: Option<StylePatch>,
        f: impl FnOnce(&mut RowUi<'_>),
    ) -> Buffer {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            let mut row = RowUi::new_with_patches(
                &mut ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::index(0),
                Rect::new(0, 0, 20, 1),
                container_patch,
                label_patch,
            );
            f(&mut row);
        }
        page
    }

    fn theme_with_part_glyph(part: Part, glyph: Slot<GlyphRole>) -> Theme {
        let mut theme = Theme::junie();
        theme.recipes.get_mut(Family::LIST).parts.entry(part).glyph = glyph;
        theme
    }

    fn row_text(buf: &Buffer, y: u16, from: u16, to: u16) -> String {
        let mut out = String::new();
        let mut x = from;
        while x < to {
            let Some(c) = buf.cell((x, y)) else { break };
            out.push_str(c.symbol());
            x = x.saturating_add(width(c.symbol()).max(1));
        }
        out
    }

    /// R5: the label path writes straight into cells; the only allocation a
    /// row may make is none at all. Proved structurally here (the painter
    /// takes `&str` and routes to `Buffer::set_stringn`) and by cell content.
    #[test]
    fn row_ui_label_writes_cells_without_an_intermediate_string() {
        let page = paint(Rect::new(0, 0, 10, 1), |r| r.label("hello"));
        assert_eq!(row_text(&page, 0, 0, 10), "hello     ");
        // the ellipsis path truncates in place, ending with the theme glyph
        let page = paint(Rect::new(0, 0, 5, 1), |r| r.label("hello world"));
        let painted = row_text(&page, 0, 0, 5);
        assert_eq!(painted.chars().count(), 5);
        assert!(painted.ends_with('…'), "{painted:?}");
        // `label_fmt` formats into cells too
        let page = paint(Rect::new(0, 0, 10, 1), |r| {
            r.label_fmt(format_args!("{}-{}", 12, 7));
        });
        assert_eq!(row_text(&page, 0, 0, 10), "12-7      ");
    }

    #[test]
    fn forwarded_patches_reach_only_the_automatic_container_and_labels() {
        let container = StylePatch::new().add(Modifier::REVERSED);
        let label = StylePatch::new().add(Modifier::BOLD);
        let page = paint_with_patches(Some(container), Some(label), |row| {
            row.marker(GlyphRole::WarningMark);
            row.meta("m");
            row.label("label");
        });

        assert!(
            page.cell((10, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::REVERSED)),
            "the automatic container fill receives its forwarded patch"
        );
        assert!(
            page.cell((2, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD)),
            "automatic label painters receive the forwarded label patch"
        );
        assert!(
            page.cell((0, 0))
                .is_some_and(|cell| !cell.modifier.contains(Modifier::BOLD)),
            "MARKER remains row-owned"
        );
        assert!(
            page.cell((19, 0))
                .is_some_and(|cell| !cell.modifier.contains(Modifier::BOLD)),
            "META remains row-owned"
        );
    }

    #[test]
    fn explicit_label_patch_wins_over_the_forwarded_label_patch() {
        let forwarded = StylePatch::new().add(Modifier::BOLD);
        let local = StylePatch::new().remove(Modifier::BOLD);
        let page = paint_with_patches(None, Some(forwarded), |row| {
            row.label_patched("label", &local);
        });
        assert!(
            page.cell((0, 0))
                .is_some_and(|cell| !cell.modifier.contains(Modifier::BOLD))
        );
    }

    /// `DESIGN.md:478`: meta is right-aligned and dropped **all or none**
    /// when it does not fit after a two-cell gap.
    #[test]
    fn row_ui_meta_is_dropped_all_or_none() {
        // room for the label, the gap and the meta
        let page = paint(Rect::new(0, 0, 12, 1), |r| {
            r.meta("42");
            r.label("name");
        });
        assert_eq!(row_text(&page, 0, 0, 12), "name      42");
        // one column short: nothing of the meta is painted, not a truncation
        let page = paint(Rect::new(0, 0, 3, 1), |r| {
            r.meta("42");
            r.label("name");
        });
        assert_eq!(row_text(&page, 0, 0, 3), "na…");
        // exactly at the boundary (need + 2 == width) it still fits
        let page = paint(Rect::new(0, 0, 4, 1), |r| r.meta("42"));
        assert_eq!(row_text(&page, 0, 0, 4), "  42");
    }

    #[test]
    fn row_ui_marker_honours_resolved_glyph_slot() {
        let set = theme_with_part_glyph(Part::MARKER, Slot::Set(GlyphRole::Checked));
        let page = paint_with(
            &set,
            Family::LIST,
            StateFlags::empty(),
            Rect::new(0, 0, 8, 1),
            |r| {
                r.marker(GlyphRole::WarningMark);
                r.label("item");
            },
        );
        assert_eq!(row_text(&page, 0, 0, 2), "✓ ");

        let clear = theme_with_part_glyph(Part::MARKER, Slot::Clear);
        let page = paint_with(
            &clear,
            Family::LIST,
            StateFlags::empty(),
            Rect::new(0, 0, 8, 1),
            |r| {
                r.marker(GlyphRole::WarningMark);
                r.label("item");
            },
        );
        assert_eq!(row_text(&page, 0, 0, 2), "  ");
    }

    #[test]
    fn row_ui_part_paints_a_resolved_glyph_as_a_reserved_suffix() {
        let theme = theme_with_part_glyph(Part::META, Slot::Set(GlyphRole::Checked));
        let page = paint_with(
            &theme,
            Family::LIST,
            StateFlags::empty(),
            Rect::new(0, 0, 8, 1),
            |r| {
                r.part(Part::META, 4).text("x");
            },
        );
        assert_eq!(row_text(&page, 0, 4, 8), "x✓  ");
    }

    /// §29.1: at a `part`, `Slot::Clear` stays distinguishable from
    /// `Slot::Inherit` — it paints a blank cell and still consumes the width a
    /// `Slot::Set` glyph would have reserved, so the caller's content is
    /// truncated identically under `Set` and `Clear`.
    #[test]
    fn row_ui_part_clear_reserves_a_blank_cell_and_differs_from_inherit() {
        let cell_text = |glyph| {
            let theme = theme_with_part_glyph(Part::META, glyph);
            let page = paint_with(
                &theme,
                Family::LIST,
                StateFlags::empty(),
                Rect::new(0, 0, 8, 1),
                |r| {
                    r.part(Part::META, 4).text("xyzw");
                },
            );
            row_text(&page, 0, 4, 8)
        };
        let set = cell_text(Slot::Set(GlyphRole::Checked));
        let clear = cell_text(Slot::Clear);
        let inherit = cell_text(Slot::Inherit);
        // `Set` reserves the glyph's width and paints it.
        assert_eq!(set, "xyz✓");
        // `Clear` reserves the same cell and paints it blank.
        assert_eq!(clear, "xyz ");
        // `Inherit` reserves nothing: the caller's content owns the cell.
        assert_eq!(inherit, "xyzw");
        assert_ne!(clear, inherit);
    }

    /// Columns are clipped to the row: nothing is written past `row.right()`,
    /// and the fixed `[u16; MAX_COLUMNS]` never overflows the rect.
    #[test]
    fn row_ui_columns_clip_to_the_row() {
        let page = paint(Rect::new(2, 0, 12, 1), |r| {
            let mut c = r.columns(&[Track::Flex(1), Track::Flex(1)]);
            c.cell(0).text("aaaaaaaa");
            c.cell(1).text("bbbbbbbb");
        });
        // outside the row on both sides the page is untouched
        assert_eq!(
            page.cell((0, 0)).map(|c| c.symbol().to_owned()),
            Some(" ".to_owned())
        );
        assert_eq!(
            page.cell((1, 0)).map(|c| c.symbol().to_owned()),
            Some(" ".to_owned())
        );
        for x in 14..30u16 {
            assert_eq!(
                page.cell((x, 0)).map(|c| c.symbol().to_owned()),
                Some(" ".to_owned())
            );
        }
        let inside = row_text(&page, 0, 2, 14);
        assert_eq!(inside.chars().count(), 12);
        // more tracks than MAX_COLUMNS are ignored, not painted past the row
        let many: Vec<Track> = (0..MAX_COLUMNS + 4).map(|_| Track::Flex(1)).collect();
        let page = paint(Rect::new(0, 0, 20, 1), |r| {
            let mut c = r.columns(&many);
            for i in 0..4 {
                c.cell(i).text("xx");
            }
        });
        for x in 20..30u16 {
            assert_eq!(
                page.cell((x, 0)).map(|c| c.symbol().to_owned()),
                Some(" ".to_owned())
            );
        }
    }

    #[test]
    fn in_place_number_formatting() {
        let mut b = [0u8; 24];
        assert_eq!(format_i64(0, &mut b), "0");
        assert_eq!(format_i64(-1234, &mut b), "-1234");
        assert_eq!(format_i64(i64::MIN, &mut b), "-9223372036854775808");
        let mut m = [0u8; 32];
        assert_eq!(format_money(0, &mut m), "0.00");
        assert_eq!(format_money(-123_456, &mut m), "-1,234.56");
        assert_eq!(format_money(5, &mut m), "0.05");
        assert_eq!(format_money(100_000_000, &mut m), "1,000,000.00");
    }
}
