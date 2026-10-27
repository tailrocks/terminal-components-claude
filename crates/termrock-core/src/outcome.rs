//! Input outcome (ported from PR #26 `src/core/event.rs::Outcome`,

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Outcome {
    #[default]
    Ignored,

    Consumed,

    Changed,
}

impl Outcome {
    pub fn consumed(self) -> bool {
        !matches!(self, Outcome::Ignored)
    }

    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Outcome::Changed, _) | (_, Outcome::Changed) => Outcome::Changed,
            (Outcome::Consumed, _) | (_, Outcome::Consumed) => Outcome::Consumed,
            _ => Outcome::Ignored,
        }
    }
}
