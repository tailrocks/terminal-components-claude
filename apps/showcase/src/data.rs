//! Deterministic data used by showcase pages.

use tui_next::{ItemKey, TreeNode};

/// A row in the collection demos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskRow {
    /// Stable row id.
    pub id: u32,
    /// Task title.
    pub name: &'static str,
    /// Assignee.
    pub owner: &'static str,
    /// Current status.
    pub status: TaskStatus,
}

/// A deterministic task status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaskStatus {
    /// Waiting to start.
    Queued,
    /// Currently running.
    Running,
    /// Completed successfully.
    Done,
    /// Failed.
    Failed,
}

/// Sample tasks, intentionally static so frame captures are stable.
pub(crate) const TASKS: &[TaskRow] = &[
    TaskRow {
        id: 1,
        name: "Add rate limiting",
        owner: "mira",
        status: TaskStatus::Done,
    },
    TaskRow {
        id: 2,
        name: "Migrate sessions",
        owner: "jonas",
        status: TaskStatus::Running,
    },
    TaskRow {
        id: 3,
        name: "Fix checkout test",
        owner: "ana",
        status: TaskStatus::Failed,
    },
    TaskRow {
        id: 4,
        name: "Write release notes",
        owner: "mira",
        status: TaskStatus::Queued,
    },
    TaskRow {
        id: 5,
        name: "Refresh dependency graph",
        owner: "kai",
        status: TaskStatus::Running,
    },
];

/// Programming languages for list and picker examples.
pub(crate) const LANGUAGES: &[&str] = &[
    "Rust",
    "TypeScript",
    "Python",
    "Kotlin",
    "Go",
    "Java",
    "Swift",
    "C#",
    "Ruby",
    "Zig",
];

/// A keyed tree fixture. Labels are rendered by the row closure, while the
/// structure is owned by `TreeNode`, matching the public component contract.
pub(crate) const TREE: &[TreeNode] = &[
    TreeNode::parent(0).keyed(ItemKey::Num(1)),
    TreeNode::parent(1).keyed(ItemKey::Num(2)),
    TreeNode::leaf(2).keyed(ItemKey::Num(3)),
    TreeNode::leaf(2).keyed(ItemKey::Num(4)),
    TreeNode::parent(1).keyed(ItemKey::Num(5)),
    TreeNode::leaf(2).keyed(ItemKey::Num(6)),
    TreeNode::leaf(0).keyed(ItemKey::Num(7)),
];

/// Sample code for the editor and viewport pages.
pub(crate) const CODE: &str = "fn main() {\n    println!(\"hello from showcase\");\n}\n";
