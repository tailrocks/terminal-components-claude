use termrock::ItemKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBrowserEntry {
    pub path: String,
    pub kind: &'static str,
    pub selectable: bool,
}

impl FileBrowserEntry {
    pub fn new(path: impl Into<String>, kind: &'static str) -> Self {
        Self {
            path: path.into(),
            kind,
            selectable: true,
        }
    }

    #[must_use]
    pub const fn disabled(mut self) -> Self {
        self.selectable = false;
        self
    }

    pub fn key(&self) -> ItemKey {
        ItemKey::text(&self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileBrowserAction {
    Choose(String),
    ToggleUrlMode,
    Refresh,
    Cancel,
}

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
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    pub const fn read_only(&self) -> bool {
        self.read_only
    }

    pub const fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub const fn url_mode(&self) -> bool {
        self.url_mode
    }

    pub fn toggle_url_mode(&mut self) -> FileBrowserAction {
        self.url_mode = !self.url_mode;
        self.resolving = false;
        self.error = None;
        FileBrowserAction::ToggleUrlMode
    }

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

    pub fn entries(&self) -> &[FileBrowserEntry] {
        &self.entries
    }

    pub const fn selected(&self) -> Option<ItemKey> {
        self.selected
    }

    pub fn select(&mut self, key: ItemKey) -> bool {
        if self.entry(key).is_some_and(|entry| entry.selectable) {
            self.selected = Some(key);
            true
        } else {
            false
        }
    }

    pub fn selected_path(&self) -> Option<&str> {
        self.selected
            .and_then(|key| self.entry(key))
            .filter(|entry| entry.selectable)
            .map(|entry| entry.path.as_str())
    }

    pub fn begin_refresh(&mut self) {
        self.resolving = true;
        self.error = None;
    }

    pub fn finish_refresh(&mut self, error: Option<String>) {
        self.resolving = false;
        self.error = error;
    }

    pub const fn resolving(&self) -> bool {
        self.resolving
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

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
