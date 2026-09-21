//! Production construction through `App::for_scenario` using oracle ticks.

use holla_app::{App, Motion};

use crate::clock::tick_to_ms;
use crate::worlds::{intern_world, production_scenario};

/// Why an oracle world cannot be constructed from current production.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnavailableReason {
    /// Name is in the oracle 34 but not in production `Scenario`.
    MissingProductionWorld,
    /// Name is not an oracle world.
    UnknownWorld,
}

/// Result of observing production construction for one oracle world/tick.
#[derive(Debug)]
pub enum Construction {
    /// Production constructed the named world.
    Ready {
        /// Interned oracle world name.
        world: &'static str,
        /// Motion policy restored after construction.
        motion: Motion,
        /// Oracle tick ordinal.
        tick: u64,
        /// Millisecond argument passed to `App::for_scenario`.
        frame_ms: u64,
        /// Production application.
        app: Box<App>,
    },
    /// Production cannot construct this oracle world. No substitute is used.
    Unavailable {
        /// Requested world name.
        world: String,
        /// Oracle tick ordinal that would have been applied.
        tick: u64,
        /// Honest unavailability cause.
        reason: UnavailableReason,
    },
}

impl Construction {
    /// Virtual milliseconds observed on a successful construction.
    #[must_use]
    pub fn now_ms(&self) -> Option<i64> {
        match self {
            Self::Ready { app, .. } => Some(app.fixture_time_ms()),
            Self::Unavailable { .. } => None,
        }
    }

    /// Successful production app, if any.
    #[must_use]
    pub fn app(&self) -> Option<&App> {
        match self {
            Self::Ready { app, .. } => Some(app.as_ref()),
            Self::Unavailable { .. } => None,
        }
    }

    /// Take the production app.
    #[must_use]
    pub fn into_app(self) -> Option<App> {
        match self {
            Self::Ready { app, .. } => Some(*app),
            Self::Unavailable { .. } => None,
        }
    }

    /// Whether construction used a production world without substitution.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

/// Construct through production `App::for_scenario` with oracle tick meaning.
///
/// `tick` is the oracle frame ordinal. The adapter converts it to milliseconds
/// (`tick * 80`) before calling `App::for_scenario`. It never passes the tick
/// ordinal as if it were already milliseconds.
#[must_use]
pub fn construct(world: &str, motion: Motion, tick: u64) -> Construction {
    let Some(oracle_world) = intern_world(world) else {
        return Construction::Unavailable {
            world: world.to_owned(),
            tick,
            reason: UnavailableReason::UnknownWorld,
        };
    };
    let Some(scenario) = production_scenario(oracle_world) else {
        return Construction::Unavailable {
            world: oracle_world.to_owned(),
            tick,
            reason: UnavailableReason::MissingProductionWorld,
        };
    };
    let frame_ms = tick_to_ms(tick);
    let app = Box::new(App::for_scenario(scenario, motion, frame_ms));
    Construction::Ready {
        world: oracle_world,
        motion,
        tick,
        frame_ms,
        app,
    }
}

/// HO-BASE default constructor: paused oracle tick 40.
#[must_use]
pub fn construct_ho_base(world: &str) -> Construction {
    construct(world, Motion::Paused, 40)
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use holla_app::{App, Motion, Scenario};

    #[test]
    fn ho_base_01_first_use_paused_tick_40_is_now_ms_3200() {
        let built = construct("first-use", Motion::Paused, 40);
        assert_eq!(built.now_ms(), Some(3_200));
        assert!(built.is_ready());
        let naive = App::for_scenario(Scenario::FirstUse, Motion::Paused, 40);
        assert_ne!(
            naive.fixture_time_ms(),
            3_200,
            "production seek(40) is milliseconds, not oracle tick 40"
        );
        let zero = construct("first-use", Motion::Paused, 0);
        assert_eq!(zero.now_ms(), Some(0));
        let one = construct("first-use", Motion::Paused, 1);
        assert_eq!(one.now_ms(), Some(80));
    }

    #[test]
    fn missing_parity_world_is_unavailable_not_substituted() {
        let built = construct("parity-discovery", Motion::Paused, 40);
        assert!(!built.is_ready());
        assert_eq!(built.now_ms(), None);
        assert!(matches!(
            built,
            Construction::Unavailable {
                reason: UnavailableReason::MissingProductionWorld,
                ..
            }
        ));
    }

    #[test]
    fn unknown_world_is_unavailable() {
        let built = construct("no-such-world", Motion::Paused, 40);
        assert!(matches!(
            built,
            Construction::Unavailable {
                reason: UnavailableReason::UnknownWorld,
                ..
            }
        ));
    }
}
