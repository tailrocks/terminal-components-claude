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
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::theme::{ButtonKind, Kind};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::ui::tooltip::Tip;

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

    /// The hit id of item `i`.
    pub fn item_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::ROW, ItemKey::index(i)))
    }

    /// Which item `id` is.
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..self.items.len())
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || self.locate(id).is_some()
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
        if self.items.is_empty() {
            return (Outcome::Ignored, None);
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
        let mut c = self.cursor();
        let moved = c.apply(nav, &*self, 1);
        if let Some(i) = moved.index() {
            self.cursor = i;
        }
        (moved.outcome(), None)
    }

    /// A click on `id`: the item under it runs.
    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<ToolbarAction>) {
        match self.locate(id) {
            Some(i) => match self.activate(i) {
                Some(a) => (Outcome::Changed, Some(a)),
                None => (Outcome::Consumed, None),
            },
            None => (Outcome::Ignored, None),
        }
    }

    /// One row: the items left to right; an item that does not fit is not
    /// drawn (an app gives the toolbar its [`Self::width`]).
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        self.areas = vec![Rect::ZERO; self.items.len()];
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        // one focus stop with the geometry it draws in; the items' regions
        // registered after it take the pointer
        ctx.control(self.id, area, false);
        self.cursor = self.cursor.min(self.items.len().saturating_sub(1));
        let y = area.y;
        let mut x = area.x;
        for i in 0..self.items.len() {
            let it = &self.items[i];
            let w = it.width();
            if x + w > area.right() {
                break;
            }
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
            if it.separator_after && x < area.right() {
                buf.set_string(
                    x,
                    y,
                    t.glyphs.divider,
                    ratatui::style::Style::new().fg(t.border_subtle).bg(bg),
                );
                x += 1 + self.gap;
            }
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
/// turns its first toggle on, narrow cuts the items that do not fit.
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
            // items that do not fit are not drawn yet (no overflow menu)
            Overflow | Busy | Error | Editing | Empty | Loading => false,
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
        t.render(area, buf, ctx, bg);
    }
}

/// The toolbar's pointer gesture: a completed click on an item runs it
/// ([`Toolbar::on_click`]).
impl crate::core::pointer::MouseTarget for Toolbar {
    type Action = ToolbarAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
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
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
        {
            let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
            t.render(Rect::new(0, 0, 60, 1), &mut buf, &mut ctx, theme.canvas);
        }
        (buf, ring, hits)
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
