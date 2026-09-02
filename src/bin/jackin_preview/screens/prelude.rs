//! Create Workspace prelude: a five-step modal chain over an empty stage.
//! Nothing is persisted here; the last step hands a pending Workspace to
//! the Editor. Every rewind reopens the previous modal with its last state.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::text::width;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::picker::{Picker, PickerItem};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::modals::{BrowserResult, ChoiceDialog, FileBrowser};
use super::{Cx, Go, Modal, ModalResult, ModalTag, Screen};
use crate::domain::workspace::{Mount, Workspace};
use crate::sim::world::World;

const BROWSER: WidgetId = WidgetId::of("prelude.browser");
const DEST: WidgetId = WidgetId::of("prelude.dest");
const EDIT: WidgetId = WidgetId::of("prelude.edit");
const WORKDIR: WidgetId = WidgetId::of("prelude.workdir");
const NAME: WidgetId = WidgetId::of("prelude.name");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Source,
    Destination,
    Edit,
    Workdir,
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Source {
    Host { path: String, readonly: bool },
    Git { url: String },
}

impl Source {
    /// Absolute path the mount lands on when the destination is not edited.
    fn default_destination(&self) -> String {
        match self {
            Source::Host { path, .. } => path.clone(),
            Source::Git { url } => format!(
                "/work/{}",
                basename(url.trim_end_matches('/').trim_end_matches(".git"))
            ),
        }
    }
}

fn basename(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
}

pub struct PreludeScreen {
    step: Step,
    /// Last browser location, restored on rewind.
    cwd: String,
    source: Option<Source>,
    dest_choice: usize,
    /// Edited destination text, kept across rewinds.
    dest_text: Option<String>,
    destination: Option<String>,
    edit_used: bool,
    workdir: Option<String>,
    workdir_rows: Vec<String>,
    name: Option<String>,
}

impl PreludeScreen {
    pub fn new(w: &World) -> Self {
        Self {
            step: Step::Source,
            cwd: w.cwd.clone(),
            source: None,
            dest_choice: 0,
            dest_text: None,
            destination: None,
            edit_used: false,
            workdir: None,
            workdir_rows: vec![],
            name: None,
        }
    }

    /// `✓ Source · Destination · Edit skipped · Working dir · Name`
    pub fn stepper_line(&self) -> String {
        let order = [
            Step::Source,
            Step::Destination,
            Step::Edit,
            Step::Workdir,
            Step::Name,
        ];
        let idx = |s: Step| order.iter().position(|x| *x == s).unwrap_or(0);
        let cur = idx(self.step);
        let parts: Vec<String> = order
            .iter()
            .map(|s| {
                let label = match s {
                    Step::Source => "Source",
                    Step::Destination => "Destination",
                    Step::Edit => "Edit",
                    Step::Workdir => "Working dir",
                    Step::Name => "Name",
                };
                let i = idx(*s);
                if *s == Step::Edit && i < cur && !self.edit_used {
                    "Edit skipped".into()
                } else if i < cur {
                    format!("✓ {label}")
                } else {
                    label.into()
                }
            })
            .collect();
        parts.join(" · ")
    }

