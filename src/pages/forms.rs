use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use crate::ui::ctx::RenderCtx;
use crate::widgets::button::{Button, row_layout};
use crate::widgets::choice::{Checkbox, RadioGroup, Toggle};
use crate::widgets::input::{InputEvent, TextInput};
use crate::widgets::panel::Panel;
use crate::widgets::textarea::TextArea;

const ID: WidgetId = WidgetId::of("forms");

fn email(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else if !s.contains('@') || !s.contains('.') {
        Some("Enter a valid email address".into())
    } else {
        None
    }
}

fn name(s: &str) -> Option<String> {
    if s.trim().is_empty() {
        Some("Required".into())
    } else if s.len() < 4 {
        Some("At least 4 characters".into())
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Submit {
    Idle,
    Busy(std::time::Instant),
    Done,
}

pub struct FormsPage {
    name: TextInput,
    description: TextArea,
    mode: RadioGroup,
    run_tests: Checkbox,
    open_pr: Checkbox,
    auto_approve: Toggle,
    notify: Toggle,
    reviewer: TextInput,
    submit: Button,
    reset: Button,
    state: Submit,
    attempted: bool,
}

impl FormsPage {
    pub fn new() -> Self {
        Self {
            name: TextInput::new(ID.sub("name"), "Task name")
                .required(true)
                .validator(name)
                .placeholder("Short imperative summary"),
            description: TextArea::new(ID.sub("desc"), "Description", 4)
                .placeholder("What should Junie do, and what does done look like?")
                .help("Optional · Markdown"),
            mode: RadioGroup::new(ID.sub("mode"), "Mode", &["Fast", "Balanced", "Thorough"], 1),
            run_tests: Checkbox::new(ID.sub("tests"), "Run tests before opening a PR", true),
            open_pr: Checkbox::new(ID.sub("pr"), "Open a pull request when done", false),
            auto_approve: Toggle::new(ID.sub("auto"), "Auto-approve changes", false),
            notify: Toggle::new(ID.sub("notify"), "Notify on completion", true).disabled(true),
            reviewer: TextInput::new(ID.sub("reviewer"), "Reviewer")
                .validator(email)
                .placeholder("name@company.com")
                .help("Optional"),
            submit: Button::primary(ID.sub("submit"), "Create task"),
            reset: Button::subtle(ID.sub("reset"), "Reset"),
            state: Submit::Idle,
            attempted: false,
        }
    }

    fn validate(&mut self) -> bool {
        let a = self.name.validate();
        let b = self.reviewer.validate();
        a && b
    }

    fn do_submit(&mut self, cx: &mut PageCtx) {
        self.attempted = true;
        if self.name.editing {
            self.name.commit();
        }
        if self.reviewer.editing {
            self.reviewer.commit();
        }
        if !self.validate() {
            cx.status("Fix the highlighted fields");
            if self.name.error.is_some() {
                cx.focus.focus(self.name.id);
            } else {
                cx.focus.focus(self.reviewer.id);
            }
            return;
        }
        self.submit.busy = true;
        self.state = Submit::Busy(std::time::Instant::now());
        cx.status("Creating task…");
    }

    fn do_reset(&mut self, cx: &mut PageCtx) {
        *self = Self::new();
        cx.focus.focus(self.name.id);
        cx.status("Form reset");
    }
}

impl Page for FormsPage {
    fn title(&self) -> &'static str {
        "Forms"
    }
    fn blurb(&self) -> &'static str {
        "Sections, required fields, validation, submission"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let panel = Panel::card(Some("New task")).meta("Ctrl+S Submit");
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(area.x, area.y, area.width, area.height.min(24)),
            buf,
            t,
        );
        let (l, r) = crate::pages::layout::columns(inner, inner.width / 2 - 2, 4);

        // left: task section
        let mut y = l.y;
        buf.set_string(l.x, y, "Task", t.faint().bg(bg));
        y += 1;
        self.name
            .render(Rect::new(l.x, y, l.width, TextInput::HEIGHT), buf, ctx, bg);
        y += TextInput::HEIGHT;
        self.description.render(
            Rect::new(l.x, y, l.width, self.description.height()),
            buf,
            ctx,
            bg,
        );
        y += self.description.height() + 1;
        buf.set_string(l.x, y, "Review", t.faint().bg(bg));
        y += 1;
        self.reviewer
            .render(Rect::new(l.x, y, l.width, TextInput::HEIGHT), buf, ctx, bg);

        // right: options section
        let mut y = r.y;
        buf.set_string(r.x, y, "Options", t.faint().bg(bg));
        y += 1;
        self.mode
            .render(Rect::new(r.x, y, r.width, self.mode.height()), buf, ctx, bg);
        y += self.mode.height() + 1;
        self.run_tests
            .render(Rect::new(r.x, y, r.width, 1), buf, ctx, bg);
        y += 1;
        self.open_pr
            .render(Rect::new(r.x, y, r.width, 1), buf, ctx, bg);
        y += 2;
        self.auto_approve
            .render(Rect::new(r.x, y, r.width, 1), buf, ctx, bg);
        y += 1;
        self.notify
            .render(Rect::new(r.x, y, r.width, 1), buf, ctx, bg);
        y += 1;
        buf.set_string(r.x + 2, y, "Managed by your organization", t.faint().bg(bg));

        // actions
        let ay = inner.bottom().saturating_sub(1);
        let widths = [self.submit.width(), self.reset.width()];
        let rects = row_layout(Rect::new(inner.x, ay, inner.width, 1), &widths, 2);
        self.submit.render(rects[0], buf, ctx, bg);
        self.reset.render(rects[1], buf, ctx, bg);
        let msg = match self.state {
            Submit::Idle
                if self.attempted
                    && (self.name.error.is_some() || self.reviewer.error.is_some()) =>
            {
                Some(("Fix the highlighted fields", t.error_fg()))
            }
            Submit::Busy(_) => Some(("Creating task…", t.secondary())),
            Submit::Done => Some(("Task created ✓", t.accent_fg())),
            _ => None,
        };
        if let Some((m, st)) = msg {
            let x = rects[1].right() + 3;
            if x + m.len() as u16 <= inner.right() {
                buf.set_string(x, ay, m, st.bg(bg));
            }
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                if let Submit::Busy(at) = self.state
                    && at.elapsed() > std::time::Duration::from_millis(1800)
                {
                    self.state = Submit::Done;
                    self.submit.busy = false;
                    cx.status("Task created ✓");
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Key(key) => {
                if key.ctrl_char('s') {
                    self.do_submit(cx);
                    return Outcome::Changed;
                }
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let route_input = |inp: &mut TextInput, cx: &mut PageCtx| -> Outcome {
                    let (out, iev) = inp.on_key(key);
                    match iev {
                        Some(InputEvent::CommittedTab { backward: false }) => cx.focus_next(),
                        Some(InputEvent::CommittedTab { backward: true }) => cx.focus_prev(),
                        _ => {}
                    }
                    out
                };
                if f == self.name.id {
                    return route_input(&mut self.name, cx);
                }
                if f == self.reviewer.id {
                    return route_input(&mut self.reviewer, cx);
                }
                if f == self.description.id {
                    let (out, iev) = self.description.on_key(key);
                    match iev {
                        Some(InputEvent::CommittedTab { backward: false }) => cx.focus_next(),
                        Some(InputEvent::CommittedTab { backward: true }) => cx.focus_prev(),
                        _ => {}
                    }
                    return out;
                }
                if f == self.mode.id {
                    return self.mode.on_key(key);
                }
                if f == self.run_tests.id {
                    return self.run_tests.on_key(key);
                }
                if f == self.open_pr.id {
                    return self.open_pr.on_key(key);
                }
                if f == self.auto_approve.id {
                    return self.auto_approve.on_key(key);
                }
                if f == self.notify.id {
                    return self.notify.on_key(key);
                }
                if f == self.submit.id {
                    let (out, act) = self.submit.on_key(key);
                    if act {
                        self.do_submit(cx);
                    }
                    return out;
                }
                if f == self.reset.id {
                    let (out, act) = self.reset.on_key(key);
                    if act {
                        self.do_reset(cx);
                    }
                    return out;
                }
                Outcome::Ignored
            }
            PageEvent::Paste(text) => {
                if self.name.editing {
                    return self.name.on_paste(text);
                }
                if self.reviewer.editing {
                    return self.reviewer.on_paste(text);
                }
                if self.description.editing {
                    return self.description.on_paste(text);
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                let id = *id;
                if id == self.name.id {
                    let was = cx.focus.is(id);
                    cx.focus.focus(id);
                    return self.name.on_click(*pos, was);
                }
                if id == self.reviewer.id {
                    let was = cx.focus.is(id);
                    cx.focus.focus(id);
                    return self.reviewer.on_click(*pos, was);
                }
                if id == self.description.id {
                    let was = cx.focus.is(id);
                    cx.focus.focus(id);
                    return self.description.on_click(*pos, was);
                }
                for i in 0..self.mode.options.len() {
                    if self.mode.option_id(i) == id {
                        cx.focus.focus(self.mode.id);
                        return self.mode.on_click(i);
                    }
                }
                if id == self.run_tests.id {
                    return self.run_tests.on_click();
                }
                if id == self.open_pr.id {
                    return self.open_pr.on_click();
                }
                if id == self.auto_approve.id {
                    return self.auto_approve.on_click();
                }
                if id == self.notify.id {
                    return self.notify.on_click();
                }
                if id == self.submit.id {
                    if self.submit.on_click() {
                        self.do_submit(cx);
                    }
                    return Outcome::Changed;
                }
                if id == self.reset.id {
                    if self.reset.on_click() {
                        self.do_reset(cx);
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn editing(&self) -> bool {
        self.name.editing || self.reviewer.editing || self.description.editing
    }

    fn animating(&self) -> bool {
        matches!(self.state, Submit::Busy(_))
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.editing() {
            return vec![
                ("Enter", "Commit"),
                ("Esc", "Cancel"),
                ("Tab", "Next field"),
            ];
        }
        match focus {
            Some(f) if f == self.mode.id => vec![("↑ ↓", "Choose"), ("Ctrl+S", "Submit")],
            Some(f)
                if f == self.run_tests.id || f == self.open_pr.id || f == self.auto_approve.id =>
            {
                vec![("Space", "Toggle"), ("Ctrl+S", "Submit")]
            }
            Some(f) if f == self.submit.id || f == self.reset.id => {
                vec![("Enter", "Activate"), ("Ctrl+S", "Submit")]
            }
            _ => vec![("Enter", "Edit"), ("Ctrl+S", "Submit")],
        }
    }
}
