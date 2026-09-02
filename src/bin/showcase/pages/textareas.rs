use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::input::InputEvent;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::textarea::TextArea;

const ID: WidgetId = WidgetId::of("textareas");

pub struct TextAreasPage {
    areas: Vec<TextArea>,
}

impl TextAreasPage {
    pub fn new() -> Self {
        let long = (1..=28)
            .map(|i| match i % 4 {
                0 => format!("{i:>2}. Run the integration suite and attach the report."),
                1 => format!("{i:>2}. Read src/api/billing.rs before touching invoices."),
                2 => format!("{i:>2}. Keep the public API stable; add, never rename."),
                _ => format!("{i:>2}. Open a PR against main with a clear summary."),
            })
            .collect::<Vec<_>>()
            .join("\n");
        let areas = vec![
            TextArea::new(ID.child(0), "Task description", 8)
                .value(&long)
                .help("Enter inserts a newline · Esc finishes"),
            TextArea::new(ID.child(1), "Notes", 8)
                .placeholder("Anything the agent should know…")
                .help("Optional"),
            TextArea::new(ID.child(2), "Read-only transcript", 4)
                .value("Junie: Reading 14 files…\nJunie: Plan ready. 3 steps.")
                .disabled(true),
            TextArea::new(ID.child(3), "Commit message", 4)
                .value("fix stuff")
                .error(Some("Use the imperative mood and explain why")),
        ];
        Self { areas }
    }

    fn index_of(&self, id: WidgetId) -> Option<usize> {
        self.areas
            .iter()
            .position(|a| a.id == id || scrollbar::id_for(a.id) == id)
    }
}

impl Page for TextAreasPage {
    fn title(&self) -> &'static str {
        "Text areas"
    }
    fn blurb(&self) -> &'static str {
        "Multi-line editing, wrapping cursor motion, scroll position"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[13, 1, 10]);
        let panel = Panel::card(Some("Playground")).meta("Enter Edit · Esc Done · Tab Next");
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let (l, r) = crate::pages::layout::columns(inner, inner.width / 2 - 2, 4);
        self.areas[0].render(l, buf, ctx, bg);
        self.areas[1].render(r, buf, ctx, bg);

        let panel = Panel::card(Some("Disabled and error"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[2], buf, t);
        let (l, r) = crate::pages::layout::columns(inner, inner.width / 2 - 2, 4);
        self.areas[2].render(l, buf, ctx, bg);
        self.areas[3].render(r, buf, ctx, bg);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let Some(i) = self.index_of(f) else {
                    return Outcome::Ignored;
                };
                let (out, iev) = self.areas[i].on_key(key);
                match iev {
                    Some(InputEvent::CommittedTab { backward: false }) => cx.focus_next(),
                    Some(InputEvent::CommittedTab { backward: true }) => cx.focus_prev(),
                    Some(InputEvent::Committed) => cx.status("Saved"),
                    _ => {}
                }
                out
            }
            PageEvent::Paste(text) => {
                let Some(i) = cx.focus.current().and_then(|f| self.index_of(f)) else {
                    return Outcome::Ignored;
                };
                self.areas[i].on_paste(text)
            }
            PageEvent::Click { id, pos } => {
                let Some(i) = self.index_of(*id) else {
                    return Outcome::Ignored;
                };
                if *id == scrollbar::id_for(self.areas[i].id) {
                    let a = &mut self.areas[i];
                    let track =
                        Rect::new(a.area.right().saturating_sub(1), a.area.y, 1, a.area.height);
                    a.scroll
                        .scroll_to(scrollbar::offset_for_click(track, *pos, &a.scroll));
                    return Outcome::Changed;
                }
                let was = cx.focus.is(*id);
                cx.focus.focus(*id);
                self.areas[i].on_click(*pos, was)
            }
            PageEvent::Drag { pressed, pos } => {
                let Some(i) = self.index_of(*pressed) else {
                    return Outcome::Ignored;
                };
                if *pressed == scrollbar::id_for(self.areas[i].id) {
                    let a = &mut self.areas[i];
                    let track =
                        Rect::new(a.area.right().saturating_sub(1), a.area.y, 1, a.area.height);
                    a.scroll
                        .scroll_to(scrollbar::offset_for_click(track, *pos, &a.scroll));
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => match self.index_of(*id) {
                Some(i) => self.areas[i].on_wheel(*delta),
                None => Outcome::Ignored,
            },
            _ => Outcome::Ignored,
        }
    }

    fn editing(&self) -> bool {
        self.areas.iter().any(|a| a.editing)
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        let editing = focus
            .and_then(|f| self.index_of(f))
            .map(|i| self.areas[i].editing)
            .unwrap_or(false);
        if editing {
            vec![
                ("Enter", "Newline"),
                ("Esc", "Done"),
                ("Shift+↑↓", "Select"),
                ("Tab", "Next"),
            ]
        } else {
            vec![("Enter", "Edit"), ("↑ ↓", "Scroll")]
        }
    }
}
