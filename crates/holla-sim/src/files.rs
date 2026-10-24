//! A deterministic, read-only filesystem snapshot for the Holla simulator.
//!
//! Paths in this module are simulated keys. No method reads or changes the host filesystem.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

pub const PREVIEW_MAX_BYTES: usize = 256 * 1024;
pub const PREVIEW_MAX_LINES: usize = 2_000;
pub const PREVIEW_MAX_LINE_CHARS: usize = 4_096;

/// The subset of simulated node kinds needed by the first Files browser stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink { target: String },
    Special,
}

/// One immutable-in-use entry in a [`FilesSnapshot`].
#[derive(Clone, PartialEq, Eq)]
pub struct FileEntry {
    path: String,
    kind: FileKind,
    readable: bool,
    content: Option<Arc<[u8]>>,
}

impl fmt::Debug for FileEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("FileEntry");
        debug
            .field("path", &self.path)
            .field("kind", &self.kind)
            .field("readable", &self.readable);
        if let Some(content) = &self.content {
            debug.field("content_bytes", &content.len());
        }
        debug.finish()
    }
}

impl FileEntry {
    /// Create a readable regular file at a simulated path.
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::File,
            readable: true,
            content: None,
        }
    }

    /// Create a readable regular file with explicitly modeled bytes.
    pub fn file_with_bytes(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self::file(path).with_bytes(bytes)
    }

    /// Attach explicitly modeled bytes to this simulated file.
    #[must_use]
    fn with_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.content = Some(Arc::<[u8]>::from(bytes.into()));
        self
    }

    /// Create a readable directory at a simulated path.
    pub fn directory(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Directory,
            readable: true,
            content: None,
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
            content: None,
        }
    }

    /// Create a special node that the first browser stage does not open.
    pub fn special(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Special,
            readable: true,
            content: None,
        }
    }

    /// Mark this simulated entry as unreadable.
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

/// The reason a modeled file's inspected bytes must not be displayed as text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryPreviewReason {
    NulByte,
    InvalidUtf8,
}

impl fmt::Display for BinaryPreviewReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NulByte => formatter.write_str("NUL byte in the first 256 KiB"),
            Self::InvalidUtf8 => formatter.write_str("invalid UTF-8"),
        }
    }
}

/// Safe content outcomes for one explicitly modeled file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePreviewContent {
    Unavailable,
    Empty,
    Binary {
        reason: BinaryPreviewReason,
    },
    Text {
        lines: Vec<String>,
        bytes_shown: usize,
        total_bytes: usize,
        truncated_bytes: bool,
        truncated_lines: bool,
        long_lines: usize,
    },
}

/// A preview result bound to both the selected path and its resolved target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePreview {
    requested_path: String,
    resolved_path: String,
    link_target: Option<String>,
    content: FilePreviewContent,
}

impl FilePreview {
    pub fn requested_path(&self) -> &str {
        &self.requested_path
    }

    pub fn resolved_path(&self) -> &str {
        &self.resolved_path
    }

    pub fn link_target(&self) -> Option<&str> {
        self.link_target.as_deref()
    }

    pub fn content(&self) -> &FilePreviewContent {
        &self.content
    }
}

/// A typed failure while resolving a simulated file preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePreviewError {
    NotFound(String),
    PermissionDenied(String),
    NotRegularFile(String),
    SpecialFile(String),
    BrokenLink(String),
}

impl fmt::Display for FilePreviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(formatter, "{path}: no such file or directory"),
            Self::PermissionDenied(path) => write!(formatter, "{path}: permission denied"),
            Self::NotRegularFile(path) => write!(formatter, "{path}: not a regular file"),
            Self::SpecialFile(path) => write!(formatter, "{path}: special file · not read"),
            Self::BrokenLink(path) => write!(formatter, "{path}: broken symbolic link"),
        }
    }
}

