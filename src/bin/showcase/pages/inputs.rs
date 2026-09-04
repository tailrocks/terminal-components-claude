use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::{RenderCtx, VisualState, fill};
use junie_tui::widgets::input::{InputEvent, TextInput};
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
                .help("Selection: Shift+← →  ·  words: Ctrl+← →  ·  clear: Ctrl+U"),
            TextInput::new(ID.child(5), "API key")
                .masked()
                .reveal_tail(4)
                .value("sk-live-0f3a91c2e7d4b6a8c1f2")
                .help("Masked while typing; the last four characters show once committed"),
        ];
        Self { fields }
    }

    fn focused_index(&self, cx: &PageCtx) -> Option<usize> {
        let f = cx.focus.current()?;
        self.fields.iter().position(|i| i.id == f)
    }
}

fn static_field(
    buf: &mut Buffer,
    t: &junie_tui::theme::Theme,
    at: Rect,
    label: &str,
    text: &str,
    s: VisualState,
) {
    let bg = t.surface;
    let (x, y, w) = (at.x, at.y, at.width);
    buf.set_string(x, y, label, t.secondary().bg(bg));
    let field = Rect::new(x + 16, y, w, 1);
    let fs = t.field_style(s);
    fill(buf, field, fs);
    buf.set_string(field.x, y, "▎", t.gutter(s, fs.bg.unwrap_or(bg), false));
    let style = if text.starts_with('(') {
        t.placeholder(s)
    } else {
        fs
    };
    let style = if s.editing {
        style
            .add_modifier(ratatui::style::Modifier::UNDERLINED)
            .underline_color(t.accent)
    } else {
        style
    };
    buf.set_string(field.x + 2, y, text, style);
    if s.editing {
        // simulated cursor cell
        let cx = field.x + 2 + junie_tui::ui::text::width(text) as u16;
        buf.set_string(cx, y, " ", ratatui::style::Style::new().bg(t.text_primary));
    }
    if s.error {
        buf.set_string(
            field.right() - 2,
            y,
            "!",
            fs.fg(t.error).add_modifier(ratatui::style::Modifier::BOLD),
        );
    }
}

impl Page for InputsPage {
    fn title(&self) -> &'static str {
        "Inputs"
    }
    fn blurb(&self) -> &'static str {
        "Focus is a bar; editing is a cursor. Enter to edit, Esc to revert."
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[17, 1, 0]);
        let panel =
            Panel::card(Some("Playground")).meta("Enter Edit · Esc Cancel · Tab Commit + next");
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

        let panel = Panel::card(Some("State reference")).meta("static");
        let inner = panel.render(rows[2], buf, t);
        let w = inner.width.saturating_sub(18).min(34);
        let states: [(&str, &str, VisualState); 8] = [
            ("default", "payments-gateway", VisualState::default()),
            ("placeholder", "(feat/…)", VisualState::default()),
            (
                "hover",
                "payments-gateway",
                VisualState {
                    hovered: true,
                    ..Default::default()
                },
            ),
            (
                "focused",
                "payments-gateway",
                VisualState {
                    focused: true,
                    ..Default::default()
                },
            ),
            (
                "editing",
                "payments-gateway",
                VisualState {
                    focused: true,
                    editing: true,
                    ..Default::default()
                },
            ),
            (
                "error",
                "mira@example",
                VisualState {
                    error: true,
                    ..Default::default()
                },
            ),
            (
                "error + focus",
                "mira@example",
                VisualState {
                    error: true,
                    focused: true,
                    ..Default::default()
                },
            ),
            (
                "disabled",
                "jb_live_••••",
                VisualState {
                    disabled: true,
                    ..Default::default()
                },
            ),
        ];
        for (i, (name, text, s)) in states.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            static_field(buf, t, Rect::new(inner.x, y, w, 1), name, text, *s);
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                let Some(i) = self.focused_index(cx) else {
                    return Outcome::Ignored;
                };
                let (out, iev) = self.fields[i].on_key(key);
                match iev {
                    Some(InputEvent::CommittedTab { backward: false }) => cx.focus_next(),
                    Some(InputEvent::CommittedTab { backward: true }) => cx.focus_prev(),
                    Some(InputEvent::Committed) => {
                        cx.status(format!("{} saved", self.fields[i].label))
                    }
                    Some(InputEvent::Cancelled) => cx.status("Reverted"),
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
            PageEvent::Click { id, pos } => {
                let Some(i) = self.fields.iter().position(|f| f.id == *id) else {
                    return Outcome::Ignored;
                };
                let was = cx.focus.is(*id);
                cx.focus.focus(*id);
                self.fields[i].on_click(*pos, was)
            }
            _ => Outcome::Ignored,
        }
    }

    fn editing(&self) -> bool {
        self.fields.iter().any(|f| f.editing)
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        let editing = focus
            .and_then(|f| self.fields.iter().find(|i| i.id == f))
            .map(|i| i.editing)
            .unwrap_or(false);
        if editing {
            vec![
                ("Enter", "Commit"),
                ("Esc", "Cancel"),
                ("Shift+← →", "Select"),
                ("Ctrl+U", "Clear"),
            ]
        } else {
            vec![("Enter", "Edit")]
        }
    }
}
