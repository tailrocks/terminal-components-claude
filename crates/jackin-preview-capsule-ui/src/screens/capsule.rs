use termrock::Id;

pub const ROOT: Id = Id::root("jackin.capsule");
pub const TABS: Id = ROOT.sub("tabs");
pub const PANES: Id = ROOT.sub("panes");
pub const NEW_TAB: Id = ROOT.sub("new-tab");
pub const SPLIT: Id = ROOT.sub("split");
pub const DETACH: Id = ROOT.sub("detach");
pub const APP_MENU: Id = ROOT.sub("app-menu");
pub const EXIT_DIALOG: Id = ROOT.sub("exit-dialog");
pub const EXIT_CHOICE: Id = EXIT_DIALOG.sub("choice");

pub const PREFIX_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsuleLayer {
    #[default]
    Normal,
    Prefix,
    AppMenu,
    ExitConfirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsuleFocus {
    #[default]
    Pane,
    Tabs,
    AppMenu,
    ExitChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixCommand {
    NewTab,
    NextTab,
    PreviousTab,
    ClosePane,
    CloseTab,
    SplitBelow,
    SplitRight,
    Zoom,
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    Detach,
    Usage,
    ContainerInfo,
    RenameTab,
    TabMenu,
    Palette,
    Redraw,
    SelectTab(u8),
    ClearPane,
}

impl PrefixCommand {
    pub const fn from_char(key: char) -> Option<Self> {
        match key {
            'c' => Some(Self::NewTab),
            'n' => Some(Self::NextTab),
            'p' => Some(Self::PreviousTab),
            'x' => Some(Self::ClosePane),
            '&' => Some(Self::CloseTab),
            '"' => Some(Self::SplitBelow),
            '%' => Some(Self::SplitRight),
            'z' => Some(Self::Zoom),
            'h' => Some(Self::FocusLeft),
            'j' => Some(Self::FocusDown),
            'k' => Some(Self::FocusUp),
            'l' => Some(Self::FocusRight),
            'd' => Some(Self::Detach),
            'u' => Some(Self::Usage),
            'i' => Some(Self::ContainerInfo),
            ',' => Some(Self::RenameTab),
            'm' => Some(Self::TabMenu),
            ' ' | ':' => Some(Self::Palette),
            'r' => Some(Self::Redraw),
            '0' => Some(Self::SelectTab(10)),
            '1' => Some(Self::SelectTab(1)),
            '2' => Some(Self::SelectTab(2)),
            '3' => Some(Self::SelectTab(3)),
            '4' => Some(Self::SelectTab(4)),
            '5' => Some(Self::SelectTab(5)),
            '6' => Some(Self::SelectTab(6)),
            '7' => Some(Self::SelectTab(7)),
            '8' => Some(Self::SelectTab(8)),
            '9' => Some(Self::SelectTab(9)),
            _ => None,
        }
    }

    pub const fn from_control_char(key: char) -> Option<Self> {
        match key {
            'l' => Some(Self::ClearPane),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitDecision {
    StayInside,
    KeepChanges,
    DiscardChanges,
    Cancel,
}

impl ExitDecision {
    pub const COUNT: u8 = 4;

    pub const fn from_index(index: u8) -> Self {
        match index {
            1 => Self::KeepChanges,
            2 => Self::DiscardChanges,
            3 => Self::Cancel,
            _ => Self::StayInside,
        }
    }

    pub const fn index(self) -> u8 {
        match self {
            Self::StayInside => 0,
            Self::KeepChanges => 1,
            Self::DiscardChanges => 2,
            Self::Cancel => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapsuleInteraction {
    layer: CapsuleLayer,
    focus: CapsuleFocus,
    exit_choice: u8,
}

impl Default for CapsuleInteraction {
    fn default() -> Self {
        Self {
            layer: CapsuleLayer::Normal,
            focus: CapsuleFocus::Pane,
            exit_choice: ExitDecision::StayInside.index(),
        }
    }
}

impl CapsuleInteraction {
    pub const fn layer(self) -> CapsuleLayer {
        self.layer
    }

    pub const fn focus(self) -> CapsuleFocus {
        self.focus
    }

    pub const fn exit_choice(self) -> u8 {
        self.exit_choice
    }

    pub fn begin_prefix(&mut self) -> bool {
        if self.layer != CapsuleLayer::Normal {
            return false;
        }
        self.layer = CapsuleLayer::Prefix;
        self.focus = CapsuleFocus::Pane;
        true
    }

    pub fn resolve_prefix(&mut self, key: char) -> Option<PrefixCommand> {
        if self.layer != CapsuleLayer::Prefix {
            return None;
        }
        self.layer = CapsuleLayer::Normal;
        self.focus = CapsuleFocus::Pane;
        PrefixCommand::from_char(key)
    }

    pub fn open_app_menu(&mut self) -> bool {
        if self.layer == CapsuleLayer::ExitConfirmation {
            return false;
        }
        self.layer = CapsuleLayer::AppMenu;
        self.focus = CapsuleFocus::AppMenu;
        true
    }

    pub fn open_exit_confirmation(&mut self) -> bool {
        self.layer = CapsuleLayer::ExitConfirmation;
        self.focus = CapsuleFocus::ExitChoice;
        self.exit_choice = ExitDecision::StayInside.index();
        true
    }

    pub fn move_exit_choice(&mut self, delta: i8) -> Option<ExitDecision> {
        if self.layer != CapsuleLayer::ExitConfirmation {
            return None;
        }
        let last = ExitDecision::COUNT.saturating_sub(1);
        self.exit_choice = if delta.is_negative() {
            self.exit_choice
                .saturating_sub(delta.unsigned_abs().min(self.exit_choice))
        } else {
            self.exit_choice.saturating_add(delta as u8).min(last)
        };
        Some(ExitDecision::from_index(self.exit_choice))
    }

    pub fn take_exit_decision(&mut self) -> Option<ExitDecision> {
        if self.layer != CapsuleLayer::ExitConfirmation {
            return None;
        }
        let decision = ExitDecision::from_index(self.exit_choice);
        self.dismiss();
        Some(decision)
    }

    pub fn dismiss(&mut self) -> bool {
        if self.layer == CapsuleLayer::Normal {
            return false;
        }
        self.layer = CapsuleLayer::Normal;
        self.focus = CapsuleFocus::Pane;
        self.exit_choice = ExitDecision::StayInside.index();
        true
    }

    pub fn focus_tabs(&mut self) {
        if self.layer == CapsuleLayer::Normal {
            self.focus = CapsuleFocus::Tabs;
        }
    }

    pub fn focus_pane(&mut self) {
        if self.layer == CapsuleLayer::Normal {
            self.focus = CapsuleFocus::Pane;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapsuleState {
    pub tab: u8,
    pub zoomed: bool,
    pub selected_pane: u64,
    pub context_open: bool,
}

#[cfg(test)]
mod tests {
    use super::{CapsuleFocus, CapsuleInteraction, CapsuleLayer, ExitDecision, PrefixCommand};

    #[test]
    fn prefix_detach_is_routed_once_and_restores_pane_focus() {
        let mut state = CapsuleInteraction::default();
        assert!(state.begin_prefix());
        assert_eq!(state.layer(), CapsuleLayer::Prefix);
        assert_eq!(state.resolve_prefix('d'), Some(PrefixCommand::Detach));
        assert_eq!(state.layer(), CapsuleLayer::Normal);
        assert_eq!(state.focus(), CapsuleFocus::Pane);
        assert_eq!(state.resolve_prefix('d'), None);
    }

    #[test]
    fn exit_cursor_is_clamped_and_commit_dismisses_the_layer() {
        let mut state = CapsuleInteraction::default();
        assert!(state.open_exit_confirmation());
        assert_eq!(
            state.move_exit_choice(2),
            Some(ExitDecision::DiscardChanges)
        );
        assert_eq!(state.move_exit_choice(9), Some(ExitDecision::Cancel));
        assert_eq!(state.move_exit_choice(-9), Some(ExitDecision::StayInside));
        assert_eq!(state.take_exit_decision(), Some(ExitDecision::StayInside));
        assert_eq!(state.layer(), CapsuleLayer::Normal);
        assert_eq!(state.focus(), CapsuleFocus::Pane);
    }

    #[test]
    fn exit_confirmation_blocks_prefix_and_menu_focus_restores_on_dismiss() {
        let mut state = CapsuleInteraction::default();
        state.open_exit_confirmation();
        assert!(!state.begin_prefix());
        assert!(!state.open_app_menu());
        assert!(state.dismiss());
        assert!(state.open_app_menu());
        assert_eq!(state.focus(), CapsuleFocus::AppMenu);
        assert!(state.dismiss());
        assert_eq!(state.focus(), CapsuleFocus::Pane);
    }
}
