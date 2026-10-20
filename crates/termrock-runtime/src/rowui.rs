use core::fmt::{self, Write as _};

use crate::theme::PaintStyle;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use crate::id::{Id, ItemKey, Part};
use crate::layout::{Track, distribute_into};
use crate::response::StateFlags;
use crate::text::{Span, width};
use crate::theme::{Align, CursorStyle, Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

pub trait RowFn<T> {
    fn row(&self, item: &T, row: &mut RowUi<'_>);
}

impl<T, F: Fn(&T, &mut RowUi<'_>)> RowFn<T> for F {
    fn row(&self, item: &T, row: &mut RowUi<'_>) {
        self(item, row);
    }
}

impl<T: core::fmt::Display> RowFn<T> for termrock_collections::DefaultRow {
    fn row(&self, item: &T, row: &mut RowUi<'_>) {
        row.label_fmt(format_args!("{item}"));
    }
}

pub const MAX_COLUMNS: usize = 16;

const fn selected_row_patch() -> StylePatch {
    StylePatch::new()
        .set_bg(Role::SelectionBg)
        .set_fg(Role::SelectionFg)
}

fn resolve_row_part(
    ui: &mut Ui<'_>,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    part: Part,
    patch: Option<StylePatch>,
    cursor_style: CursorStyle,
) -> crate::theme::Resolved {
    let patch = if cursor_style == CursorStyle::Fill && flags.contains(StateFlags::FOCUSED) {
        Some(patch.unwrap_or_default().merge(selected_row_patch()))
    } else {
        patch
    };
    match patch {
        Some(patch) => ui.style_patched(family, variant, part, flags, &patch),
        None => ui.style(family, variant, part, flags),
    }
}

pub struct RowUi<'u> {
    ui: Ui<'u>,
    owner: Id,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    key: ItemKey,
    row: Rect,
    cursor_style: CursorStyle,
    label_patch: Option<StylePatch>,
    left: u16,
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

    #[expect(
        clippy::too_many_arguments,
        reason = "the crate-private constructor extends RowUi's public phase context with two scoped patches"
    )]
    pub fn new_with_patches<'a: 'u>(
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
            cursor_style: CursorStyle::Bar,
            label_patch,
            left: 0,
            right: 0,
        }
    }

    pub const fn flags(&self) -> StateFlags {
        self.flags
    }

    pub const fn key(&self) -> ItemKey {
        self.key
    }

    pub const fn owner(&self) -> Id {
        self.owner
    }

    pub const fn area(&self) -> Rect {
        self.row
    }

    /// Apply the active-row cursor policy before painting row contents.
    pub fn with_cursor_style(mut self, style: CursorStyle) -> Self {
        self.cursor_style = style;
        if style == CursorStyle::Fill && self.flags.contains(StateFlags::FOCUSED) {
            let resolved = self.ui.style_patched(
                self.family,
                self.variant,
                Part::ROW,
                self.flags,
                &selected_row_patch(),
            );
            self.ui.fill(self.row, resolved.style);
        }
        self
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

    fn style_of(&mut self, part: Part) -> PaintStyle {
        let patch = (part == Part::LABEL).then_some(self.label_patch).flatten();
        let r = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            part,
            patch,
            self.cursor_style,
        );
        #[cfg(feature = "testing")]
        self.ui
            .note_styled(self.owner, self.family, self.variant, part, r);
        r.style
    }

    pub fn marker(&mut self, g: GlyphRole) {
        let r = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::MARKER,
            None,
            self.cursor_style,
        );
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

    pub fn gutter(&mut self) {
        if self.cursor_style == CursorStyle::Fill && self.flags.contains(StateFlags::FOCUSED) {
            self.left = self.left.saturating_add(1);
            return;
        }
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

    pub fn indent(&mut self, depth: u16) {
        let step = self.ui.design().space.tree_indent;
        self.left = self.left.saturating_add(depth.saturating_mul(step));
    }

    pub fn label(&mut self, s: &str) {
        let st = self.style_of(Part::LABEL);
        self.label_in(s, st);
    }

    pub fn label_patched(&mut self, s: &str, p: &StylePatch) {
        let patch = self.label_patch.map_or(*p, |forwarded| forwarded.merge(*p));
        let st = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::LABEL,
            Some(patch),
            self.cursor_style,
        )
        .style;
        self.label_in(s, st);
    }

    fn label_in(&mut self, s: &str, st: PaintStyle) {
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

    pub fn label_spans(&mut self, spans: &[Span<'_>]) {
        let st = self.style_of(Part::LABEL);
        let area = self.remaining();
        let used = self.ui.paint_spans(area, spans, st);
        self.left = self.left.saturating_add(used);
    }

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

    pub fn trailing(&mut self, s: &str, p: &StylePatch) {
        let need = width(s);
        let area = self.remaining();
        if need == 0 || need > area.width {
            return;
        }
        let st = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::META,
            Some(*p),
            self.cursor_style,
        )
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
        let r = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            p,
            None,
            self.cursor_style,
        );
        #[cfg(feature = "testing")]
        self.ui
            .note_styled(self.owner, self.family, self.variant, p, r);
        CellUi::with_resolved_glyph(self.ui.reborrow(), cell, r.style, r.glyph)
    }

    pub fn columns(&mut self, widths: &[Track]) -> ColumnsUi<'_> {
        self.columns_with_gap(widths, self.ui.design().space.column_gap)
    }

    pub fn columns_with_gap(&mut self, widths: &[Track], gap: u16) -> ColumnsUi<'_> {
        let area = self.remaining();
        let mut sizes = [0u16; MAX_COLUMNS];
        let n = widths.len().min(MAX_COLUMNS);
        distribute_into(area.width, widths.get(..n).unwrap_or(&[]), gap, &mut sizes);
        let style = self.style_of(Part::CELL);
        self.left = self.left.saturating_add(area.width);
        ColumnsUi {
            #[cfg(feature = "testing")]
            owner: self.owner,
            family: self.family,
            variant: self.variant,
            flags: self.flags,
            label_patch: self.label_patch,
            ui: self.ui.reborrow(),
            area,
            sizes,
            n,
            gap,
            style,
            cursor_style: self.cursor_style,
        }
    }

    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let row = self.row;
        let (buf, _) = self.ui.buffer_in(row);
        (buf, row)
    }
}

