//! Column width solver: one rule for laying out the columns of a row.
//!
//! Each [`Col`] states the narrowest width it is useful at (`min`), the
//! width its content measures (`ideal`), the widest it may grow (`max`), a
//! `flex` weight for spare space and a `drop` priority. [`solve`] gives
//! every column its `min`, drops columns (lowest `drop` first) while even
//! the minimums do not fit, grows the survivors toward their `ideal`, and
//! hands what is left to the flexible columns up to their `max`; anything
//! still left goes to any column below its `max`, in order. Space a dropped
//! or short column does not use is redistributed, never left blank.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::{RenderCtx, VisualState};
use crate::widgets::menu::MenuItem;
use crate::widgets::table::SortDir;

/// One column's sizing rule. Build it from measured content with
/// [`Col::content`], or [`Col::fixed`] for a column that is exactly as wide
/// as it says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Col {
    /// Narrowest useful width; below it the column is dropped instead.
    pub min: u16,
    /// What the content measures: reached before any column flexes.
    pub ideal: u16,
    /// Widest the column may grow. `u16::MAX`: unbounded.
    pub max: u16,
    /// Share of the space left once every column has its ideal. 0: never
    /// grows past `ideal`.
    pub flex: u16,
    /// Lower drops first when the minimums do not fit. `None`: never drops
    /// (it is squeezed below `min` only when nothing else can give way).
    pub drop: Option<u8>,
}

impl Col {
    /// A column sized by its content: `min` cells at least, the measured
    /// width ideally, no wider than that unless given flex.
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

    /// Exactly `w` cells, never dropped.
    pub const fn fixed(w: u16) -> Self {
        Self::content(w, w)
    }

    /// Grow into spare space with weight `weight`, up to `max`.
    pub const fn flex(mut self, weight: u16, max: u16) -> Self {
        self.flex = weight;
        self.max = if max > self.ideal { max } else { self.ideal };
        self
    }

    /// Allow the column to grow up to `max` with whatever is left after
    /// the flexible columns (for example a label capped at a share of the
    /// row that takes back the room of a dropped neighbour).
    pub const fn max(mut self, max: u16) -> Self {
        self.max = if max > self.ideal { max } else { self.ideal };
        self
    }

    /// Let the column drop when the row is short; lower drops first.
    pub const fn droppable(mut self, priority: u8) -> Self {
        self.drop = Some(priority);
        self
    }
}