impl std::error::Error for FilePreviewError {}

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

    /// Resolve and safely inspect one file using only entries in this snapshot.
    pub fn preview(&self, path: &str) -> Result<FilePreview, FilePreviewError> {
        let (entry, link_target) = self.resolve_preview_entry(path)?;
        if !entry.readable {
            return Err(FilePreviewError::PermissionDenied(entry.path.clone()));
        }

        let content = match entry.kind() {
            FileKind::File => match entry.content.as_deref() {
                None => FilePreviewContent::Unavailable,
                Some(bytes) => preview_bytes(bytes),
            },
            FileKind::Directory => {
                return Err(FilePreviewError::NotRegularFile(entry.path.clone()));
            }
            FileKind::Special => {
                return Err(FilePreviewError::SpecialFile(entry.path.clone()));
            }
            FileKind::Symlink { .. } => {
                return Err(FilePreviewError::BrokenLink(path.to_owned()));
            }
        };

        Ok(FilePreview {
            requested_path: path.to_owned(),
            resolved_path: entry.path.clone(),
            link_target,
            content,
        })
    }

    fn resolve_preview_entry(
        &self,
        path: &str,
    ) -> Result<(&FileEntry, Option<String>), FilePreviewError> {
        let mut current = path;
        let mut visited = BTreeSet::new();
        let mut link_target = None;
        loop {
            let Some(entry) = self.entries.get(current) else {
                return Err(if current == path {
                    FilePreviewError::NotFound(path.to_owned())
                } else {
                    FilePreviewError::BrokenLink(path.to_owned())
                });
            };
            let FileKind::Symlink { target } = entry.kind() else {
                return Ok((entry, link_target));
            };
            if !visited.insert(current.to_owned()) {
                return Err(FilePreviewError::BrokenLink(path.to_owned()));
            }
            if link_target.is_none() {
                link_target = Some(target.clone());
            }
            current = target;
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

fn preview_bytes(raw: &[u8]) -> FilePreviewContent {
    if raw.is_empty() {
        return FilePreviewContent::Empty;
    }

    let truncated_bytes = raw.len() > PREVIEW_MAX_BYTES;
    let mut cut = raw.len().min(PREVIEW_MAX_BYTES);
    while cut < raw.len() && cut > 0 && (raw[cut] & 0b1100_0000) == 0b1000_0000 {
        cut -= 1;
    }
    let head = &raw[..cut];
    if head.contains(&0) {
        return FilePreviewContent::Binary {
            reason: BinaryPreviewReason::NulByte,
        };
    }
    let Ok(text) = std::str::from_utf8(head) else {
        return FilePreviewContent::Binary {
            reason: BinaryPreviewReason::InvalidUtf8,
        };
    };

    let mut lines = Vec::new();
    let mut long_lines = 0;
    let mut truncated_lines = false;
    for line in text.split('\n') {
        if lines.len() >= PREVIEW_MAX_LINES {
            truncated_lines = true;
            break;
        }
        let mut line = sanitize_preview_line(line);
        if line.chars().count() > PREVIEW_MAX_LINE_CHARS {
            line = line
                .chars()
                .take(PREVIEW_MAX_LINE_CHARS)
                .collect::<String>()
                + " …";
            long_lines += 1;
        }
        lines.push(line);
    }
    if lines.last().is_some_and(String::is_empty) && text.ends_with('\n') {
        lines.pop();
    }

    FilePreviewContent::Text {
        lines,
        bytes_shown: cut,
        total_bytes: raw.len(),
        truncated_bytes,
        truncated_lines,
        long_lines,
    }
}

fn sanitize_preview_line(line: &str) -> String {
    let mut sanitized = String::with_capacity(line.len());
    for character in line.chars() {
        match character {
            '\t' => sanitized.push_str("    "),
            character if character.is_control() => sanitized.push('�'),
            character => sanitized.push(character),
        }
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::{
        BinaryPreviewReason, FileEntry, FileKind, FilePreviewContent, FilePreviewError, FilesError,
        FilesSnapshot, PREVIEW_MAX_BYTES, PREVIEW_MAX_LINE_CHARS, PREVIEW_MAX_LINES,
    };

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

    #[test]
    fn preview_resolves_file_links_and_keeps_unmodeled_empty_and_error_states_distinct() {
        let mut files = FilesSnapshot::new();
        files.insert(FileEntry::directory("/"));
        files.insert(FileEntry::directory("/work"));
        files.insert(FileEntry::file_with_bytes(
            "/work/plain",
            b"hello\n".to_vec(),
        ));
        files.insert(FileEntry::symlink("/work/link", "/work/plain"));
        files.insert(FileEntry::file_with_bytes("/work/empty", Vec::<u8>::new()));
        files.insert(FileEntry::file("/work/unmodeled"));
        files.insert(FileEntry::file_with_bytes(
            "/work/nul",
            b"safe\0hidden".to_vec(),
        ));
        files.insert(FileEntry::file_with_bytes("/work/invalid", vec![0xff]));
        files.insert(FileEntry::file_with_bytes("/work/locked", b"secret".to_vec()).unreadable());
        files.insert(FileEntry::symlink("/work/broken", "/work/missing"));
        files.insert(FileEntry::symlink("/work/cycle-a", "/work/cycle-b"));
        files.insert(FileEntry::symlink("/work/cycle-b", "/work/cycle-a"));
        files.insert(FileEntry::special("/work/device"));

        let link = files.preview("/work/link");
        assert!(link.is_ok(), "modeled file symlink resolves");
        let Ok(link) = link else { return };
        assert_eq!(link.requested_path(), "/work/link");
        assert_eq!(link.resolved_path(), "/work/plain");
        assert_eq!(link.link_target(), Some("/work/plain"));
        assert!(matches!(
            link.content(),
            FilePreviewContent::Text { lines, .. }
                if lines.len() == 1 && lines[0] == "hello"
        ));
        assert!(matches!(
            files
                .preview("/work/empty")
                .map(|preview| preview.content().clone()),
            Ok(FilePreviewContent::Empty)
        ));
        assert!(matches!(
            files
                .preview("/work/unmodeled")
                .map(|preview| preview.content().clone()),
            Ok(FilePreviewContent::Unavailable)
        ));
        assert!(matches!(
            files
                .preview("/work/nul")
                .map(|preview| preview.content().clone()),
            Ok(FilePreviewContent::Binary {
                reason: BinaryPreviewReason::NulByte
            })
        ));
        assert!(matches!(
            files
                .preview("/work/invalid")
                .map(|preview| preview.content().clone()),
            Ok(FilePreviewContent::Binary {
                reason: BinaryPreviewReason::InvalidUtf8
            })
        ));
        assert_eq!(
            files.preview("/work/locked"),
            Err(FilePreviewError::PermissionDenied("/work/locked".into()))
        );
        assert_eq!(
            files.preview("/work/broken"),
            Err(FilePreviewError::BrokenLink("/work/broken".into()))
        );
        assert_eq!(
            files.preview("/work/cycle-a"),
            Err(FilePreviewError::BrokenLink("/work/cycle-a".into()))
        );
        assert_eq!(
            files.preview("/work/device"),
            Err(FilePreviewError::SpecialFile("/work/device".into()))
        );
        assert_eq!(
            files.preview("/work/missing"),
            Err(FilePreviewError::NotFound("/work/missing".into()))
        );
    }

    #[test]
    fn preview_applies_original_byte_line_and_control_character_limits() {
        let mut files = FilesSnapshot::new();
        files.insert(FileEntry::directory("/"));
        files.insert(FileEntry::directory("/work"));
        files.insert(FileEntry::file_with_bytes(
            "/work/controls",
            b"tab\tand\x1b\x7f\n".to_vec(),
        ));
        files.insert(FileEntry::file_with_bytes(
            "/work/long-line",
            format!("{}\n", "界".repeat(PREVIEW_MAX_LINE_CHARS + 3)).into_bytes(),
        ));
        files.insert(FileEntry::file_with_bytes(
            "/work/many-lines",
            (0..PREVIEW_MAX_LINES + 5)
                .map(|_| "row")
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
        ));
        let mut boundary = vec![b'a'; PREVIEW_MAX_BYTES - 1];
        boundary.extend_from_slice("é".as_bytes());
        files.insert(FileEntry::file_with_bytes("/work/boundary", boundary));

        let controls = files.preview("/work/controls");
        assert!(controls.is_ok(), "modeled control text previews");
        let Ok(controls) = controls else { return };
        assert!(matches!(
            controls.content(),
            FilePreviewContent::Text { lines, .. }
                if lines.len() == 1 && lines[0] == "tab    and��"
        ));
        let long_line = files.preview("/work/long-line");
        assert!(long_line.is_ok(), "modeled long line previews");
        let Ok(long_line) = long_line else { return };
        assert!(matches!(
            long_line.content(),
            FilePreviewContent::Text { lines, long_lines: 1, .. }
                if lines.len() == 1
                    && lines[0].chars().count() == PREVIEW_MAX_LINE_CHARS + 2
                    && lines[0].ends_with(" …")
        ));
        let many_lines = files.preview("/work/many-lines");
        assert!(many_lines.is_ok(), "modeled line list previews");
        let Ok(many_lines) = many_lines else { return };
        assert!(matches!(
            many_lines.content(),
            FilePreviewContent::Text { lines, truncated_lines: true, .. }
                if lines.len() == PREVIEW_MAX_LINES
        ));
        let boundary = files.preview("/work/boundary");
        assert!(
            boundary.is_ok(),
            "valid UTF-8 before the byte boundary previews"
        );
        let Ok(boundary) = boundary else { return };
        assert!(matches!(
            boundary.content(),
            FilePreviewContent::Text {
                bytes_shown,
                total_bytes,
                truncated_bytes: true,
                ..
            } if *bytes_shown == PREVIEW_MAX_BYTES - 1
                && *total_bytes == PREVIEW_MAX_BYTES + 1
        ));
    }
}
