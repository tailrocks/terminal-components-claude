//! A sidebar's navigation list: sections, the current item, a keyboard
//! cursor and a collapsed (icon-only) mode.
//!
//! ```text
//!    Workspace            ····
//! ▎› T Tasks        3     ▎› T      (the current item, the focused cursor)
//!    R Runs                  R
//!    B Branches              B
//!
//!    Project              ····
//!    M Members               M
//!    $ Billing               $      (disabled: skipped, inert)
//! ```
//!
//! One focus stop (WAI-ARIA navigation list): ↑↓ move the cursor over the
//! enabled items, Enter or Space opens the item under it, and a click opens
//! an item at once (a navigation role activates on a single click, macOS
//! sidebars). The current item keeps its `›` while focus is elsewhere; the
//! `▎` cursor shows only while focused. Section headings and blank rows
//! between sections are drawn but never take the cursor. The list scrolls
//! when it overflows, with a scrollbar in its divider column.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::intent::Phase;
use crate::core::keymap::{Nav, NavKeys};
use crate::core::pointer::PointerEvent;
use crate::core::response::Response;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, VisualState, fill};
use crate::widgets::scrollbar;

/// One entry of a [`NavList`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavItem {
    pub label: String,
    /// A one-cell glyph or initial: shown before the label, and alone in
    /// the collapsed mode.
    pub icon: String,
    /// The section heading the item sits under. Consecutive items with the
    /// same section share one heading.
    pub section: String,
    /// A short count or flag at the row's right end (`3`, `new`).
    pub badge: Option<String>,
    /// Shown, skipped by the cursor and inert under the pointer.
    pub disabled: bool,
}

impl NavItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            icon: String::new(),
            section: String::new(),
            badge: None,
            disabled: false,
        }
    }

    /// See [`Self::icon`].
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_owned();
        self
    }

    /// See [`Self::section`].
    pub fn section(mut self, section: &str) -> Self {
        self.section = section.to_owned();
        self
    }

    /// See [`Self::badge`].
    pub fn badge(mut self, badge: &str) -> Self {
        self.badge = Some(badge.to_owned());
        self
    }

    /// See [`Self::disabled`].
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// What the list asks of its owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    /// Item `i` was opened (Enter, Space or a click): it is now current.
    Activated(usize),
}

/// One drawn row of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavRow {
    Section(usize),
    Blank,
    Item(usize),
}

/// A navigation list with sections; see the module docs.
#[derive(Debug, Clone)]
pub struct NavList {
    pub id: WidgetId,
    pub items: Vec<NavItem>,
    /// The open item (`›`).
    pub current: usize,
    /// The keyboard cursor (`▎` while focused).
    pub cursor: usize,
    /// Icon-only: initials and section dots, no labels or badges.
    pub collapsed: bool,
    /// Which keys move the cursor. Default [`NavKeys::DEFAULT`] (arrows,
    /// paging, Home/End); [`NavKeys::VIM`] adds `j k g G`.
    pub nav: NavKeys,
    /// The list's width open and collapsed, divider included
    /// ([`Self::width`]). Default 24 and 6.
    pub widths: (u16, u16),
    /// Scroll over the flattened rows (section headings and items).
    pub scroll: ScrollState,
    /// The current item's look while the list is unfocused; `None` (the
    /// default) takes [`crate::theme::Theme::inactive_selection`].
    pub inactive_selection: Option<crate::theme::InactiveSelection>,
    area: Rect,
    track: Rect,
}

impl NavList {
    pub fn new(id: WidgetId, items: Vec<NavItem>) -> Self {
        let cursor = items.iter().position(|i| !i.disabled).unwrap_or(0);
        Self {
            id,
            items,
            current: cursor,
            cursor,
            collapsed: false,
            nav: NavKeys::DEFAULT,
            widths: (24, 6),
            scroll: ScrollState::default(),
            inactive_selection: None,
            area: Rect::ZERO,
            track: Rect::ZERO,
        }
    }

    /// See [`Self::inactive_selection`].
    pub fn inactive_selection(mut self, look: crate::theme::InactiveSelection) -> Self {
        self.inactive_selection = Some(look);
        self
    }

    /// Open item `i` (the cursor follows it).
    pub fn current(mut self, i: usize) -> Self {
        self.current = i;
        self.cursor = i;
        self
    }

