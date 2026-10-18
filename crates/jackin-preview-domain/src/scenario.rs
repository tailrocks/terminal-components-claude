#![doc = r" Deterministic scenario contract: which fixture world the preview starts"]
#![doc = r" in, how motion behaves, and which tick a paused capture shows."]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = r" Deterministic fixture world and journey selected by the preview."]
pub enum Scenario {
    #[doc = r" Zero instances, no saved Workspaces, no accounts: intro then an"]
    #[doc = r" empty manager."]
    FirstUse,
    #[doc = r" An instance is already running: no intro, populated manager."]
    Returning,
    #[doc = r" Several providers, several accounts, mixed health."]
    AccountsMixed,
    #[doc = r" Straight into an active launch cockpit."]
    LaunchRunning,
    #[doc = r" A launch that fails at a stage."]
    LaunchFailure,
    #[doc = r" Attached Capsule with several tabs and nested panes."]
    CapsuleMulti,
    #[doc = r" Attached to the last running instance; exiting plays the outro."]
    OutroLast,
    #[doc = r" Long labels, missing daemon data, discovery failure, many rows."]
    HardCases,
}

impl Scenario {
    #[doc = r" All scenarios in their stable CLI/display order."]
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

    #[doc = r" Return the stable CLI name for this scenario."]
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

    #[doc = r" Parse a stable CLI scenario name."]
    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sc| sc.name() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[doc = r" Deterministic animation policy for a preview scenario."]
pub enum Motion {
    #[doc = r" Tick-driven rituals and atmosphere."]
    #[default]
    Full,
    #[doc = r" One static meaningful boundary frame, then an immediate transition."]
    Reduced,
    #[doc = r" Ticks never advance; `--frame` selects the exact frame shown."]
    Paused,
}

impl Motion {
    #[doc = r" Parse a stable CLI motion name."]
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Motion::Full),
            "reduced" => Some(Motion::Reduced),
            "paused" => Some(Motion::Paused),
            _ => None,
        }
    }

    #[doc = r" Explicit CLI motion wins; otherwise `JACKIN_NO_MOTION` selects the"]
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
        assert_eq!(Motion::resolve(None, true), Motion::Reduced);
        assert_eq!(Motion::resolve(Some(Motion::Full), true), Motion::Full);
        assert_eq!(Motion::resolve(None, false), Motion::Full);
    }
}
