//! Motion preference shared by every application: how much tick-driven
//! movement the interface shows.
//!
//! WCAG 2.2 SC 2.3.3 (Animation from Interactions) and SC 2.2.2 (Pause,
//! Stop, Hide) ask that non-essential motion can be turned off, and macOS
//! exposes the same choice as "Reduce motion". The default is [`Motion::Full`];
//! `--motion` on the command line ([`crate::cli::CommonArgs`]) or an
//! application's `<APP>_NO_MOTION` environment variable selects another
//! level ([`Motion::resolve`]).

/// Tick delivery mode.
///
/// - [`Full`](Motion::Full): live ticks advance the application's state
///   and its decorative motion (spinners, rituals, atmosphere).
/// - [`Reduced`](Motion::Reduced): state still advances, decorative motion
///   stays still (a spinner keeps its first frame). What else an
///   application shortens (a ritual skipped to its final frame) is the
///   application's choice.
/// - [`Paused`](Motion::Paused): ticks never advance; `--frame N` selects
///   the exact frame shown, so captures are deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Motion {
    /// Tick-driven state and decorative motion.
    #[default]
    Full,
    /// State advances; decorative motion stays still.
    Reduced,
    /// Ticks never advance; `--frame` selects the frame shown.
    Paused,
}

impl Motion {
    /// An explicit choice (the command line) wins; otherwise a set
    /// `<APP>_NO_MOTION` environment variable selects
    /// [`Reduced`](Motion::Reduced); otherwise [`Full`](Motion::Full).
    pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion {
        match cli {
            Some(m) => m,
            None if no_motion_env => Motion::Reduced,
            None => Motion::Full,
        }
    }

    /// Whether the environment variable `var` asks for reduced motion: set,
    /// not empty and not `0`.
    pub fn env_requests_reduced(var: &str) -> bool {
        std::env::var_os(var).is_some_and(|v| !v.is_empty() && v != "0")
    }

    /// [`resolve`](Motion::resolve) with the environment variable `var`.
    pub fn from_cli_or_env(cli: Option<Motion>, var: &str) -> Motion {
        Motion::resolve(cli, Motion::env_requests_reduced(var))
    }

    /// Whether ticks advance at all (everything but
    /// [`Paused`](Motion::Paused)).
    pub fn ticks(self) -> bool {
        self != Motion::Paused
    }

    /// Whether decorative motion (spinner frames, animated glyphs) moves:
    /// [`Full`](Motion::Full) only.
    pub fn animates(self) -> bool {
        self == Motion::Full
    }

    /// Whether this is [`Paused`](Motion::Paused).
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
