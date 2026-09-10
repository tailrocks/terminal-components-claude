//! The one application-owned filesystem removal boundary (HP20–HP22).
//!
//! - `Category` and `policy` describe every insight category's roots, age
//!   rule, platform and process guard; eligibility is computed once and the
//!   same way from the tree, an insight or a custom entry.
//! - `validate` applies the lexical, protected-root, user deny and symlink
//!   ancestor rules; `authorize` freezes an immutable `DeletePlan` with a
//!   revision that any material change invalidates.
//! - `execute` revalidates each item at commit, sizes it, deduplicates
//!   ancestors and duplicates, honours the process guard, moves to Trash
//!   by default or removes permanently only when the plan says so, never
//!   falls back from Trash to permanent, writes one JSONL v1 record per
//!   requested item, and reports removed/trashed/would-remove/failed/skipped
//!   separately with log health.
//!
//! Threat model: an unprivileged single user. Parent revalidation mitigates
//! a replaced ancestor between review and commit; pathname APIs cannot pin
//! the final syscall against a concurrent rename, and this preview does not
//! claim otherwise. No sudo boundary is supported.

use std::collections::BTreeSet;

use crate::domain::context::Os;
use crate::sim::fs::{Fs, NodeKind};

pub const CACHE_SCHEMA: u32 = 3;
pub const CACHE_TTL_SECS: i64 = 7 * 86_400;
pub const CACHE_DEPTH: usize = 2;
pub const REPORT_DETAIL: usize = 6;
pub const ARTIFACT_DEPTH: usize = 6;

// ------------------------------------------------------------ taxonomy

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Safety {
    /// Regenerated on demand: preselected at any age.
    Rebuildable,
    /// Safe once old enough: preselected when older than the minimum.
    OldOnly,
    /// Needs a human look: never preselected.
    ReviewFirst,
}

