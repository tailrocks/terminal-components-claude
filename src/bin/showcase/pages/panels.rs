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

const ID: WidgetId = WidgetId::of("panels");

pub struct PanelsPage {
    prose: ScrollPanel,
    log: ScrollPanel,
    nested: ListBox,
}

fn prose_style(t: &Theme, _line: &str) -> ratatui::style::Style {
    t.secondary()
}

pub fn log_style(t: &Theme, line: &str) -> ratatui::style::Style {
    if line.contains(" error ") {
        t.error_fg()
    } else if line.contains(" warn ") {
        t.primary().fg(t.warning)
    } else {
        t.secondary()
    }
}

impl PanelsPage {
    pub fn new() -> Self {
        let prose = ScrollPanel::new(
            ID.sub("prose"),
            crate::data::prose()
                .split('\n')
                .map(str::to_owned)
                .collect(),
        )
        .wrap(true);
        let log = ScrollPanel::new(ID.sub("log"), crate::data::log_lines(60));
        let nested = ListBox::new(
            ID.sub("nested"),
            vec![
                ListItem::new("Local"),
                ListItem::new("CLI"),
                ListItem::new("Cloud").disabled(true),
            ],
            SelectMode::Single,
        );
        Self { prose, log, nested }
    }

    fn panels(&mut self) -> [&mut ScrollPanel; 2] {
        [&mut self.prose, &mut self.log]
    }
}

impl Page for PanelsPage {
    fn title(&self) -> &'static str {
        "Panels"
    }
    fn blurb(&self) -> &'static str {
        "Cards group; a frame only where a pane needs an edge; nothing boxed twice"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let (l, r) = crate::pages::layout::columns(area, area.width / 2 - 1, 2);
        let lrows = crate::pages::layout::rows(l, &[7, 1, 6, 1, 7, 0]);

        // titled card
        let panel = Panel::card(Some("Titled card")).meta("surface");
        let bg = panel.bg(t);
        let inner = panel.render(lrows[0], buf, t);
        for (i, line) in junie_tui::ui::text::wrap("A card is a filled surface. Its title sits in the top-left and metadata on the right. It never has a border.", inner.width as usize).iter().enumerate() {
            if i < inner.height as usize {
                buf.set_string(inner.x, inner.y + i as u16, line, t.secondary().bg(bg));
            }
        }

        // untitled card
        let panel = Panel::card(None);
        let bg = panel.bg(t);
        let inner = panel.render(lrows[2], buf, t);
        for (i, line) in junie_tui::ui::text::wrap(
            "Untitled card. Same surface, content starts at the padding edge.",
            inner.width as usize,
        )
        .iter()
        .enumerate()
        {
            if i < inner.height as usize {
                buf.set_string(inner.x, inner.y + i as u16, line, t.secondary().bg(bg));
            }
        }

        // nested: card containing a framed pane containing a list
        let panel = Panel::card(Some("Nested"));
        let bg = panel.bg(t);
        let inner = panel.render(lrows[4], buf, t);
        // nested grouping is a muted label and indent, never a second box
        buf.set_string(inner.x, inner.y, "Target", t.muted().bg(bg));
        let group = Rect::new(
            inner.x,
            inner.y + 1,
            inner.width.min(30),
            inner.height.saturating_sub(1).min(3),
        );
        self.nested.render(group, buf, ctx, bg);
        let note_x = group.right() + 2;
        if note_x + 20 < inner.right() {
            for (i, line) in junie_tui::ui::text::wrap("A group inside a card is a muted label plus indent. The focus bar stays on the control.", (inner.right() - note_x) as usize).iter().enumerate() {
                if (i as u16) < inner.height {
                    buf.set_string(note_x, inner.y + i as u16, line, t.muted().bg(bg));
                }
            }
        }

        // right: two framed scrollable panes
        let rrows = crate::pages::layout::rows(r, &[r.height / 2, 0]);
        let pf = ctx.interaction.focused(self.prose.id);
        let pos = scrollbar::position_label(&self.prose.scroll);
        let panel = Panel::framed(Some("Framed · split pane"))
            .focused(pf)
            .meta(&pos);
        let bg = panel.bg(t);
        let inner = panel.render(rrows[0], buf, t);
        self.prose.render(inner, buf, ctx, bg, prose_style);

        let lf = ctx.interaction.focused(self.log.id);
        let pos = scrollbar::position_label(&self.log.scroll);
        let follow = if self.log.follow { "following" } else { "" };
        let meta = if follow.is_empty() {
            pos
        } else {
            format!("{pos} · {follow}")
        };
        let panel = Panel::card(Some("Card · scrollable"))
            .focused(lf)
            .meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(
                rrows[1].x,
                rrows[1].y + 1,
                rrows[1].width,
                rrows[1].height.saturating_sub(1),
            ),
            buf,
            t,
        );
        self.log.render(inner, buf, ctx, bg, log_style);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if f == self.nested.id {
                    return self.nested.on_key(key);
                }
                for p in self.panels() {
                    if p.id == f {
                        return p.on_key(key);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                if let Some(row) = self.nested.locate(*id) {
                    cx.focus.focus(self.nested.id);
                    return self.nested.on_click(row);
                }
                for p in self.panels() {
                    if scrollbar::id_for(p.id) == *id {
                        return p.on_scrollbar(*pos);
                    }
                    if p.id == *id {
                        return Outcome::Changed;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                for p in self.panels() {
                    if scrollbar::id_for(p.id) == *pressed {
                        return p.on_scrollbar(*pos);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.nested.owns(*id) {
                    return self.nested.on_wheel(*delta);
                }
                for p in self.panels() {
                    if p.id == *id || scrollbar::id_for(p.id) == *id {
                        return p.on_wheel(*delta);
                    }
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.nested.id) {
            vec![("↑ ↓", "Move"), ("Enter", "Choose")]
        } else if focus == Some(self.log.id) {
            vec![("↑ ↓", "Scroll"), ("f", "Follow tail"), ("g G", "Ends")]
        } else {
            vec![("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        }
    }
}
