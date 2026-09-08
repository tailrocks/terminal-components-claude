//! disk: pressure plus progressive cleanup candidates. Each candidate has a
//! family, a size and a freshness fact — the reason a cleanup is safe.

/// Cleanup families (mole §families): generated artifacts, not user data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Family {
    CargoTarget,
    Gradle,
    NodeModules,
    DistBuild,
    PackageCache,
    Logs,
    Temp,
    DockerData,
}

impl Family {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Family::CargoTarget => "cargo target",
            Family::Gradle => "gradle build",
            Family::NodeModules => "node_modules",
            Family::DistBuild => "dist build",
            Family::PackageCache => "package cache",
            Family::Logs => "logs",
            Family::Temp => "temp data",
            Family::DockerData => "docker data",
        }
    }
}

/// How recently anything touched the candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Freshness {
    ActiveToday,
    InactiveDays(u32),
    Unknown,
}

impl Freshness {
    pub(crate) fn label(self) -> String {
        match self {
            Freshness::ActiveToday => "active today".to_owned(),
            Freshness::InactiveDays(d) => format!("inactive {d} days"),
            Freshness::Unknown => "activity unknown".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) path: String,
    pub(crate) family: Family,
    pub(crate) size_bytes: u64,
    pub(crate) freshness: Freshness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiskState {
    pub(crate) total_bytes: u64,
    pub(crate) used_bytes: u64,
    pub(crate) candidates: Vec<Candidate>,
}

impl DiskState {
    pub(crate) fn used_percent(&self) -> u32 {
        if self.total_bytes == 0 {
            return 0;
        }
        let percent = u128::from(self.used_bytes)
            .saturating_mul(100)
            .checked_div(u128::from(self.total_bytes))
            .unwrap_or(0);
        u32::try_from(percent).unwrap_or(u32::MAX)
    }

    /// All inspected generated artifacts, including policy-protected active data.
    pub(crate) fn inspected_bytes(&self) -> u64 {
        self.candidates.iter().map(|c| c.size_bytes).sum()
    }

    /// Bytes eligible for the cleanup policy shown by the plan.
    pub(crate) fn reclaimable_bytes(&self) -> u64 {
        self.candidates
            .iter()
            .filter(|c| c.freshness != Freshness::ActiveToday)
            .map(|c| c.size_bytes)
            .sum()
    }
}

/// Resolve fixture paths lexically without reading or mutating a filesystem.
/// Relative paths belong to the simulated cwd; `.`/`..` cannot disguise an
/// overlapping target. Root and traversal above root are not cleanup targets.
pub(crate) fn cleanup_path(cwd: &str, path: &str) -> Option<std::path::PathBuf> {
    use std::path::{Component, Path, PathBuf};
    if path.is_empty() {
        return None;
    }
    let expand = |value: &str| match value.strip_prefix("~/") {
        Some(rest) => format!("{}/{rest}", super::fixtures::HOME),
        None if value == "~" => super::fixtures::HOME.to_owned(),
        None => value.to_owned(),
    };
    let expanded = expand(path);
    let absolute = if Path::new(&expanded).is_absolute() {
        PathBuf::from(expanded)
    } else {
        PathBuf::from(expand(cwd)).join(expanded)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir => normalized.push("/"),
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Prefix(_) => return None,
        }
    }
    (normalized.is_absolute() && normalized != Path::new("/")).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maximum_capacity_percentage_does_not_overflow() {
        let disk = DiskState {
            total_bytes: u64::MAX,
            used_bytes: u64::MAX,
            candidates: vec![],
        };
        assert_eq!(disk.used_percent(), 100);
    }
    #[test]
    fn cleanup_paths_resolve_aliases_without_filesystem_access() {
        assert_eq!(cleanup_path("/work", "."), cleanup_path("/work", "/work"));
        assert_eq!(
            cleanup_path("/work", "/a/x/../b"),
            cleanup_path("/work", "/a/b")
        );
        assert_eq!(cleanup_path("/work", "/"), None);
        assert_eq!(cleanup_path("/work", "/../../outside"), None);
    }
}
