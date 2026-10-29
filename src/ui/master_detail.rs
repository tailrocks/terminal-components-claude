//! A list beside the details of its current row, responsive (S19).
//!
//! Wide, the master (a list, a tree, a table) and its detail sit side by
//! side; the seam between them is either fixed (the detail takes a share
//! of the width, clamped) or a draggable [`Splitter`] that the panes also
//! resize from the keyboard ([`SplitKeys`], K18). Narrow (below
//! [`MasterDetail::collapse_below`]), the master takes the whole body
//! with an optional summary of the current row under it, and the detail
//! becomes a drawer that covers the body: focusing the detail (Tab, a
//! click, [`MasterDetail::toggle_drawer`]) opens it, focusing the master
//! or Esc closes it. The pane that is not drawn stays a focus stop, so
//! Tab and Shift+Tab still reach it, as macOS keeps a collapsed split
//! view's sidebar one command away.
//!
//! The owner draws the panes: [`MasterDetail::layout`] decides the mode
//! and returns the rects, the owner renders its master, detail and summary
//! into them, then calls [`MasterDetail::finish`]. Two calls (not one with
//! closures) keep the focus order master → detail and let the panes borrow
//! the owner freely.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::focus::Focus;
use crate::core::id::WidgetId;
use crate::core::intent::Phase;
use crate::core::pointer::PointerEvent;
use crate::ui::ctx::RenderCtx;
use crate::ui::layout::{Split, SplitDir, SplitKeys};
use crate::widgets::keyhint::{Hint, hint};
use crate::widgets::menu::MenuItem;
use crate::widgets::splitter::Splitter;

/// The width below which a master-detail collapses to one pane: the
/// shell's 120-column default minus its margins keeps both panes, an
/// 80- or 100-column terminal does not.
pub const SPLIT_MIN: u16 = 108;

/// How the detail pane is sized while both panes show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSize {
    /// The detail takes `pct` percent of the width, clamped to
    /// `min..=max` columns; the seam does not move.
    Share { pct: u16, min: u16, max: u16 },
    /// The seam is a draggable [`Splitter`]; [`MasterDetail::split`]
    /// holds the master's share and both minimums, and
    /// [`MasterDetail::keys`] resizes it from either pane.
    Resizable,
}

/// What a [`MasterDetail`] shows this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Both panes, side by side.
    #[default]
    Split,
    /// Narrow: the master (and the summary under it).
    Master,
    /// Narrow: the detail drawer covers the body.
    Drawer,
}

/// Where a [`MasterDetail`] placed its panes. A pane that is not drawn
/// this frame has an empty rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MdLayout {
    pub mode: Mode,
    pub master: Rect,
    pub detail: Rect,
    /// Narrow only: the summary of the current row under the master.
    pub summary: Rect,
    /// The drawn seam (resizable split only).
    pub seam: Rect,
}

/// A responsive master-detail layout. See the module docs.
#[derive(Debug, Clone)]
pub struct MasterDetail {
    /// The master pane's focus id.
    pub master: WidgetId,
    /// The detail pane's focus id.
    pub detail: WidgetId,
    /// Further focus stops inside the detail (its action buttons): focus
    /// on any of them keeps the drawer open, like focus on the detail.
    pub detail_owns: Vec<WidgetId>,
    /// How the detail is sized while both show. Default a resizable seam.
    pub size: DetailSize,
    /// The master's share and the panes' minimums ([`DetailSize::Resizable`]).
    pub split: Split,
    /// The seam handle ([`DetailSize::Resizable`]).
    pub seam: Splitter,
    /// Keyboard resize from either pane. Default [`SplitKeys::default`].
    pub keys: SplitKeys,
    /// Columns between the panes; a drawn seam sits in the middle one.
    /// Default 2.
    pub gap: u16,
    /// Below this width the layout collapses to one pane. Default
    /// [`SPLIT_MIN`].
    pub collapse_below: u16,
    /// Rows of the narrow summary under the master (at most a third of
    /// the body), after one blank row. Default 0: no summary.
    pub summary_rows: u16,
    /// Esc on the detail returns focus to the master (and closes the
    /// drawer). Default on.
    pub esc_returns: bool,
    drawer: bool,
    last: MdLayout,
    /// The rect the split was laid out in, for the seam's drags.
    container: Rect,
}

