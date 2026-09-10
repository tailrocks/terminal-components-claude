//! A deterministic in-memory filesystem. Every path Holla lists, previews,
//! measures, indexes or removes is a node here: files, directories,
//! symlinks, FIFOs and devices with apparent and allocated bytes, hardlink
//! identity, modification time, readability and macOS dataless flags. The
//! preview never touches the real filesystem; the operational adapters that
//! replace this model must honour the same contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::clock::EPOCH_SECS;

pub const BLOCK: u64 = 4096;
/// Preview bounds (I-P01).
pub const PREVIEW_MAX_BYTES: usize = 256 * 1024;
pub const PREVIEW_MAX_LINES: usize = 2000;
pub const PREVIEW_MAX_LINE_CHARS: usize = 4096;
/// Directory count bound for a listing preview (I-B12).
pub const COUNT_CAP: usize = 2048;
/// Home index result bound (I-F04).
pub const FIND_MAX_RESULTS: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Dir,
    Symlink { target: String },
    Fifo,
    Device,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    None,
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub path: String,
    pub kind: NodeKind,
    pub apparent: u64,
    pub allocated: u64,
    pub mtime: i64,
    pub readable: bool,
    /// Hardlink identity: two paths with one inode count once.
    pub inode: u64,
    /// macOS dataless (evicted to iCloud): never materialised by a scan.
    pub dataless: bool,
    pub content: Content,
}

