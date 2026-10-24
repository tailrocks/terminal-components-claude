use std::collections::BTreeMap;
use std::fmt;

use crate::sim::{FileEntry, FileKind, FilesError, FilesSnapshot};
use termrock::{
    Cx, EmptyState, FrameRead, Id, ItemKey, List, ListAction, ListState, Panel, Rect, Response, Ui,
};

pub(crate) const LIST: Id = Id::root("holla.files.list");
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

/// Private synchronous state for the first, read-only Files route.
pub(crate) struct State {
    snapshot: FilesSnapshot,
    directory: String,
    rows: Vec<Row>,
    selection: ListState,
    show_hidden: bool,
    remembered: BTreeMap<String, String>,
    listing_error: Option<FilesError>,
    activation_error: Option<FilesError>,
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
        };
        state.reload(preferred);
        state
    }

    #[cfg(test)]
    pub(crate) fn directory(&self) -> &str {
        &self.directory
    }

    pub(crate) fn selected_path(&self) -> Option<&str> {
        self.selection.cursor().and_then(|key| {
            self.rows
                .iter()
                .find(|row| row_key(row) == key)
                .map(|row| row.entry.path())
        })
    }

    #[cfg(test)]
    pub(crate) fn visible_paths(&self) -> impl Iterator<Item = &str> {
        self.rows.iter().map(|row| row.entry.path())
    }

    #[cfg(test)]
    pub(crate) fn listing_error(&self) -> Option<&FilesError> {
        self.listing_error.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn activation_error(&self) -> Option<&FilesError> {
        self.activation_error.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn show_hidden(&self) -> bool {
        self.show_hidden
    }

    pub(crate) fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = list().update(cx, &mut self.selection, &self.rows);
        let action = response.take_action();
        let response = response.erase();
        match action {
            Some(ListAction::Activated(key)) => self.activate(key),
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

    pub(crate) fn open_selected(&mut self) {
        if let Some(key) = self.selection.cursor() {
            self.activate(key);
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
        let selected = self.selected_path().map(str::to_owned);
        self.show_hidden = !self.show_hidden;
        self.reload(selected);
    }

    fn activate(&mut self, key: ItemKey) {
        let Some(entry) = self
            .rows
            .iter()
            .find(|row| row_key(row) == key)
            .map(|row| row.entry.clone())
        else {
            return;
        };
        match entry.kind() {
            FileKind::Directory | FileKind::Symlink { .. } => {
                match self.snapshot.directory_target(entry.path()) {
                    Ok(target) => self.navigate(target),
                    Err(error) => self.activation_error = Some(error),
                }
            }
            FileKind::File | FileKind::Special => {}
        }
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

    #[cfg(test)]
    fn replace_snapshot(&mut self, snapshot: FilesSnapshot) {
        let selected = self.selected_path().map(str::to_owned);
        self.snapshot = snapshot;
        self.reload(selected);
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

#[cfg(test)]
mod tests {
    use super::State;
    use crate::sim::{FileEntry, FilesSnapshot};

    #[test]
    fn refreshed_rows_keep_selection_by_full_path_and_fall_back_when_removed() {
        let mut first = FilesSnapshot::new();
        first.insert(FileEntry::directory("/"));
        first.insert(FileEntry::directory("/work"));
        first.insert(FileEntry::directory("/work/Alpha"));
        first.insert(FileEntry::directory("/work/beta"));
        first.insert(FileEntry::file("/work/README.md"));
        let mut state = State::new(first, "/work");
        assert_eq!(state.selected_path(), Some("/work/Alpha"));

        let mut reordered = FilesSnapshot::new();
        reordered.insert(FileEntry::directory("/"));
        reordered.insert(FileEntry::directory("/work"));
        reordered.insert(FileEntry::file("/work/0-first.txt"));
        reordered.insert(FileEntry::file("/work/README.md"));
        reordered.insert(FileEntry::directory("/work/Alpha"));
        reordered.insert(FileEntry::directory("/work/Aardvark"));
        reordered.insert(FileEntry::directory("/work/beta"));
        state.replace_snapshot(reordered);
        assert_eq!(state.selected_path(), Some("/work/Alpha"));
        assert_eq!(state.visible_paths().nth(1), Some("/work/Alpha"));

        let mut removed = FilesSnapshot::new();
        removed.insert(FileEntry::directory("/"));
        removed.insert(FileEntry::directory("/work"));
        removed.insert(FileEntry::directory("/work/beta"));
        removed.insert(FileEntry::file("/work/README.md"));
        state.replace_snapshot(removed);
        assert_eq!(state.selected_path(), Some("/work/beta"));
    }
}
