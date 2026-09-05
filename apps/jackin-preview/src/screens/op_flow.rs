//! Domain half of the staged 1Password picker.
//!
//! `junie_tui::PickerChain` owns breadcrumb focus and retry/back bindings.  The
//! app keeps provider-specific ids and translates simulator errors to safe
//! display state; credential material remains inside `SimOnePassword`'s
//! closure and is never stored here.

use junie_tui::{ItemKey, Status};

use crate::sim::onepassword::OpError;

/// Ordered stages in the 1Password reference flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpFlowStage {
    Account,
    Vault,
    Item,
    Field,
}

impl OpFlowStage {
    /// Stable stage key used by `PickerChain`.
    pub const fn key(self) -> ItemKey {
        ItemKey::num(match self {
            Self::Account => 1,
            Self::Vault => 2,
            Self::Item => 3,
            Self::Field => 4,
        })
    }

    /// Breadcrumb label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Vault => "Vault",
            Self::Item => "Item",
            Self::Field => "Field",
        }
    }

    /// Next stage, if any.
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Account => Some(Self::Vault),
            Self::Vault => Some(Self::Item),
            Self::Item => Some(Self::Field),
            Self::Field => None,
        }
    }

    /// Previous stage, if any.
    pub const fn previous(self) -> Option<Self> {
        match self {
            Self::Account => None,
            Self::Vault => Some(Self::Account),
            Self::Item => Some(Self::Vault),
            Self::Field => Some(Self::Item),
        }
    }
}

/// Safe loading/error state projected below a picker breadcrumb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpFlowStatus {
    Ready,
    Loading { label: String },
    Error { message: String, detail: String },
}

impl OpFlowStatus {
    /// Library collection status for this stage.
    pub const fn collection_status(&self) -> Status {
        match self {
            Self::Ready => Status::Ready,
            Self::Loading { .. } => Status::Loading,
            Self::Error { .. } => Status::Error,
        }
    }
}

/// Domain actions returned by the staged flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpFlowAction {
    /// Move to a new stage with a stable provider id.
    Entered { stage: OpFlowStage, id: String },
    /// Return to an earlier breadcrumb.
    Back(OpFlowStage),
    /// Retry the current operation.
    Retry(OpFlowStage),
    /// Accept a metadata-only reference.
    Completed { stage: OpFlowStage, id: String },
}

/// State retained by the app-side OpFlow composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpFlowState {
    current: OpFlowStage,
    selected: [Option<String>; 4],
    status: OpFlowStatus,
}

impl Default for OpFlowState {
    fn default() -> Self {
        Self {
            current: OpFlowStage::Account,
            selected: [None, None, None, None],
            status: OpFlowStatus::Ready,
        }
    }
}

impl OpFlowState {
    /// Current stage.
    pub const fn current(&self) -> OpFlowStage {
        self.current
    }

    /// Current status.
    pub const fn status(&self) -> &OpFlowStatus {
        &self.status
    }

    /// Selection at `stage`, if one exists.
    pub fn selected(&self, stage: OpFlowStage) -> Option<&str> {
        self.selected
            .get(Self::index(stage))
            .and_then(Option::as_deref)
    }

    /// Begin a deterministic loading state for one stage.
    pub fn begin_load(&mut self, stage: OpFlowStage, label: impl Into<String>) {
        self.current = stage;
        self.status = OpFlowStatus::Loading {
            label: label.into(),
        };
    }

    /// Store a provider id and advance the chain.
    pub fn choose(&mut self, id: impl Into<String>) -> Option<OpFlowAction> {
        let id = id.into();
        if id.is_empty() {
            return None;
        }
        let stage = self.current;
        let selection = self.selected.get_mut(Self::index(stage))?;
        *selection = Some(id.clone());
        self.status = OpFlowStatus::Ready;
        match stage.next() {
            Some(next) => {
                self.current = next;
                Some(OpFlowAction::Entered { stage: next, id })
            }
            None => Some(OpFlowAction::Completed { stage, id }),
        }
    }

    /// Rewind one stage and retain earlier selections.
    pub fn back(&mut self) -> Option<OpFlowAction> {
        let previous = self.current.previous()?;
        self.current = previous;
        self.status = OpFlowStatus::Ready;
        Some(OpFlowAction::Back(previous))
    }

    /// Rewind to an earlier stage and clear selections after it.
    pub fn back_to(&mut self, stage: OpFlowStage) -> Option<OpFlowAction> {
        if Self::index(stage) >= Self::index(self.current) {
            return None;
        }
        self.current = stage;
        for slot in self.selected.iter_mut().skip(Self::index(stage) + 1) {
            *slot = None;
        }
        self.status = OpFlowStatus::Ready;
        Some(OpFlowAction::Back(stage))
    }

    /// Record a retryable or terminal simulator error without its material.
    pub fn set_error(&mut self, error: OpError) {
        let message = error.message();
        self.status = OpFlowStatus::Error {
            message,
            detail: match error {
                OpError::Locked => "Unlock 1Password, then retry.".into(),
                OpError::AuthorizationRequired { .. } => "Authorize the selected account.".into(),
                OpError::PermissionDenied { .. } => "Choose a vault you can read.".into(),
                OpError::MissingAccount { .. }
                | OpError::MissingVault { .. }
                | OpError::MissingItem { .. }
                | OpError::MissingField { .. }
                | OpError::EmptyMaterial { .. }
                | OpError::WrongFieldShape { .. } => {
                    "The referenced object changed; choose again.".into()
                }
            },
        };
    }

    /// Retry the current failed stage.
    pub fn retry(&mut self) -> Option<OpFlowAction> {
        if matches!(self.status, OpFlowStatus::Error { .. }) {
            self.status = OpFlowStatus::Loading {
                label: format!("Loading {}…", self.current.label()),
            };
            Some(OpFlowAction::Retry(self.current))
        } else {
            None
        }
    }

    /// Reset the chain without retaining prior choices.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    const fn index(stage: OpFlowStage) -> usize {
        match stage {
            OpFlowStage::Account => 0,
            OpFlowStage::Vault => 1,
            OpFlowStage::Item => 2,
            OpFlowStage::Field => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_advances_and_rewinds_without_losing_prior_selection() {
        let mut state = OpFlowState::default();
        assert_eq!(
            state.choose("acct").unwrap(),
            OpFlowAction::Entered {
                stage: OpFlowStage::Vault,
                id: "acct".into()
            }
        );
        assert_eq!(
            state.choose("vault").unwrap(),
            OpFlowAction::Entered {
                stage: OpFlowStage::Item,
                id: "vault".into()
            }
        );
        assert_eq!(state.back(), Some(OpFlowAction::Back(OpFlowStage::Vault)));
        assert_eq!(state.selected(OpFlowStage::Account), Some("acct"));
        assert_eq!(state.selected(OpFlowStage::Vault), Some("vault"));
    }

    #[test]
    fn errors_are_safe_and_retryable() {
        let mut state = OpFlowState::default();
        state.set_error(OpError::Locked);
        assert!(state.status().collection_status() == Status::Error);
        assert!(
            !matches!(state.status(), OpFlowStatus::Error { message, .. } if message.contains("valid-ant"))
        );
        assert_eq!(
            state.retry(),
            Some(OpFlowAction::Retry(OpFlowStage::Account))
        );
        assert!(matches!(state.status(), OpFlowStatus::Loading { .. }));
    }
}
