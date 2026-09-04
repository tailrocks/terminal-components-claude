use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::panel::{Panel, ScrollPanel};
use junie_tui::widgets::scrollbar;

const ID: WidgetId = WidgetId::of("scrolling");

pub struct ScrollingPage {
    prose: ScrollPanel,
    list: ListBox,
    log: ScrollPanel,
}

fn prose_style(t: &Theme, _l: &str) -> ratatui::style::Style {
    t.secondary()
}

impl ScrollingPage {
    pub fn new() -> Self {
        let mut text: Vec<String> = Vec::new();
        for _ in 0..3 {
            text.extend(crate::data::prose().split('\n').map(str::to_owned));
            text.push(String::new());
        }
        let items = (1..=120)
            .map(|i| {
                ListItem::new(&format!("Row {i:03}")).meta(if i % 7 == 0 { "flagged" } else { "" })
            })
            .collect();
        let mut log = ScrollPanel::new(ID.sub("log"), crate::data::log_lines(400));
        log.follow = true;
        Self {
            prose: ScrollPanel::new(ID.sub("prose"), text).wrap(true),
            list: ListBox::new(ID.sub("list"), items, SelectMode::Single),
            log,
        }
    }
}

impl Page for ScrollingPage {
    fn title(&self) -> &'static str {
        "Scrolling"
    }
    fn blurb(&self) -> &'static str {
        "Wheel under the pointer, keys on the focused container, thumb shows where you are"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let third = area.width / 3;
        let cols = [
            Rect::new(area.x, area.y, third.saturating_sub(1), area.height),
            Rect::new(
                area.x + third + 1,
                area.y,
                third.saturating_sub(1),
                area.height,
            ),
            Rect::new(
                area.x + 2 * third + 2,
                area.y,
                area.width.saturating_sub(2 * third + 2),
                area.height,
            ),
        ];
        let pos = scrollbar::position_label(&self.prose.scroll);
        let panel = Panel::card(Some("Wrapped text"))
            .meta(&pos)
            .focused(ctx.interaction.focused(self.prose.id));
        let bg = panel.bg(t);
        let inner = panel.render(cols[0], buf, t);
        self.prose.render(inner, buf, ctx, bg, prose_style);

        let pos = scrollbar::position_label(&self.list.scroll);
        let panel = Panel::card(Some("Long list"))
            .meta(&pos)
            .focused(ctx.interaction.focused(self.list.id));
        let bg = panel.bg(t);
        let inner = panel.render(cols[1], buf, t);
        self.list.render(inner, buf, ctx, bg);

        let pos = scrollbar::position_label(&self.log.scroll);
        let meta = if self.log.follow {
            format!("{pos} · following")
        } else {
            pos
        };
        let lf = ctx.interaction.focused(self.log.id);
        let panel = Panel::card(Some("Log")).focused(lf).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(cols[2], buf, t);
        self.log
            .render(inner, buf, ctx, bg, crate::pages::panels::log_style);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                // keep the log alive so follow-tail is observable
                if self.log.lines.len() < 2000 && self.log.follow {
                    let n = self.log.lines.len();
                    if let Some(line) = crate::data::log_lines(n + 1).pop() {
                        self.log.push(line);
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if f == self.prose.id {
                    return self.prose.on_key(key);
                }
                if f == self.log.id {
                    return self.log.on_key(key);
                }
                if f == self.list.id {
                    return self.list.on_key(key);
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                if let Some(row) = self.list.locate(*id) {
                    cx.focus.focus(self.list.id);
                    return self.list.on_click(row);
                }
                if *id == scrollbar::id_for(self.list.id) {
                    return self.list.on_scrollbar(*pos);
                }
                for p in [&mut self.prose, &mut self.log] {
                    if scrollbar::id_for(p.id) == *id {
                        return p.on_scrollbar(*pos);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if *pressed == scrollbar::id_for(self.list.id) {
                    return self.list.on_scrollbar(*pos);
                }
                for p in [&mut self.prose, &mut self.log] {
                    if scrollbar::id_for(p.id) == *pressed {
                        return p.on_scrollbar(*pos);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.list.owns(*id) {
                    return self.list.on_wheel(*delta);
                }
                for p in [&mut self.prose, &mut self.log] {
                    if p.id == *id || scrollbar::id_for(p.id) == *id {
                        return p.on_wheel(*delta);
                    }
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn animating(&self) -> bool {
        self.log.follow && self.log.lines.len() < 2000
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.log.id) {
            vec![("↑ ↓", "Scroll"), ("f", "Follow"), ("G", "End")]
        } else if focus == Some(self.list.id) {
            vec![("↑ ↓", "Move"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        } else {
            vec![("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        }
    }
}
