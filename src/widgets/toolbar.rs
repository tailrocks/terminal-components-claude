//! A toolbar: a row of commands that is one focus stop (WAI-ARIA APG
//! toolbar pattern; a macOS window toolbar).
//!
//! Tab enters and leaves the toolbar as one stop; inside it ←/→ move the
//! cursor between items (skipping disabled ones by default), Home/End jump
//! to the ends, Enter or Space activates the cursor item and flips a
//! toggle. A click activates an item directly. Items look like subtle
//! buttons (a toggle shows its on/off dot), each with an optional shortcut
//! shown muted after its label and a tooltip; separators group them. The
//! focused toolbar draws the one `▎` gutter on its cursor item, as a tab
//! strip does on its cursor tab.
//!
//! Items that do not fit go behind an overflow button at the right end
//! (`»`, [`Toolbar::more_button`]), the last stop of the arrow keys: Enter,
//! Space, ↓ or a click opens a [`ContextMenu`] of the hidden items (their
//! shortcuts, toggle state and why a disabled one is unavailable) through
//! the shared overlay layer, as macOS window toolbars and the APG toolbar
//! keep every command reachable. [`Toolbar::overflow`] turns it off.
//!
//! Navigation goes through [`Cursor`] (one stepping rule for every
//! collection); the keys are [`NavKeys`], overridable per toolbar.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::NavKeys;
use crate::core::list::{Cursor, Navigable, RowFn, RowSource};
use crate::theme::{ButtonKind, Kind};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::ui::tooltip::Tip;
use crate::widgets::icon_button::IconButton;
use crate::widgets::menu::{ContextMenu, MenuAction, MenuItem};

/// One command of a [`Toolbar`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolItem {
    pub label: String,
    /// The key that runs the command anywhere in the app (shown muted after
    /// the label, and in the tooltip). The owner binds it; the toolbar only
    /// shows it.
    pub shortcut: Option<String>,
    pub disabled: bool,
    /// A toggle's state: `Some(on)` draws the on/off dot and flips on
    /// activation. `None`: a plain command.
    pub on: Option<bool>,
    /// A separator follows the item.
    pub separator_after: bool,
    /// What the command does, shown on hover.
    pub tip: Option<String>,
    /// Why a disabled command is unavailable, shown in the overflow menu
    /// in place of the shortcut ([`MenuItem::reason`]).
    pub reason: Option<String>,
}

impl ToolItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            ..Default::default()
        }
    }
    pub fn shortcut(mut self, s: &str) -> Self {
        self.shortcut = Some(s.to_owned());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Make the item a toggle, starting `on`.
    pub fn toggle(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }
    pub fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }
    pub fn tip(mut self, tip: &str) -> Self {
        self.tip = Some(tip.to_owned());
        self
    }
    /// Disable the item and say why (shown in the overflow menu).
    pub fn reason(mut self, why: &str) -> Self {
        self.reason = Some(why.to_owned());
        self.disabled = true;
        self
    }

    /// Cells the item takes: gutter, dot, label, shortcut, padding.
    fn width(&self) -> u16 {
        let mut w = 1 + width(&self.label) as u16 + 1;
        if self.on.is_some() {
            w += 2;
        }
        if let Some(s) = &self.shortcut {
            w += width(s) as u16 + 1;
        }
        w
    }
}

/// What a [`Toolbar`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// The command at this index ran.
    Activated(usize),
    /// The toggle at this index flipped to the given state.
    Toggled(usize, bool),
}

