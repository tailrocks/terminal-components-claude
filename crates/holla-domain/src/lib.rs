//! Typed fixture model for the simulated world: hosts, places, projects,
//! tools, containers, activities. Plain data; nothing here spawns a process.

pub mod accounting;
pub mod action;
pub mod clock;
pub mod debian;
pub mod disk;
pub mod docker;
pub mod effect;
pub mod git;
pub mod github;
pub mod host;
pub mod mise;
pub mod pg;
pub mod scenario;
pub mod ssh;

pub use clock::Clock;
pub use host::Environment;
pub use scenario::{Motion, Scenario};

/// Fixture home directory. Display paths use `~`; typed confirmation
/// phrases expand through this, e.g. `~/work/scratch` -> `/home/dev/work/scratch`.
pub const HOME: &str = "/home/dev";

pub fn expand_home(path: &str) -> String {
    path.replacen('~', HOME, 1)
}

/// Human byte size per the design-system number grammar: `8.4 GB`, `900 MB`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_grammar() {
        assert_eq!(human_bytes(9_007_199_254), "8.4 GB");
        assert_eq!(human_bytes(943_718_400), "900 MB");
        assert_eq!(human_bytes(512), "512 B");
    }
}
