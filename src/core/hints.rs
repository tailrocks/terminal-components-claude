use std::borrow::Cow;

use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, NavKeys};
use crate::widgets::keyhint::{Hint, HintPriority};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HintCx {
    pub focus: Option<WidgetId>,
}

impl HintCx {
    pub const fn new(focus: Option<WidgetId>) -> Self {
        Self { focus }
    }
}

pub trait HasHints {
    fn hints(&self, cx: &HintCx) -> Vec<Hint>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub key: Option<KeyBinding>,

    pub label: Cow<'static, str>,

    pub action: Cow<'static, str>,
}

impl Binding {
    pub fn key(key: KeyBinding, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: Some(key),
            label: key.label().into(),
            action: action.into(),
        }
    }

    pub fn group(label: &'static str, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: None,
            label: Cow::Borrowed(label),
            action: action.into(),
        }
    }
}

pub trait Bindings {
    fn bindings(&self) -> Vec<Binding>;

    fn chords(&self) -> Vec<KeyBinding> {
        self.bindings().into_iter().filter_map(|b| b.key).collect()
    }
}

pub fn keymap_bindings<A: Copy>(
    map: &[(KeyBinding, A)],
    label: impl Fn(A) -> &'static str,
) -> Vec<Binding> {
    map.iter()
        .filter(|(k, _)| k.deliverable())
        .map(|&(k, a)| Binding::key(k, label(a)))
        .collect()
}

pub fn actions_bindings<A: Actions>(map: &[(KeyBinding, A)]) -> Vec<Binding> {
    keymap_bindings(map, A::label)
}

pub fn nav_bindings(nav: &NavKeys, type_ahead: bool) -> Vec<Binding> {
    let mut v = vec![];
    if let Some(k) = nav.move_hint(type_ahead) {
        v.push(Binding::group(k, "Move"));
    }
    if let Some(k) = nav.page_hint() {
        v.push(Binding::group(k, "Page"));
    }
    if let Some(k) = nav.ends_hint(type_ahead) {
        v.push(Binding::group(k, "First / last"));
    }
    if type_ahead {
        v.push(Binding::group("a–z", "Jump to a row by name"));
    }
    v
}

pub const TYPE_TO_JUMP: Hint = Hint {
    key: "Type",
    action: Cow::Borrowed("to jump"),
    priority: HintPriority::Normal,
};

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
        h.iter()
            .map(|h| format!("{} {}", h.key, h.action))
            .collect()
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
