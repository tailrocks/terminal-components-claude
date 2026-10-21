//! The quit guard: what the app stands to lose when it leaves, and the
//! confirmation that protects it.
//!
//! Ported from PR #26 `src/ui/leave.rs` (commit `f4867e9b5`). The parity
//! adaptation describes the confirmation as a [`LeaveDialog`] (kind, title,
//! body and the resolved meaning of each action) instead of building the
//! source's `Dialog` widget directly, because the parity dialog is owned by
//! the overlays crate and resolves through `ActionKey`s; the app maps the
//! guard's [`LeaveDialogKind`] onto whatever dialog component it renders.

use termrock_core::event::Key;
use termrock_core::id::Id;
use termrock_core::keys::KeyCode;

/// What the app stands to lose when it leaves the current context.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Stakes {
    /// Nothing to lose; the leave goes now.
    #[default]
    Clean,
    /// Unsaved changes: the count of lost changes.
    Unsaved {
        /// How many changes would be lost.
        changes: usize,
    },
    /// Work in flight that stopping abandons.
    Running {
        /// What is running, in the app's words.
        what: String,
    },
}

impl Stakes {
    /// Is there nothing at stake?
    pub fn is_clean(&self) -> bool {
        *self == Self::Clean
    }

    /// The stronger of two stakes. Running work outranks unsaved changes;
    /// unsaved changes add up.
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

/// What a key classifies as, when it names a way to leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveKey {
    /// The key copies instead of quitting (a selection is up).
    Copy,
    /// The key asks to leave.
    Quit,
}

/// The situation a quit key lands in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyContext {
    /// A text field owns typing: bare letters belong to it.
    pub text_focus: bool,
    /// There are unsaved changes: a quit letter must not fire.
    pub dirty: bool,
    /// A selection is up: Ctrl+C copies instead of quitting.
    pub selection: bool,
}

/// Which keys ask to leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitKeys {
    /// Does Ctrl+C quit (or copy with a selection up)?
    pub ctrl_c: bool,
    /// Does Ctrl+Q quit?
    pub ctrl_q: bool,
    /// Does a bare letter quit? Named so Vim-style apps can drop it.
    pub letter: Option<char>,
    /// Does Esc quit when nothing else answers it?
    pub esc_quits: bool,
}

impl Default for QuitKeys {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl QuitKeys {
    /// Ctrl+C, Ctrl+Q and the `q` letter. Esc never quits.
    pub const DEFAULT: Self = Self {
        ctrl_c: true,
        ctrl_q: true,
        letter: Some('q'),
        esc_quits: false,
    };

    /// Set the quit letter (`None` removes it).
    #[must_use]
    pub const fn letter(mut self, letter: Option<char>) -> Self {
        self.letter = letter;
        self
    }

    /// Let Esc quit when nothing else answers it.
    #[must_use]
    pub const fn esc_quits(mut self, on: bool) -> Self {
        self.esc_quits = on;
        self
    }

    /// Classify a key against its context.
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

/// Does the key name the Ctrl chord of `c`? (PR #26 `Key::ctrl_char`.)
pub(crate) fn ctrl_char(key: &Key, c: char) -> bool {
    key.code == KeyCode::Char(c) && key.mods == termrock_core::keys::KeyModifiers::CONTROL
}

/// Does the key name the bare (or Shifted) letter `c`? (PR #26 `Key::is_char`.)
pub(crate) fn is_char(key: &Key, c: char) -> bool {
    key.code == KeyCode::Char(c)
        && key
            .mods
            .difference(termrock_core::keys::KeyModifiers::SHIFT)
            .is_empty()
}

/// May a bare quit letter fire in this context?
pub fn letters_allowed(cx: KeyContext) -> bool {
    !cx.text_focus && !cx.dirty
}

/// A leave request: quit the app, or go somewhere in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leave<T> {
    /// Leave the app.
    Quit,
    /// Navigate to `T`.
    To(T),
}

/// What a resolved confirmation means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved<T> {
    /// Stay where we are.
    Stay,
    /// Save first, then carry out the leave.
    Save(Leave<T>),
    /// Abandon what is unsaved and carry out the leave.
    Discard(Leave<T>),
}

/// Which confirmation the guard wants shown, and what each action means.
///
/// The app renders this with its own dialog component; the meanings of the
/// actions follow [`LeaveDialogKind`] exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaveDialog {
    /// The widget id the dialog renders under.
    pub id: Id,
    /// Which actions the dialog offers and which starts focused.
    pub kind: LeaveDialogKind,
    /// The dialog title.
    pub title: &'static str,
    /// The dialog body.
    pub body: String,
}