impl MasterDetail {
    /// A resizable master-detail: the master takes 40 % (at least 28
    /// columns), the detail at least 40.
    pub fn new(master: WidgetId, detail: WidgetId) -> Self {
        Self {
            master,
            detail,
            detail_owns: Vec::new(),
            size: DetailSize::Resizable,
            split: Split::new(40, 28, 40),
            seam: Splitter::new(master.sub("seam"), SplitDir::Horizontal),
            keys: SplitKeys::default(),
            gap: 2,
            collapse_below: SPLIT_MIN,
            summary_rows: 0,
            esc_returns: true,
            drawer: false,
            last: MdLayout::default(),
            container: Rect::ZERO,
        }
    }

    /// The natural width of the layout (plan §1.4 `Measure`): the fewest
    /// columns that show both panes side by side with the master at least
    /// `master` and the detail at least `detail` columns wide, at the
    /// detail's share or the split's starting ratio and minima (a dragged
    /// seam does not change it). Below [`Self::collapse_below`] the layout
    /// still collapses: that threshold is the owner's choice.
    pub fn natural_width(&self, master: u16, detail: u16) -> u16 {
        let both = match self.size {
            // at the starting ratio: a dragged seam does not change it
            DetailSize::Resizable => {
                let mut split = self.split;
                split.reset();
                split.extent_for(master, detail, self.gap)
            }
            DetailSize::Share { pct, min, max } => {
                // the share never draws the detail wider than its max
                let detail = detail.min(max.max(min));
                let fits = |w: u16| {
                    let dw = ((w as u32 * pct as u32 / 100) as u16).clamp(min, max.max(min));
                    dw >= detail && w.saturating_sub(dw + self.gap) >= master
                };
                let mut w = master.saturating_add(detail).saturating_add(self.gap);
                while w < u16::MAX && !fits(w) {
                    w += 1;
                }
                w
            }
        };
        both
    }

    /// See [`Self::collapse_below`].
    pub fn collapse_below(mut self, width: u16) -> Self {
        self.collapse_below = width;
        self
    }

    /// A fixed seam: the detail takes `pct` percent of the width, clamped
    /// to `min..=max` columns ([`DetailSize::Share`]).
    pub fn detail_share(mut self, pct: u16, min: u16, max: u16) -> Self {
        self.size = DetailSize::Share { pct, min, max };
        self
    }

    /// A draggable seam with this split ([`DetailSize::Resizable`]).
    pub fn resizable(mut self, split: Split) -> Self {
        self.size = DetailSize::Resizable;
        self.split = split;
        self
    }

    /// See [`Self::seam`].
    pub fn seam(mut self, seam: Splitter) -> Self {
        self.seam = seam;
        self
    }

    /// See [`Self::keys`].
    pub fn keys(mut self, keys: SplitKeys) -> Self {
        self.keys = keys;
        self
    }

    /// See [`Self::gap`].
    pub fn gap(mut self, cols: u16) -> Self {
        self.gap = cols;
        self
    }

    /// See [`Self::summary_rows`].
    pub fn summary_rows(mut self, rows: u16) -> Self {
        self.summary_rows = rows;
        self
    }

    /// See [`Self::esc_returns`].
    pub fn esc_returns(mut self, on: bool) -> Self {
        self.esc_returns = on;
        self
    }

    /// See [`Self::detail_owns`].
    pub fn detail_owns(mut self, ids: &[WidgetId]) -> Self {
        self.detail_owns = ids.to_vec();
        self
    }

