//! Deterministic scenario contract: which fixture world the preview starts
//! in, how motion behaves, and which tick a paused capture shows.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Deterministic fixture world used by the preview.
pub enum Scenario {
    /// Zero instances, no saved Workspaces, no accounts: intro then an
    /// empty manager.
    FirstUse,
    /// An instance is already running: no intro, populated manager.
    Returning,
    /// Several providers, several accounts, mixed health.
    AccountsMixed,
    /// Straight into an active launch cockpit.
    LaunchRunning,
    /// A launch that fails at a stage.
    LaunchFailure,
    /// Attached Capsule with several tabs and nested panes.
    CapsuleMulti,
    /// Attached to the last running instance; exiting plays the outro.
    OutroLast,
    /// Long labels, missing daemon data, discovery failure, many rows.
    HardCases,
}

impl Scenario {
    /// All supported scenarios in stable capture order.
    pub const ALL: [Scenario; 8] = [
        Scenario::FirstUse,
        Scenario::Returning,
        Scenario::AccountsMixed,
        Scenario::LaunchRunning,
        Scenario::LaunchFailure,
        Scenario::CapsuleMulti,
        Scenario::OutroLast,
        Scenario::HardCases,
    ];

    /// Stable command-line and capture name.
    pub fn name(self) -> &'static str {
        match self {
            Scenario::FirstUse => "first-use",
            Scenario::Returning => "returning",
            Scenario::AccountsMixed => "accounts-mixed",
            Scenario::LaunchRunning => "launch-running",
            Scenario::LaunchFailure => "launch-failure",
            Scenario::CapsuleMulti => "capsule-multi",
            Scenario::OutroLast => "outro-last",
            Scenario::HardCases => "hard-cases",
        }
    }

    /// Parse a stable scenario name.
    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sc| sc.name() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// How the deterministic preview advances virtual time.
pub enum Motion {
    /// Tick-driven rituals and atmosphere.
    #[default]
    Full,
    /// One static meaningful boundary frame, then an immediate transition.
    Reduced,
    /// Ticks never advance; `--frame` selects the exact frame shown.
    Paused,
}

impl Motion {
    /// Parse a command-line motion name.
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Motion::Full),
            "reduced" => Some(Motion::Reduced),
            "paused" => Some(Motion::Paused),
            _ => None,
        }
    }

    /// Explicit CLI motion wins; otherwise `JACKIN_NO_MOTION` selects the
    /// reduced path.
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
        assert_eq!(Motion::resolve(None, true), Motion::Reduced);
        assert_eq!(Motion::resolve(Some(Motion::Full), true), Motion::Full);
        assert_eq!(Motion::resolve(None, false), Motion::Full);
    }
}
