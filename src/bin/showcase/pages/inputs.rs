use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::template::{Section, Template};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::input::{InputAction, TextInput};
use junie_tui::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("inputs");

fn email(s: &str) -> Option<String> {
    if s.is_empty() {
        Some("Required".into())
    } else if !s.contains('@') || !s.contains('.') {
        Some("Enter a valid email address".into())
    } else {
        None
    }
}

pub struct InputsPage {
    template: Template,
    fields: Vec<TextInput>,
}

impl InputsPage {
    pub fn new() -> Self {
        let mut owner = TextInput::new(ID.child(2), "Owner email")
            .value("mira@example")
            .required(true)
            .validator(email);
        owner.validate();
        let fields = vec![
            TextInput::new(ID.child(0), "Project name")
                .value("payments-gateway")
                .required(true)
                .help("Used as the working directory name"),
            TextInput::new(ID.child(1), "Branch")
                .placeholder("feat/…")
                .help("Leave empty to work on a detached checkout"),
            owner,
            TextInput::new(ID.child(3), "API token")
                .value("jb_live_••••••••••••")
                .disabled(true)
                .help("Managed by the organization"),
            TextInput::new(ID.child(4), "Search files")
                .placeholder("Type a path or symbol…")
                .edit_entry(junie_tui::core::edit::EditEntry::TypeToEdit)
                .help("Selection: Shift+←→  ·  words: Ctrl+←→  ·  clear: Ctrl+U"),
            TextInput::new(ID.child(5), "API key")
                .masked()
                .reveal_tail(4)
                .value("sk-live-0f3a91c2e7d4b6a8c1f2")
                .help("Masked while typing; the last four characters show once committed"),
        ];
        Self {
            template: Template::new(ID, states_grid()),
            fields,
        }
    }

    fn focused_index(&self, cx: &PageCtx) -> Option<usize> {
        let f = cx.focus.current()?;
        self.fields.iter().position(|i| i.id == f)
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[17, 0]);
        let panel = Panel::card(Some("Playground"))
            .meta("Type to edit · Enter Apply · Esc Cancel · Tab Next");
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let (l, r) = crate::pages::layout::columns(inner, inner.width / 2 - 2, 4);
        let fh = TextInput::HEIGHT;
        let slots = [
            Rect::new(l.x, l.y, l.width, fh),
            Rect::new(r.x, r.y, r.width, fh),
            Rect::new(l.x, l.y + fh, l.width, fh),
            Rect::new(r.x, r.y + fh, r.width, fh),
            Rect::new(l.x, l.y + fh * 2, l.width, fh),
            Rect::new(r.x, r.y + fh * 2, r.width, fh),
        ];
        for (f, slot) in self.fields.iter_mut().zip(slots) {
            if slot.bottom() <= inner.bottom() {
                f.render(slot, buf, ctx, bg);
            }
        }
    }
}

/// The States grid: a plain and a masked field in every state they have,
/// drawn by `TextInput` itself ([`junie_tui::ui::specimen`]).
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    StateGrid::new(id)
        .column_width(34)
        .rows(&[
            Rest, Hover, Focused, Editing, Error, Disabled, Empty, Overflow, Narrow,
        ])
        .column(
            "Text",
            TextInput::new(id.child(0), "Project name")
                .value("payments-gateway")
                .placeholder("(feat/…)")
                .plain_label(),
        )
        .column(
            "Masked",
            TextInput::new(id.child(1), "API key")
                .masked()
                .reveal_tail(4)
                .value("sk-live-0f3a91c2e7d4b6a8c1f2")
                .plain_label(),
        )
}

impl Page for InputsPage {
    fn title(&self) -> &'static str {
        "Inputs"
    }
    fn blurb(&self) -> &'static str {
        "Focus is a bar; editing is a cursor. Enter to edit, Esc to revert."
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
        if let PageEvent::Blur(ch) = ev {
            let Some(i) = self.fields.iter().position(|f| f.id == ch.from) else {
                return Outcome::Ignored;
            };
            return match self.fields[i].on_blur() {
                Some(InputAction::Committed) => {
                    cx.status(format!("{} saved", self.fields[i].label));
                    Outcome::Changed
                }
                Some(InputAction::Cancelled) => {
                    cx.status("Reverted");
                    Outcome::Changed
                }
                _ => Outcome::Ignored,
            };
        }
        match ev {
            PageEvent::Key(key) => {
                let Some(i) = self.focused_index(cx) else {
                    return Outcome::Ignored;
                };
                let (out, iev) = self.fields[i].on_key(key);
                match iev {
                    Some(InputAction::CommittedTab { backward: false }) => cx.focus_next(),
                    Some(InputAction::CommittedTab { backward: true }) => cx.focus_prev(),
                    Some(InputAction::Committed) => {
                        cx.status(format!("{} saved", self.fields[i].label))
                    }
                    Some(InputAction::Cancelled) => cx.status("Reverted"),
                    _ => {}
                }
                out
            }
            PageEvent::Paste(text) => {
                let Some(i) = self.focused_index(cx) else {
                    return Outcome::Ignored;
                };
                self.fields[i].on_paste(text)
            }
            PageEvent::Press { id, pos } => {
                let Some(i) = self.fields.iter().position(|f| f.id == *id) else {
                    return Outcome::Ignored;
                };
                self.fields[i].on_press_count(*pos, cx.clicks.max(1))
            }
            PageEvent::Drag { pressed, pos } => {
                match self.fields.iter().position(|f| f.id == *pressed) {
                    Some(i) => self.fields[i].on_drag(*pos),
                    None => Outcome::Ignored,
                }
            }
            PageEvent::Click { id, pos } => {
                let Some(i) = self.fields.iter().position(|f| f.id == *id) else {
                    return Outcome::Ignored;
                };
                cx.focus.focus(*id);
                self.fields[i].on_click_count(*pos, cx.clicks.max(1))
            }
            _ => Outcome::Ignored,
        }
    }

    fn editing(&self) -> bool {
        self.fields.iter().any(|f| f.editing)
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        let editing = focus
            .and_then(|f| self.fields.iter().find(|i| i.id == f))
            .map(|i| i.editing)
            .unwrap_or(false);
        if editing {
            vec![
                ("Enter", "Apply"),
                ("Esc", "Cancel"),
                ("Shift+←→", "Select"),
                ("Ctrl+U", "Clear"),
            ]
        } else {
            vec![("Enter", "Edit")]
        }
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}
