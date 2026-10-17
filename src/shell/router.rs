//! Key routing (plan §1.8, `KeyRouter` and `ShellKeys`): one fixed order
//! every app's keys go through, so no app writes its own `on_key` ladder.
//!
//! The order, first taker wins:
//!
//! 1. **Too small**: only the quit keys work (through the app's quit
//!    guard); every other key is consumed.
//! 2. **Layer** ([`KeyHost::layer_key`]): an open dialog, popup, overlay
//!    or full-screen sequence holds the keyboard. A key it does not take
//!    still quits by chord (Ctrl+Q; Ctrl+C with nothing selected), never
//!    reaches the page.
//! 3. **Open menu bar** ([`KeyHost::menu_key`]), then the **priority**
//!    bindings ([`ShellKeys::priority`]): app chords that act even over an
//!    editing field (a menu's key equivalents), the plain printable ones
//!    and [`ShellKeys::idle_only`] actions excepted.
//! 4. **Capture** ([`KeyHost::capturing`]): a field that is editing, a
//!    page menu, an attached program gets the key first, so printable
//!    keys type (WCAG 2.1.4) and a chord the field binds is the field's.
//!    A key it passes on continues below, but the shell's letter
//!    bindings stay quiet while it captures.
//! 5. **Interrupt**: Ctrl+C goes to [`KeyHost::interrupt`]
//!    ([`super::Page::on_interrupt`]: cancel a running query, the
//!    remaining steps of a plan) before it may quit.
//! 6. **Shell keys before the page** ([`ShellKeys::keys`]): the quit
//!    chords, help, the menu bar, next and previous page, and the app's
//!    own commands ([`ShellAction::App`]).
//! 7. **Focused chrome** ([`KeyHost::chrome_key`]): a focused menu bar
//!    (K11), tab strip or sidebar outside the page.
//! 8. **Page** ([`KeyHost::page_key`]) and its focused widget. A plain
//!    Enter or Space that changed something flashes the focused control.
//! 9. **Fallback** ([`ShellKeys::fallback`]): the quit letter (never over
//!    a field or unsaved work), Tab and Shift+Tab, and Esc, which runs the
//!    page's [`KeyHost::esc_top`] (the page's ladder, then the app's back
//!    navigation) and otherwise quits only under [`QuitKeys::esc_quits`].
//!
//! Single-character shortcuts win over type-ahead: the router runs the
//! whole order inside [`route_key`] (CO2, `TypeAheadPrecedence`), so a
//! letter a shortcut takes is never typed into a focused list, and a
//! letter none takes types.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::core::event::{Key, Outcome};
use crate::core::keymap::{KeyBinding, KeyMap, lookup};
use crate::core::list::route_key;
use crate::ui::leave::{KeyContext, LeaveKey, QuitKeys};

use super::App;

/// What a shell key binding does. `C` is the app's own command type for
/// app-wide chords (tablepro's Run, holla's Close tab).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellAction<C = ()> {
    /// Quit through the app's guard (the quit keys of [`QuitKeys`] are
    /// classified on their own; bind this for an extra quit key).
    Quit,
    /// Open the keyboard reference.
    Help,
    /// Open the menu bar (F10; Ctrl+F2 as macOS's "move focus to the menu
    /// bar", since F10 is a media key on Mac keyboards, SI10).
    MenuBar,
    /// Show the next page or tab (Ctrl+PgDn, SI23).
    NextPage,
    /// Show the previous page or tab (Ctrl+PgUp, SI23).
    PrevPage,
    /// Move focus to the next control (Tab).
    FocusNext,
    /// Move focus to the previous control (Shift+Tab).
    FocusPrev,
    /// Esc at the top of the page: the page's ladder, then back.
    Back,
    /// An app command.
    App(C),
}

