//! Deterministic change sets for the preview's inspect-changes surface.
//!
//! A simulated instance has no repository, so this module turns its touched
//! paths into stable fixture hunks.  The data model intentionally belongs to
//! the app instead of importing the retired diff widget.  A future screen can
//! project it into `tui_next::DiffSource` without exposing credentials or
//! coupling the simulation to a renderer.

use std::collections::BTreeSet;

use tui_next::{FgStep, Role};

/// The kind of one line in a unified diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    /// Present on both sides.
    Context,
    /// Present only on the new side.
    Add,
    /// Present only on the old side.
    Remove,
}

/// One owned diff line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// The semantic line kind.
    pub kind: DiffLineKind,
    /// Source text without the diff marker.
    pub text: String,
}

impl DiffLine {
    /// Construct a context line.
    pub fn context(text: impl Into<String>) -> Self {
        Self {
            kind: DiffLineKind::Context,
            text: text.into(),
        }
    }

    /// Construct an added line.
    pub fn add(text: impl Into<String>) -> Self {
        Self {
            kind: DiffLineKind::Add,
            text: text.into(),
        }
    }

    /// Construct a removed line.
    pub fn remove(text: impl Into<String>) -> Self {
        Self {
            kind: DiffLineKind::Remove,
            text: text.into(),
        }
    }

    /// Whether the line exists only on the new side.
    pub const fn is_addition(&self) -> bool {
        matches!(self.kind, DiffLineKind::Add)
    }

    /// Whether the line exists only on the old side.
    pub const fn is_deletion(&self) -> bool {
        matches!(self.kind, DiffLineKind::Remove)
    }

    /// The theme role a diff renderer should use for this line.
    pub const fn role(&self) -> Role {
        match self.kind {
            DiffLineKind::Context => Role::Fg(FgStep::Primary),
            DiffLineKind::Add => Role::Success,
            DiffLineKind::Remove => Role::Danger,
        }
    }
}

/// One hunk in a changed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// First old-side line number.
    pub old_start: usize,
    /// First new-side line number.
    pub new_start: usize,
    /// Hunk lines.
    pub lines: Vec<DiffLine>,
}

impl Hunk {
    /// Number of old-side lines represented by this hunk.
    pub fn old_len(&self) -> usize {
        self.lines.iter().filter(|line| !line.is_addition()).count()
    }

    /// Number of new-side lines represented by this hunk.
    pub fn new_len(&self) -> usize {
        self.lines.iter().filter(|line| !line.is_deletion()).count()
    }

    /// Conventional unified-diff header.
    pub fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start,
            self.old_len(),
            self.new_start,
            self.new_len()
        )
    }
}

/// File-level change status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffStatus {
    /// A newly created file.
    Added,
    /// An existing file changed in place.
    Modified,
    /// A file was removed.
    Deleted,
    /// A file changed path.
    Renamed {
        /// The old path.
        from: String,
    },
}

impl DiffStatus {
    /// One-letter status marker.
    pub const fn marker(&self) -> &'static str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Deleted => "D",
            Self::Renamed { .. } => "R",
        }
    }

    /// Human-readable status label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed { .. } => "renamed",
        }
    }

    /// Theme role for the status marker.
    pub const fn role(&self) -> Role {
        match self {
            Self::Added => Role::Success,
            Self::Modified => Role::Warning,
            Self::Deleted => Role::Danger,
            Self::Renamed { .. } => Role::Fg(FgStep::Secondary),
        }
    }
}

/// One changed file in an instance's deterministic diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    /// Current repository-relative path.
    pub path: String,
    /// File status.
    pub status: DiffStatus,
    /// Stable hunks for this path.
    pub hunks: Vec<Hunk>,
}

impl ChangedFile {
    /// Count added lines.
    pub fn additions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .filter(|line| line.is_addition())
            .count()
    }

    /// Count removed lines.
    pub fn deletions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .filter(|line| line.is_deletion())
            .count()
    }

    /// Compact file summary.
    pub fn summary(&self) -> String {
        let n = self.hunks.len();
        format!(
            "+{} −{} · {} hunk{}",
            self.additions(),
            self.deletions(),
            n,
            if n == 1 { "" } else { "s" }
        )
    }

    /// File header suitable for a diff list.
    pub fn header(&self) -> String {
        match &self.status {
            DiffStatus::Renamed { from } => {
                format!("{} {from} → {}", self.status.marker(), self.path)
            }
            _ => format!("{} {}", self.status.marker(), self.path),
        }
    }
}

