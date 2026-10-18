use ratatui::layout::{Position, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDir {
    Horizontal,

    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maximized {
    None,
    First,
    Second,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub percent: u16,
    pub min_first: u16,
    pub min_second: u16,
    pub maximized: Maximized,

    pub default_percent: u16,

    precise: Option<(u16, u16)>,
}

impl Split {
    pub const fn new(percent: u16, min_first: u16, min_second: u16) -> Self {
        Self {
            percent,
            min_first,
            min_second,
            maximized: Maximized::None,
            default_percent: percent,
            precise: None,
        }
    }

    pub fn reset(&mut self) {
        self.percent = self.default_percent;
        self.precise = None;
        self.maximized = Maximized::None;
    }

    fn first_of(&self, usable: u16) -> u16 {
        match self.precise {
            Some((p, bps)) if p == self.percent => (usable as u32 * bps as u32 / 10_000) as u16,
            _ => (usable as u32 * self.percent as u32 / 100) as u16,
        }
    }

    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }

    pub fn grow(&mut self, delta: i16) {
        self.percent = (self.percent as i16 + delta).clamp(5, 95) as u16;
    }

    pub fn layout(&self, dir: SplitDir, area: Rect, gap: u16) -> (Rect, Rect) {
        match dir {
            SplitDir::Horizontal => self.horizontal(area, gap),
            SplitDir::Vertical => self.vertical(area, gap),
        }
    }

    pub fn handle(&self, dir: SplitDir, area: Rect, gap: u16) -> Rect {
        let (a, b) = self.layout(dir, area, gap);
        if a.is_empty() || b.is_empty() || gap == 0 {
            return Rect::ZERO;
        }
        match dir {
            SplitDir::Horizontal => Rect::new(a.right(), area.y, gap, area.height),
            SplitDir::Vertical => Rect::new(area.x, a.bottom(), area.width, gap),
        }
    }

    pub fn drag_to(&mut self, dir: SplitDir, area: Rect, gap: u16, pos: Position) -> bool {
        let (usable, offset) = match dir {
            SplitDir::Horizontal => (area.width.saturating_sub(gap), pos.x.saturating_sub(area.x)),
            SplitDir::Vertical => (
                area.height.saturating_sub(gap),
                pos.y.saturating_sub(area.y),
            ),
        };
        if usable < self.min_first + self.min_second || usable == 0 {
            return false;
        }
        let first = offset.clamp(self.min_first, usable - self.min_second);

        let bps = (first as u32 * 10_000)
            .div_ceil(usable as u32)
            .clamp(500, 9_500) as u16;
        let percent = ((bps as u32 + 50) / 100) as u16;
        let before = self.first_of(usable);
        self.percent = percent;
        self.precise = Some((percent, bps));
        self.first_of(usable) != before
    }

    pub fn nudge(&mut self, dir: SplitDir, area: Rect, gap: u16, delta: i16) {
        let (first, _) = self.layout(dir, area, gap);
        let cur = match dir {
            SplitDir::Horizontal => first.width,
            SplitDir::Vertical => first.height,
        } as i32;
        let target = (cur + delta as i32).max(0) as u16;
        let pos = match dir {
            SplitDir::Horizontal => Position::new(area.x + target, area.y),
            SplitDir::Vertical => Position::new(area.x, area.y + target),
        };
        self.drag_to(dir, area, gap, pos);
    }

    pub fn natural_extent(&self, first: u16, second: u16, gap: u16) -> u16 {
        let start = Self {
            percent: self.default_percent,
            precise: None,
            ..*self
        };
        start.extent_for(first, second, gap)
    }

    pub fn extent_for(&self, first: u16, second: u16, gap: u16) -> u16 {
        match self.maximized {
            Maximized::First => return first,
            Maximized::Second => return second,
            Maximized::None => {}
        }
        let fits = |usable: u16| {
            if usable < self.min_first + self.min_second {
                return false;
            }
            let a = self
                .first_of(usable)
                .clamp(self.min_first, usable - self.min_second);
            a >= first && usable - a >= second
        };
        let mut usable = first.saturating_add(second);
        while usable < u16::MAX && !fits(usable) {
            usable += 1;
        }
        usable.saturating_add(gap)
    }

    pub fn vertical(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        match self.maximized {
            Maximized::First => (area, Rect::ZERO),
            Maximized::Second => (Rect::ZERO, area),
            Maximized::None => {
                let usable = area.height.saturating_sub(gap);
                if usable < self.min_first + self.min_second {
                    return (area, Rect::ZERO);
                }
                let mut first = self.first_of(usable);
                first = first.clamp(self.min_first, usable - self.min_second);
                (
                    Rect::new(area.x, area.y, area.width, first),
                    Rect::new(area.x, area.y + first + gap, area.width, usable - first),
                )
            }
        }
    }

    pub fn horizontal(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        match self.maximized {
            Maximized::First => (area, Rect::ZERO),
            Maximized::Second => (Rect::ZERO, area),
            Maximized::None => {
                let usable = area.width.saturating_sub(gap);
                if usable < self.min_first + self.min_second {
                    return (Rect::ZERO, area);
                }
                let mut first = self.first_of(usable);
                first = first.clamp(self.min_first, usable - self.min_second);
                (
                    Rect::new(area.x, area.y, first, area.height),
                    Rect::new(area.x + first + gap, area.y, usable - first, area.height),
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Scroll,

    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackChild {
    pub height: u16,

    pub min: u16,
}

impl StackChild {
    pub const fn fixed(height: u16) -> Self {
        Self {
            height,
            min: height,
        }
    }

    pub const fn flex(min: u16, height: u16) -> Self {
        Self { height, min }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack {
    pub gap: u16,

    pub overflow: Overflow,

    pub min_child_rows: u16,

    pub scrollbar_gap: u16,

    pub track_click: crate::core::scroll::TrackClick,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            gap: 0,
            overflow: Overflow::Scroll,
            min_child_rows: 1,
            scrollbar_gap: 1,
            track_click: crate::core::scroll::TrackClick::Page,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StackScroll {
    pub scroll: crate::core::scroll::ScrollState,

    kept: Option<usize>,

    track: Rect,

    click: crate::core::scroll::TrackClick,
}

impl StackScroll {
    pub fn on_wheel(&mut self, delta: i32) -> bool {
        self.scroll.scroll_by(delta.signum() as isize)
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> bool {
        !self.track.is_empty()
            && crate::widgets::scrollbar::press_with(self.track, pos, &mut self.scroll, self.click)
    }

    pub fn on_scrollbar_drag(&mut self, pos: Position) -> bool {
        !self.track.is_empty() && crate::widgets::scrollbar::drag(self.track, pos, &mut self.scroll)
    }

    pub fn pointer(
        &mut self,
        id: crate::core::id::WidgetId,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::event::Outcome> {
        use crate::core::event::Outcome;
        use crate::core::intent::Phase;
        if !ev.on(id) {
            return None;
        }
        let moved = |b: bool| {
            if b {
                Outcome::Changed
            } else {
                Outcome::Consumed
            }
        };
        if let Some((axis, delta)) = ev.wheel() {
            return Some(match axis {
                crate::core::hit::Axis::Vertical if !self.scroll.overflows() => Outcome::Ignored,
                crate::core::hit::Axis::Vertical => moved(self.on_wheel(delta)),
                crate::core::hit::Axis::Horizontal => Outcome::Ignored,
            });
        }
        ev.bar?;
        Some(match ev.phase() {
            Some(Phase::Press) => moved(self.on_scrollbar(ev.pos())),
            Some(Phase::Drag) => moved(self.on_scrollbar_drag(ev.pos())),
            Some(Phase::Click | Phase::Release | Phase::DragEnd) => Outcome::Consumed,
            _ => Outcome::Ignored,
        })
    }

    pub fn reset(&mut self) {
        self.scroll.offset = 0;
        self.kept = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackLayout {
    pub slots: Vec<Option<Rect>>,

    pub track: Option<Rect>,

    pub hidden_above: usize,
    pub hidden_below: usize,
}

impl StackLayout {
    pub fn slot(&self, i: usize) -> Option<Rect> {
        self.slots.get(i).copied().flatten()
    }
}

impl Stack {
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }
    pub fn min_child_rows(mut self, rows: u16) -> Self {
        self.min_child_rows = rows;
        self
    }
    pub fn scrollbar_gap(mut self, cols: u16) -> Self {
        self.scrollbar_gap = cols;
        self
    }
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.track_click = click;
        self
    }

    fn total(&self, heights: &[u16]) -> u32 {
        let shown = heights.iter().filter(|&&h| h > 0).count() as u32;
        heights.iter().map(|&h| h as u32).sum::<u32>() + self.gap as u32 * shown.saturating_sub(1)
    }

    pub fn measure(&self, children: &[StackChild]) -> u16 {
        let h: Vec<u16> = children.iter().map(|c| c.height).collect();
        self.total(&h).min(u16::MAX as u32) as u16
    }

    pub fn layout(
        &self,
        area: Rect,
        children: &[StackChild],
        state: &mut StackScroll,
        focus: Option<usize>,
    ) -> StackLayout {
        let n = children.len();
        let mut h: Vec<u16> = children.iter().map(|c| c.height).collect();
        let room = area.height as u32;

        let mut deficit = self.total(&h).saturating_sub(room);
        for (i, c) in children.iter().enumerate().rev() {
            if deficit == 0 {
                break;
            }
            let floor = c.min.max(self.min_child_rows).min(c.height);
            let give = (h[i] - floor).min(deficit.min(u16::MAX as u32) as u16);
            h[i] -= give;
            deficit -= give as u32;
        }
        let fits = self.total(&h) <= room;
        let scrolls = !fits && self.overflow == Overflow::Scroll && n > 0;
        if scrolls {
            for (hi, c) in h.iter_mut().zip(children) {
                *hi = c.height.min(area.height);
            }
        }
        let (content, track) = if scrolls && area.width > self.scrollbar_gap + 1 {
            let w = area.width - self.scrollbar_gap - 1;
            (
                Rect::new(area.x, area.y, w, area.height),
                Some(Rect::new(area.right() - 1, area.y, 1, area.height)),
            )
        } else {
            (area, None)
        };

        let max_first = if scrolls {
            (0..n)
                .find(|&i| self.total(&h[i..]) <= room)
                .unwrap_or(n - 1)
        } else {
            0
        };
        let mut first = if scrolls {
            state.scroll.offset.min(max_first)
        } else {
            0
        };
        if scrolls
            && let Some(k) = focus.filter(|&k| k < n)
            && state.kept != Some(k)
        {
            if k < first {
                first = k;
            } else {
                while first < k && self.total(&h[first..=k]) > room {
                    first += 1;
                }
            }
        }
        state.kept = focus;
        state.scroll.content_len = if scrolls { n } else { 0 };
        state.scroll.viewport_len = n - max_first;
        state.scroll.offset = first;
        state.track = track.unwrap_or_default();
        state.click = self.track_click;

        let mut slots = vec![None; n];
        let mut y = area.y as u32;
        let mut placed = false;
        let mut hidden_below = 0;
        let mut full = false;
        for i in first..n {
            if h[i] == 0 {
                continue;
            }
            if full {
                hidden_below += 1;
                continue;
            }
            let top = if placed { y + self.gap as u32 } else { y };
            if top + h[i] as u32 <= area.bottom() as u32 {
                slots[i] = Some(Rect::new(content.x, top as u16, content.width, h[i]));
                y = top + h[i] as u32;
                placed = true;
            } else if !placed && top < area.bottom() as u32 {
                slots[i] = Some(Rect::new(
                    content.x,
                    top as u16,
                    content.width,
                    area.bottom() - top as u16,
                ));
                y = area.bottom() as u32;
                placed = true;
                full = true;
            } else {
                full = true;
                hidden_below += 1;
            }
        }
        let hidden_above = h[..first].iter().filter(|&&r| r > 0).count();
        StackLayout {
            slots,
            track,
            hidden_above,
            hidden_below,
        }
    }

    pub fn chrome(
        layout: &StackLayout,
        area: Rect,
        buf: &mut ratatui::buffer::Buffer,
        ctx: &mut crate::ui::ctx::RenderCtx,
        id: crate::core::id::WidgetId,
        state: &StackScroll,
    ) {
        if let Some(track) = layout.track {
            ctx.scrollable(id, area);
            crate::widgets::scrollbar::render_vertical(track, buf, ctx, id, &state.scroll, false);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitCmd {
    Left,

    Right,

    Up,

    Down,
}

impl crate::core::keymap::Actions for SplitCmd {
    fn default_keymap() -> crate::core::keymap::KeyMap<Self> {
        use crate::core::keymap::KeyBinding;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let alt = KeyModifiers::ALT.union(KeyModifiers::SHIFT);
        let ctrl = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
        let arrows = [
            (KeyCode::Left, SplitCmd::Left),
            (KeyCode::Right, SplitCmd::Right),
            (KeyCode::Up, SplitCmd::Up),
            (KeyCode::Down, SplitCmd::Down),
        ];
        let alt_chords = arrows.map(|(k, c)| (KeyBinding::new(k, alt), c));
        let stand_ins = arrows.map(|(k, c)| (KeyBinding::new(k, ctrl).alt_stand_in(), c));
        alt_chords.into_iter().chain(stand_ins).collect()
    }

    fn label(self) -> &'static str {
        match self {
            SplitCmd::Left => "Move seam left",
            SplitCmd::Right => "Move seam right",
            SplitCmd::Up => "Move seam up",
            SplitCmd::Down => "Move seam down",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitKeys {
    pub keymap: crate::core::keymap::KeyMap<SplitCmd>,

    pub step: u16,
}

impl Default for SplitKeys {
    fn default() -> Self {
        use crate::core::keymap::Actions;
        Self {
            keymap: SplitCmd::default_keymap(),
            step: 2,
        }
    }
}

impl SplitKeys {
    pub fn keymap(mut self, keymap: crate::core::keymap::KeyMap<SplitCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn step(mut self, cells: u16) -> Self {
        self.step = cells;
        self
    }

    pub fn delta(&self, key: &crate::core::event::Key, dir: SplitDir) -> Option<i16> {
        let step = self.step.min(i16::MAX as u16) as i16;
        match (crate::core::keymap::lookup(&self.keymap, key)?, dir) {
            (SplitCmd::Left, SplitDir::Horizontal) | (SplitCmd::Up, SplitDir::Vertical) => {
                Some(-step)
            }
            (SplitCmd::Right, SplitDir::Horizontal) | (SplitCmd::Down, SplitDir::Vertical) => {
                Some(step)
            }
            _ => None,
        }
    }

    pub fn apply(
        &self,
        key: &crate::core::event::Key,
        split: &mut Split,
        dir: SplitDir,
        area: Rect,
        gap: u16,
    ) -> Option<crate::core::event::Outcome> {
        use crate::core::event::Outcome;
        let delta = self.delta(key, dir)?;
        if split.maximized != Maximized::None {
            return Some(Outcome::Consumed);
        }

        let before = split.layout(dir, area, gap);
        split.nudge(dir, area, gap, delta);
        Some(if split.layout(dir, area, gap) == before {
            Outcome::Consumed
        } else {
            Outcome::Changed
        })
    }

    pub fn hint(&self, dir: SplitDir) -> Option<crate::widgets::keyhint::Hint> {
        use crate::core::keymap::{deliverable_binding_for, hint_key, intern};
        use ratatui::crossterm::event::KeyCode;
        let (a, b, ka, kb, pair) = match dir {
            SplitDir::Horizontal => (
                SplitCmd::Left,
                SplitCmd::Right,
                KeyCode::Left,
                KeyCode::Right,
                "←→",
            ),
            SplitDir::Vertical => (
                SplitCmd::Up,
                SplitCmd::Down,
                KeyCode::Up,
                KeyCode::Down,
                "↑↓",
            ),
        };
        let both = match (
            deliverable_binding_for(&self.keymap, a),
            deliverable_binding_for(&self.keymap, b),
        ) {
            (Some(x), Some(y)) if x.code == ka && y.code == kb && x.mods == y.mods => {
                let label = x.label();
                label
                    .strip_suffix(if ka == KeyCode::Left { '←' } else { '↑' })
                    .map(|prefix| intern(format!("{prefix}{pair}")))
            }
            _ => None,
        };
        let key = match both {
            Some(k) => k,
            None => hint_key(&self.keymap, a).or_else(|| hint_key(&self.keymap, b))?,
        };
        Some(crate::widgets::keyhint::hint(key, "Resize"))
    }
}

#[cfg(test)]
mod split_keys_tests {
    use super::*;
    use crate::core::event::{Key, Outcome};
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn alt_shift_arrows_move_the_seam_on_its_axis_only() {
        let keys = SplitKeys::default();
        let m = KeyModifiers::ALT | KeyModifiers::SHIFT;
        let area = Rect::new(0, 0, 81, 10);
        let mut split = Split::new(50, 10, 10);
        let before = split.layout(SplitDir::Horizontal, area, 1).0.width;
        let o = keys.apply(
            &key(KeyCode::Right, m),
            &mut split,
            SplitDir::Horizontal,
            area,
            1,
        );
        assert_eq!(o, Some(Outcome::Changed));
        assert_eq!(
            split.layout(SplitDir::Horizontal, area, 1).0.width,
            before + 2
        );

        let up = keys.apply(
            &key(KeyCode::Up, m),
            &mut split,
            SplitDir::Horizontal,
            area,
            1,
        );
        assert_eq!(up, None);

        let plain = keys.apply(
            &key(KeyCode::Left, KeyModifiers::NONE),
            &mut split,
            SplitDir::Horizontal,
            area,
            1,
        );
        assert_eq!(plain, None);

        let mut tight = Split::new(50, 40, 40);
        let o = keys.apply(
            &key(KeyCode::Left, m),
            &mut tight,
            SplitDir::Horizontal,
            area,
            1,
        );
        assert_eq!(o, Some(Outcome::Consumed));
    }

    #[test]
    fn the_hint_names_the_bound_chord() {
        use crate::core::keyboard::{AltKeys, set_alt_keys};
        set_alt_keys(AltKeys::Delivered);
        let keys = SplitKeys::default();
        assert_eq!(keys.hint(SplitDir::Horizontal).unwrap().key, "Alt+Shift+←→");
        let custom = SplitKeys::default().keymap(vec![(
            crate::core::keymap::KeyBinding::ctrl('h'),
            SplitCmd::Left,
        )]);
        assert_eq!(custom.hint(SplitDir::Horizontal).unwrap().key, "Ctrl+H");
        assert!(
            SplitKeys::default()
                .keymap(vec![])
                .hint(SplitDir::Vertical)
                .is_none()
        );
    }
}

#[cfg(test)]
mod split_extent_tests {
    use super::*;

    #[test]
    fn a_split_at_its_extent_gives_each_pane_its_size() {
        for split in [
            Split::new(30, 10, 10),
            Split::new(65, 4, 3),
            Split::new(50, 0, 0),
        ] {
            for (a, b) in [(20, 40), (40, 8), (5, 5)] {
                let e = split.extent_for(a, b, 1);
                let (f, g) = split.horizontal(Rect::new(0, 0, e, 3), 1);
                assert!(
                    f.width >= a && g.width >= b,
                    "{split:?} {a},{b}: {f:?} {g:?}"
                );
                let (f, g) = split.vertical(Rect::new(0, 0, 3, e), 1);
                assert!(
                    f.height >= a && g.height >= b,
                    "{split:?} {a},{b}: {f:?} {g:?}"
                );
                let (f, g) = split.horizontal(Rect::new(0, 0, e - 1, 3), 1);
                assert!(f.width < a || g.width < b, "tight: {split:?} {a},{b}");
            }
        }
        let mut zoomed = Split::new(30, 10, 10);
        zoomed.toggle_max(Maximized::Second);
        assert_eq!(zoomed.extent_for(20, 40, 1), 40, "the zoomed pane alone");
    }

    #[test]
    fn the_natural_extent_ignores_a_moved_seam() {
        let fresh = Split::new(30, 10, 10);
        let start = fresh.natural_extent(20, 40, 1);
        assert_eq!(start, fresh.extent_for(20, 40, 1));
        let mut dragged = fresh;
        dragged.drag_to(
            SplitDir::Horizontal,
            Rect::new(0, 0, 100, 3),
            1,
            Position::new(70, 1),
        );
        assert_ne!(dragged.extent_for(20, 40, 1), start, "the ratio moved");
        assert_eq!(dragged.natural_extent(20, 40, 1), start);
        let mut keyed = fresh;
        keyed.grow(20);
        assert_eq!(keyed.natural_extent(20, 40, 1), start);
        keyed.toggle_max(Maximized::First);
        assert_eq!(keyed.natural_extent(20, 40, 1), 20, "zoomed: first alone");
    }
}

#[cfg(test)]
mod split_keys_alt_tests {
    use super::*;
    use crate::core::event::{Key, Outcome};
    use crate::core::keyboard::{AltKeys, set_alt_keys};
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn ctrl_shift_arrows_stand_in_where_alt_is_not_delivered() {
        let keys = SplitKeys::default();
        let ctrl = Key {
            code: KeyCode::Right,
            mods: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        };
        let area = Rect::new(0, 0, 81, 10);
        let mut split = Split::new(50, 10, 10);
        set_alt_keys(AltKeys::UntilSeen);
        assert_eq!(
            keys.hint(SplitDir::Horizontal).unwrap().key,
            "Ctrl+Shift+←→"
        );
        assert_eq!(keys.hint(SplitDir::Vertical).unwrap().key, "Ctrl+Shift+↑↓");
        assert_eq!(
            keys.apply(&ctrl, &mut split, SplitDir::Horizontal, area, 1),
            Some(Outcome::Changed)
        );
        set_alt_keys(AltKeys::Delivered);
        assert_eq!(keys.hint(SplitDir::Horizontal).unwrap().key, "Alt+Shift+←→");
        assert_eq!(
            keys.apply(&ctrl, &mut split, SplitDir::Horizontal, area, 1),
            None,
            "Ctrl+Shift+→ selects a word in the pane"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kids(h: &[u16]) -> Vec<StackChild> {
        h.iter().map(|&h| StackChild::fixed(h)).collect()
    }

    #[test]
    fn a_stack_that_fits_places_children_with_gaps_and_no_scrollbar() {
        let s = Stack::default().gap(1);
        let mut st = StackScroll::default();
        let l = s.layout(Rect::new(2, 3, 20, 10), &kids(&[3, 0, 3, 1]), &mut st, None);
        assert_eq!(l.slot(0), Some(Rect::new(2, 3, 20, 3)));
        assert_eq!(l.slot(1), None, "a hidden child takes no gap");
        assert_eq!(l.slot(2), Some(Rect::new(2, 7, 20, 3)));
        assert_eq!(l.slot(3), Some(Rect::new(2, 11, 20, 1)));
        assert_eq!(l.track, None);
        assert!(!st.scroll.overflows());
    }

    #[test]
    fn overflow_never_places_a_child_past_the_rect() {
        let area = Rect::new(0, 0, 30, 8);
        for overflow in [Overflow::Scroll, Overflow::Clip] {
            let s = Stack::default().overflow(overflow);
            let mut st = StackScroll::default();
            let l = s.layout(area, &kids(&[3, 3, 3, 3, 1]), &mut st, None);
            for r in l.slots.iter().flatten() {
                assert!(r.bottom() <= area.bottom(), "{overflow:?}: {r:?}");
            }
            assert_eq!(l.slot(2), None, "the third field does not fit");
            assert_eq!(l.hidden_below, 3);
            assert_eq!(l.track.is_some(), overflow == Overflow::Scroll);
        }
    }

    #[test]
    fn flexible_children_shrink_before_the_stack_scrolls() {
        let s = Stack::default().min_child_rows(2);
        let mut st = StackScroll::default();
        let l = s.layout(
            Rect::new(0, 0, 30, 8),
            &[StackChild::fixed(3), StackChild::flex(0, 10)],
            &mut st,
            None,
        );
        assert_eq!(l.slot(1), Some(Rect::new(0, 3, 30, 5)));
        assert_eq!(l.track, None);

        let l = s.layout(
            Rect::new(0, 0, 30, 4),
            &[StackChild::fixed(3), StackChild::flex(0, 10)],
            &mut st,
            None,
        );
        assert!(l.track.is_some());
    }

    #[test]
    fn focus_scrolls_the_least_and_the_wheel_can_move_away() {
        let s = Stack::default();
        let area = Rect::new(0, 0, 30, 7);
        let mut st = StackScroll::default();
        let ch = kids(&[3, 3, 3, 3, 1]);
        let l = s.layout(area, &ch, &mut st, Some(3));
        assert_eq!(
            st.scroll.offset, 2,
            "the fourth field is now the last in view"
        );
        assert_eq!(l.slot(3), Some(Rect::new(0, 3, 28, 3)));
        assert_eq!(l.slot(1), None);
        assert_eq!(l.hidden_above, 2);

        st.on_wheel(-3);
        st.on_wheel(-3);
        let l = s.layout(area, &ch, &mut st, Some(3));
        assert_eq!(st.scroll.offset, 0);
        assert_eq!(l.slot(3), None);

        let l = s.layout(area, &ch, &mut st, Some(4));
        assert!(l.slot(4).is_some());
        let l = s.layout(area, &ch, &mut st, Some(0));
        assert_eq!(l.slot(0), Some(Rect::new(0, 0, 28, 3)));
    }

    #[test]
    fn splits_respect_minimums_and_maximize() {
        let s = Split::new(60, 5, 5);
        let (a, b) = s.vertical(Rect::new(0, 0, 80, 30), 1);
        assert_eq!(a.height + b.height + 1, 30);
        assert_eq!(a.height, 17);
        let mut m = s;
        m.toggle_max(Maximized::Second);
        let (a, b) = m.vertical(Rect::new(0, 0, 80, 30), 1);
        assert!(a.is_empty());
        assert_eq!(b.height, 30);
        let (a, b) = s.vertical(Rect::new(0, 0, 80, 8), 1);
        assert_eq!(a.height, 8);
        assert!(b.is_empty());
    }

    #[test]
    fn drag_moves_the_seam_and_respects_minima() {
        let mut s = Split::new(50, 10, 10);
        let area = Rect::new(0, 0, 101, 20);
        assert_eq!(
            s.handle(SplitDir::Horizontal, area, 1),
            Rect::new(50, 0, 1, 20)
        );
        assert!(s.drag_to(SplitDir::Horizontal, area, 1, Position::new(70, 3)));
        let (a, _) = s.horizontal(area, 1);
        assert_eq!(a.width, 70);
        s.drag_to(SplitDir::Horizontal, area, 1, Position::new(2, 3));
        let (a, _) = s.horizontal(area, 1);
        assert_eq!(a.width, 10, "clamped to min_first");
        s.nudge(SplitDir::Horizontal, area, 1, 5);
        let (a, _) = s.horizontal(area, 1);
        assert_eq!(a.width, 15);
    }

    #[test]
    fn drag_places_the_seam_on_the_exact_cell() {
        for width in [37u16, 62, 133, 241] {
            let area = Rect::new(0, 0, width, 4);
            let mut s = Split::new(50, 2, 2);
            let usable = width - 1;
            for x in (usable / 20).max(2)..usable * 19 / 20 {
                s.drag_to(SplitDir::Horizontal, area, 1, Position::new(x, 0));
                let (a, _) = s.horizontal(area, 1);
                assert_eq!(a.width, x, "width {width} x {x}");
            }
        }

        let area = Rect::new(0, 0, 62, 4);
        let mut s = Split::new(50, 2, 2);
        s.drag_to(SplitDir::Horizontal, area, 1, Position::new(33, 0));
        s.grow(1);
        assert_eq!(s.horizontal(area, 1).0.width, 61 * s.percent / 100);
        s.reset();
        assert_eq!(s, Split::new(50, 2, 2));
    }
}

#[cfg(test)]
mod track_click_tests {
    use super::*;
    use crate::core::scroll::TrackClick;

    fn pressed_at_the_bottom(click: TrackClick) -> usize {
        let kids: Vec<StackChild> = (0..30).map(|_| StackChild::fixed(2)).collect();
        let mut st = StackScroll::default();
        let area = Rect::new(0, 0, 30, 10);
        let l = Stack::default()
            .track_click(click)
            .layout(area, &kids, &mut st, None);
        let track = l.track.expect("the stack scrolls");
        assert!(st.on_scrollbar(Position::new(track.x, track.bottom() - 1)));
        st.scroll.offset
    }

    #[test]
    fn a_stack_track_click_can_page() {
        let page = pressed_at_the_bottom(TrackClick::Page);
        let jump = pressed_at_the_bottom(TrackClick::Jump);
        assert!(page > 0 && page <= 5, "one page of children: {page}");
        assert!(jump > page, "the default jumps: {jump}");
    }
}
