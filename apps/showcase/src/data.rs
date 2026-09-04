//! Demo fixtures carried over from the legacy showcase.

use tui_next::{ItemKey, TreeNode};

/// One deterministic task row shared by the table and task-runner pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TaskRow {
    /// Stable row identity.
    pub id: u32,
    /// Task title.
    pub name: &'static str,
    /// Assignee.
    pub owner: &'static str,
    /// Status.
    pub status: TaskStatus,
    /// Source branch.
    pub branch: &'static str,
    /// Number of changed files.
    pub changes: u32,
    /// Duration in seconds.
    pub duration_s: u32,
}

/// Domain status used in the table fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TaskStatus {
    /// Waiting to start.
    Queued,
    /// Currently running.
    Running,
    /// Completed.
    Done,
    /// Failed.
    Failed,
    /// Paused.
    Paused,
}

/// The exact legacy 24-row table fixture.
pub(crate) const TASKS: &[TaskRow] = &[
    TaskRow { id: 1040, name: "Add rate limiting to auth endpoints", owner: "mira", status: TaskStatus::Done, branch: "feat/rate-limit", changes: 14, duration_s: 412 },
    TaskRow { id: 1041, name: "Migrate sessions table to UUID keys", owner: "jonas", status: TaskStatus::Running, branch: "chore/uuid-sessions", changes: 31, duration_s: 96 },
    TaskRow { id: 1042, name: "Fix flaky checkout integration test", owner: "ana", status: TaskStatus::Failed, branch: "fix/checkout-flake", changes: 3, duration_s: 58 },
    TaskRow { id: 1043, name: "Write release notes for 3.2", owner: "mira", status: TaskStatus::Queued, branch: "docs/release-3.2", changes: 0, duration_s: 0 },
    TaskRow { id: 1044, name: "Replace deprecated Vue mixins", owner: "kai", status: TaskStatus::Done, branch: "refactor/mixins", changes: 87, duration_s: 1330 },
    TaskRow { id: 1045, name: "Upgrade Postgres driver to 0.9", owner: "jonas", status: TaskStatus::Paused, branch: "chore/pg-driver", changes: 5, duration_s: 240 },
    TaskRow { id: 1046, name: "Extract billing service module", owner: "sofia", status: TaskStatus::Done, branch: "refactor/billing", changes: 52, duration_s: 908 },
    TaskRow { id: 1047, name: "Add OpenTelemetry tracing spans", owner: "kai", status: TaskStatus::Running, branch: "feat/otel", changes: 22, duration_s: 130 },
    TaskRow { id: 1048, name: "Remove legacy feature flags", owner: "ana", status: TaskStatus::Queued, branch: "chore/flags", changes: 0, duration_s: 0 },
    TaskRow { id: 1049, name: "Generate API client from OpenAPI", owner: "sofia", status: TaskStatus::Done, branch: "feat/api-client", changes: 118, duration_s: 2210 },
    TaskRow { id: 1050, name: "Harden CSP headers", owner: "mira", status: TaskStatus::Done, branch: "sec/csp", changes: 4, duration_s: 77 },
    TaskRow { id: 1051, name: "Speed up cold start of worker", owner: "jonas", status: TaskStatus::Failed, branch: "perf/worker-boot", changes: 9, duration_s: 601 },
    TaskRow { id: 1052, name: "Localize onboarding emails", owner: "kai", status: TaskStatus::Queued, branch: "feat/i18n-emails", changes: 0, duration_s: 0 },
    TaskRow { id: 1053, name: "Add pagination to audit log", owner: "ana", status: TaskStatus::Done, branch: "feat/audit-pages", changes: 16, duration_s: 344 },
    TaskRow { id: 1054, name: "Refactor retry helper into crate", owner: "sofia", status: TaskStatus::Running, branch: "refactor/retry", changes: 11, duration_s: 45 },
    TaskRow { id: 1055, name: "Rotate signing keys quarterly", owner: "mira", status: TaskStatus::Queued, branch: "sec/key-rotation", changes: 0, duration_s: 0 },
    TaskRow { id: 1056, name: "Fix timezone bug in scheduler", owner: "jonas", status: TaskStatus::Done, branch: "fix/tz-scheduler", changes: 7, duration_s: 188 },
    TaskRow { id: 1057, name: "Document webhook retry semantics", owner: "kai", status: TaskStatus::Done, branch: "docs/webhooks", changes: 2, duration_s: 65 },
    TaskRow { id: 1058, name: "Add dark mode to admin panel", owner: "ana", status: TaskStatus::Paused, branch: "feat/admin-dark", changes: 40, duration_s: 720 },
    TaskRow { id: 1059, name: "Bump minimum Node to 22", owner: "sofia", status: TaskStatus::Queued, branch: "chore/node-22", changes: 0, duration_s: 0 },
    TaskRow { id: 1060, name: "Cache dependency graph between runs", owner: "mira", status: TaskStatus::Running, branch: "perf/dep-cache", changes: 19, duration_s: 210 },
    TaskRow { id: 1061, name: "Clean up unused SQL views", owner: "jonas", status: TaskStatus::Done, branch: "chore/sql-views", changes: 12, duration_s: 155 },
    TaskRow { id: 1062, name: "Add health endpoint for gateway", owner: "kai", status: TaskStatus::Done, branch: "feat/health", changes: 3, duration_s: 42 },
    TaskRow { id: 1063, name: "Investigate memory growth in parser", owner: "ana", status: TaskStatus::Running, branch: "perf/parser-mem", changes: 6, duration_s: 380 },
];