    /// Whether `id` is the detail or a stop inside it.
    pub fn in_detail(&self, id: WidgetId) -> bool {
        id == self.detail || self.detail_owns.contains(&id)
    }

    /// Whether the narrow drawer is open (it stays open across frames
    /// until the master takes focus, Esc, or [`Self::toggle_drawer`]).
    pub fn drawer_open(&self) -> bool {
        self.drawer
    }

    /// The layout of the last frame.
    pub fn last(&self) -> MdLayout {
        self.last
    }

    /// Open or close the narrow drawer and move focus with it (the
    /// "Show details" command). Wide, it only moves focus between the
    /// panes.
    pub fn toggle_drawer(&mut self, focus: &mut Focus) {
        self.drawer = !self.drawer;
        focus.focus(if self.drawer {
            self.detail
        } else {
            self.master
        });
    }

    /// Open the drawer (narrow) and focus the detail: a master row whose
    /// "open" shows its details.
    pub fn open_drawer(&mut self, focus: &mut Focus) {
        self.drawer = true;
        focus.focus(self.detail);
    }

    /// Close the drawer and give focus back to the master.
    pub fn close_drawer(&mut self, focus: &mut Focus) {
        self.drawer = false;
        focus.focus(self.master);
    }

    /// The command that toggles the details, for an app's menu (its
    /// shortcut is the app's: add it with [`MenuItem::shortcut`]).
    pub fn menu_item(&self) -> MenuItem {
        MenuItem::new(if self.drawer {
            "Hide details"
        } else {
            "Show details"
        })
    }

    /// Decide this frame's mode and place the panes in `area`. Draws the
    /// seam of a resizable split (on `bg`); in the drawer registers the
    /// master's hidden focus stop. Render the panes into the returned
    /// rects, then call [`Self::finish`].
    pub fn layout(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) -> MdLayout {
        let mut l = MdLayout::default();
        if area.width >= self.collapse_below {
            l.mode = Mode::Split;
            match self.size {
                DetailSize::Share { pct, min, max } => {
                    let dw = (area.width as u32 * pct as u32 / 100) as u16;
                    let dw = dw.clamp(min, max.max(min));
                    l.master = Rect::new(
                        area.x,
                        area.y,
                        area.width.saturating_sub(dw + self.gap),
                        area.height,
                    );
                    l.detail = Rect::new(l.master.right() + self.gap, area.y, dw, area.height);
                }
                DetailSize::Resizable => {
                    self.container = area;
                    let (m, d) = self.split.horizontal(area, self.gap);
                    l.master = m;
                    l.detail = d;
                    let handle = self.split.handle(SplitDir::Horizontal, area, self.gap);
                    if !handle.is_empty() {
                        let off = (self.gap / 2).min(handle.width.saturating_sub(1));
                        l.seam = Rect::new(handle.x + off, handle.y, 1, handle.height);
                    }
                    self.seam.render(l.seam, buf, ctx, bg);
                }
            }
        } else {
            let focus = ctx.interaction;
            if focus.focused(self.master) {
                // focus returned to the master: the drawer gives the body back
                self.drawer = false;
            }
            let detail_focused =
                focus.focus.is_some_and(|f| self.in_detail(f)) && !focus.focus_hidden;
            if detail_focused || self.drawer {
                self.drawer = true;
                l.mode = Mode::Drawer;
                l.detail = area;
                // the master keeps its focus stop so Shift+Tab still reaches it
                ctx.control(self.master, Rect::ZERO, false);
            } else {
                l.mode = Mode::Master;
                let rows = self.summary_rows.min(area.height / 3);
                if rows == 0 {
                    l.master = area;
                } else {
                    l.master = Rect::new(
                        area.x,
                        area.y,
                        area.width,
                        area.height.saturating_sub(rows + 1),
                    );
                    l.summary = Rect::new(area.x, l.master.bottom() + 1, area.width, rows);
                }
            }
        }
        self.last = l;
        l
    }

