//! Tiered key routing: which keys the shell answers before the page, which
//! the page answers first, and which only fire when nothing else did.
//!
//! Ported from PR #26 `src/shell/router.rs` (commit `f4867e9b5`). The parity
//! adaptation routes on [`termrock_core::event::Key`] chords directly and
//! reports through [`termrock_core::response::Response`], the runtime's
//! update contract, instead of the source's `Outcome`; the focus movement
//! actions use the runtime focus ring's `Cx::focus_next` and `Cx::focus_prev`.
//! Chord matching mirrors the
//! source's `KeyBinding::deliverable` + `lookup` pair: a binding matches an
//! exact key, and a printable key binding stays quiet while a text field
//! owns typing (as do `idle_only` actions). One ordering difference is
//! deliberate: the parity runtime serves Tab and BackTab out of its focus
//! ring before any app code runs, so [`ShellAction::FocusNext`] and
//! [`ShellAction::FocusPrev`] stay in the vocabulary for custom chords an
//! app binds and routes itself, but the default Tab bindings below are
//! documentation of the standard behaviour rather than the mechanism.

use termrock_core::event::{Chord, Input, Key, KeyCode};
use termrock_core::keys::KeyModifiers;
use termrock_core::response::Response;

use crate::quit::{KeyContext, LeaveKey, QuitKeys, ctrl_char};

/// One key chord bound to an action, with the priority hints use to order
/// the footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding<C> {
    /// The chord that fires the action.
    pub chord: Chord,
    /// What fires.
    pub action: ShellAction<C>,
    /// Ordering weight for hints; lower shows first.
    pub priority: u8,
}

/// Make a chord from a key code and optional modifier set.
pub fn chord(code: KeyCode, mods: KeyModifiers) -> Chord {
    Chord { code, mods }
}

/// Look up the action bound to `key` in `map`. Chords match exactly.
pub fn lookup<'a, C: PartialEq>(map: &'a [KeyBinding<C>], key: &Key) -> Option<&'a KeyBinding<C>> {
    let chord = key.chord();
    map.iter()
        .find(|b| b.chord.code == chord.code && b.chord.mods == chord.mods)
}

/// What the shell does with a key of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAction<C = ()> {
    /// Ask to leave the app.
    Quit,
    /// Open the help reference.
    Help,
    /// Open the menu bar.
    MenuBar,
    /// Next page.
    NextPage,
    /// Previous page.
    PrevPage,
    /// Move focus to the next control.
    FocusNext,
    /// Move focus to the previous control.
    FocusPrev,
    /// Go back one level (Esc's meaning).
    Back,
    /// An app-specific command.
    App(C),
}

impl<C> From<C> for ShellAction<C> {
    fn from(c: C) -> Self {
        Self::App(c)
    }
}

/// The shell's key vocabulary, in three tiers.
///
/// `priority` wins over everything the page could do; `keys` are the shell's
/// own bindings, answered when the page did not consume the key; `fallback`
/// fire only when nothing else wanted the key. Printables and `idle_only`
/// actions stay quiet while a text field owns typing.
#[derive(Debug, Clone)]
pub struct ShellKeys<C = ()> {
    /// Which keys ask to leave.
    pub quit: QuitKeys,
    /// Answered before the page sees the key.
    pub priority: Vec<KeyBinding<C>>,
    /// Answered after the page passed on the key.
    pub keys: Vec<KeyBinding<C>>,
    /// Answered only when nothing else wanted the key.
    pub fallback: Vec<KeyBinding<C>>,
    /// Actions that never fire while a text field owns typing, even on a
    /// non-printable chord.
    pub idle_only: Vec<ShellAction<C>>,
    /// What Esc means here, for the footer and the note when it lands.
    pub esc_note: Option<String>,
    /// What a quit letter says when dirty changes block it.
    pub dirty_letter_note: Option<String>,
}