#[derive(Debug, Clone)]
pub struct Toolbar {
    pub id: WidgetId,
    pub items: Vec<ToolItem>,
    /// The item ←/→ move and Enter activates.
    pub cursor: usize,
    /// Which keys move the cursor (default [`NavKeys::default`]: ←/→ and
    /// Home/End; the vim letters under the Vim profile).
    pub nav: NavKeys,
    /// → on the last item lands on the first. Default off (APG: optional;
    /// macOS toolbars stop at the ends).
    pub wrap: bool,
    /// Keyboard steps pass over disabled items. Default on; off keeps them
    /// reachable so a screen reader user can find them (APG allows both).
    pub skip_disabled: bool,
    /// Cells between items. Default 1.
    pub gap: u16,
    /// Where each item was drawn in the last render (`Rect::ZERO` when it
    /// did not fit).
    pub areas: Vec<Rect>,
    /// Items that do not fit go behind [`Self::more_button`]. Default on;
    /// off: they are not drawn (the app gives the toolbar its
    /// [`Self::width`]).
    pub overflow: bool,
    /// The overflow button: a padded plate on the title ladder. Its glyph
    /// `None` is the theme's [`crate::theme::Glyphs::more_titles`] (`»`,
    /// ASCII `>`); tooltip `Show hidden`
    /// ([`crate::theme::Strings::tip_overflow`]).
    pub more_button: IconButton,
    /// The open overflow menu and the item each of its rows stands for.
    menu: Option<(ContextMenu, Vec<usize>)>,
    /// Where the overflow button was drawn (`Rect::ZERO`: nothing hidden).
    more_area: Rect,
    /// Items behind the overflow button as of the last render.
    hidden: Vec<usize>,
    /// The theme's toggle marks as of the last render (the menu's rows).
    marks: (&'static str, &'static str),
}

