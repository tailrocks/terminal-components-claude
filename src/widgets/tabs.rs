//! Horizontal tab strip. One focus stop; ← → move the cursor and switch.
//! Active tab: one plane up with an underline (accent while the strip has
//! focus, neutral otherwise: see [`TabRule`]). Keyboard cursor on
//! another tab: two planes up and bold. Hover: two planes up. Tabs never
//! carry the `▎` gutter — that glyph belongs to rows and fields.
//! Tabs carry state glyphs (dirty, busy, error, a trailing state suffix) and
//! a close affordance; the strip scrolls with `‹ ›` indicators instead of
//! shrinking labels.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::theme::Kind;
use crate::ui::ctx::RenderCtx;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabItem {
    pub label: String,
    pub dirty: bool,
    pub busy: bool,
    pub error: bool,
    pub closable: bool,
    /// Muted prefix such as a type glyph or a schema.
    pub prefix: Option<String>,
    /// Trailing state glyph (`▶ ● ○ ◆`), muted unless the tab is active.
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
}

#[derive(Debug, Clone)]
pub struct Tabs {
    pub id: WidgetId,
    pub items: Vec<TabItem>,
    pub active: usize,
    pub cursor: usize,
    /// First visible tab (strip scroll).
    pub first: usize,
    pub areas: Vec<Rect>,
    /// Show a trailing `+` that emits `TabEvent::New`.
    pub allow_new: bool,
    /// How the active tab's underline is coloured. Default
    /// [`TabRule::Auto`]: accent only while the strip has keyboard focus.
    pub rule: TabRule,
    /// A middle click on a closable tab closes it (browsers, editors).
    /// Default `true`.
    pub middle_click_closes: bool,
    /// ←/→ past the last tab lands on the first and back. Default off.
    pub wrap: bool,
    /// Which keys move between tabs (default [`NavKeys::DEFAULT`]: ←/→,
    /// `h`/`l`, Home/End, `g`/`G`).
    pub nav: NavKeys,
    /// The strip's action keys (default [`TabAction::default_keymap`]:
    /// `x`/Delete close, `n` new, digits 1-9 jump).
    pub keymap: KeyMap<TabAction>,
    /// Hit width of a tab's `×` in cells, centred on the glyph and kept
    /// inside the tab (a one-cell target is too small to hit reliably).
    /// Default 3.
    pub close_target_w: u16,
    /// Number of tabs that fit in the last render.
    fit: usize,
    /// Largest useful [`Self::first`] in the last render: the window that
    /// shows the last tab.
    max_first: usize,
    /// The active tab the window last scrolled into view. The window is
    /// independent of the active tab: `‹`/`›` and the wheel move it freely,
    /// and a render only brings the active tab back when it changed.
    revealed: Option<usize>,
}

/// Colour of the active tab's underline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabRule {
    /// Accent while the strip holds keyboard focus, neutral
    /// (`border_strong`) otherwise: an active-but-unfocused tab is a fact,
    /// not a "you are here" cue, so it never competes with the focus bar.
    #[default]
    Auto,
    /// Always the accent.
    Accent,
    /// Always neutral (a secondary strip under a document strip).
    Neutral,
}

/// The keys that close the cursor tab by default.
pub const DEFAULT_CLOSE_KEYS: &[KeyCode] = &[KeyCode::Char('x'), KeyCode::Delete];

/// A [`Tabs`] action a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAction {
    /// Close the cursor tab (when it is closable). Default `x` and Delete.
    Close,
    /// Ask for a new tab (with [`Tabs::allow_new`]). Default `n`.
    New,
    /// Activate tab `i` (0-based). Default digits `1`-`9`.
    Jump(usize),
}

impl TabAction {
    /// The digit bindings `1`-`9` (see [`Tabs::digit_jump`]).
    pub fn digit_keys() -> KeyMap<Self> {
        ('1'..='9')
            .enumerate()
            .map(|(i, c)| (KeyBinding::char(c), TabAction::Jump(i)))
            .collect()
    }
}

