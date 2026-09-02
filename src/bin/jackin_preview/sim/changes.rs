//! Deterministic change sets for the Inspect Changes views. A simulated
//! instance never has a real repository, so the files an agent touched are
//! turned into realistic hunks from a fixed snippet library; the same
//! instance id and touched paths always produce the same diff.

pub use junie_tui::widgets::diff::{DiffFile as ChangedFile, DiffHunk as Hunk, DiffLine, DiffLineKind, DiffStatus};

/// Everything the inspector shows for one instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet {
    pub files: Vec<ChangedFile>,
    /// Commits ahead of the remote (shown as a row, not as files).
    pub unpushed: usize,
}

impl ChangeSet {
    pub fn additions(&self) -> usize {
        self.files.iter().map(|f| f.additions()).sum()
    }
    pub fn deletions(&self) -> usize {
        self.files.iter().map(|f| f.deletions()).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.unpushed == 0
    }
    /// `3 files · +41 −12`
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
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

fn ext(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or("")
}

fn c(s: &str) -> DiffLine {
    DiffLine::context(s)
}
fn a(s: &str) -> DiffLine {
    DiffLine::add(s)
}
fn r(s: &str) -> DiffLine {
    DiffLine::remove(s)
}

/// A modified file's hunks, chosen by extension; `k` varies the line
/// numbers so two files of the same kind do not look cloned.
fn hunks_for(path: &str, k: u64) -> Vec<Hunk> {
    let base = 8 + (k % 7) as usize * 5;
    match ext(path) {
        "rs" if path.contains("retry") => vec![
            Hunk {
                old_start: base,
                new_start: base,
                lines: vec![
                    c("use std::time::Duration;"),
                    a("use rand::Rng;"),
                    c(""),
                    c("pub struct RetryPolicy {"),
                    r("    pub attempts: u32,"),
                    a("    pub max_attempts: u32,"),
                    a("    pub base_delay: Duration,"),
                    c("}"),
                ],
            },
            Hunk {
                old_start: base + 30,
                new_start: base + 32,
                lines: vec![
                    c("    pub fn delay_for(&self, attempt: u32) -> Duration {"),
                    r("        self.base_delay"),
                    a("        let factor = 2u32.saturating_pow(attempt.min(5));"),
                    a("        let jitter = rand::thread_rng().gen_range(0..250);"),
                    a("        self.base_delay * factor + Duration::from_millis(jitter)"),
                    c("    }"),
                ],
            },
            Hunk {
                old_start: base + 58,
                new_start: base + 63,
                lines: vec![
                    c("        loop {"),
                    r("            if attempt >= self.attempts {"),
                    a("            if attempt >= self.max_attempts {"),
                    c("                return Err(SettleError::Exhausted(attempt));"),
                    c("            }"),
                ],
            },
        ],
        "rs" if path.contains("config") => vec![Hunk {
            old_start: base + 4,
            new_start: base + 4,
            lines: vec![
                c("#[derive(Debug, Clone, Deserialize)]"),
                c("pub struct SettlementConfig {"),
                c("    pub batch_size: usize,"),
                r("    pub retry_attempts: u32,"),
                a("    pub retry_max_attempts: u32,"),
                a("    #[serde(default = \"default_backoff_ms\")]"),
                a("    pub retry_backoff_ms: u64,"),
                c("}"),
                a(""),
                a("fn default_backoff_ms() -> u64 {"),
                a("    200"),
                a("}"),
            ],
        }],
        "rs" => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![
                c("pub mod config;"),
                c("pub mod retry;"),
                a("pub mod backoff;"),
                c(""),
                r("pub use retry::RetryPolicy;"),
                a("pub use retry::{RetryPolicy, SettleError};"),
            ],
        }],
        "toml" => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![
                c("[dependencies]"),
                c("serde = { version = \"1\", features = [\"derive\"] }"),
                a("rand = \"0.8\""),
                r("tokio = { version = \"1.36\", features = [\"full\"] }"),
                a("tokio = { version = \"1.38\", features = [\"full\"] }"),
            ],
        }],
        "md" => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![
                c("## Retries"),
                c(""),
                r("Failed batches are retried three times immediately."),
                a("Failed batches back off exponentially (200 ms base, 5 attempts,"),
                a("jitter up to 250 ms) before the batch is marked exhausted."),
            ],
        }],
        "tsx" | "ts" => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![
                c("export function SkeletonRow({ columns }: Props) {"),
                r("  return <tr className=\"skeleton\">{columns.map(() => <td />)}</tr>;"),
                a("  return ("),
                a("    <tr className=\"skeleton\" aria-busy=\"true\">"),
                a("      {columns.map((c) => <td key={c.id} style={{ width: c.width }} />)}"),
                a("    </tr>"),
                a("  );"),
                c("}"),
            ],
        }],
        "yml" | "yaml" => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![
                c("      - name: Build release"),
                r("        run: cargo build --release"),
                a("        run: cargo build --release --locked"),
                a("      - name: Verify signatures"),
                a("        run: ./scripts/verify-signatures.sh"),
            ],
        }],
        "tf" => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![
                c("resource \"google_container_node_pool\" \"primary\" {"),
                r("  node_count = 3"),
                a("  node_count = 5"),
                c("  autoscaling {"),
                r("    max_node_count = 6"),
                a("    max_node_count = 10"),
                c("  }"),
            ],
        }],
        _ => vec![Hunk {
            old_start: base,
            new_start: base,
            lines: vec![c("# settings"), r("timeout = 30"), a("timeout = 45")],
        }],
    }
}