struct CellWriter<'a, 'u> {
    ui: &'a mut Ui<'u>,
    area: Rect,
    x: u16,
    style: PaintStyle,
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

pub struct CellUi<'u> {
    ui: Ui<'u>,
    area: Rect,
    style: PaintStyle,
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
    pub fn new(ui: Ui<'u>, area: Rect, style: PaintStyle) -> Self {
        Self::with_resolved_glyph(ui, area, style, Slot::Inherit)
    }

    fn with_resolved_glyph(
        ui: Ui<'u>,
        area: Rect,
        style: PaintStyle,
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

            Slot::Clear => {
                self.ui.fill(cell, self.style);
                cell.width
            }
            Slot::Inherit => 0,
        };
        self.used = self.used.saturating_add(used);
    }

    pub fn text(&mut self, s: &str) -> &mut Self {
        let area = self.free();
        let used = self.ui.paint_str(area, s, self.style);
        self.used = self.used.saturating_add(used);
        self
    }

    pub fn remove_modifier(&mut self, modifiers: Modifier) -> &mut Self {
        self.style = self.style.remove_modifier(modifiers);
        self
    }

    pub fn authored_modifiers(&self) -> Modifier {
        self.style.add_modifier
    }

    pub fn text_matched(&mut self, s: &str, matched: &[usize]) -> &mut Self {
        let area = self.free();
        let used = if width(s) <= area.width {
            self.ui.paint_matched(area, s, matched, self.style)
        } else {
            let head = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            let used = self.ui.paint_matched(head, s, matched, self.style);
            let tail = Rect {
                x: area.x.saturating_add(used),
                width: area.width.saturating_sub(used),
                ..area
            };
            used.saturating_add(self.ui.glyph(tail, GlyphRole::Ellipsis, self.style))
        };
        self.used = self.used.saturating_add(used);
        self
    }

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

    pub fn num(&mut self, n: i64) -> &mut Self {
        let mut buf = [0u8; 24];
        let s = format_i64(n, &mut buf);
        self.text(s)
    }

    pub fn money(&mut self, cents: i64) -> &mut Self {
        let mut buf = [0u8; 32];
        let s = format_money(cents, &mut buf);
        self.text(s)
    }

    pub const fn align(&mut self, a: Align) -> &mut Self {
        self.align = a;
        self
    }

    pub const fn tone(&mut self, r: Role) -> &mut Self {
        self.tone = Some(r);
        self
    }

    pub fn italic(&mut self, yes: bool) -> &mut Self {
        self.add = if yes {
            self.add.union(Modifier::ITALIC)
        } else {
            self.add.difference(Modifier::ITALIC)
        };
        self
    }

    pub fn suffix(&mut self, g: GlyphRole) -> &mut Self {
        let area = self.free();
        let used = self.ui.glyph(area, g, self.style);
        self.used = self.used.saturating_add(used);
        self
    }

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

        let free = self.area.width.saturating_sub(used);
        let shift = match self.align {
            Align::Left => 0,
            Align::Center => free / 2,
            Align::Right => free,
        };
        let y = self.area.y;
        if shift > 0 {
            let mut x = self.area.x.saturating_add(used);
            while x > self.area.x {
                x = x.saturating_sub(1);
                let src = Position::new(x, y);
                let dst = Position::new(x.saturating_add(shift), y);
                self.ui.move_cell(src, dst, self.style);
            }
        }

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
        if st != PaintStyle::new() {
            self.ui.paint_style(painted, st);
        }
    }
}

