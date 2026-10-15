//! Plan dependency rules, activity/output lifecycle, cancellation and target-bound safety decisions.
#![forbid(unsafe_code)]

pub use holla_domain::activity::*;
pub use holla_domain::plan::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_state_labels() {
        assert_eq!(StepState::Ready.label(), "ready");
        assert_eq!(StepState::Succeeded.label(), "succeeded");
    }
}
