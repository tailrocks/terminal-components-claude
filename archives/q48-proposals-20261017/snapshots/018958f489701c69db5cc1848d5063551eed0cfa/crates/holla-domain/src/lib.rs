//! Typed model of everything Holla knows: context, results, plans,
//! activities, the stack fixtures, search and ranking, and the scenario
//! worlds.

pub mod action;
pub mod activity;
pub mod clock;
pub mod context;
pub mod custom;
pub mod digest;
pub mod effect;
pub mod exec;
pub mod manifest;
pub mod plan;
pub mod ranking;
pub mod scenario;
pub mod scripts;
pub mod stack;
pub mod usage;

pub use clock::{Clock, EPOCH_SECS};
pub use scenario::{Motion, Scenario};

/// Fixture home directory matching visual baseline oracle.
pub const HOME: &str = "/Users/alex";

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

pub fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}

pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
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