impl Toolbar {
    pub fn new(id: WidgetId, items: Vec<ToolItem>) -> Self {
        let mut t = Self {
            id,
            items,
            cursor: 0,
            nav: NavKeys::default(),
            wrap: false,
            skip_disabled: true,
            gap: 1,
            areas: Vec::new(),
            overflow: true,
            more_button: IconButton {
                glyph: None,
                ..IconButton::new(WidgetId::of("toolbar.more"), "")
            },
            menu: None,
            more_area: Rect::ZERO,
            hidden: Vec::new(),
            marks: ("●", "○"),
        };
        t.cursor = (0..t.items.len())
            .find(|&i| !t.items[i].disabled)
            .unwrap_or(0);
        t
    }
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }
    /// See [`Self::wrap`].
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }
    /// See [`Self::skip_disabled`].
    pub fn skip_disabled(mut self, on: bool) -> Self {
        self.skip_disabled = on;
        self
    }
    /// See [`Self::gap`].
    pub fn gap(mut self, cells: u16) -> Self {
        self.gap = cells;
        self
    }
    /// See [`Self::overflow`].
    pub fn overflow(mut self, on: bool) -> Self {
        self.overflow = on;
        self
    }
    /// See [`Self::more_button`].
    pub fn more_button(mut self, button: IconButton) -> Self {
        self.more_button = button;
        self
    }
    /// The overflow button showing `glyph` (`…`, `⋯`) instead of the
    /// theme's.
    pub fn more_glyph(mut self, glyph: &str) -> Self {
        self.more_button.glyph = Some(glyph.to_owned());
        self
    }

    /// The overflow button's id.
    pub fn more_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::OVERFLOW))
    }

    /// The overflow menu's id.
    pub fn menu_id(&self) -> WidgetId {
        self.id.sub("more-menu")
    }

    /// The overflow menu is open: it owns the keyboard.
    pub fn menu_open(&self) -> bool {
        self.menu.is_some()
    }

    /// Items behind the overflow button as of the last render, in order.
    pub fn hidden(&self) -> &[usize] {
        &self.hidden
    }

    /// The cursor is on the overflow button (index `items.len()`).
    pub fn on_more(&self) -> bool {
        self.cursor == self.items.len() && !self.more_area.is_empty()
    }

    /// The stops ←/→ visit, in order: the drawn items, then the overflow
    /// button (`items.len()`) when it is shown.
    fn stops(&self) -> Vec<usize> {
        let n = self.items.len();
        let drawn = |i: usize| self.areas.get(i).is_none_or(|r| !r.is_empty());
        let mut v: Vec<usize> = (0..n).filter(|&i| drawn(i)).collect();
        if !self.more_area.is_empty() {
            v.push(n);
        }
        v
    }

    fn open_menu(&mut self) {
        if self.hidden.is_empty() {
            return;
        }
        let (on, off) = self.marks;
        let rows = self
            .hidden
            .iter()
            .map(|&i| {
                let it = &self.items[i];
                let label = match it.on {
                    Some(true) => format!("{on} {}", it.label),
                    Some(false) => format!("{off} {}", it.label),
                    None => it.label.clone(),
                };
                let mut m = MenuItem::new(label)
                    .disabled(it.disabled)
                    .separator_if(it.separator_after);
                if let Some(sc) = &it.shortcut {
                    m = m.shortcut(sc.clone());
                }
                if it.disabled
                    && let Some(r) = &it.reason
                {
                    m = m.reason(r.clone());
                }
                m
            })
            .collect();
        let menu =
            ContextMenu::new(self.menu_id(), rows).anchor_control(self.more_id(), self.more_area);
        self.menu = Some((menu, self.hidden.clone()));
    }

    fn menu_event(&mut self, ev: Option<MenuAction>) -> Option<ToolbarAction> {
        match ev? {
            MenuAction::Chosen(k) => {
                let (_, rows) = self.menu.take()?;
                let a = self.activate(*rows.get(k)?);
                // focus returns to the button that opened the menu (APG)
                self.cursor = self.items.len();
                a
            }
            MenuAction::Dismissed => {
                self.menu = None;
                None
            }
        }
    }

    /// The hit id of item `i`.
    pub fn item_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::ROW, ItemKey::index(i)))
    }

    /// Which item `id` is (the inverse [`Self::on_click`] needs until S15
    /// hands it the routed part).
    fn item_at(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..self.items.len())
    }

    /// Cells the toolbar takes with every item shown.
    pub fn width(&self) -> u16 {
        let n = self.items.len() as u16;
        let seps = self.items.iter().filter(|i| i.separator_after).count() as u16;
        self.items.iter().map(ToolItem::width).sum::<u16>()
            + self.gap * n.saturating_sub(1)
            + seps * (1 + self.gap)
    }

    fn activate(&mut self, i: usize) -> Option<ToolbarAction> {
        let it = self.items.get_mut(i)?;
        if it.disabled {
            return None;
        }
        self.cursor = i;
        Some(match it.on.as_mut() {
            Some(on) => {
                *on = !*on;
                ToolbarAction::Toggled(i, *on)
            }
            None => ToolbarAction::Activated(i),
        })
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ToolbarAction>) {
        if let Some((menu, _)) = self.menu.as_mut() {
            let (o, ev) = menu.on_key(key);
            return (o, self.menu_event(ev));
        }
        if self.items.is_empty() {
            return (Outcome::Ignored, None);
        }
        if self.on_more()
            && key.plain()
            && matches!(
                key.code,
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down
            )
        {
            // a menu button: Enter, Space and ↓ open it (APG)
            self.open_menu();
            return (Outcome::Changed, None);
        }
        if key.plain() && matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            return match self.activate(self.cursor) {
                Some(a) => (Outcome::Changed, Some(a)),
                None => (Outcome::Consumed, None),
            };
        }
        let Some(nav) = self.nav.action(key).filter(|_| key.plain()) else {
            return (Outcome::Ignored, None);
        };
        // step along the stops: the drawn items, then the overflow button
        let n = self.items.len();
        let stops = self.stops();
        let at = stops
            .iter()
            .position(|&i| i == self.cursor)
            .or_else(|| stops.iter().position(|&i| i == n))
            .unwrap_or(0);
        let enabled = |k: usize| stops[k] == n || !self.items[stops[k]].disabled;
        let label = |k: usize| {
            self.items
                .get(stops[k])
                .map_or("", |it: &ToolItem| it.label.as_str())
        };
        let rows = RowFn {
            len: stops.len(),
            enabled: &enabled,
            label: &label,
        };
        let moved = Cursor::new(at)
            .wrap(self.wrap)
            .axis(Axis::Horizontal)
            .skip_disabled(self.skip_disabled)
            .apply(nav, &rows, 1);
        if let Some(k) = moved.index() {
            self.cursor = stops[k];
        }
        (moved.outcome(), None)
    }

    /// A click on `id`: the item under it runs; the overflow button opens
    /// or closes its menu; a row of the open menu runs its item.
    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<ToolbarAction>) {
        if let Some((menu, _)) = self.menu.as_mut()
            && menu.contains_id(id)
        {
            let ev = menu.on_click(id);
            return (Outcome::Changed, self.menu_event(ev));
        }
        if id == self.more_id() {
            self.cursor = self.items.len();
            if self.menu.take().is_none() {
                self.open_menu();
            }
            return (Outcome::Changed, None);
        }
        // any other click on the toolbar closes an open overflow menu
        self.menu = None;
        match self.item_at(id) {
            Some(i) => match self.activate(i) {
                Some(a) => (Outcome::Changed, Some(a)),
                None => (Outcome::Consumed, None),
            },
            None => (Outcome::Ignored, None),
        }
    }

    /// One row: the items left to right. Items that do not fit go behind
    /// the overflow button at the right end ([`Self::overflow`]); with it
    /// off they are not drawn.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        let n = self.items.len();
        self.areas = vec![Rect::ZERO; n];
        self.hidden.clear();
        self.more_area = Rect::ZERO;
        if area.is_empty() {
            self.menu = None;
            return;
        }
        let t = ctx.theme;
        self.marks = (t.glyphs.dot_on, t.glyphs.dot_off);
        let focused = ctx.interaction.focused_under(self.id, self.menu.is_some());
        // one focus stop with the geometry it draws in; the items' regions
        // registered after it take the pointer
        ctx.control(self.id, area, false);
        let more = match &self.more_button.glyph {
            Some(_) => self.more_button.clone(),
            None => self.more_button.with_glyph(t.glyphs.more_titles),
        };
        let more_w = more.width(t);
        let overflowing = self.overflow && n > 0 && self.width() > area.width;
        // with the button, items stop where it still fits after them
        let limit = if overflowing {
            area.right().saturating_sub(more_w + self.gap)
        } else {
            area.right()
        };
        let y = area.y;
        let mut x = area.x;
        // the separator after the last drawn item, cleared if nothing
        // drawn follows it (a group's divider never ends the row)
        let mut sep_at: Option<u16> = None;
        for i in 0..n {
            let it = &self.items[i];
            let w = it.width();
            if x + w > limit {
                // the order is kept: every item after a hidden one hides
                if overflowing {
                    self.hidden.extend(i..n);
                    if let Some(sx) = sep_at {
                        buf.set_string(sx, y, " ", ratatui::style::Style::new().bg(bg));
                    }
                }
                break;
            }
            sep_at = None;
            let r = Rect::new(x, y, w, 1);
            self.areas[i] = r;
            let iid = self.item_id(i);
            let mut s = ctx.child_state(self.id, iid, i == self.cursor);
            s.focused = focused && i == self.cursor;
            s.disabled = it.disabled;
            s.selected = it.on.unwrap_or(false);
            if it.disabled {
                s.hovered = false;
                s.pressed = false;
            }
            let kind = if it.on.is_some() {
                ButtonKind::Toggle
            } else {
                ButtonKind::Subtle
            };
            let look = t.interactive(Kind::Button(kind), s, t.plane_of(bg));
            let st = look.style;
            crate::ui::ctx::fill(buf, r, st);
            buf.set_string(x, y, look.gutter, look.gutter_style);
            let mut cx = x + 1;
            if let Some(on) = it.on {
                let dot = if on {
                    t.glyphs.dot_on
                } else {
                    t.glyphs.dot_off
                };
                let ink = if it.disabled || s.pressed {
                    st
                } else {
                    st.fg(if on { t.accent } else { t.text_muted })
                };
                buf.set_string(cx, y, dot, ink);
                cx += 2;
            }
            buf.set_string(cx, y, &it.label, st);
            cx += width(&it.label) as u16;
            if let Some(sc) = &it.shortcut {
                let ink = if look.on_fill || it.disabled {
                    st
                } else {
                    st.fg(t.text_muted)
                };
                buf.set_string(cx + 1, y, sc, ink.remove_modifier(Modifier::BOLD));
            }
            ctx.register_part(self.id, PartRef::item(Part::ROW, ItemKey::index(i)), r);
            if let Some(tip) = &it.tip {
                let tip = match &it.shortcut {
                    Some(sc) => Tip::new(tip).shortcut(sc),
                    None => Tip::new(tip),
                };
                crate::ui::tooltip::show(ctx, iid, r, &tip, None);
            }
            x += w + self.gap;
            if it.separator_after && x < limit {
                sep_at = Some(x);
                buf.set_string(
                    x,
                    y,
                    t.glyphs.divider,
                    ratatui::style::Style::new().fg(t.border_subtle).bg(bg),
                );
                x += 1 + self.gap;
            }
        }
        if self.hidden.is_empty() {
            // nothing hides: no button, and its menu has nothing to list
            self.menu = None;
            self.cursor = self.cursor.min(n.saturating_sub(1));
        } else {
            let mx = area.right().saturating_sub(more_w);
            let host = ratatui::style::Style::new().bg(bg);
            let plate = more.part_of(self.id, PartRef::of(Part::OVERFLOW)).render(
                mx,
                y,
                buf,
                ctx,
                host,
                Some(Tip::new(t.strings.tip_overflow)),
            );
            self.more_area = plate;
            // a hidden cursor item moves to the button that holds it
            if self.cursor > n || self.hidden.contains(&self.cursor) {
                self.cursor = n;
            }
            if focused && self.cursor == n {
                let mut s = ctx.state(self.more_id());
                s.focused = true;
                let under = buf[(mx, y)].bg;
                buf.set_string(mx, y, t.gutter_symbol(s), t.gutter(s, under, false));
            }
        }
        if let Some((menu, _)) = self.menu.as_mut() {
            menu.render(*buf.area(), buf, ctx);
        }
    }
}

