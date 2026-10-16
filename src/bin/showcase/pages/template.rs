//! The widget-page template (plan §7, SCP2): a `Tabs` strip of three
//! sections at the top of the page, then the section's body.
//!
//! - **Playground**: the live widget with demo data (the page draws it).
//! - **Options**: the page's option table (`crate::options::page_options`),
//!   the same declarations the Inspector's Options tab and `--opt` read.
//!   [`Template::render_options`] is the extension point: a page with
//!   options draws an `OptionsView` over its table there.
//! - **States**: the page's [`StateGrid`], rows = states and columns =
//!   variants, drawn by the real widget at the current colour level, with a
//!   "Compare levels" toggle that draws it at all four.
//!
//! The template is showcase wiring only: the strip is the library `Tabs`,
//! the toggle a library `Button`, the grid the library `StateGrid`, the
//! scrolling the library `Stack`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Hint, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::ColorLevel;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::tabs::Tabs;

/// One section of a templated page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Playground,
    Options,
    States,
}

impl Section {
    pub const ALL: [Self; 3] = [Self::Playground, Self::Options, Self::States];

    pub fn label(self) -> &'static str {
        match self {
            Self::Playground => "Playground",
            Self::Options => "Options",
            Self::States => "States",
        }
    }

    /// What the section shows, beside the strip when there is room.
    pub fn caption(self) -> &'static str {
        match self {
            Self::Playground => "the live widget",
            Self::Options => "one control per option",
            Self::States => "every state, drawn by the widget",
        }
    }
}

/// Page width from which the strip carries its section caption and the
/// States section its focus legend.
pub const WIDE: u16 = 76;

/// The legend beside the Compare toggle: what the focus mark means.
pub const FOCUS_LEGEND: &str = "▏ depicts a focused sample; it is not where your focus is";

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

pub struct Template {
    pub tabs: Tabs,
    /// "Compare levels": the grid at all four colour levels.
    pub compare: Button,
    pub grid: StateGrid,
    /// The States body scrolls by whole children on a short page.
    states: super::layout::Sections,
    /// Rows the States grid drew whole in the last frame.
    shown: Vec<SpecimenState>,
}

impl Template {
    pub fn new(owner: WidgetId, grid: StateGrid) -> Self {
        let labels: Vec<&str> = Section::ALL.iter().map(|s| s.label()).collect();
        Self {
            tabs: Tabs::new(owner.sub("sections"), &labels),
            compare: Button::toggle(owner.sub("compare"), "Compare levels", false),
            grid,
            states: {
                // the grid's rows sit flush; the toggle keeps its own gap
                let mut s = super::layout::Sections::new(owner.sub("states"));
                s.stack = s.stack.gap(0);
                s
            },
            shown: Vec::new(),
        }
    }

    pub fn section(&self) -> Section {
        Section::ALL[self.tabs.active.min(Section::ALL.len() - 1)]
    }

    #[allow(dead_code)] // tests and the coverage gate drive it
    pub fn set_section(&mut self, section: Section) {
        let i = Section::ALL.iter().position(|s| *s == section).unwrap_or(0);
        self.tabs.set_active(i);
        self.tabs.cursor = i;
        self.states.state.reset();
    }