impl Node {
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
    pub fn hidden(&self) -> bool {
        self.name().starts_with('.')
    }
    pub fn is_dir(&self) -> bool {
        self.kind == NodeKind::Dir
    }
    pub fn is_file(&self) -> bool {
        self.kind == NodeKind::File
    }
    pub fn kind_word(&self) -> &'static str {
        match self.kind {
            NodeKind::File => "file",
            NodeKind::Dir => "directory",
            NodeKind::Symlink { .. } => "symlink",
            NodeKind::Fifo => "fifo",
            NodeKind::Device => "device",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    NotFound(String),
    PermissionDenied(String),
    NotADirectory(String),
    /// A FIFO or device: refused before any read.
    SpecialFile(String),
    /// A symlink loop or dangling link while resolving.
    BrokenLink(String),
    Dataless(String),
}

impl FsError {
    pub fn message(&self) -> String {
        match self {
            FsError::NotFound(p) => format!("{p}: no such file or directory"),
            FsError::PermissionDenied(p) => format!("{p}: permission denied"),
            FsError::NotADirectory(p) => format!("{p}: not a directory"),
            FsError::SpecialFile(p) => format!("{p}: special file · not read"),
            FsError::BrokenLink(p) => format!("{p}: broken symbolic link"),
            FsError::Dataless(p) => format!("{p}: dataless · not materialised"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trashed {
    pub original: String,
    pub trash_path: String,
    pub bytes: u64,
    pub at_secs: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    pub mount: String,
    pub total: u64,
    pub used: u64,
    /// Purgeable bytes the OS may reclaim: excluded from estimates.
    pub purgeable: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Fs {
    nodes: BTreeMap<String, Node>,
    next_inode: u64,
    pub trash: Vec<Trashed>,
    pub volumes: Vec<Volume>,
    /// The Trash cannot accept items (full, unwritable): every Trash
    /// removal fails visibly instead of falling back to permanent.
    pub trash_broken: Option<String>,
    /// Transient `AlreadyExists` failures left in the Trash mount before a
    /// retry succeeds (LD057: at most three attempts).
    pub trash_collisions: u32,
}

/// Directory-size scan node (LD004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanNode {
    pub path: String,
    pub is_dir: bool,
    pub allocated: u64,
    pub apparent: u64,
    /// Non-directory entries below (including self for a file).
    pub entries: u64,
    pub error: Option<FsError>,
    pub children: Vec<ScanNode>,
    /// Symlink leaf: never traversed.
    pub link: bool,
}

impl ScanNode {
    pub fn errors(&self) -> usize {
        usize::from(self.error.is_some())
            + self.children.iter().map(ScanNode::errors).sum::<usize>()
    }
    pub fn find(&self, path: &str) -> Option<&ScanNode> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanOptions {
    pub include_hidden: bool,
    /// Whole subtrees to omit.
    pub skip: Vec<String>,
    pub max_depth: Option<usize>,
}

/// Bounded, sanitised text preview (I-P01…I-P04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    Text {
        lines: Vec<String>,
        bytes_shown: usize,
        total_bytes: u64,
        truncated_bytes: bool,
        truncated_lines: bool,
        long_lines: usize,
        /// The link target when the path was a symlink.
        link: Option<String>,
    },
    Empty,
    Binary {
        reason: String,
    },
    Error(FsError),
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirCount {
    pub visible: usize,
    pub hidden: usize,
    /// The count stopped at the cap: a lower bound.
    pub capped: bool,
}

/// One home-index hit (I-F04/I-F05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindHit {
    pub path: String,
    pub is_dir: bool,
    /// Byte offsets into the file name that matched.
    pub matched: Vec<usize>,
    pub score: u32,
}

impl Fs {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc(&mut self) -> u64 {
        self.next_inode += 1;
        self.next_inode
    }

    fn ensure_parents(&mut self, path: &str) {
        let mut cur = String::new();
        for seg in path.trim_start_matches('/').split('/') {
            if seg.is_empty() {
                continue;
            }
            let next = format!("{cur}/{seg}");
            if next != path && !self.nodes.contains_key(&next) {
                let inode = self.alloc();
                self.nodes.insert(
                    next.clone(),
                    Node {
                        path: next.clone(),
                        kind: NodeKind::Dir,
                        apparent: 0,
                        allocated: BLOCK,
                        mtime: EPOCH_SECS - 30 * 86_400,
                        readable: true,
                        inode,
                        dataless: false,
                        content: Content::None,
                    },
                );
            }
            cur = next;
        }
        if !self.nodes.contains_key("/") {
            let inode = self.alloc();
            self.nodes.insert(
                "/".into(),
                Node {
                    path: "/".into(),
                    kind: NodeKind::Dir,
                    apparent: 0,
                    allocated: BLOCK,
                    mtime: EPOCH_SECS - 365 * 86_400,
                    readable: true,
                    inode,
                    dataless: false,
                    content: Content::None,
                },
            );
        }
    }

    fn put(&mut self, mut node: Node) {
        self.ensure_parents(&node.path);
        if node.inode == 0 {
            node.inode = self.alloc();
        }
        self.nodes.insert(node.path.clone(), node);
    }

    pub fn dir(&mut self, path: &str, age_days: i64) -> &mut Self {
        self.put(Node {
            path: path.into(),
            kind: NodeKind::Dir,
            apparent: 0,
            allocated: BLOCK,
            mtime: EPOCH_SECS - age_days * 86_400,
            readable: true,
            inode: 0,
            dataless: false,
            content: Content::None,
        });
        self
    }

    /// A file with `bytes` apparent size; allocation rounds up to blocks.
    pub fn file(&mut self, path: &str, bytes: u64, age_days: i64) -> &mut Self {
        self.put(Node {
            path: path.into(),
            kind: NodeKind::File,
            apparent: bytes,
            allocated: bytes.div_ceil(BLOCK) * BLOCK,
            mtime: EPOCH_SECS - age_days * 86_400,
            readable: true,
            inode: 0,
            dataless: false,
            content: Content::None,
        });
        self
    }

    /// A sparse file: apparent bytes larger than what is allocated.
    pub fn sparse(
        &mut self,
        path: &str,
        apparent: u64,
        allocated: u64,
        age_days: i64,
    ) -> &mut Self {
        self.file(path, apparent, age_days);
        if let Some(n) = self.nodes.get_mut(path) {
            n.allocated = allocated;
        }
        self
    }

    pub fn text(&mut self, path: &str, text: &str, age_days: i64) -> &mut Self {
        self.file(path, text.len() as u64, age_days);
        if let Some(n) = self.nodes.get_mut(path) {
            n.content = Content::Text(text.into());
        }
        self
    }

    pub fn bytes(&mut self, path: &str, bytes: Vec<u8>, age_days: i64) -> &mut Self {
        self.file(path, bytes.len() as u64, age_days);
        if let Some(n) = self.nodes.get_mut(path) {
            n.content = Content::Bytes(bytes);
        }
        self
    }

    pub fn symlink(&mut self, path: &str, target: &str) -> &mut Self {
        self.put(Node {
            path: path.into(),
            kind: NodeKind::Symlink {
                target: target.into(),
            },
            // a link owns no data: neither size counts toward a tree
            apparent: 0,
            allocated: 0,
            mtime: EPOCH_SECS - 86_400,
            readable: true,
            inode: 0,
            dataless: false,
            content: Content::None,
        });
        self
    }

    pub fn fifo(&mut self, path: &str) -> &mut Self {
        self.put(Node {
            path: path.into(),
            kind: NodeKind::Fifo,
            apparent: 0,
            allocated: 0,
            mtime: EPOCH_SECS,
            readable: true,
            inode: 0,
            dataless: false,
            content: Content::None,
        });
        self
    }

    pub fn device(&mut self, path: &str) -> &mut Self {
        self.put(Node {
            path: path.into(),
            kind: NodeKind::Device,
            apparent: 0,
            allocated: 0,
            mtime: EPOCH_SECS,
            readable: true,
            inode: 0,
            dataless: false,
            content: Content::None,
        });
        self
    }

    /// A second name for an existing file: same inode, counted once.
    pub fn hardlink(&mut self, path: &str, of: &str) -> &mut Self {
        if let Some(src) = self.nodes.get(of).cloned() {
            let mut n = src;
            n.path = path.into();
            self.ensure_parents(path);
            self.nodes.insert(path.into(), n);
        }
        self
    }

    pub fn unreadable(&mut self, path: &str) -> &mut Self {
        if let Some(n) = self.nodes.get_mut(path) {
            n.readable = false;
        }
        self
    }

    pub fn dataless(&mut self, path: &str) -> &mut Self {
        if let Some(n) = self.nodes.get_mut(path) {
            n.dataless = true;
        }
        self
    }

    pub fn volume(&mut self, mount: &str, total: u64, used: u64) -> &mut Self {
        self.volumes.push(Volume {
            mount: mount.into(),
            total,
            used,
            purgeable: 0,
        });
        self
    }

    pub fn touch(&mut self, path: &str, age_days: i64) {
        if let Some(n) = self.nodes.get_mut(path) {
            n.mtime = EPOCH_SECS - age_days * 86_400;
        }
    }

    // ------------------------------------------------------------ queries

    pub fn get(&self, path: &str) -> Option<&Node> {
        self.nodes.get(path)
    }

    pub fn exists(&self, path: &str) -> bool {
        self.nodes.contains_key(path)
    }

    pub fn is_dir(&self, path: &str) -> bool {
        self.nodes.get(path).is_some_and(Node::is_dir)
    }

    pub fn is_file(&self, path: &str) -> bool {
        self.nodes.get(path).is_some_and(Node::is_file)
    }

    pub fn parent(path: &str) -> Option<String> {
        if path == "/" {
            return None;
        }
        match path.rsplit_once('/') {
            Some(("", _)) => Some("/".into()),
            Some((p, _)) => Some(p.to_owned()),
            None => None,
        }
    }

    pub fn join(dir: &str, name: &str) -> String {
        if dir == "/" {
            format!("/{name}")
        } else {
            format!("{dir}/{name}")
        }
    }

    /// Direct children of a directory, directories first, then names.
    pub fn list(&self, dir: &str) -> Result<Vec<&Node>, FsError> {
        let d = self
            .nodes
            .get(dir)
            .ok_or_else(|| FsError::NotFound(dir.into()))?;
        if !d.readable {
            return Err(FsError::PermissionDenied(dir.into()));
        }
        if !d.is_dir() {
            return Err(FsError::NotADirectory(dir.into()));
        }
        let prefix = if dir == "/" {
            "/".to_owned()
        } else {
            format!("{dir}/")
        };
        let mut v: Vec<&Node> = self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(p, _)| p.starts_with(&prefix))
            .filter(|(p, _)| !p[prefix.len()..].contains('/'))
            .map(|(_, n)| n)
            .collect();
        v.sort_by(|a, b| {
            b.is_dir()
                .cmp(&a.is_dir())
                .then_with(|| a.name().to_lowercase().cmp(&b.name().to_lowercase()))
                .then_with(|| a.name().cmp(b.name()))
        });
        Ok(v)
    }

    /// Every node below `root` (not the root itself), in path order.
    pub fn subtree(&self, root: &str) -> Vec<&Node> {
        let prefix = if root == "/" {
            "/".to_owned()
        } else {
            format!("{root}/")
        };
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(p, _)| p.starts_with(&prefix))
            .map(|(_, n)| n)
            .collect()
    }

    /// Count a directory's entries with the listing-preview bound (I-B12).
    pub fn count(&self, dir: &str) -> Result<DirCount, FsError> {
        let all = self.list(dir)?;
        let mut c = DirCount {
            visible: 0,
            hidden: 0,
            capped: false,
        };
        for n in all {
            if c.visible + c.hidden >= COUNT_CAP {
                c.capped = true;
                break;
            }
            if n.hidden() {
                c.hidden += 1;
            } else {
                c.visible += 1;
            }
        }
        Ok(c)
    }

    /// Resolve a path's ancestors through symlinks. The final component is
    /// preserved unresolved (a selected link removes the link only).
    /// Ancestors that are symlinks are reported so a deletion policy can
    /// refuse them.
    pub fn canonical_parent(&self, path: &str) -> Result<(String, Vec<String>), FsError> {
        let Some(parent) = Self::parent(path) else {
            return Ok(("/".into(), vec![]));
        };
        let mut resolved = String::new();
        let mut links = vec![];
        for seg in parent.trim_start_matches('/').split('/') {
            if seg.is_empty() {
                continue;
            }
            let next = format!("{resolved}/{seg}");
            match self.nodes.get(&next) {
                Some(Node {
                    kind: NodeKind::Symlink { target },
                    ..
                }) => {
                    links.push(next.clone());
                    if !self.nodes.contains_key(target) {
                        return Err(FsError::BrokenLink(next));
                    }
                    resolved = target.clone();
                }
                Some(_) => resolved = next,
                None => return Err(FsError::NotFound(next)),
            }
        }
        if resolved.is_empty() {
            resolved = "/".into();
        }
        Ok((resolved, links))
    }

    /// Follow a symlink chain to its final target.
    pub fn resolve(&self, path: &str) -> Result<&Node, FsError> {
        let mut cur = path.to_owned();
        for _ in 0..16 {
            match self.nodes.get(&cur) {
                None => return Err(FsError::BrokenLink(path.into())),
                Some(Node {
                    kind: NodeKind::Symlink { target },
                    ..
                }) => cur = target.clone(),
                Some(n) => return Ok(n),
            }
        }
        Err(FsError::BrokenLink(path.into()))
    }

    // ------------------------------------------------------------ preview

    /// Bounded, sanitised preview. Symlinks follow their target and disclose
    /// it; FIFOs and devices are refused by the opened descriptor's kind, not
    /// by pre-open metadata; NUL or invalid UTF-8 classifies binary.
    pub fn preview(&self, path: &str) -> Preview {
        let mut link = None;
        let node = match self.nodes.get(path) {
            None => return Preview::Error(FsError::NotFound(path.into())),
            Some(Node {
                kind: NodeKind::Symlink { target },
                ..
            }) => {
                link = Some(target.clone());
                match self.resolve(path) {
                    Ok(n) => n,
                    Err(e) => return Preview::Error(e),
                }
            }
            Some(n) => n,
        };
        // the opened descriptor is rechecked: a file swapped for a FIFO or
        // device between stat and open is never read
        match node.kind {
            NodeKind::Fifo | NodeKind::Device => {
                return Preview::Error(FsError::SpecialFile(path.into()));
            }
            NodeKind::Dir => return Preview::Directory,
            _ => {}
        }
        if !node.readable {
            return Preview::Error(FsError::PermissionDenied(path.into()));
        }
        if node.dataless {
            return Preview::Error(FsError::Dataless(path.into()));
        }
        let raw: Vec<u8> = match &node.content {
            Content::Text(t) => t.as_bytes().to_vec(),
            Content::Bytes(b) => b.clone(),
            Content::None => {
                if node.apparent == 0 {
                    return Preview::Empty;
                }
                return Preview::Binary {
                    reason: "content not modelled · treated as binary".into(),
                };
            }
        };
        if raw.is_empty() {
            return Preview::Empty;
        }
        let total = raw.len() as u64;
        let truncated_bytes = raw.len() > PREVIEW_MAX_BYTES;
        let mut cut = raw.len().min(PREVIEW_MAX_BYTES);
        // keep the valid prefix when the byte cap splits a code point
        while cut < raw.len() && cut > 0 && (raw[cut] & 0b1100_0000) == 0b1000_0000 {
            cut -= 1;
        }
        let head = &raw[..cut];
        if head.contains(&0) {
            return Preview::Binary {
                reason: "NUL byte in the first 256 KiB".into(),
            };
        }
        let text = match std::str::from_utf8(head) {
            Ok(t) => t,
            Err(_) => {
                return Preview::Binary {
                    reason: "invalid UTF-8".into(),
                };
            }
        };
        let mut lines = vec![];
        let mut long_lines = 0;
        let mut truncated_lines = false;
        for l in text.split('\n') {
            if lines.len() >= PREVIEW_MAX_LINES {
                truncated_lines = true;
                break;
            }
            let mut s = sanitize(l);
            if s.chars().count() > PREVIEW_MAX_LINE_CHARS {
                s = s.chars().take(PREVIEW_MAX_LINE_CHARS).collect::<String>() + " …";
                long_lines += 1;
            }
            lines.push(s);
        }
        if lines.last().is_some_and(|l| l.is_empty()) && text.ends_with('\n') {
            lines.pop();
        }
        Preview::Text {
            lines,
            bytes_shown: cut,
            total_bytes: total,
            truncated_bytes,
            truncated_lines,
            long_lines,
            link,
        }
    }

    // ------------------------------------------------------------ sizes

    /// Recursive allocated/apparent totals with hardlink deduplication,
    /// symlink leaves, hidden policy and skip roots (LD004–LD006).
    pub fn scan(&self, root: &str, opts: &ScanOptions) -> ScanNode {
        let mut seen = BTreeSet::new();
        self.scan_node(root, opts, 0, &mut seen)
    }

    fn scan_node(
        &self,
        path: &str,
        opts: &ScanOptions,
        depth: usize,
        seen: &mut BTreeSet<u64>,
    ) -> ScanNode {
        let Some(node) = self.nodes.get(path) else {
            return ScanNode {
                path: path.into(),
                is_dir: false,
                allocated: 0,
                apparent: 0,
                entries: 0,
                error: Some(FsError::NotFound(path.into())),
                children: vec![],
                link: false,
            };
        };
        match &node.kind {
            NodeKind::Symlink { .. } => {
                return ScanNode {
                    path: path.into(),
                    is_dir: false,
                    allocated: node.allocated,
                    apparent: node.apparent,
                    entries: 1,
                    error: None,
                    children: vec![],
                    link: true,
                };
            }
            NodeKind::Dir => {}
            _ => {
                let dup = !seen.insert(node.inode);
                return ScanNode {
                    path: path.into(),
                    is_dir: false,
                    allocated: if dup { 0 } else { node.allocated },
                    apparent: if dup { 0 } else { node.apparent },
                    entries: 1,
                    error: None,
                    children: vec![],
                    link: false,
                };
            }
        }
        if node.dataless {
            return ScanNode {
                path: path.into(),
                is_dir: true,
                allocated: node.allocated,
                apparent: 0,
                entries: 0,
                error: Some(FsError::Dataless(path.into())),
                children: vec![],
                link: false,
            };
        }
        let mut out = ScanNode {
            path: path.into(),
            is_dir: true,
            allocated: node.allocated,
            apparent: node.apparent,
            entries: 0,
            error: None,
            children: vec![],
            link: false,
        };
        if !node.readable {
            out.error = Some(FsError::PermissionDenied(path.into()));
            return out;
        }
        if opts.max_depth.is_some_and(|d| depth >= d) {
            // below the depth bound the totals are still exact: sum without
            // materialising children
            for n in self.subtree(path) {
                if matches!(n.kind, NodeKind::Symlink { .. }) {
                    out.entries += 1;
                    continue;
                }
                if opts.skip.iter().any(|s| n.path.starts_with(s)) {
                    continue;
                }
                if !opts.include_hidden && n.hidden() {
                    continue;
                }
                if n.is_dir() {
                    out.allocated = out.allocated.saturating_add(n.allocated);
                    continue;
                }
                if seen.insert(n.inode) {
                    out.allocated = out.allocated.saturating_add(n.allocated);
                    out.apparent = out.apparent.saturating_add(n.apparent);
                }
                out.entries += 1;
            }
            return out;
        }
        let Ok(children) = self.list(path) else {
            out.error = Some(FsError::PermissionDenied(path.into()));
            return out;
        };
        for c in children {
            if opts
                .skip
                .iter()
                .any(|s| c.path == *s || c.path.starts_with(&format!("{s}/")))
            {
                continue;
            }
            if !opts.include_hidden && c.hidden() {
                continue;
            }
            let child = self.scan_node(&c.path, opts, depth + 1, seen);
            out.allocated = out.allocated.saturating_add(child.allocated);
            out.apparent = out.apparent.saturating_add(child.apparent);
            out.entries = out.entries.saturating_add(child.entries);
            out.children.push(child);
        }
        out.children.sort_by(|a, b| {
            b.allocated
                .cmp(&a.allocated)
                .then_with(|| a.path.cmp(&b.path))
        });
        out
    }

    /// Allocated bytes below a path without following links.
    pub fn size_of(&self, path: &str) -> u64 {
        self.scan(
            path,
            &ScanOptions {
                include_hidden: true,
                ..Default::default()
            },
        )
        .allocated
    }

    pub fn volume_for(&self, path: &str) -> Option<&Volume> {
        self.volumes
            .iter()
            .filter(|v| {
                path == v.mount
                    || path.starts_with(&format!("{}/", v.mount.trim_end_matches('/')))
                    || v.mount == "/"
            })
            .max_by_key(|v| v.mount.len())
    }

    // ------------------------------------------------------------ removal

    /// Move a path to the Trash: the entry leaves its place but the volume's
    /// used bytes stay until the Trash is emptied (LD016). Transient
    /// AlreadyExists collisions retry at most three times; every other
    /// failure is final and never falls back to permanent deletion.
    pub fn trash(&mut self, path: &str, now_secs: i64) -> Result<u64, String> {
        if let Some(reason) = &self.trash_broken {
            return Err(format!("Trash unavailable: {reason}"));
        }
        let mut attempts = 0;
        while self.trash_collisions > 0 {
            self.trash_collisions -= 1;
            attempts += 1;
            if attempts >= 3 {
                return Err("Trash: AlreadyExists after 3 attempts".into());
            }
        }
        let bytes = self.detach(path)?;
        let name = path.rsplit('/').next().unwrap_or("item").to_owned();
        let trash_path = format!("~/.Trash/{name}");
        self.trash.push(Trashed {
            original: path.into(),
            trash_path,
            bytes,
            at_secs: now_secs,
        });
        Ok(bytes)
    }

    /// Remove a path permanently: files and links by unlink, directories
    /// recursively. Volume used bytes drop by the allocated size.
    pub fn remove_permanent(&mut self, path: &str) -> Result<u64, String> {
        let bytes = self.detach(path)?;
        if let Some(v) = self.volume_mut_for(path) {
            v.used = v.used.saturating_sub(bytes);
        }
        Ok(bytes)
    }

    fn volume_mut_for(&mut self, path: &str) -> Option<&mut Volume> {
        let mount = self.volume_for(path).map(|v| v.mount.clone())?;
        self.volumes.iter_mut().find(|v| v.mount == mount)
    }

    /// Detach a node (and its subtree for a directory) from the tree.
    /// Returns the allocated bytes that left. A symlink removes the link
    /// only; its target is untouched.
    fn detach(&mut self, path: &str) -> Result<u64, String> {
        let node = self
            .nodes
            .get(path)
            .cloned()
            .ok_or_else(|| FsError::NotFound(path.into()).message())?;
        if let Some(parent) = Self::parent(path)
            && self.nodes.get(&parent).is_some_and(|p| !p.readable)
        {
            return Err(FsError::PermissionDenied(parent).message());
        }
        match node.kind {
            NodeKind::Symlink { .. } => {
                self.nodes.remove(path);
                Ok(0)
            }
            NodeKind::Dir => {
                if !node.readable {
                    return Err(FsError::PermissionDenied(path.into()).message());
                }
                let bytes = self.size_of(path);
                let doomed: Vec<String> =
                    self.subtree(path).iter().map(|n| n.path.clone()).collect();
                for p in doomed {
                    self.nodes.remove(&p);
                }
                self.nodes.remove(path);
                Ok(bytes)
            }
            _ => {
                self.nodes.remove(path);
                Ok(node.allocated)
            }
        }
    }

    /// Empty the Trash: the volume's used bytes drop now. Holla never
    /// empties the Trash itself; the fixture proves the accounting.
    #[cfg(test)]
    pub fn empty_trash(&mut self) -> u64 {
        let total: u64 = self.trash.iter().map(|t| t.bytes).sum();
        let items: Vec<(String, u64)> = self
            .trash
            .drain(..)
            .map(|t| (t.original, t.bytes))
            .collect();
        for (p, b) in items {
            if let Some(v) = self.volume_mut_for(&p) {
                v.used = v.used.saturating_sub(b);
            }
        }
        total
    }

    // ------------------------------------------------------------ index

    /// Index the home directory the way the old finder did: hidden entries
    /// and `.ignore`d names skipped, symlinks never followed, cloud roots
    /// (`Library/Mobile Documents*`) omitted, no filesystem-root scan.
    pub fn home_index(&self, home: &str) -> Vec<String> {
        let mut out = vec![];
        let ignores = self.ignore_names(home);
        self.index_dir(home, home, &ignores, &mut out, 0);
        out
    }

    fn ignore_names(&self, home: &str) -> BTreeSet<String> {
        let mut set = BTreeSet::new();
        if let Some(Node {
            content: Content::Text(t),
            ..
        }) = self.nodes.get(&format!("{home}/.ignore"))
        {
            for l in t.lines() {
                let l = l.trim();
                if !l.is_empty() && !l.starts_with('#') {
                    set.insert(l.trim_end_matches('/').to_owned());
                }
            }
        }
        set
    }

    fn index_dir(
        &self,
        home: &str,
        dir: &str,
        ignores: &BTreeSet<String>,
        out: &mut Vec<String>,
        depth: usize,
    ) {
        let Ok(children) = self.list(dir) else {
            return;
        };
        for c in children {
            if c.hidden() || ignores.contains(c.name()) {
                continue;
            }
            if dir == home && c.name() == "Library" {
                // Library is split so cloud roots are omitted
                if let Ok(lib) = self.list(&c.path) {
                    for l in lib {
                        if l.name().starts_with("Mobile Documents") || l.hidden() {
                            continue;
                        }
                        out.push(l.path.clone());
                        if l.is_dir() && depth < 12 {
                            self.index_dir(home, &l.path, ignores, out, depth + 1);
                        }
                    }
                }
                continue;
            }
            match c.kind {
                NodeKind::Symlink { .. } => {
                    out.push(c.path.clone());
                }
                NodeKind::Dir => {
                    out.push(c.path.clone());
                    if depth < 12 {
                        self.index_dir(home, &c.path, ignores, out, depth + 1);
                    }
                }
                _ => out.push(c.path.clone()),
            }
        }
    }

    /// Rank indexed paths for a query: exact file name or stem, then name
    /// substring, then fuzzy, then path; at most 100 results. Match offsets
    /// are byte offsets into the file name (I-F04/I-F05).
    pub fn find(&self, index: &[String], query: &str) -> Vec<FindHit> {
        let q = query.trim();
        if q.is_empty() {
            return vec![];
        }
        let lq = q.to_lowercase();
        let mut hits: Vec<FindHit> = vec![];
        for p in index {
            let name = p.rsplit('/').next().unwrap_or(p);
            let lname = name.to_lowercase();
            let stem = lname.rsplit_once('.').map(|(s, _)| s).unwrap_or(&lname);
            let (score, matched) = if lname == lq || stem == lq {
                (
                    0,
                    junie_tui::ui::text::fuzzy(name, q)
                        .map(|(_, m)| m)
                        .unwrap_or_default(),
                )
            } else if let Some(i) = lname.find(&lq) {
                (
                    10,
                    junie_tui::ui::text::fuzzy(name, q)
                        .map(|(_, m)| m)
                        .unwrap_or_else(|| (i..i + lq.len()).collect()),
                )
            } else if let Some((s, m)) = junie_tui::ui::text::fuzzy(name, q) {
                (50 + s, m)
            } else {
                continue;
            };
            hits.push(FindHit {
                path: p.clone(),
                is_dir: self.is_dir(p),
                matched,
                score,
            });
        }
        hits.sort_by(|a, b| a.score.cmp(&b.score).then_with(|| a.path.cmp(&b.path)));
        hits.truncate(FIND_MAX_RESULTS);
        hits
    }
}

/// Replace every control byte so a preview cannot drive the terminal
/// (I-P03); tabs become four spaces.
pub fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\t' => out.push_str("    "),
            c if c.is_control() => out.push('�'),
            c => out.push(c),
        }
    }
    out
}

