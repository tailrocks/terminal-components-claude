//! Toolbars (Inputs & forms, plan §7.5 SC28): the library `Toolbar`, a
//! row of commands that is one focus stop (WAI-ARIA APG toolbar).
//!
//! - **Playground**: a query toolbar (commands with shortcuts and tips, a
//!   Stop that is disabled until a run starts, separators, toggles), the
//!   same commands in a narrow toolbar whose hidden items sit behind the
//!   `»` overflow menu, and a formatting toolbar of toggles, with a
//!   readout of the last `ToolbarAction`.
//! - **Options**: `toolbars.*`, one control per builder (nav keys, wrap,
//!   disabled items reachable, gap, overflow button).
//! - **States**: a command toolbar and a toggle toolbar in every state.
//!
//! Wiring and demo data only: keys, pointer, focus and drawing are the
//! toolbar's.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::NavKeys;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::toolbar::{ToolItem, Toolbar, ToolbarAction};

const ID: WidgetId = WidgetId::of("toolbars");

/// The query toolbar's commands; `running` enables Stop and disables Run.
fn query_items(running: bool) -> Vec<ToolItem> {
    vec![
        busy_if(
            ToolItem::new("Run")
                .shortcut("F5")
                .tip("Run the statement under the cursor"),
            running,
            "A query is running",
        ),
        busy_if(
            ToolItem::new("Run all")
                .shortcut("Shift+F5")
                .tip("Run every statement"),
            running,
            "A query is running",
        ),
        busy_if(
            ToolItem::new("Stop")
                .tip("Stop the running query")
                .separator(),
            !running,
            "No query is running",
        ),
        ToolItem::new("Format").tip("Format the script"),
        ToolItem::new("Explain")
            .tip("Show the query plan")
            .separator(),
        ToolItem::new("Wrap").toggle(false).tip("Wrap long lines"),
        ToolItem::new("Line numbers").toggle(true),
    ]
}

/// `item`, disabled with `why` when `off`.
fn busy_if(item: ToolItem, off: bool, why: &str) -> ToolItem {
    if off { item.reason(why) } else { item }
}

/// The narrow toolbar's width: Run and Run all fit, the rest goes behind
/// the overflow button.
const NARROW_W: u16 = 32;

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
    /// The query commands in [`NARROW_W`] cells: the overflow menu.
    pub narrow: Toolbar,
    pub format: Toolbar,
    pub running: bool,
    pub last: String,
    /// `toolbars.nav`, `.wrap`, `.skip_disabled`, `.gap`, `.overflow`.
    opts: [usize; 5],
}

const OPT_KEYS: [&str; 5] = [
    "toolbars.nav",
    "toolbars.wrap",
    "toolbars.skip_disabled",
    "toolbars.gap",
    "toolbars.overflow",
];
const OPT_DEFAULTS: [usize; 5] = [0, 0, 1, 1, 0];

