//! Leaving safely. Every way out of a screen or the app (a route change,
//! `q`, Esc, Ctrl+C, Ctrl+Q) goes through one guard, so unsaved edits and
//! running work are never thrown away without a question, and a clean
//! quit never asks one.
//!
//! - [`QuitKeys`] says which keys ask to leave and when: Ctrl+C copies a
//!   selection first, a single-letter quit (`q`) never fires while a text
//!   field has focus or anything is unsaved, and Esc never quits by
//!   default.
//! - [`Stakes`] is what leaving would lose, reported by the app or page.
//! - [`LeaveGuard`] turns a request plus its stakes into "go now" or a
//!   dialog (Save / Discard / Cancel for unsaved edits, Stop / Stay for
//!   running work), remembers where the user was going, and resolves the
//!   dialog's answer back into that destination.

use ratatui::crossterm::event::KeyCode;

use crate::core::event::Key;
use crate::core::id::WidgetId;
use crate::theme::Strings;
use crate::widgets::button::Button;
use crate::widgets::dialog::{Dialog, DialogResult};

/// What leaving the current screen (or the app) would lose.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Stakes {
    /// Nothing: leave without asking.
    #[default]
    Clean,
    /// Unsaved edits; `changes` counts them (0 when unknown).
    Unsaved { changes: usize },
    /// Work still running that leaving stops; `what` names it for the
    /// dialog ("2 activities").
    Running { what: String },
}

impl Stakes {
    /// Nothing would be lost.
    pub fn is_clean(&self) -> bool {
        *self == Self::Clean
    }

    /// The stronger of two stakes: running work over unsaved edits over
    /// nothing (counts of unsaved edits add up).
    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (r @ Self::Running { .. }, _) | (_, r @ Self::Running { .. }) => r,
            (Self::Unsaved { changes: a }, Self::Unsaved { changes: b }) => {
                Self::Unsaved { changes: a + b }
            }
            (u @ Self::Unsaved { .. }, _) | (_, u @ Self::Unsaved { .. }) => u,
            _ => Self::Clean,
        }
    }
}

/// What a key asks for ([`QuitKeys::classify`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveKey {
    /// Ctrl+C over a selection: copy it, do not quit.
    Copy,
    /// Ask to quit the app (through [`LeaveGuard::request`]).
    Quit,
}

/// What the app knows about the keyboard's target when a key arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyContext {
    /// A text-input control has focus (editing or not): letters type.
    pub text_focus: bool,
    /// Something on screen is unsaved.
    pub dirty: bool,
    /// There is a text selection Ctrl+C would copy.
    pub selection: bool,
}

/// Which keys ask to quit, and when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitKeys {
    /// Ctrl+C quits (through the guard) when nothing is selected; with a
    /// selection it copies. Apps give the page the first chance at it.
    /// Default `true`.
    pub ctrl_c: bool,
    /// Ctrl+Q quits (through the guard). Default `true`.
    pub ctrl_q: bool,
    /// A plain letter that quits (through the guard), never while a text
    /// field has focus or anything is unsaved. Default `Some('q')`.
    pub letter: Option<char>,
    /// Esc at the top level quits (through the guard). Default `false`:
    /// Esc closes, cancels and goes back, but never quits.
    pub esc_quits: bool,
}

impl Default for QuitKeys {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl QuitKeys {
    /// Ctrl+C, Ctrl+Q and `q`; Esc never quits.
    pub const DEFAULT: Self = Self {
        ctrl_c: true,
        ctrl_q: true,
        letter: Some('q'),
        esc_quits: false,
    };

    /// See [`Self::letter`].
    pub const fn letter(mut self, letter: Option<char>) -> Self {
        self.letter = letter;
        self
    }

    /// See [`Self::esc_quits`].
    pub const fn esc_quits(mut self, on: bool) -> Self {
        self.esc_quits = on;
        self
    }

