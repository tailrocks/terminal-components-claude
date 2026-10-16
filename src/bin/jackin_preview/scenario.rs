//! Deterministic scenario contract: which fixture world the preview starts
//! in, how motion behaves, and which tick a paused capture shows.

use junie_tui::clock::SimClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sc| sc.name() == s)
    }
}

/// Fixed epoch for fixture timestamps: 2026-09-03 09:14:00 local (UTC+7).
pub const EPOCH_SECS: i64 = 1_788_401_640;
/// The fixture zone: UTC+7.
pub const UTC_OFFSET_SECS: i64 = 7 * 3600;
/// The fixture clock at its epoch, running. The preview never reads
/// wall-clock time: every duration, timestamp and animation frame derives
/// from virtual milliseconds that advance only with runtime ticks, so a
/// given `--scenario … --frame N` renders the same picture every time.
pub const CLOCK: SimClock = SimClock::new(EPOCH_SECS, UTC_OFFSET_SECS);

/// Tick delivery (`--motion`): the library's [`Motion`].
pub use junie_tui::motion::Motion;

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
