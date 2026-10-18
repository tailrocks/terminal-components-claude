use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};

use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Actions, KeyBinding, KeyMap, NavKeys, lookup, unassigned_chord};
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::theme::Kind;
use crate::ui::ctx::RenderCtx;
use crate::ui::tooltip::Tip;
use crate::widgets::icon_button::IconButton;
use crate::widgets::menu::{ContextMenu, MenuAction, MenuItem};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabItem {
    pub label: String,
    pub dirty: bool,
    pub busy: bool,
    pub error: bool,

    pub done: bool,
    pub closable: bool,

    pub prefix: Option<String>,

    pub suffix: Option<String>,
}

impl TabItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            ..Default::default()
        }
    }
    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }
    pub fn prefix(mut self, p: &str) -> Self {
        self.prefix = Some(p.to_owned());
        self
    }
    pub fn suffix(mut self, s: &str) -> Self {
        self.suffix = Some(s.to_owned());
        self
    }

    pub fn done(mut self, done: bool) -> Self {
        self.done = done;
        self
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
}

impl crate::ui::measure::Measure for Tabs {
    fn measure(
        &self,
        _avail: ratatui::layout::Size,
        _ctx: &crate::ui::ctx::RenderCtx,
    ) -> ratatui::layout::Size {
        let tabs: u16 = self.items.iter().map(|it| self.tab_width(it) + 1).sum();
        let new = if self.allow_new { 4 } else { 0 };
        ratatui::layout::Size::new(tabs + new, 2)
    }

