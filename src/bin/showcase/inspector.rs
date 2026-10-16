//! The Inspector (`i`, plan §7 SC1): a "State | Options" pane.
//!
//! - **State**: the shell's diagnostics (mode, focus, hover, press, pointer,
//!   last key, focus ring, hit regions, tick, colour level, size, input
//!   counters) and a log of the last [`LOG_LEN`] inputs with the outcome
//!   each had.
//! - **Options**: the current page's option table, then the shell-wide
//!   options ([`crate::options::SHELL`]).
//!
//! At 100 columns and wider the pane docks on the right of the page; below
//! that it opens as an overlay drawn by the app (light dismiss, Esc
//! closes). Everything here is library widgets: `Tabs`, `props`, and the
//! option controls of [`OptionsView`].

use std::collections::VecDeque;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::app::{NAV_ENTRIES, PageId};
use crate::options::{Group, OptSet, OptionsView, SHELL, page_options};
use crate::pages::{Hint, PageCtx, PageEvent};
use junie_tui::core::event::{Input, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::tabs::Tabs;

pub const ID: WidgetId = WidgetId::of("app.inspector");

/// Inputs the State tab's log keeps.
pub const LOG_LEN: usize = 50;

/// One tab of the Inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    State,
    Options,
}

impl Tab {
    pub const ALL: [Self; 2] = [Self::State, Self::Options];
    pub fn label(self) -> &'static str {
        match self {
            Self::State => "State",
            Self::Options => "Options",
        }
    }
}

pub struct Inspector {
    pub tabs: Tabs,
    pub options: OptionsView,
    /// The page [`Self::options`] was built for.
    page: PageId,
    /// The last inputs, newest first: `(input, outcome)`.
    pub log: VecDeque<(String, String)>,
    /// Inputs handled, ticks not counted.
    pub inputs: u64,
    pub ticks: u64,
}

fn view(page: PageId) -> OptionsView {
    let heading: &'static str = NAV_ENTRIES[page.index()].label;
    OptionsView::new(
        ID.sub("options"),
        &[
            Group {
                heading,
                opts: page_options(page),
                empty: "No page options yet",
            },
            Group {
                heading: "Shell",
                opts: SHELL,
                empty: "",
            },
        ],
    )
}

impl Inspector {
    pub fn new(page: PageId) -> Self {
        let labels: Vec<&str> = Tab::ALL.iter().map(|t| t.label()).collect();
        Self {
            tabs: Tabs::new(ID.sub("tabs"), &labels),
            options: view(page),
            page,
            log: VecDeque::new(),
            inputs: 0,
            ticks: 0,
        }
    }

    pub fn tab(&self) -> Tab {
        Tab::ALL[self.tabs.active.min(Tab::ALL.len() - 1)]
    }

    #[allow(dead_code)] // tests and the coverage gate open a tab directly
    pub fn set_tab(&mut self, tab: Tab) {
        let i = Tab::ALL.iter().position(|t| *t == tab).unwrap_or(0);
        self.tabs.set_active(i);
        self.tabs.cursor = i;
    }

    /// Show `page`'s option table (rebuilt when the page changed).
    pub fn for_page(&mut self, page: PageId) {
        if self.page != page {
            self.page = page;
            self.options = view(page);
        }
    }

    /// Log an input and what it did.
    pub fn record(&mut self, input: &Input, out: Outcome) {
        let what = match input {
            Input::Tick => {
                self.ticks += 1;
                return;
            }
            Input::Key(k) => format!("Key {}", crate::app::describe_key(k)),
            Input::Mouse(m) => {
                let mods = crate::app::describe_mods(m.mods);
                format!("{mods}{:?} {}·{}", m.kind, m.pos.x, m.pos.y)
            }
            Input::Resize(w, h) => format!("Resize {w}×{h}"),
            Input::Paste(text) => format!("Paste {} chars", text.chars().count()),
        };
        self.inputs += 1;
        self.log.push_front((what, format!("{out:?}")));
        self.log.truncate(LOG_LEN);
    }

    /// Whether `id` is one of the pane's targets.
    pub fn owns(&self, id: WidgetId) -> bool {
        self.tabs.owns(id) || (self.tab() == Tab::Options && self.options.owns(id))
    }

    /// Whether focus is on one of the pane's controls.
    pub fn has_focus(&self, focus: Option<WidgetId>) -> bool {
        focus.is_some_and(|f| f == self.tabs.id || self.options.has_control(f))
    }

    /// Draw the pane into `area` (inside its card): the tab strip, then the
    /// tab. `state` is the State tab's diagnostics.
    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        state: &[Prop],
    ) {
        let strip = Rect::new(area.x, area.y, area.width, area.height.min(2));
        self.tabs.render(strip, buf, ctx, bg);
        let top = area.y.saturating_add(3).min(area.bottom());
        let body = Rect::new(area.x, top, area.width, area.bottom() - top);
        match self.tab() {
            Tab::State => self.render_state(body, buf, ctx, bg, state),
            Tab::Options => self.options.render(body, buf, ctx, bg),
        }
    }

    fn render_state(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        state: &[Prop],
    ) {
        let t = ctx.theme;
        let used = props::render(area, buf, t, state, bg);
        let y = area.y + used + 1;
        if y + 1 >= area.bottom() {
            return;
        }
        buf.set_string(area.x, y, "Events", t.title().bg(bg));
        let log: Vec<Prop> = self
            .log
            .iter()
            .map(|(input, out)| Prop::new(out, input.clone()))
            .collect();
        let rest = Rect::new(area.x, y + 1, area.width, area.bottom() - y - 1);
        if log.is_empty() {
            buf.set_string(rest.x, rest.y, "No input yet", t.faint().bg(bg));
        } else {
            props::render(rest, buf, t, &log, bg);
        }
    }

    pub fn on_blur(&mut self, from: Option<WidgetId>) -> Outcome {
        self.options.on_blur(from)
    }

    /// Route `ev` to the pane; `None` when it is not the pane's.
    pub fn handle(
        &mut self,
        ev: &PageEvent,
        cx: &mut PageCtx,
    ) -> Option<(Outcome, Option<OptSet>)> {
        match ev {
            PageEvent::Key(key) if cx.focus.current() == Some(self.tabs.id) => {
                return Some((self.tabs.on_key(key).0, None));
            }
            PageEvent::Click { id, .. } if self.tabs.owns(*id) => {
                cx.focus.focus(self.tabs.id);
                return Some((self.tabs.on_click(*id).0, None));
            }
            _ => {}
        }
        if self.tab() == Tab::Options {
            return self.options.handle(ev, cx);
        }
        None
    }

    /// Footer hints while focus is in the pane.
    pub fn hints(&self, focus: Option<WidgetId>) -> Option<Vec<Hint>> {
        if focus == Some(self.tabs.id) {
            return Some(vec![("←→", "Switch tab")]);
        }
        self.options.hints(focus)
    }
}
