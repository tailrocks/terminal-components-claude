use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::button::Button;
use crate::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("sidebars");

#[derive(Debug, Clone)]
pub struct NavItem {
    pub label: &'static str,
    pub icon: &'static str,
    pub section: &'static str,
    pub badge: Option<&'static str>,
    pub disabled: bool,
}

/// Standalone navigation list with sections, a current item, a keyboard
/// cursor and a collapsed (icon-only) mode.
#[derive(Debug, Clone)]
pub struct NavList {
    pub id: WidgetId,
    pub items: Vec<NavItem>,
    pub current: usize,
    pub cursor: usize,
    pub collapsed: bool,
}

impl NavList {
    pub fn item_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.items.len()).find(|&i| self.item_id(i) == id)
    }

    pub fn width(&self) -> u16 {
        if self.collapsed { 6 } else { 24 }
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let mut c = self.cursor;
                while c > 0 {
                    c -= 1;
                    if !self.items[c].disabled {
                        self.cursor = c;
                        break;
                    }
                }
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let mut c = self.cursor;
                while c + 1 < self.items.len() {
                    c += 1;
                    if !self.items[c].disabled {
                        self.cursor = c;
                        break;
                    }
                }
                Outcome::Changed
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if !self.items[self.cursor].disabled {
                    self.current = self.cursor;
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, i: usize) -> Outcome {
        if i < self.items.len() && !self.items[i].disabled {
            self.cursor = i;
            self.current = i;
        }
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let mut y = area.y;
        let mut section = "";
        for (i, it) in self.items.iter().enumerate() {
            if it.section != section {
                if y > area.y {
                    y += 1;
                }
                if y >= area.bottom() {
                    break;
                }
                if self.collapsed {
                    buf.set_string(area.x + 1, y, "····", t.faint().bg(bg));
                } else {
                    buf.set_string(area.x + 3, y, it.section, t.faint().bg(bg));
                }
                section = it.section;
                y += 1;
            }
            if y >= area.bottom() {
                break;
            }
            let row = Rect::new(area.x, y, area.width, 1);
            let rid = self.item_id(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.cursor;
            s.disabled = it.disabled;
            if it.disabled {
                s.hovered = false;
            }
            let current = i == self.current;
            let st = t.row(s, bg);
            fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            if current {
                buf.set_string(row.x + 1, y, "›", st.fg(t.accent));
            }
            let label_style = if it.disabled {
                st
            } else if current || s.focused || s.hovered {
                st.fg(t.text_primary)
            } else {
                st.fg(t.text_secondary)
            };
            if self.collapsed {
                buf.set_string(row.x + 3, y, it.icon, label_style);
            } else {
                buf.set_string(
                    row.x + 3,
                    y,
                    it.icon,
                    label_style.fg(if it.disabled {
                        t.disabled
                    } else {
                        t.text_muted
                    }),
                );
                buf.set_string(row.x + 5, y, it.label, label_style);
                if let Some(b) = it.badge {
                    let bx = row.right().saturating_sub(b.len() as u16 + 1);
                    let bs = if it.disabled { st } else { st.fg(t.accent) };
                    buf.set_string(bx, y, b, bs);
                }
            }
            if !it.disabled {
                ctx.clickable(rid, row);
            }
            y += 1;
        }
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
    }
}

pub struct SidebarsPage {
    nav: NavList,
    collapse: Button,
}

impl SidebarsPage {
    pub fn new() -> Self {
        let items = vec![
            NavItem {
                label: "Tasks",
                icon: "T",
                section: "Workspace",
                badge: Some("3"),
                disabled: false,
            },
            NavItem {
                label: "Runs",
                icon: "R",
                section: "Workspace",
                badge: None,
                disabled: false,
            },
            NavItem {
                label: "Branches",
                icon: "B",
                section: "Workspace",
                badge: None,
                disabled: false,
            },
            NavItem {
                label: "Members",
                icon: "M",
                section: "Project",
                badge: None,
                disabled: false,
            },
            NavItem {
                label: "Environment",
                icon: "E",
                section: "Project",
                badge: None,
                disabled: false,
            },
            NavItem {
                label: "Billing",
                icon: "$",
                section: "Project",
                badge: None,
                disabled: true,
            },
            NavItem {
                label: "Keyboard",
                icon: "K",
                section: "Preferences",
                badge: None,
                disabled: false,
            },
            NavItem {
                label: "Appearance",
                icon: "A",
                section: "Preferences",
                badge: None,
                disabled: false,
            },
        ];
        Self {
            nav: NavList {
                id: ID.sub("nav"),
                items,
                current: 0,
                cursor: 0,
                collapsed: false,
            },
            collapse: Button::secondary(ID.sub("collapse"), "Collapse"),
        }
    }
}

impl Page for SidebarsPage {
    fn title(&self) -> &'static str {
        "Sidebars"
    }
    fn blurb(&self) -> &'static str {
        "Sections, current item, focus cursor, hover, collapsed mode; text first, no icons"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let w = self.nav.width() + 4;
        let side = Rect::new(area.x, area.y, w, area.height.min(20));
        let panel = Panel::card(None);
        let bg = panel.bg(t);
        let inner = Rect::new(side.x, side.y + 1, side.width, side.height - 2);
        panel.render(side, buf, t);
        self.nav.render(
            Rect::new(
                inner.x,
                inner.y,
                inner.width,
                inner.height.saturating_sub(2),
            ),
            buf,
            ctx,
            bg,
        );
        self.collapse.label = if self.nav.collapsed {
            "Expand".into()
        } else {
            "Collapse".into()
        };
        if self.nav.collapsed {
            self.collapse.label = "›".into();
        }
        self.collapse.render(
            Rect::new(inner.x + 1, inner.bottom() - 1, inner.width, 1),
            buf,
            ctx,
            bg,
        );

        let content = Rect::new(
            side.right() + 2,
            area.y,
            area.width.saturating_sub(w + 2),
            area.height,
        );
        let current = self.nav.items[self.nav.current].label;
        let panel = Panel::card(Some(current));
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(content.x, content.y, content.width, content.height.min(20)),
            buf,
            t,
        );
        let lines = [
            "One focus stop. ↑ ↓ move the cursor, Enter opens.",
            "",
            "›  current item · persists when focus leaves",
            "▎  keyboard cursor · only while focused",
            "░  hover · follows the pointer",
            "",
            "Disabled items are skipped and ignore the pointer.",
            "Collapsed mode keeps rows and markers, initials only.",
        ];
        let mut y = inner.y;
        for (i, l) in lines.iter().enumerate() {
            let accent = (2..=4).contains(&i);
            for (j, wl) in crate::ui::text::wrap(l, inner.width as usize)
                .iter()
                .enumerate()
            {
                if y >= inner.bottom() {
                    break;
                }
                if accent && j == 0 {
                    let (glyph, rest) =
                        wl.split_at(wl.chars().next().map(|c| c.len_utf8()).unwrap_or(0));
                    buf.set_string(inner.x, y, glyph, t.accent_fg().bg(bg));
                    buf.set_string(inner.x + 1, y, rest, t.secondary().bg(bg));
                } else {
                    buf.set_string(inner.x, y, wl, t.secondary().bg(bg));
                }
                y += 1;
            }
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                if cx.focus.is(self.nav.id) {
                    return self.nav.on_key(key);
                }
                if cx.focus.is(self.collapse.id) {
                    let (o, act) = self.collapse.on_key(key);
                    if act {
                        self.nav.collapsed = !self.nav.collapsed;
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, .. } => {
                if let Some(i) = self.nav.locate(*id) {
                    cx.focus.focus(self.nav.id);
                    return self.nav.on_click(i);
                }
                if *id == self.collapse.id {
                    if self.collapse.on_click() {
                        self.nav.collapsed = !self.nav.collapsed;
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![("↑ ↓", "Move"), ("Enter", "Open")]
    }
}