    /// What `key` asks for in `cx`, if it is a quit key at all.
    pub fn classify(&self, key: &Key, cx: KeyContext) -> Option<LeaveKey> {
        if self.ctrl_c && key.ctrl_char('c') {
            return Some(if cx.selection {
                LeaveKey::Copy
            } else {
                LeaveKey::Quit
            });
        }
        if self.ctrl_q && key.ctrl_char('q') {
            return Some(LeaveKey::Quit);
        }
        if let Some(c) = self.letter
            && key.is_char(c)
            && letters_allowed(cx)
        {
            return Some(LeaveKey::Quit);
        }
        if self.esc_quits && key.is(KeyCode::Esc) && !cx.text_focus {
            return Some(LeaveKey::Quit);
        }
        None
    }
}

/// Whether single-letter shortcuts (quit, back, navigation) may fire: never
/// while a text field has focus (the letter is typing) or anything is
/// unsaved (a stray letter must not leave the form).
pub fn letters_allowed(cx: KeyContext) -> bool {
    !cx.text_focus && !cx.dirty
}

/// Where a request to leave goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leave<T> {
    /// Quit the app.
    Quit,
    /// Go to another route or screen.
    To(T),
}

/// The answer of a [`LeaveGuard`] dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved<T> {
    /// Cancel / Stay: remain where the user is.
    Stay,
    /// Save, then go.
    Save(Leave<T>),
    /// Throw the edits away (or stop the running work), then go.
    Discard(Leave<T>),
}

/// One guard for every way out. Ask with [`Self::request`]; on `Err`, show
/// the dialog; when it closes, pass its result to [`Self::resolve`].
#[derive(Debug, Clone)]
pub struct LeaveGuard<T> {
    /// The id of the dialog this guard opens ([`Self::owns`]).
    pub id: WidgetId,
    /// The request waiting for the dialog's answer.
    pending: Option<(Leave<T>, bool)>,
}

impl<T: Clone> LeaveGuard<T> {
    pub fn new(id: WidgetId) -> Self {
        Self { id, pending: None }
    }

    /// Ask to leave for `to` with `stakes` at risk: `Ok(to)` to go now
    /// (nothing would be lost), or `Err(dialog)` to show first.
    pub fn request(
        &mut self,
        to: Leave<T>,
        stakes: &Stakes,
        strings: &Strings,
    ) -> Result<Leave<T>, Box<Dialog>> {
        let d = match stakes {
            Stakes::Clean => return Ok(to),
            Stakes::Unsaved { changes } => {
                self.pending = Some((to, true));
                unsaved_dialog(self.id, *changes, strings)
            }
            Stakes::Running { what } => {
                self.pending = Some((to, false));
                running_dialog(self.id, what, strings)
            }
        };
        Err(Box::new(d))
    }

    /// [`Self::request`] for quitting the app.
    pub fn request_quit(
        &mut self,
        stakes: &Stakes,
        strings: &Strings,
    ) -> Result<Leave<T>, Box<Dialog>> {
        self.request(Leave::Quit, stakes, strings)
    }

    /// Whether `dialog` is the one this guard opened.
    pub fn owns(&self, dialog: WidgetId) -> bool {
        dialog == self.id
    }

    /// Where the open dialog would go.
    pub fn pending(&self) -> Option<&Leave<T>> {
        self.pending.as_ref().map(|(to, _)| to)
    }

    /// The guard's dialog closed with `result`: what to do now.
    pub fn resolve(&mut self, result: DialogResult) -> Resolved<T> {
        let Some((to, unsaved)) = self.pending.take() else {
            return Resolved::Stay;
        };
        match (result, unsaved) {
            (DialogResult::Action(2), true) => Resolved::Save(to),
            (DialogResult::Action(1), _) => Resolved::Discard(to),
            _ => Resolved::Stay,
        }
    }
}

/// The unsaved-changes dialog: Cancel, Discard, Save (Save focused, Esc
/// and a backdrop click cancel). Action indices are 0, 1, 2.
pub fn unsaved_dialog(id: WidgetId, changes: usize, s: &Strings) -> Dialog {
    let lost = match changes {
        0 => s.leave_unsaved_some.to_owned(),
        1 => s.leave_unsaved_one.to_owned(),
        n => Strings::fill(s.leave_unsaved_many, &[&n]),
    };
    let body = Strings::fill(s.leave_unsaved_body, &[&lost]);
    let mut d = Dialog::confirm(id, s.leave_unsaved_title, &body, s.save).actions(
        vec![
            Button::subtle(id.sub("cancel"), s.cancel),
            Button::danger(id.sub("discard"), s.discard),
            Button::primary(id.sub("save"), s.save),
        ],
        Some(0),
    );
    d.initial_focus = id.sub("save");
    d
}

