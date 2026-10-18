pub mod terminal_view;
pub use terminal_view::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_interaction_modes() {
        assert!(TerminalInteraction::Interactive.allows_forwarding());
        assert!(TerminalInteraction::Interactive.allows_selection());
        assert!(!TerminalInteraction::ReadOnly.allows_forwarding());
        assert!(!TerminalInteraction::ReadOnly.allows_selection());
        assert!(!TerminalInteraction::SelectionOnly.allows_forwarding());
        assert!(TerminalInteraction::SelectionOnly.allows_selection());
    }
}