/// The shell's key bindings: what every app gets by default, with every
/// binding overridable (rebind, unbind, add the app's own commands).
#[derive(Debug, Clone)]
pub struct ShellKeys<C = ()> {
    /// Which keys quit, and when ([`QuitKeys::DEFAULT`]).
    pub quit: QuitKeys,
    /// Bindings tried before a capturing field (app chords such as Run,
    /// which act while the query editor edits). Plain letters and
    /// [`Self::idle_only`] actions here are skipped while a field captures.
    /// Empty by default.
    pub priority: KeyMap<ShellAction<C>>,
    /// Bindings tried before the page (chords and function keys the page
    /// does not own), after a capturing field passed the key on. Plain letters here are skipped while a field
    /// captures the keyboard.
    pub keys: KeyMap<ShellAction<C>>,
    /// Bindings tried after the page passed on the key (Tab, Esc, `?`,
    /// letters a page may claim first). Plain letters here are skipped
    /// while a field captures the keyboard.
    pub fallback: KeyMap<ShellAction<C>>,
    /// Actions that stay quiet while a field captures the keyboard, chords
    /// included (Ctrl+W closes a tab, never over a field that is editing).
    pub idle_only: Vec<ShellAction<C>>,
    /// The footer note when Esc reaches the top and does not quit.
    /// Default "Esc never quits · Ctrl+Q quits"; `None` stays quiet.
    pub esc_note: Option<String>,
    /// The footer note when the quit letter is refused over unsaved work.
    /// Default "Unsaved changes · Ctrl+Q asks before quitting".
    pub dirty_letter_note: Option<String>,
}

impl<C> Default for ShellKeys<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ShellKeys<C> {
    /// The standard bindings: F1 help, F10 and Ctrl+F2 the menu bar,
    /// Ctrl+PgDn and Ctrl+PgUp next and previous page; after the page,
    /// `?` help, Tab and Shift+Tab focus, Esc back.
    pub fn new() -> Self {
        Self {
            quit: QuitKeys::DEFAULT,
            priority: vec![],
            keys: vec![
                (KeyBinding::key(KeyCode::F(1)), ShellAction::Help),
                (KeyBinding::key(KeyCode::F(10)), ShellAction::MenuBar),
                (
                    KeyBinding::new(KeyCode::F(2), KeyModifiers::CONTROL),
                    ShellAction::MenuBar,
                ),
                (
                    KeyBinding::new(KeyCode::PageDown, KeyModifiers::CONTROL),
                    ShellAction::NextPage,
                ),
                (
                    KeyBinding::new(KeyCode::PageUp, KeyModifiers::CONTROL),
                    ShellAction::PrevPage,
                ),
            ],
            fallback: vec![
                (KeyBinding::char('?'), ShellAction::Help),
                (KeyBinding::key(KeyCode::Tab), ShellAction::FocusNext),
                (
                    KeyBinding::new(KeyCode::BackTab, KeyModifiers::SHIFT),
                    ShellAction::FocusPrev,
                ),
                (KeyBinding::key(KeyCode::BackTab), ShellAction::FocusPrev),
                (KeyBinding::key(KeyCode::Esc), ShellAction::Back),
            ],
            idle_only: vec![],
            esc_note: Some("Esc never quits · Ctrl+Q quits".into()),
            dirty_letter_note: Some("Unsaved changes · Ctrl+Q asks before quitting".into()),
        }
    }

    /// With these quit keys.
    pub fn quit(mut self, quit: QuitKeys) -> Self {
        self.quit = quit;
        self
    }

    /// Bind `key` to `action` before the page (first match wins, so it
    /// overrides a standard binding of the same key).
    pub fn bind(mut self, key: impl Into<KeyBinding>, action: ShellAction<C>) -> Self {
        self.keys.insert(0, (key.into(), action));
        self
    }

    /// Bind `key` to `action` before a capturing field
    /// ([`Self::priority`]).
    pub fn bind_priority(mut self, key: impl Into<KeyBinding>, action: ShellAction<C>) -> Self {
        self.priority.insert(0, (key.into(), action));
        self
    }

    /// Bind `key` to `action` after the page.
    pub fn bind_fallback(mut self, key: impl Into<KeyBinding>, action: ShellAction<C>) -> Self {
        self.fallback.insert(0, (key.into(), action));
        self
    }