impl<C> Default for ShellKeys<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ShellKeys<C> {
    /// The standard vocabulary: F1 help, the menu bar, Ctrl+PgDn/PgUp pages,
    /// Tab focus movement, Esc back.
    pub fn new() -> Self {
        let bind = |code: KeyCode, mods: KeyModifiers, action| KeyBinding {
            chord: chord(code, mods),
            action,
            priority: 5,
        };
        Self {
            quit: QuitKeys::DEFAULT,
            priority: Vec::new(),
            keys: vec![
                bind(KeyCode::F(1), KeyModifiers::NONE, ShellAction::Help),
                bind(KeyCode::F(10), KeyModifiers::NONE, ShellAction::MenuBar),
                bind(KeyCode::F(2), KeyModifiers::CONTROL, ShellAction::MenuBar),
                bind(
                    KeyCode::PageDown,
                    KeyModifiers::CONTROL,
                    ShellAction::NextPage,
                ),
                bind(
                    KeyCode::PageUp,
                    KeyModifiers::CONTROL,
                    ShellAction::PrevPage,
                ),
            ],
            fallback: vec![
                bind(KeyCode::Tab, KeyModifiers::NONE, ShellAction::FocusNext),
                bind(
                    KeyCode::BackTab,
                    KeyModifiers::SHIFT,
                    ShellAction::FocusPrev,
                ),
                bind(KeyCode::BackTab, KeyModifiers::NONE, ShellAction::FocusPrev),
                bind(KeyCode::Char('?'), KeyModifiers::NONE, ShellAction::Help),
                bind(KeyCode::Esc, KeyModifiers::NONE, ShellAction::Back),
            ],
            idle_only: Vec::new(),
            esc_note: Some("Esc never quits · Ctrl+Q quits".into()),
            dirty_letter_note: Some("Unsaved changes · Ctrl+Q asks before quitting".into()),
        }
    }

    /// Set the quit keys.
    #[must_use]
    pub fn quit(mut self, quit: QuitKeys) -> Self {
        self.quit = quit;
        self
    }

    /// Bind a key in the `keys` tier, in front of its peers.
    #[must_use]
    pub fn bind(mut self, code: KeyCode, mods: KeyModifiers, action: ShellAction<C>) -> Self {
        self.keys.insert(
            0,
            KeyBinding {
                chord: chord(code, mods),
                action,
                priority: 5,
            },
        );
        self
    }

    /// Bind a key in the `priority` tier, in front of its peers.
    #[must_use]
    pub fn bind_priority(
        mut self,
        code: KeyCode,
        mods: KeyModifiers,
        action: ShellAction<C>,
    ) -> Self {
        self.priority.insert(
            0,
            KeyBinding {
                chord: chord(code, mods),
                action,
                priority: 5,
            },
        );
        self
    }

    /// Bind a key in the `fallback` tier, in front of its peers.
    #[must_use]
    pub fn bind_fallback(
        mut self,
        code: KeyCode,
        mods: KeyModifiers,
        action: ShellAction<C>,
    ) -> Self {
        self.fallback.insert(
            0,
            KeyBinding {
                chord: chord(code, mods),
                action,
                priority: 5,
            },
        );
        self
    }

    /// Bind a key with an explicit hint priority.
    #[must_use]
    pub fn bind_with(mut self, tier: Tier, binding: KeyBinding<C>) -> Self {
        let list = match tier {
            Tier::Priority => &mut self.priority,
            Tier::Keys => &mut self.keys,
            Tier::Fallback => &mut self.fallback,
        };
        list.insert(0, binding);
        self
    }

    /// Remove a key from every tier.
    #[must_use]
    pub fn unbind(mut self, code: KeyCode, mods: KeyModifiers) -> Self {
        let drop = |list: &mut Vec<KeyBinding<C>>| {
            list.retain(|b| !(b.chord.code == code && b.chord.mods == mods));
        };
        drop(&mut self.priority);
        drop(&mut self.keys);
        drop(&mut self.fallback);
        self
    }

    /// An action additionally quieted while a text field owns typing.
    #[must_use]
    pub fn idle_only(mut self, action: ShellAction<C>) -> Self {
        self.idle_only.push(action);
        self
    }

