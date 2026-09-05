//! File and repository chooser state.
//!
//! This is the app-side half of the modal decomposition.  Rendering belongs
//! to `junie_tui::Dialog`, `Form`, and `List`; this type keeps the source path,
//! URL-mode toggle, and keyed selection stable while the list is reconciled.

use junie_tui::ItemKey;

/// A source the operator can choose for a workspace mount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileBrowserEntry {
    /// Stable repository-relative or host path.
    pub path: String,
    /// Human-readable kind (`directory`, `repository`, or `file`).
    pub kind: &'static str,
    /// Whether selecting the entry is currently valid.
    pub selectable: bool,
}

impl FileBrowserEntry {
    /// Construct a selectable entry.
    pub(crate) fn new(path: impl Into<String>, kind: &'static str) -> Self {
        Self {
            path: path.into(),
            kind,
            selectable: true,
        }
    }

    /// Construct an entry that is visible but cannot be selected.
    #[must_use]
    pub(crate) const fn disabled(mut self) -> Self {
        self.selectable = false;
        self
    }

    /// Key used by the library `List`/`Picker` collection.
    pub(crate) fn key(&self) -> ItemKey {
        ItemKey::text(&self.path)
    }
}

/// Actions returned by the file-browser composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileBrowserAction {
    /// Accept the selected entry.
    Choose(String),
    /// Switch between local path and URL resolution.
    ToggleUrlMode,
    /// Refresh the source inventory.
    Refresh,
    /// Close without changing the draft.
    Cancel,
}

/// Durable state for one file-browser layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FileBrowserState {
    path: String,
    read_only: bool,
    url_mode: bool,
    resolving: bool,
    error: Option<String>,
    entries: Vec<FileBrowserEntry>,
    selected: Option<ItemKey>,
}

impl FileBrowserState {
    /// Build an empty browser rooted at `path`.
    pub(crate) fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }

    /// Current draft path.
    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    /// Replace the draft path.
    pub(crate) fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    /// Whether the form is read-only.
    pub(crate) const fn read_only(&self) -> bool {
        self.read_only
    }

    /// Mark the browser read-only (for inspection flows).
    pub(crate) const fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Whether the current path is being resolved as a URL.
    pub(crate) const fn url_mode(&self) -> bool {
        self.url_mode
    }

    /// Toggle URL resolution and clear an old resolution error.
    pub(crate) fn toggle_url_mode(&mut self) -> FileBrowserAction {
        self.url_mode = !self.url_mode;
        self.resolving = false;
        self.error = None;
        FileBrowserAction::ToggleUrlMode
    }

    /// Replace visible entries and reconcile the keyed selection.
    ///
    /// A removed item never remains selected.  If the old key disappears,
    /// the first selectable entry is selected; this is the same contract used
    /// by the library keyed collections and avoids stale-index actions.
    pub(crate) fn replace_entries(&mut self, entries: Vec<FileBrowserEntry>) {
        let old = self.selected;
        self.entries = entries;
        self.selected = old
            .filter(|key| self.entry(*key).is_some_and(|entry| entry.selectable))
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|entry| entry.selectable)
                    .map(FileBrowserEntry::key)
            });
    }

    /// Visible entries.
    pub(crate) fn entries(&self) -> &[FileBrowserEntry] {
        &self.entries
    }

    /// Current keyed selection.
    pub(crate) const fn selected(&self) -> Option<ItemKey> {
        self.selected
    }

    /// Select a visible, selectable entry.
    pub(crate) fn select(&mut self, key: ItemKey) -> bool {
        if self.entry(key).is_some_and(|entry| entry.selectable) {
            self.selected = Some(key);
            true
        } else {
            false
        }
    }

    /// Selected path, if it still exists and is selectable.
    pub(crate) fn selected_path(&self) -> Option<&str> {
        self.selected
            .and_then(|key| self.entry(key))
            .filter(|entry| entry.selectable)
            .map(|entry| entry.path.as_str())
    }

    /// Start a deterministic inventory refresh.
    pub(crate) fn begin_refresh(&mut self) {
        self.resolving = true;
        self.error = None;
    }

    /// Finish a refresh with a safe operator-facing result.
    pub(crate) fn finish_refresh(&mut self, error: Option<String>) {
        self.resolving = false;
        self.error = error;
    }

    /// Whether a refresh is pending.
    pub(crate) const fn resolving(&self) -> bool {
        self.resolving
    }

    /// Current non-secret resolution error.
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Return the selected source as an action, unless the form is read-only.
    pub(crate) fn choose(&self) -> Option<FileBrowserAction> {
        (!self.read_only)
            .then(|| self.selected_path().map(str::to_owned))
            .flatten()
            .map(FileBrowserAction::Choose)
    }

    fn entry(&self, key: ItemKey) -> Option<&FileBrowserEntry> {
        self.entries.iter().find(|entry| entry.key() == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_reconciles_after_item_removal() {
        let mut state = FileBrowserState::new("~/src");
        let old = FileBrowserEntry::new("~/src/old", "directory");
        let keep = FileBrowserEntry::new("~/src/keep", "directory");
        state.replace_entries(vec![old.clone(), keep.clone()]);
        assert!(state.select(old.key()));
        state.replace_entries(vec![keep.clone()]);
        assert_eq!(state.selected_path(), Some("~/src/keep"));
    }

    #[test]
    fn disabled_entries_are_visible_but_not_choosable() {
        let mut state = FileBrowserState::new("~/src");
        let entry = FileBrowserEntry::new("~/src/missing", "directory").disabled();
        state.replace_entries(vec![entry.clone()]);
        assert_eq!(state.selected(), None);
        assert!(!state.select(entry.key()));
        assert_eq!(state.choose(), None);
    }
}
