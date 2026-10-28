use std::collections::BTreeMap;
use std::fmt;

use crate::sim::{
    FileEntry, FileKind, FilePreview, FilePreviewContent, FilePreviewError, FilesError,
    FilesSnapshot,
};
use termrock::{
    Cx, EmptyState, FrameRead, Id, ItemKey, List, ListAction, ListState, Panel, Rect, Response,
    TextViewport, Ui, ViewportLine, ViewportState,
};

pub(crate) const LIST: Id = Id::root("holla.files.list");
pub(crate) const PREVIEW: Id = Id::root("holla.files.preview");
const PANEL: Id = Id::root("holla.files.panel");

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    entry: FileEntry,
    label: String,
}

impl fmt::Display for Row {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

fn row_key(row: &Row) -> ItemKey {
    ItemKey::text(row.entry.path())
}

fn list<'a>() -> List<'a, Row, impl Fn(&Row) -> ItemKey> {
    List::new(LIST).key(row_key)
}

struct PreviewSession {
    selected_path: String,
    result: Result<FilePreview, FilePreviewError>,
    lines: Vec<String>,
    viewport: ViewportState,
    visible: bool,
}

impl PreviewSession {
    fn new(selected_path: String, result: Result<FilePreview, FilePreviewError>) -> Self {
        let lines = match &result {
            Ok(preview) => match preview.content() {
                FilePreviewContent::Text { lines, .. } => lines
                    .iter()
                    .enumerate()
                    .map(|(index, line)| format!("{:>5} {line}", index.saturating_add(1)))
                    .collect(),
                FilePreviewContent::Empty => vec!["Empty file".to_owned()],
                FilePreviewContent::Unavailable => {
                    vec!["Content was not modeled for this file".to_owned()]
                }
                FilePreviewContent::Binary { .. } => {
                    vec!["Binary content is not shown".to_owned()]
                }
            },
            Err(error) => vec![preview_error_message(error).to_owned()],
        };
        let mut viewport = ViewportState::default();
        viewport.set_follow(false);
        Self {
            selected_path,
            result,
            lines,
            viewport,
            visible: true,
        }
    }

    fn meta(&self) -> String {
        let mut parts = vec!["Esc/Left back".to_owned()];
        let Ok(preview) = &self.result else {
            if let Err(error) = &self.result {
                parts.push(preview_error_message(error).to_owned());
            }
            return parts.join(" · ");
        };
        if let Some(target) = preview.link_target() {
            parts.push(format!("link → {}", safe_preview_path(target)));
        }
        match preview.content() {
            FilePreviewContent::Text {
                bytes_shown,
                total_bytes,
                truncated_bytes,
                truncated_lines,
                long_lines,
                ..
            } => {
                parts.push(format!("{bytes_shown} of {total_bytes} bytes"));
                if *truncated_bytes {
                    parts.push("first 256 KiB".to_owned());
                }
                if *truncated_lines {
                    parts.push("first 2000 lines".to_owned());
                }
                if *long_lines > 0 {
                    parts.push(format!("{long_lines} long lines cut at 4096"));
                }
            }
            FilePreviewContent::Empty => parts.push("empty file".to_owned()),
            FilePreviewContent::Unavailable => parts.push("content not modeled".to_owned()),
            FilePreviewContent::Binary { reason } => {
                parts.push(format!("binary · {reason} · not shown"));
            }
        }
        parts.join(" · ")
    }

    fn title(&self) -> String {
        format!("Preview · {}", safe_preview_path(&self.selected_path))
    }
}

fn viewport_lines(lines: &[String]) -> Vec<ViewportLine<'_>> {
    lines
        .iter()
        .map(|line| ViewportLine::Plain(line.as_str()))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettleIntentKind {
    Empty,
    FocusOnly,
    Other,
}

fn settle_intents(cx: &Cx<'_>, owner: Id) -> SettleIntentKind {
    let mut saw_intent = false;
    let mut has_other = false;
    for intent in cx.intents(owner) {
        saw_intent = true;
        if !matches!(
            intent,
            termrock::Intent::FocusIn { .. } | termrock::Intent::FocusOut { .. }
        ) {
            has_other = true;
        }
    }

    if has_other {
        SettleIntentKind::Other
    } else if saw_intent {
        SettleIntentKind::FocusOnly
    } else {
        SettleIntentKind::Empty
    }
}

fn update_list_during_settle(
    cx: &mut Cx<'_>,
    selection: &mut ListState,
    rows: &[Row],
) -> Response<()> {
    let widget = list();
    if let Some(response) = widget.update_focus_out(cx, selection) {
        response
    } else {
        widget.update(cx, selection, rows).erase()
    }
}

fn preview_error_message(error: &FilePreviewError) -> &'static str {
    match error {
        FilePreviewError::NotFound(_) => "File not found in the simulated snapshot",
        FilePreviewError::PermissionDenied(_) => "File is not readable in the simulated snapshot",
        FilePreviewError::NotRegularFile(_) => "This entry is not a regular file",
        FilePreviewError::SpecialFile(_) => "Special files are not read",
        FilePreviewError::BrokenLink(_) => "The symbolic link target is missing or cyclic",
    }
}

