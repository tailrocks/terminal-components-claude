use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

pub const PREVIEW_MAX_BYTES: usize = 256 * 1024;
pub const PREVIEW_MAX_LINES: usize = 2_000;
pub const PREVIEW_MAX_LINE_CHARS: usize = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanMetadata {
    pub modeled_identity: u64,

    pub allocated_bytes: u64,

    pub apparent_bytes: u64,

    pub modified_at_secs: i64,

    pub dataless: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink { target: String },
    Special,
}

#[derive(Clone, PartialEq, Eq)]
pub struct FileEntry {
    path: String,
    kind: FileKind,
    readable: bool,
    content: Option<Arc<[u8]>>,
    scan_metadata: Option<ScanMetadata>,
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
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::File,
            readable: true,
            content: None,
            scan_metadata: None,
        }
    }

    pub fn file_with_bytes(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self::file(path).with_bytes(bytes)
    }

    #[must_use]
    fn with_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.content = Some(Arc::<[u8]>::from(bytes.into()));
        self
    }

    pub fn directory(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Directory,
            readable: true,
            content: None,
            scan_metadata: None,
        }
    }

    pub fn symlink(path: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Symlink {
                target: target.into(),
            },
            readable: true,
            content: None,
            scan_metadata: None,
        }
    }

    pub fn special(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FileKind::Special,
            readable: true,
            content: None,
            scan_metadata: None,
        }
    }

    #[must_use]
    pub fn unreadable(mut self) -> Self {
        self.readable = false;
        self
    }

    #[must_use]
    pub fn with_scan_metadata(mut self, metadata: ScanMetadata) -> Self {
        self.scan_metadata = Some(metadata);
        self
    }

    pub(crate) fn scan_metadata(&self) -> Option<&ScanMetadata> {
        self.scan_metadata.as_ref()
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilesSnapshot {
    entries: BTreeMap<String, FileEntry>,
}

impl FilesSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, entry: FileEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn get(&self, path: &str) -> Option<&FileEntry> {
        self.entries.get(path)
    }

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