fn added_file() -> ChangedFile {
    ChangedFile {
        path: "src/settlement/backoff.rs".into(),
        status: DiffStatus::Added,
        hunks: vec![Hunk {
            old_start: 0,
            new_start: 1,
            lines: vec![
                a("//! Exponential backoff with bounded jitter."),
                a(""),
                a("use std::time::Duration;"),
                a(""),
                a("pub struct Backoff {"),
                a("    base: Duration,"),
                a("    cap: u32,"),
                a("}"),
                a(""),
                a("impl Backoff {"),
                a("    pub fn exponential(base: Duration, cap: u32) -> Self {"),
                a("        Self { base, cap }"),
                a("    }"),
                a("}"),
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
                r("# Legacy retry notes"),
                r(""),
                r("Retries happen inline, three times, with no delay."),
                r("Superseded by the backoff policy in `src/settlement/retry.rs`."),
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
                c("#[test]"),
                r("fn retries_three_times() {"),
                a("fn backs_off_and_caps_at_five() {"),
                c("    let policy = RetryPolicy::default();"),
                r("    assert_eq!(policy.attempts, 3);"),
                a("    assert_eq!(policy.max_attempts, 5);"),
                a("    assert!(policy.delay_for(4) > policy.delay_for(1));"),
                c("}"),
            ],
        }],
    }
}

/// The change set of an instance. Touched paths become modified files;
/// when the instance's uncommitted count exceeds them, an added file, a
/// deleted file and a rename fill the gap in that order.
pub fn changes_for(instance_id: &str, touched: &[String], uncommitted: usize, unpushed: usize) -> ChangeSet {
    let k = seed(instance_id);
    let mut files: Vec<ChangedFile> = vec![];
    let mut seen = std::collections::BTreeSet::new();
    for (i, path) in touched.iter().enumerate() {
        if !seen.insert(path.clone()) {
            continue;
        }
        files.push(ChangedFile {
            path: path.clone(),
            status: DiffStatus::Modified,
            hunks: hunks_for(path, k.wrapping_add(i as u64)),
        });
    }
    let extras = [added_file, deleted_file, renamed_file];
    let mut e = 0;
    while files.len() < uncommitted && e < extras.len() {
        let f = extras[e]();
        if seen.insert(f.path.clone()) {
            files.push(f);
        }
        e += 1;
    }
    ChangeSet { files, unpushed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_realistic() {
        let touched = vec!["src/settlement/retry.rs".to_owned(), "src/settlement/mod.rs".to_owned()];
        let a1 = changes_for("jk-7f3a", &touched, 5, 1);
        let a2 = changes_for("jk-7f3a", &touched, 5, 1);
        assert_eq!(a1, a2);
        assert_eq!(a1.files.len(), 5);
        assert_eq!(a1.files[0].status, DiffStatus::Modified);
        assert_eq!(a1.files[0].hunks.len(), 3);
        assert!(a1.files[0].additions() > a1.files[0].deletions());
        assert!(matches!(a1.files[2].status, DiffStatus::Added));
        assert!(matches!(a1.files[3].status, DiffStatus::Deleted));
        assert!(matches!(a1.files[4].status, DiffStatus::Renamed { .. }));
        assert_eq!(a1.unpushed, 1);
        assert!(a1.summary().starts_with("5 files · +"));
        // a different instance shifts line numbers but keeps the shape
        let b = changes_for("jk-9b02", &touched, 5, 0);
        assert_eq!(b.files.len(), 5);
        assert_eq!(b.files[0].hunks.len(), 3);
    }

    #[test]
    fn fewer_uncommitted_than_touched_keeps_every_touched_file() {
        let touched = vec!["a.rs".to_owned(), "b.toml".to_owned(), "a.rs".to_owned()];
        let s = changes_for("x", &touched, 1, 0);
        assert_eq!(s.files.len(), 2, "duplicates collapse, touched files always show");
        assert!(changes_for("x", &[], 0, 0).is_empty());
    }

    #[test]
    fn no_secret_shaped_content() {
        let touched = ["src/settlement/retry.rs", "Cargo.toml", "README.md", "x.tsx", "ci.yml", "main.tf", "other.txt"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let s = changes_for("jk-5e5e", &touched, 12, 2);
        for f in &s.files {
            for h in &f.hunks {
                for l in &h.lines {
                    let t = l.text.to_lowercase();
                    assert!(!t.contains("sk-") && !t.contains("api_key") && !t.contains("secret"), "{t}");
                }
            }
        }
    }
}