    /// See [`Self::collapsed`].
    pub fn collapsed(mut self, on: bool) -> Self {
        self.collapsed = on;
        self
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::widths`].
    pub fn widths(mut self, open: u16, collapsed: u16) -> Self {
        self.widths = (open, collapsed);
        self
    }

    /// Item `i`'s hit id.
    pub fn item_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::item_part(i))
    }

    fn item_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    /// The width the list wants: [`Self::widths`] for its mode.
    pub fn width(&self) -> u16 {
        if self.collapsed {
            self.widths.1
        } else {
            self.widths.0
        }
    }

    /// The rows in drawing order: a heading before every section, a blank
    /// row between sections.
    fn rows(&self) -> Vec<NavRow> {
        let mut rows = vec![];
        let mut section: Option<&str> = None;
        for (i, it) in self.items.iter().enumerate() {
            if section != Some(it.section.as_str()) {
                if !rows.is_empty() {
                    rows.push(NavRow::Blank);
                }
                // an item without a section starts no heading
                if !it.section.is_empty() {
                    rows.push(NavRow::Section(i));
                }
                section = Some(it.section.as_str());
            }
            rows.push(NavRow::Item(i));
        }
        rows
    }

    fn row_of(&self, item: usize) -> usize {
        self.rows()
            .iter()
            .position(|r| *r == NavRow::Item(item))
            .unwrap_or(0)
    }

    /// Keep the cursor's row in view with the smallest scroll.
    fn reveal_cursor(&mut self) {
        let row = self.row_of(self.cursor);
        self.scroll.set_content(self.rows().len());
        self.scroll.ensure_visible(row);
    }

    fn scrolled(moved: bool) -> Outcome {
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The nearest enabled item `steps` items from the cursor (negative:
    /// up), stopping at the ends.
    fn step(&self, steps: isize) -> usize {
        let mut c = self.cursor;
        let mut left = steps.unsigned_abs();
        let mut i = c;
        while left > 0 {
            let next = if steps < 0 {
                i.checked_sub(1)
            } else {
                (i + 1 < self.items.len()).then_some(i + 1)
            };
            let Some(n) = next else { break };
            i = n;
            if !self.items[i].disabled {
                c = i;
                left -= 1;
            }
        }
        c
    }

    /// A key while the list has focus.
    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<NavAction>) {
        if self.items.is_empty() {
            return (Outcome::Ignored, None);
        }
        if let Some(nav) = self.nav.action(key) {
            let page = (self.scroll.viewport_len.max(1) as isize - 1).max(1);
            self.cursor = match nav {
                Nav::Up => self.step(-1),
                Nav::Down => self.step(1),
                Nav::PageUp => self.step(-page),
                Nav::PageDown => self.step(page),
                Nav::Home => self
                    .items
                    .iter()
                    .position(|i| !i.disabled)
                    .unwrap_or(self.cursor),
                Nav::End => self
                    .items
                    .iter()
                    .rposition(|i| !i.disabled)
                    .unwrap_or(self.cursor),
                Nav::Left | Nav::Right => return (Outcome::Ignored, None),
            };
            self.reveal_cursor();
            return (Outcome::Changed, None);
        }
        match key.code {
            ratatui::crossterm::event::KeyCode::Enter
            | ratatui::crossterm::event::KeyCode::Char(' ') => {
                if self.items[self.cursor].disabled {
                    return (Outcome::Changed, None);
                }
                self.current = self.cursor;
                (Outcome::Changed, Some(NavAction::Activated(self.cursor)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    /// A click on item `i`: it opens at once.
    pub fn on_click(&mut self, i: usize) -> (Outcome, Option<NavAction>) {
        if i < self.items.len() && !self.items[i].disabled {
            self.cursor = i;
            self.current = i;
            self.reveal_cursor();
            return (Outcome::Changed, Some(NavAction::Activated(i)));
        }
        (Outcome::Changed, None)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.area = area;
        let rows = self.rows();
        self.scroll.set_content(rows.len());
        self.scroll.set_viewport(area.height as usize);
        let has_sb = self.scroll.overflows();
        // the last column is the divider, and the scrollbar on overflow
        let row_w = area.width.saturating_sub(1);
        let mut cursor_y = None;
        for (k, ri) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            match rows[ri] {
                NavRow::Blank => {}
                NavRow::Section(i) => {
                    if self.collapsed {
                        buf.set_string(area.x + 1, y, "····", t.faint().bg(bg));
                    } else {
                        buf.set_string(area.x + 3, y, &self.items[i].section, t.faint().bg(bg));
                    }
                }
                NavRow::Item(i) => {
                    let it = &self.items[i];
                    let row = Rect::new(area.x, y, row_w, 1);
                    let rid = self.item_id(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.disabled = it.disabled;
                    // the open item is the list's selection: it keeps its
                    // fill while the list is unfocused (InactiveSelection)
                    s.selected = i == self.current;
                    s.inactive_selection = self.inactive_selection;
                    if it.disabled {
                        s.hovered = false;
                    }
                    if i == self.cursor {
                        cursor_y = Some(y);
                    }
                    let current = i == self.current;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(
                        row.x,
                        y,
                        t.gutter_symbol(s),
                        t.gutter(s, st.bg.unwrap_or(bg), false),
                    );
                    if current {
                        let ink = t.chosen_in(VisualState { focused, ..s });
                        buf.set_string(
                            row.x + 1,
                            y,
                            "›",
                            st.fg(t.legible(ink, st.bg.unwrap_or(bg))),
                        );
                    }
                    let label_style = if it.disabled {
                        st
                    } else if current || s.focused || s.hovered {
                        st.fg(t.text_primary)
                    } else {
                        st.fg(t.text_secondary)
                    };
                    if self.collapsed {
                        buf.set_string(row.x + 3, y, &it.icon, label_style);
                    } else {
                        buf.set_string(
                            row.x + 3,
                            y,
                            &it.icon,
                            label_style.fg(if it.disabled {
                                t.disabled
                            } else {
                                t.text_muted
                            }),
                        );
                        buf.set_string(row.x + 5, y, &it.label, label_style);
                        if let Some(b) = &it.badge {
                            let bx = row
                                .right()
                                .saturating_sub(crate::ui::text::width(b) as u16 + 1);
                            let bs = if it.disabled { st } else { st.fg(t.accent) };
                            buf.set_string(bx, y, b, bs);
                        }
                    }
                    if !it.disabled {
                        ctx.register_part(self.id, Self::item_part(i), row);
                    }
                }
            }
        }
        ctx.scrollable(self.id, area);
        self.track = if has_sb {
            Rect::new(area.right() - 1, area.y, 1, area.height)
        } else {
            Rect::ZERO
        };
        if has_sb {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                Rect::new(area.x, area.y, row_w, area.height),
                &self.scroll,
                &cursor_y.into_iter().collect::<Vec<_>>(),
            );
        }
        scrollbar::render_divider(
            Rect::new(area.right().saturating_sub(1), area.y, 1, area.height),
            buf,
            ctx,
            self.id,
            &self.scroll,
            focused,
        );
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
    }
}

/// A navigation list drawn alone, in a forced state, for a States grid
/// ([`crate::ui::specimen`]): its first three items under one section
/// heading, the first one current and the cursor on the second. Hover and
/// press land on the cursor row (where the pointer would be), selected
/// opens the cursor row, disabled disables it under the pointer (no
/// reaction), overflow lengthens its label past the row and gives it a
/// badge. Collapsed, the sample is the icon-only list.
impl crate::ui::specimen::Specimen for NavList {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(
            state,
            Rest | Hover
                | Pressed
                | Focused
                | FocusedHover
                | Selected
                | Disabled
                | Overflow
                | Narrow
        ) && !(self.collapsed && state == Overflow)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        self.sample().rows().len() as u16
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let mut l = self.sample();
        fill(buf, area, ratatui::style::Style::new().bg(bg));
        if l.items.is_empty() {
            return;
        }
        let c = l.cursor;
        match state {
            Selected => l.current = c,
            Disabled => l.items[c].disabled = true,
            Overflow => {
                let it = &mut l.items[c];
                it.label = format!("{} and a label far longer than its row", it.label);
                it.badge = Some("12".into());
            }
            _ => {}
        }
        // the pointer rests on the cursor row, not on the list
        let v = state.visual();
        if v.hovered || state == Disabled {
            ctx.interaction.hover = Some(l.item_id(c));
            if v.pressed {
                ctx.interaction.pressed = Some(l.item_id(c));
            }
        }
        let w = l.width().min(area.width);
        l.render(Rect::new(area.x, area.y, w, area.height), buf, ctx, bg);
    }
}

impl NavList {
    /// The States grid's sample: the first three items in one section,
    /// all enabled, the first current and the cursor on the second.
    fn sample(&self) -> Self {
        let mut l = self.clone();
        l.items.truncate(3);
        let section = l
            .items
            .first()
            .map(|i| i.section.clone())
            .unwrap_or_default();
        for it in &mut l.items {
            it.section.clone_from(&section);
            it.disabled = false;
        }
        l.current = 0;
        l.cursor = 1.min(l.items.len().saturating_sub(1));
        l.scroll = ScrollState::default();
        l
    }
}

/// The list's pointer gestures: a click on an item opens it, the wheel
/// scrolls, the divider's scrollbar pages and drags. Focus is the
/// router's ([`crate::core::pointer::PointerConfig::focus_on`]).
impl crate::core::pointer::MouseTarget for NavList {
    type Action = NavAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn route(&mut self, ev: &PointerEvent) -> Option<Response<NavAction>> {
        (ev.on(self.id) || ev.id == scrollbar::id_for(self.id)).then(|| self.pointer(ev))
    }

    fn pointer(&mut self, ev: &PointerEvent) -> Response<NavAction> {
        let bar = scrollbar::id_for(self.id);
        if let Some((axis, delta)) = ev.wheel() {
            return match axis {
                // rows that fit pass the notch on to the container
                // (scroll chaining, as a `ScrollPane` does)
                Axis::Vertical if self.scroll.overflows() => {
                    Self::scrolled(self.scroll.scroll_by(delta as isize)).into()
                }
                _ => Response::ignored(),
            };
        }
        if ev.id == bar {
            return match ev.phase() {
                Some(Phase::Click) => {
                    Self::scrolled(scrollbar::press(self.track, ev.pos(), &mut self.scroll)).into()
                }
                Some(Phase::Drag) => {
                    Self::scrolled(scrollbar::drag(self.track, ev.pos(), &mut self.scroll)).into()
                }
                _ => Response::ignored(),
            };
        }
        match ev.index_of(Part::ROW) {
            Some(i) if ev.is(Phase::Click) && i < self.items.len() => {
                let (o, a) = self.on_click(i);
                let r: Response<NavAction> = o.into();
                match a {
                    Some(a) => r.with_action(a),
                    None => r,
                }
            }
            _ => Response::ignored(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyCode;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }
    }

    fn list() -> NavList {
        NavList::new(
            WidgetId::of("nav"),
            vec![
                NavItem::new("Tasks").icon("T").section("Work").badge("3"),
                NavItem::new("Runs").icon("R").section("Work"),
                NavItem::new("Billing")
                    .icon("$")
                    .section("Project")
                    .disabled(true),
                NavItem::new("Members").icon("M").section("Project"),
            ],
        )
    }

    fn draw(nav: &mut NavList, focus: bool) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let inter = Interaction {
            focus: focus.then_some(nav.id),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
        let area = Rect::new(0, 0, nav.width(), 8);
        let mut buf = Buffer::empty(area);
        nav.render(area, &mut buf, &mut ctx, theme.canvas);
        (buf, hits)
    }

    /// A list whose rows fit leaves the wheel to its container (scroll
    /// chaining); one that overflows takes it.
    #[test]
    fn the_wheel_passes_on_while_the_rows_fit() {
        use crate::core::pointer::MouseTarget;
        let mut nav = list();
        let (_, hits) = draw(&mut nav, false);
        let id = nav.id;
        let wheel = |hits: &HitRegistry| {
            PointerEvent::wheel_on(
                hits,
                id,
                Axis::Vertical,
                1,
                ratatui::layout::Position::new(2, 2),
                ratatui::crossterm::event::KeyModifiers::NONE,
            )
        };
        let ev = wheel(&hits);
        assert_eq!(
            nav.pointer(&ev).outcome(),
            Outcome::Ignored,
            "fits: passed on"
        );
        let mut long = NavList::new(
            WidgetId::of("nav"),
            (0..20)
                .map(|i| NavItem::new(&format!("Item {i}")))
                .collect(),
        );
        let (_, hits) = draw(&mut long, false);
        let ev = wheel(&hits);
        assert_eq!(
            long.pointer(&ev).outcome(),
            Outcome::Changed,
            "overflows: scrolls"
        );
    }

    /// Row `y` without the divider column.
    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width - 1)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn sections_head_their_items_and_the_cursor_skips_disabled_ones() {
        let mut nav = list();
        let (buf, _) = draw(&mut nav, true);
        assert_eq!(row(&buf, 0).trim(), "Work");
        assert!(row(&buf, 1).contains("› T Tasks"), "{}", row(&buf, 1));
        assert!(row(&buf, 1).trim_end().ends_with('3'), "the badge");
        assert_eq!(row(&buf, 3).trim(), "", "a blank row between sections");
        assert_eq!(row(&buf, 4).trim(), "Project");
        let down = key(KeyCode::Down);
        nav.on_key(&down);
        assert_eq!(nav.cursor, 1);
        nav.on_key(&down);
        assert_eq!(nav.cursor, 3, "Billing is skipped");
        assert_eq!(
            nav.on_key(&key(KeyCode::Enter)),
            (Outcome::Changed, Some(NavAction::Activated(3)))
        );
        assert_eq!(nav.current, 3);
        // the vim letters only with NavKeys::VIM
        assert_eq!(nav.on_key(&key(KeyCode::Char('k'))).0, Outcome::Ignored);
        let mut nav = nav.nav(NavKeys::VIM);
        nav.on_key(&key(KeyCode::Char('g')));
        assert_eq!(nav.cursor, 0);
    }

    #[test]
    fn a_click_opens_an_item_and_a_disabled_one_is_no_target() {
        let mut nav = list();
        let (_, hits) = draw(&mut nav, false);
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(6, 2)),
            Some(nav.item_id(1))
        );
        assert_eq!(hits.hit(ratatui::layout::Position::new(6, 5)), None);
        assert_eq!(
            nav.on_click(1),
            (Outcome::Changed, Some(NavAction::Activated(1)))
        );
        assert_eq!((nav.current, nav.cursor), (1, 1));
        assert_eq!(nav.on_click(2), (Outcome::Changed, None));
        assert_eq!(nav.current, 1);
    }

    #[test]
    fn collapsed_keeps_rows_and_marks_with_initials_only() {
        let mut nav = list().collapsed(true);
        assert_eq!(nav.width(), 6);
        let (buf, _) = draw(&mut nav, true);
        assert_eq!(row(&buf, 0).trim(), "····");
        assert_eq!(row(&buf, 1).trim(), "▎› T");
        assert!(!row(&buf, 1).contains("Tasks"));
    }

    #[test]
    fn a_states_grid_draws_the_list_in_each_state() {
        use crate::ui::specimen::{SpecimenState as S, StateGrid};
        let t = Theme::junie();
        let id = WidgetId::of("nav-grid");
        let states = [
            S::Rest,
            S::Hover,
            S::Focused,
            S::Selected,
            S::Disabled,
            S::Overflow,
        ];
        let mut g = StateGrid::new(id)
            .rows(&states)
            .label_width(10)
            .column_width(26)
            .column("Open", list().widths(24, 6));
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 40, 30);
        let mut buf = Buffer::empty(area);
        g.render(area, &mut buf, &mut ctx, t.canvas);
        assert!(ring.reachable().is_empty(), "samples take no focus");
        let text: Vec<String> = (0..30)
            .map(|y| (0..40).map(|x| buf[(x, y)].symbol()).collect())
            .collect();
        // each sample: the heading, then Tasks (current), Runs (cursor), Billing
        let at = |label: &str| -> usize {
            text.iter()
                .position(|r| r.starts_with(label))
                .unwrap_or_else(|| panic!("{label} row\n{}", text.join("\n")))
        };
        let runs = |label: &str| text[at(label) + 2].clone();
        assert!(text[at("rest") + 1].contains("› T Tasks"));
        assert_ne!(
            buf[(15, at("hover") as u16 + 2)].bg,
            buf[(15, at("rest") as u16 + 2)].bg,
            "hover lands on the cursor row"
        );
        assert!(runs("focused").contains('▏'), "focus is depicted");
        assert!(!buf.content().iter().any(|c| c.symbol() == "▎"));
        assert!(
            runs("selected").contains("› R Runs"),
            "the cursor row opens"
        );
        assert_eq!(
            buf[(16, at("disabled") as u16 + 2)].fg,
            t.disabled,
            "a disabled row in the disabled ink"
        );
        assert!(runs("overflow").contains("12"), "the badge stays");
    }
}
