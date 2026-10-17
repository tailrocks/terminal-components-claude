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
//! Every section's body sits in a library
//! [`ScrollView`](junie_tui::ui::scroll_view::ScrollView): it is laid out
//! at its natural size (the page's [`super::Page::measure`] for the
//! Playground, the option rows, the grid) and, when that is larger than the
//! page, it scrolls on the axes that overflow, with their scrollbars, by
//! wheel, Shift+wheel, bar drag and keyboard focus, instead of being cut
//! off. The page shell brackets the page's drawing with
//! [`Template::begin`] and [`Template::end`] (`pages::adapter`), so no page
//! has scrolling code of its own.
//!
//! The template is showcase wiring only: the strip is the library `Tabs`,
//! the toggle a library `Button`, the grid the library `StateGrid`, the
//! scrolling the library `ScrollView`.

use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};

use super::{Cx, CxExt, Hint, PageEvent};
use crate::options::{Group, Opt, OptionsView};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::measure::Measure;
use junie_tui::ui::scroll_view::{ScrollFrame, ScrollView};
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
/// It starts with the grid's own mark ([`StateGrid::mark`]): `▏` by
/// default, `:` under ASCII glyphs.
pub const FOCUS_LEGEND: &str = "▏ depicts a focused sample; it is not where your focus is";

/// [`FOCUS_LEGEND`] after `mark`, the glyph the grid really draws.
pub fn focus_legend(mark: &str) -> String {
    let rest = FOCUS_LEGEND.trim_start_matches('▏');
    format!("{mark}{rest}")
}

pub struct Template {
    pub tabs: Tabs,
    /// "Compare levels": the grid at all four colour levels.
    pub compare: Button,
    pub grid: StateGrid,
    /// The section body: laid out at its natural size, scrolled on the
    /// axes where that is larger than the page.
    pub view: ScrollView,
    /// Where the body draws this frame, between [`Self::begin`] and
    /// [`Self::end`]: the full-size content.
    content: Option<Rect>,
    /// Rows the States grid drew whole in the last frame.
    shown: Vec<SpecimenState>,
    /// The page's option table (`crate::options::page_options`) as live
    /// controls, built on the first [`Self::sync_options`].
    options: Option<OptionsView>,
    owner: WidgetId,
}

impl Template {
    pub fn new(owner: WidgetId, grid: StateGrid) -> Self {
        let labels: Vec<&str> = Section::ALL.iter().map(|s| s.label()).collect();
        Self {
            tabs: Tabs::new(owner.sub("sections"), &labels),
            compare: Button::toggle(owner.sub("compare"), "Compare levels", false),
            grid,
            view: ScrollView::new(owner.sub("body")),
            content: None,
            shown: Vec::new(),
            options: None,
            owner,
        }
    }

    /// Load the page's option values before a frame: `opts` is the page's
    /// table, `value` its current value index per option. The controls
    /// are the same declarations the Inspector's Options tab and `--opt`
    /// read; a change goes back to the shell as [`super::Go::SetOption`], so
    /// all three take one path (`App::apply_opt`).
    pub fn sync_options(&mut self, opts: &'static [Opt], value: impl Fn(&Opt) -> usize) {
        if opts.is_empty() {
            return;
        }
        let owner = self.owner;
        let view = self.options.get_or_insert_with(|| {
            OptionsView::new(
                owner.sub("options"),
                &[Group {
                    heading: "Page options",
                    opts,
                    empty: "",
                }],
            )
        });
        view.sync(value);
    }

    /// The control of option `key` in the Options section.
    #[allow(dead_code)] // tests reach controls by key
    pub fn option_control(&self, key: &str) -> Option<WidgetId> {
        self.options.as_ref()?.control_id(key)
    }

    /// Whether an option's select has its list open in the Options
    /// section: a layer over the page.
    pub fn popup_open(&self) -> bool {
        self.section() == Section::Options && self.options.as_ref().is_some_and(|v| v.popup_open())
    }

    pub fn section(&self) -> Section {
        Section::ALL[self.tabs.active.min(Section::ALL.len() - 1)]
    }

    #[allow(dead_code)] // tests and the coverage gate drive it
    pub fn set_section(&mut self, section: Section) {
        let i = Section::ALL.iter().position(|s| *s == section).unwrap_or(0);
        self.tabs.set_active(i);
        self.tabs.cursor = i;
        self.view.reset();
    }

    /// The section body of a page drawn in `area`: below the strip and a
    /// blank row.
    pub fn body_of(area: Rect) -> Rect {
        let top = area.y.saturating_add(3).min(area.bottom());
        Rect::new(area.x, top, area.width, area.bottom() - top)
    }

    /// The natural size of the shown section's body offered `avail`:
    /// `playground` for the Playground (the page's measure), the option
    /// rows, or the toggle and the grid (all four levels when comparing).
    pub fn measure(&self, avail: Size, playground: Size, ctx: &RenderCtx) -> Size {
        match self.section() {
            Section::Playground => playground,
            Section::Options => match &self.options {
                Some(view) => view.measure(avail),
                None => avail,
            },
            Section::States => {
                let grid = self.grid.measure(avail, ctx);
                let rows = if self.compare.on == Some(true) {
                    self.grid.levels_height(avail.width)
                } else {
                    grid.height
                };
                // the toggle and its legend, a blank row, the grid
                let legend = focus_legend(self.grid.mark(ctx.theme));
                let toggle = (self.compare.width() + 3)
                    .saturating_add(junie_tui::ui::text::width(&legend) as u16);
                Size::new(grid.width.max(toggle), rows.saturating_add(2))
            }
        }
    }