    /// Remove every binding of `key` (before and after the page).
    pub fn unbind(mut self, key: impl Into<KeyBinding>) -> Self {
        let key = key.into();
        self.priority.retain(|(b, _)| *b != key);
        self.keys.retain(|(b, _)| *b != key);
        self.fallback.retain(|(b, _)| *b != key);
        self
    }

    /// `action` stays quiet while a field captures the keyboard
    /// ([`Self::idle_only`]).
    pub fn idle_only(mut self, action: ShellAction<C>) -> Self {
        self.idle_only.push(action);
        self
    }

    /// See [`Self::esc_note`].
    pub fn esc_note(mut self, note: Option<&str>) -> Self {
        self.esc_note = note.map(Into::into);
        self
    }

    /// See [`Self::dirty_letter_note`].
    pub fn dirty_letter_note(mut self, note: Option<&str>) -> Self {
        self.dirty_letter_note = note.map(Into::into);
        self
    }
}

/// What the router needs from an app: one hook per stage of the order
/// (module docs). Default bodies make a stage a no-op, so a small app
/// implements [`Self::shell_keys`], [`Self::page_key`] and
/// [`Self::request_quit`] only.
pub trait KeyHost: App {
    /// The app's own command type ([`ShellAction::App`]).
    type Cmd: Copy + PartialEq + 'static;

    fn shell_keys(&self) -> &ShellKeys<Self::Cmd>;

    /// An open layer takes the key. `None` when no layer is open.
    fn layer_key(&mut self, _key: &Key) -> Option<Outcome> {
        None
    }

    /// An open menu bar takes the key. `None` when no menu is open.
    fn menu_key(&mut self, _key: &Key) -> Option<Outcome> {
        None
    }

    /// The page captures the keyboard: a field is editing, a page menu is
    /// open, an attached program has the keys.
    fn capturing(&self) -> bool {
        false
    }

    /// Ctrl+C, before it may quit ([`super::Page::on_interrupt`]).
    fn interrupt(&mut self) -> Outcome {
        Outcome::Ignored
    }

    /// Something on screen is unsaved: the quit letter stays quiet.
    fn dirty(&mut self) -> bool {
        false
    }

    /// There is a selection Ctrl+C would copy.
    fn selection(&self) -> bool {
        false
    }

    /// Run a shell action. `Ignored` passes the key on. The router runs
    /// [`ShellAction::FocusNext`], [`ShellAction::FocusPrev`],
    /// [`ShellAction::Back`] and [`ShellAction::Quit`] itself.
    fn shell_action(&mut self, _action: ShellAction<Self::Cmd>, _key: &Key) -> Outcome {
        Outcome::Ignored
    }

    /// A focused control outside the page (a menu bar, a tab strip, a
    /// sidebar) takes the key.
    fn chrome_key(&mut self, _key: &Key) -> Outcome {
        Outcome::Ignored
    }

    /// The shown page (and its focused widget) takes the key.
    fn page_key(&mut self, key: &Key) -> Outcome;

    /// Esc at the top: the page's ladder ([`super::Page::on_esc_top`])
    /// and the app's back navigation. `Ignored` when there is nowhere to
    /// go back to.
    fn esc_top(&mut self) -> Outcome {
        Outcome::Ignored
    }

    /// Quit through the app's guard.
    fn request_quit(&mut self) -> Outcome;

    /// Show a footer note (an information status).
    fn note(&mut self, _text: &str) {}
}

/// Routes keys in the shell's fixed order (module docs).
pub struct KeyRouter;

impl KeyRouter {
    /// Route a typed `key` through `host`. Typing hides the hover
    /// highlight until the pointer moves again.
    pub fn route<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Outcome {
        host.shell_mut().pointer.suppress_hover();
        Self::replay(host, key)
    }

    /// Route `key` in the same order for a click that stands for it (a
    /// footer hint, a menu item): the pointer keeps its hover.
    pub fn replay<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Outcome {
        route_key(key, || Self::once(host, key))
    }