/// The shape of a leave confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveDialogKind {
    /// Cancel, Discard, Save — Save is the default and resolves to
    /// [`Resolved::Save`]; Discard resolves to [`Resolved::Discard`].
    SaveDiscardCancel,
    /// Cancel, Discard — Discard resolves to [`Resolved::Discard`].
    DiscardOnly,
    /// Stay, Stop and leave — Stop resolves to [`Resolved::Discard`].
    StopStay,
}

/// The answer the app reports when its dialog finishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    /// The first action (Cancel / Stay).
    First,
    /// The second action (Discard / Stop and leave).
    Second,
    /// The third action (Save), where the kind offers it.
    Third,
}

/// Guards one leave path: it remembers the pending leave while its
/// confirmation shows and resolves the dialog's answer into a [`Resolved`].
#[derive(Debug, Clone)]
pub struct LeaveGuard<T> {
    /// The widget id the guard's dialogs render under.
    pub id: Id,
    pending: Option<(Leave<T>, bool)>,
    /// May the guard offer to save? `false` for read-only contexts, whose
    /// only choices are to stay or to discard.
    pub save: bool,
}

impl<T: Clone> LeaveGuard<T> {
    /// A guard whose dialogs render under `id`, save offered.
    pub fn new(id: Id) -> Self {
        Self {
            id,
            pending: None,
            save: true,
        }
    }

    /// Set whether the guard may offer to save.
    #[must_use]
    pub fn save(mut self, on: bool) -> Self {
        self.save = on;
        self
    }

    /// Ask to leave. `Ok(leave)` goes now; `Err(dialog)` must be shown, and
    /// its answer reported to [`LeaveGuard::resolve`].
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

    /// Ask to quit the app.
    pub fn request_quit(&mut self, stakes: &Stakes) -> Result<Leave<T>, LeaveDialog> {
        self.request(Leave::Quit, stakes)
    }

    /// Is this dialog id the guard's?
    pub fn is_dialog(&self, dialog: Id) -> bool {
        dialog == self.id
    }

    /// The leave the guard is waiting on, if any.
    pub fn pending(&self) -> Option<&Leave<T>> {
        self.pending.as_ref().map(|(to, _)| to)
    }

    /// Resolve the dialog's answer into what it means.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: termrock_core::keys::KeyModifiers::NONE,
        }
    }

    fn ctrl(c: char) -> Key {
        Key {
            code: KeyCode::Char(c),
            mods: termrock_core::keys::KeyModifiers::CONTROL,
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
        let mut g: LeaveGuard<&str> = LeaveGuard::new(Id::root("leave"));
        assert_eq!(
            g.request(Leave::To("home"), &Stakes::Clean).ok(),
            Some(Leave::To("home"))
        );
        let d = g
            .request(Leave::To("home"), &Stakes::Unsaved { changes: 2 })
            .unwrap_err();
        assert!(g.is_dialog(d.id));
        assert_eq!(d.kind, LeaveDialogKind::SaveDiscardCancel);
        assert!(
            d.body.contains("2 unsaved changes"),
            "the body names the cost: {d:?}"
        );
        assert_eq!(g.pending(), Some(&Leave::To("home")));
        assert_eq!(
            g.resolve(DialogAnswer::Third),
            Resolved::Save(Leave::To("home"))
        );
        assert_eq!(g.pending(), None);
        g.request_quit(&Stakes::Unsaved { changes: 1 }).unwrap_err();
        assert_eq!(
            g.resolve(DialogAnswer::Second),
            Resolved::Discard(Leave::Quit)
        );
        g.request_quit(&Stakes::Unsaved { changes: 1 }).unwrap_err();
        assert_eq!(g.resolve(DialogAnswer::First), Resolved::Stay);
    }

    #[test]
    fn a_guard_that_cannot_save_offers_cancel_and_discard() {
        let mut g: LeaveGuard<()> = LeaveGuard::new(Id::root("leave")).save(false);
        let d = g.request_quit(&Stakes::Unsaved { changes: 1 }).unwrap_err();
        assert_eq!(d.kind, LeaveDialogKind::DiscardOnly);
        assert_eq!(
            g.resolve(DialogAnswer::Second),
            Resolved::Discard(Leave::Quit)
        );
    }

    #[test]
    fn running_work_asks_stop_or_stay() {
        let mut g: LeaveGuard<()> = LeaveGuard::new(Id::root("leave"));
        let d = g
            .request_quit(&Stakes::Running {
                what: "2 activities".into(),
            })
            .unwrap_err();
        assert_eq!(d.kind, LeaveDialogKind::StopStay);
        assert!(d.body.contains("2 activities"), "{d:?}");
        assert_eq!(
            g.resolve(DialogAnswer::Second),
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