    /// Start a frame of a page drawn in `area` whose Playground measures
    /// `playground`: the section body gets its natural size in the
    /// returned frame. The page draws into [`ScrollFrame::buffer`] (its
    /// strip too: it lands on screen unchanged), then [`Self::end`].
    pub fn begin(
        &mut self,
        area: Rect,
        playground: Size,
        buf: &Buffer,
        ctx: &mut RenderCtx,
    ) -> ScrollFrame {
        let body = Self::body_of(area);
        let size = self.measure(body.as_size(), playground, ctx);
        let frame = self.view.begin(body, size, buf, ctx);
        self.content = Some(frame.area());
        frame
    }

    /// End a frame [`Self::begin`] started: the body's window, its bars.
    pub fn end(&mut self, frame: ScrollFrame, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.content = None;
        self.view.end(frame, buf, ctx);
    }

    /// A key the page left unused: the body's scroll keys
    /// ([`ScrollView::on_key`]).
    pub fn on_unused_key(&mut self, key: &junie_tui::core::event::Key) -> Option<Outcome> {
        self.view.on_key(key)
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
        // inside a frame the body is the full-size content
        self.content.unwrap_or_else(|| Self::body_of(area))
    }

    /// The Options body of a page with no option table yet: its page step
    /// adds the table (and draws an `OptionsView` here); the shell-wide
    /// options are in the Inspector meanwhile.
    pub fn render_options(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        if let Some(view) = &mut self.options {
            view.render(area, buf, ctx, t.canvas);
            return;
        }
        let e = EmptyState::new("No page options yet")
            .hint("Shell-wide options: i, then Options · or --opt shell.<key>=<value>");
        empty::render_ctx(area, buf, ctx, &e, t.canvas);
    }

    /// The States body: the toggle (and, when wide, the legend), a blank
    /// row, then the grid at the current level, or at all four. Inside the
    /// template's frame the body has the grid's natural size; the view
    /// scrolls it.
    pub fn render_states(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let comparing = self.compare.on == Some(true);
        let (w, _) = self.grid.size();
        self.shown.clear();
        if area.is_empty() {
            ctx.focus_stop(self.compare.id);
            return;
        }
        let bw = self.compare.width().min(area.width);
        self.compare
            .render(Rect::new(area.x, area.y, bw, 1), buf, ctx, t.canvas);
        let x = area.x + bw + 3;
        let room = area.right().saturating_sub(x) as usize;
        let legend = focus_legend(self.grid.mark(t));
        if area.width >= WIDE && room >= junie_tui::ui::text::width(&legend) {
            buf.set_string(x, area.y, &legend, t.faint().bg(t.canvas));
        }
        let top = area.y.saturating_add(2).min(area.bottom());
        let grid = Rect::new(area.x, top, area.width, area.bottom() - top);
        if grid.is_empty() {
            return;
        }
        if comparing {
            self.grid.render_levels(grid, buf, ctx);
        } else {
            let r = Rect::new(grid.x, grid.y, grid.width.min(w), grid.height);
            self.grid.render(r, buf, ctx, t.canvas);
        }
        self.shown.extend_from_slice(self.grid.rendered_rows());
    }

    /// Rows the States grid drew whole in the last frame (at the current
    /// level, or at every level when comparing them side by side).
    #[allow(dead_code)] // the coverage gate reads it
    pub fn state_rows(&self) -> &[SpecimenState] {
        &self.shown
    }

    /// The template's own events: the strip, and in the States section the
    /// toggle and the body's scroll. `None` leaves the event to the page.
    pub fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Option<Outcome> {
        let before = self.tabs.active;
        let out = self.handle_inner(ev, cx)?;
        if self.tabs.active != before {
            self.view.reset();
        }
        Some(out)
    }

    fn handle_inner(&mut self, ev: &PageEvent, cx: &mut Cx) -> Option<Outcome> {
        let states = self.section() == Section::States;
        // the body's own wheel and bars
        if let PageEvent::Pointer(p) = ev
            && let Some(r) = self.view.pointer(p)
        {
            return Some(r.outcome());
        }
        if let Some(view) = &mut self.options {
            if let PageEvent::Blur(ch) = ev
                && view.has_control(ch.from)
            {
                return Some(view.on_blur(Some(ch.from)));
            }
            if self.tabs.active == 1
                && let Some((o, set)) = view.handle(ev, cx)
            {
                if let Some(set) = set {
                    cx.set_option(set);
                }
                return Some(o);
            }
        }
        match ev {
            PageEvent::Key(key) => {
                let f = cx.focus.current()?;
                if f == self.tabs.id {
                    return Some(self.tabs.on_key(key).0);
                }
                if states && f == self.compare.id {
                    let (o, _) = self.compare.on_key(key);
                    return Some(o);
                }
                None
            }
            PageEvent::Pointer(p) => {
                use junie_tui::core::intent::Phase;
                use junie_tui::core::pointer::MouseTarget;
                if let Some(r) = self.tabs.route(p) {
                    if p.is(Phase::Click) {
                        cx.focus.focus(self.tabs.id);
                    }
                    return Some(r.outcome());
                }
                if states && p.on(self.compare.id) && p.is(Phase::Click) {
                    self.compare.on_click();
                    return Some(Outcome::Changed);
                }
                None
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
        if let Some(h) = self.options.as_ref().and_then(|v| v.hints(focus)) {
            return Some(h);
        }
        (f == self.compare.id).then(|| vec![("Enter / Space", "Compare levels")])
    }
}
