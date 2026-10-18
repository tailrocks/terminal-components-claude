#![doc = r" Capsule route state and public pane controls."]

use termrock::Id;

#[doc = r" Capsule root."]
pub const ROOT: Id = Id::root("jackin.capsule");
#[doc = r" Capsule tabs."]
pub const TABS: Id = ROOT.sub("tabs");
#[doc = r" Pane list."]
pub const PANES: Id = ROOT.sub("panes");
#[doc = r" New-tab action."]
pub const NEW_TAB: Id = ROOT.sub("new-tab");
#[doc = r" Split action."]
pub const SPLIT: Id = ROOT.sub("split");
#[doc = r" Detach action."]
pub const DETACH: Id = ROOT.sub("detach");
#[doc = r" Capsule application-menu layer."]
pub const APP_MENU: Id = ROOT.sub("app-menu");
#[doc = r" Capsule exit-confirmation layer."]
pub const EXIT_DIALOG: Id = ROOT.sub("exit-dialog");
#[doc = r" Exit-confirmation choice control."]
pub const EXIT_CHOICE: Id = EXIT_DIALOG.sub("choice");

#[doc = r" Prefix-key timeout used by the Capsule workbench."]
pub const PREFIX_TIMEOUT_MS: u64 = 2_000;

#[doc = r" Top-level transient layer that owns Capsule key routing."]
#[doc = r""]
#[doc = r" The layer is deliberately separate from [`CapsuleState`].  The latter is"]
#[doc = r" part of the existing public projection and remains a compact pane model;"]
#[doc = r" this state machine owns only ephemeral input routing."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsuleLayer {
    #[doc = r" Normal pane interaction."]
    #[default]
    Normal,
    #[doc = r" Waiting for the second key of the `Ctrl+B` prefix chord."]
    Prefix,
    #[doc = r" Application menu is open and owns focus."]
    AppMenu,
    #[doc = r" Exit choices are open and own focus."]
    ExitConfirmation,
}

#[doc = r" Focus owner for Capsule's transient routing state."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsuleFocus {
    #[doc = r" The active pane owns focus."]
    #[default]
    Pane,
    #[doc = r" The tab strip owns focus."]
    Tabs,
    #[doc = r" The application menu owns focus."]
    AppMenu,
    #[doc = r" The exit-choice control owns focus."]
    ExitChoice,
}

#[doc = r" Prefix command emitted after a complete `Ctrl+B <key>` sequence."]
#[doc = r""]
#[doc = r" The app shell owns the effects (daemon mutation, route changes and"]
#[doc = r" dialogs); this enum makes the input-to-action mapping total and testable."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixCommand {
    #[doc = r" Open a new tab."]
    NewTab,
    #[doc = r" Select the next tab."]
    NextTab,
    #[doc = r" Select the previous tab."]
    PreviousTab,
    #[doc = r" Close the focused pane."]
    ClosePane,
    #[doc = r" Close the active tab."]
    CloseTab,
    #[doc = r" Split below the focused pane."]
    SplitBelow,
    #[doc = r" Split right of the focused pane."]
    SplitRight,
    #[doc = r" Toggle pane zoom."]
    Zoom,
    #[doc = r" Focus the pane on the left."]
    FocusLeft,
    #[doc = r" Focus the pane below."]
    FocusDown,
    #[doc = r" Focus the pane above."]
    FocusUp,
    #[doc = r" Focus the pane on the right."]
    FocusRight,
    #[doc = r" Detach to the workspace manager."]
    Detach,
    #[doc = r" Open usage for the active Capsule."]
    Usage,
    #[doc = r" Open typed container identity details."]
    ContainerInfo,
    #[doc = r" Open the active tab's rename control."]
    RenameTab,
    #[doc = r" Open the active tab context menu."]
    TabMenu,
    #[doc = r" Open the command palette."]
    Palette,
    #[doc = r" Request a redraw."]
    Redraw,
    #[doc = r" Select a numbered tab; zero is the conventional tenth tab."]
    SelectTab(u8),
    #[doc = r" Clear the focused pane."]
    ClearPane,
}

impl PrefixCommand {
    #[doc = r" Resolve one unmodified prefix character."]
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

    #[doc = r" Resolve one control character after the prefix."]
    pub const fn from_control_char(key: char) -> Option<Self> {
        match key {
            'l' => Some(Self::ClearPane),
            _ => None,
        }
    }
}

#[doc = r" Choice returned when the exit confirmation is committed."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitDecision {
    #[doc = r" Leave the current instance running and return to the manager."]
    StayInside,
    #[doc = r" Exit while preserving dirty state for reconnect/restore."]
    KeepChanges,
    #[doc = r" Exit and discard dirty state."]
    DiscardChanges,
    #[doc = r" Close the confirmation without exiting."]
    Cancel,
}