fn safe_preview_path(path: &str) -> String {
    let mut display = String::with_capacity(path.len());
    for character in path.chars() {
        match character {
            '\t' => display.push_str("    "),
            character if character.is_control() => display.push('�'),
            character => display.push(character),
        }
    }
    display
}

fn preview_viewport() -> TextViewport<'static> {
    TextViewport::new(PREVIEW)
        .wrap(false)
        .horizontal_pan(true)
        .reserve_gutter(false)
}

pub(crate) struct State {
    snapshot: FilesSnapshot,
    directory: String,
    rows: Vec<Row>,
    selection: ListState,
    show_hidden: bool,
    remembered: BTreeMap<String, String>,
    listing_error: Option<FilesError>,
    activation_error: Option<FilesError>,
    preview: Option<PreviewSession>,
}

impl State {
    pub(crate) fn new(snapshot: FilesSnapshot, start: &str) -> Self {
        let (directory, preferred) = match snapshot.get(start) {
            Some(entry) if entry.is_directory() => (start.to_owned(), None),
            Some(_) => (
                FilesSnapshot::parent(start).unwrap_or_else(|| start.to_owned()),
                Some(start.to_owned()),
            ),
            None => (start.to_owned(), None),
        };
        let mut state = Self {
            snapshot,
            directory,
            rows: Vec::new(),
            selection: ListState::default(),
            show_hidden: false,
            remembered: BTreeMap::new(),
            listing_error: None,
            activation_error: None,
            preview: None,
        };
        state.reload(preferred);
        state
    }

    pub(crate) fn selected_path(&self) -> Option<&str> {
        self.selection.cursor().and_then(|key| {
            self.rows
                .iter()
                .find(|row| row_key(row) == key)
                .map(|row| row.entry.path())
        })
    }

    pub(crate) fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.preview.as_ref().is_some_and(|preview| preview.visible) {
            let Some(preview) = self.preview.as_mut() else {
                return Response::ignored();
            };

            let mut response = Response::ignored();
            if cx.update_cause() == termrock::UpdateCause::Settle {
                if settle_intents(cx, LIST) == SettleIntentKind::Other {
                    cx.diagnostic(termrock::Diagnostic::UndeliveredIntent { owner: LIST });
                } else {
                    response |= update_list_during_settle(cx, &mut self.selection, &self.rows);
                }
            }
            let lines = viewport_lines(&preview.lines);
            response |= preview_viewport()
                .update(cx, &mut preview.viewport, &lines)
                .erase();
            return response;
        }

        if self.preview.is_some() {
            if settle_intents(cx, PREVIEW) == SettleIntentKind::Other {
                cx.diagnostic(termrock::Diagnostic::UndeliveredIntent { owner: PREVIEW });
                return Response::ignored();
            }
            let mut response = {
                let Some(preview) = self.preview.as_mut() else {
                    return Response::ignored();
                };
                let lines = viewport_lines(&preview.lines);
                preview_viewport()
                    .update(cx, &mut preview.viewport, &lines)
                    .erase()
            };
            if cx.update_cause() == termrock::UpdateCause::Settle {
                if settle_intents(cx, LIST) == SettleIntentKind::Other {
                    cx.diagnostic(termrock::Diagnostic::UndeliveredIntent { owner: LIST });
                    return response;
                }
                response |= update_list_during_settle(cx, &mut self.selection, &self.rows);
                if !cx.state(PREVIEW).contains(termrock::StateFlags::FOCUSED) {
                    self.preview = None;
                }
                return response;
            }
            if cx.state(PREVIEW).contains(termrock::StateFlags::FOCUSED) {
                return response;
            }
            self.preview = None;
        }