/// Widths for `cols` in `avail` cells with `gap` cells between visible
/// columns. A dropped column gets 0 (and no gap). The result never exceeds
/// `avail`.
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
    // drop the least important columns until the minimums fit
    while mins(&visible) + gaps(&visible) > avail as u32 {
        let victim = cols
            .iter()
            .enumerate()
            .filter(|(i, c)| visible[*i] && c.drop.is_some())
            // lowest priority first, the rightmost on ties
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
    // nothing can drop: squeeze from the right so the row still fits
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
    // grow toward the ideal, one cell at a time in turn, so a short row
    // shares the shortfall instead of starving the last column
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
    // what is left goes to the flexible columns by weight
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
    // anything still left: columns below their max, in order
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

// ---- user widths and header seams ---------------------------------------

/// Narrowest width a user can drag a column to (cells).
pub const MIN_USER_WIDTH: u16 = 3;

/// Default width of a header seam's hit area (cells): the two-cell gap
/// between grid and table columns.
pub const DEFAULT_SEAM_W: u16 = 2;

/// Column widths a user set by dragging a header seam (or autofit),
/// layered over the widths a widget measures from its content. A column
/// with no user width keeps its measured width; [`Self::reset`] forgets
/// every user width. The drag keeps its grab offset: the seam moves by
/// exactly the cells the pointer moved since the press, so it never jumps
/// to snap under the pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSizing {
    user: Vec<Option<u16>>,
    drag: Option<SeamDrag>,
    /// Header seams take presses and drags. Default `true`; off, headers
    /// keep only their sort clicks.
    pub resizable: bool,
    /// Hit width of a seam, centred on the gap after a column. Default
    /// [`DEFAULT_SEAM_W`].
    pub seam_w: u16,
    /// Narrowest width a drag may leave. Default [`MIN_USER_WIDTH`].
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
    /// The user width of `col`, if one was set.
    pub fn user(&self, col: usize) -> Option<u16> {
        self.user.get(col).copied().flatten()
    }

    /// The width `col` draws at: its user width, else `measured`.
    pub fn width(&self, col: usize, measured: u16) -> u16 {
        self.user(col).unwrap_or(measured)
    }

    /// Set `col`'s user width (at least [`Self::min_width`]).
    pub fn set(&mut self, col: usize, w: u16) {
        if self.user.len() <= col {
            self.user.resize(col + 1, None);
        }
        self.user[col] = Some(w.max(self.min_width));
    }

    /// Forget `col`'s user width.
    pub fn clear(&mut self, col: usize) {
        if let Some(u) = self.user.get_mut(col) {
            *u = None;
        }
    }

    /// Forget every user width.
    pub fn reset(&mut self) {
        self.user.clear();
        self.drag = None;
    }

    /// No column has a user width.
    pub fn is_default(&self) -> bool {
        self.user.iter().all(Option::is_none)
    }

    /// A press on the seam after `col`, drawn `current` cells wide, at
    /// pointer column `x`: the drag that follows resizes it.
    pub fn press(&mut self, col: usize, current: u16, x: u16) {
        self.drag = Some(SeamDrag {
            col,
            start_w: current,
            start_x: x,
        });
    }

    /// The column a seam drag is resizing.
    pub fn dragging(&self) -> Option<usize> {
        self.drag.map(|d| d.col)
    }

    /// The pointer moved to column `x` during a seam drag: the column's new
    /// width (its press width plus the cells moved, never below
    /// [`Self::min_width`]). `None` without a drag or when the width did
    /// not change.
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

    /// The seam drag ended.
    pub fn release(&mut self) {
        self.drag = None;
    }

    /// `cols` with every user width applied: a user-sized column is exactly
    /// that wide and never dropped, so [`solve`] honours it.
    pub fn apply(&self, cols: &[Col]) -> Vec<Col> {
        cols.iter()
            .enumerate()
            .map(|(i, c)| match self.user(i) {
                Some(w) => Col::fixed(w),
                None => *c,
            })
            .collect()
    }

    /// The seam hit area after a column ending at `col_right` (exclusive)
    /// on row `y`, with `gap` cells before the next column, clipped to
    /// `limit`.
    pub fn seam_rect(&self, col_right: u16, y: u16, gap: u16, limit: Rect) -> Rect {
        let x = (col_right + gap / 2).saturating_sub(self.seam_w / 2);
        Rect::new(x, y, self.seam_w.max(1), 1).intersection(limit)
    }
}

/// Draw a header seam and register its hit area. At rest it is invisible
/// (the header keeps its look); under the pointer it shows a rule in the
/// seam ink, and a heavy rule while held, so rest, hover and pressed
/// differ.
pub fn render_seam(
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    id: WidgetId,
    hit: Rect,
    glyph_x: u16,
    bg: Color,
    held: bool,
) {
    if hit.is_empty() {
        return;
    }
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
        let glyph = if pressed {
            t.glyphs.scroll_thumb
        } else {
            t.glyphs.divider
        };
        buf.set_string(glyph_x, hit.y, glyph, style);
    }
    ctx.clickable(id, hit);
}

/// A command of a column header's context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    SortAsc,
    SortDesc,
    ClearSort,
    /// Fit the column to its header and visible values.
    Autofit,
    /// Forget every user width.
    ResetWidths,
}

/// The labels of a header menu, overridable per app (copy, language).
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

/// The header menu for a column: sort commands (disabled with a reason
/// when the column does not sort, or already sorts that way), then the
/// sizing commands. Each item pairs with the action it runs.
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
        // label ideal 30, reason droppable: at 40 cells the reason (min 12)
        // does not fit beside a full label, so it drops and the label gets
        // its ideal instead of leaving cells blank
        let cols = [
            Col::content(14, 30).flex(1, 30),
            Col::fixed(6),
            Col::content(12, 40).flex(1, u16::MAX).droppable(1),
        ];
        let w = solve(&cols, 40, 2);
        assert!(w[2] >= 12, "the reason still fits at its minimum: {w:?}");
        let w = solve(&cols, 33, 2);
        assert_eq!(w, vec![25, 6, 0], "dropped; the label takes its room");
        // a label capped at 20 with room to 30: the flexible reason takes
        // the spare while it shows; once it drops the label takes it back
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
        // nothing droppable and the minimums do not fit: squeezed, in bounds
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
        // pressed one cell right of the column edge
        z.press(2, 10, 31);
        assert_eq!(z.drag_to(31), Some((2, 10)), "no jump on the first move");
        assert_eq!(z.drag_to(35), Some((2, 14)));
        assert_eq!(z.drag_to(20), Some((2, MIN_USER_WIDTH)));
        z.release();
        assert_eq!(z.drag_to(40), None);
    }
}
