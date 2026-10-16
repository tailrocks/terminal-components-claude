//! Toolbars (Inputs & forms, plan §7.5 SC28): the library `Toolbar`, a
//! row of commands that is one focus stop (WAI-ARIA APG toolbar).
//!
//! - **Playground**: a query toolbar (commands with shortcuts and tips, a
//!   Stop that is disabled until a run starts, separators, toggles) and a
//!   formatting toolbar of toggles, with a readout of the last
//!   `ToolbarAction`.
//! - **Options**: `toolbars.*`, one control per builder (nav keys, wrap,
//!   disabled items reachable, gap).
//! - **States**: a command toolbar and a toggle toolbar in every state.
//!
//! Wiring and demo data only: keys, pointer, focus and drawing are the
//! toolbar's.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::template::{Section, Template};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::core::keymap::NavKeys;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::toolbar::{ToolItem, Toolbar, ToolbarAction};

const ID: WidgetId = WidgetId::of("toolbars");

/// The query toolbar's commands; `running` enables Stop and disables Run.
fn query_items(running: bool) -> Vec<ToolItem> {
    vec![
        ToolItem::new("Run")
            .shortcut("F5")
            .tip("Run the statement under the cursor")
            .disabled(running),
        ToolItem::new("Run all")
            .shortcut("Shift+F5")
            .tip("Run every statement")
            .disabled(running),
        ToolItem::new("Stop")
            .tip("Stop the running query")
            .disabled(!running)
            .separator(),
        ToolItem::new("Format").tip("Format the script"),
        ToolItem::new("Explain")
            .tip("Show the query plan")
            .separator(),
        ToolItem::new("Wrap").toggle(false).tip("Wrap long lines"),
        ToolItem::new("Line numbers").toggle(true),
    ]
}

fn format_items() -> Vec<ToolItem> {
    vec![
        ToolItem::new("Bold").toggle(false),
        ToolItem::new("Italic").toggle(false),
        ToolItem::new("Code").toggle(false).separator(),
        ToolItem::new("Left").toggle(true),
        ToolItem::new("Center").toggle(false),
        ToolItem::new("Right").toggle(false),
    ]
}

pub struct ToolbarsPage {
    template: Template,
    pub query: Toolbar,
    pub format: Toolbar,
    pub running: bool,
    pub last: String,
    /// `toolbars.nav`, `.wrap`, `.skip_disabled`, `.gap`.
    opts: [usize; 4],
}

const OPT_KEYS: [&str; 4] = [
    "toolbars.nav",
    "toolbars.wrap",
    "toolbars.skip_disabled",
    "toolbars.gap",
];
const OPT_DEFAULTS: [usize; 4] = [0, 0, 1, 1];

impl ToolbarsPage {
    pub fn new() -> Self {
        let mut p = Self {
            template: Template::new(ID, states_grid()),
            query: Toolbar::new(ID.sub("query"), query_items(false)),
            format: Toolbar::new(ID.sub("format"), format_items()),
            running: false,
            last: "none yet · Activated(i) or Toggled(i, on)".into(),
            opts: OPT_DEFAULTS,
        };
        p.rebuild();
        p
    }

    /// Apply the options through the builders an app calls, keeping each
    /// toolbar's items and cursor.
    fn rebuild(&mut self) {
        let style = |t: &Toolbar, o: &[usize; 4]| {
            let cursor = t.cursor;
            let mut n = Toolbar::new(t.id, t.items.clone())
                .nav(if o[0] == 1 {
                    NavKeys::VIM
                } else {
                    NavKeys::DESKTOP
                })
                .wrap(o[1] == 1)
                .skip_disabled(o[2] == 1)
                .gap(o[3] as u16);
            n.cursor = cursor;
            n
        };
        self.query = style(&self.query, &self.opts);
        self.format = style(&self.format, &self.opts);
    }

