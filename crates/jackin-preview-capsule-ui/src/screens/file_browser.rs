#![doc = r" File and repository chooser state."]
#![doc = r""]
#![doc = r" This is the app-side half of the modal decomposition.  Rendering belongs"]
#![doc = r" to `termrock::Dialog`, `Form`, and `List`; this type keeps the source path,"]
#![doc = r" URL-mode toggle, and keyed selection stable while the list is reconciled."]

use termrock::ItemKey;

#[doc = r" A source the operator can choose for a workspace mount."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBrowserEntry {
    #[doc = r" Stable repository-relative or host path."]
    pub path: String,
    #[doc = r" Human-readable kind (`directory`, `repository`, or `file`)."]
    pub kind: &'static str,
    #[doc = r" Whether selecting the entry is currently valid."]
    pub selectable: bool,
}

impl FileBrowserEntry {
    #[doc = r" Construct a selectable entry."]
    pub fn new(path: impl Into<String>, kind: &'static str) -> Self {
        Self {
            path: path.into(),
            kind,
            selectable: true,
        }
    }

    #[doc = r" Construct an entry that is visible but cannot be selected."]
    #[must_use]
    pub const fn disabled(mut self) -> Self {
        self.selectable = false;
        self
    }

    #[doc = r" Key used by the library `List`/`Picker` collection."]
    pub fn key(&self) -> ItemKey {
        ItemKey::text(&self.path)
    }
}

#[doc = r" Actions returned by the file-browser composition."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileBrowserAction {
    #[doc = r" Accept the selected entry."]
    Choose(String),
    #[doc = r" Switch between local path and URL resolution."]
    ToggleUrlMode,
    #[doc = r" Refresh the source inventory."]
    Refresh,
    #[doc = r" Close without changing the draft."]
    Cancel,
}

#[doc = r" Durable state for one file-browser layer."]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileBrowserState {
    path: String,
    read_only: bool,
    url_mode: bool,
    resolving: bool,
    error: Option<String>,
    entries: Vec<FileBrowserEntry>,
    selected: Option<ItemKey>,
}

impl FileBrowserState {
    #[doc = r" Build an empty browser rooted at `path`."]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }

    #[doc = r" Current draft path."]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[doc = r" Replace the draft path."]
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    #[doc = r" Whether the form is read-only."]
    pub const fn read_only(&self) -> bool {
        self.read_only
    }

    #[doc = r" Mark the browser read-only (for inspection flows)."]
    pub const fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    #[doc = r" Whether the current path is being resolved as a URL."]
    pub const fn url_mode(&self) -> bool {
        self.url_mode
    }

    #[doc = r" Toggle URL resolution and clear an old resolution error."]
    pub fn toggle_url_mode(&mut self) -> FileBrowserAction {
        self.url_mode = !self.url_mode;
        self.resolving = false;
        self.error = None;
        FileBrowserAction::ToggleUrlMode
    }

    #[doc = r" Replace visible entries and reconcile the keyed selection."]
    #[doc = r""]
    #[doc = r" A removed item never remains selected.  If the old key disappears,"]
    #[doc = r" the first selectable entry is selected; this is the same contract used"]
    #[doc = r" by the library keyed collections and avoids stale-index actions."]
    pub fn replace_entries(&mut self, entries: Vec<FileBrowserEntry>) {
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

    #[doc = r" Visible entries."]
    pub fn entries(&self) -> &[FileBrowserEntry] {
        &self.entries
    }

    #[doc = r" Current keyed selection."]
    pub const fn selected(&self) -> Option<ItemKey> {
        self.selected
    }

    #[doc = r" Select a visible, selectable entry."]
    pub fn select(&mut self, key: ItemKey) -> bool {
        if self.entry(key).is_some_and(|entry| entry.selectable) {
            self.selected = Some(key);
            true
        } else {
            false
        }
    }

    #[doc = r" Selected path, if it still exists and is selectable."]
    pub fn selected_path(&self) -> Option<&str> {
        self.selected
            .and_then(|key| self.entry(key))
            .filter(|entry| entry.selectable)
            .map(|entry| entry.path.as_str())
    }

    #[doc = r" Start a deterministic inventory refresh."]
    pub fn begin_refresh(&mut self) {
        self.resolving = true;
        self.error = None;
    }

    #[doc = r" Finish a refresh with a safe operator-facing result."]
    pub fn finish_refresh(&mut self, error: Option<String>) {
        self.resolving = false;
        self.error = error;
    }

    #[doc = r" Whether a refresh is pending."]
    pub const fn resolving(&self) -> bool {
        self.resolving
    }

    #[doc = r" Current non-secret resolution error."]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    #[doc = r" Return the selected source as an action, unless the form is read-only."]
    pub fn choose(&self) -> Option<FileBrowserAction> {
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
