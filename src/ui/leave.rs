use ratatui::crossterm::event::KeyCode;

use crate::core::event::Key;
use crate::core::id::WidgetId;
use crate::theme::Strings;
use crate::widgets::button::Button;
use crate::widgets::dialog::{Dialog, DialogResult};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Stakes {
    #[default]
    Clean,

    Unsaved {
        changes: usize,
    },

    Running {
        what: String,
    },
}

impl Stakes {
    pub fn is_clean(&self) -> bool {
        *self == Self::Clean
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveKey {
    Copy,

    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyContext {
    pub text_focus: bool,

    pub dirty: bool,

    pub selection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitKeys {
    pub ctrl_c: bool,

    pub ctrl_q: bool,

    pub letter: Option<char>,

    pub esc_quits: bool,
}

impl Default for QuitKeys {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl QuitKeys {
    pub const DEFAULT: Self = Self {
        ctrl_c: true,
        ctrl_q: true,
        letter: Some('q'),
        esc_quits: false,
    };

    pub const fn letter(mut self, letter: Option<char>) -> Self {
        self.letter = letter;
        self
    }

    pub const fn esc_quits(mut self, on: bool) -> Self {
        self.esc_quits = on;
        self
    }

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

pub fn letters_allowed(cx: KeyContext) -> bool {
    !cx.text_focus && !cx.dirty
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leave<T> {
    Quit,

    To(T),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved<T> {
    Stay,

    Save(Leave<T>),

    Discard(Leave<T>),
}

#[derive(Debug, Clone)]
pub struct LeaveGuard<T> {
    pub id: WidgetId,

    pending: Option<(Leave<T>, bool)>,

    pub save: bool,
}

impl<T: Clone> LeaveGuard<T> {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            pending: None,
            save: true,
        }
    }

    pub fn save(mut self, on: bool) -> Self {
        self.save = on;
        self
    }

    pub fn request(
        &mut self,
        to: Leave<T>,
        stakes: &Stakes,
        strings: &Strings,
    ) -> Result<Leave<T>, Box<Dialog>> {
        let d = match stakes {
            Stakes::Clean => return Ok(to),
            Stakes::Unsaved { changes } if self.save => {
                self.pending = Some((to, true));
                unsaved_dialog(self.id, *changes, strings)
            }
            Stakes::Unsaved { changes } => {
                self.pending = Some((to, false));
                discard_dialog(self.id, *changes, strings)
            }
            Stakes::Running { what } => {
                self.pending = Some((to, false));
                running_dialog(self.id, what, strings)
            }
        };
        Err(Box::new(d))
    }

    pub fn request_quit(
        &mut self,
        stakes: &Stakes,
        strings: &Strings,
    ) -> Result<Leave<T>, Box<Dialog>> {
        self.request(Leave::Quit, stakes, strings)
    }

    pub fn is_dialog(&self, dialog: WidgetId) -> bool {
        dialog == self.id
    }

    pub fn pending(&self) -> Option<&Leave<T>> {
        self.pending.as_ref().map(|(to, _)| to)
    }

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

pub fn unsaved_dialog(id: WidgetId, changes: usize, s: &Strings) -> Dialog {
    let body = Strings::fill(s.leave_unsaved_body, &[&lost(changes, s)]);
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

pub fn discard_dialog(id: WidgetId, changes: usize, s: &Strings) -> Dialog {
    let body = Strings::fill(s.leave_discard_body, &[&lost(changes, s)]);
    Dialog::destructive(id, s.leave_unsaved_title, &body, s.discard)
}

fn lost(changes: usize, s: &Strings) -> String {
    match changes {
        0 => s.leave_unsaved_some.to_owned(),
        1 => s.leave_unsaved_one.to_owned(),
        n => Strings::fill(s.leave_unsaved_many, &[&n]),
    }
}

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

        assert_eq!(k.classify(&ctrl('q'), dirty), Some(LeaveKey::Quit));
        assert_eq!(k.classify(&ctrl('c'), typing), Some(LeaveKey::Quit));

        let sel = KeyContext {
            selection: true,
            ..clean
        };
        assert_eq!(k.classify(&ctrl('c'), sel), Some(LeaveKey::Copy));

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
        assert!(g.is_dialog(d.id));
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
    fn a_guard_that_cannot_save_offers_cancel_and_discard() {
        let s = Strings::DEFAULT;
        let mut g: LeaveGuard<()> = LeaveGuard::new(WidgetId::of("leave")).save(false);
        let d = g
            .request_quit(&Stakes::Unsaved { changes: 1 }, &s)
            .unwrap_err();
        let labels: Vec<_> = d.actions.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, ["Cancel", "Discard"]);
        assert_eq!(d.initial_focus, d.actions[0].id, "Cancel is the default");
        assert_eq!(
            g.resolve(DialogResult::Action(1)),
            Resolved::Discard(Leave::Quit)
        );
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