impl ExitDecision {
    #[doc = r" Number of choices exposed by the state machine."]
    pub const COUNT: u8 = 4;

    #[doc = r" Convert the stable choice index into its semantic decision."]
    pub const fn from_index(index: u8) -> Self {
        match index {
            1 => Self::KeepChanges,
            2 => Self::DiscardChanges,
            3 => Self::Cancel,
            _ => Self::StayInside,
        }
    }

    #[doc = r" Return the stable choice index."]
    pub const fn index(self) -> u8 {
        match self {
            Self::StayInside => 0,
            Self::KeepChanges => 1,
            Self::DiscardChanges => 2,
            Self::Cancel => 3,
        }
    }
}

#[doc = r" Ephemeral Capsule input/focus state."]
#[doc = r""]
#[doc = r" This type is intentionally independent of daemon state.  It can therefore"]
#[doc = r" survive a route handoff and reconnect without resetting the active tab or"]
#[doc = r" pane projection in [`CapsuleState`]."]
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
    #[doc = r" Current transient routing layer."]
    pub const fn layer(self) -> CapsuleLayer {
        self.layer
    }

    #[doc = r" Current focus owner."]
    pub const fn focus(self) -> CapsuleFocus {
        self.focus
    }

    #[doc = r" Current exit-choice index."]
    pub const fn exit_choice(self) -> u8 {
        self.exit_choice
    }

    #[doc = r" Enter prefix mode if no higher-priority layer owns input."]
    pub fn begin_prefix(&mut self) -> bool {
        if self.layer != CapsuleLayer::Normal {
            return false;
        }
        self.layer = CapsuleLayer::Prefix;
        self.focus = CapsuleFocus::Pane;
        true
    }

    #[doc = r" Resolve the second key in a prefix sequence and return its action."]
    #[doc = r""]
    #[doc = r" Prefix mode is consumed even for an unknown key.  This prevents a"]
    #[doc = r" stale prefix from stealing the next unrelated key after a miss."]
    pub fn resolve_prefix(&mut self, key: char) -> Option<PrefixCommand> {
        if self.layer != CapsuleLayer::Prefix {
            return None;
        }
        self.layer = CapsuleLayer::Normal;
        self.focus = CapsuleFocus::Pane;
        PrefixCommand::from_char(key)
    }

    #[doc = r" Open the app menu and move focus to it."]
    pub fn open_app_menu(&mut self) -> bool {
        if self.layer == CapsuleLayer::ExitConfirmation {
            return false;
        }
        self.layer = CapsuleLayer::AppMenu;
        self.focus = CapsuleFocus::AppMenu;
        true
    }

    #[doc = r" Open exit confirmation and select its first option."]
    pub fn open_exit_confirmation(&mut self) -> bool {
        self.layer = CapsuleLayer::ExitConfirmation;
        self.focus = CapsuleFocus::ExitChoice;
        self.exit_choice = ExitDecision::StayInside.index();
        true
    }

    #[doc = r" Move the exit choice, clamped to the available options."]
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

    #[doc = r" Commit and clear the exit confirmation."]
    pub fn take_exit_decision(&mut self) -> Option<ExitDecision> {
        if self.layer != CapsuleLayer::ExitConfirmation {
            return None;
        }
        let decision = ExitDecision::from_index(self.exit_choice);
        self.dismiss();
        Some(decision)
    }

    #[doc = r" Dismiss any transient layer and return focus to the active pane."]
    pub fn dismiss(&mut self) -> bool {
        if self.layer == CapsuleLayer::Normal {
            return false;
        }
        self.layer = CapsuleLayer::Normal;
        self.focus = CapsuleFocus::Pane;
        self.exit_choice = ExitDecision::StayInside.index();
        true
    }

    #[doc = r" Move focus to the tab strip after a tab activation."]
    pub fn focus_tabs(&mut self) {
        if self.layer == CapsuleLayer::Normal {
            self.focus = CapsuleFocus::Tabs;
        }
    }

    #[doc = r" Restore focus to the active pane."]
    pub fn focus_pane(&mut self) {
        if self.layer == CapsuleLayer::Normal {
            self.focus = CapsuleFocus::Pane;
        }
    }
}

#[doc = r" Capsule interaction state."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapsuleState {
    #[doc = r" Selected capsule tab index."]
    pub tab: u8,
    #[doc = r" Whether the selected pane is zoomed."]
    pub zoomed: bool,
    #[doc = r" Stable identifier of the selected pane."]
    pub selected_pane: u64,
    #[doc = r" Whether the pane context menu is open."]
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