    /// After the panes are drawn: in the narrow master view, register the
    /// detail's hidden focus stop (after the master, so Tab reaches it
    /// and opens the drawer).
    pub fn finish(&self, ctx: &mut RenderCtx) {
        if self.last.mode == Mode::Master {
            ctx.control(self.detail, Rect::ZERO, false);
        }
    }

    /// The master-detail's own keys while `focus` is on one of its panes:
    /// the resize chords ([`Self::keys`]) on a resizable split, and Esc
    /// on the detail, which returns to the master. `None` for any other
    /// key: the pane has it.
    pub fn on_key(&mut self, key: &Key, focus: &mut Focus) -> Option<Outcome> {
        let f = focus.current()?;
        let on_master = f == self.master;
        let on_detail = self.in_detail(f);
        if !on_master && !on_detail {
            return None;
        }
        if self.last.mode == Mode::Split && self.size == DetailSize::Resizable {
            let (container, gap) = (self.container, self.gap);
            if let Some(o) =
                self.keys
                    .apply(key, &mut self.split, SplitDir::Horizontal, container, gap)
            {
                return Some(o);
            }
        }
        if on_detail && self.esc_returns && key.code == KeyCode::Esc && key.mods.is_empty() {
            self.close_drawer(focus);
            return Some(Outcome::Changed);
        }
        None
    }

    /// A press, drag or click on the seam: it drags cell by cell and a
    /// double-click restores the starting ratio. `None` when the event is
    /// not the seam's.
    pub fn on_pointer(&mut self, ev: &PointerEvent) -> Option<Outcome> {
        if ev.id != self.seam.id || self.size != DetailSize::Resizable {
            return None;
        }
        let (container, gap) = (self.container, self.gap);
        match ev.phase()? {
            Phase::Press => {
                self.seam.on_press(&self.split, container, gap, ev.pos());
                Some(Outcome::Consumed)
            }
            Phase::Drag => Some(self.seam.on_drag(&mut self.split, container, gap, ev.pos())),
            Phase::Click => Some(self.seam.on_click(&mut self.split, ev.count())),
            _ => None,
        }
    }

    /// The footer hints for the master-detail's own keys while `focus` is
    /// on one of its panes (add them after the pane's own).
    pub fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        let Some(f) = focus else { return Vec::new() };
        let on_detail = self.in_detail(f);
        if f != self.master && !on_detail {
            return Vec::new();
        }
        let mut v = Vec::new();
        if self.last.mode == Mode::Split
            && self.size == DetailSize::Resizable
            && let Some(h) = self.keys.hint(SplitDir::Horizontal)
        {
            v.push(h);
        }
        if on_detail && self.esc_returns {
            v.push(hint("Esc", "Back"));
        }
        v
    }
}

