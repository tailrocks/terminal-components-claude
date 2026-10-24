//! A deterministic, read-only filesystem snapshot for the Holla simulator.
//!
//! Paths in this module are simulated keys. No method reads or changes the host filesystem.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The subset of simulated node kinds needed by the first Files browser stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink { target: String },
    Special,
}

/// One immutable-in-use entry in a [`FilesSnapshot`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    path: String,
    kind: FileKind,
    readable: bool,
}

impl FileEntry {
    /// Create a readable regular file at a simulated path.
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::File,
            readable: true,
        }
    }

    /// Create a readable directory at a simulated path.
    pub fn directory(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Directory,
            readable: true,
        }
    }

    /// Create a symbolic link whose target is another snapshot key.
    pub fn symlink(path: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Symlink {
                target: target.into(),
            },
            readable: true,
        }
    }

    /// Create a special node that the first browser stage does not open.
    pub fn special(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Special,
            readable: true,
        }
    }

    /// Mark a directory as unreadable in the simulated snapshot.
    #[must_use]
    pub fn unreadable(mut self) -> Self {
        self.readable = false;
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> &FileKind {
        &self.kind
    }

    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.kind, FileKind::Directory)
    }

    pub fn is_hidden(&self) -> bool {
        self.name().starts_with('.')
    }
}

/// A typed failure while listing or opening a simulated directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilesError {
    NotFound(String),
    PermissionDenied(String),
    NotADirectory(String),
    BrokenLink(String),
}

impl fmt::Display for FilesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(formatter, "{path}: no such file or directory"),
            Self::PermissionDenied(path) => write!(formatter, "{path}: permission denied"),
            Self::NotADirectory(path) => write!(formatter, "{path}: not a directory"),
            Self::BrokenLink(path) => write!(formatter, "{path}: broken symbolic link"),
        }
    }
}

impl std::error::Error for FilesError {}

/// A deterministic snapshot of simulated paths and entries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilesSnapshot {
    entries: BTreeMap<String, FileEntry>,
}

