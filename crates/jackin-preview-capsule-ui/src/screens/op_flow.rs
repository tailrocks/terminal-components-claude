use termrock::{ItemKey, Status};

use jackin_preview_sim::onepassword::OpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpFlowStage {
    Account,

    Vault,

    Item,

    Field,
}

impl OpFlowStage {
    pub const fn key(self) -> ItemKey {
        ItemKey::num(match self {
            Self::Account => 1,
            Self::Vault => 2,
            Self::Item => 3,
            Self::Field => 4,
        })
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Vault => "Vault",
            Self::Item => "Item",
            Self::Field => "Field",
        }
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Account => Some(Self::Vault),
            Self::Vault => Some(Self::Item),
            Self::Item => Some(Self::Field),
            Self::Field => None,
        }
    }

    pub const fn previous(self) -> Option<Self> {
        match self {
            Self::Account => None,
            Self::Vault => Some(Self::Account),
            Self::Item => Some(Self::Vault),
            Self::Field => Some(Self::Item),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpFlowStatus {
    Ready,

    Loading { label: String },

    Error { message: String, detail: String },
}

impl OpFlowStatus {
    pub const fn collection_status(&self) -> Status {
        match self {
            Self::Ready => Status::Ready,
            Self::Loading { .. } => Status::Loading,
            Self::Error { .. } => Status::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpFlowAction {
    Entered { stage: OpFlowStage, id: String },

    Back(OpFlowStage),

    Retry(OpFlowStage),

    Completed { stage: OpFlowStage, id: String },
}

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
    pub const fn current(&self) -> OpFlowStage {
        self.current
    }

    pub const fn status(&self) -> &OpFlowStatus {
        &self.status
    }

    pub fn selected(&self, stage: OpFlowStage) -> Option<&str> {
        self.selected
            .get(Self::index(stage))
            .and_then(Option::as_deref)
    }

    pub fn reconcile_selection(&mut self, stage: OpFlowStage, valid: &[String]) -> bool {
        let index = Self::index(stage);
        let Some(selected) = self.selected.get(index).and_then(Option::as_deref) else {
            return false;
        };
        if valid.iter().any(|candidate| candidate == selected) {
            return false;
        }
        self.clear_from(index);
        if Self::index(self.current) >= index {
            self.current = stage;
        }
        self.status = OpFlowStatus::Ready;
        true
    }

    pub fn begin_load(&mut self, stage: OpFlowStage, label: impl Into<String>) {
        self.current = stage;
        self.status = OpFlowStatus::Loading {
            label: label.into(),
        };
    }

    pub fn choose(&mut self, id: impl Into<String>) -> Option<OpFlowAction> {
        let id = id.into();
        if id.is_empty() {
            return None;
        }
        let stage = self.current;
        let index = Self::index(stage);
        let selection = self.selected.get_mut(index)?;
        *selection = Some(id.clone());
        self.clear_after(index);
        self.status = OpFlowStatus::Ready;
        match stage.next() {
            Some(next) => {
                self.current = next;
                Some(OpFlowAction::Entered { stage: next, id })
            }
            None => Some(OpFlowAction::Completed { stage, id }),
        }
    }

    pub fn back(&mut self) -> Option<OpFlowAction> {
        let previous = self.current.previous()?;
        self.current = previous;
        self.clear_after(Self::index(previous));
        self.status = OpFlowStatus::Ready;
        Some(OpFlowAction::Back(previous))
    }

    pub fn back_to(&mut self, stage: OpFlowStage) -> Option<OpFlowAction> {
        if Self::index(stage) >= Self::index(self.current) {
            return None;
        }
        self.current = stage;
        self.clear_after(Self::index(stage));
        self.status = OpFlowStatus::Ready;
        Some(OpFlowAction::Back(stage))
    }

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

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn clear_after(&mut self, index: usize) {
        for slot in self.selected.iter_mut().skip(index.saturating_add(1)) {
            *slot = None;
        }
    }

    fn clear_from(&mut self, index: usize) {
        for slot in self.selected.iter_mut().skip(index) {
            *slot = None;
        }
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
