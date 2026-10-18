use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Cx, CxExt, Hint, PageEvent};
use crate::options::{Group, Opt, OptionsView};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::ColorLevel;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::tabs::Tabs;

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

    pub fn caption(self) -> &'static str {
        match self {
            Self::Playground => "the live widget",
            Self::Options => "one control per option",
            Self::States => "every state, drawn by the widget",
        }
    }
}

pub const WIDE: u16 = 76;

pub const FOCUS_LEGEND: &str = "▏ depicts a focused sample; it is not where your focus is";

pub fn focus_legend(mark: &str) -> String {
    let rest = FOCUS_LEGEND.trim_start_matches('▏');
    format!("{mark}{rest}")
}

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

pub struct Template {
    pub tabs: Tabs,

    pub compare: Button,
    pub grid: StateGrid,

    states: super::layout::Sections,

    shown: Vec<SpecimenState>,

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
            states: {
                let mut s = super::layout::Sections::new(owner.sub("states"));
                s.stack = s.stack.gap(0);
                s
            },
            shown: Vec::new(),
            options: None,
            owner,
        }
    }

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

    #[allow(dead_code)]
    pub fn option_control(&self, key: &str) -> Option<WidgetId> {
        self.options.as_ref()?.control_id(key)
    }

    pub fn popup_open(&self) -> bool {
        self.section() == Section::Options && self.options.as_ref().is_some_and(|v| v.popup_open())
    }

    pub fn section(&self) -> Section {
        Section::ALL[self.tabs.active.min(Section::ALL.len() - 1)]
    }

    #[allow(dead_code)]
    pub fn set_section(&mut self, section: Section) {
        let i = Section::ALL.iter().position(|s| *s == section).unwrap_or(0);
        self.tabs.set_active(i);
        self.tabs.cursor = i;
        self.states.state.reset();
    }

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

    pub fn render_states(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let comparing = self.compare.on == Some(true);
        let (w, h) = self.grid.size();
        let heights = self.grid.row_heights();

        let mut children = vec![StackChild::fixed(2)];
        let together = self.grid.levels_height(area.width);
        if !comparing {
            children.extend(
                heights
                    .iter()
                    .enumerate()
                    .map(|(i, &rh)| StackChild::fixed(rh + u16::from(i == 0))),
            );
        } else if together <= area.height.saturating_sub(2) {
            children.push(StackChild::fixed(together));
        } else {
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
                let legend = focus_legend(self.grid.mark(t));
                if r.width >= WIDE && room >= junie_tui::ui::text::width(&legend) {
                    buf.set_string(x, r.y, &legend, t.faint().bg(t.canvas));
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

    #[allow(dead_code)]
    pub fn state_rows(&self) -> &[SpecimenState] {
        &self.shown
    }

    pub fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Option<Outcome> {
        let before = self.tabs.active;
        let out = self.handle_inner(ev, cx)?;
        if self.tabs.active != before {
            self.states.state.reset();
        }
        Some(out)
    }

    fn handle_inner(&mut self, ev: &PageEvent, cx: &mut Cx) -> Option<Outcome> {
        let states = self.section() == Section::States;
        if states && let Some(o) = self.states.handle(ev) {
            return Some(o);
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
                    let (o, toggled) = self.compare.on_key(key);
                    if toggled {
                        self.states.state.reset();
                    }
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
                    self.states.state.reset();
                    return Some(Outcome::Changed);
                }
                None
            }
            _ => None,
        }
    }

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
