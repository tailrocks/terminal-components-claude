//! Sidebars: the library `NavList` (sections, the current item, the
//! focus cursor, hover, a collapsed icon mode) in a sidebar whose frame
//! carries the `SidebarRail` collapse button; hidden, the sidebar leaves
//! the rail, and a click on it (or Ctrl+B) brings it back.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::NavKeys;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::nav_list::{NavAction, NavItem, NavList};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::rail::{RailAction, SidebarRail};

const ID: WidgetId = WidgetId::of("sidebars");

pub struct SidebarsPage {
    pub nav: NavList,
    collapse: Button,
    /// The sidebar's collapse button and the rail it leaves.
    rail: SidebarRail,
    /// The sidebar is hidden behind its rail.
    pub hidden: bool,
    /// What the list and the rail reported last.
    last: String,
}

impl SidebarsPage {
    pub fn new() -> Self {
        let items = vec![
            NavItem::new("Tasks")
                .icon("T")
                .section("Workspace")
                .badge("3"),
            NavItem::new("Runs").icon("R").section("Workspace"),
            NavItem::new("Branches").icon("B").section("Workspace"),
            NavItem::new("Members").icon("M").section("Project"),
            NavItem::new("Environment").icon("E").section("Project"),
            NavItem::new("Billing")
                .icon("$")
                .section("Project")
                .disabled(true),
            NavItem::new("Keyboard").icon("K").section("Preferences"),
            NavItem::new("Appearance").icon("A").section("Preferences"),
        ];
        Self {
            // the page's list also takes the vim letters (NavKeys::VIM)
            nav: NavList::new(ID.sub("nav"), items)
                .current(0)
                .collapsed(false)
                .widths(24, 6)
                .nav(NavKeys::VIM),
            collapse: Button::secondary(ID.sub("collapse"), "Collapse"),
            rail: SidebarRail::new(ID.sub("rail"), "Navigation"),
            hidden: false,
            last: "none yet".into(),
        }
    }

    fn act(&mut self, a: Option<NavAction>) {
        if let Some(NavAction::Activated(i)) = a {
            self.last = format!("NavAction::Activated({i}) · {}", self.nav.items[i].label);
        }
    }

    /// Hide the sidebar behind its rail, or bring it back (the rail's
    /// buttons or Ctrl+B).
    fn set_hidden(&mut self, on: bool, cx: &mut Cx) {
        self.hidden = on;
        self.last = if on {
            "RailAction::Collapse"
        } else {
            "RailAction::Expand"
        }
        .into();
        if !on {
            cx.focus.focus(self.nav.id);
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
        let content_x = if self.hidden {
            // the hidden sidebar leaves its rail
            let col = Rect::new(area.x, area.y, SidebarRail::WIDTH, area.height.min(20));
            self.rail.render_rail(col, buf, ctx, t.canvas);
            area.x + self.rail.width() + 1
        } else {
            let w = self.nav.width() + 4;
            let side = Rect::new(area.x, area.y, w, area.height.min(20));
            let panel = Panel::card(None);
            let bg = panel.bg(t);
            let inner = Rect::new(side.x, side.y + 1, side.width, side.height - 2);
            panel.render(side, buf, t);
            self.rail.render_collapse(side, buf, ctx, bg);
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
                "›".into()
            } else {
                "Collapse".into()
            };
            self.collapse.render(
                Rect::new(inner.x + 1, inner.bottom() - 1, inner.width, 1),
                buf,
                ctx,
                bg,
            );
            side.right() + 2
        };

        let content = Rect::new(
            content_x,
            area.y,
            area.right().saturating_sub(content_x),
            area.height,
        );
        let current = self.nav.items[self.nav.current].label.clone();
        let panel = Panel::card(Some(&current));
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(content.x, content.y, content.width, content.height.min(20)),
            buf,
            t,
        );
        let last = format!("Last: {}", self.last);
        let lines = [
            "One focus stop. ↑↓ (or j k) move the cursor, Enter opens.",
            "",
            "›  current item · persists when focus leaves",
            "▎  keyboard cursor · only while focused",
            "░  hover · follows the pointer",
            "",
            "Disabled items are skipped and ignore the pointer.",
            "Collapse keeps rows and markers, initials only.",
            "‹ on the frame or Ctrl+B hides the sidebar behind its rail.",
            "",
            last.as_str(),
        ];
        let mut y = inner.y;
        let mut clipped = false;
        for (i, l) in lines.iter().enumerate() {
            let accent = (2..=4).contains(&i);
            for (j, wl) in junie_tui::ui::text::wrap(l, inner.width as usize)
                .iter()
                .enumerate()
            {
                if y >= inner.bottom() {
                    clipped |= !wl.is_empty();
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
        if clipped {
            panel.mark_clipped(inner, buf, t);
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        match ev {
            // Ctrl+B is the sidebar's shortcut wherever focus is on the page
            PageEvent::Key(key) if key.ctrl_char('b') => {
                self.set_hidden(!self.hidden, cx);
                Outcome::Changed
            }
            PageEvent::Key(key) => {
                if cx.focus.is(self.nav.id) {
                    let (o, a) = self.nav.on_key(key);
                    self.act(a);
                    return o;
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
            PageEvent::Pointer(ev) => {
                if let Some(r) = self.rail.route(ev) {
                    let (o, a) = r.into_parts();
                    if let Some(a) = a {
                        self.set_hidden(a == RailAction::Collapse, cx);
                        return Outcome::Changed;
                    }
                    return o;
                }
                if ev.on(self.nav.id) && ev.is(Phase::Click) && ev.index_of(Part::ROW).is_some() {
                    cx.focus.focus(self.nav.id);
                }
                if let Some(r) = self.nav.route(ev) {
                    let (o, a) = r.into_parts();
                    self.act(a);
                    return o;
                }
                if let Some(r) = self.collapse.route(ev) {
                    if !ev.is(Phase::Click) {
                        return r.outcome();
                    }
                    if r.into_action().is_some() {
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
        vec![("↑↓", "Move"), ("Enter", "Open"), ("Ctrl+B", "Sidebar")]
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