impl ToolbarsPage {
    pub fn new() -> Self {
        let mut p = Self {
            template: Template::new(ID, states_grid()),
            query: Toolbar::new(ID.sub("query"), query_items(false)),
            narrow: Toolbar::new(ID.sub("narrow"), query_items(false)),
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
        let style = |t: &Toolbar, o: &[usize; 5]| {
            let cursor = t.cursor;
            let mut n = Toolbar::new(t.id, t.items.clone())
                .nav(if o[0] == 1 {
                    NavKeys::VIM
                } else {
                    NavKeys::DESKTOP
                })
                .wrap(o[1] == 1)
                .skip_disabled(o[2] == 1)
                .gap(o[3] as u16)
                .overflow(o[4] != 2);
            if o[4] == 1 {
                n = n.more_glyph("…");
            }
            n.cursor = cursor;
            n
        };
        self.query = style(&self.query, &self.opts);
        self.narrow = style(&self.narrow, &self.opts);
        self.format = style(&self.format, &self.opts);
    }

    /// The toolbars in focus order: query, narrow, format.
    fn bar_mut(&mut self, k: usize) -> &mut Toolbar {
        match k {
            0 => &mut self.query,
            1 => &mut self.narrow,
            _ => &mut self.format,
        }
    }

    /// `k` is a toolbar of query commands (the wide or the narrow one).
    fn act(&mut self, a: ToolbarAction, k: usize, cx: &mut Cx) {
        let label = match a {
            ToolbarAction::Activated(i) | ToolbarAction::Toggled(i, _) => {
                self.bar_mut(k).items[i].label.clone()
            }
        };
        self.last = format!("{a:?} · {label}");
        if k < 2
            && let ToolbarAction::Activated(i) = a
        {
            match i {
                0 | 1 => self.running = true,
                2 => self.running = false,
                _ => {}
            }
            // the run state enables Stop and disables Run, keeping toggles
            for bar in [&mut self.query, &mut self.narrow] {
                let mut items = query_items(self.running);
                for (n, o) in items.iter_mut().zip(&bar.items) {
                    if n.on.is_some() {
                        n.on = o.on;
                    }
                }
                bar.items = items;
            }
        }
        cx.status(self.last.clone());
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[5, 1, 5, 1, 5, 1, 6, 0]);

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

        let panel = Panel::card(Some("Narrow toolbar")).meta("» holds what does not fit");
        let bg = panel.bg(t);
        let inner = panel.render(rows[2], buf, t);
        let w = NARROW_W.min(inner.width);
        self.narrow
            .render(Rect::new(inner.x, inner.y, w, 1), buf, ctx, bg);
        // what the overflow menu holds, with why a disabled item waits
        let hidden: Vec<String> = self
            .narrow
            .hidden()
            .iter()
            .map(|&i| {
                let it = &self.narrow.items[i];
                match (&it.reason, it.disabled) {
                    (Some(r), true) => format!("{} ({r})", it.label),
                    _ => it.label.clone(),
                }
            })
            .collect();
        let text = if hidden.is_empty() {
            "everything fits: no » button".to_owned()
        } else {
            format!("behind »: {}", hidden.join(" · "))
        };
        buf.set_string(
            inner.x,
            inner.y + 2,
            junie_tui::ui::text::truncate(&text, inner.width as usize),
            t.muted().bg(bg),
        );

        let panel = Panel::card(Some("Formatting toolbar")).meta("toggles");
        let bg = panel.bg(t);
        let inner = panel.render(rows[4], buf, t);
        self.format
            .render(Rect::new(inner.x, inner.y, inner.width, 1), buf, ctx, bg);

        let panel = Panel::card(Some("Last"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[6], buf, t);
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
            Overflow,
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

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() != Section::Playground {
            return Outcome::Ignored;
        }
        match ev {
            PageEvent::Key(key) => {
                let f = cx.focus.current();
                for k in 0..3 {
                    let bar = self.bar_mut(k);
                    if f == Some(bar.id) {
                        let (out, a) = bar.on_key(key);
                        if let Some(a) = a {
                            self.act(a, k, cx);
                        }
                        return out;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Pointer(ev) => {
                for k in 0..3 {
                    let bar = self.bar_mut(k);
                    let bid = bar.id;
                    // the overflow menu is the bar's too
                    if let Some(r) = bar.route(ev) {
                        if !ev.is(Phase::Click) {
                            return r.outcome();
                        }
                        let (out, a) = r.into_parts();
                        cx.focus.focus(bid);
                        if let Some(a) = a {
                            self.act(a, k, cx);
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
        if self.narrow.menu_open() {
            return vec![("↑↓", "Move"), ("Enter", "Run"), ("Esc", "Close")];
        }
        if focus == Some(self.narrow.id) && self.narrow.on_more() {
            return vec![("←→", "Move"), ("Enter / ↓", "Show hidden")];
        }
        if [self.query.id, self.narrow.id, self.format.id]
            .iter()
            .any(|&id| focus == Some(id))
        {
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
