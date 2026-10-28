pub mod accounting;
pub mod action;
pub mod activity;
pub mod args;
pub mod debian;
pub mod disk;
pub mod docker;
pub mod effect;
pub mod git;
pub mod github;
pub mod host;
pub mod mise;
pub mod pg;
pub mod ssh;

pub use host::Environment;

pub const HOME: &str = "/home/dev";

pub fn human_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = KB * 1_024;
    const GB: u64 = MB * 1_024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{} KB", bytes / KB)
    } else {
        format!("{bytes} B")
    }
}