/// The running-work dialog: Stay, Stop and leave (Stay focused).
pub fn running_dialog(id: WidgetId, what: &str, s: &Strings) -> Dialog {
    let body = Strings::fill(s.leave_running_body, &[&what]);
    Dialog::destructive(id, s.leave_running_title, &body, s.leave_stop).cancel_label(s.leave_stay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn ctrl(c: char) -> Key {
        Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::CONTROL,
        }
    }

    #[test]
    fn quit_keys_respect_text_focus_dirt_and_selection() {
        let k = QuitKeys::DEFAULT;
        let clean = KeyContext::default();
        assert_eq!(
            k.classify(&key(KeyCode::Char('q')), clean),
            Some(LeaveKey::Quit)
        );
        // `q` types into a field and never leaves a dirty form
        let typing = KeyContext {
            text_focus: true,
            ..clean
        };
        let dirty = KeyContext {
            dirty: true,
            ..clean
        };
        assert_eq!(k.classify(&key(KeyCode::Char('q')), typing), None);
        assert_eq!(k.classify(&key(KeyCode::Char('q')), dirty), None);
        // Ctrl+Q and Ctrl+C still ask (the guard decides)
        assert_eq!(k.classify(&ctrl('q'), dirty), Some(LeaveKey::Quit));
        assert_eq!(k.classify(&ctrl('c'), typing), Some(LeaveKey::Quit));
        // Ctrl+C copies a selection
        let sel = KeyContext {
            selection: true,
            ..clean
        };
        assert_eq!(k.classify(&ctrl('c'), sel), Some(LeaveKey::Copy));
        // Esc never quits by default
        assert_eq!(k.classify(&key(KeyCode::Esc), clean), None);
        assert_eq!(
            k.esc_quits(true).classify(&key(KeyCode::Esc), clean),
            Some(LeaveKey::Quit)
        );
        assert_eq!(
            k.letter(None).classify(&key(KeyCode::Char('q')), clean),
            None
        );
    }

    #[test]
    fn a_clean_leave_goes_now_and_a_dirty_one_asks_save_discard_cancel() {
        let s = Strings::DEFAULT;
        let mut g: LeaveGuard<&str> = LeaveGuard::new(WidgetId::of("leave"));
        assert_eq!(
            g.request(Leave::To("home"), &Stakes::Clean, &s).ok(),
            Some(Leave::To("home"))
        );
        let d = g
            .request(Leave::To("home"), &Stakes::Unsaved { changes: 2 }, &s)
            .unwrap_err();
        assert!(g.owns(d.id));
        let labels: Vec<_> = d.actions.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, ["Cancel", "Discard", "Save"]);
        assert_eq!(d.initial_focus, d.actions[2].id, "Save is the default");
        assert!(
            matches!(&d.body, crate::widgets::dialog::DialogBody::Text(t) if t.contains("2 unsaved changes"))
        );
        assert_eq!(g.pending(), Some(&Leave::To("home")));
        assert_eq!(
            g.resolve(DialogResult::Action(2)),
            Resolved::Save(Leave::To("home"))
        );
        assert_eq!(g.pending(), None);
        g.request_quit(&Stakes::Unsaved { changes: 1 }, &s)
            .unwrap_err();
        assert_eq!(
            g.resolve(DialogResult::Action(1)),
            Resolved::Discard(Leave::Quit)
        );
        g.request_quit(&Stakes::Unsaved { changes: 1 }, &s)
            .unwrap_err();
        assert_eq!(g.resolve(DialogResult::Action(0)), Resolved::Stay);
    }

    #[test]
    fn running_work_asks_stop_or_stay_with_stay_focused() {
        let s = Strings::DEFAULT;
        let mut g: LeaveGuard<()> = LeaveGuard::new(WidgetId::of("leave"));
        let d = g
            .request_quit(
                &Stakes::Running {
                    what: "2 activities".into(),
                },
                &s,
            )
            .unwrap_err();
        let labels: Vec<_> = d.actions.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, ["Stay", "Stop and leave"]);
        assert_eq!(d.initial_focus, d.actions[0].id);
        // index 2 does not exist here: only Stop discards
        assert_eq!(
            g.resolve(DialogResult::Action(1)),
            Resolved::Discard(Leave::Quit)
        );
    }

    #[test]
    fn stakes_combine_to_the_strongest() {
        let u = |n| Stakes::Unsaved { changes: n };
        let r = Stakes::Running { what: "x".into() };
        assert_eq!(Stakes::Clean.max(u(1)), u(1));
        assert_eq!(u(1).max(u(2)), u(3));
        assert_eq!(u(1).max(r.clone()), r);
        assert!(Stakes::Clean.max(Stakes::Clean).is_clean());
    }
}
