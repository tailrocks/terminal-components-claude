//! Ported from PR #26 `src/ui/leave.rs` (commit `f4867e9b5`).

use termrock_core::event::Key;
use termrock_core::id::Id;
use termrock_core::keys::KeyCode;

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
            (Self::Unsaved { changes: a }, Self::Unsaved { changes: b }) => Self::Unsaved {
                changes: a.saturating_add(b),
            },
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

    #[must_use]
    pub const fn letter(mut self, letter: Option<char>) -> Self {
        self.letter = letter;
        self
    }

    #[must_use]
    pub const fn esc_quits(mut self, on: bool) -> Self {
        self.esc_quits = on;
        self
    }

    pub fn classify(&self, key: &Key, cx: KeyContext) -> Option<LeaveKey> {
        if self.ctrl_c && ctrl_char(key, 'c') {
            return Some(if cx.selection {
                LeaveKey::Copy
            } else {
                LeaveKey::Quit
            });
        }
        if self.ctrl_q && ctrl_char(key, 'q') {
            return Some(LeaveKey::Quit);
        }
        if let Some(c) = self.letter
            && is_char(key, c)
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

pub(crate) fn ctrl_char(key: &Key, c: char) -> bool {
    key.code == KeyCode::Char(c) && key.mods == termrock_core::keys::KeyModifiers::CONTROL
}

pub(crate) fn is_char(key: &Key, c: char) -> bool {
    key.code == KeyCode::Char(c)
        && key
            .mods
            .difference(termrock_core::keys::KeyModifiers::SHIFT)
            .is_empty()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaveDialog {
    pub id: Id,

    pub kind: LeaveDialogKind,

    pub title: &'static str,

    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveDialogKind {
    SaveDiscardCancel,

    DiscardOnly,

    StopStay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    First,

    Second,

    Third,
}

#[derive(Debug, Clone)]
pub struct LeaveGuard<T> {
    pub id: Id,
    pending: Option<(Leave<T>, bool)>,

    pub save: bool,
}

impl<T: Clone> LeaveGuard<T> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            pending: None,
            save: true,
        }
    }

    #[must_use]
    pub fn save(mut self, on: bool) -> Self {
        self.save = on;
        self
    }

    pub fn request(&mut self, to: Leave<T>, stakes: &Stakes) -> Result<Leave<T>, LeaveDialog> {
        let (kind, body) = match stakes {
            Stakes::Clean => return Ok(to),
            Stakes::Unsaved { changes } if self.save => {
                self.pending = Some((to, true));
                (LeaveDialogKind::SaveDiscardCancel, unsaved_body(*changes))
            }
            Stakes::Unsaved { changes } => {
                self.pending = Some((to, false));
                (LeaveDialogKind::DiscardOnly, unsaved_body(*changes))
            }
            Stakes::Running { what } => {
                self.pending = Some((to, false));
                (LeaveDialogKind::StopStay, running_body(what))
            }
        };
        Err(LeaveDialog {
            id: self.id,
            kind,
            title: if matches!(stakes, Stakes::Running { .. }) {
                "Work in progress"
            } else {
                "Leave with unsaved changes?"
            },
            body,
        })
    }

    pub fn request_quit(&mut self, stakes: &Stakes) -> Result<Leave<T>, LeaveDialog> {
        self.request(Leave::Quit, stakes)
    }

    pub fn is_dialog(&self, dialog: Id) -> bool {
        dialog == self.id
    }

    pub fn pending(&self) -> Option<&Leave<T>> {
        self.pending.as_ref().map(|(to, _)| to)
    }

    pub fn resolve(&mut self, answer: DialogAnswer) -> Resolved<T> {
        let Some((to, unsaved)) = self.pending.take() else {
            return Resolved::Stay;
        };
        match (answer, unsaved) {
            (DialogAnswer::Third, true) => Resolved::Save(to),
            (DialogAnswer::Second, _) => Resolved::Discard(to),
            _ => Resolved::Stay,
        }
    }
}

fn unsaved_body(changes: usize) -> String {
    match changes {
        0 => "There are unsaved changes.".to_owned(),
        1 => "1 change will be lost if you leave now.".to_owned(),
        n => format!("{n} unsaved changes will be lost if you leave now."),
    }
}

fn running_body(what: &str) -> String {
    format!("{what} is still running. Leaving stops it.")
}
