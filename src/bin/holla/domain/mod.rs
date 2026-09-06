//! Typed fixture model for the simulated world: hosts, places, projects,
//! tools, containers, activities. Plain data; nothing here spawns a process.

// Fixture contract lands whole; P2+ renders into the parts P1 only seeds.
#[allow(dead_code)]
pub mod action;
#[allow(dead_code)]
pub mod activity;
#[allow(dead_code)]
pub mod debian;
#[allow(dead_code)]
pub mod disk;
#[allow(dead_code)]
pub mod docker;
pub mod fixtures;
#[allow(dead_code)]
pub mod git;
#[allow(dead_code)]
pub mod github;
pub mod host;
#[allow(dead_code)]
pub mod mise;
#[allow(dead_code)]
pub mod pg;
#[allow(dead_code)]
pub mod plan;
#[allow(dead_code)]
pub mod ranking;
#[allow(dead_code)]
pub mod ssh;

pub use host::Environment;

/// Human byte size per the design-system number grammar: `8.4 GB`, `900 MB`.
#[allow(dead_code)] // P2 renders sizes
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