impl FilesSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace one simulated path. Parent creation is explicit.
    pub fn insert(&mut self, entry: FileEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn get(&self, path: &str) -> Option<&FileEntry> {
        self.entries.get(path)
    }

    /// List direct children in directory-first, case-insensitive name order.
    pub fn list(&self, directory: &str) -> Result<Vec<FileEntry>, FilesError> {
        let Some(parent) = self.entries.get(directory) else {
            return Err(FilesError::NotFound(directory.to_owned()));
        };
        if !parent.readable {
            return Err(FilesError::PermissionDenied(directory.to_owned()));
        }
        if !parent.is_directory() {
            return Err(FilesError::NotADirectory(directory.to_owned()));
        }

        let prefix = if directory == "/" {
            "/".to_owned()
        } else {
            format!("{directory}/")
        };
        let mut entries: Vec<_> = self
            .entries
            .range(prefix.clone()..)
            .take_while(|(path, _)| path.starts_with(&prefix))
            .filter(|(path, _)| {
                path.strip_prefix(&prefix)
                    .is_some_and(|remainder| !remainder.is_empty() && !remainder.contains('/'))
            })
            .map(|(_, entry)| entry.clone())
            .collect();
        entries.sort_by(|left, right| {
            right
                .is_directory()
                .cmp(&left.is_directory())
                .then_with(|| left.name().to_lowercase().cmp(&right.name().to_lowercase()))
                .then_with(|| left.name().cmp(right.name()))
                .then_with(|| left.path().cmp(right.path()))
        });
        Ok(entries)
    }

    /// Resolve a selected directory or directory symlink to its snapshot key.
    pub fn directory_target(&self, path: &str) -> Result<String, FilesError> {
        let mut current = path;
        let mut visited = BTreeSet::new();
        loop {
            let Some(entry) = self.entries.get(current) else {
                return Err(FilesError::NotFound(current.to_owned()));
            };
            match entry.kind() {
                FileKind::Directory if entry.readable => return Ok(entry.path().to_owned()),
                FileKind::Directory => {
                    return Err(FilesError::PermissionDenied(entry.path().to_owned()));
                }
                FileKind::Symlink { target } => {
                    if !visited.insert(current.to_owned()) {
                        return Err(FilesError::BrokenLink(path.to_owned()));
                    }
                    if !self.entries.contains_key(target) {
                        return Err(FilesError::BrokenLink(entry.path().to_owned()));
                    }
                    current = target;
                }
                FileKind::File | FileKind::Special => {
                    return Err(FilesError::NotADirectory(entry.path().to_owned()));
                }
            }
        }
    }

    pub fn parent(path: &str) -> Option<String> {
        if path == "/" || path == "~" {
            return None;
        }
        match path.rsplit_once('/') {
            Some(("~", _)) => Some("~".to_owned()),
            Some(("", _)) => Some("/".to_owned()),
            Some((parent, _)) if !parent.is_empty() => Some(parent.to_owned()),
            _ => None,
        }
    }

    pub fn join(directory: &str, name: &str) -> String {
        if directory == "/" {
            format!("/{name}")
        } else if directory == "~" {
            format!("~/{name}")
        } else {
            format!("{directory}/{name}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileEntry, FileKind, FilesError, FilesSnapshot};

    fn sample() -> FilesSnapshot {
        let mut files = FilesSnapshot::new();
        files.insert(FileEntry::directory("/"));
        files.insert(FileEntry::directory("/work"));
        files.insert(FileEntry::directory("/work/Alpha"));
        files.insert(FileEntry::directory("/work/beta"));
        files.insert(FileEntry::directory("/work/locked").unreadable());
        files.insert(FileEntry::file("/work/Zulu.txt"));
        files.insert(FileEntry::file("/work/éclair.txt"));
        files.insert(FileEntry::file("/work/.private"));
        files.insert(FileEntry::symlink("/work/broken", "/work/missing"));
        files
    }

    #[test]
    fn listing_is_directory_first_and_stably_orders_unicode_names() {
        let files = sample();
        let root_paths: Vec<_> = files
            .list("/")
            .unwrap_or_default()
            .into_iter()
            .map(|entry| entry.path().to_owned())
            .collect();
        assert_eq!(root_paths, vec!["/work".to_owned()]);

        let entries = files.list("/work").unwrap_or_default();
        let paths: Vec<_> = entries.iter().map(FileEntry::path).collect();
        assert_eq!(
            paths,
            [
                "/work/Alpha",
                "/work/beta",
                "/work/locked",
                "/work/.private",
                "/work/broken",
                "/work/Zulu.txt",
                "/work/éclair.txt",
            ]
        );
        assert!(
            entries
                .iter()
                .find(|entry| entry.path() == "/work/.private")
                .is_some_and(FileEntry::is_hidden)
        );
        assert!(
            entries
                .iter()
                .find(|entry| entry.path() == "/work/Alpha")
                .is_some_and(FileEntry::is_directory)
        );
    }

    #[test]
    fn listing_and_directory_activation_keep_typed_path_errors() {
        let files = sample();
        assert_eq!(
            files.list("/absent"),
            Err(FilesError::NotFound("/absent".into()))
        );
        assert_eq!(
            files.list("/work/Zulu.txt"),
            Err(FilesError::NotADirectory("/work/Zulu.txt".into()))
        );
        assert_eq!(
            files.list("/work/locked"),
            Err(FilesError::PermissionDenied("/work/locked".into()))
        );
        assert_eq!(
            files.directory_target("/work/broken"),
            Err(FilesError::BrokenLink("/work/broken".into()))
        );
        assert_eq!(
            files.directory_target("/work/Zulu.txt"),
            Err(FilesError::NotADirectory("/work/Zulu.txt".into()))
        );
        assert!(matches!(
            FileEntry::special("/work/device").kind(),
            FileKind::Special
        ));
    }

    #[test]
    fn simulated_parent_and_join_keep_tilde_paths_lexical() {
        assert_eq!(FilesSnapshot::parent("~/work/notes"), Some("~/work".into()));
        assert_eq!(FilesSnapshot::parent("~/work"), Some("~".into()));
        assert_eq!(FilesSnapshot::parent("~"), None);
        assert_eq!(FilesSnapshot::join("~/work", "notes"), "~/work/notes");
    }
}
