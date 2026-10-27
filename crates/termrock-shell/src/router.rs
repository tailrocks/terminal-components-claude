//! Ported from PR #26 `src/shell/router.rs` (commit `f4867e9b5`). The parity

use termrock_core::event::{Chord, Input, Key, KeyCode};
use termrock_core::keys::KeyModifiers;
use termrock_core::response::Response;

use crate::quit::{KeyContext, LeaveKey, QuitKeys, ctrl_char};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding<C> {
    pub chord: Chord,

    pub action: ShellAction<C>,

    pub priority: u8,
}

pub fn chord(code: KeyCode, mods: KeyModifiers) -> Chord {
    Chord { code, mods }
}

pub fn lookup<'a, C: PartialEq>(map: &'a [KeyBinding<C>], key: &Key) -> Option<&'a KeyBinding<C>> {
    let chord = key.chord();
    map.iter()
        .find(|b| b.chord.code == chord.code && b.chord.mods == chord.mods)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAction<C = ()> {
    Quit,

    Help,

    MenuBar,

    NextPage,

    PrevPage,

    FocusNext,

    FocusPrev,

    Back,

    App(C),
}

impl<C> From<C> for ShellAction<C> {
    fn from(c: C) -> Self {
        Self::App(c)
    }
}

#[derive(Debug, Clone)]
pub struct ShellKeys<C = ()> {
    pub quit: QuitKeys,

    pub priority: Vec<KeyBinding<C>>,

    pub keys: Vec<KeyBinding<C>>,

    pub fallback: Vec<KeyBinding<C>>,

    pub idle_only: Vec<ShellAction<C>>,

    pub esc_note: Option<String>,

    pub dirty_letter_note: Option<String>,
}

impl<C> Default for ShellKeys<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ShellKeys<C> {
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

    #[must_use]
    pub fn quit(mut self, quit: QuitKeys) -> Self {
        self.quit = quit;
        self
    }

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

    #[must_use]
    pub fn idle_only(mut self, action: ShellAction<C>) -> Self {
        self.idle_only.push(action);
        self
    }

    #[must_use]
    pub fn esc_note(mut self, note: Option<&str>) -> Self {
        self.esc_note = note.map(Into::into);
        self
    }

    #[must_use]
    pub fn dirty_letter_note(mut self, note: Option<&str>) -> Self {
        self.dirty_letter_note = note.map(Into::into);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Priority,

    Keys,

    Fallback,
}

pub trait KeyHost {
    type Cmd: Copy + PartialEq + 'static;

    fn shell_keys(&self) -> &ShellKeys<Self::Cmd>;

    fn layer_key(&mut self, _key: &Key) -> Option<Response<()>> {
        None
    }

    fn menu_key(&mut self, _key: &Key) -> Option<Response<()>> {
        None
    }

    fn capturing(&self) -> bool {
        false
    }

    fn text_focus(&self) -> bool {
        self.capturing()
    }

    fn interrupt(&mut self) -> Response<()> {
        Response::ignored()
    }

    fn dirty(&mut self) -> bool {
        false
    }

    fn has_selection(&self) -> bool {
        false
    }

    fn shell_action(&mut self, _action: ShellAction<Self::Cmd>, _key: &Key) -> Response<()> {
        Response::ignored()
    }

    fn chrome_key(&mut self, _key: &Key) -> Response<()> {
        Response::ignored()
    }

    fn page_key(&mut self, key: &Key) -> Response<()>;

    fn esc_top(&mut self) -> Response<()> {
        Response::ignored()
    }

    fn layer_paste(&mut self, _text: &str) -> Option<Response<()>> {
        None
    }

    fn page_paste(&mut self, _text: &str) -> Response<()> {
        Response::ignored()
    }

    fn request_quit(&mut self) -> Response<()>;

    fn note(&mut self, _text: &str) {}
}

pub struct KeyRouter;

impl KeyRouter {
    pub fn input<H: KeyHost + ?Sized>(host: &mut H, input: &Input) -> Response<()> {
        match input {
            Input::Key(key) => Self::route(host, key),
            Input::Paste(text) => Self::paste(host, text),
            _ => Response::ignored(),
        }
    }

    pub fn paste<H: KeyHost + ?Sized>(host: &mut H, text: &str) -> Response<()> {
        host.layer_paste(text)
            .unwrap_or_else(|| host.page_paste(text))
    }

    pub fn route<H: KeyHost + ?Sized>(host: &mut H, key: &Key) -> Response<()> {
        Self::replay(host, key)
    }

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
