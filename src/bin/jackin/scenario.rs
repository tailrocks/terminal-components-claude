use junie_tui::clock::SimClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    FirstUse,

    Returning,

    AccountsMixed,

    LaunchRunning,

    LaunchFailure,

    CapsuleMulti,

    OutroLast,

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

pub const EPOCH_SECS: i64 = 1_788_401_640;

pub const UTC_OFFSET_SECS: i64 = 7 * 3600;

pub const CLOCK: SimClock = SimClock::new(EPOCH_SECS, UTC_OFFSET_SECS);

pub use junie_tui::motion::Motion;