pub struct ColumnsUi<'u> {
    #[cfg(feature = "testing")]
    owner: Id,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    label_patch: Option<StylePatch>,
    ui: Ui<'u>,
    area: Rect,
    sizes: [u16; MAX_COLUMNS],
    n: usize,
    gap: u16,
    style: PaintStyle,
    cursor_style: CursorStyle,
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
    pub const fn len(&self) -> usize {
        self.n
    }

    pub const fn is_empty(&self) -> bool {
        self.n == 0
    }

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

    pub fn cell_part(&mut self, i: usize, part: Part) -> CellUi<'_> {
        let rect = self.rect(i);
        let patch = (part == Part::LABEL).then_some(self.label_patch).flatten();
        let resolved = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            part,
            patch,
            self.cursor_style,
        );
        #[cfg(feature = "testing")]
        self.ui
            .note_styled(self.owner, self.family, self.variant, part, resolved);
        CellUi::with_resolved_glyph(self.ui.reborrow(), rect, resolved.style, resolved.glyph)
    }

    pub fn cell(&mut self, i: usize) -> CellUi<'_> {
        let rect = self.rect(i);
        let style = self.style;
        CellUi::new(self.ui.reborrow(), rect, style)
    }
}

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

impl termrock_text::secret::CellWriter for CellUi<'_> {
    fn glyphs(&mut self, mask: crate::theme::GlyphRole, n: usize) {
        self.glyphs(mask, n);
    }

    fn text(&mut self, s: &str) {
        self.text(s);
    }
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

    #[test]
    fn row_ui_label_writes_cells_without_an_intermediate_string() {
        let page = paint(Rect::new(0, 0, 10, 1), |r| r.label("hello"));
        assert_eq!(row_text(&page, 0, 0, 10), "hello     ");

        let page = paint(Rect::new(0, 0, 5, 1), |r| r.label("hello world"));
        let painted = row_text(&page, 0, 0, 5);
        assert_eq!(painted.chars().count(), 5);
        assert!(painted.ends_with('…'), "{painted:?}");

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

    #[test]
    fn row_ui_meta_is_dropped_all_or_none() {
        let page = paint(Rect::new(0, 0, 12, 1), |r| {
            r.meta("42");
            r.label("name");
        });
        assert_eq!(row_text(&page, 0, 0, 12), "name      42");

        let page = paint(Rect::new(0, 0, 3, 1), |r| {
            r.meta("42");
            r.label("name");
        });
        assert_eq!(row_text(&page, 0, 0, 3), "na…");

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

        assert_eq!(set, "xyz✓");

        assert_eq!(clear, "xyz ");

        assert_eq!(inherit, "xyzw");
        assert_ne!(clear, inherit);
    }

    #[test]
    fn row_ui_columns_clip_to_the_row() {
        let page = paint(Rect::new(2, 0, 12, 1), |r| {
            let mut c = r.columns(&[Track::Flex(1), Track::Flex(1)]);
            c.cell(0).text("aaaaaaaa");
            c.cell(1).text("bbbbbbbb");
        });

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