/// The exact legacy language fixture used by single and multi lists.
pub(crate) const LANGUAGES: &[&str] = &[
    "Rust", "TypeScript", "Python", "Kotlin", "Go", "Java", "Swift", "C#", "Ruby", "Scala",
    "Elixir", "Haskell", "Zig", "Dart", "PHP", "C++", "Lua", "OCaml", "Clojure", "Erlang",
];

/// Stable pre-order project tree. Keys are unique identities, never derived
/// from depth, so expanding a branch cannot alias a sibling node.
pub(crate) const TREE: &[TreeNode] = &[
    TreeNode::parent(0).keyed(ItemKey::Num(1)),
    TreeNode::parent(1).keyed(ItemKey::Num(2)),
    TreeNode::leaf(2).keyed(ItemKey::Num(3)),
    TreeNode::leaf(2).keyed(ItemKey::Num(4)),
    TreeNode::leaf(2).keyed(ItemKey::Num(5)),
    TreeNode::parent(2).keyed(ItemKey::Num(6)),
    TreeNode::leaf(3).keyed(ItemKey::Num(7)),
    TreeNode::leaf(3).keyed(ItemKey::Num(8)),
    TreeNode::leaf(3).keyed(ItemKey::Num(9)),
    TreeNode::parent(1).keyed(ItemKey::Num(10)),
    TreeNode::leaf(2).keyed(ItemKey::Num(11)),
    TreeNode::leaf(2).keyed(ItemKey::Num(12)),
    TreeNode::leaf(2).keyed(ItemKey::Num(13)),
    TreeNode::parent(1).keyed(ItemKey::Num(14)),
    TreeNode::leaf(2).keyed(ItemKey::Num(15)),
    TreeNode::leaf(2).keyed(ItemKey::Num(16)),
    TreeNode::leaf(1).keyed(ItemKey::Num(17)),
    TreeNode::leaf(1).keyed(ItemKey::Num(18)),
    TreeNode::parent(0).keyed(ItemKey::Num(19)),
    TreeNode::leaf(1).keyed(ItemKey::Num(20)),
    TreeNode::leaf(1).keyed(ItemKey::Num(21)),
    TreeNode::parent(1).keyed(ItemKey::Num(22)),
    TreeNode::leaf(2).keyed(ItemKey::Num(23)),
    TreeNode::leaf(2).keyed(ItemKey::Num(24)),
    TreeNode::leaf(0).keyed(ItemKey::Num(25)),
    TreeNode::leaf(0).keyed(ItemKey::Num(26)),
];

/// Label and metadata for each tree key, in the same order as [`TREE`].
pub(crate) const TREE_LABELS: &[(&str, &str)] = &[
    ("src", "directory"), ("api", "directory"), ("auth.rs", "2.1 KB"),
    ("billing.rs", "6.4 KB"), ("mod.rs", "312 B"), ("webhooks", "directory"),
    ("dispatch.rs", "3.9 KB"), ("retry.rs", "1.7 KB"), ("mod.rs", "180 B"),
    ("db", "directory"), ("migrations.rs", "9.2 KB"), ("pool.rs", "1.1 KB"),
    ("schema.rs", "14.8 KB"), ("workers", "directory"), ("scheduler.rs", "4.6 KB"),
    ("mailer.rs", "2.8 KB"), ("config.rs", "1.9 KB"), ("lib.rs", "640 B"),
    ("tests", "directory"), ("checkout.rs", "5.3 KB"), ("auth_flow.rs", "3.0 KB"),
    ("fixtures", "directory"), ("users.json", "18 KB"), ("orders.json", "44 KB"),
    ("Cargo.toml", "1.4 KB"), ("README.md", "3.5 KB"),
];

