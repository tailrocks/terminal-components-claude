#![doc = r" Deterministic scenario contract: which fixture world the launcher starts"]
#![doc = r" in, how motion behaves, and which tick a paused capture shows."]

#[doc = r" A deterministic launcher fixture available to the CLI and capture harness."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    #[doc = r" Empty directory on a local dev host: nothing discovered yet."]
    FirstUse,
    #[doc = r" A Rust project with a dirty worktree and a branch behind upstream."]
    RustDirty,
    #[doc = r" Monorepository root with bounded child projects and mise tasks."]
    MonorepoRoot,
    #[doc = r" Inside `apps/frontend` of the monorepo; parent ecosystem visible."]
    MonorepoChild,
    #[doc = r" Docker host where the complete-cleanup journey is the point."]
    DockerCleanup,
    #[doc = r" Disk pressure; progressive analysis and cleanup candidates."]
    DiskCleanup,
    #[doc = r" Debian host with global mise tools; the upgrade-everything plan."]
    UpgradePlan,
    #[doc = r" Several named activities running at once (dev server, logs, tests)."]
    ActivitiesMulti,
    #[doc = r" SSH into a sensitive remote host; stronger identity and gates."]
    RemoteHost,
    #[doc = r" An action whose simulated execution fails."]
    LaunchFailure,
    #[doc = r" Long labels, missing data, discovery failure, narrow width."]
    HardCases,
}

impl Scenario {
    #[doc = r" Every supported deterministic simulation fixture in CLI inventory order."]
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

    #[doc = r" Stable CLI and capture-inventory name for this simulation fixture."]
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

#[doc = r" Animation and simulation time policy for the launcher."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[doc = r" Tick-driven spinners and progressive discovery."]
    #[default]
    Full,
    #[doc = r" Static as soon as the fixture settles; ticks still advance time."]
    Reduced,
    #[doc = r" Ticks never advance; `--frame` selects the exact fixture tick shown."]
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

    #[doc = r" Explicit CLI motion wins; otherwise `HOLLA_NO_MOTION` selects the"]
    #[doc = r" reduced path."]
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
