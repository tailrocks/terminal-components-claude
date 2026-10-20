//! Input outcome (ported from PR #26 `src/core/event.rs::Outcome`,
//! commit f4867e9b5).

/// What a widget did with an input event.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Outcome {
    /// The widget did not handle it.
    #[default]
    Ignored,
    /// The widget handled it without a visible change.
    Consumed,
    /// The widget handled it and something changed.
    Changed,
}

impl Outcome {
    /// Whether the event was handled at all (consumed or changed).
    pub fn consumed(self) -> bool {
        !matches!(self, Outcome::Ignored)
    }

    /// Fold two outcomes, keeping the stronger one.
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Outcome::Changed, _) | (_, Outcome::Changed) => Outcome::Changed,
            (Outcome::Consumed, _) | (_, Outcome::Consumed) => Outcome::Consumed,
            _ => Outcome::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn or_keeps_the_stronger_outcome() {
        assert_eq!(Outcome::Ignored.or(Outcome::Changed), Outcome::Changed);
        assert_eq!(Outcome::Consumed.or(Outcome::Changed), Outcome::Changed);
        assert_eq!(Outcome::Changed.or(Outcome::Consumed), Outcome::Changed);
        assert_eq!(Outcome::Consumed.or(Outcome::Ignored), Outcome::Consumed);
        assert_eq!(Outcome::Ignored.or(Outcome::Ignored), Outcome::Ignored);
        assert!(Outcome::Consumed.consumed());
        assert!(!Outcome::Ignored.consumed());
    }
}