/// Repeated terminal output used by the scrolling and terminal screens.
pub(crate) fn log_lines(n: usize) -> Vec<String> {
    let steps = [
        ("info", "Resolving workspace members"),
        ("info", "Fetching crates.io index"),
        ("info", "Compiling proc-macro2 v1.0.86"),
        ("info", "Compiling serde v1.0.210"),
        ("warn", "unused import: `std::fmt` in src/api/mod.rs:3"),
        ("info", "Compiling tokio v1.40.0"),
        ("info", "Running unittests src/lib.rs"),
        ("info", "test api::auth::tests::rejects_expired ... ok"),
        ("info", "test db::pool::tests::reuses_connections ... ok"),
        ("error", "test checkout::places_order ... FAILED"),
        ("info", "test workers::scheduler::tests::respects_timezone ... ok"),
        ("info", "Linking target/debug/deps/app-4f2c1b"),
    ];
    (0..n)
        .map(|i| {
            let (level, message) = steps[i % steps.len()];
            let seconds = i as f64 * 0.37;
            format!("{seconds:7.2}s  {level:<5}  {message}")
        })
        .collect()
}

/// Long explanatory prose for the scrolling screen.
pub(crate) const PROSE: &str = "Junie works through a task the way a careful engineer would: it reads the relevant code, forms a plan, makes focused changes, runs the tests, and reports back with a summary you can review before anything is merged.\n\nEach step is visible. You can pause, redirect, or take over at any point, and every change lands as an ordinary diff in your working tree.\n\nThe design system in this prototype exists so that the terminal version of that experience feels as deliberate as the web version: quiet surfaces, one accent, clear focus, and no decoration that does not carry information.\n\nScroll with the mouse wheel, PageUp/PageDown, or the arrow keys while this panel has focus. The scrollbar on the right shows where you are and how much remains.";

/// The exact long-list fixture used by the legacy scrolling page.
pub(crate) const SCROLL_ROWS: &[&str] = &[
+    "Row 001",
    "Row 002",
    "Row 003",
    "Row 004",
    "Row 005",
    "Row 006",
    "Row 007",
    "Row 008",
    "Row 009",
    "Row 010",
    "Row 011",
    "Row 012",
    "Row 013",
    "Row 014",
    "Row 015",
    "Row 016",
    "Row 017",
    "Row 018",
    "Row 019",
    "Row 020",
    "Row 021",
    "Row 022",
    "Row 023",
    "Row 024",
    "Row 025",
    "Row 026",
    "Row 027",
    "Row 028",
    "Row 029",
    "Row 030",
    "Row 031",
    "Row 032",
    "Row 033",
    "Row 034",
    "Row 035",
    "Row 036",
    "Row 037",
    "Row 038",
    "Row 039",
    "Row 040",
    "Row 041",
    "Row 042",
    "Row 043",
    "Row 044",
    "Row 045",
    "Row 046",
    "Row 047",
    "Row 048",
    "Row 049",
    "Row 050",
    "Row 051",
    "Row 052",
    "Row 053",
    "Row 054",
    "Row 055",
    "Row 056",
    "Row 057",
    "Row 058",
    "Row 059",
    "Row 060",
    "Row 061",
    "Row 062",
    "Row 063",
    "Row 064",
    "Row 065",
    "Row 066",
    "Row 067",
    "Row 068",
    "Row 069",
    "Row 070",
    "Row 071",
    "Row 072",
    "Row 073",
    "Row 074",
    "Row 075",
    "Row 076",
    "Row 077",
    "Row 078",
    "Row 079",
    "Row 080",
    "Row 081",
    "Row 082",
    "Row 083",
    "Row 084",
    "Row 085",
    "Row 086",
    "Row 087",
    "Row 088",
    "Row 089",
    "Row 090",
    "Row 091",
    "Row 092",
    "Row 093",
    "Row 094",
    "Row 095",
    "Row 096",
    "Row 097",
    "Row 098",
    "Row 099",
    "Row 100",
    "Row 101",
    "Row 102",
    "Row 103",
    "Row 104",
    "Row 105",
    "Row 106",
    "Row 107",
    "Row 108",
    "Row 109",
    "Row 110",
    "Row 111",
    "Row 112",
    "Row 113",
    "Row 114",
    "Row 115",
    "Row 116",
    "Row 117",
    "Row 118",
    "Row 119",
    "Row 120",
];

/// Small source document used by the code editor page.
pub(crate) const CODE: &str = "fn main() {\n    println!(\"hello from showcase\");\n}\n";