    /// Set what Esc means here (`None` makes Esc a silent back).
    #[must_use]
    pub fn esc_note(mut self, note: Option<&str>) -> Self {
        self.esc_note = note.map(Into::into);
        self
    }

    /// Set the note a blocked quit letter shows.
    #[must_use]
    pub fn dirty_letter_note(mut self, note: Option<&str>) -> Self {
        self.dirty_letter_note = note.map(Into::into);
        self
    }
}

/// Which tier a binding lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Before the page.
    Priority,
    /// The shell's own tier.
    Keys,
    /// After everything else.
    Fallback,
}

/// The host a [`KeyRouter`] routes through: the app answers for its layers,
/// menus, pages and commands, and the router supplies the shell's own tiers
/// and quit vocabulary around them.
pub trait KeyHost {
    /// The app's command type.
    type Cmd: Copy + PartialEq + 'static;

    /// The shell's key vocabulary.
    fn shell_keys(&self) -> &ShellKeys<Self::Cmd>;

    /// A layer above the page answers first. `Some(response)` when taken.
    fn layer_key(&mut self, _key: &Key) -> Option<Response<()>> {
        None
    }

    /// An open menu bar answers next. `Some(response)` when taken.
    fn menu_key(&mut self, _key: &Key) -> Option<Response<()>> {
        None
    }

    /// Is a component capturing keys (a command field, a palette)?
    fn capturing(&self) -> bool {
        false
    }

    /// Does a text field own typing? Defaults to [`KeyHost::capturing`].
    fn text_focus(&self) -> bool {
        self.capturing()
    }

    /// The user interrupted (Ctrl+C with nothing to copy).
    fn interrupt(&mut self) -> Response<()> {
        Response::ignored()
    }

    /// Are there unsaved changes (quit letters must not fire)?
    fn dirty(&mut self) -> bool {
        false
    }

    /// Is a selection up (Ctrl+C copies instead of quitting)?
    fn has_selection(&self) -> bool {
        false
    }

    /// The app's answer for a shell action it bound.
    fn shell_action(&mut self, _action: ShellAction<Self::Cmd>, _key: &Key) -> Response<()> {
        Response::ignored()
    }

    /// The app's chrome (menu bar toggles, tab strips) on a non-printable key.
    fn chrome_key(&mut self, _key: &Key) -> Response<()> {
        Response::ignored()
    }

    /// The page's own key handling.
    fn page_key(&mut self, key: &Key) -> Response<()>;

    /// Esc at the top of the app (a layer's back, a dialog's cancel).
    fn esc_top(&mut self) -> Response<()> {
        Response::ignored()
    }

    /// A paste an open layer takes first. `Some(response)` when taken.
    fn layer_paste(&mut self, _text: &str) -> Option<Response<()>> {
        None
    }

    /// A paste the page takes.
    fn page_paste(&mut self, _text: &str) -> Response<()> {
        Response::ignored()
    }

    /// The quit path: run the quit guard, open its dialog if it must ask.
    fn request_quit(&mut self) -> Response<()>;

    /// Show a note on the status line.
    fn note(&mut self, _text: &str) {}
}

/// The shell's key router.
pub struct KeyRouter;

impl KeyRouter {
    /// Route one input: keys through [`KeyRouter::route`], pastes through
    /// [`KeyRouter::paste`], everything else ignored.
    pub fn input<H: KeyHost + ?Sized>(host: &mut H, input: &Input) -> Response<()> {
        match input {
            Input::Key(key) => Self::route(host, key),
            Input::Paste(text) => Self::paste(host, text),
            _ => Response::ignored(),
        }
    }

    /// Route a paste: a layer takes it first, then the page.
    pub fn paste<H: KeyHost + ?Sized>(host: &mut H, text: &str) -> Response<()> {
        host.layer_paste(text)
            .unwrap_or_else(|| host.page_paste(text))
    }

    /// Route a key: a layer above the page answers first, then an open menu
    /// bar, then the shell's own tiers around the page. (Hover suppression
    /// is the runtime's: on parity the runtime drops hover state on any key
    /// press before app code runs, so the router does no hover work.)
    pub fn route<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Response<()> {
        Self::replay(host, key)
    }