/// Human byte size in binary units.
pub fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> Fs {
        let mut fs = Fs::new();
        fs.volume("/", 900 * 1024 * 1024 * 1024, 600 * 1024 * 1024 * 1024);
        fs.dir("/Users/alex", 400);
        fs.file("/Users/alex/work/a/big.bin", 10 * BLOCK, 40);
        fs.file("/Users/alex/work/a/small.txt", 100, 2);
        fs.hardlink(
            "/Users/alex/work/a/big-link.bin",
            "/Users/alex/work/a/big.bin",
        );
        fs.symlink("/Users/alex/work/a/to-b", "/Users/alex/work/b");
        fs.file("/Users/alex/work/b/other.txt", 3 * BLOCK, 5);
        fs.dir("/Users/alex/work/.hidden", 1);
        fs.file("/Users/alex/work/.hidden/x", BLOCK, 1);
        fs.sparse("/Users/alex/work/sparse.img", 100 * BLOCK, 2 * BLOCK, 1);
        fs.text(
            "/Users/alex/notes.md",
            "# notes\nhello\tworld\n\u{1b}[31mred\n",
            3,
        );
        fs.bytes("/Users/alex/blob.dat", vec![1, 2, 0, 4], 3);
        fs.fifo("/Users/alex/pipe");
        fs.dir("/Users/alex/locked", 9);
        fs.unreadable("/Users/alex/locked");
        fs
    }

    #[test]
    fn listing_sorts_directories_first_and_counts_hidden() {
        let fs = home();
        let names: Vec<&str> = fs
            .list("/Users/alex/work")
            .unwrap()
            .iter()
            .map(|n| n.name())
            .collect();
        assert_eq!(names, vec![".hidden", "a", "b", "sparse.img"]);
        let c = fs.count("/Users/alex/work").unwrap();
        assert_eq!((c.visible, c.hidden, c.capped), (3, 1, false));
        assert_eq!(
            fs.list("/Users/alex/locked"),
            Err(FsError::PermissionDenied("/Users/alex/locked".into()))
        );
        assert_eq!(
            fs.list("/Users/alex/notes.md"),
            Err(FsError::NotADirectory("/Users/alex/notes.md".into()))
        );
        assert_eq!(fs.list("/nope"), Err(FsError::NotFound("/nope".into())));
        assert_eq!(Fs::parent("/Users/alex"), Some("/Users".into()));
        assert_eq!(Fs::parent("/x"), Some("/".into()));
        assert_eq!(Fs::parent("/"), None);
    }

    #[test]
    fn scan_deduplicates_hardlinks_and_never_follows_links() {
        let fs = home();
        let s = fs.scan("/Users/alex/work", &ScanOptions::default());
        // a: big 10 blocks counted once (hardlink), small 1 block, dir 1 block, link 0
        let a = s.find("/Users/alex/work/a").unwrap();
        assert_eq!(a.allocated, BLOCK * (10 + 1 + 1));
        assert_eq!(a.apparent, 10 * BLOCK + 100);
        assert_eq!(
            a.entries, 4,
            "both hardlink names and the symlink are entries"
        );
        assert!(
            a.children
                .iter()
                .any(|c| c.link && c.path.ends_with("to-b"))
        );
        assert!(a.children.iter().all(|c| c.children.is_empty()));
        let hidden = fs.scan(
            "/Users/alex/work",
            &ScanOptions {
                include_hidden: true,
                ..Default::default()
            },
        );
        assert!(hidden.allocated > s.allocated);
        assert!(s.find("/Users/alex/work/.hidden").is_none());
        // sparse: apparent 100 blocks, allocated 2
        let sp = hidden.find("/Users/alex/work/sparse.img").unwrap();
        assert_eq!((sp.apparent, sp.allocated), (100 * BLOCK, 2 * BLOCK));
        // skip roots and depth bounds keep exact totals
        let skipped = fs.scan(
            "/Users/alex/work",
            &ScanOptions {
                include_hidden: true,
                skip: vec!["/Users/alex/work/a".into()],
                max_depth: None,
            },
        );
        assert!(skipped.find("/Users/alex/work/a").is_none());
        assert_eq!(skipped.allocated, hidden.allocated - a.allocated);
        let shallow = fs.scan(
            "/Users/alex/work",
            &ScanOptions {
                include_hidden: true,
                skip: vec![],
                max_depth: Some(1),
            },
        );
        assert_eq!(shallow.allocated, hidden.allocated);
        assert!(
            shallow
                .find("/Users/alex/work/a")
                .unwrap()
                .children
                .is_empty()
        );
        // errors stay visible and partial accounting proceeds
        let e = fs.scan("/Users/alex", &ScanOptions::default());
        assert_eq!(e.errors(), 1);
        assert!(e.find("/Users/alex/locked").unwrap().error.is_some());
        assert!(e.allocated > 0);
    }

    #[test]
    fn preview_is_bounded_sanitised_and_refuses_special_files() {
        let fs = home();
        match fs.preview("/Users/alex/notes.md") {
            Preview::Text { lines, link, .. } => {
                assert_eq!(lines, vec!["# notes", "hello    world", "�[31mred"]);
                assert!(link.is_none());
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            fs.preview("/Users/alex/blob.dat"),
            Preview::Binary { .. }
        ));
        assert_eq!(
            fs.preview("/Users/alex/pipe"),
            Preview::Error(FsError::SpecialFile("/Users/alex/pipe".into()))
        );
        assert_eq!(fs.preview("/Users/alex/work"), Preview::Directory);
        assert!(matches!(
            fs.preview("/Users/alex/missing"),
            Preview::Error(FsError::NotFound(_))
        ));
        assert!(matches!(
            fs.preview("/Users/alex/work/a/to-b"),
            Preview::Directory
        ));
        let mut f2 = home();
        f2.symlink("/Users/alex/link.md", "/Users/alex/notes.md");
        assert!(matches!(
            f2.preview("/Users/alex/link.md"),
            Preview::Text { link: Some(_), .. }
        ));
        f2.symlink("/Users/alex/dangling", "/nowhere");
        assert!(matches!(
            f2.preview("/Users/alex/dangling"),
            Preview::Error(FsError::BrokenLink(_))
        ));
        // byte cap keeps a valid UTF-8 prefix; line and length caps annotate
        let mut big = String::new();
        for _ in 0..(PREVIEW_MAX_BYTES / 2) {
            big.push('é');
        }
        big.push('\n');
        f2.text("/Users/alex/big.txt", &big, 1);
        match f2.preview("/Users/alex/big.txt") {
            Preview::Text {
                bytes_shown,
                truncated_bytes,
                long_lines,
                ..
            } => {
                assert!(truncated_bytes);
                assert!(bytes_shown <= PREVIEW_MAX_BYTES);
                assert_eq!(bytes_shown % 2, 0, "no split code point");
                assert_eq!(long_lines, 1);
            }
            other => panic!("{other:?}"),
        }
        let many: String = (0..PREVIEW_MAX_LINES + 5)
            .map(|i| format!("l{i}\n"))
            .collect();
        f2.text("/Users/alex/many.txt", &many, 1);
        assert!(matches!(
            f2.preview("/Users/alex/many.txt"),
            Preview::Text {
                truncated_lines: true,
                ..
            }
        ));
        f2.text("/Users/alex/empty.txt", "", 1);
        assert_eq!(f2.preview("/Users/alex/empty.txt"), Preview::Empty);
        f2.bytes("/Users/alex/bad.txt", vec![0x66, 0xff, 0x6f], 1);
        assert!(matches!(
            f2.preview("/Users/alex/bad.txt"),
            Preview::Binary { .. }
        ));
    }

    #[test]
    fn trash_keeps_used_bytes_and_permanent_frees_them() {
        let mut fs = home();
        let used = fs.volume_for("/Users/alex").unwrap().used;
        let bytes = fs.trash("/Users/alex/work/a", EPOCH_SECS).unwrap();
        assert_eq!(bytes, BLOCK * 12);
        assert!(!fs.exists("/Users/alex/work/a/big.bin"));
        assert_eq!(
            fs.volume_for("/").unwrap().used,
            used,
            "same-volume Trash frees nothing yet"
        );
        assert_eq!(fs.trash.len(), 1);
        assert_eq!(fs.empty_trash(), bytes);
        assert_eq!(fs.volume_for("/").unwrap().used, used - bytes);
        let mut fs = home();
        let b = fs.remove_permanent("/Users/alex/work/b/other.txt").unwrap();
        assert_eq!(b, 3 * BLOCK);
        assert_eq!(fs.volume_for("/").unwrap().used, used - b);
        // a selected link removes the link only
        assert!(fs.exists("/Users/alex/work/a/to-b"));
        fs.remove_permanent("/Users/alex/work/a/to-b").unwrap();
        assert!(!fs.exists("/Users/alex/work/a/to-b"));
        assert!(fs.exists("/Users/alex/work/b"));
        assert!(
            fs.remove_permanent("/gone")
                .unwrap_err()
                .contains("no such file")
        );
        assert!(
            fs.remove_permanent("/Users/alex/locked")
                .unwrap_err()
                .contains("permission denied")
        );
        // Trash failures are final: no permanent fallback
        fs.trash_broken = Some("~/.Trash is not writable".into());
        assert!(
            fs.trash("/Users/alex/notes.md", 0)
                .unwrap_err()
                .contains("Trash unavailable")
        );
        assert!(fs.exists("/Users/alex/notes.md"));
        fs.trash_broken = None;
        fs.trash_collisions = 2;
        assert!(
            fs.trash("/Users/alex/notes.md", 0).is_ok(),
            "two collisions retry"
        );
        fs.trash_collisions = 5;
        assert!(
            fs.trash("/Users/alex/blob.dat", 0)
                .unwrap_err()
                .contains("3 attempts")
        );
        assert!(fs.exists("/Users/alex/blob.dat"));
    }

    #[test]
    fn canonical_parent_reports_link_ancestors_and_keeps_the_leaf() {
        let fs = home();
        let (p, links) = fs
            .canonical_parent("/Users/alex/work/a/to-b/other.txt")
            .unwrap();
        assert_eq!(p, "/Users/alex/work/b");
        assert_eq!(links, vec!["/Users/alex/work/a/to-b"]);
        let (p, links) = fs.canonical_parent("/Users/alex/work/a/to-b").unwrap();
        assert_eq!(p, "/Users/alex/work/a");
        assert!(links.is_empty(), "the leaf link itself is not resolved");
        assert!(matches!(
            fs.canonical_parent("/nope/x"),
            Err(FsError::NotFound(_))
        ));
    }

    #[test]
    fn home_index_and_find_follow_the_legacy_policy() {
        let mut fs = home();
        fs.text(
            "/Users/alex/.ignore",
            "node_modules\n# comment\nscratch/\n",
            1,
        );
        fs.file("/Users/alex/work/a/node_modules/x.js", 10, 1);
        fs.file("/Users/alex/scratch/y", 10, 1);
        fs.file(
            "/Users/alex/Library/Mobile Documents/com~apple~CloudDocs/doc.txt",
            10,
            1,
        );
        fs.file("/Users/alex/Library/Caches/c.bin", 10, 1);
        let idx = fs.home_index("/Users/alex");
        assert!(idx.iter().any(|p| p.ends_with("small.txt")));
        assert!(!idx.iter().any(|p| p.contains("node_modules")));
        assert!(!idx.iter().any(|p| p.contains("scratch")));
        assert!(!idx.iter().any(|p| p.contains("Mobile Documents")));
        assert!(idx.iter().any(|p| p.ends_with("Library/Caches/c.bin")));
        assert!(!idx.iter().any(|p| p.contains("/.hidden")));
        assert!(idx.iter().any(|p| p.ends_with("to-b")), "links are entries");
        assert!(
            !idx.iter().any(|p| p.contains("to-b/")),
            "links are not traversed"
        );
        let hits = fs.find(&idx, "small");
        assert_eq!(hits[0].path, "/Users/alex/work/a/small.txt");
        assert_eq!(hits[0].score, 0, "stem match ranks first");
        let hits = fs.find(&idx, "txt");
        assert!(hits.iter().all(|h| h.score >= 10));
        assert!(
            fs.find(&idx, "   ").is_empty(),
            "a blank query has no results"
        );
        assert!(fs.find(&idx, "zzz").is_empty());
        let many: Vec<String> = (0..300).map(|i| format!("/Users/alex/f{i}.txt")).collect();
        assert_eq!(fs.find(&many, "f").len(), FIND_MAX_RESULTS);
    }

    #[test]
    fn human_sizes_and_sanitise() {
        assert_eq!(human(512), "512 B");
        assert_eq!(human(1536), "1.5 KiB");
        assert_eq!(human(5 * 1024 * 1024 * 1024), "5.0 GiB");
        assert_eq!(sanitize("a\tb\u{7}c"), "a    b�c");
    }
}