    fn title(step: Step) -> &'static str {
        match step {
            Step::Source => "New workspace · step 1 of 5 · Source",
            Step::Destination => "New workspace · step 2 of 5 · Destination",
            Step::Edit => "New workspace · step 3 of 5 · Edit destination",
            Step::Workdir => "New workspace · step 4 of 5 · Working dir",
            Step::Name => "New workspace · step 5 of 5 · Name",
        }
    }

    fn open(&mut self, step: Step, w: &World, cx: &mut Cx, error: Option<String>) {
        self.step = step;
        match step {
            Step::Source => {
                let b = FileBrowser::new(BROWSER, Self::title(step), &self.cwd, true, true, w);
                cx.open(Modal::Browser(b), ModalTag::new("prelude.source"));
            }
            Step::Destination => {
                let Some(src) = &self.source else { return };
                let default = src.default_destination();
                let (source_line, same) = match src {
                    Source::Host { path, readonly } => (
                        format!(
                            "Source  {}{}",
                            w.tilde(path),
                            if *readonly { " · read-only" } else { "" }
                        ),
                        format!("Same path   {default}"),
                    ),
                    Source::Git { url } => (
                        format!("Source  {url} · git"),
                        format!("Default   {default}"),
                    ),
                };
                let c = ChoiceDialog::new(
                    DEST,
                    Self::title(step),
                    "Mount destination",
                    &[&same, "Edit destination…"],
                    self.dest_choice,
                )
                .line(source_line, Tone::Secondary)
                .stepper(&self.stepper_line())
                .buttons(
                    vec![
                        Button::subtle(DEST.sub("back"), "Back"),
                        Button::primary(DEST.sub("ok"), "Next"),
                    ],
                    0,
                )
                .width(58);
                cx.open(Modal::Choice(c), ModalTag::new("prelude.dest"));
            }
            Step::Edit => {
                let value = self
                    .dest_text
                    .clone()
                    .or_else(|| self.source.as_ref().map(|s| s.default_destination()))
                    .unwrap_or_default();
                let mut input = TextInput::new(EDIT.sub("input"), "Destination")
                    .required(true)
                    .placeholder("/work/project")
                    .help("Absolute path inside the Construct")
                    .value(&value);
                input.error = error;
                input.begin_edit();
                let mut d = Dialog::prompt(EDIT, Self::title(step), input, "Next").with_actions(
                    vec![
                        Button::subtle(EDIT.sub("back"), "Back"),
                        Button::primary(EDIT.sub("ok"), "Next"),
                    ],
                    Some(0),
                );
                d.width = 62;
                cx.open(Modal::Dialog(d), ModalTag::new("prelude.edit"));
            }
            Step::Workdir => {
                let Some(dest) = self.destination.clone() else {
                    return;
                };
                let mut p = Picker::new(WORKDIR, Self::title(step));
                p.placeholder = "Search folders…".into();
                p.width = 72;
                p.max_rows = 8;
                p.empty_text = "No folder matches".into();
                self.workdir_rows = self.workdir_candidates(w);
                let rows = self.workdir_rows.clone();
                p.set_items(self.workdir_items(&rows, "", w));
                if let Some(cur) = self
                    .workdir
                    .as_ref()
                    .and_then(|wd| rows.iter().position(|r| r == wd))
                {
                    p.cursor = cur;
                    p.scroll.ensure_visible(cur);
                }
                let _ = dest;
                cx.open(Modal::Picker(p), ModalTag::new("prelude.workdir"));
            }
            Step::Name => {
                let value = self
                    .name
                    .clone()
                    .or_else(|| self.destination.as_deref().map(|d| basename(d).to_owned()))
                    .unwrap_or_default();
                let mut input = TextInput::new(NAME.sub("input"), "Name")
                    .required(true)
                    .placeholder("workspace name")
                    .help("Directory basename by default")
                    .value(&value);
                input.error = error;
                input.begin_edit();
                let mut d = Dialog::prompt(NAME, Self::title(step), input, "Create").with_actions(
                    vec![
                        Button::subtle(NAME.sub("back"), "Back"),
                        Button::primary(NAME.sub("ok"), "Create"),
                    ],
                    Some(0),
                );
                d.width = 62;
                cx.open(Modal::Dialog(d), ModalTag::new("prelude.name"));
            }
        }
    }

    /// Destination first, then its first-level subdirectories mapped from the
    /// source folder onto the destination.
    fn workdir_candidates(&self, w: &World) -> Vec<String> {
        let Some(dest) = self.destination.clone() else {
            return vec![];
        };
        let mut rows = vec![dest.clone()];
        if let Some(Source::Host { path, .. }) = &self.source {
            let src = path.trim_end_matches('/');
            let mut subs: Vec<String> =
                w.fs.iter()
                    .filter(|e| {
                        e.dir
                            && e.path
                                .rsplit_once('/')
                                .is_some_and(|(parent, _)| parent == src)
                    })
                    .map(|e| format!("{}/{}", dest.trim_end_matches('/'), basename(&e.path)))
                    .collect();
            subs.sort();
            rows.extend(subs);
        }
        rows
    }

    /// Rows show the home-shortened path (the picker caps its label column).
    fn workdir_items(&self, rows: &[String], query: &str, w: &World) -> Vec<PickerItem> {
        let q = query.trim().to_lowercase();
        rows.iter()
            .enumerate()
            .filter(|(_, r)| q.is_empty() || r.to_lowercase().contains(&q))
            .map(|(i, r)| PickerItem {
                label: w.tilde(r),
                detail: String::new(),
                glyph: if i == 0 { "▪" } else { "·" },
                group: "",
                tag: if i == 0 { Some("destination") } else { None },
                matched: vec![],
                disabled: false,
            })
            .collect()
    }

    fn rewind(&mut self, w: &World, cx: &mut Cx) {
        let prev = match self.step {
            Step::Source => {
                cx.status("Cancelled · nothing created");
                cx.go(Go::Manager);
                return;
            }
            Step::Destination => Step::Source,
            Step::Edit => Step::Destination,
            Step::Workdir => {
                if self.edit_used {
                    Step::Edit
                } else {
                    Step::Destination
                }
            }
            Step::Name => Step::Workdir,
        };
        self.open(prev, w, cx, None);
    }

    fn validate_destination(text: &str) -> Result<String, String> {
        let t = text.trim_end();
        if t != text {
            return Err("Destination must not end with spaces".into());
        }
        if t.is_empty() {
            return Err("Destination is required".into());
        }
        if !t.starts_with('/') {
            return Err("Destination must be an absolute path".into());
        }
        if t.contains("//") {
            return Err("Destination has an empty path segment".into());
        }
        Ok(t.to_owned())
    }

    fn validate_name(text: &str, w: &World) -> Result<String, String> {
        let t = text.trim();
        if t.is_empty() {
            return Err("Name is required".into());
        }
        if t.contains('/') {
            return Err("Name cannot contain /".into());
        }
        if w.workspaces.iter().any(|ws| ws.name == t) {
            return Err(format!("A workspace named {t} already exists"));
        }
        Ok(t.to_owned())
    }

    fn create(&mut self, w: &World, cx: &mut Cx) {
        let (Some(src), Some(dest), Some(name)) = (&self.source, &self.destination, &self.name)
        else {
            return;
        };
        let workdir = self.workdir.clone().unwrap_or_else(|| dest.clone());
        let mut ws = Workspace::new(w.next_workspace_id, name, &workdir);
        ws.mounts = vec![match src {
            Source::Host { path, readonly } => {
                Mount::host(&w.tilde(path), dest).readonly(*readonly)
            }
            Source::Git { url } => Mount::git(url, dest),
        }];
        cx.status(format!("Workspace {name} · review and save in the editor"));
        cx.go(Go::Editor {
            workspace: None,
            pending: Some(Box::new(ws)),
        });
    }
}