    fn act(&mut self, a: ToolbarAction, query: bool, cx: &mut PageCtx) {
        let bar = if query { &self.query } else { &self.format };
        let label = match a {
            ToolbarAction::Activated(i) | ToolbarAction::Toggled(i, _) => {
                bar.items[i].label.clone()
            }
        };
        self.last = format!("{a:?} · {label}");
        if query && let ToolbarAction::Activated(i) = a {
            match i {
                0 | 1 => self.running = true,
                2 => self.running = false,
                _ => {}
            }
            // the run state enables Stop and disables Run, keeping toggles
            let mut items = query_items(self.running);
            for (n, o) in items.iter_mut().zip(&self.query.items) {
                if n.on.is_some() {
                    n.on = o.on;
                }
            }
            self.query.items = items;
        }
        cx.status(self.last.clone());
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[5, 1, 5, 1, 6, 0]);

        let panel = Panel::card(Some("Query toolbar")).meta("one Tab stop · ←→ · Enter");
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        self.query
            .render(Rect::new(inner.x, inner.y, inner.width, 1), buf, ctx, bg);
        let state = if self.running {
            "running · Stop is enabled, Run waits"
        } else {
            "idle · Stop is disabled until a run starts"
        };
        buf.set_string(inner.x, inner.y + 2, state, t.muted().bg(bg));

        let panel = Panel::card(Some("Formatting toolbar")).meta("toggles");
        let bg = panel.bg(t);
        let inner = panel.render(rows[2], buf, t);
        self.format
            .render(Rect::new(inner.x, inner.y, inner.width, 1), buf, ctx, bg);

        let panel = Panel::card(Some("Last"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[4], buf, t);
        let lines = [
            ("Action", self.last.clone()),
            (
                "Keys",
                "Tab enters once · ←→ move (disabled skipped) · Home End · Enter Space run".into(),
            ),
        ];
        for (i, (k, v)) in lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, k, t.muted().bg(bg));
            let w = inner.width.saturating_sub(9) as usize;
            buf.set_string(
                inner.x + 9,
                y,
                junie_tui::ui::text::truncate(v, w),
                t.primary().bg(bg),
            );
        }
    }
}

/// The States grid: a command toolbar and a toggle toolbar.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    StateGrid::new(id)
        .rows(&[
            Rest,
            Hover,
            Pressed,
            Focused,
            FocusedHover,
            Selected,
            Disabled,
            Narrow,
        ])
        .column_width(30)
        .narrow_width(16)
        .column(
            "Commands",
            Toolbar::new(
                id.child(0),
                vec![
                    ToolItem::new("Run").shortcut("F5"),
                    ToolItem::new("Stop").separator(),
                    ToolItem::new("Wrap").toggle(false),
                ],
            ),
        )
        .column(
            "Toggles",
            Toolbar::new(
                id.child(1),
                vec![
                    ToolItem::new("Bold").toggle(false),
                    ToolItem::new("Italic").toggle(false),
                    ToolItem::new("Code").toggle(false),
                ],
            ),
        )
}

impl Page for ToolbarsPage {
    fn title(&self) -> &'static str {
        "Toolbars"
    }

    fn blurb(&self) -> &'static str {
        "A row of commands that is one focus stop: shortcuts, tips, toggles, separators"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() != Section::Playground {
            return Outcome::Ignored;
        }
        match ev {
            PageEvent::Key(key) => {
                let f = cx.focus.current();
                for query in [true, false] {
                    let bar = if query {
                        &mut self.query
                    } else {
                        &mut self.format
                    };
                    if f == Some(bar.id) {
                        let (out, a) = bar.on_key(key);
                        if let Some(a) = a {
                            self.act(a, query, cx);
                        }
                        return out;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, .. } => {
                for query in [true, false] {
                    let bar = if query {
                        &mut self.query
                    } else {
                        &mut self.format
                    };
                    if bar.owns(*id) {
                        let bid = bar.id;
                        let (out, a) = bar.on_click(*id);
                        cx.focus.focus(bid);
                        if let Some(a) = a {
                            self.act(a, query, cx);
                        }
                        return out.or(Outcome::Changed);
                    }
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        if focus == Some(self.query.id) || focus == Some(self.format.id) {
            return vec![("←→", "Move"), ("Enter / Space", "Run")];
        }
        Vec::new()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        OPT_KEYS
            .iter()
            .position(|k| *k == key)
            .map(|i| self.opts[i])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        if let Some(i) = OPT_KEYS.iter().position(|k| *k == key) {
            self.opts[i] = value;
            self.rebuild();
        }
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}

pub fn page() -> Box<dyn Page> {
    Box::new(ToolbarsPage::new())
}
