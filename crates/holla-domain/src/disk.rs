#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
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
    pub fn label(self) -> &'static str {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    ActiveToday,
    InactiveDays(u32),
    Unknown,
}

impl Freshness {
    pub fn label(self) -> String {
        match self {
            Freshness::ActiveToday => "active today".to_owned(),
            Freshness::InactiveDays(d) => format!("inactive {d} days"),
            Freshness::Unknown => "activity unknown".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: String,
    pub family: Family,
    pub size_bytes: u64,
    pub freshness: Freshness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskState {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub candidates: Vec<Candidate>,
}

impl DiskState {
    pub fn validate(&self, cwd: &str) -> Result<(), crate::accounting::InventoryError> {
        use crate::accounting::InventoryError;
        if self.inspected_bytes()? > self.used_bytes || self.used_bytes > self.total_bytes {
            return Err(InventoryError::InvalidCapacity);
        }
        let mut paths = Vec::new();
        for candidate in &self.candidates {
            let path = cleanup_path(cwd, &candidate.path).ok_or(InventoryError::InvalidTarget)?;
            if paths.iter().any(|prior: &std::path::PathBuf| {
                prior.starts_with(&path) || path.starts_with(prior)
            }) {
                return Err(InventoryError::OverlappingTargets);
            }
            paths.push(path);
        }
        Ok(())
    }

    pub fn used_percent(&self) -> u32 {
        if self.total_bytes == 0 {
            return 0;
        }
        let percent = u128::from(self.used_bytes)
            .saturating_mul(100)
            .checked_div(u128::from(self.total_bytes))
            .unwrap_or(0);
        u32::try_from(percent).unwrap_or(u32::MAX)
    }

    pub fn inspected_bytes(&self) -> Result<u64, crate::accounting::InventoryError> {
        crate::accounting::bytes(self.candidates.iter().map(|c| c.size_bytes))
    }

    pub fn reclaimable_bytes(&self) -> Result<u64, crate::accounting::InventoryError> {
        crate::accounting::bytes(
            self.candidates
                .iter()
                .filter(|candidate| candidate.freshness != Freshness::ActiveToday)
                .map(|candidate| candidate.size_bytes),
        )
    }
}

pub fn cleanup_path(cwd: &str, path: &str) -> Option<std::path::PathBuf> {
    use std::path::{Component, Path, PathBuf};
    if path.is_empty() {
        return None;
    }
    let expand = |value: &str| match value.strip_prefix("~/") {
        Some(rest) => format!("{}/{rest}", crate::HOME),
        None if value == "~" => crate::HOME.to_owned(),
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