impl Screen for PreludeScreen {
    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.open(Step::Source, w, cx, None);
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        // no modal on top: the chain was interrupted; reopen the current step
        match key.code {
            KeyCode::Esc => self.rewind(w, cx),
            _ => self.open(self.step, w, cx, None),
        }
        Outcome::Changed
    }

    fn picker_items(&mut self, tag: &ModalTag, query: &str, w: &World) -> Option<Vec<PickerItem>> {
        if tag.kind != "prelude.workdir" {
            return None;
        }
        Some(self.workdir_items(&self.workdir_rows.clone(), query, w))
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        match (tag.kind, result) {
            ("prelude.source", ModalResult::Browser(r)) => match r {
                BrowserResult::Chosen { path, readonly } => {
                    if let Some((parent, _)) = path.rsplit_once('/')
                        && !parent.is_empty()
                    {
                        self.cwd = parent.to_owned();
                    }
                    self.source = Some(Source::Host { path, readonly });
                    self.open(Step::Destination, w, cx, None);
                }
                BrowserResult::GitUrl(url) => {
                    self.source = Some(Source::Git { url });
                    self.open(Step::Destination, w, cx, None);
                }
                BrowserResult::Cancelled => self.rewind(w, cx),
            },
            ("prelude.dest", ModalResult::Choice(Some(choice))) => {
                self.dest_choice = choice;
                if choice == 0 {
                    self.destination = self.source.as_ref().map(|s| s.default_destination());
                    self.edit_used = false;
                    self.open(Step::Workdir, w, cx, None);
                } else {
                    self.open(Step::Edit, w, cx, None);
                }
            }
            ("prelude.dest", ModalResult::Choice(None)) => self.rewind(w, cx),
            (
                "prelude.edit",
                ModalResult::Dialog {
                    action: Some(1),
                    text,
                },
            ) => {
                let text = text.unwrap_or_default();
                self.dest_text = Some(text.clone());
                match Self::validate_destination(&text) {
                    Ok(dest) => {
                        self.destination = Some(dest);
                        self.edit_used = true;
                        self.open(Step::Workdir, w, cx, None);
                    }
                    Err(e) => self.open(Step::Edit, w, cx, Some(e)),
                }
            }
            ("prelude.edit", ModalResult::Dialog { text, .. }) => {
                self.dest_text = text;
                self.rewind(w, cx);
            }
            ("prelude.workdir", ModalResult::Picked(i)) => {
                self.workdir = self.workdir_rows.get(i).cloned();
                self.open(Step::Name, w, cx, None);
            }
            ("prelude.workdir", ModalResult::Cancelled) => self.rewind(w, cx),
            (
                "prelude.name",
                ModalResult::Dialog {
                    action: Some(1),
                    text,
                },
            ) => {
                let text = text.unwrap_or_default();
                self.name = Some(text.clone());
                match Self::validate_name(&text, w) {
                    Ok(name) => {
                        self.name = Some(name);
                        self.create(w, cx);
                    }
                    Err(e) => self.open(Step::Name, w, cx, Some(e)),
                }
            }
            ("prelude.name", ModalResult::Dialog { text, .. }) => {
                self.name = text.filter(|t| !t.trim().is_empty());
                self.rewind(w, cx);
            }
            (_, ModalResult::Cancelled) => self.rewind(w, cx),
            _ => {}
        }
        Outcome::Changed
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        let t = ctx.theme;
        let line = "New workspace · follow the prompts";
        let lw = width(line) as u16;
        if area.width > lw && area.height > 2 {
            let x = area.x + (area.width - lw) / 2;
            let y = area.y + area.height / 2;
            buf.set_string(x, y, line, t.faint().bg(t.canvas));
        }
        ctx.control(
            WidgetId::of("prelude"),
            Rect::new(area.x, area.y, 1, 1),
            false,
        );
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        match self.step {
            Step::Source => vec![
                hint("Enter", "Open"),
                hint("Backspace", "Up"),
                hint("Space", "Choose"),
                hint("g", "Git URL"),
                hint("Tab", "Next"),
                hint("Esc", "Cancel"),
            ],
            Step::Destination => vec![
                hint("↑↓", "Choose"),
                hint("Enter", "Next"),
                hint("Esc", "Back"),
            ],
            Step::Edit | Step::Name => vec![
                hint("Enter", "Next"),
                hint("Tab", "Buttons"),
                hint("Esc", "Back"),
            ],
            Step::Workdir => vec![
                hint("↑↓", "Move"),
                hint("Type", "Filter"),
                hint("Enter", "Next"),
                hint("Esc", "Back"),
            ],
        }
    }

    fn crumb(&self, _w: &World) -> String {
        "Workspaces › new workspace".into()
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(WidgetId::of("prelude"))
    }

    fn on_esc_top(&mut self, w: &mut World, cx: &mut Cx) -> Outcome {
        self.rewind(w, cx);
        Outcome::Changed
    }
}
