use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::{RenderCtx, VisualState};
use crate::widgets::menu::MenuItem;
use crate::widgets::table::SortDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Col {
    pub min: u16,

    pub ideal: u16,

    pub max: u16,

    pub flex: u16,

    pub drop: Option<u8>,
}

impl Col {
    pub const fn content(min: u16, measured: u16) -> Self {
        let ideal = if measured > min { measured } else { min };
        Self {
            min,
            ideal,
            max: ideal,
            flex: 0,
            drop: None,
        }
    }

    pub const fn fixed(w: u16) -> Self {
        Self::content(w, w)
    }

    pub const fn flex(mut self, weight: u16, max: u16) -> Self {
        self.flex = weight;
        self.max = if max > self.ideal { max } else { self.ideal };
        self
    }

    pub const fn max(mut self, max: u16) -> Self {
        self.max = if max > self.ideal { max } else { self.ideal };
        self
    }

    pub const fn droppable(mut self, priority: u8) -> Self {
        self.drop = Some(priority);
        self
    }
}

pub fn solve(cols: &[Col], avail: u16, gap: u16) -> Vec<u16> {
    let n = cols.len();
    let mut visible = vec![true; n];
    let gaps = |v: &[bool]| gap as u32 * v.iter().filter(|x| **x).count().saturating_sub(1) as u32;
    let mins = |v: &[bool]| -> u32 {
        cols.iter()
            .zip(v)
            .filter(|(_, on)| **on)
            .map(|(c, _)| c.min as u32)
            .sum()
    };

    while mins(&visible) + gaps(&visible) > avail as u32 {
        let victim = cols
            .iter()
            .enumerate()
            .filter(|(i, c)| visible[*i] && c.drop.is_some())
            .min_by_key(|(i, c)| (c.drop, std::cmp::Reverse(*i)))
            .map(|(i, _)| i);
        match victim {
            Some(i) => visible[i] = false,
            None => break,
        }
    }
    let room = (avail as u32).saturating_sub(gaps(&visible));
    let mut w: Vec<u32> = cols
        .iter()
        .zip(&visible)
        .map(|(c, on)| if *on { c.min as u32 } else { 0 })
        .collect();

    let mut used: u32 = w.iter().sum();
    for i in (0..n).rev() {
        if used <= room {
            break;
        }
        let cut = (used - room).min(w[i]);
        w[i] -= cut;
        used -= cut;
    }
    let mut spare = room.saturating_sub(used);

    while spare > 0 {
        let mut grew = false;
        for i in 0..n {
            if spare == 0 {
                break;
            }
            if visible[i] && w[i] < cols[i].ideal as u32 {
                w[i] += 1;
                spare -= 1;
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }

    while spare > 0 {
        let open: Vec<usize> = (0..n)
            .filter(|&i| visible[i] && cols[i].flex > 0 && w[i] < cols[i].max as u32)
            .collect();
        if open.is_empty() {
            break;
        }
        let total: u32 = open.iter().map(|&i| cols[i].flex as u32).sum();
        let round = spare;
        let mut given = 0;
        for &i in &open {
            let share = (round * cols[i].flex as u32 / total)
                .max(1)
                .min(cols[i].max as u32 - w[i])
                .min(spare);
            w[i] += share;
            spare -= share;
            given += share;
        }
        if given == 0 {
            break;
        }
    }

    for i in 0..n {
        if spare == 0 {
            break;
        }
        if visible[i] && w[i] < cols[i].max as u32 {
            let give = (cols[i].max as u32 - w[i]).min(spare);
            w[i] += give;
            spare -= give;
        }
    }
    w.into_iter().map(|x| x as u16).collect()
}

pub const MIN_USER_WIDTH: u16 = 3;

pub const DEFAULT_SEAM_W: u16 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSizing {
    user: Vec<Option<u16>>,
    drag: Option<SeamDrag>,

    pub resizable: bool,

    pub seam_w: u16,

    pub min_width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SeamDrag {
    col: usize,
    start_w: u16,
    start_x: u16,
}

impl Default for ColumnSizing {
    fn default() -> Self {
        Self {
            user: Vec::new(),
            drag: None,
            resizable: true,
            seam_w: DEFAULT_SEAM_W,
            min_width: MIN_USER_WIDTH,
        }
    }
}

impl ColumnSizing {
    pub fn user(&self, col: usize) -> Option<u16> {
        self.user.get(col).copied().flatten()
    }

    pub fn width(&self, col: usize, measured: u16) -> u16 {
        self.user(col).unwrap_or(measured)
    }

    pub fn set(&mut self, col: usize, w: u16) {
        if self.user.len() <= col {
            self.user.resize(col + 1, None);
        }
        self.user[col] = Some(w.max(self.min_width));
    }

    pub fn clear(&mut self, col: usize) {
        if let Some(u) = self.user.get_mut(col) {
            *u = None;
        }
    }

    pub fn reset(&mut self) {
        self.user.clear();
        self.drag = None;
    }

    pub fn is_default(&self) -> bool {
        self.user.iter().all(Option::is_none)
    }

    pub fn press(&mut self, col: usize, current: u16, x: u16) {
        self.drag = Some(SeamDrag {
            col,
            start_w: current,
            start_x: x,
        });
    }

    pub fn dragging(&self) -> Option<usize> {
        self.drag.map(|d| d.col)
    }

    pub fn drag_to(&mut self, x: u16) -> Option<(usize, u16)> {
        let d = self.drag?;
        let w = (i32::from(d.start_w) + i32::from(x) - i32::from(d.start_x))
            .clamp(i32::from(self.min_width), i32::from(u16::MAX)) as u16;
        if self.user(d.col) == Some(w) {
            return None;
        }
        self.set(d.col, w);
        Some((d.col, w))
    }

    pub fn release(&mut self) {
        self.drag = None;
    }

    pub fn apply(&self, cols: &[Col]) -> Vec<Col> {
        cols.iter()
            .enumerate()
            .map(|(i, c)| match self.user(i) {
                Some(w) => Col::fixed(w),
                None => *c,
            })
            .collect()
    }

    pub fn seam_rect(&self, col_right: u16, y: u16, gap: u16, limit: Rect) -> Rect {
        let x = (col_right + gap / 2).saturating_sub(self.seam_w / 2);
        Rect::new(x, y, self.seam_w.max(1), 1).intersection(limit)
    }
}

pub fn render_seam(
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    (owner, col): (WidgetId, usize),
    hit: Rect,
    glyph_x: u16,
    bg: Color,
    held: bool,
) {
    if hit.is_empty() {
        return;
    }
    let part = seam_part(col);
    let id = owner.part(part);
    let t = ctx.theme;
    let hovered = ctx.interaction.hovered(id);
    let pressed = held || ctx.interaction.pressed == Some(id);
    if (hovered || pressed) && buf.area.contains(Position::new(glyph_x, hit.y)) {
        let s = VisualState {
            hovered,
            pressed,
            ..Default::default()
        };
        let style = t.interactive(Kind::Seam, s, t.plane_of(bg)).style.bg(bg);

        let glyph = t.glyphs.scroll_thumb;
        buf.set_string(glyph_x, hit.y, glyph, style);
    }
    ctx.register_part(owner, part, hit);
}

pub fn seam_part(col: usize) -> crate::core::id::PartRef {
    crate::core::id::PartRef::item(
        crate::core::id::Part::SEAM,
        crate::core::id::ItemKey::index(col),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    SortAsc,
    SortDesc,
    ClearSort,

    Autofit,

    ResetWidths,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderLabels {
    pub sort_asc: &'static str,
    pub sort_desc: &'static str,
    pub clear_sort: &'static str,
    pub autofit: &'static str,
    pub reset_widths: &'static str,
}

impl HeaderLabels {
    pub const DEFAULT: Self = Self {
        sort_asc: "Sort Ascending",
        sort_desc: "Sort Descending",
        clear_sort: "Clear Sort",
        autofit: "Size Column to Fit",
        reset_widths: "Reset Column Widths",
    };
}

impl Default for HeaderLabels {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn header_menu(
    labels: &HeaderLabels,
    sortable: bool,
    sorted: Option<SortDir>,
    resizable: bool,
    has_user_widths: bool,
) -> Vec<(HeaderAction, MenuItem)> {
    let no_sort = (!sortable).then_some("This column does not sort");
    let item = |label: &str, off: Option<&str>| {
        let it = MenuItem::new(label);
        match off {
            Some(r) => it.reason(r),
            None => it,
        }
    };
    let mut v = vec![
        (
            HeaderAction::SortAsc,
            item(
                labels.sort_asc,
                no_sort.or((sorted == Some(SortDir::Asc)).then_some("Already sorted ascending")),
            ),
        ),
        (
            HeaderAction::SortDesc,
            item(
                labels.sort_desc,
                no_sort.or((sorted == Some(SortDir::Desc)).then_some("Already sorted descending")),
            ),
        ),
        (
            HeaderAction::ClearSort,
            item(
                labels.clear_sort,
                no_sort.or(sorted.is_none().then_some("Not sorted")),
            )
            .separator_if(resizable),
        ),
    ];
    if resizable {
        v.push((HeaderAction::Autofit, item(labels.autofit, None)));
        v.push((
            HeaderAction::ResetWidths,
            item(
                labels.reset_widths,
                (!has_user_widths).then_some("Columns have their default widths"),
            ),
        ));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_fits_at_its_ideal_and_flex_takes_the_rest() {
        let cols = [
            Col::content(4, 10),
            Col::fixed(6),
            Col::content(8, 20).flex(1, u16::MAX),
        ];
        assert_eq!(solve(&cols, 60, 2), vec![10, 6, 40]);
    }

    #[test]
    fn a_dropped_column_gives_its_space_to_the_others() {
        let cols = [
            Col::content(14, 30).flex(1, 30),
            Col::fixed(6),
            Col::content(12, 40).flex(1, u16::MAX).droppable(1),
        ];
        let w = solve(&cols, 40, 2);
        assert!(w[2] >= 12, "the reason still fits at its minimum: {w:?}");
        let w = solve(&cols, 33, 2);
        assert_eq!(w, vec![25, 6, 0], "dropped; the label takes its room");

        let cols = [
            Col::content(14, 20).max(30),
            Col::content(12, 12).flex(1, u16::MAX).droppable(1),
        ];
        assert_eq!(solve(&cols, 60, 2), vec![20, 38]);
        assert_eq!(solve(&cols, 27, 2), vec![27, 0]);
    }

    #[test]
    fn a_short_row_shares_the_shortfall_and_never_overflows() {
        let cols = [Col::content(2, 20), Col::content(2, 20)];
        assert_eq!(solve(&cols, 21, 1), vec![10, 10]);

        let cols = [Col::fixed(10), Col::fixed(10)];
        let w = solve(&cols, 15, 1);
        assert!(w.iter().sum::<u16>() < 15, "{w:?}");
    }

    #[test]
    fn the_lowest_priority_drops_first() {
        let cols = [
            Col::fixed(10),
            Col::fixed(10).droppable(5),
            Col::fixed(10).droppable(1),
        ];
        assert_eq!(solve(&cols, 21, 1), vec![10, 10, 0]);
        assert_eq!(solve(&cols, 15, 1), vec![10, 0, 0]);
    }

    #[test]
    fn user_widths_override_the_solver_and_reset() {
        let cols = [Col::content(4, 10), Col::content(4, 10).flex(1, 40)];
        let mut z = ColumnSizing::default();
        z.set(0, 20);
        let w = solve(&z.apply(&cols), 60, 2);
        assert_eq!(w[0], 20, "the user width is kept");
        z.set(0, 1);
        assert_eq!(z.user(0), Some(MIN_USER_WIDTH), "never below the minimum");
        z.reset();
        assert!(z.is_default());
        assert_eq!(solve(&z.apply(&cols), 60, 2)[0], 10);
    }

    #[test]
    fn a_seam_drag_keeps_its_grab_offset() {
        let mut z = ColumnSizing::default();

        z.press(2, 10, 31);
        assert_eq!(z.drag_to(31), Some((2, 10)), "no jump on the first move");
        assert_eq!(z.drag_to(35), Some((2, 14)));
        assert_eq!(z.drag_to(20), Some((2, MIN_USER_WIDTH)));
        z.release();
        assert_eq!(z.drag_to(40), None);
    }
}