/// In a States grid: both panes with the seam at rest, under the pointer
/// and pressed, and the narrow master with its summary.
impl crate::ui::specimen::Specimen for MasterDetail {
    fn specimen_id(&self) -> WidgetId {
        self.seam.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Hover | Pressed | Narrow)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        3
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState;
        let t = ctx.theme;
        let mut md = self.clone();
        md.summary_rows = 1;
        md.collapse_below = if state == SpecimenState::Narrow {
            u16::MAX
        } else {
            0
        };
        md.split = Split::new(45, 4, 4);
        md.size = DetailSize::Resizable;
        md.gap = 1;
        let l = ctx.visual_order(|ctx| {
            let inert = std::mem::replace(&mut ctx.inert, true);
            let l = md.layout(area, buf, ctx, bg);
            ctx.inert = inert;
            l
        });
        let line = |buf: &mut Buffer, r: Rect, text: &str, style| {
            if !r.is_empty() {
                buf.set_string(
                    r.x,
                    r.y,
                    crate::ui::text::truncate(text, r.width as usize),
                    style,
                );
            }
        };
        let rows = ["▸ orders", "  users", "  events"];
        for (i, row) in rows.iter().enumerate() {
            let r = l.master;
            if (i as u16) < r.height {
                line(
                    buf,
                    Rect::new(r.x, r.y + i as u16, r.width, 1),
                    row,
                    t.secondary().bg(bg),
                );
            }
        }
        let facts = ["rows  12,408", "size  3.1 MB"];
        for (i, fact) in facts.iter().enumerate() {
            let r = l.detail;
            if (i as u16) < r.height {
                line(
                    buf,
                    Rect::new(r.x + 1, r.y + i as u16, r.width.saturating_sub(1), 1),
                    fact,
                    t.muted().bg(bg),
                );
            }
        }
        line(buf, l.summary, "→ orders · 12,408 rows", t.muted().bg(bg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    const M: WidgetId = WidgetId::of("md.master");
    const D: WidgetId = WidgetId::of("md.detail");

    fn frame(
        md: &mut MasterDetail,
        width: u16,
        focus: Option<WidgetId>,
    ) -> (MdLayout, Vec<WidgetId>) {
        let t = Theme::for_level(ColorLevel::TrueColor);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = Interaction {
            focus,
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
        let area = Rect::new(0, 0, width, 20);
        let mut buf = Buffer::empty(area);
        let l = md.layout(area, &mut buf, &mut ctx, t.canvas);
        if !l.master.is_empty() {
            ctx.control(md.master, l.master, false);
        }
        if !l.detail.is_empty() {
            ctx.control(md.detail, l.detail, false);
        }
        md.finish(&mut ctx);
        drop(ctx);
        (l, ring.reachable().to_vec())
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    /// Laid out at its natural width (and not under its collapse
    /// threshold), the layout shows both panes at least as wide as asked,
    /// resizable or shared.
    #[test]
    fn at_its_natural_width_both_panes_get_their_columns() {
        for mut md in [
            MasterDetail::new(M, D),
            MasterDetail::new(M, D).collapse_below(40),
            MasterDetail::new(M, D).detail_share(40, 20, 50),
            MasterDetail::new(M, D)
                .resizable(Split::new(30, 10, 10))
                .gap(3),
        ] {
            for (m, d) in [(30, 44), (60, 20), (12, 12)] {
                let w = md.natural_width(m, d).max(md.collapse_below);
                let (l, _) = frame(&mut md, w, None);
                assert_eq!(l.mode, Mode::Split, "{w} for {m},{d}");
                assert!(l.master.width >= m, "master {:?} for {m}", l.master);
                // a share draws the detail at most its max
                let d = match md.size {
                    DetailSize::Share { max, .. } => d.min(max),
                    DetailSize::Resizable => d,
                };
                assert!(l.detail.width >= d, "detail {:?} for {d}", l.detail);
            }
        }
    }

    #[test]
    fn wide_shows_both_panes_and_narrow_collapses_to_the_master() {
        let mut md = MasterDetail::new(M, D).summary_rows(1);
        let (wide, ring) = frame(&mut md, 120, None);
        assert_eq!(wide.mode, Mode::Split);
        assert!(!wide.master.is_empty() && !wide.detail.is_empty());
        assert_eq!(wide.seam.width, 1, "a resizable split draws its seam");
        assert_eq!(ring, vec![M, D]);
        let (narrow, ring) = frame(&mut md, 80, None);
        assert_eq!(narrow.mode, Mode::Master);
        assert!(narrow.detail.is_empty());
        assert_eq!(narrow.summary.height, 1);
        assert_eq!(narrow.master.height, 18, "summary and its blank row");
        // the hidden detail is still the next stop after the master
        assert_eq!(ring, vec![M, D]);
    }

    #[test]
    fn focusing_the_detail_opens_the_drawer_and_the_master_closes_it() {
        let mut md = MasterDetail::new(M, D);
        let (l, ring) = frame(&mut md, 80, Some(D));
        assert_eq!(l.mode, Mode::Drawer);
        assert_eq!(l.detail.width, 80);
        assert_eq!(ring, vec![M, D], "the master stays reachable");
        // sticky while focus is elsewhere
        let (l, _) = frame(&mut md, 80, None);
        assert_eq!(l.mode, Mode::Drawer);
        let (l, _) = frame(&mut md, 80, Some(M));
        assert_eq!(l.mode, Mode::Master);
        assert!(!md.drawer_open());
    }

    #[test]
    fn esc_on_the_detail_returns_to_the_master() {
        let mut md = MasterDetail::new(M, D);
        let mut focus = Focus::default();
        md.toggle_drawer(&mut focus);
        assert!(focus.is(D) && md.drawer_open());
        let o = md.on_key(&key(KeyCode::Esc, KeyModifiers::NONE), &mut focus);
        assert_eq!(o, Some(Outcome::Changed));
        assert!(focus.is(M) && !md.drawer_open());
        // Esc on the master is the master's
        assert_eq!(
            md.on_key(&key(KeyCode::Esc, KeyModifiers::NONE), &mut focus),
            None
        );
        let mut off = MasterDetail::new(M, D).esc_returns(false);
        focus.focus(D);
        assert_eq!(
            off.on_key(&key(KeyCode::Esc, KeyModifiers::NONE), &mut focus),
            None
        );
    }

    #[test]
    fn the_resize_chords_move_the_seam_from_either_pane() {
        // Alt chords delivered (a Linux client, WezTerm): the hints this
        // test reads do not depend on the host terminal (A29)
        crate::core::keyboard::set_alt_keys(crate::core::keyboard::AltKeys::Delivered);
        let mut md = MasterDetail::new(M, D);
        let (l, _) = frame(&mut md, 120, None);
        let mut focus = Focus::default();
        focus.focus(D);
        let chord = key(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT);
        assert_eq!(md.on_key(&chord, &mut focus), Some(Outcome::Changed));
        let (after, _) = frame(&mut md, 120, None);
        assert_eq!(after.master.width, l.master.width + 2);
        assert_eq!(md.hints(Some(M))[0].key, "Alt+Shift+←→");
        // a fixed share has no resize chord
        let mut fixed = MasterDetail::new(M, D).detail_share(34, 34, 48);
        let (l, _) = frame(&mut fixed, 120, None);
        assert_eq!(l.detail.width, 40);
        assert_eq!(l.master.width, 120 - 42);
        assert!(l.seam.is_empty());
        assert_eq!(fixed.on_key(&chord, &mut focus), None);
    }

    #[test]
    fn the_seam_drags_and_resets() {
        use crate::core::pointer::PointerEvent;
        let mut md = MasterDetail::new(M, D);
        let (l, _) = frame(&mut md, 120, None);
        let seam = md.seam.id;
        let x = l.seam.x;
        let press = PointerEvent::new(
            &HitRegistry::default(),
            Phase::Press,
            seam,
            ratatui::layout::Position::new(x, 3),
            1,
            KeyModifiers::NONE,
        );
        assert!(md.on_pointer(&press).is_some());
        let drag = PointerEvent::new(
            &HitRegistry::default(),
            Phase::Drag,
            seam,
            ratatui::layout::Position::new(x + 6, 3),
            1,
            KeyModifiers::NONE,
        );
        assert_eq!(md.on_pointer(&drag), Some(Outcome::Changed));
        let (moved, _) = frame(&mut md, 120, None);
        assert_eq!(moved.master.width, l.master.width + 6);
        // not the seam: not the master-detail's
        let other = PointerEvent::new(
            &HitRegistry::default(),
            Phase::Click,
            M,
            ratatui::layout::Position::new(1, 1),
            1,
            KeyModifiers::NONE,
        );
        assert_eq!(md.on_pointer(&other), None);
    }
}
