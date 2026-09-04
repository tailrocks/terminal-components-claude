use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::dialog::{Dialog, DialogResult};
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("dialogs");
const CONFIRM: WidgetId = ID.sub("confirm");
const DELETE: WidgetId = ID.sub("delete");
const RENAME: WidgetId = ID.sub("rename");
const CHOICE: WidgetId = ID.sub("choice");

pub struct DialogsPage {
    buttons: Vec<Button>,
    history: Vec<String>,
    task_name: String,
}

fn rename_validator(s: &str) -> Option<String> {
    if s.trim().is_empty() {
        Some("Name cannot be empty".into())
    } else if s.len() > 40 {
        Some("Keep it under 40 characters".into())
    } else {
        None
    }
}

impl DialogsPage {
    pub fn new() -> Self {
        Self {
            buttons: vec![
                Button::primary(ID.child(0), "Confirm run"),
                Button::secondary(ID.child(1), "Rename task…"),
                Button::secondary(ID.child(2), "Three choices…"),
                Button::danger(ID.child(3), "Delete branch…"),
            ],
            history: vec![],
            task_name: "Migrate sessions table".into(),
        }
    }

    fn open(&mut self, i: usize, cx: &mut PageCtx) {
        match i {
            0 => cx.open(Dialog::confirm(
                CONFIRM,
                "Run task now?",
                "Junie will check out chore/uuid-sessions, apply the plan and run the test suite. You can pause at any step.",
                "Run",
            )),
            1 => {
                let input = TextInput::new(RENAME.sub("input"), "Task name")
                    .value(&self.task_name)
                    .required(true)
                    .validator(rename_validator)
                    .help("Shown in the task list and PR title");
                cx.open(Dialog::prompt(RENAME, "Rename task", input, "Rename"));
            }
            2 => {
                let d = Dialog::confirm(
                    CHOICE,
                    "Unsaved changes",
                    "The description was edited. Save before leaving this page?",
                    "Save",
                )
                .with_actions(
                    vec![
                        Button::subtle(CHOICE.sub("cancel"), "Cancel"),
                        Button::secondary(CHOICE.sub("discard"), "Discard"),
                        Button::primary(CHOICE.sub("save"), "Save"),
                    ],
                    Some(0),
                );
                let mut d = d;
                d.initial_focus = CHOICE.sub("save");
                cx.open(d);
            }
            _ => cx.open(Dialog::destructive(
                DELETE,
                "Delete branch?",
                "feat/rate-limit has 14 commits that are not on main. This cannot be undone.",
                "Delete branch",
            )),
        }
    }
}

impl Page for DialogsPage {
    fn title(&self) -> &'static str {
        "Dialogs"
    }
    fn blurb(&self) -> &'static str {
        "Focus is trapped, the page dims, Esc always cancels"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[9, 1, 0]);
        let panel = Panel::card(Some("Open a dialog"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let widths: Vec<u16> = self.buttons.iter().map(|b| b.width()).collect();
        let rects = row_layout(Rect::new(inner.x, inner.y, inner.width, 1), &widths, 2);
        for (b, r) in self.buttons.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
        let notes = [
            "Confirm: primary action focused first · y / n answer directly",
            "Prompt: editing inside a modal, Enter submits, validation blocks",
            "Destructive: Cancel focused first, action in danger style",
        ];
        for (i, n) in notes.iter().enumerate() {
            buf.set_string(
                inner.x,
                inner.y + 2 + i as u16,
                junie_tui::ui::text::truncate(n, inner.width as usize),
                t.muted().bg(bg),
            );
        }
        buf.set_string(
            inner.x,
            inner.y + 5,
            format!("Task: {}", self.task_name),
            t.secondary().bg(bg),
        );

        let panel = Panel::card(Some("Results"));
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(rows[2].x, rows[2].y, rows[2].width, rows[2].height.min(12)),
            buf,
            t,
        );
        if self.history.is_empty() {
            buf.set_string(inner.x, inner.y, "Nothing yet", t.muted().bg(bg));
        }
        for (i, h) in self
            .history
            .iter()
            .rev()
            .take(inner.height as usize)
            .enumerate()
        {
            let st = if i == 0 { t.primary() } else { t.muted() };
            buf.set_string(inner.x, inner.y + i as u16, h, st.bg(bg));
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let Some(i) = self.buttons.iter().position(|b| b.id == f) else {
                    return Outcome::Ignored;
                };
                let (o, act) = self.buttons[i].on_key(key);
                if act {
                    self.open(i, cx);
                }
                o
            }
            PageEvent::Click { id, .. } => {
                let Some(i) = self.buttons.iter().position(|b| b.id == *id) else {
                    return Outcome::Ignored;
                };
                if self.buttons[i].on_click() {
                    self.open(i, cx);
                }
                Outcome::Changed
            }
            PageEvent::DialogClosed { id, result, value } => {
                if *id == RENAME
                    && *result == DialogResult::Action(1)
                    && let Some(v) = value
                {
                    self.task_name = v.clone();
                }
                let (label, msg) = match (*id, *result) {
                    (d, DialogResult::Action(1)) if d == CONFIRM => {
                        ("Run", "Task started".to_owned())
                    }
                    (d, DialogResult::Action(1)) if d == DELETE => {
                        ("Delete", "Branch feat/rate-limit deleted".to_owned())
                    }
                    (d, DialogResult::Action(1)) if d == RENAME => {
                        ("Rename", format!("Renamed to “{}”", self.task_name))
                    }
                    (d, DialogResult::Action(2)) if d == CHOICE => {
                        ("Save", "Description saved".to_owned())
                    }
                    (d, DialogResult::Action(1)) if d == CHOICE => {
                        ("Discard", "Changes discarded".to_owned())
                    }
                    _ => ("Cancel", "Cancelled".to_owned()),
                };
                self.history.push(format!("{label:<8} {msg}"));
                cx.status(msg);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![("Enter", "Open")]
    }
}