    /// Route a key as if the user had typed it (replay from a menu choice).
    pub fn replay<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Response<()> {
        Self::once(host, key)
    }

    fn once<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Response<()> {
        let quit = host.shell_keys().quit;
        if let Some(out) = host.layer_key(key) {
            if out.is_consumed() {
                return out;
            }
            if !key.mods.is_empty()
                && quit.classify(key, KeyContext::default()) == Some(LeaveKey::Quit)
            {
                return host.request_quit();
            }
            // A layer that passes still keeps the key from the page, but its
            // own response payload (invalidate, state) survives the fold.
            return out | Response::consumed();
        }
        if let Some(out) = host.menu_key(key) {
            return out;
        }
        let capture = host.capturing();
        let text = capture || host.text_focus();
        if let Some(action) = Self::bound(host.shell_keys(), &host.shell_keys().priority, key, text)
        {
            let out = Self::act(host, action, key);
            if out.is_consumed() {
                return out;
            }
        }
        if capture {
            let out = host.page_key(key);
            if out.is_consumed() {
                return out;
            }
        }

        let copy = quit.ctrl_c && ctrl_char(key, 'c') && host.has_selection();
        if quit.ctrl_c && ctrl_char(key, 'c') && !copy {
            let out = host.interrupt();
            if out.is_consumed() {
                return out;
            }
        }
        if !key.mods.is_empty() {
            let kc = KeyContext {
                text_focus: text,
                dirty: false,
                selection: copy,
            };
            if quit.classify(key, kc) == Some(LeaveKey::Quit) {
                return host.request_quit();
            }
        }
        if let Some(action) = Self::bound(host.shell_keys(), &host.shell_keys().keys, key, text) {
            let out = Self::act(host, action, key);
            if out.is_consumed() {
                return out;
            }
        }
        if !capture {
            let out = host.chrome_key(key);
            if out.is_consumed() {
                return out;
            }
            let out = host.page_key(key);
            if out.is_consumed() {
                return out;
            }
            if let Some(c) = quit.letter
                && crate::quit::is_char(key, c)
                && !text
            {
                let dirty = host.dirty();
                let kc = KeyContext {
                    text_focus: false,
                    dirty,
                    selection: false,
                };
                if quit.classify(key, kc) == Some(LeaveKey::Quit) {
                    return host.request_quit();
                }
                if dirty && let Some(note) = host.shell_keys().dirty_letter_note.clone() {
                    host.note(&note);
                    return Response::changed();
                }
            }
        }
        if let Some(action) = Self::bound(host.shell_keys(), &host.shell_keys().fallback, key, text)
        {
            return Self::act(host, action, key);
        }
        Response::ignored()
    }

    fn bound<C: Copy + PartialEq>(
        keys: &ShellKeys<C>,
        map: &[KeyBinding<C>],
        key: &Key,
        capture: bool,
    ) -> Option<ShellAction<C>> {
        let binding = lookup(map, key)?;
        let printable = key.chord().is_bare_char();
        let quiet = capture && (printable || keys.idle_only.contains(&binding.action));
        (!quiet).then_some(binding.action)
    }

    fn act<H: KeyHost + ?Sized>(
        host: &mut H,
        action: ShellAction<H::Cmd>,
        key: &Key,
    ) -> Response<()> {
        match action {
            // On parity the runtime serves Tab/BackTab out of its focus ring
            // before the router runs; these fire only for chords an app
            // bound to focus movement itself.
            ShellAction::Quit => host.request_quit(),
            ShellAction::Back => {
                let out = host.esc_top();
                if out.is_consumed() {
                    return out;
                }
                let quit = host.shell_keys().quit;
                if quit.classify(key, KeyContext::default()) == Some(LeaveKey::Quit) {
                    return host.request_quit();
                }
                match host.shell_keys().esc_note.clone() {
                    Some(note) => {
                        host.note(&note);
                        Response::changed()
                    }
                    None => Response::consumed(),
                }
            }
            other => host.shell_action(other, key),
        }
    }
}

#[cfg(test)]
mod tests;