impl Safety {
    pub fn label(self) -> &'static str {
        match self {
            Safety::Rebuildable => "rebuilt on demand",
            Safety::OldOnly => "safe if old",
            Safety::ReviewFirst => "review first",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detect {
    /// A directory root exists.
    Dir,
    /// The named executable is on PATH.
    Tool(&'static str),
    /// Tool on PATH or the directory exists.
    ToolOrDir(&'static str),
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category {
    pub id: &'static str,
    pub label: &'static str,
    pub safety: Safety,
    /// Minimum age in days for age-gated rules; 0 means any age.
    pub min_age_days: i64,
    pub detect: Detect,
    /// Home-relative roots; `~/Library/...`.
    pub roots: &'static [&'static str],
    pub macos_only: bool,
    /// Candidates are the children of the root; the root itself is protected.
    pub children_of_root: bool,
    /// A running process by this name blocks cleanup.
    pub guard_process: Option<&'static str>,
    pub note: &'static str,
}

pub const CATEGORIES: [Category; 18] = [
    Category {
        id: "xcode.derived-data",
        label: "Xcode DerivedData",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Dir,
        roots: &["Library/Developer/Xcode/DerivedData"],
        macos_only: true,
        children_of_root: false,
        guard_process: Some("Xcode"),
        note: "rebuilt by the next build",
    },
    Category {
        id: "xcode.device-support",
        label: "Xcode device support",
        safety: Safety::OldOnly,
        min_age_days: 90,
        detect: Detect::Dir,
        roots: &[
            "Library/Developer/Xcode/iOS DeviceSupport",
            "Library/Developer/Xcode/watchOS DeviceSupport",
            "Library/Developer/Xcode/tvOS DeviceSupport",
        ],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "re-downloaded when a device of that version connects",
    },
    Category {
        id: "xcode.archives",
        label: "Xcode archives",
        safety: Safety::ReviewFirst,
        min_age_days: 0,
        detect: Detect::Dir,
        roots: &["Library/Developer/Xcode/Archives"],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "signed archives may be needed for symbolication or redistribution",
    },
    Category {
        id: "simulator.caches",
        label: "Simulator caches",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Dir,
        roots: &["Library/Developer/CoreSimulator/Caches"],
        macos_only: true,
        children_of_root: false,
        guard_process: Some("Simulator"),
        note: "recreated by the Simulator",
    },
    Category {
        id: "brew.cache",
        label: "Homebrew cache",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Tool("brew"),
        roots: &["Library/Caches/Homebrew"],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "downloads are fetched again on install",
    },
    Category {
        id: "npm.cache",
        label: "npm cache",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Tool("npm"),
        roots: &[".npm/_cacache", ".npm/_logs"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "packages are fetched again on install",
    },
    Category {
        id: "pnpm.store",
        label: "pnpm store",
        safety: Safety::OldOnly,
        min_age_days: 30,
        detect: Detect::Tool("pnpm"),
        roots: &["Library/pnpm/store"],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "content-addressable store · resolved through pnpm store path",
    },
    Category {
        id: "yarn.cache",
        label: "Yarn cache",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Tool("yarn"),
        roots: &[".yarn/cache", "Library/Caches/Yarn"],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "packages are fetched again on install",
    },
    Category {
        id: "bun.cache",
        label: "Bun cache",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Tool("bun"),
        roots: &[".bun/install/cache"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "packages are fetched again on install",
    },
    Category {
        id: "cargo.registry-cache",
        label: "Cargo registry cache",
        safety: Safety::OldOnly,
        min_age_days: 30,
        detect: Detect::Tool("cargo"),
        roots: &[".cargo/registry/cache", ".cargo/git"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "crates are downloaded again on build",
    },
    Category {
        id: "gradle.caches",
        label: "Gradle caches",
        safety: Safety::OldOnly,
        min_age_days: 30,
        detect: Detect::Dir,
        roots: &[".gradle/caches", ".gradle/daemon", ".gradle/wrapper/dists"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "gradle --stop runs first · dependencies are downloaded again",
    },
    Category {
        id: "maven.repository",
        label: "Maven repository",
        safety: Safety::ReviewFirst,
        min_age_days: 0,
        detect: Detect::Dir,
        roots: &[".m2/repository"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "snapshots and offline builds may need it",
    },
    Category {
        id: "pip.cache",
        label: "pip cache",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::ToolOrDir("pip3"),
        roots: &["Library/Caches/pip"],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "wheels are downloaded again on install",
    },
    Category {
        id: "uv.cache",
        label: "uv cache",
        safety: Safety::Rebuildable,
        min_age_days: 0,
        detect: Detect::Tool("uv"),
        roots: &[".cache/uv"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "$XDG_CACHE_HOME/uv or ~/.cache/uv",
    },
    Category {
        id: "user.caches",
        label: "User caches",
        safety: Safety::ReviewFirst,
        min_age_days: 30,
        detect: Detect::Always,
        roots: &["Library/Caches"],
        macos_only: true,
        children_of_root: true,
        guard_process: None,
        note: "each application cache is reviewed on its own · the root stays",
    },
    Category {
        id: "user.logs",
        label: "User logs",
        safety: Safety::Rebuildable,
        min_age_days: 7,
        detect: Detect::Always,
        roots: &["Library/Logs"],
        macos_only: true,
        children_of_root: true,
        guard_process: None,
        note: "older than a week · the root stays",
    },
    Category {
        id: "ide.jetbrains-logs",
        label: "JetBrains logs",
        safety: Safety::Rebuildable,
        min_age_days: 7,
        detect: Detect::Dir,
        roots: &["Library/Logs/JetBrains"],
        macos_only: true,
        children_of_root: false,
        guard_process: None,
        note: "older than a week",
    },
    Category {
        id: "project.artifacts",
        label: "Project artifacts",
        safety: Safety::ReviewFirst,
        min_age_days: 7,
        detect: Detect::Always,
        roots: &["Projects"],
        macos_only: false,
        children_of_root: false,
        guard_process: None,
        note: "generated directories beside a project indicator · rebuilt by the project's own tooling",
    },
];

pub fn category(id: &str) -> Option<&'static Category> {
    CATEGORIES.iter().find(|c| c.id == id)
}

pub const ARTIFACT_NAMES: [&str; 12] = [
    "node_modules",
    "target",
    "build",
    "dist",
    ".venv",
    "venv",
    ".next",
    ".turbo",
    ".gradle",
    "DerivedData",
    "Pods",
    "__pycache__",
];

pub const INDICATORS: [&str; 8] = [
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    ".git",
];

/// Folder names the disk tree folds into one selectable aggregate row.
pub const NOISE_NAMES: [&str; 14] = [
    "node_modules",
    ".git",
    "target",
    "build",
    "dist",
    ".venv",
    "venv",
    "__pycache__",
    "DerivedData",
    "Pods",
    ".gradle",
    ".next",
    ".turbo",
    ".cache",
];

/// Observed process state for guards: a failed probe is unknown, never
/// "not running".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessObservation {
    Running,
    NotRunning,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eligibility {
    /// Selected by default.
    Preselected,
    /// Selectable, unselected by default (review-first).
    Selectable,
    /// Visible, cannot be selected.
    Ineligible(String),
}

impl Eligibility {
    pub fn selectable(&self) -> bool {
        !matches!(self, Eligibility::Ineligible(_))
    }
}

/// Age in days from an mtime; `None` when unknown or in the future.
pub fn age_days(mtime: i64, now_secs: i64) -> Option<i64> {
    if mtime > now_secs {
        return None;
    }
    Some((now_secs - mtime) / 86_400)
}

/// The shared eligibility rule for a category candidate.
pub fn eligibility(cat: &Category, age: Option<i64>, process: &ProcessObservation) -> Eligibility {
    if let Some(name) = cat.guard_process {
        match process {
            ProcessObservation::Running => {
                return Eligibility::Ineligible(format!("{name} is running"));
            }
            ProcessObservation::Unknown(why) => {
                return Eligibility::Ineligible(format!("{name} state unknown · {why}"));
            }
            ProcessObservation::NotRunning => {}
        }
    }
    if cat.min_age_days > 0 {
        match age {
            None => return Eligibility::Ineligible("age unknown · treated as recent".into()),
            Some(d) if d < cat.min_age_days => {
                return Eligibility::Ineligible(format!(
                    "too recent · {d} d < {} d",
                    cat.min_age_days
                ));
            }
            _ => {}
        }
    }
    match cat.safety {
        Safety::ReviewFirst => Eligibility::Selectable,
        Safety::Rebuildable | Safety::OldOnly => Eligibility::Preselected,
    }
}

/// Is a category detected on this host: platform, tool and root rules.
pub fn detected(cat: &Category, os: Os, home: &str, fs: &Fs, tools: &BTreeSet<String>) -> bool {
    if cat.macos_only && os != Os::MacOs {
        return false;
    }
    let root_exists = cat.roots.iter().any(|r| fs.is_dir(&format!("{home}/{r}")));
    match cat.detect {
        Detect::Always => true,
        Detect::Dir => root_exists,
        Detect::Tool(t) => tools.contains(t),
        Detect::ToolOrDir(t) => tools.contains(t) || root_exists,
    }
}

/// Resolve the pnpm store: accept only an absolute canonical path below a
/// known home root; anything else falls back to `Library/pnpm/store`.
pub fn pnpm_store_root(home: &str, resolved: Result<&str, &str>) -> (String, Option<String>) {
    let fallback = format!("{home}/Library/pnpm/store");
    let allowed = [
        format!("{home}/Library/pnpm/store"),
        format!("{home}/.local/share/pnpm/store"),
        format!("{home}/.pnpm-store"),
    ];
    match resolved {
        Ok(p)
            if allowed
                .iter()
                .any(|a| p == a || p.starts_with(&format!("{a}/"))) =>
        {
            (p.to_owned(), None)
        }
        Ok(p) => (
            fallback,
            Some(format!(
                "pnpm store path {p} is outside the known roots · using the default"
            )),
        ),
        Err(e) => (
            fallback,
            Some(format!("pnpm store path failed: {e} · using the default")),
        ),
    }
}

/// Classify a directory as a project artifact: an exact artifact name with
/// a project indicator beside it (LD041).
pub fn is_artifact(fs: &Fs, path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or("");
    if !ARTIFACT_NAMES.contains(&name) {
        return false;
    }
    let Some(parent) = Fs::parent(path) else {
        return false;
    };
    INDICATORS
        .iter()
        .any(|i| fs.exists(&format!("{parent}/{i}")))
}

/// Recursively find artifacts under roots (max depth six), never descending
/// into a found artifact or a directory symlink; roots are deduplicated.
pub fn find_artifacts(fs: &Fs, roots: &[String]) -> Vec<String> {
    let mut uniq: Vec<String> = vec![];
    for r in roots {
        if !uniq.contains(r) && !uniq.iter().any(|u| r.starts_with(&format!("{u}/"))) {
            uniq.push(r.clone());
        }
    }
    let mut out = vec![];
    for r in uniq {
        walk_artifacts(fs, &r, 0, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

fn walk_artifacts(fs: &Fs, dir: &str, depth: usize, out: &mut Vec<String>) {
    if depth > ARTIFACT_DEPTH {
        return;
    }
    let Ok(children) = fs.list(dir) else {
        return;
    };
    for c in children {
        if !c.is_dir() {
            continue;
        }
        if is_artifact(fs, &c.path) {
            out.push(c.path.clone());
            continue;
        }
        walk_artifacts(fs, &c.path, depth + 1, out);
    }
}

/// The shared Gradle/IDEA candidate walker (OP50): skips every symlink,
/// never enters node_modules, depth five, does not descend a selected
/// directory, unreadable entries skipped, sorted unique paths.
pub fn walk_candidates(fs: &Fs, root: &str, select: &dyn Fn(&str, bool) -> bool) -> Vec<String> {
    walk_candidates_report(fs, root, select).0
}

/// The walk plus every directory it could not read: a folder it had to
/// skip is reported, so a partial answer is never presented as complete.
pub fn walk_candidates_report(
    fs: &Fs,
    root: &str,
    select: &dyn Fn(&str, bool) -> bool,
) -> (Vec<String>, Vec<String>) {
    let mut out = vec![];
    let mut unreadable = vec![];
    walk_sel(fs, root, 0, select, &mut out, &mut unreadable);
    out.sort();
    out.dedup();
    unreadable.sort();
    unreadable.dedup();
    (out, unreadable)
}

fn walk_sel(
    fs: &Fs,
    dir: &str,
    depth: usize,
    select: &dyn Fn(&str, bool) -> bool,
    out: &mut Vec<String>,
    unreadable: &mut Vec<String>,
) {
    if depth > 5 {
        return;
    }
    let children = match fs.list(dir) {
        Ok(c) => c,
        Err(_) => {
            unreadable.push(dir.to_owned());
            return;
        }
    };
    for c in children {
        if matches!(c.kind, NodeKind::Symlink { .. }) {
            continue;
        }
        if c.name() == "node_modules" {
            continue;
        }
        if select(&c.path, c.is_dir()) {
            out.push(c.path.clone());
            continue;
        }
        if c.is_dir() {
            if c.readable {
                walk_sel(fs, &c.path, depth + 1, select, out, unreadable);
            } else {
                unreadable.push(c.path.clone());
            }
        }
    }
}

// ------------------------------------------------------------ validation

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deny(pub String);

fn lexical(path: &str) -> Result<Vec<&str>, Deny> {
    if !path.starts_with('/') {
        return Err(Deny("path must be absolute".into()));
    }
    if path == "/" {
        return Err(Deny("the filesystem root is protected".into()));
    }
    if path.ends_with('/') {
        return Err(Deny("trailing slash".into()));
    }
    let comps: Vec<&str> = path[1..].split('/').collect();
    for c in &comps {
        if c.is_empty() {
            return Err(Deny("empty path component (repeated slash)".into()));
        }
        if *c == ".." || *c == "." {
            return Err(Deny(format!("relative component `{c}`")));
        }
    }
    Ok(comps)
}

const PROTECTED_SUBTREES: [&str; 8] = [
    "/bin",
    "/sbin",
    "/etc",
    "/private/etc",
    "/System",
    "/var/db",
    "/private/var/db",
    "/usr",
];
const PROTECTED_EXACT: [&str; 5] = ["/usr/local", "/Library", "/Applications", "/Users", "/home"];

fn under(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{root}/"))
}

/// Protected system roots and subtrees (LD053).
pub fn system_rule(path: &str) -> Option<Deny> {
    for s in PROTECTED_SUBTREES {
        if under(path, s) {
            if s == "/usr" && path.starts_with("/usr/local/") {
                return None;
            }
            return Some(Deny(format!("{s} is protected")));
        }
    }
    if PROTECTED_EXACT.contains(&path) {
        return Some(Deny(format!("{path} itself is protected")));
    }
    None
}

/// User-path deny rules under the home directory (LD054).
pub fn user_rule(path: &str, home: &str) -> Option<Deny> {
    if path == home {
        return Some(Deny("your home directory is protected".into()));
    }
    let rel = path.strip_prefix(&format!("{home}/"))?;
    let lower = rel.to_lowercase();
    let first = rel.split('/').next().unwrap_or("");
    if first == ".Trash" {
        return Some(Deny("the Trash is managed by the system".into()));
    }
    if rel == "Library/Caches" || rel == "Library/Logs" {
        return Some(Deny(format!(
            "~/{rel} itself is protected · its children may be cleaned"
        )));
    }
    if let Some(lib) = rel.strip_prefix("Library/") {
        let seg = lib.split('/').next().unwrap_or("");
        if seg.starts_with("Mobile Documents") {
            return Some(Deny("iCloud Drive data is never a cleanup target".into()));
        }
        if seg == "Keychains" || seg == "Application Support" || seg == "Safari" || seg == "WebKit"
        {
            return Some(Deny(format!("~/Library/{seg} holds user data")));
        }
        if seg == "Containers" || seg == "Group Containers" {
            let sub = lower.split('/').nth(2).unwrap_or("");
            if sub.contains("safari")
                || sub.contains("webkit")
                || sub.contains("docker")
                || sub.contains("chrome")
                || sub.contains("firefox")
            {
                return Some(Deny(format!(
                    "{seg} for a browser or Docker holds user data"
                )));
            }
        }
    }
    None
}

/// Every rule, in order: lexical, system, user, symlink ancestors.
pub fn validate(path: &str, home: &str, os: Os, fs: &Fs) -> Result<String, Deny> {
    lexical(path)?;
    if let Some(d) = system_rule(path) {
        return Err(d);
    }
    if let Some(d) = user_rule(path, home) {
        return Err(d);
    }
    // a missing ancestor cannot be a link: the deny rules above already ran
    // lexically and the commit reports the leaf as not found
    let (parent, links) = match fs.canonical_parent(path) {
        Ok(v) => v,
        Err(crate::sim::fs::FsError::NotFound(_)) => {
            (Fs::parent(path).unwrap_or("/".into()), vec![])
        }
        Err(e) => return Err(Deny(e.message())),
    };
    for l in &links {
        let alias = os == Os::MacOs && (l == "/tmp" || l == "/var");
        if !alias {
            return Err(Deny(format!("{l} is a symbolic link on the path")));
        }
    }
    let canonical = Fs::join(&parent, path.rsplit('/').next().unwrap_or(""));
    if let Some(d) = system_rule(&canonical) {
        return Err(d);
    }
    if let Some(d) = user_rule(&canonical, home) {
        return Err(d);
    }
    Ok(canonical)
}

/// A selected ancestor must not contain a protected descendant.
pub fn protected_descendant(path: &str, home: &str, fs: &Fs) -> Option<String> {
    fs.subtree(path)
        .iter()
        .find(|n| user_rule(&n.path, home).is_some() || system_rule(&n.path).is_some())
        .map(|n| n.path.clone())
}

// ------------------------------------------------------------ plan

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Trash,
    Permanent,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Trash => "trash",
            Mode::Permanent => "permanent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteItem {
    pub path: String,
    pub category: String,
    /// Estimated allocated bytes at review time.
    pub estimate: u64,
    pub guard: Option<&'static str>,
}

/// An immutable, authorized deletion. Any material change (paths, mode,
/// dry run, policy) yields a new revision; the executor refuses a stale one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePlan {
    pub host: String,
    pub root: String,
    pub items: Vec<DeleteItem>,
    pub mode: Mode,
    pub dry_run: bool,
    pub revision: u64,
}

impl DeletePlan {
    pub fn new(host: &str, root: &str, items: Vec<DeleteItem>, mode: Mode, dry_run: bool) -> Self {
        let mut p = Self {
            host: host.into(),
            root: root.into(),
            items,
            mode,
            dry_run,
            revision: 0,
        };
        p.revision = p.fingerprint();
        p
    }

    /// A stable fingerprint of everything material.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |s: &str| {
            for b in s.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            h ^= 0xff;
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        feed(&self.host);
        feed(&self.root);
        feed(self.mode.label());
        feed(if self.dry_run { "dry" } else { "live" });
        for i in &self.items {
            feed(&i.path);
            feed(&i.category);
        }
        h
    }

    pub fn estimate(&self) -> u64 {
        self.items.iter().map(|i| i.estimate).sum()
    }

    /// The typed phrase for a broad operation, bound to host and root.
    pub fn phrase(&self) -> String {
        format!(
            "{} {} UNDER {} ON {}",
            if self.mode == Mode::Permanent {
                "PERMANENTLY DELETE"
            } else {
                "TRASH"
            },
            self.items.len(),
            self.root,
            self.host
        )
    }
}

// ------------------------------------------------------------ execution

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Removed,
    Trashed,
    WouldRemove,
    Failed,
    Skipped,
}

impl Outcome {
    pub fn label(self) -> &'static str {
        match self {
            Outcome::Removed => "removed",
            Outcome::Trashed => "trashed",
            Outcome::WouldRemove => "would_remove",
            Outcome::Failed => "failed",
            Outcome::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemResult {
    pub path: String,
    pub outcome: Outcome,
    pub bytes: u64,
    pub error: Option<String>,
}

/// One JSONL v1 record per requested item (LD060).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    pub timestamp_ms: i64,
    pub mode: Mode,
    pub dry_run: bool,
    pub path: String,
    pub size: u64,
    pub outcome: Outcome,
    pub error: Option<String>,
}

impl LogRecord {
    pub fn json(&self) -> String {
        let esc = |s: &str| {
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        };
        format!(
            "{{\"v\":1,\"timestamp_ms\":{},\"mode\":\"{}\",\"dry_run\":{},\"path\":\"{}\",\"size\":{},\"outcome\":\"{}\"{}}}",
            self.timestamp_ms,
            self.mode.label(),
            self.dry_run,
            esc(&self.path),
            self.size,
            self.outcome.label(),
            match &self.error {
                Some(e) => format!(",\"error\":\"{}\"", esc(e)),
                None => String::new(),
            }
        )
    }
}

/// The in-memory operation log at `$XDG_CACHE_HOME/holla/ops.log`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpsLog {
    pub path: String,
    pub lines: Vec<String>,
    /// Appends fail (read-only cache): failures are counted, never hidden,
    /// and never block or roll back a deletion.
    pub write_failure: Option<String>,
    pub failures: u32,
}

impl OpsLog {
    pub fn append(&mut self, r: &LogRecord) {
        if let Some(_e) = &self.write_failure {
            self.failures += 1;
            return;
        }
        self.lines.push(r.json());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub plan_revision: u64,
    pub mode: Mode,
    pub dry_run: bool,
    pub items: Vec<ItemResult>,
    pub log_failures: u32,
    pub log_path: String,
    /// Allocated bytes that left the tree (Trash or permanent).
    pub bytes: u64,
    /// Bytes physically freed on the volume now (permanent only).
    pub freed_now: u64,
}

impl Report {
    pub fn count(&self, o: Outcome) -> usize {
        self.items.iter().filter(|i| i.outcome == o).count()
    }
    /// First six failed/skipped details and the overflow count.
    pub fn details(&self) -> (Vec<&ItemResult>, usize) {
        let all: Vec<&ItemResult> = self
            .items
            .iter()
            .filter(|i| matches!(i.outcome, Outcome::Failed | Outcome::Skipped))
            .collect();
        let over = all.len().saturating_sub(REPORT_DETAIL);
        (all.into_iter().take(REPORT_DETAIL).collect(), over)
    }
    pub fn summary(&self) -> String {
        let mut parts = vec![];
        for o in [
            Outcome::Trashed,
            Outcome::Removed,
            Outcome::WouldRemove,
            Outcome::Failed,
            Outcome::Skipped,
        ] {
            let n = self.count(o);
            if n > 0 {
                parts.push(format!(
                    "{n} {}",
                    match o {
                        Outcome::WouldRemove => "would be removed",
                        other => other.label(),
                    }
                ));
            }
        }
        if parts.is_empty() {
            parts.push("nothing to do".into());
        }
        if self.log_failures > 0 {
            parts.push(format!("{} log write failures", self.log_failures));
        }
        parts.join(" · ")
    }
    pub fn incomplete(&self) -> bool {
        self.count(Outcome::Failed) + self.count(Outcome::Skipped) > 0
    }
}

pub struct ExecContext<'a> {
    pub home: &'a str,
    pub os: Os,
    pub now_secs: i64,
    pub processes: &'a dyn Fn(&str) -> ProcessObservation,
}

/// Execute an authorized plan against the filesystem. `expected_revision`
/// must match the plan's fingerprint; a drifted plan is refused before any
/// effect.
pub fn execute(
    plan: &DeletePlan,
    expected_revision: u64,
    fs: &mut Fs,
    log: &mut OpsLog,
    cx: &ExecContext,
) -> Result<Report, String> {
    if plan.revision != expected_revision || plan.fingerprint() != plan.revision {
        return Err("the reviewed plan changed · confirm again".into());
    }
    let mut report = Report {
        plan_revision: plan.revision,
        mode: plan.mode,
        dry_run: plan.dry_run,
        items: vec![],
        log_failures: 0,
        log_path: log.path.clone(),
        bytes: 0,
        freed_now: 0,
    };
    let mut done: BTreeSet<String> = BTreeSet::new();
    let selected: Vec<&str> = plan.items.iter().map(|i| i.path.as_str()).collect();
    let mut processes: std::collections::BTreeMap<&str, ProcessObservation> = Default::default();
    for item in &plan.items {
        let path = item.path.as_str();
        let record = |outcome: Outcome,
                      bytes: u64,
                      error: Option<String>,
                      log: &mut OpsLog,
                      report: &mut Report| {
            log.append(&LogRecord {
                timestamp_ms: cx.now_secs * 1000,
                mode: plan.mode,
                dry_run: plan.dry_run,
                path: path.into(),
                size: bytes,
                outcome,
                error: error.clone(),
            });
            report.items.push(ItemResult {
                path: path.into(),
                outcome,
                bytes,
                error,
            });
        };
        // duplicates and descendants of another selected ancestor
        if !done.insert(path.to_owned()) {
            record(
                Outcome::Skipped,
                0,
                Some("duplicate of an earlier item".into()),
                log,
                &mut report,
            );
            continue;
        }
        if let Some(anc) = selected
            .iter()
            .find(|a| **a != path && path.starts_with(&format!("{a}/")))
        {
            record(
                Outcome::Skipped,
                0,
                Some(format!("covered by its ancestor {anc}")),
                log,
                &mut report,
            );
            continue;
        }
        // process guard, observed once per batch
        if let Some(g) = item.guard {
            let obs = processes
                .entry(g)
                .or_insert_with(|| (cx.processes)(g))
                .clone();
            match obs {
                ProcessObservation::Running => {
                    record(
                        Outcome::Skipped,
                        0,
                        Some(format!("{g} is running")),
                        log,
                        &mut report,
                    );
                    continue;
                }
                ProcessObservation::Unknown(why) => {
                    record(
                        Outcome::Skipped,
                        0,
                        Some(format!("{g} state unknown · {why}")),
                        log,
                        &mut report,
                    );
                    continue;
                }
                ProcessObservation::NotRunning => {}
            }
        }
        // validation is re-run at commit, after sizing, immediately before mutation
        if let Err(Deny(why)) = validate(path, cx.home, cx.os, fs) {
            record(Outcome::Skipped, 0, Some(why), log, &mut report);
            continue;
        }
        if let Some(child) = protected_descendant(path, cx.home, fs) {
            record(
                Outcome::Skipped,
                0,
                Some(format!("contains protected {child}")),
                log,
                &mut report,
            );
            continue;
        }
        if !fs.exists(path) {
            record(
                Outcome::Failed,
                0,
                Some("no such file or directory".into()),
                log,
                &mut report,
            );
            continue;
        }
        let scan = fs.scan(
            path,
            &crate::sim::fs::ScanOptions {
                include_hidden: true,
                ..Default::default()
            },
        );
        if scan.errors() > 0 {
            record(
                Outcome::Failed,
                0,
                Some("size traversal failed · unreadable entry".into()),
                log,
                &mut report,
            );
            continue;
        }
        let bytes = scan.allocated;
        if let Err(Deny(why)) = validate(path, cx.home, cx.os, fs) {
            record(
                Outcome::Skipped,
                0,
                Some(format!("changed after sizing · {why}")),
                log,
                &mut report,
            );
            continue;
        }
        if plan.dry_run {
            record(Outcome::WouldRemove, bytes, None, log, &mut report);
            continue;
        }
        match plan.mode {
            Mode::Trash => match fs.trash(path, cx.now_secs) {
                Ok(b) => {
                    report.bytes += b;
                    record(Outcome::Trashed, b, None, log, &mut report);
                }
                Err(e) => record(Outcome::Failed, 0, Some(e), log, &mut report),
            },
            Mode::Permanent => match fs.remove_permanent(path) {
                Ok(b) => {
                    report.bytes += b;
                    report.freed_now += b;
                    record(Outcome::Removed, b, None, log, &mut report);
                }
                Err(e) => record(Outcome::Failed, 0, Some(e), log, &mut report),
            },
        }
    }
    report.log_failures = log.failures;
    log.failures = 0;
    Ok(report)
}

// ------------------------------------------------------------ size cache

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    pub path: String,
    pub allocated: u64,
    pub mtime: i64,
    pub recorded_secs: i64,
}

/// `sizes.json` v3: root plus depth two, seven-day TTL and exact mtime
/// validity (LD017/LD018).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SizeCache {
    pub entries: Vec<CacheEntry>,
    pub load_error: Option<String>,
    pub save_error: Option<String>,
}

impl SizeCache {
    /// A valid hint for `path`: fresh and matching the current mtime.
    pub fn hint(&self, path: &str, mtime: i64, now_secs: i64) -> Option<&CacheEntry> {
        self.entries.iter().find(|e| {
            e.path == path && e.mtime == mtime && now_secs - e.recorded_secs <= CACHE_TTL_SECS
        })
    }

    /// Record a scan: the root and nodes within two levels, omitting paths
    /// whose mtime changed since the pre-scan snapshot.
    pub fn record(
        &mut self,
        root: &str,
        tree: &crate::sim::fs::ScanNode,
        snapshot: &[(String, i64)],
        fs: &Fs,
        now_secs: i64,
    ) {
        fn walk(n: &crate::sim::fs::ScanNode, depth: usize, out: &mut Vec<(String, u64)>) {
            if depth > CACHE_DEPTH {
                return;
            }
            if n.is_dir && n.error.is_none() {
                out.push((n.path.clone(), n.allocated));
            }
            for c in &n.children {
                walk(c, depth + 1, out);
            }
        }
        let mut found = vec![];
        walk(tree, 0, &mut found);
        for (p, allocated) in found {
            let Some(node) = fs.get(&p) else { continue };
            let changed = snapshot
                .iter()
                .find(|(sp, _)| *sp == p)
                .is_some_and(|(_, m)| *m != node.mtime);
            if changed {
                continue;
            }
            self.entries.retain(|e| e.path != p);
            self.entries.push(CacheEntry {
                path: p,
                allocated,
                mtime: node.mtime,
                recorded_secs: now_secs,
            });
        }
        let _ = root;
        self.entries.sort_by(|a, b| a.path.cmp(&b.path));
    }

    pub fn serialize(&self) -> String {
        let items: Vec<String> = self
            .entries
            .iter()
            .map(|e| {
                format!(
                    "{{\"path\":\"{}\",\"allocated\":{},\"mtime\":{},\"at\":{}}}",
                    e.path.replace('"', "\\\""),
                    e.allocated,
                    e.mtime,
                    e.recorded_secs
                )
            })
            .collect();
        format!("{{\"v\":{CACHE_SCHEMA},\"entries\":[{}]}}", items.join(","))
    }

    /// Load; corrupt or unknown schema is ignored (empty with a reason).
    pub fn load(text: Option<&str>, now_secs: i64) -> Self {
        let Some(text) = text else {
            return Self::default();
        };
        use crate::domain::manifest::{Json, parse_json};
        let json = match parse_json(text) {
            Ok(j) => j,
            Err(e) => {
                return Self {
                    load_error: Some(format!("sizes.json unreadable: {e}")),
                    ..Default::default()
                };
            }
        };
        match json.get("v") {
            Some(Json::Num(n)) if *n as u32 == CACHE_SCHEMA => {}
            other => {
                return Self {
                    load_error: Some(format!(
                        "sizes.json schema {} ignored",
                        match other {
                            Some(Json::Num(n)) => (*n as i64).to_string(),
                            _ => "missing".into(),
                        }
                    )),
                    ..Default::default()
                };
            }
        }
        let mut c = Self::default();
        if let Some(Json::Arr(es)) = json.get("entries") {
            for e in es {
                let (Some(path), Some(Json::Num(a)), Some(Json::Num(m)), Some(Json::Num(at))) = (
                    e.get("path").and_then(Json::as_str),
                    e.get("allocated"),
                    e.get("mtime"),
                    e.get("at"),
                ) else {
                    continue;
                };
                if now_secs - (*at as i64) > CACHE_TTL_SECS {
                    continue;
                }
                c.entries.push(CacheEntry {
                    path: path.into(),
                    allocated: *a as u64,
                    mtime: *m as i64,
                    recorded_secs: *at as i64,
                });
            }
        }
        c
    }

    /// Merge another writer's entries: newer wins, expired dropped.
    pub fn merge(&mut self, other: &SizeCache, now_secs: i64) {
        for o in &other.entries {
            if now_secs - o.recorded_secs > CACHE_TTL_SECS {
                continue;
            }
            match self.entries.iter_mut().find(|e| e.path == o.path) {
                Some(e) if e.recorded_secs < o.recorded_secs => *e = o.clone(),
                Some(_) => {}
                None => self.entries.push(o.clone()),
            }
        }
        self.entries
            .retain(|e| now_secs - e.recorded_secs <= CACHE_TTL_SECS);
        self.entries.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::EPOCH_SECS;
    use crate::sim::fs::BLOCK;

    const HOME: &str = "/Users/alex";

    fn fs() -> Fs {
        let mut fs = Fs::new();
        fs.volume("/", 1000 * BLOCK, 500 * BLOCK);
        fs.dir(HOME, 400);
        fs.file(
            &format!("{HOME}/Library/Caches/com.app/a.bin"),
            3 * BLOCK,
            45,
        );
        fs.file(&format!("{HOME}/Library/Caches/fresh/b.bin"), BLOCK, 2);
        fs.file(&format!("{HOME}/Library/Logs/old.log"), BLOCK, 20);
        fs.dir(
            &format!("{HOME}/Library/Developer/Xcode/DerivedData/App-abc"),
            3,
        );
        fs.file(
            &format!("{HOME}/Library/Developer/Xcode/DerivedData/App-abc/x.o"),
            5 * BLOCK,
            3,
        );
        fs.file(&format!("{HOME}/Projects/web/package.json"), 100, 40);
        fs.file(
            &format!("{HOME}/Projects/web/node_modules/a/index.js"),
            4 * BLOCK,
            40,
        );
        fs.file(
            &format!("{HOME}/Projects/web/src/build/decoy.txt"),
            BLOCK,
            1,
        );
        fs.file(&format!("{HOME}/Projects/plain/build/x"), BLOCK, 40);
        fs.file(&format!("{HOME}/Projects/rs/Cargo.toml"), 10, 40);
        fs.file(
            &format!("{HOME}/Projects/rs/target/debug/bin"),
            6 * BLOCK,
            10,
        );
        fs.file(
            &format!("{HOME}/Projects/rs/target/node_modules/nested/x"),
            BLOCK,
            10,
        );
        fs.symlink(
            &format!("{HOME}/Projects/link"),
            &format!("{HOME}/Projects/web"),
        );
        fs.file(
            &format!("{HOME}/Library/Keychains/login.keychain"),
            BLOCK,
            100,
        );
        fs.symlink("/tmp", "/private/tmp");
        fs.file("/private/tmp/junk", BLOCK, 5);
        fs
    }

    #[test]
    fn eligibility_follows_age_safety_and_guards() {
        let xc = category("xcode.derived-data").unwrap();
        assert_eq!(
            eligibility(xc, Some(0), &ProcessObservation::NotRunning),
            Eligibility::Preselected
        );
        assert_eq!(
            eligibility(xc, Some(0), &ProcessObservation::Running),
            Eligibility::Ineligible("Xcode is running".into())
        );
        assert!(matches!(
            eligibility(xc, Some(0), &ProcessObservation::Unknown("pgrep failed".into())),
            Eligibility::Ineligible(m) if m.contains("unknown")
        ));
        let old = category("xcode.device-support").unwrap();
        assert!(matches!(
            eligibility(old, Some(89), &ProcessObservation::NotRunning),
            Eligibility::Ineligible(_)
        ));
        assert_eq!(
            eligibility(old, Some(90), &ProcessObservation::NotRunning),
            Eligibility::Preselected
        );
        assert!(
            matches!(eligibility(old, None, &ProcessObservation::NotRunning), Eligibility::Ineligible(m) if m.contains("age unknown"))
        );
        let rf = category("xcode.archives").unwrap();
        assert_eq!(
            eligibility(rf, Some(0), &ProcessObservation::NotRunning),
            Eligibility::Selectable
        );
        let uc = category("user.caches").unwrap();
        assert_eq!(
            eligibility(uc, Some(31), &ProcessObservation::NotRunning),
            Eligibility::Selectable
        );
        assert!(matches!(
            eligibility(uc, Some(29), &ProcessObservation::NotRunning),
            Eligibility::Ineligible(_)
        ));
        assert_eq!(
            age_days(EPOCH_SECS + 10, EPOCH_SECS),
            None,
            "future is unknown"
        );
        assert_eq!(age_days(EPOCH_SECS - 7 * 86_400, EPOCH_SECS), Some(7));
        assert_eq!(CATEGORIES.len(), 18);
        let ids: BTreeSet<&str> = CATEGORIES.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), 18);
    }

    #[test]
    fn detection_respects_platform_tools_and_roots() {
        let fs = fs();
        let tools: BTreeSet<String> = ["brew", "cargo"].iter().map(|s| s.to_string()).collect();
        let d = |id: &str, os: Os| detected(category(id).unwrap(), os, HOME, &fs, &tools);
        assert!(d("xcode.derived-data", Os::MacOs));
        assert!(
            !d("xcode.derived-data", Os::Debian),
            "mac-only categories hide on Linux"
        );
        assert!(!d("xcode.archives", Os::MacOs), "no root, no category");
        assert!(d("brew.cache", Os::MacOs));
        assert!(d("cargo.registry-cache", Os::Debian));
        assert!(!d("npm.cache", Os::MacOs));
        assert!(d("user.caches", Os::MacOs));
        assert!(d("project.artifacts", Os::Debian));
        let (root, note) = pnpm_store_root(HOME, Ok("/Users/alex/.local/share/pnpm/store/v3"));
        assert!(root.ends_with("pnpm/store/v3") && note.is_none());
        let (root, note) = pnpm_store_root(HOME, Ok("/elsewhere/store"));
        assert!(root.ends_with("Library/pnpm/store") && note.unwrap().contains("outside"));
        let (root, note) = pnpm_store_root(HOME, Err("exit 1"));
        assert!(root.ends_with("Library/pnpm/store") && note.unwrap().contains("failed"));
    }

    #[test]
    fn artifacts_need_an_indicator_and_never_nest() {
        let fs = fs();
        let found = find_artifacts(
            &fs,
            &[format!("{HOME}/Projects"), format!("{HOME}/Projects/web")],
        );
        assert_eq!(
            found,
            vec![
                format!("{HOME}/Projects/rs/target"),
                format!("{HOME}/Projects/web/node_modules")
            ],
            "decoys without an indicator and nested artifacts are excluded"
        );
        assert!(!is_artifact(&fs, &format!("{HOME}/Projects/plain/build")));
        // the shared walker for Gradle/IDEA skips links and node_modules
        let sel = |p: &str, is_dir: bool| is_dir && p.ends_with("/target");
        let c = walk_candidates(&fs, &format!("{HOME}/Projects"), &sel);
        assert_eq!(c, vec![format!("{HOME}/Projects/rs/target")]);
        let sel = |p: &str, _: bool| p.ends_with("index.js");
        assert!(
            walk_candidates(&fs, &format!("{HOME}/Projects"), &sel).is_empty(),
            "node_modules is never entered"
        );
    }

    #[test]
    fn validation_denies_every_documented_rule() {
        let fs = fs();
        let v = |p: &str| validate(p, HOME, Os::MacOs, &fs);
        assert!(v("relative/x").is_err());
        assert!(v("/").is_err());
        assert!(v("/Users/alex/x/").is_err());
        assert!(v("/Users//alex/x").is_err());
        assert!(v("/Users/alex/../root").is_err());
        assert!(v("/usr/bin/git").is_err());
        assert!(v("/usr/local").is_err());
        assert!(v("/Library").is_err());
        assert!(v("/Users").is_err());
        assert!(v(HOME).is_err());
        assert!(v(&format!("{HOME}/.Trash/x")).is_err());
        assert!(v(&format!("{HOME}/Library/Keychains/login.keychain")).is_err());
        assert!(v(&format!("{HOME}/Library/Application Support/App")).is_err());
        assert!(v(&format!("{HOME}/Library/Safari/History.db")).is_err());
        assert!(v(&format!("{HOME}/Library/Containers/com.apple.Safari/Data")).is_err());
        assert!(v(&format!("{HOME}/Library/Containers/com.docker.docker/Data")).is_err());
        assert!(
            v(&format!(
                "{HOME}/Library/Mobile Documents/com~apple~CloudDocs/x"
            ))
            .is_err()
        );
        assert!(v(&format!("{HOME}/Library/Caches")).is_err());
        assert!(v(&format!("{HOME}/Library/Logs")).is_err());
        // allowed: children of Caches/Logs, /usr/local children, /Library children, Unicode and newline names
        assert!(v(&format!("{HOME}/Library/Caches/com.app")).is_ok());
        assert!(v(&format!("{HOME}/Library/Logs/old.log")).is_ok());
        assert!(v("/usr/local/lib/x").is_ok());
        assert!(v("/Library/Caches/x").is_ok());
        assert!(v(&format!("{HOME}/Projects/ünï\ncode")).is_ok());
        // a valid-looking missing path passes lexical validation
        assert!(v(&format!("{HOME}/Projects/nothing-here")).is_ok());
        // symlink ancestors are refused, except the macOS /tmp and /var aliases
        assert!(v(&format!("{HOME}/Projects/link/node_modules")).is_err());
        assert!(
            v(&format!("{HOME}/Projects/link")).is_ok(),
            "a selected link removes the link only"
        );
        assert_eq!(v("/tmp/junk").unwrap(), "/private/tmp/junk");
        assert!(
            validate("/tmp/junk", HOME, Os::Debian, &fs).is_err(),
            "no alias on Linux"
        );
        // an ancestor containing a protected child is refused
        assert!(protected_descendant(&format!("{HOME}/Library"), HOME, &fs).is_some());
        assert!(protected_descendant(&format!("{HOME}/Projects"), HOME, &fs).is_none());
    }

    #[test]
    fn execution_is_truthful_across_modes_dedup_and_failures() {
        let mut fs = fs();
        fs.file(&format!("{HOME}/Projects/web/node_modules/b"), BLOCK, 40);
        let used = fs.volume_for("/").unwrap().used;
        let item = |p: &str, guard: Option<&'static str>| DeleteItem {
            path: p.into(),
            category: "project.artifacts".into(),
            estimate: 0,
            guard,
        };
        let nm = format!("{HOME}/Projects/web/node_modules");
        let plan = DeletePlan::new(
            "mbp",
            &format!("{HOME}/Projects"),
            vec![
                item(&nm, None),
                item(&nm, None),
                item(&format!("{nm}/a"), None),
                item(&format!("{HOME}/Projects/rs/target"), Some("Xcode")),
                item(&format!("{HOME}/Projects/gone"), None),
                item(&format!("{HOME}/Library/Keychains/login.keychain"), None),
                item(&format!("{HOME}/Library"), None),
                item(&format!("{HOME}/Projects/plain/build"), None),
            ],
            Mode::Trash,
            true,
        );
        let mut log = OpsLog {
            path: "~/.cache/holla/ops.log".into(),
            ..Default::default()
        };
        let probe = |name: &str| {
            if name == "Xcode" {
                ProcessObservation::Running
            } else {
                ProcessObservation::NotRunning
            }
        };
        let cx = ExecContext {
            home: HOME,
            os: Os::MacOs,
            now_secs: EPOCH_SECS,
            processes: &probe,
        };
        // a stale revision is refused before any effect
        assert!(execute(&plan, plan.revision + 1, &mut fs, &mut log, &cx).is_err());
        assert!(log.lines.is_empty());
        // dry run: exact would-results, no mutation, no process stop
        let r = execute(&plan, plan.revision, &mut fs, &mut log, &cx).unwrap();
        assert_eq!(r.count(Outcome::WouldRemove), 2);
        assert_eq!(r.count(Outcome::Skipped), 5);
        assert_eq!(r.count(Outcome::Failed), 1);
        assert!(fs.exists(&nm));
        assert_eq!(fs.volume_for("/").unwrap().used, used);
        assert_eq!(
            log.lines.len(),
            8,
            "one record per requested item, duplicates included"
        );
        assert!(
            log.lines[0].contains("\"outcome\":\"would_remove\"")
                && log.lines[0].contains("\"dry_run\":true")
        );
        assert!(log.lines[1].contains("duplicate"));
        assert!(log.lines[2].contains("covered by its ancestor"));
        assert!(log.lines[3].contains("Xcode is running"));
        assert!(log.lines[4].contains("\"outcome\":\"failed\""));
        assert!(
            log.lines[5].contains("covered by its ancestor"),
            "a selected ancestor owns the keychain item, and is itself refused"
        );
        assert!(log.lines[6].contains("contains protected"));
        assert!(r.summary().contains("2 would be removed"));
        let (details, over) = r.details();
        assert_eq!(details.len(), 6);
        assert_eq!(over, 0);
        // live Trash: bytes leave the tree, the volume keeps them until emptied
        let live = DeletePlan::new(
            "mbp",
            &format!("{HOME}/Projects"),
            vec![item(&nm, None)],
            Mode::Trash,
            false,
        );
        let r = execute(&live, live.revision, &mut fs, &mut log, &cx).unwrap();
        assert_eq!(r.count(Outcome::Trashed), 1);
        assert_eq!(r.bytes, 7 * BLOCK, "two files and two directory blocks");
        assert_eq!(r.freed_now, 0);
        assert!(!fs.exists(&nm));
        assert_eq!(
            fs.volume_for("/").unwrap().used,
            used,
            "same-volume Trash frees nothing yet"
        );
        // permanent frees now; a Trash failure never falls back to permanent
        fs.trash_broken = Some("Trash full".into());
        let t = DeletePlan::new(
            "mbp",
            HOME,
            vec![item(&format!("{HOME}/Projects/plain/build"), None)],
            Mode::Trash,
            false,
        );
        let r = execute(&t, t.revision, &mut fs, &mut log, &cx).unwrap();
        assert_eq!(r.count(Outcome::Failed), 1);
        assert!(fs.exists(&format!("{HOME}/Projects/plain/build")));
        let p = DeletePlan::new(
            "mbp",
            HOME,
            vec![item(&format!("{HOME}/Projects/plain/build"), None)],
            Mode::Permanent,
            false,
        );
        let r = execute(&p, p.revision, &mut fs, &mut log, &cx).unwrap();
        assert_eq!(r.count(Outcome::Removed), 1);
        assert_eq!(r.freed_now, 2 * BLOCK);
        assert_eq!(fs.volume_for("/").unwrap().used, used - 2 * BLOCK);
        // log failure is visible and never rolls back
        log.write_failure = Some("EROFS".into());
        let p2 = DeletePlan::new(
            "mbp",
            HOME,
            vec![item(&format!("{HOME}/Library/Logs/old.log"), None)],
            Mode::Permanent,
            false,
        );
        let r = execute(&p2, p2.revision, &mut fs, &mut log, &cx).unwrap();
        assert_eq!(r.count(Outcome::Removed), 1);
        assert_eq!(r.log_failures, 1);
        assert!(r.summary().contains("1 log write failures"));
        assert!(!fs.exists(&format!("{HOME}/Library/Logs/old.log")));
        // the phrase is bound to mode, count, root and host
        assert_eq!(
            p.phrase(),
            format!("PERMANENTLY DELETE 1 UNDER {HOME} ON mbp")
        );
        assert_ne!(
            live.revision,
            DeletePlan::new(
                "mbp",
                &format!("{HOME}/Projects"),
                vec![item(&nm, None)],
                Mode::Permanent,
                false
            )
            .revision
        );
        // JSON escaping
        let rec = LogRecord {
            timestamp_ms: 1,
            mode: Mode::Trash,
            dry_run: false,
            path: "/a \"b\"\nc".into(),
            size: 2,
            outcome: Outcome::Failed,
            error: Some("x\"y".into()),
        };
        assert_eq!(
            rec.json(),
            "{\"v\":1,\"timestamp_ms\":1,\"mode\":\"trash\",\"dry_run\":false,\"path\":\"/a \\\"b\\\"\\nc\",\"size\":2,\"outcome\":\"failed\",\"error\":\"x\\\"y\"}"
        );
    }

    #[test]
    fn size_cache_validates_ttl_mtime_schema_and_merges() {
        let fs = fs();
        let now = EPOCH_SECS;
        let tree = fs.scan(
            &format!("{HOME}/Projects"),
            &crate::sim::fs::ScanOptions {
                include_hidden: true,
                ..Default::default()
            },
        );
        let snapshot: Vec<(String, i64)> = fs
            .subtree(&format!("{HOME}/Projects"))
            .iter()
            .map(|n| (n.path.clone(), n.mtime))
            .collect();
        let mut c = SizeCache::default();
        c.record(&format!("{HOME}/Projects"), &tree, &snapshot, &fs, now);
        assert!(
            c.entries
                .iter()
                .any(|e| e.path == format!("{HOME}/Projects/rs/target")),
            "depth two is kept"
        );
        assert!(
            !c.entries
                .iter()
                .any(|e| e.path == format!("{HOME}/Projects/rs/target/debug")),
            "depth three is not"
        );
        let target = c
            .entries
            .iter()
            .find(|e| e.path == format!("{HOME}/Projects/rs/target"))
            .unwrap();
        assert!(c.hint(&target.path, target.mtime, now).is_some());
        assert!(
            c.hint(&target.path, target.mtime + 1, now).is_none(),
            "mtime must match exactly"
        );
        assert!(
            c.hint(&target.path, target.mtime, now + CACHE_TTL_SECS + 1)
                .is_none(),
            "seven-day TTL"
        );
        let text = c.serialize();
        let back = SizeCache::load(Some(&text), now);
        assert_eq!(back.entries, c.entries);
        assert!(
            SizeCache::load(Some(&text), now + CACHE_TTL_SECS + 1)
                .entries
                .is_empty()
        );
        assert!(
            SizeCache::load(Some("{\"v\":2,\"entries\":[]}"), now)
                .load_error
                .unwrap()
                .contains("schema 2")
        );
        assert!(SizeCache::load(Some("nope"), now).load_error.is_some());
        // a changed path is omitted after the scan
        let mut fs2 = fs.clone();
        fs2.touch(&format!("{HOME}/Projects/rs/target"), 0);
        let mut c2 = SizeCache::default();
        c2.record(&format!("{HOME}/Projects"), &tree, &snapshot, &fs2, now);
        assert!(
            !c2.entries
                .iter()
                .any(|e| e.path == format!("{HOME}/Projects/rs/target"))
        );
        // two writers merge by recency
        let mut a = SizeCache::default();
        a.entries.push(CacheEntry {
            path: "/x".into(),
            allocated: 1,
            mtime: 1,
            recorded_secs: now - 10,
        });
        let mut b = SizeCache::default();
        b.entries.push(CacheEntry {
            path: "/x".into(),
            allocated: 2,
            mtime: 1,
            recorded_secs: now - 5,
        });
        b.entries.push(CacheEntry {
            path: "/old".into(),
            allocated: 2,
            mtime: 1,
            recorded_secs: now - CACHE_TTL_SECS - 1,
        });
        a.merge(&b, now);
        assert_eq!(a.entries.len(), 1);
        assert_eq!(a.entries[0].allocated, 2);
    }
}