impl RowSource for Toolbar {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn enabled(&self, i: usize) -> bool {
        !self.items[i].disabled
    }
    fn label(&self, i: usize) -> &str {
        &self.items[i].label
    }
}

impl Navigable for Toolbar {
    fn cursor(&self) -> Cursor {
        Cursor::new(self.cursor)
            .wrap(self.wrap)
            .axis(Axis::Horizontal)
            .skip_disabled(self.skip_disabled)
    }
}

/// A toolbar sample: hover and press land on its second item, focus is
/// the toolbar's (its cursor item), disabled disables that item, selected
/// turns its first toggle on, overflow draws it one cell short of its
/// width and narrow in the narrow slot (the rest behind `»`).
impl crate::ui::specimen::Specimen for Toolbar {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        match state {
            Rest | Hover | Pressed | Focused | FocusedHover | Disabled | Narrow => {
                self.items.len() > 1
            }
            Selected => self.items.iter().any(|i| i.on.is_some()),
            Overflow => self.items.len() > 1 && self.overflow,
            Busy | Error | Editing | Empty | Loading => false,
        }
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut t = self.clone();
        for it in &mut t.items {
            it.tip = None;
        }
        let v = state.visual();
        let target = 1.min(t.items.len() - 1);
        if v.hovered {
            ctx.interaction.hover = Some(t.item_id(target));
        }
        if v.pressed {
            ctx.interaction.pressed = Some(t.item_id(target));
        }
        if v.focused {
            t.cursor = target;
        }
        if v.disabled {
            t.items[target].disabled = true;
        }
        if v.selected
            && let Some(it) = t.items.iter_mut().find(|i| i.on.is_some())
        {
            it.on = Some(true);
        }
        let area = if state == crate::ui::specimen::SpecimenState::Overflow {
            // one cell short of every item: the last ones go behind `»`
            Rect {
                width: area.width.min(t.width().saturating_sub(1)),
                ..area
            }
        } else {
            area
        };
        t.render(area, buf, ctx, bg);
    }
}

