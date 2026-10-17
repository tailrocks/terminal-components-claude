//! Hints and bindings a widget derives from its own state (plan S8b):
//! [`HasHints`] is what the footer shows while the widget is on top or has
//! focus, [`Bindings`] is every key it takes (what the help overlay and
//! the component docs list, and the chords [`crate::shell::KeyRouter`]
//! leaves to the app when the focused widget does not bind them).
//!
//! Hints never advertise a key the terminal cannot deliver
//! ([`KeyBinding::deliverable`]) or a key the widget's configuration does
//! not take: navigation hints come from the widget's own [`NavKeys`]
//! ([`nav_hints`]), so `j k` and `g G` show only under vim keys and a
//! type-selecting widget says "Type to jump" (CO3).

use std::borrow::Cow;

use crate::core::id::WidgetId;
use crate::core::keymap::{KeyBinding, NavKeys};
use crate::widgets::keyhint::{Hint, HintPriority};

/// What a widget's hints depend on beyond its own state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HintCx {
    /// The control with keyboard focus (a dialog names its focused
    /// button; a form hints the focused field).
    pub focus: Option<WidgetId>,
}

impl HintCx {
    /// Hints for the frame where `focus` has keyboard focus.
    pub const fn new(focus: Option<WidgetId>) -> Self {
        Self { focus }
    }
}

/// A widget that knows its footer hints: built from its state, so a hint
/// can never disagree with what the keys do.
pub trait HasHints {
    /// The hints while the widget has focus (or is the top layer).
    fn hints(&self, cx: &HintCx) -> Vec<Hint>;
}

/// One key a widget takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// The chord, when it is one key a key map binds (`None` for a group
    /// such as `↑↓` or "letters").
    pub key: Option<KeyBinding>,
    /// How hints and help spell it (`Ctrl+S`, `↑↓`, `A–Z`).
    pub label: Cow<'static, str>,
    /// What it does (`Move`, `Select all`).
    pub action: Cow<'static, str>,
}

impl Binding {
    /// A key map row: `key` spelled by [`KeyBinding::label`].
    pub fn key(key: KeyBinding, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: Some(key),
            label: key.label().into(),
            action: action.into(),
        }
    }

    /// A row naming a group of keys (`↑↓ Move`).
    pub fn group(label: &'static str, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: None,
            label: Cow::Borrowed(label),
            action: action.into(),
        }
    }
}

/// A widget that lists every key it takes, from its live key maps.
pub trait Bindings {
    /// Every key the widget takes under its current configuration.
    fn bindings(&self) -> Vec<Binding>;

    /// The chords among [`Self::bindings`]: what the key router asks
    /// before it hands a Ctrl, Alt or Cmd character chord to the focused
    /// widget (CO1).
    fn chords(&self) -> Vec<KeyBinding> {
        self.bindings().into_iter().filter_map(|b| b.key).collect()
    }
}

/// The rows of a key map whose action has a label, as [`Binding`]s.
/// Undeliverable chords (Cmd without the kitty protocol) are left out
/// (SI7, SIg4).
pub fn keymap_bindings<A: Copy>(
    map: &[(KeyBinding, A)],
    label: impl Fn(A) -> &'static str,
) -> Vec<Binding> {
    map.iter()
        .filter(|(k, _)| k.deliverable())
        .map(|&(k, a)| Binding::key(k, label(a)))
        .collect()
}

/// The text a type-selecting widget's hint shows: `Type to jump`.
pub const TYPE_TO_JUMP: Hint = Hint {
    key: "Type",
    action: Cow::Borrowed("to jump"),
    priority: HintPriority::Normal,
};

/// The navigation hints of a widget configured with `nav`: `↑↓ Move`
/// (`j k Move` when only vim letters move), `g G` or `Home End` for the
/// ends when `ends` (a list) asks for them, and [`TYPE_TO_JUMP`] when
/// `type_ahead` is on. A key the configuration does not take is never
/// named (CO3).
pub fn nav_hints(nav: &NavKeys, type_ahead: bool, ends: bool) -> Vec<Hint> {
    let mut v = Vec::new();
    if let Some(k) = nav.move_hint(type_ahead) {
        v.push(Hint::new(k, "Move"));
    }
    if ends && let Some(k) = nav.ends_hint(type_ahead) {
        v.push(Hint::new(k, "Ends").priority(HintPriority::Normal));
    }
    if type_ahead {
        v.push(TYPE_TO_JUMP);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(h: &[Hint]) -> Vec<String> {
        h.iter().map(|h| format!("{} {}", h.key, h.action)).collect()
    }

    #[test]
    fn desktop_navigation_names_arrows_and_type_ahead_never_vim_letters() {
        let h = nav_hints(&NavKeys::DESKTOP, true, true);
        assert_eq!(keys(&h), ["↑↓ Move", "Home End Ends", "Type to jump"]);
    }

    #[test]
    fn vim_navigation_names_the_letters_it_takes() {
        let vim_only = NavKeys::DESKTOP.arrows(false).vim(true);
        assert_eq!(
            keys(&nav_hints(&vim_only, false, true)),
            ["j k Move", "g G Ends"]
        );
        // a type-selecting widget gives the letters to type-ahead
        let typing = NavKeys::DESKTOP.vim(true);
        assert_eq!(
            keys(&nav_hints(&typing, true, true)),
            ["↑↓ Move", "Home End Ends", "Type to jump"]
        );
    }

    #[test]
    fn keymap_rows_skip_chords_the_terminal_does_not_deliver() {
        #[derive(Clone, Copy)]
        struct A;
        let rows = keymap_bindings(
            &[(KeyBinding::cmd('a'), A), (KeyBinding::ctrl('a'), A)],
            |_| "Select all",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "Ctrl+A");
    }
}