    fn min_width(&self, _avail: ratatui::layout::Size, _ctx: &crate::ui::ctx::RenderCtx) -> u16 {
        let widest = self
            .items
            .iter()
            .map(|it| self.tab_width(it))
            .max()
            .unwrap_or(0);
        let more = 3 + self.items.len().to_string().len() as u16;
        widest + 1 + 8 + more + if self.allow_new { 4 } else { 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Tabs {
    pub id: WidgetId,
    pub items: Vec<TabItem>,
    pub active: usize,
    pub cursor: usize,

    pub first: usize,
    pub areas: Vec<Rect>,

    pub allow_new: bool,

    pub rule: TabRule,

    pub middle_click_closes: bool,

    pub wrap: bool,

    pub nav: NavKeys,

    pub keymap: KeyMap<TabCmd>,

    pub close_button: IconButton,

    pub close_reveal: CloseReveal,

    pub overflow_menu: bool,

    pub more_button: IconButton,

    pub passive: bool,

    pub passive_focused: bool,

    menu: Option<(ContextMenu, Vec<usize>)>,

    more_area: Rect,

    fit: usize,

    max_first: usize,

    revealed: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CloseReveal {
    #[default]
    Always,

    Hover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabRule {
    #[default]
    Auto,

    Accent,

    Neutral,

    FocusOnly,
}

pub const DEFAULT_CLOSE_KEYS: &[KeyCode] = &[KeyCode::Char('x'), KeyCode::Delete];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabCmd {
    Close,

    New,

    Jump(usize),

    ShowHidden,
}

impl TabCmd {
    pub fn digit_keys() -> KeyMap<Self> {
        ('1'..='9')
            .enumerate()
            .map(|(i, c)| (KeyBinding::char(c), TabCmd::Jump(i)))
            .collect()
    }
}

impl Actions for TabCmd {
    fn label(self) -> &'static str {
        match self {
            TabCmd::Close => "Close tab",
            TabCmd::New => "New tab",
            TabCmd::Jump(_) => "Jump to tab",
            TabCmd::ShowHidden => "Show hidden",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        let mut map: KeyMap<Self> = DEFAULT_CLOSE_KEYS
            .iter()
            .map(|&code| (KeyBinding::key(code), TabCmd::Close))
            .collect();
        map.push((KeyBinding::char('n'), TabCmd::New));
        map.push((KeyBinding::key(KeyCode::Down), TabCmd::ShowHidden));
        map.extend(Self::digit_keys());
        map
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAction {
    Activated(usize),
    Close(usize),
    New,
}

impl Tabs {
    pub fn new(id: WidgetId, labels: &[&str]) -> Self {
        Self {
            id,
            items: labels.iter().map(|l| TabItem::new(l)).collect(),
            active: 0,
            cursor: 0,
            first: 0,
            areas: vec![],
            allow_new: false,
            rule: TabRule::Auto,
            middle_click_closes: true,
            wrap: false,
            nav: NavKeys::default(),
            keymap: TabCmd::default_keymap(),
            close_button: IconButton::close(WidgetId::of("tabs.close")),
            close_reveal: CloseReveal::Always,
            overflow_menu: true,
            more_button: IconButton::new(WidgetId::of("tabs.more"), "+"),
            passive: false,
            passive_focused: false,
            menu: None,
            more_area: Rect::ZERO,
            fit: 0,
            max_first: 0,
            revealed: None,
        }
    }

    pub fn with_items(id: WidgetId, items: Vec<TabItem>) -> Self {
        Self {
            id,
            items,
            active: 0,
            cursor: 0,
            first: 0,
            areas: vec![],
            allow_new: false,
            rule: TabRule::Auto,
            middle_click_closes: true,
            wrap: false,
            nav: NavKeys::default(),
            keymap: TabCmd::default_keymap(),
            close_button: IconButton::close(WidgetId::of("tabs.close")),
            close_reveal: CloseReveal::Always,
            overflow_menu: true,
            more_button: IconButton::new(WidgetId::of("tabs.more"), "+"),
            passive: false,
            passive_focused: false,
            menu: None,
            more_area: Rect::ZERO,
            fit: 0,
            max_first: 0,
            revealed: None,
        }
    }

    pub const STRIP_H: u16 = 2;

    pub fn layout(area: Rect) -> (Rect, Rect) {
        Self::layout_gap(area, 1)
    }

    pub fn layout_gap(area: Rect, gap: u16) -> (Rect, Rect) {
        let strip_h = Self::STRIP_H.min(area.height);
        let strip = Rect::new(area.x, area.y, area.width, strip_h);
        let off = (Self::STRIP_H + gap).min(area.height);
        let body = Rect::new(area.x, area.y + off, area.width, area.height - off);
        (strip, body)
    }

    pub fn passive(mut self, on: bool) -> Self {
        self.passive = on;
        self
    }

    pub fn passive_focused(mut self, on: bool) -> Self {
        self.passive_focused = on;
        self
    }

    pub fn rule(mut self, rule: TabRule) -> Self {
        self.rule = rule;
        self
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn keymap(mut self, keymap: KeyMap<TabCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn digit_jump(mut self, on: bool) -> Self {
        self.keymap.retain(|(_, a)| !matches!(a, TabCmd::Jump(_)));
        if on {
            self.keymap.extend(TabCmd::digit_keys());
        }
        self
    }

    pub fn close_keys(mut self, keys: &[KeyCode]) -> Self {
        self.keymap.retain(|(_, a)| *a != TabCmd::Close);
        self.keymap.extend(
            keys.iter()
                .map(|&code| (KeyBinding::key(code), TabCmd::Close)),
        );
        self
    }

    pub fn new_key(mut self, key: Option<char>) -> Self {
        self.keymap.retain(|(_, a)| *a != TabCmd::New);
        if let Some(c) = key {
            self.keymap.push((KeyBinding::char(c), TabCmd::New));
        }
        self
    }

    pub fn allow_new(mut self, on: bool) -> Self {
        self.allow_new = on;
        self
    }

    pub fn overflow_menu(mut self, on: bool) -> Self {
        self.overflow_menu = on;
        self
    }

    fn overflow_tip(&self, t: &crate::theme::Theme) -> Tip {
        let tip = Tip::new(t.strings.tip_overflow);
        match crate::core::keymap::label_for(&self.keymap, TabCmd::ShowHidden) {
            Some(k) => tip.shortcut(&k),
            None => tip,
        }
    }

    pub fn more_button(mut self, button: IconButton) -> Self {
        self.more_button = button;
        self
    }

    pub fn menu_open(&self) -> bool {
        self.menu.is_some()
    }

    pub fn more_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::OVERFLOW))
    }

    pub fn menu_id(&self) -> WidgetId {
        self.id.sub("more-menu")
    }

    fn hidden_tabs(&self) -> Vec<usize> {
        let end = (self.first + self.fit).min(self.items.len());
        (0..self.first.min(self.items.len()))
            .chain(end..self.items.len())
            .collect()
    }

    fn open_menu(&mut self) {
        let tabs = self.hidden_tabs();
        if tabs.is_empty() {
            return;
        }
        let items = tabs
            .iter()
            .map(|&i| {
                let it = &self.items[i];
                match &it.prefix {
                    Some(p) => MenuItem::new(format!("{p} {}", it.label)),
                    None => MenuItem::new(it.label.clone()),
                }
            })
            .collect();
        let menu =
            ContextMenu::new(self.menu_id(), items).anchor_control(self.more_id(), self.more_area);
        self.menu = Some((menu, tabs));
    }

    fn menu_event(&mut self, ev: Option<MenuAction>) -> Option<TabAction> {
        match ev? {
            MenuAction::Chosen(k) => {
                let (_, tabs) = self.menu.take()?;
                let i = *tabs.get(k)?;
                self.set_active(i);
                Some(TabAction::Activated(i))
            }
            MenuAction::Dismissed => {
                self.menu = None;
                None
            }
        }
    }

    pub fn close_target_w(mut self, cells: u16) -> Self {
        self.close_button = self.close_button.plate(cells);
        self
    }

    pub fn close_button(mut self, button: IconButton) -> Self {
        self.close_button = button;
        self
    }

    pub fn close_reveal(mut self, reveal: CloseReveal) -> Self {
        self.close_reveal = reveal;
        self
    }

    pub fn set_items(&mut self, items: Vec<TabItem>) {
        self.items = items;
        let last = self.items.len().saturating_sub(1);
        self.active = self.active.min(last);
        self.cursor = self.cursor.min(last);
        self.first = self.first.min(last);
    }

    pub fn on_wheel(&mut self, dx: i32, dy: i32) -> Outcome {
        if let Some((menu, _)) = self.menu.as_mut() {
            return if dy != 0 {
                crate::core::scroll::Scrollable::wheel(&mut menu.list, Axis::Vertical, dy).outcome()
            } else {
                Outcome::Consumed
            };
        }
        let delta = if dx != 0 { dx } else { dy };
        if delta == 0 || self.max_first == 0 {
            return Outcome::Ignored;
        }
        let to = (self.first as i64 + delta as i64).clamp(0, self.max_first as i64) as usize;
        if to == self.first {
            return Outcome::Consumed;
        }
        self.first = to;
        Outcome::Changed
    }

    pub fn middle_click_closes(mut self, on: bool) -> Self {
        self.middle_click_closes = on;
        self
    }

    pub fn on_middle_click(&self, id: WidgetId) -> Option<TabAction> {
        if !self.middle_click_closes {
            return None;
        }
        let i = self.tab_of(id).or_else(|| self.close_of(id))?;
        self.items[i].closable.then_some(TabAction::Close(i))
    }

    pub fn tab_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::TAB, ItemKey::index(i)))
    }
    pub fn close_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::CLOSE, ItemKey::index(i)))
    }
    pub fn new_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::NEW))
    }
    pub fn left_id(&self) -> WidgetId {
        self.id.part(PartRef::OVERFLOW_BEFORE)
    }
    pub fn right_id(&self) -> WidgetId {
        self.id.part(PartRef::OVERFLOW_AFTER)
    }

    fn close_of(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::CLOSE, 0..self.items.len())
    }

    fn tab_of(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::TAB, 0..self.items.len())
    }

    pub fn hidden(&self) -> (usize, usize) {
        let after = self.items.len().saturating_sub(self.first + self.fit);
        (self.first, after)
    }

    pub fn set_active(&mut self, i: usize) {
        if self.items.is_empty() {
            self.active = 0;
            self.cursor = 0;
            return;
        }
        let to = i.min(self.items.len() - 1);
        self.cursor = to;
        if to != self.active {
            self.active = to;
            self.ensure_visible(to);
        }
    }

    pub fn remove(&mut self, i: usize) {
        if i < self.items.len() {
            self.items.remove(i);
            if self.active >= self.items.len() {
                self.active = self.items.len().saturating_sub(1);
            } else if self.active > i {
                self.active -= 1;
            }
            self.cursor = self.active;
            self.first = self.first.min(self.items.len().saturating_sub(1));
        }
    }

    fn ensure_visible(&mut self, i: usize) {
        if i < self.first {
            self.first = i;
        } else if self.fit > 0 && i >= self.first + self.fit {
            self.first = i + 1 - self.fit;
        }
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TabAction>) {
        if let Some((menu, _)) = self.menu.as_mut() {
            let (o, ev) = menu.on_key(key);
            return (o, self.menu_event(ev));
        }
        let action = lookup(&self.keymap, key);
        if unassigned_chord(key, &self.keymap) {
            return (Outcome::Ignored, None);
        }
        if self.items.is_empty() {
            return match (action, key.code) {
                (_, KeyCode::Enter) | (Some(TabCmd::New), _) if self.allow_new => {
                    (Outcome::Changed, Some(TabAction::New))
                }
                _ => (Outcome::Ignored, None),
            };
        }
        match action {
            Some(TabCmd::Jump(i)) => {
                return if i < self.items.len() {
                    self.set_active(i);
                    (Outcome::Changed, Some(TabAction::Activated(i)))
                } else {
                    (Outcome::Ignored, None)
                };
            }
            Some(TabCmd::Close) if self.items[self.cursor].closable => {
                return (Outcome::Changed, Some(TabAction::Close(self.cursor)));
            }
            Some(TabCmd::New) if self.allow_new => {
                return (Outcome::Changed, Some(TabAction::New));
            }
            Some(TabCmd::ShowHidden) if self.overflow_menu && !self.hidden_tabs().is_empty() => {
                self.open_menu();
                return (Outcome::Changed, None);
            }
            _ => {}
        }
        let to = match self.nav.action(key).filter(|_| key.plain()) {
            Some(nav) => self.cursor().apply(nav, self, 1).index(),
            None => None,
        };
        if let Some(to) = to {
            self.set_active(to);
            return (Outcome::Changed, Some(TabAction::Activated(self.active)));
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.set_active(self.cursor);
                (Outcome::Changed, Some(TabAction::Activated(self.active)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<TabAction>) {
        if let Some((menu, _)) = self.menu.as_mut()
            && menu.contains_id(id)
        {
            let ev = menu.on_click(id);
            return (Outcome::Changed, self.menu_event(ev));
        }
        if id == self.more_id() {
            if self.menu.take().is_none() {
                self.open_menu();
            }
            return (Outcome::Changed, None);
        }

        self.menu = None;
        if id == self.new_id() {
            return (Outcome::Changed, Some(TabAction::New));
        }
        if id == self.left_id() {
            self.first = self.first.saturating_sub(1);
            return (Outcome::Changed, None);
        }
        if id == self.right_id() {
            self.first = (self.first + 1).min(self.max_first);
            return (Outcome::Changed, None);
        }
        if let Some(i) = self.close_of(id) {
            return (Outcome::Changed, Some(TabAction::Close(i)));
        }
        if let Some(i) = self.tab_of(id) {
            self.set_active(i);
            return (Outcome::Changed, Some(TabAction::Activated(i)));
        }
        (Outcome::Ignored, None)
    }

    fn tab_width(&self, it: &TabItem) -> u16 {
        let mut w = 1 + crate::ui::text::width(&it.label) as u16 + 2;
        if let Some(p) = &it.prefix {
            w += crate::ui::text::width(p) as u16 + 1;
        }
        if let Some(x) = &it.suffix {
            w += crate::ui::text::width(x) as u16 + 1;
        }
        if it.dirty || it.busy || it.error || it.done {
            w += 2;
        }
        if it.closable {
            w += 2;
        }
        w
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = if self.passive {
            self.passive_focused
        } else {
            ctx.interaction.focused_under(self.id, self.menu.is_some())
        };

        if !self.passive {
            ctx.control(self.id, area, false);
            ctx.scrollable(self.id, area);
        }
        self.areas = vec![Rect::ZERO; self.items.len()];
        let y = area.y;
        if area.height >= 2 {
            for xx in area.left()..area.right() {
                buf.set_string(
                    xx,
                    y + 1,
                    t.glyphs.bar_empty,
                    ratatui::style::Style::new().fg(t.border_subtle).bg(bg),
                );
            }
        }

        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let mut x = area.x;

        let widths: Vec<u16> = self.items.iter().map(|it| self.tab_width(it)).collect();
        let total: u16 = widths.iter().map(|w| w + 1).sum();
        let overflow = total + new_w > area.width;

        let left_w: u16 = if overflow { 4 } else { 0 };
        let right_w: u16 = if overflow { 4 } else { 0 };

        let more_w: u16 = if overflow && self.overflow_menu {
            3 + self.items.len().to_string().len() as u16
        } else {
            0
        };
        let avail = area.width.saturating_sub(left_w + right_w + more_w + new_w);

        self.max_first = {
            let mut used = 0u16;
            let mut f = self.items.len();
            while f > 0 && used + widths[f - 1] < avail {
                used += widths[f - 1] + 1;
                f -= 1;
            }
            if overflow {
                f.min(self.items.len().saturating_sub(1))
            } else {
                0
            }
        };

        let reveal = self.revealed != Some(self.active);
        self.revealed = Some(self.active);
        self.first = self.first.min(self.max_first);
        let mut fit;
        if reveal && self.active < self.first {
            self.first = self.active;
        }
        loop {
            fit = 0;
            let mut used = 0u16;
            for w in widths.iter().skip(self.first) {
                if used + w + 1 > avail {
                    break;
                }
                used += w + 1;
                fit += 1;
            }
            if fit == 0 && !self.items.is_empty() {
                fit = 1;
            }
            if reveal && self.active >= self.first + fit && self.first + 1 < self.items.len() {
                self.first += 1;
                continue;
            }
            break;
        }
        self.fit = fit;
        if overflow {
            let more_left = self.first > 0;
            let st = if more_left {
                ctx.title_style(self.left_id(), bg)
            } else {
                t.faint().bg(bg)
            };
            let text = if more_left {
                format!("{}{:<3}", t.glyphs.overflow_left, self.first.min(99))
            } else {
                "    ".into()
            };
            buf.set_string(x, y, &text, st);
            if more_left {
                ctx.register_part(self.id, PartRef::OVERFLOW_BEFORE, Rect::new(x, y, 4, 1));
            }
            x += 4;
        }

        #[allow(clippy::needless_range_loop)]
        for i in self.first..(self.first + fit).min(self.items.len()) {
            let it = &self.items[i];
            let w = widths[i];
            if x + w > area.right() {
                break;
            }
            let r = Rect::new(x, y, w, 1);
            self.areas[i] = r;
            let tid = self.tab_id(i);

            let mut s = ctx.child_state(self.id, tid, i == self.cursor);
            s.focused = focused && i == self.cursor;
            let active = i == self.active;
            s.selected = active;

            let look = t.interactive(Kind::Tab, s, t.plane_of(bg));
            let st = look.style;
            crate::ui::ctx::fill(buf, r, st);
            if s.focused {
                buf.set_string(x, y, look.gutter, look.gutter_style);
            }
            let mut cx = x + 1;

            let plane = st.bg.unwrap_or(bg);
            if let Some(p) = &it.prefix {
                let label = st.fg.unwrap_or(t.text_primary);
                buf.set_string(
                    cx,
                    y,
                    p,
                    st.fg(t.beside_label(t.text_muted, plane, label))
                        .remove_modifier(Modifier::BOLD),
                );
                cx += crate::ui::text::width(p) as u16 + 1;
            }
            buf.set_string(cx, y, &it.label, st);
            cx += crate::ui::text::width(&it.label) as u16;
            if let Some(x) = &it.suffix {
                buf.set_string(
                    cx + 1,
                    y,
                    x,
                    st.fg(t.legible_mark(
                        if active {
                            t.text_secondary
                        } else {
                            t.text_muted
                        },
                        plane,
                    ))
                    .remove_modifier(Modifier::BOLD),
                );
                cx += crate::ui::text::width(x) as u16 + 1;
            }
            if it.busy {
                buf.set_string(
                    cx + 1,
                    y,
                    t.glyphs.spinner_frame(ctx.interaction.tick),
                    st.fg(t.legible_mark(t.accent, plane)),
                );
                cx += 2;
            } else if it.error {
                let ink = t.tone_on(crate::theme::Tone::Error, plane);
                buf.set_string(cx + 1, y, "!", st.fg(ink));
                cx += 2;
            } else if it.dirty {
                buf.set_string(
                    cx + 1,
                    y,
                    t.glyphs.dirty,
                    st.fg(t.legible_mark(t.warning, plane)),
                );
                cx += 2;
            } else if it.done {
                let ink = t.tone_on(crate::theme::Tone::Success, plane);
                buf.set_string(
                    cx + 1,
                    y,
                    t.glyphs.done,
                    st.fg(ink).remove_modifier(Modifier::BOLD),
                );
                cx += 2;
            }

            let cid = self.close_id(i);
            let show_close = it.closable
                && match self.close_reveal {
                    CloseReveal::Always => true,
                    CloseReveal::Hover => {
                        let cs = ctx.state(cid);
                        active || s.focused || s.hovered || cs.hovered || cs.pressed
                    }
                };
            let close_plate = {
                let cw = self.close_button.width(t).max(1).min(w);
                let left = (cx + 1)
                    .saturating_sub(self.close_button.pad)
                    .max(x)
                    .min(x + w - cw);
                Rect::new(left, y, cw, 1)
            };
            if active && area.height >= 2 {
                let accent = match self.rule {
                    TabRule::Auto => !ctx.interaction.focus_hidden,
                    TabRule::Accent => true,
                    TabRule::Neutral => false,
                    TabRule::FocusOnly => focused,
                };
                let rule = if accent { t.accent } else { t.border_strong };

                for xx in x..x + w {
                    buf.set_string(
                        xx,
                        y + 1,
                        t.glyphs.bar_fill,
                        ratatui::style::Style::new().fg(rule).bg(bg),
                    );
                }
            }
            ctx.register_part(self.id, PartRef::item(Part::TAB, ItemKey::index(i)), r);

            if show_close {
                let tip = Tip::new(t.strings.tip_close_tab);
                let tip = match crate::core::keymap::label_for(&self.keymap, TabCmd::Close) {
                    Some(k) => tip.shortcut(&k),
                    None => tip,
                };
                self.close_button
                    .part_of(self.id, PartRef::item(Part::CLOSE, ItemKey::index(i)))
                    .render_in(close_plate, cx + 1, buf, ctx, st, Some(tip));
            }
            x += w + 1;
        }
        if overflow {
            let hidden_right = self.items.len().saturating_sub(self.first + fit);
            let more_right = hidden_right > 0;
            let rx = area.right().saturating_sub(right_w + more_w + new_w);
            let st = if more_right {
                ctx.title_style(self.right_id(), bg)
            } else {
                t.faint().bg(bg)
            };
            let text = if more_right {
                format!("{:>3}{}", hidden_right.min(99), t.glyphs.overflow_right)
            } else {
                "    ".into()
            };
            buf.set_string(rx, y, &text, st);
            if more_right {
                ctx.register_part(self.id, PartRef::OVERFLOW_AFTER, Rect::new(rx, y, 4, 1));
            }
        }
        if more_w > 0 {
            let hidden = self.hidden_tabs().len();
            let mx = area.right().saturating_sub(more_w + new_w);
            let r = Rect::new(mx, y, more_w, 1);
            self.more_area = r;

            let host = ratatui::style::Style::new().bg(bg);
            crate::ui::ctx::fill(buf, r, host);
            self.more_button
                .with_glyph(&format!("+{hidden}"))
                .part_of(self.id, PartRef::of(Part::OVERFLOW))
                .render_in(r, mx + 1, buf, ctx, host, Some(self.overflow_tip(t)));
        } else {
            self.more_area = Rect::ZERO;

            self.menu = None;
        }
        if self.allow_new {
            let nx = area.right().saturating_sub(new_w);
            let nid = self.new_id();
            let tip = Tip::new(t.strings.tip_new_tab);
            let tip = match crate::core::keymap::label_for(&self.keymap, TabCmd::New) {
                Some(k) => tip.shortcut(&k),
                None => tip,
            };
            IconButton::new(nid, "+")
                .part_of(self.id, PartRef::of(Part::NEW))
                .render(
                    nx,
                    y,
                    buf,
                    ctx,
                    ratatui::style::Style::new().bg(bg),
                    Some(tip),
                );
        }
        if let Some((menu, _)) = self.menu.as_mut() {
            menu.render(*buf.area(), buf, ctx);
        }
    }
}

impl RowSource for Tabs {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn label(&self, i: usize) -> &str {
        &self.items[i].label
    }
}

impl Navigable for Tabs {
    fn cursor(&self) -> Cursor {
        Cursor::new(self.cursor)
            .wrap(self.wrap)
            .axis(Axis::Horizontal)
    }
}

impl crate::core::pointer::MouseTarget for Tabs {
    type Action = TabAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn route(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::response::Response<TabAction>> {
        let menu = self.menu.as_ref().map(|(m, _)| m.id);
        (ev.on(self.id) || menu.is_some_and(|m| ev.on(m))).then(|| self.pointer(ev))
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<TabAction> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;
        if let Some((axis, delta)) = ev.wheel() {
            let step = delta.signum();
            return match axis {
                Axis::Horizontal => self.on_wheel(step, 0).into(),
                Axis::Vertical => self.on_wheel(0, step).into(),
            };
        }
        match ev.phase() {
            Some(Phase::Click) => self.on_click(ev.id).into(),
            Some(Phase::Middle) => match self.on_middle_click(ev.id) {
                Some(a) => Response::emit(a),
                None => Response::ignored(),
            },
            _ => Response::ignored(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_shows_every_tab() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::ui::measure::Measure;
        let t = crate::theme::Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("tabs"), &["orders", "invoices", "audit log"]);
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let w = {
            let ctx = RenderCtx::new(
                &t,
                crate::ui::ctx::Interaction::default(),
                &mut hits,
                &mut ring,
            );
            tabs.measure(ratatui::layout::Size::new(1, 1), &ctx).width
        };
        let min = {
            let ctx = RenderCtx::new(
                &t,
                crate::ui::ctx::Interaction::default(),
                &mut hits,
                &mut ring,
            );
            tabs.min_width(ratatui::layout::Size::new(1, 1), &ctx)
        };
        assert!(min < w, "the window and +N menu take the rest");
        for (width, hidden) in [(w, false), (w - 1, true), (min, true)] {
            let mut ctx = RenderCtx::new(
                &t,
                crate::ui::ctx::Interaction::default(),
                &mut hits,
                &mut ring,
            );
            let mut buf = Buffer::empty(Rect::new(0, 0, width, 2));
            tabs.render(Rect::new(0, 0, width, 2), &mut buf, &mut ctx, t.canvas);
            let (before, after) = tabs.hidden();
            assert_eq!(before + after > 0, hidden, "width {width}");
        }
    }

    #[test]
    fn home_end_and_configurable_keys() {
        let k = |code| Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut t = Tabs::with_items(
            WidgetId::of("t"),
            vec![
                TabItem::new("a").closable(),
                TabItem::new("b").closable(),
                TabItem::new("c").closable(),
            ],
        );
        assert_eq!(t.on_key(&k(KeyCode::End)).1, Some(TabAction::Activated(2)));
        assert_eq!(t.on_key(&k(KeyCode::Home)).1, Some(TabAction::Activated(0)));
        assert_eq!(
            t.on_key(&k(KeyCode::Char('2'))).1,
            Some(TabAction::Activated(1))
        );
        assert_eq!(
            t.on_key(&k(KeyCode::Char('x'))).1,
            Some(TabAction::Close(1))
        );
        let mut quiet = t
            .clone()
            .digit_jump(false)
            .close_keys(&[KeyCode::Delete])
            .allow_new(true)
            .new_key(Some('+'));
        assert_eq!(quiet.on_key(&k(KeyCode::Char('3'))).1, None);
        assert_eq!(quiet.on_key(&k(KeyCode::Char('x'))).1, None);
        assert_eq!(quiet.on_key(&k(KeyCode::Char('n'))).1, None);
        assert_eq!(quiet.on_key(&k(KeyCode::Char('+'))).1, Some(TabAction::New));
    }

    #[test]
    fn middle_click_closes_closable_tabs_unless_configured_off() {
        let id = WidgetId::of("strip");
        let mut t = Tabs::with_items(
            id,
            vec![TabItem::new("Here"), TabItem::new("Logs").closable()],
        );
        assert_eq!(t.on_middle_click(t.tab_id(1)), Some(TabAction::Close(1)));
        assert_eq!(t.on_middle_click(t.close_id(1)), Some(TabAction::Close(1)));
        assert_eq!(t.on_middle_click(t.tab_id(0)), None, "not closable");
        t = t.middle_click_closes(false);
        assert_eq!(t.on_middle_click(t.tab_id(1)), None);
    }
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn render_with(tabs: &mut Tabs, inter: Interaction) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 2));
        tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
        buf
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn active_tab_has_a_plane_and_the_only_accent_underline() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);
        let focused = Interaction {
            focus: Some(tabs.id),
            ..Default::default()
        };
        let buf = render_with(&mut tabs, focused);
        assert_eq!(
            row(&buf, 0).matches('▎').count(),
            1,
            "the focused strip's only gutter is on the cursor tab"
        );
        let a = tabs.areas[1];
        let b = tabs.areas[0];
        assert_eq!(
            buf[(a.x + 1, 0)].bg,
            t.ladder(crate::theme::Plane::Raised).rest,
            "active tab sits one plane up"
        );
        assert_eq!(
            buf[(b.x + 1, 0)].bg,
            t.canvas,
            "inactive tab stays on the strip plane"
        );
        let under_active = &buf[(a.x + 1, 1)];
        assert_eq!(under_active.symbol(), "━");
        assert_eq!(under_active.fg, t.accent);
        let under_inactive = &buf[(b.x + 1, 1)];
        assert_eq!(
            under_inactive.symbol(),
            "─",
            "only the active tab carries the rule"
        );
    }

    #[test]
    fn the_active_rule_is_the_accent_unless_a_layer_covers_the_strip() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);
        render_with(&mut tabs, Interaction::default());
        let a = tabs.areas[1];
        let unfocused = Interaction::default();
        let focused = Interaction {
            focus: Some(tabs.id),
            ..Default::default()
        };

        let hidden = Interaction {
            focus: Some(tabs.id),
            focus_hidden: true,
            ..Default::default()
        };
        let under = Interaction {
            focus_hidden: true,
            ..Default::default()
        };
        let rule = |rule: TabRule, i: Interaction| {
            let mut s = tabs.clone().rule(rule);
            let buf = render_with(&mut s, i);
            let c = &buf[(a.x + 1, 1)];
            assert_eq!(c.symbol(), "━", "the active tab keeps its rule");
            c.fg
        };

        assert_eq!(tabs.rule, TabRule::Auto);
        assert_eq!(rule(TabRule::Auto, unfocused), t.accent);
        assert_eq!(rule(TabRule::Auto, focused), t.accent);
        assert_eq!(rule(TabRule::Auto, hidden), t.border_strong);
        assert_eq!(rule(TabRule::Auto, under), t.border_strong);

        assert_eq!(rule(TabRule::FocusOnly, unfocused), t.border_strong);
        assert_eq!(rule(TabRule::FocusOnly, focused), t.accent);
        assert_eq!(rule(TabRule::FocusOnly, hidden), t.border_strong);

        for i in [unfocused, focused, hidden] {
            assert_eq!(rule(TabRule::Accent, i), t.accent);
            assert_eq!(rule(TabRule::Neutral, i), t.border_strong);
        }
    }

    #[test]
    fn layout_puts_the_body_one_row_under_the_strip() {
        let (strip, body) = Tabs::layout(Rect::new(2, 3, 40, 10));
        assert_eq!(strip, Rect::new(2, 3, 40, 2));
        assert_eq!(body, Rect::new(2, 6, 40, 7));
        let (strip, body) = Tabs::layout_gap(Rect::new(0, 0, 10, 10), 0);
        assert_eq!((strip.height, body.y, body.height), (2, 2, 8));

        let (strip, body) = Tabs::layout(Rect::new(0, 0, 10, 2));
        assert_eq!((strip.height, body.y, body.height), (2, 2, 0));
    }

    #[test]
    fn a_passive_strip_is_no_stop_its_tabs_are_targets_and_its_owner_shows_focus() {
        let theme = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["Overview", "Claude", "Codex"])
            .passive(true)
            .passive_focused(true);
        tabs.set_active(1);
        tabs.cursor = 1;
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 2));
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
        }
        assert!(!ring.contains(tabs.id), "not a Tab stop");
        let a = tabs.areas[2];
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(a.x + 1, 0)),
            Some(tabs.tab_id(2)),
            "each tab is still a click target"
        );
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(59, 0)),
            None,
            "the strip itself is no target"
        );
        assert_eq!(row(&buf, 0).matches('▎').count(), 1, "the owner's focus");
        assert_eq!(buf[(tabs.areas[1].x, 0)].symbol(), "▎");
        let mut tabs = tabs.passive_focused(false);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
        assert!(!row(&buf, 0).contains('▎'));
    }

    #[test]
    fn strip_focus_is_the_one_gutter_on_the_cursor_tab_apart_from_the_rule() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);
        let focused = Interaction {
            focus: Some(tabs.id),
            ..Default::default()
        };

        let buf = render_with(&mut tabs, focused);
        let a = tabs.areas[1];
        assert_eq!(row(&buf, 0).matches('▎').count(), 1, "one focus indicator");
        assert_eq!(buf[(a.x, 0)].symbol(), "▎");
        assert_eq!(buf[(a.x, 0)].fg, t.focus);
        assert_eq!(
            buf[(a.x + 1, 0)].symbol(),
            "M",
            "the gutter sits in the padding"
        );
        assert_eq!(buf[(a.x + 1, 1)].fg, t.accent);

        let buf = render_with(&mut tabs, Interaction::default());
        assert!(!row(&buf, 0).contains('▎'));
        assert_eq!(buf[(a.x + 1, 1)].fg, t.accent);
        let hidden = Interaction {
            focus: Some(tabs.id),
            focus_hidden: true,
            ..Default::default()
        };
        let buf = render_with(&mut tabs, hidden);
        assert!(!row(&buf, 0).contains('▎'));

        tabs.cursor = 2;
        let buf = render_with(&mut tabs, focused);
        assert_eq!(buf[(tabs.areas[2].x, 0)].symbol(), "▎");
        assert_eq!(row(&buf, 0).matches('▎').count(), 1);
    }

    #[test]
    fn the_overflow_button_is_a_padded_plate_with_a_tooltip() {
        let t = Theme::junie();
        let labels: Vec<String> = (0..12).map(|i| format!("Query {i}")).collect();
        let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
        let mut tabs = Tabs::new(WidgetId::of("t"), &labels);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut rest = Buffer::empty(Rect::new(0, 0, 60, 2));
        {
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            tabs.render(Rect::new(0, 0, 60, 2), &mut rest, &mut ctx, t.canvas);
        }
        let plate = hits.area_of(tabs.more_id()).expect("the +N button");
        assert_eq!(plate, tabs.more_area);
        let text: String = (plate.left()..plate.right())
            .map(|x| rest[(x, 0)].symbol().to_owned())
            .collect();
        assert!(text.starts_with(" +"), "{text:?}");
        for x in plate.left()..plate.right() {
            assert_eq!(
                hits.hit(ratatui::layout::Position::new(x, 0)),
                Some(tabs.more_id()),
                "the whole plate is the target"
            );
        }
        let hover = Interaction {
            hover: Some(tabs.more_id()),
            hover_ms: Some(10_000),
            ..Default::default()
        };
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut lit = Buffer::empty(Rect::new(0, 0, 60, 6));
        {
            let mut ctx = RenderCtx::new(&t, hover, &mut hits, &mut ring);
            tabs.render(Rect::new(0, 0, 60, 2), &mut lit, &mut ctx, t.canvas);
            ctx.flush_overlays(&mut lit);
        }
        let tips: String = (0..6).map(|y| row(&lit, y) + "\n").collect();
        for x in plate.left()..plate.right() {
            assert_ne!(lit[(x, 0)].bg, rest[(x, 0)].bg, "cell {x} lights");
            assert_eq!(lit[(x, 0)].bg, lit[(plate.x, 0)].bg, "one plate");
        }
        assert!(tips.contains(t.strings.tip_overflow), "{tips}");
    }

    #[test]
    fn hover_and_cursor_differ_from_active() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);

        let hover = Interaction {
            hover: Some(tabs.tab_id(0)),
            ..Default::default()
        };
        let buf = render_with(&mut tabs, hover);
        let b = tabs.areas[0];
        let a = tabs.areas[1];
        assert_eq!(buf[(b.x + 1, 0)].bg, t.lift(t.canvas));
        assert_ne!(
            buf[(b.x + 1, 0)].bg,
            buf[(a.x + 1, 0)].bg,
            "hover plane differs from active"
        );
        assert!(!buf[(b.x + 1, 0)].modifier.contains(Modifier::BOLD));
        assert_eq!(buf[(b.x + 1, 1)].symbol(), "─");

        tabs.cursor = 2;
        let focus = Interaction {
            focus: Some(tabs.id),
            ..Default::default()
        };
        let buf = render_with(&mut tabs, focus);
        let c = tabs.areas[2];
        assert_eq!(buf[(c.x + 1, 0)].bg, t.canvas);
        assert!(buf[(c.x + 1, 0)].modifier.contains(Modifier::BOLD));
        assert_eq!(buf[(c.x + 1, 1)].symbol(), "─");
        assert_ne!(
            buf[(c.x + 1, 0)].bg,
            buf[(a.x + 1, 0)].bg,
            "cursor plane differs from active"
        );
    }

    #[test]
    fn suffix_state_glyph_renders_after_the_label() {
        let mut tabs = Tabs::with_items(
            WidgetId::of("t"),
            vec![TabItem::new("Claude").suffix("▶"), TabItem::new("Shell")],
        );
        tabs.set_active(0);
        let buf = render_with(&mut tabs, Interaction::default());
        assert!(row(&buf, 0).contains("Claude ▶"));
    }

    #[test]
    fn a_finished_tab_shows_a_success_check_after_busy_error_and_dirty() {
        let mut tabs = Tabs::with_items(
            WidgetId::of("t"),
            vec![
                TabItem::new("Build").done(true),
                TabItem::new("Lint").done(true),
                TabItem::new("Docs"),
            ],
        );
        tabs.items[1].dirty = true;
        tabs.set_active(2);
        let buf = render_with(&mut tabs, Interaction::default());
        let t = Theme::junie();
        let a = tabs.areas[0];
        let x = a.x + 1 + 5 + 1;
        assert_eq!(buf[(x, 0)].symbol(), "✓");
        assert_eq!(buf[(x, 0)].fg, t.success);

        let b = tabs.areas[1];
        assert_eq!(buf[(b.x + 1 + 4 + 1, 0)].symbol(), t.glyphs.dirty);
        assert!(!row(&buf, 0).contains("Docs ✓"));
    }

    #[test]
    fn the_active_tabs_prefix_and_marks_read_on_its_fill_at_every_level() {
        use crate::theme::{ColorLevel, contrast_ratio};
        for level in ColorLevel::ALL {
            let theme = Theme::for_level(level);
            for state in ["busy", "dirty", "suffix"] {
                let mut item = TabItem::new("Shell").prefix("1");
                match state {
                    "busy" => item.busy = true,
                    "dirty" => item.dirty = true,
                    _ => item = item.suffix("○"),
                }
                let mut tabs =
                    Tabs::with_items(WidgetId::of("t"), vec![TabItem::new("Docs"), item]);
                tabs.set_active(1);
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
                let mut buf = Buffer::empty(Rect::new(0, 0, 60, 2));
                tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
                let a = tabs.areas[1];
                let cell = |x: u16| &buf[(x, 0)];
                let ratio = |x: u16| contrast_ratio(cell(x).fg, cell(x).bg).unwrap();
                let ctx = format!("{level:?} {state}");
                let num = a.x + 1;
                assert_eq!(cell(num).symbol(), "1", "{ctx}");
                assert!(ratio(num) >= 4.5, "{ctx}: prefix {:.2}", ratio(num));
                let mark = a.x + 1 + 2 + 5 + 1;
                assert_ne!(cell(mark).symbol(), " ", "{ctx}");
                assert!(ratio(mark) >= 3.0, "{ctx}: mark {:.2}", ratio(mark));
            }
        }
    }

    #[test]
    fn every_tab_shows_rest_hover_and_press_and_the_close_glyph_its_own() {
        use crate::theme::ColorLevel;
        for level in ColorLevel::ALL {
            let theme = Theme::for_level(level);
            let draw = |tabs: &mut Tabs, inter: Interaction| {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
                let mut buf = Buffer::empty(Rect::new(0, 0, 60, 2));
                tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
                buf
            };
            let look = |b: &Buffer, x: u16| {
                let c = &b[(x, 0)];
                let bg = if level == ColorLevel::Mono {
                    None
                } else {
                    Some(c.bg)
                };
                (bg, c.modifier)
            };
            let mut tabs = Tabs::with_items(
                WidgetId::of("t"),
                vec![
                    TabItem::new("General").closable(),
                    TabItem::new("Mounts").closable(),
                ],
            );
            tabs.set_active(1);
            draw(&mut tabs, Interaction::default());
            for i in [0, 1] {
                let id = tabs.tab_id(i);
                let x = tabs.areas[i].x + 1;
                let rest = look(&draw(&mut tabs, Interaction::default()), x);
                let hover = look(
                    &draw(
                        &mut tabs,
                        Interaction {
                            hover: Some(id),
                            ..Default::default()
                        },
                    ),
                    x,
                );
                let pressed = look(
                    &draw(
                        &mut tabs,
                        Interaction {
                            hover: Some(id),
                            pressed: Some(id),
                            ..Default::default()
                        },
                    ),
                    x,
                );
                let ctx = format!("{level:?} tab {i}");
                assert_ne!(rest, hover, "{ctx}: rest = hover");
                assert_ne!(hover, pressed, "{ctx}: hover = pressed");
                assert_ne!(rest, pressed, "{ctx}: rest = pressed");
            }

            let cid = tabs.close_id(1);
            let rest = draw(&mut tabs, Interaction::default());
            let r = tabs.areas[1];
            let cx = (r.x..r.right())
                .find(|&x| rest[(x, 0)].symbol() == "×")
                .unwrap();
            let hover = draw(
                &mut tabs,
                Interaction {
                    hover: Some(cid),
                    ..Default::default()
                },
            );
            assert_eq!(
                look(&hover, r.x + 1),
                look(&rest, r.x + 1),
                "{level:?}: tab stays at rest"
            );
            assert_ne!(look(&hover, cx), look(&rest, cx), "{level:?}: × hovers");
            let pressed = draw(
                &mut tabs,
                Interaction {
                    hover: Some(cid),
                    pressed: Some(cid),
                    ..Default::default()
                },
            );
            assert_ne!(look(&pressed, cx), look(&hover, cx), "{level:?}: × presses");
        }
    }

    fn many(n: usize) -> Tabs {
        let labels: Vec<String> = (0..n).map(|i| format!("Tab{i:02}")).collect();
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        Tabs::new(WidgetId::of("many"), &refs)
    }

    fn draw_hits(tabs: &mut Tabs, w: u16) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 2));
        tabs.render(Rect::new(0, 0, w, 2), &mut buf, &mut ctx, theme.canvas);
        drop(ctx);
        (buf, hits)
    }

    #[test]
    fn the_arrows_scroll_the_window_while_the_active_tab_sits_at_its_edge() {
        let mut tabs = many(12);
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 0);
        assert!(tabs.hidden().1 > 0, "the strip overflows");

        let right = tabs.right_id();
        tabs.on_click(right);
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 1, "the window stays scrolled after a render");
        assert_eq!(tabs.active, 0, "scrolling does not activate");
        tabs.on_click(right);
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 2);

        tabs.on_click(tabs.left_id());
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 1);

        tabs.set_active(11);
        draw_hits(&mut tabs, 40);
        assert!(tabs.areas[11].width > 0, "the new active tab is shown");
    }

    #[test]
    fn the_wheel_scrolls_the_strip_on_both_axes_and_stops_at_its_ends() {
        let mut tabs = many(12);
        let (_, hits) = draw_hits(&mut tabs, 40);
        let over = hits.hit_scroll(ratatui::layout::Position::new(10, 0));
        assert!(
            over.and_then(|id| hits.part_of(id)).map(|h| h.owner) == Some(tabs.id),
            "the wheel over the strip reaches it"
        );
        assert_eq!(tabs.on_wheel(0, 1), Outcome::Changed, "vertical wheel");
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 1);
        assert_eq!(tabs.on_wheel(1, 0), Outcome::Changed, "horizontal wheel");
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 2);
        assert_eq!(tabs.on_wheel(-1, 0), Outcome::Changed);
        assert_eq!(tabs.first, 1);
        for _ in 0..30 {
            tabs.on_wheel(0, 1);
        }
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.hidden().1, 0, "the last tab is in view");
        assert_eq!(tabs.on_wheel(0, 1), Outcome::Consumed, "at the end");
        assert_eq!(tabs.active, 0, "the wheel never activates");

        let mut few = many(2);
        draw_hits(&mut few, 40);
        assert_eq!(few.on_wheel(0, 1), Outcome::Ignored);
    }

    #[test]
    fn set_items_keeps_the_scrolled_window() {
        let mut tabs = many(12);
        draw_hits(&mut tabs, 40);
        tabs.on_wheel(0, 2);
        draw_hits(&mut tabs, 40);
        let first = tabs.first;
        assert!(first > 0);
        let items = tabs.items.clone();
        tabs.set_items(items);
        tabs.set_active(0);
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, first, "an unchanged active tab does not scroll");
    }

    fn draw_full(tabs: &mut Tabs, w: u16) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 20));
        tabs.render(Rect::new(0, 0, w, 2), &mut buf, &mut ctx, theme.canvas);
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        (buf, hits)
    }

    #[test]
    fn the_plus_n_button_lists_the_hidden_tabs_and_choosing_one_activates_it() {
        let mut tabs = many(12);
        let (buf, hits) = draw_full(&mut tabs, 40);
        let hidden = tabs.hidden().1;
        assert!(hidden > 0);
        let text = row(&buf, 0);
        assert!(text.contains(&format!("+{hidden}")), "{text}");
        let more = hits
            .area_of(tabs.more_id())
            .expect("the +N button is a target");

        tabs.on_click(tabs.more_id());
        assert!(tabs.menu_open());
        let (buf, hits) = draw_full(&mut tabs, 40);
        let menu_rows: Vec<String> = (1..20).map(|y| row(&buf, y)).collect();
        assert!(
            menu_rows.iter().any(|r| r.contains("Tab11")),
            "the last (hidden) tab is listed:\n{}",
            menu_rows.join("\n")
        );
        assert!(
            !menu_rows.iter().any(|r| r.contains("Tab00")),
            "a shown tab is not listed"
        );

        let last_row = tabs.menu.as_ref().unwrap().0.row_id(hidden - 1);
        assert!(hits.area_of(last_row).is_some());
        assert_eq!(
            hits.part_of(last_row).map(|h| h.owner),
            Some(tabs.menu_id())
        );
        assert_eq!(
            tabs.on_click(last_row).1,
            Some(TabAction::Activated(11)),
            "choosing activates"
        );
        assert!(!tabs.menu_open());
        draw_full(&mut tabs, 40);
        assert!(
            tabs.areas[11].width > 0,
            "the chosen tab is scrolled into view"
        );

        tabs.on_click(tabs.more_id());
        let esc = Key {
            code: KeyCode::Esc,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        assert_eq!(tabs.on_key(&esc).1, None);
        assert!(!tabs.menu_open());
        tabs.on_click(tabs.more_id());
        let (_, hits) = draw_full(&mut tabs, 40);
        let outside = hits
            .hit(ratatui::layout::Position::new(more.x, 15))
            .expect("the menu's backdrop");
        assert_eq!(hits.part_of(outside).map(|h| h.owner), Some(tabs.menu_id()));
        tabs.on_click(outside);
        assert!(!tabs.menu_open());

        let mut plain = many(12).overflow_menu(false);
        let (_, hits) = draw_full(&mut plain, 40);
        assert!(hits.area_of(plain.more_id()).is_none());
    }

    #[test]
    fn the_close_target_is_three_cells_wide_by_default() {
        let mut tabs = Tabs::with_items(
            WidgetId::of("t"),
            vec![
                TabItem::new("General").closable(),
                TabItem::new("Logs").closable(),
            ],
        );
        let (buf, hits) = draw_hits(&mut tabs, 60);
        let r = tabs.areas[0];
        let cx = (r.x..r.right())
            .find(|&x| buf[(x, 0)].symbol() == "×")
            .unwrap();
        let at = |x| hits.hit(ratatui::layout::Position::new(x, 0));
        assert_eq!(at(cx), Some(tabs.close_id(0)));
        assert_eq!(at(cx - 1), Some(tabs.close_id(0)));
        assert_eq!(at(cx + 1), Some(tabs.close_id(0)));
        assert_eq!(at(cx - 2), Some(tabs.tab_id(0)));

        let mut one = tabs.clone().close_target_w(1);
        let (_, hits) = draw_hits(&mut one, 60);
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(cx - 1, 0)),
            Some(one.tab_id(0))
        );
    }

    #[test]
    fn the_close_plate_lights_as_wide_as_it_hits_at_every_level() {
        use crate::theme::ColorLevel;
        for level in ColorLevel::ALL {
            let theme = Theme::for_level(level);
            let mut tabs = Tabs::with_items(
                WidgetId::of("t"),
                vec![
                    TabItem::new("Query 1").prefix("≡").closable(),
                    TabItem::new("Query 2").prefix("≡").closable(),
                ],
            );
            tabs.set_active(1);
            let draw = |tabs: &mut Tabs, inter: Interaction| {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut buf = Buffer::empty(Rect::new(0, 0, 60, 2));
                {
                    let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
                    tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
                }
                (buf, hits)
            };
            let look = |b: &Buffer, x: u16| {
                let c = &b[(x, 0)];
                ((level != ColorLevel::Mono).then_some(c.bg), c.modifier)
            };
            for i in [0, 1] {
                let cid = tabs.close_id(i);
                let (rest, hits) = draw(&mut tabs, Interaction::default());
                let plate = hits.area_of(cid).unwrap();
                assert_eq!(plate.width, 3);
                assert_eq!(rest[(plate.x + 1, 0)].symbol(), theme.glyphs.close);
                for (name, inter) in [
                    (
                        "hover",
                        Interaction {
                            hover: Some(cid),
                            ..Default::default()
                        },
                    ),
                    (
                        "pressed",
                        Interaction {
                            hover: Some(cid),
                            pressed: Some(cid),
                            ..Default::default()
                        },
                    ),
                ] {
                    let (lit, _) = draw(&mut tabs, inter);
                    let ctx = format!("{level:?} tab {i} {name}");
                    for x in plate.left()..plate.right() {
                        assert_ne!(look(&lit, x), look(&rest, x), "{x}: {ctx}");
                        assert_eq!(look(&lit, x), look(&lit, plate.x), "{x}: {ctx}");
                    }

                    let label = tabs.areas[i].x + 3;
                    assert_eq!(look(&lit, label), look(&rest, label), "{ctx}");
                }
            }
        }
    }

    #[test]
    fn close_reveal_hover_shows_the_x_on_the_active_and_the_hovered_tab_only() {
        let mut tabs = Tabs::with_items(
            WidgetId::of("t"),
            vec![
                TabItem::new("General").closable(),
                TabItem::new("Logs").closable(),
                TabItem::new("Mounts").closable(),
            ],
        )
        .close_reveal(CloseReveal::Hover);
        tabs.set_active(0);
        let (buf, hits) = draw_hits(&mut tabs, 60);
        let has_x = |buf: &Buffer, r: Rect| (r.x..r.right()).any(|x| buf[(x, 0)].symbol() == "×");
        assert!(has_x(&buf, tabs.areas[0]), "the active tab keeps its ×");
        assert!(!has_x(&buf, tabs.areas[1]));
        assert!(hits.area_of(tabs.close_id(1)).is_none(), "no hidden target");
        let widths: Vec<u16> = tabs.areas.iter().map(|r| r.width).collect();

        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 2));
        {
            let inter = Interaction {
                hover: Some(tabs.tab_id(1)),
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
            tabs.render(Rect::new(0, 0, 60, 2), &mut buf, &mut ctx, theme.canvas);
        }
        assert!(has_x(&buf, tabs.areas[1]));
        assert_eq!(hits.area_of(tabs.close_id(1)).map(|r| r.width), Some(3));
        assert!(!has_x(&buf, tabs.areas[2]));
        assert_eq!(
            widths,
            tabs.areas.iter().map(|r| r.width).collect::<Vec<_>>()
        );
    }

    #[test]
    fn enter_on_the_strip_flashes_the_cursor_tab() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts"]);
        tabs.set_active(1);
        tabs.cursor = 0;
        let id = tabs.id;
        let buf = render_with(
            &mut tabs,
            Interaction {
                focus: Some(id),
                flash: Some(id),
                ..Default::default()
            },
        );
        let b = tabs.areas[0];
        assert_eq!(
            buf[(b.x + 1, 0)].bg,
            t.ladder(crate::theme::Plane::Canvas).pressed
        );
    }
}