/// The toolbar's pointer gesture: a completed click on an item or a row
/// of its open overflow menu runs it ([`Toolbar::on_click`]). The overflow
/// menu is the toolbar's too ([`Self::route`]).
impl crate::core::pointer::MouseTarget for Toolbar {
    type Action = ToolbarAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn route(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::response::Response<ToolbarAction>> {
        let menu = self.menu_open() && ev.on(self.menu_id());
        (ev.on(self.id) || menu).then(|| self.pointer(ev))
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<ToolbarAction> {
        if !ev.is(crate::core::intent::Phase::Click) {
            return crate::core::response::Response::ignored();
        }
        self.on_click(ev.id).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn k(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn bar() -> Toolbar {
        Toolbar::new(
            WidgetId::of("tb"),
            vec![
                ToolItem::new("Run").shortcut("F5"),
                ToolItem::new("Explain").disabled(true),
                ToolItem::new("Format").separator(),
                ToolItem::new("Wrap").toggle(false),
            ],
        )
    }

    fn render(t: &mut Toolbar, inter: Interaction) -> (Buffer, FocusRing, HitRegistry) {
        render_w(t, inter, 60, &Theme::junie())
    }

    /// Draw `t` in a `w`-cell row of a 60×12 screen (room for its menu),
    /// overlays flushed.
    fn render_w(
        t: &mut Toolbar,
        inter: Interaction,
        w: u16,
        theme: &Theme,
    ) -> (Buffer, FocusRing, HitRegistry) {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 12));
        {
            let mut ctx = RenderCtx::new(theme, inter, &mut hits, &mut ring);
            t.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, theme.canvas);
            ctx.flush_overlays(&mut buf);
        }
        (buf, ring, hits)
    }

    fn line(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    /// Run F5 · Explain │ Format(read-only) · Wrap(on): 38 cells.
    fn wide() -> Toolbar {
        Toolbar::new(
            WidgetId::of("tb"),
            vec![
                ToolItem::new("Run").shortcut("F5"),
                ToolItem::new("Explain").separator(),
                ToolItem::new("Format").shortcut("F6").reason("Read-only"),
                ToolItem::new("Wrap").toggle(true),
            ],
        )
    }

    #[test]
    fn the_toolbar_is_one_stop_and_arrows_skip_disabled_items() {
        let mut t = bar();
        let (_, ring, _) = render(&mut t, Interaction::default());
        assert_eq!(ring.reachable(), &[t.id], "one focus stop");
        assert_eq!(t.on_key(&k(KeyCode::Right)).0, Outcome::Changed);
        assert_eq!(t.cursor, 2, "Explain is disabled: skipped");
        t.on_key(&k(KeyCode::End));
        assert_eq!(t.cursor, 3);
        assert_eq!(t.on_key(&k(KeyCode::Right)).0, Outcome::Consumed, "no wrap");
        let mut w = bar().wrap(true);
        w.cursor = 3;
        w.on_key(&k(KeyCode::Right));
        assert_eq!(w.cursor, 0, "wraps on request");
        let mut all = bar().skip_disabled(false);
        all.on_key(&k(KeyCode::Right));
        assert_eq!(all.cursor, 1, "disabled items reachable on request");
        assert_eq!(all.on_key(&k(KeyCode::Enter)).1, None, "but never run");
    }

    #[test]
    fn enter_runs_the_cursor_item_and_flips_a_toggle() {
        let mut t = bar();
        assert_eq!(
            t.on_key(&k(KeyCode::Enter)).1,
            Some(ToolbarAction::Activated(0))
        );
        t.cursor = 3;
        assert_eq!(
            t.on_key(&k(KeyCode::Char(' '))).1,
            Some(ToolbarAction::Toggled(3, true))
        );
        assert_eq!(t.items[3].on, Some(true));
        let id = t.item_id(2);
        assert_eq!(t.on_click(id).1, Some(ToolbarAction::Activated(2)));
        assert_eq!(t.cursor, 2, "a click moves the cursor too");
        let id = t.item_id(1);
        assert_eq!(t.on_click(id).1, None, "a disabled item never runs");
    }

    #[test]
    fn items_that_do_not_fit_go_behind_an_overflow_button_at_the_end() {
        let mut t = wide();
        let focus = Interaction {
            focus: Some(t.id),
            ..Default::default()
        };
        let (buf, ring, hits) = render_w(&mut t, focus, 24, &Theme::junie());
        assert_eq!(ring.reachable(), &[t.id], "still one focus stop");
        assert_eq!(t.hidden(), &[2, 3], "Format and Wrap do not fit");
        assert!(t.areas[2].is_empty() && t.areas[3].is_empty());
        let row = line(&buf, 0);
        assert!(row.contains('»'), "the overflow button: {row:?}");
        assert!(
            !row.contains('│'),
            "no divider before hidden items: {row:?}"
        );
        let plate = hits
            .area_of(t.more_id())
            .expect("the button is a hit target");
        assert_eq!(plate.right(), 24, "at the right end");
        // ←/→ visit the drawn items, then the button as the last stop
        t.on_key(&k(KeyCode::End));
        assert!(t.on_more());
        let (buf, _, _) = render_w(&mut t, focus, 24, &Theme::junie());
        let row = line(&buf, 0);
        assert_eq!(row.matches('▎').count(), 1, "one focus mark: {row:?}");
        assert_eq!(buf[(plate.x, 0)].symbol(), "▎", "on the button");
        // Enter opens the menu of hidden items through the overlay layer
        assert_eq!(t.on_key(&k(KeyCode::Enter)), (Outcome::Changed, None));
        assert!(t.menu_open());
        let (buf, _, hits) = render_w(&mut t, focus, 24, &Theme::junie());
        let screen: Vec<String> = (0..12).map(|y| line(&buf, y)).collect();
        let menu = screen[1..].join("\n");
        assert!(
            menu.contains("Format") && menu.contains("Read-only"),
            "{menu}"
        );
        assert!(menu.contains("● Wrap"), "a toggle shows its state: {menu}");
        assert_eq!(
            screen.iter().map(|r| r.matches('▎').count()).sum::<usize>(),
            0,
            "the toolbar under its open menu shows no focus"
        );
        let wrap_row = hits.area_of(
            t.menu_id()
                .part(PartRef::item(Part::ROW, ItemKey::index(1))),
        );
        assert!(wrap_row.is_some(), "the menu rows are hit targets");
        // the read-only Format is skipped; Enter flips Wrap
        assert_eq!(
            t.on_key(&k(KeyCode::Enter)).1,
            Some(ToolbarAction::Toggled(3, false))
        );
        assert!(!t.menu_open());
        assert!(t.on_more(), "focus returns to the button");
        // a click on the button opens it, a second closes it
        t.on_click(t.more_id());
        assert!(t.menu_open());
        t.on_click(t.more_id());
        assert!(!t.menu_open());
        // ↓ opens it too; Esc closes
        t.on_key(&k(KeyCode::Down));
        assert!(t.menu_open());
        t.on_key(&k(KeyCode::Esc));
        assert!(!t.menu_open());
        // ← leaves the button for the last drawn item
        t.on_key(&k(KeyCode::Left));
        assert_eq!(t.cursor, 1);
    }

    #[test]
    fn the_overflow_button_is_configurable() {
        let mut t = wide().more_glyph("…");
        render_w(&mut t, Interaction::default(), 24, &Theme::junie());
        let (buf, _, _) = render_w(&mut t, Interaction::default(), 24, &Theme::junie());
        assert!(line(&buf, 0).contains('…'));
        let mut t = wide();
        let mut ascii = Theme::junie();
        ascii.glyphs = crate::theme::Glyphs::ASCII;
        let (buf, _, _) = render_w(&mut t, Interaction::default(), 24, &ascii);
        assert!(line(&buf, 0).trim_end().ends_with('>'), "the theme's glyph");
        // off: the items that do not fit are not drawn, and nothing hides
        let mut t = wide().overflow(false);
        let (buf, _, hits) = render_w(&mut t, Interaction::default(), 24, &Theme::junie());
        assert!(t.hidden().is_empty());
        assert!(hits.area_of(t.more_id()).is_none());
        assert!(!line(&buf, 0).contains('»'));
        // with room for everything there is no button
        let mut t = wide();
        let (_, _, hits) = render_w(&mut t, Interaction::default(), 60, &Theme::junie());
        assert!(hits.area_of(t.more_id()).is_none());
        assert!(t.hidden().is_empty());
    }

    #[test]
    fn the_focused_toolbar_draws_one_gutter_on_its_cursor_item() {
        let mut t = bar();
        t.cursor = 2;
        let focus = Interaction {
            focus: Some(t.id),
            ..Default::default()
        };
        let (buf, _, hits) = render(&mut t, focus);
        let row: String = (0..60).map(|x| buf[(x, 0)].symbol()).collect();
        assert_eq!(row.matches('▎').count(), 1);
        assert_eq!(buf[(t.areas[2].x, 0)].symbol(), "▎");
        assert!(row.contains("Run F5"), "the shortcut follows the label");
        assert!(row.contains('│'), "a separator after Format");
        let mid = ratatui::layout::Position::new(t.areas[0].x + 2, 0);
        assert_eq!(hits.hit(mid), Some(t.item_id(0)), "items are hit targets");
        let (buf, _, _) = render(&mut t, Interaction::default());
        let row: String = (0..60).map(|x| buf[(x, 0)].symbol()).collect();
        assert_eq!(row.matches('▎').count(), 0, "no focus, no gutter");
    }
}
