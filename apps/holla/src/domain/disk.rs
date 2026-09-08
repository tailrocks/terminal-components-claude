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
        ((self.used_bytes * 100) / self.total_bytes) as u32
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
