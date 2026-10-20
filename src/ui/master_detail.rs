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

pub const SPLIT_MIN: u16 = 108;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSize {
    Share { pct: u16, min: u16, max: u16 },

    MasterShare { pct: u16, min: u16, max: u16 },

    Resizable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Split,

    Master,

    Drawer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MdLayout {
    pub mode: Mode,
    pub master: Rect,
    pub detail: Rect,

    pub summary: Rect,

    pub seam: Rect,
}

#[derive(Debug, Clone)]
pub struct MasterDetail {
    pub master: WidgetId,

    pub detail: WidgetId,

    pub detail_owns: Vec<WidgetId>,

    pub size: DetailSize,

    pub split: Split,

    pub seam: Splitter,

    pub keys: SplitKeys,

    pub gap: u16,

    pub collapse_below: u16,

    pub summary_rows: u16,

    pub esc_returns: bool,
    drawer: bool,
    last: MdLayout,

    container: Rect,
}

impl MasterDetail {
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

    pub fn natural_width(&self, master: u16, detail: u16) -> u16 {
        self.panes_width(master, detail).max(self.collapse_below)
    }

    pub fn panes_width(&self, master: u16, detail: u16) -> u16 {
        match self.size {
            DetailSize::Resizable => self.split.natural_extent(master, detail, self.gap),
            DetailSize::Share { pct, min, max } => {
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
            DetailSize::MasterShare { pct, min, max } => {
                let master = master.min(max.max(min));
                let fits = |w: u16| {
                    let mw = (((w as u32 * pct as u32 + 50) / 100) as u16).clamp(min, max.max(min));
                    mw >= master && w.saturating_sub(mw + self.gap) >= detail
                };
                let mut w = master.saturating_add(detail).saturating_add(self.gap);
                while w < u16::MAX && !fits(w) {
                    w += 1;
                }
                w
            }
        }
    }

    pub fn split_rects(&self, area: Rect) -> Option<(Rect, Rect)> {
        if area.width < self.collapse_below {
            return None;
        }
        Some(match self.size {
            DetailSize::Share { pct, min, max } => {
                let dw = (area.width as u32 * pct as u32 / 100) as u16;
                let dw = dw.clamp(min, max.max(min));
                let master = Rect::new(
                    area.x,
                    area.y,
                    area.width.saturating_sub(dw + self.gap),
                    area.height,
                );
                (
                    master,
                    Rect::new(master.right() + self.gap, area.y, dw, area.height),
                )
            }
            DetailSize::MasterShare { pct, min, max } => {
                let mw = ((area.width as u32 * pct as u32 + 50) / 100) as u16;
                let mw = mw.clamp(min, max.max(min)).min(area.width);
                let dx = (mw + self.gap).min(area.width);
                (
                    Rect::new(area.x, area.y, mw, area.height),
                    Rect::new(area.x + dx, area.y, area.width - dx, area.height),
                )
            }
            DetailSize::Resizable => self.split.horizontal(area, self.gap),
        })
    }

    pub fn collapsed_mode(&self, focus: &crate::ui::ctx::Interaction) -> Mode {
        let open = self.drawer && !focus.focused(self.master);
        let detail_focused = focus.focus.is_some_and(|f| self.in_detail(f)) && !focus.focus_hidden;
        if detail_focused || open {
            Mode::Drawer
        } else {
            Mode::Master
        }
    }

    pub fn collapse_below(mut self, width: u16) -> Self {
        self.collapse_below = width;
        self
    }

    pub fn detail_share(mut self, pct: u16, min: u16, max: u16) -> Self {
        self.size = DetailSize::Share { pct, min, max };
        self
    }

    pub fn master_share(mut self, pct: u16, min: u16, max: u16) -> Self {
        self.size = DetailSize::MasterShare { pct, min, max };
        self
    }

    pub fn resizable(mut self, split: Split) -> Self {
        self.size = DetailSize::Resizable;
        self.split = split;
        self
    }

    pub fn seam(mut self, seam: Splitter) -> Self {
        self.seam = seam;
        self
    }

    pub fn keys(mut self, keys: SplitKeys) -> Self {
        self.keys = keys;
        self
    }

    pub fn gap(mut self, cols: u16) -> Self {
        self.gap = cols;
        self
    }

    pub fn summary_rows(mut self, rows: u16) -> Self {
        self.summary_rows = rows;
        self
    }

    pub fn esc_returns(mut self, on: bool) -> Self {
        self.esc_returns = on;
        self
    }

    pub fn detail_owns(mut self, ids: &[WidgetId]) -> Self {
        self.detail_owns = ids.to_vec();
        self
    }

    pub fn in_detail(&self, id: WidgetId) -> bool {
        id == self.detail || self.detail_owns.contains(&id)
    }

    pub fn drawer_open(&self) -> bool {
        self.drawer
    }

    pub fn last(&self) -> MdLayout {
        self.last
    }

    pub fn toggle_drawer(&mut self, focus: &mut Focus) {
        self.drawer = !self.drawer;
        focus.focus(if self.drawer {
            self.detail
        } else {
            self.master
        });
    }

    pub fn open_drawer(&mut self, focus: &mut Focus) {
        self.drawer = true;
        focus.focus(self.detail);
    }

    pub fn close_drawer(&mut self, focus: &mut Focus) {
        self.drawer = false;
        focus.focus(self.master);
    }

    pub fn menu_item(&self) -> MenuItem {
        MenuItem::new(if self.drawer {
            "Hide details"
        } else {
            "Show details"
        })
    }

    pub fn layout(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) -> MdLayout {
        let mut l = MdLayout::default();
        if let Some((master, detail)) = self.split_rects(area) {
            l.mode = Mode::Split;
            l.master = master;
            l.detail = detail;
            if self.size == DetailSize::Resizable {
                self.container = area;
                let handle = self.split.handle(SplitDir::Horizontal, area, self.gap);
                if !handle.is_empty() {
                    let off = (self.gap / 2).min(handle.width.saturating_sub(1));
                    l.seam = Rect::new(handle.x + off, handle.y, 1, handle.height);
                }
                self.seam.render(l.seam, buf, ctx, bg);
            }
        } else {
            self.drawer = self.collapsed_mode(&ctx.interaction) == Mode::Drawer;
            if self.drawer {
                l.mode = Mode::Drawer;
                l.detail = area;

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

    pub fn finish(&self, ctx: &mut RenderCtx) {
        if self.last.mode == Mode::Master {
            ctx.control(self.detail, Rect::ZERO, false);
        }
    }

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

    #[test]
    fn at_its_natural_width_both_panes_get_their_columns() {
        for mut md in [
            MasterDetail::new(M, D),
            MasterDetail::new(M, D).collapse_below(40),
            MasterDetail::new(M, D).detail_share(40, 20, 50),
            MasterDetail::new(M, D).master_share(33, 10, 40),
            MasterDetail::new(M, D)
                .resizable(Split::new(30, 10, 10))
                .gap(3),
        ] {
            for (m, d) in [(30, 44), (60, 20), (12, 12)] {
                let w = md.natural_width(m, d);
                let (l, _) = frame(&mut md, w, None);
                assert_eq!(l.mode, Mode::Split, "{w} for {m},{d}");

                let m = match md.size {
                    DetailSize::MasterShare { max, .. } => m.min(max),
                    _ => m,
                };
                assert!(l.master.width >= m, "master {:?} for {m}", l.master);
                let d = match md.size {
                    DetailSize::Share { max, .. } => d.min(max),
                    _ => d,
                };
                assert!(l.detail.width >= d, "detail {:?} for {d}", l.detail);
            }
        }
    }

    #[test]
    fn split_rects_are_the_panes_the_layout_draws() {
        for mut md in [
            MasterDetail::new(M, D),
            MasterDetail::new(M, D)
                .detail_share(40, 30, 44)
                .collapse_below(72),
            MasterDetail::new(M, D).master_share(33, 10, 40),
        ] {
            for width in [60, 72, 90, 108, 140] {
                let (l, _) = frame(&mut md, width, None);
                let rects = md.split_rects(Rect::new(0, 0, width, 20));
                match l.mode {
                    Mode::Split => assert_eq!(rects, Some((l.master, l.detail)), "{width}"),
                    _ => assert_eq!(rects, None, "{width}"),
                }
            }
        }
    }

    #[test]
    fn collapsed_mode_is_the_mode_the_narrow_layout_draws() {
        let mut md = MasterDetail::new(M, D);
        for focus in [None, Some(D), None, Some(M), None] {
            let i = Interaction {
                focus,
                ..Default::default()
            };
            let before = md.collapsed_mode(&i);
            let (l, _) = frame(&mut md, 80, focus);
            assert_eq!(before, l.mode, "{focus:?}");
        }
    }

    #[test]
    fn panes_width_leaves_a_higher_collapse_threshold_in_force() {
        let mut md = MasterDetail::new(M, D)
            .detail_share(40, 30, 44)
            .collapse_below(108);
        let w = md.panes_width(50, 30);
        assert!(w < 108, "{w}");
        assert_eq!(md.natural_width(50, 30), 108);
        let (l, _) = frame(&mut md, w, None);
        assert_eq!(l.mode, Mode::Master);
        md.collapse_below = 0;
        let (l, _) = frame(&mut md, w, None);
        assert_eq!(l.mode, Mode::Split);
        assert!(l.master.width >= 50 && l.detail.width >= 30, "{l:?}");
    }

    #[test]
    fn a_moved_seam_leaves_the_natural_width_as_it_was() {
        let mut md = MasterDetail::new(M, D).resizable(Split::new(30, 10, 10));
        let before = md.natural_width(30, 44);
        md.split.grow(-20);
        assert_eq!(md.natural_width(30, 44), before);
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

        assert_eq!(ring, vec![M, D]);
    }

    #[test]
    fn focusing_the_detail_opens_the_drawer_and_the_master_closes_it() {
        let mut md = MasterDetail::new(M, D);
        let (l, ring) = frame(&mut md, 80, Some(D));
        assert_eq!(l.mode, Mode::Drawer);
        assert_eq!(l.detail.width, 80);
        assert_eq!(ring, vec![M, D], "the master stays reachable");

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

        let mut fixed = MasterDetail::new(M, D).detail_share(34, 34, 48);
        let (l, _) = frame(&mut fixed, 120, None);
        assert_eq!(l.detail.width, 40);
        assert_eq!(l.master.width, 120 - 42);
        assert!(l.seam.is_empty());
        assert_eq!(fixed.on_key(&chord, &mut focus), None);
    }

    #[test]
    fn a_master_share_sizes_the_master_and_the_detail_takes_the_rest() {
        let mut md = MasterDetail::new(M, D)
            .master_share(33, 26, 40)
            .collapse_below(80);

        for (w, mw) in [
            (80, 26),
            (81, 27),
            (99, 33),
            (110, 36),
            (121, 40),
            (200, 40),
        ] {
            let (l, _) = frame(&mut md, w, None);
            assert_eq!(l.mode, Mode::Split, "{w}");
            assert_eq!(l.master.width, mw, "{w}");
            assert_eq!(l.detail.x, mw + 2, "{w}");
            assert_eq!(l.detail.right(), w, "{w}");
            assert!(l.seam.is_empty());
        }
        let (l, _) = frame(&mut md, 79, None);
        assert_eq!(l.mode, Mode::Master);
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