    /// Draw the strip; returns the body under it (one blank row between).
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) -> Rect {
        let t = ctx.theme;
        let strip = Rect::new(area.x, area.y, area.width, area.height.min(2));
        self.tabs.render(strip, buf, ctx, t.canvas);
        if area.width >= WIDE {
            let caption = self.section().caption();
            let w = junie_tui::ui::text::width(caption) as u16;
            let end = self
                .tabs
                .areas
                .iter()
                .map(|r| r.right())
                .max()
                .unwrap_or(area.x);
            let x = area.right().saturating_sub(w + 1);
            if x > end + 2 {
                buf.set_string(x, area.y, caption, t.faint().bg(t.canvas));
            }
        }
        let top = area.y.saturating_add(3).min(area.bottom());
        Rect::new(area.x, top, area.width, area.bottom() - top)
    }

    /// The Options body of a page with no option table yet: its page step
    /// adds the table (and draws an `OptionsView` here); the shell-wide
    /// options are in the Inspector meanwhile.
    pub fn render_options(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let e = EmptyState::new("No page options yet")
            .hint("Shell-wide options: i, then Options · or --opt shell.<key>=<value>");
        empty::render_ctx(area, buf, ctx, &e, t.canvas);
    }

    /// The States body: the toggle (and, when wide, the legend), then the
    /// grid at the current level, one stack child per row so a short page
    /// scrolls row by row, or at all four levels.
    pub fn render_states(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let comparing = self.compare.on == Some(true);
        let (w, h) = self.grid.size();
        let heights = self.grid.row_heights();
        // the toggle row and a blank row under it
        let mut children = vec![StackChild::fixed(2)];
        let together = self.grid.levels_height(area.width);
        if !comparing {
            // the first row carries the column header
            children.extend(
                heights
                    .iter()
                    .enumerate()
                    .map(|(i, &rh)| StackChild::fixed(rh + u16::from(i == 0))),
            );
        } else if together <= area.height.saturating_sub(2) {
            children.push(StackChild::fixed(together));
        } else {
            // one pane per child, so a short page scrolls pane by pane
            children.extend([StackChild::fixed(h + 1); 4]);
        }
        let focus = ctx.interaction.has_focus(self.compare.id).then_some(0);
        let l = self.states.layout(area, &children, focus, buf, ctx);
        match l.slot(0) {
            Some(r) => {
                let bw = self.compare.width().min(r.width);
                self.compare
                    .render(Rect::new(r.x, r.y, bw, 1), buf, ctx, t.canvas);
                let x = r.x + bw + 3;
                let room = r.right().saturating_sub(x) as usize;
                if r.width >= WIDE && room >= junie_tui::ui::text::width(FOCUS_LEGEND) {
                    buf.set_string(x, r.y, FOCUS_LEGEND, t.faint().bg(t.canvas));
                }
            }
            None => ctx.focus_stop(self.compare.id),
        }
        self.shown.clear();
        if !comparing {
            for i in 0..heights.len() {
                if let Some(r) = l.slot(1 + i) {
                    let r = Rect::new(r.x, r.y, r.width.min(w), r.height);
                    self.grid
                        .render_rows(i..i + 1, i == 0, r, buf, ctx, t.canvas);
                    self.shown.extend_from_slice(self.grid.rendered_rows());
                }
            }
        } else if children.len() == 2 {
            if let Some(r) = l.slot(1) {
                self.grid.render_levels(r, buf, ctx);
                self.shown.extend_from_slice(self.grid.rendered_rows());
            }
        } else {
            for (i, level) in LEVELS.into_iter().enumerate() {
                if let Some(r) = l.slot(1 + i) {
                    self.grid.render_level(level, r, buf, ctx);
                }
            }
        }
    }

    /// Rows the States grid drew whole in the last frame (at the current
    /// level, or at every level when comparing them side by side).
    #[allow(dead_code)] // the coverage gate reads it
    pub fn state_rows(&self) -> &[SpecimenState] {
        &self.shown
    }

    /// The template's own events: the strip, and in the States section the
    /// toggle and the body's scroll. `None` leaves the event to the page.
    pub fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Option<Outcome> {
        let before = self.tabs.active;
        let out = self.handle_inner(ev, cx)?;
        if self.tabs.active != before {
            self.states.state.reset();
        }
        Some(out)
    }

    fn handle_inner(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Option<Outcome> {
        let states = self.section() == Section::States;
        if states && let Some(o) = self.states.handle(ev) {
            return Some(o);
        }
        match ev {
            PageEvent::Key(key) => {
                let f = cx.focus.current()?;
                if f == self.tabs.id {
                    return Some(self.tabs.on_key(key).0);
                }
                if states && f == self.compare.id {
                    let (o, toggled) = self.compare.on_key(key);
                    if toggled {
                        self.states.state.reset();
                    }
                    return Some(o);
                }
                None
            }
            PageEvent::Click { id, .. } if self.tabs.owns(*id) => {
                cx.focus.focus(self.tabs.id);
                Some(self.tabs.on_click(*id).0)
            }
            PageEvent::Wheel { id, delta } if *id == self.tabs.id => {
                Some(self.tabs.on_wheel(0, *delta))
            }
            PageEvent::Click { id, .. } if states && *id == self.compare.id => {
                self.compare.on_click();
                self.states.state.reset();
                Some(Outcome::Changed)
            }
            _ => None,
        }
    }

    /// Hints for the template's own controls; `None` for the page's.
    pub fn hints(&self, focus: Option<WidgetId>) -> Option<Vec<Hint>> {
        let f = focus?;
        if f == self.tabs.id {
            return Some(vec![("←→", "Section"), ("1–3", "Jump")]);
        }
        (f == self.compare.id).then(|| vec![("Enter / Space", "Compare levels")])
    }
}
