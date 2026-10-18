#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Motion {
    #[doc = " Tick-driven state and decorative motion."]
    #[default]
    Full,

    #[doc = " State advances; decorative motion stays still."]
    Reduced,

    #[doc = " Ticks never advance; `--frame` selects the frame shown."]
    Paused,
}

impl Motion {
    pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion {
        match cli {
            Some(m) => m,
            None if no_motion_env => Motion::Reduced,
            None => Motion::Full,
        }
    }

    pub fn env_requests_reduced(var: &str) -> bool {
        std::env::var_os(var).is_some_and(|v| !v.is_empty() && v != "0")
    }

    pub fn from_cli_or_env(cli: Option<Motion>, var: &str) -> Motion {
        Motion::resolve(cli, Motion::env_requests_reduced(var))
    }

    pub fn ticks(self) -> bool {
        self != Motion::Paused
    }

    pub fn animates(self) -> bool {
        self == Motion::Full
    }

    pub fn is_paused(self) -> bool {
        self == Motion::Paused
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_motion_wins_over_the_environment() {
        assert_eq!(Motion::resolve(None, true), Motion::Reduced);
        assert_eq!(Motion::resolve(Some(Motion::Full), true), Motion::Full);
        assert_eq!(Motion::resolve(Some(Motion::Paused), false), Motion::Paused);
        assert_eq!(Motion::resolve(None, false), Motion::Full);
    }

    #[test]
    fn levels_say_what_moves() {
        assert!(Motion::Full.ticks() && Motion::Full.animates());
        assert!(Motion::Reduced.ticks() && !Motion::Reduced.animates());
        assert!(!Motion::Paused.ticks() && !Motion::Paused.animates());
        assert!(Motion::Paused.is_paused());
    }

    #[test]
    fn an_unset_or_zero_variable_keeps_full_motion() {
        assert!(!Motion::env_requests_reduced(
            "JUNIE_TUI_TEST_MOTION_VARIABLE_THAT_IS_NEVER_SET"
        ));
    }
}
