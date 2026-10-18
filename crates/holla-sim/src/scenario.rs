#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    FirstUse,
    RustDirty,
    MonorepoRoot,
    MonorepoChild,
    DockerCleanup,
    DiskCleanup,
    UpgradePlan,
    ActivitiesMulti,
    RemoteHost,
    LaunchFailure,
    HardCases,
}

impl Scenario {
    pub const ALL: [Scenario; 11] = [
        Scenario::FirstUse,
        Scenario::RustDirty,
        Scenario::MonorepoRoot,
        Scenario::MonorepoChild,
        Scenario::DockerCleanup,
        Scenario::DiskCleanup,
        Scenario::UpgradePlan,
        Scenario::ActivitiesMulti,
        Scenario::RemoteHost,
        Scenario::LaunchFailure,
        Scenario::HardCases,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Scenario::FirstUse => "first-use",
            Scenario::RustDirty => "rust-dirty",
            Scenario::MonorepoRoot => "monorepo-root",
            Scenario::MonorepoChild => "monorepo-child",
            Scenario::DockerCleanup => "docker-cleanup",
            Scenario::DiskCleanup => "disk-cleanup",
            Scenario::UpgradePlan => "upgrade-plan",
            Scenario::ActivitiesMulti => "activities-multi",
            Scenario::RemoteHost => "remote-host",
            Scenario::LaunchFailure => "launch-failure",
            Scenario::HardCases => "hard-cases",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sc| sc.name() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Full,
    Reduced,
    Paused,
}

impl Motion {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Motion::Full),
            "reduced" => Some(Motion::Reduced),
            "paused" => Some(Motion::Paused),
            _ => None,
        }
    }

    pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion {
        match cli {
            Some(m) => m,
            None if no_motion_env => Motion::Reduced,
            None => Motion::Full,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for s in Scenario::ALL {
            assert_eq!(Scenario::from_name(s.name()), Some(s));
        }
        assert_eq!(Scenario::ALL.len(), 11);
        assert_eq!(Motion::resolve(None, true), Motion::Reduced);
        assert_eq!(Motion::resolve(Some(Motion::Full), true), Motion::Full);
        assert_eq!(Motion::resolve(None, false), Motion::Full);
    }
}