        let mut response = list().update(cx, &mut self.selection, &self.rows);
        let action = response.take_action();
        let response = response.erase();
        match action {
            Some(ListAction::Activated(key)) => self.activate(key, cx),
            Some(ListAction::Chose(_)) | Some(ListAction::Moved) => {
                self.activation_error = None;
            }
            Some(
                ListAction::Toggled(_)
                | ListAction::ToggledAll
                | ListAction::LeaveBackward
                | ListAction::LeaveForward,
            )
            | None => {}
        }
        response
    }

    pub(crate) fn open_selected(&mut self, cx: &mut Cx<'_>) {
        if self.preview.as_ref().is_some_and(|preview| preview.visible) {
            return;
        }
        if let Some(key) = self.selection.cursor() {
            self.activate(key, cx);
        }
    }

    pub(crate) fn close_preview(&mut self, cx: &mut Cx<'_>) -> bool {
        let Some(preview) = self.preview.as_mut().filter(|preview| preview.visible) else {
            return false;
        };
        preview.visible = false;
        cx.focus(LIST);
        true
    }

    pub(crate) fn clear_preview(&mut self) {
        if let Some(preview) = self.preview.as_mut() {
            preview.visible = false;
        }
    }

    pub(crate) fn go_parent(&mut self) {
        let Some(parent) = FilesSnapshot::parent(&self.directory) else {
            return;
        };
        let leaving = self.directory.clone();
        self.remember_current();
        self.remembered.insert(parent.clone(), leaving.clone());
        self.directory = parent;
        self.reload(Some(leaving));
    }

    pub(crate) fn toggle_hidden(&mut self) {
        if self.preview.as_ref().is_some_and(|preview| preview.visible) {
            return;
        }
        let selected = self.selected_path().map(str::to_owned);
        self.show_hidden = !self.show_hidden;
        self.reload(selected);
    }

    fn activate(&mut self, key: ItemKey, cx: &mut Cx<'_>) {
        let Some(entry) = self
            .rows
            .iter()
            .find(|row| row_key(row) == key)
            .map(|row| row.entry.clone())
        else {
            return;
        };
        match entry.kind() {
            FileKind::Directory => match self.snapshot.directory_target(entry.path()) {
                Ok(target) => self.navigate(target),
                Err(error) => self.activation_error = Some(error),
            },
            FileKind::Symlink { .. } => match self.snapshot.directory_target(entry.path()) {
                Ok(target) => self.navigate(target),
                Err(FilesError::NotADirectory(_)) => self.open_preview(entry.path(), cx),
                Err(error) => self.activation_error = Some(error),
            },
            FileKind::File | FileKind::Special => self.open_preview(entry.path(), cx),
        }
    }

    fn open_preview(&mut self, path: &str, cx: &mut Cx<'_>) {
        self.activation_error = None;
        let result = self.snapshot.preview(path);
        self.preview = Some(PreviewSession::new(path.to_owned(), result));
        cx.focus(PREVIEW);
    }

    fn navigate(&mut self, target: String) {
        self.remember_current();
        let preferred = self.remembered.get(&target).cloned();
        self.directory = target;
        self.reload(preferred);
    }

    fn remember_current(&mut self) {
        if let Some(path) = self.selected_path().map(str::to_owned) {
            self.remembered.insert(self.directory.clone(), path);
        }
    }

    fn reload(&mut self, preferred: Option<String>) {
        self.activation_error = None;
        match self.snapshot.list(&self.directory) {
            Ok(entries) => {
                self.listing_error = None;
                self.rows = entries
                    .into_iter()
                    .filter(|entry| self.show_hidden || !entry.is_hidden())
                    .map(|entry| {
                        let label = match entry.kind() {
                            FileKind::Directory => format!("▸ {}/", entry.name()),
                            FileKind::Symlink { .. } => format!("↪ {}", entry.name()),
                            FileKind::File => format!("  {}", entry.name()),
                            FileKind::Special => format!("· {}", entry.name()),
                        };
                        Row { entry, label }
                    })
                    .collect();
            }
            Err(error) => {
                self.listing_error = Some(error);
                self.rows.clear();
            }
        }

        let preferred = preferred.or_else(|| self.remembered.get(&self.directory).cloned());
        let index = preferred
            .as_deref()
            .and_then(|path| self.rows.iter().position(|row| row.entry.path() == path))
            .or_else(|| (!self.rows.is_empty()).then_some(0));
        if let Some(index) = index {
            if let Some(row) = self.rows.get(index) {
                self.selection.set_cursor(index, row_key(row));
            }
        } else {
            self.selection = ListState::default();
        }
    }
}

pub(crate) fn draw(state: &State, ui: &mut Ui<'_>, area: Rect) {
    if let Some(preview) = state.preview.as_ref().filter(|preview| preview.visible) {
        let title = preview.title();
        let meta = preview.meta();
        let focused = ui.state(PREVIEW).contains(termrock::StateFlags::FOCUSED);
        Panel::new(PANEL)
            .title(&title)
            .meta(&meta)
            .focused(focused)
            .draw(ui, area, |ui, inner| {
                let lines = viewport_lines(&preview.lines);
                preview_viewport().draw(ui, inner, &preview.viewport, &lines);
            });
        return;
    }
    let visible_count = state.rows.len();
    let meta = state.activation_error.as_ref().map_or_else(
        || {
            if state.show_hidden {
                format!("{visible_count} entries · hidden shown")
            } else {
                format!("{visible_count} entries · h shows hidden")
            }
        },
        ToString::to_string,
    );
    let listing_message = state.listing_error.as_ref().map(ToString::to_string);
    let panel = Panel::new(PANEL)
        .title("Files")
        .meta(&meta)
        .focused(ui.state(LIST).contains(termrock::StateFlags::FOCUSED));
    panel.draw(ui, area, |ui, inner| {
        let empty = listing_message.as_deref().map_or_else(
            || {
                state.rows.is_empty().then_some(EmptyState::Empty {
                    title: "No visible entries",
                    hint: Some("Use h to show hidden entries"),
                })
            },
            |message| {
                Some(EmptyState::Error {
                    message,
                    detail: Some(&state.directory),
                })
            },
        );
        match empty {
            Some(empty) => list()
                .empty(empty)
                .draw(ui, inner, &state.selection, &state.rows),
            None => list().draw(ui, inner, &state.selection, &state.rows),
        }
    });
}