impl Actions for TabAction {
    fn default_keymap() -> KeyMap<Self> {
        let mut map: KeyMap<Self> = DEFAULT_CLOSE_KEYS
            .iter()
            .map(|&code| (KeyBinding::key(code), TabAction::Close))
            .collect();
        map.push((KeyBinding::char('n'), TabAction::New));
        map.extend(Self::digit_keys());
        map
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabEvent {
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
            nav: NavKeys::DEFAULT,
            keymap: TabAction::default_keymap(),
            close_target_w: 3,
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
            nav: NavKeys::DEFAULT,
            keymap: TabAction::default_keymap(),
            close_target_w: 3,
            fit: 0,
            max_first: 0,
            revealed: None,
        }
    }

    pub fn rule(mut self, rule: TabRule) -> Self {
        self.rule = rule;
        self
    }

    /// See [`Self::wrap`].
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<TabAction>) -> Self {
        self.keymap = keymap;
        self
    }

    /// Digits 1-9 jump to that tab (default on): adds or removes the
    /// [`TabAction::Jump`] bindings in [`Self::keymap`].
    pub fn digit_jump(mut self, on: bool) -> Self {
        self.keymap
            .retain(|(_, a)| !matches!(a, TabAction::Jump(_)));
        if on {
            self.keymap.extend(TabAction::digit_keys());
        }
        self
    }

    /// The keys that close the cursor tab (default `x` and Delete):
    /// replaces the [`TabAction::Close`] bindings in [`Self::keymap`].
    pub fn close_keys(mut self, keys: &[KeyCode]) -> Self {
        self.keymap.retain(|(_, a)| *a != TabAction::Close);
        self.keymap.extend(
            keys.iter()
                .map(|&code| (KeyBinding::key(code), TabAction::Close)),
        );
        self
    }

    /// The letter that asks for a new tab (default `Some('n')`): replaces
    /// the [`TabAction::New`] bindings in [`Self::keymap`].
    pub fn new_key(mut self, key: Option<char>) -> Self {
        self.keymap.retain(|(_, a)| *a != TabAction::New);
        if let Some(c) = key {
            self.keymap.push((KeyBinding::char(c), TabAction::New));
        }
        self
    }

    pub fn allow_new(mut self, on: bool) -> Self {
        self.allow_new = on;
        self
    }

    /// See [`Self::close_target_w`].
    pub fn close_target_w(mut self, cells: u16) -> Self {
        self.close_target_w = cells;
        self
    }

    /// Replace the tabs, keeping the strip's scroll window and active
    /// index (clamped): an app that rebuilds its items every update keeps
    /// the window the user scrolled to.
    pub fn set_items(&mut self, items: Vec<TabItem>) {
        self.items = items;
        let last = self.items.len().saturating_sub(1);
        self.active = self.active.min(last);
        self.cursor = self.cursor.min(last);
        self.first = self.first.min(last);
    }

    /// Scroll the strip's window by `dx` (or `dy` when there is no
    /// horizontal component: a plain wheel over a horizontal strip scrolls
    /// it sideways, as macOS does for horizontal-only scroll views). The
    /// active tab does not change.
    pub fn on_wheel(&mut self, dx: i32, dy: i32) -> Outcome {
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

    /// A middle click on `id`: closes the closable tab under it (or whose
    /// × it is) when [`Self::middle_click_closes`] is on.
    pub fn on_middle_click(&self, id: WidgetId) -> Option<TabEvent> {
        if !self.middle_click_closes {
            return None;
        }
        let i = self
            .locate(id)
            .or_else(|| (0..self.items.len()).find(|&i| self.close_id(i) == id))?;
        self.items[i].closable.then_some(TabEvent::Close(i))
    }

    pub fn tab_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn close_id(&self, i: usize) -> WidgetId {
        self.id.child(i).sub("close")
    }
    pub fn new_id(&self) -> WidgetId {
        self.id.sub("new")
    }
    pub fn left_id(&self) -> WidgetId {
        self.id.sub("left")
    }
    pub fn right_id(&self) -> WidgetId {
        self.id.sub("right")
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.items.len()).find(|&i| self.tab_id(i) == id)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == self.new_id()
            || id == self.left_id()
            || id == self.right_id()
            || self.locate(id).is_some()
            || (0..self.items.len()).any(|i| self.close_id(i) == id)
    }

    /// Tabs hidden before the window and after it.
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
            // a re-assertion of the same tab (an app syncing its state)
            // leaves a window the user scrolled alone
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

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TabEvent>) {
        let action = lookup(&self.keymap, key);
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) && action.is_none() {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        if self.items.is_empty() {
            return match (action, key.code) {
                (_, KeyCode::Enter) | (Some(TabAction::New), _) if self.allow_new => {
                    (Outcome::Changed, Some(TabEvent::New))
                }
                _ => (Outcome::Ignored, None),
            };
        }
        match action {
            Some(TabAction::Jump(i)) => {
                return if i < self.items.len() {
                    self.set_active(i);
                    (Outcome::Changed, Some(TabEvent::Activated(i)))
                } else {
                    (Outcome::Ignored, None)
                };
            }
            Some(TabAction::Close) if self.items[self.cursor].closable => {
                return (Outcome::Changed, Some(TabEvent::Close(self.cursor)));
            }
            Some(TabAction::New) if self.allow_new => {
                return (Outcome::Changed, Some(TabEvent::New));
            }
            _ => {}
        }
        let last = self.items.len() - 1;
        let to = match self.nav.action(key).filter(|_| key.plain()) {
            Some(Nav::Left) => Some(crate::core::nav::step_index(
                self.cursor,
                self.items.len(),
                -1,
                self.wrap,
            )),
            Some(Nav::Right) => Some(crate::core::nav::step_index(
                self.cursor,
                self.items.len(),
                1,
                self.wrap,
            )),
            Some(Nav::Home) => Some(0),
            Some(Nav::End) => Some(last),
            _ => None,
        };
        if let Some(to) = to {
            self.set_active(to);
            return (Outcome::Changed, Some(TabEvent::Activated(self.active)));
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.set_active(self.cursor);
                (Outcome::Changed, Some(TabEvent::Activated(self.active)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<TabEvent>) {
        if id == self.new_id() {
            return (Outcome::Changed, Some(TabEvent::New));
        }
        if id == self.left_id() {
            self.first = self.first.saturating_sub(1);
            return (Outcome::Changed, None);
        }
        if id == self.right_id() {
            self.first = (self.first + 1).min(self.max_first);
            return (Outcome::Changed, None);
        }
        for i in 0..self.items.len() {
            if self.close_id(i) == id {
                return (Outcome::Changed, Some(TabEvent::Close(i)));
            }
        }
        if let Some(i) = self.locate(id) {
            self.set_active(i);
            return (Outcome::Changed, Some(TabEvent::Activated(i)));
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
        if it.dirty || it.busy || it.error {
            w += 2;
        }
        if it.closable {
            w += 2;
        }
        w
    }

    /// Two rows: labels, then the underline row.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        // the strip is a focus stop with the geometry it draws in; tab and
        // button regions registered after it take precedence under the mouse.
        // It is a scroll region too: the wheel moves its window
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
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
        // reserve space for the overflow indicators and the new-tab button
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let mut x = area.x;
        // figure out how many fit from `first`
        let widths: Vec<u16> = self.items.iter().map(|it| self.tab_width(it)).collect();
        let total: u16 = widths.iter().map(|w| w + 1).sum();
        let overflow = total + new_w > area.width;
        // `‹N` / `N›` hidden counts (the grid idiom); N is at most two digits
        let left_w: u16 = if overflow { 4 } else { 0 };
        let right_w: u16 = if overflow { 4 } else { 0 };
        let avail = area.width.saturating_sub(left_w + right_w + new_w);
        // the furthest the window scrolls: the first tab of the run that
        // ends with the last tab and fits
        self.max_first = {
            let mut used = 0u16;
            let mut f = self.items.len();
            while f > 0 && used + widths[f - 1] + 1 <= avail {
                used += widths[f - 1] + 1;
                f -= 1;
            }
            if overflow {
                f.min(self.items.len().saturating_sub(1))
            } else {
                0
            }
        };
        // the window only follows the active tab when it changed (a click,
        // a key, the app); a strip the user scrolled stays where it is
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
                ctx.clickable(self.left_id(), Rect::new(x, y, 4, 1));
            }
            x += 4;
        }
        // index is needed for ids and the parallel `widths` vector
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
            // the × is its own control: hovering it does not hover the tab
            let mut s = ctx.child_state(self.id, tid, i == self.cursor);
            s.focused = focused && i == self.cursor;
            let active = i == self.active;
            s.selected = active;
            // the active tab sits one plane up; hover and press walk that
            // plane's ladder (or the strip's for the other tabs); the
            // keyboard cursor is weight on the strip's own plane, so it never
            // reads as hover. A strip draws no `▎`: its focus is the accent
            // rule (`TabRule`)
            let st = t.interactive(Kind::Tab, s, t.plane_of(bg)).style;
            crate::ui::ctx::fill(buf, r, st);
            let mut cx = x + 1;
            if let Some(p) = &it.prefix {
                buf.set_string(
                    cx,
                    y,
                    p,
                    st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
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
                    st.fg(if active {
                        t.text_secondary
                    } else {
                        t.text_muted
                    })
                    .remove_modifier(Modifier::BOLD),
                );
                cx += crate::ui::text::width(x) as u16 + 1;
            }
            if it.busy {
                buf.set_string(
                    cx + 1,
                    y,
                    t.glyphs.spinner_frame(ctx.interaction.tick),
                    st.fg(t.accent),
                );
                cx += 2;
            } else if it.error {
                buf.set_string(cx + 1, y, "!", st.fg(t.error));
                cx += 2;
            } else if it.dirty {
                buf.set_string(cx + 1, y, t.glyphs.dirty, st.fg(t.warning));
                cx += 2;
            }
            // the ×'s hit target: `close_target_w` cells centred on the
            // glyph, kept inside the tab
            let close_hit = {
                let cw = self.close_target_w.max(1).min(w);
                let left = (cx + 1).saturating_sub((cw - 1) / 2).max(x);
                let left = left.min(x + w - cw);
                Rect::new(left, y, cw, 1)
            };
            if it.closable {
                let cid = self.close_id(i);
                let cs = t.close_glyph(ctx.state(cid), st);
                buf.set_string(cx + 1, y, t.glyphs.close, cs);
            }
            if active && area.height >= 2 {
                let accent = match self.rule {
                    TabRule::Auto => focused,
                    TabRule::Accent => true,
                    TabRule::Neutral => false,
                };
                let rule = if accent { t.accent } else { t.border_strong };
                // the rule spans exactly the tab's plane, edge to edge
                for xx in x..x + w {
                    buf.set_string(
                        xx,
                        y + 1,
                        t.glyphs.bar_fill,
                        ratatui::style::Style::new().fg(rule).bg(bg),
                    );
                }
            }
            ctx.clickable(tid, r);
            // close button on top
            if it.closable {
                ctx.clickable(self.close_id(i), close_hit);
            }
            x += w + 1;
        }
        if overflow {
            let hidden_right = self.items.len().saturating_sub(self.first + fit);
            let more_right = hidden_right > 0;
            let rx = area.right().saturating_sub(right_w + new_w);
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
                ctx.clickable(self.right_id(), Rect::new(rx, y, 4, 1));
            }
        }
        if self.allow_new {
            let nx = area.right().saturating_sub(new_w);
            let nid = self.new_id();
            let st = ctx.title_style(nid, bg);
            buf.set_string(nx, y, " + ", st);
            ctx.clickable(nid, Rect::new(nx, y, 3, 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(t.on_key(&k(KeyCode::End)).1, Some(TabEvent::Activated(2)));
        assert_eq!(t.on_key(&k(KeyCode::Home)).1, Some(TabEvent::Activated(0)));
        assert_eq!(
            t.on_key(&k(KeyCode::Char('2'))).1,
            Some(TabEvent::Activated(1))
        );
        assert_eq!(t.on_key(&k(KeyCode::Char('x'))).1, Some(TabEvent::Close(1)));
        let mut quiet = t
            .clone()
            .digit_jump(false)
            .close_keys(&[KeyCode::Delete])
            .allow_new(true)
            .new_key(Some('+'));
        assert_eq!(quiet.on_key(&k(KeyCode::Char('3'))).1, None);
        assert_eq!(quiet.on_key(&k(KeyCode::Char('x'))).1, None);
        assert_eq!(quiet.on_key(&k(KeyCode::Char('n'))).1, None);
        assert_eq!(quiet.on_key(&k(KeyCode::Char('+'))).1, Some(TabEvent::New));
    }

    #[test]
    fn middle_click_closes_closable_tabs_unless_configured_off() {
        let id = WidgetId::of("strip");
        let mut t = Tabs::with_items(
            id,
            vec![TabItem::new("Here"), TabItem::new("Logs").closable()],
        );
        assert_eq!(t.on_middle_click(t.tab_id(1)), Some(TabEvent::Close(1)));
        assert_eq!(t.on_middle_click(t.close_id(1)), Some(TabEvent::Close(1)));
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
    fn active_tab_has_a_plane_and_the_only_accent_underline_and_no_gutter() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);
        let focused = Interaction {
            focus: Some(tabs.id),
            ..Default::default()
        };
        let buf = render_with(&mut tabs, focused);
        assert!(
            !row(&buf, 0).contains('▎'),
            "no gutter glyph in a tab strip"
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
    fn an_unfocused_strip_draws_the_active_rule_neutral() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);
        let buf = render_with(&mut tabs, Interaction::default());
        let a = tabs.areas[1];
        let rule = &buf[(a.x + 1, 1)];
        assert_eq!(rule.symbol(), "━", "the active tab keeps its rule");
        assert_eq!(rule.fg, t.border_strong, "but not the focus colour");
        // a hidden focus (a layer above the page) is unfocused too
        let hidden = Interaction {
            focus: Some(tabs.id),
            focus_hidden: true,
            ..Default::default()
        };
        let buf = render_with(&mut tabs, hidden);
        assert_eq!(buf[(a.x + 1, 1)].fg, t.border_strong);
        // explicit policies
        let mut accent = tabs.clone().rule(TabRule::Accent);
        let buf = render_with(&mut accent, Interaction::default());
        assert_eq!(buf[(a.x + 1, 1)].fg, t.accent);
        let mut neutral = tabs.clone().rule(TabRule::Neutral);
        let focused = Interaction {
            focus: Some(tabs.id),
            ..Default::default()
        };
        let buf = render_with(&mut neutral, focused);
        assert_eq!(buf[(a.x + 1, 1)].fg, t.border_strong);
    }

    #[test]
    fn hover_and_cursor_differ_from_active() {
        let t = Theme::junie();
        let mut tabs = Tabs::new(WidgetId::of("t"), &["General", "Mounts", "Roles"]);
        tabs.set_active(1);
        // hover on an inactive tab: the strip's hover step, not bold, no underline
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
        // keyboard cursor on an inactive tab: bold on the strip plane (never
        // the hover plane), still no underline
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
            // the × of the active tab: its hover is not the tab's, and it presses
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
        // the active tab is the first one shown: › still scrolls
        let right = tabs.right_id();
        tabs.on_click(right);
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 1, "the window stays scrolled after a render");
        assert_eq!(tabs.active, 0, "scrolling does not activate");
        tabs.on_click(right);
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 2);
        // ‹ scrolls back
        tabs.on_click(tabs.left_id());
        draw_hits(&mut tabs, 40);
        assert_eq!(tabs.first, 1);
        // activating a tab off-screen brings it back into the window
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
            over.is_some_and(|id| tabs.owns(id)),
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
        // a strip that fits ignores the wheel
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

    #[test]
    fn the_close_target_is_three_cells_wide_by_default() {
        let mut tabs = Tabs::with_items(
            WidgetId::of("t"),
            vec![TabItem::new("General").closable(), TabItem::new("Logs").closable()],
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
        // configurable back to one cell
        let mut one = tabs.clone().close_target_w(1);
        let (_, hits) = draw_hits(&mut one, 60);
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(cx - 1, 0)),
            Some(one.tab_id(0))
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