/// Everything the inspector shows for one instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet {
    /// Changed files, in stable input order with duplicates removed.
    pub files: Vec<ChangedFile>,
    /// Commits ahead of the remote, shown separately from file rows.
    pub unpushed: usize,
}

impl ChangeSet {
    /// Count added lines across all files.
    pub fn additions(&self) -> usize {
        self.files.iter().map(ChangedFile::additions).sum()
    }

    /// Count removed lines across all files.
    pub fn deletions(&self) -> usize {
        self.files.iter().map(ChangedFile::deletions).sum()
    }

    /// Whether there is no file or unpushed commit to inspect.
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty() && self.unpushed == 0
    }

    /// Compact summary such as `3 files · +41 −12`.
    pub fn summary(&self) -> String {
        let n = self.files.len();
        format!(
            "{n} file{} · +{} −{}",
            if n == 1 { "" } else { "s" },
            self.additions(),
            self.deletions()
        )
    }
}

fn seed(instance_id: &str) -> u64 {
    instance_id
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

fn ext(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or_default()
}

fn context(text: &str) -> DiffLine {
    DiffLine::context(text)
}

fn added(text: &str) -> DiffLine {
    DiffLine::add(text)
}

fn removed(text: &str) -> DiffLine {
    DiffLine::remove(text)
}

/// Pick realistic but secret-free hunks from a fixed snippet library.
fn hunks_for(path: &str, key: u64) -> Vec<Hunk> {
    let base = 8 + (key % 7) as usize * 5;
    let lines = match ext(path) {
        "rs" if path.contains("retry") => vec![
            context("use std::time::Duration;"),
            added("use rand::Rng;"),
            context(""),
            context("pub struct RetryPolicy {"),
            removed("    pub attempts: u32,"),
            added("    pub max_attempts: u32,"),
            added("    pub base_delay: Duration,"),
            context("}"),
        ],
        "rs" if path.contains("config") => vec![
            context("#[derive(Debug, Clone, Deserialize)]"),
            context("pub struct SettlementConfig {"),
            context("    pub batch_size: usize,"),
            removed("    pub retry_attempts: u32,"),
            added("    pub retry_max_attempts: u32,"),
            added("    pub retry_backoff_ms: u64,"),
            context("}"),
        ],
        "rs" => vec![
            context("pub mod config;"),
            context("pub mod retry;"),
            added("pub mod backoff;"),
            context(""),
            removed("pub use retry::RetryPolicy;"),
            added("pub use retry::{RetryPolicy, SettleError};"),
        ],
        "toml" => vec![
            context("[dependencies]"),
            context("serde = { version = \"1\", features = [\"derive\"] }"),
            added("rand = \"0.8\""),
            removed("tokio = { version = \"1.36\", features = [\"full\"] }"),
            added("tokio = { version = \"1.38\", features = [\"full\"] }"),
        ],
        "md" => vec![
            context("## Retries"),
            context(""),
            removed("Failed batches are retried three times immediately."),
            added("Failed batches back off exponentially before exhaustion."),
        ],
        "tsx" | "ts" => vec![
            context("export function SkeletonRow({ columns }: Props) {"),
            removed("  return <tr className=\"skeleton\">{columns.map(() => <td />)}</tr>;"),
            added("  return <tr className=\"skeleton\" aria-busy=\"true\">;"),
            added("}"),
        ],
        "yml" | "yaml" => vec![
            context("      - name: Build release"),
            removed("        run: cargo build --release"),
            added("        run: cargo build --release --locked"),
            added("      - name: Verify signatures"),
        ],
        "tf" => vec![
            context("resource \"google_container_node_pool\" \"primary\" {"),
            removed("  node_count = 3"),
            added("  node_count = 5"),
            context("}"),
        ],
        _ => vec![
            context("# settings"),
            removed("timeout = 30"),
            added("timeout = 45"),
        ],
    };
    vec![Hunk {
        old_start: base,
        new_start: base,
        lines,
    }]
}

fn added_file() -> ChangedFile {
    ChangedFile {
        path: "src/settlement/backoff.rs".into(),
        status: DiffStatus::Added,
        hunks: vec![Hunk {
            old_start: 0,
            new_start: 1,
            lines: vec![
                added("//! Exponential backoff with bounded jitter."),
                added(""),
                added("pub struct Backoff {"),
                added("    base_ms: u64,"),
                added("    cap: u32,"),
                added("}"),
            ],
        }],
    }
}

fn deleted_file() -> ChangedFile {
    ChangedFile {
        path: "docs/legacy-retry.md".into(),
        status: DiffStatus::Deleted,
        hunks: vec![Hunk {
            old_start: 1,
            new_start: 0,
            lines: vec![
                removed("# Legacy retry notes"),
                removed(""),
                removed("Retries happen inline, three times, with no delay."),
                removed("Superseded by the backoff policy."),
            ],
        }],
    }
}

fn renamed_file() -> ChangedFile {
    ChangedFile {
        path: "tests/settlement_retry.rs".into(),
        status: DiffStatus::Renamed {
            from: "tests/retry.rs".into(),
        },
        hunks: vec![Hunk {
            old_start: 3,
            new_start: 3,
            lines: vec![
                context("#[test]"),
                removed("fn retries_three_times() {"),
                added("fn backs_off_and_caps_at_five() {"),
                context("    let policy = RetryPolicy::default();"),
                removed("    assert_eq!(policy.attempts, 3);"),
                added("    assert_eq!(policy.max_attempts, 5);"),
                context("}"),
            ],
        }],
    }
}

/// Build the deterministic change set for one simulated instance.
pub fn changes_for(
    instance_id: &str,
    touched: &[String],
    uncommitted: usize,
    unpushed: usize,
) -> ChangeSet {
    let key = seed(instance_id);
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, path) in touched.iter().enumerate() {
        if !seen.insert(path.clone()) {
            continue;
        }
        files.push(ChangedFile {
            path: path.clone(),
            status: DiffStatus::Modified,
            hunks: hunks_for(path, key.wrapping_add(index as u64)),
        });
    }
    let extras = [added_file, deleted_file, renamed_file];
    for make in extras {
        if files.len() >= uncommitted {
            break;
        }
        let file = make();
        if seen.insert(file.path.clone()) {
            files.push(file);
        }
    }
    ChangeSet { files, unpushed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_realistic() {
        let touched = vec![
            "src/settlement/retry.rs".to_owned(),
            "src/settlement/mod.rs".to_owned(),
        ];
        let first = changes_for("jk-7f3a", &touched, 5, 1);
        let second = changes_for("jk-7f3a", &touched, 5, 1);
        assert_eq!(first, second);
        assert_eq!(first.files.len(), 5);
        assert_eq!(first.files[0].status, DiffStatus::Modified);
        assert!(first.files[0].additions() > first.files[0].deletions());
        assert!(matches!(first.files[2].status, DiffStatus::Added));
        assert!(matches!(first.files[3].status, DiffStatus::Deleted));
        assert!(matches!(first.files[4].status, DiffStatus::Renamed { .. }));
        assert_eq!(first.unpushed, 1);
        assert!(first.summary().starts_with("5 files · +"));
    }

    #[test]
    fn duplicate_paths_collapse_but_touched_paths_win() {
        let touched = vec!["a.rs".to_owned(), "b.toml".to_owned(), "a.rs".to_owned()];
        let set = changes_for("x", &touched, 1, 0);
        assert_eq!(set.files.len(), 2);
        assert!(changes_for("x", &[], 0, 0).is_empty());
    }

    #[test]
    fn fixture_lines_never_contain_secret_shapes() {
        let touched = [
            "src/settlement/retry.rs",
            "Cargo.toml",
            "README.md",
            "x.tsx",
            "ci.yml",
            "main.tf",
            "other.txt",
        ]
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<Vec<_>>();
        let set = changes_for("jk-5e5e", &touched, 12, 2);
        for file in &set.files {
            for hunk in &file.hunks {
                for line in &hunk.lines {
                    let text = line.text.to_lowercase();
                    assert!(!text.contains("sk-") && !text.contains("api_key"));
                    assert!(!text.contains("secret"));
                }
            }
        }
    }
}