    fn once<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Outcome {
        let quit = host.shell_keys().quit;
        if host.shell().too_small {
            let letter = quit.letter.is_some_and(|c| key.is_char(c));
            if letter || quit.classify(key, KeyContext::default()) == Some(LeaveKey::Quit) {
                return host.request_quit().or(Outcome::Consumed);
            }
            return Outcome::Consumed;
        }
        if let Some(out) = host.layer_key(key) {
            if out.consumed() {
                return out;
            }
            if key.chord() && quit.classify(key, KeyContext::default()) == Some(LeaveKey::Quit) {
                return host.request_quit();
            }
            return Outcome::Consumed;
        }
        if let Some(out) = host.menu_key(key) {
            return out;
        }
        let capture = host.capturing();
        if let Some(action) =
            Self::bound(host.shell_keys(), &host.shell_keys().priority, key, capture)
        {
            let out = Self::act(host, action, key);
            if out.consumed() {
                return out;
            }
        }
        if capture {
            let out = host.page_key(key);
            if out.consumed() {
                return out;
            }
        }
        if quit.ctrl_c && key.ctrl_char('c') {
            let out = host.interrupt();
            if out.consumed() {
                return out;
            }
        }
        if key.chord() {
            let kc = KeyContext {
                text_focus: capture,
                dirty: false,
                selection: host.selection(),
            };
            if quit.classify(key, kc) == Some(LeaveKey::Quit) {
                return host.request_quit();
            }
        }
        if let Some(action) = Self::bound(host.shell_keys(), &host.shell_keys().keys, key, capture)
        {
            let out = Self::act(host, action, key);
            if out.consumed() {
                return out;
            }
        }
        if !capture {
            let out = host.chrome_key(key);
            if out.consumed() {
                return out;
            }
            let activating = matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) && key.plain();
            let out = host.page_key(key);
            if out.consumed() {
                if activating
                    && out == Outcome::Changed
                    && let Some(f) = host.shell().focus.current()
                {
                    let now = host.now_ms();
                    host.shell_mut().pointer.flash(f, now);
                }
                return out;
            }
            if let Some(c) = quit.letter
                && key.is_char(c)
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
                    return Outcome::Changed;
                }
            }
        }
        if let Some(action) =
            Self::bound(host.shell_keys(), &host.shell_keys().fallback, key, capture)
        {
            return Self::act(host, action, key);
        }
        Outcome::Ignored
    }

    /// The action `key` is bound to in `map`; a plain printable binding
    /// and an [`ShellKeys::idle_only`] action stay quiet while a field
    /// captures the keyboard.
    fn bound<C: Copy + PartialEq>(
        keys: &ShellKeys<C>,
        map: &[(KeyBinding, ShellAction<C>)],
        key: &Key,
        capture: bool,
    ) -> Option<ShellAction<C>> {
        let action = lookup(map, key)?;
        let printable = matches!(key.code, KeyCode::Char(_)) && !key.chord();
        let quiet = capture && (printable || keys.idle_only.contains(&action));
        (!quiet).then_some(action)
    }

    fn act<H: KeyHost + ?Sized>(host: &mut H, action: ShellAction<H::Cmd>, key: &Key) -> Outcome {
        match action {
            ShellAction::FocusNext => {
                host.shell_mut().focus_next();
                Outcome::Changed
            }
            ShellAction::FocusPrev => {
                host.shell_mut().focus_prev();
                Outcome::Changed
            }
            ShellAction::Quit => host.request_quit(),
            ShellAction::Back => {
                let out = host.esc_top();
                if out.consumed() {
                    return out;
                }
                let quit = host.shell_keys().quit;
                if quit.classify(key, KeyContext::default()) == Some(LeaveKey::Quit) {
                    return host.request_quit();
                }
                match host.shell_keys().esc_note.clone() {
                    Some(note) => {
                        host.note(&note);
                        Outcome::Changed
                    }
                    None => Outcome::Consumed,
                }
            }
            other => host.shell_action(other, key),
        }
    }
}

#[cfg(test)]
#[path = "router_tests.rs"]
mod tests;
