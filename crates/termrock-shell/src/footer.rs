//! The footer's ownership rules: which hints it names, in what order, and
//! what the status line does to them.
//!
//! Ported from PR #26 `src/shell/footer.rs` (commit `f4867e9b5`). The parity
//! adaptation keeps the compose rules and drops the rendering: the parity
//! hint bar is `termrock_feedback::HintBar`, which renders a runtime
//! [`termrock_runtime::keymap::HintLayer`]; an app maps the composed
//! [`Hint`]s onto that layer at adoption time. The source's `Kind` status
//! variant and the peek (`ui::reveal`) integration are not ported: parity
//! status tone is a theme [`Role`] and the reveal machinery belongs to the
//! runtime's feedback wave.

use std::borrow::Cow;

use termrock_theme::Role;

use crate::keyboard;

/// A hint the footer names, in the app's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    /// The key as it reads on the label (`"Ctrl+S"`, `"Tab"`, `"Shift+click"`).
    pub key: &'static str,
    /// What the key does.
    pub action: Cow<'static, str>,
    /// Ordering weight; [`HINT_PINNED`] survives an error, [`HINT_NORMAL`]
    /// does not.
    pub priority: u8,
}

/// A pinned hint keeps its place even when an error is up.
pub const HINT_PINNED: u8 = 2;

/// The ordinary weight; an error demotes every hint to it.
pub const HINT_NORMAL: u8 = 1;

/// The weight a hint carries when nothing said otherwise.
pub const HINT_AUTO: u8 = 0;

impl Hint {
    /// A hint at [`HINT_AUTO`] weight.
    pub fn new(key: &'static str, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key,
            action: action.into(),
            priority: HINT_AUTO,
        }
    }
}

/// The footer status message and its tone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterStatus {
    /// The message.
    pub text: String,
    /// The tone it reads in.
    pub tone: Role,
    /// Highlighted fragments of the message and their tones.
    pub parts: Vec<(String, Role)>,
}

impl FooterStatus {
    /// Is this an error (the kind that demotes every hint)?
    pub fn is_error(&self) -> bool {
        self.tone == Role::Danger
    }
}

/// The footer: the hints it names and the status message it carries.
///
/// Compose rules, ported exactly: the topmost layer that offers hints wins
/// over the fallback set; Tab-next is added once and never while a text
/// field is editing; hints never name a key the terminal cannot deliver;
/// a dirty pin holds the commit key's place; an error demotes every hint
/// and takes the status line.
#[derive(Debug, Clone, Default)]
pub struct Footer {
    /// The topmost layer's hints, when a layer offers any.
    pub hints: Option<Vec<Hint>>,
    /// The hints named when no layer offers any.
    pub fallback: Vec<Hint>,
    /// A text field owns typing (the EDIT badge; Tab-next hides).
    pub editing: bool,
    /// Advertise Tab as "Next".
    pub tab_next: bool,
    /// The status message.
    pub status: Option<FooterStatus>,
    /// The key whose hint stays pinned while dirty (`"Ctrl+S"`).
    pub pin_dirty: Option<&'static str>,
    /// There are unsaved changes.
    pub dirty: bool,
}

impl Footer {
    /// An empty footer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fold one more layer in underneath what is already set: the first
    /// layer that offers hints wins, and the fallback shows only when no
    /// layer did.
    #[must_use]
    pub fn layer(mut self, hints: Option<Vec<Hint>>) -> Self {
        if self.hints.is_none() {
            self.hints = hints.filter(|h| !h.is_empty());
        }
        self
    }

    /// Set the fallback hints.
    #[must_use]
    pub fn fallback(mut self, hints: Vec<Hint>) -> Self {
        self.fallback = hints;
        self
    }

    /// Set whether a text field owns typing.
    #[must_use]
    pub fn editing(mut self, on: bool) -> Self {
        self.editing = on;
        self
    }

    /// Advertise Tab as "Next".
    #[must_use]
    pub fn tab_next(mut self, on: bool) -> Self {
        self.tab_next = on;
        self
    }

    /// Pin a key's hint while dirty.
    #[must_use]
    pub fn pin_dirty(mut self, key: Option<&'static str>) -> Self {
        self.pin_dirty = key;
        self
    }

    /// Set whether there are unsaved changes.
    #[must_use]
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// Set the status message and its tone.
    #[must_use]
    pub fn status(mut self, status: Option<(&str, Role)>) -> Self {
        self.status = status.map(|(text, tone)| FooterStatus {
            text: text.to_owned(),
            tone,
            parts: Vec::new(),
        });
        self
    }

    /// Set the status message with highlighted fragments.
    #[must_use]
    pub fn status_parts(
        mut self,
        status: Option<(&str, Role)>,
        parts: Vec<(String, Role)>,
    ) -> Self {
        self.status = status.map(|(text, tone)| FooterStatus {
            text: text.to_owned(),
            tone,
            parts,
        });
        self
    }

    /// Resolve the hints this footer names, in the order it names them.
    pub fn compose(&self) -> Vec<Hint> {
        let mut hints = self.hints.clone().unwrap_or_else(|| self.fallback.clone());
        if self.tab_next && !self.editing && !hints.iter().any(|h| h.key == "Tab") {
            hints.push(Hint::new("Tab", "Next"));
        }
        hints.retain(|h| keyboard::label_deliverable(h.key));
        if self.dirty
            && let Some(key) = self.pin_dirty
        {
            for h in hints.iter_mut().filter(|h| h.key == key) {
                h.priority = HINT_PINNED;
            }
        }
        if self.status.as_ref().is_some_and(FooterStatus::is_error) {
            for h in &mut hints {
                h.priority = HINT_NORMAL;
            }
        }
        hints
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard;
    use crate::status::{TONE_SECONDARY, TONE_SUCCESS};

    fn keyboard_lock() -> std::sync::MutexGuard<'static, ()> {
        keyboard::test_lock()
    }

    fn keys(h: &[Hint]) -> Vec<String> {
        h.iter()
            .map(|h| format!("{} {}", h.key, h.action))
            .collect()
    }

    #[test]
    fn the_topmost_context_with_hints_wins_then_the_fallback() {
        let f = Footer::new()
            .layer(None)
            .layer(Some(vec![]))
            .layer(Some(vec![Hint::new("Enter", "Open")]))
            .layer(Some(vec![Hint::new("d", "Delete")]))
            .fallback(vec![Hint::new("F1", "Help")]);
        assert_eq!(keys(&f.compose()), ["Enter Open"]);
        let f = Footer::new()
            .layer(None)
            .fallback(vec![Hint::new("F1", "Help")]);
        assert_eq!(keys(&f.compose()), ["F1 Help"]);
    }

    #[test]
    fn tab_next_is_added_once_and_never_while_editing() {
        let f = Footer::new()
            .layer(Some(vec![Hint::new("d", "Delete")]))
            .tab_next(true);
        assert_eq!(keys(&f.compose()), ["d Delete", "Tab Next"]);
        let named = Footer::new()
            .layer(Some(vec![Hint::new("Tab", "Into page")]))
            .tab_next(true);
        assert_eq!(keys(&named.compose()), ["Tab Into page"]);
        let editing = Footer::new()
            .layer(Some(vec![Hint::new("Enter", "Apply")]))
            .tab_next(true)
            .editing(true);
        assert_eq!(keys(&editing.compose()), ["Enter Apply"]);
    }

    #[test]
    fn hints_never_name_shift_click_or_an_undelivered_cmd_chord() {
        let _guard = keyboard_lock();
        keyboard::set_super_delivered(false);
        let f = Footer::new().layer(Some(vec![
            Hint::new("Shift+click", "Extend"),
            Hint::new("Cmd+A", "Select all"),
            Hint::new("Ctrl+A", "Select all"),
        ]));
        assert_eq!(keys(&f.compose()), ["Ctrl+A Select all"]);
    }

    #[test]
    fn an_error_demotes_every_hint_and_takes_the_status_line() {
        let hints = || {
            vec![
                Hint::new("Enter", "Open"),
                Hint::new("d", "Delete"),
                Hint::new("Esc", "Back"),
            ]
        };
        let msg = "Connection refused by the server at db.example";
        // Info keeps every hint at its own weight; the way out stays named.
        let info = Footer::new()
            .layer(Some(hints()))
            .status(Some((msg, TONE_SECONDARY)));
        assert!(!info.status.as_ref().is_some_and(FooterStatus::is_error));
        assert_eq!(
            keys(&info.compose()),
            ["Enter Open", "d Delete", "Esc Back"]
        );
        // An error demotes every hint to normal weight and owns the line.
        let error = Footer::new()
            .layer(Some(hints()))
            .status(Some((msg, Role::Danger)));
        assert!(error.status.as_ref().is_some_and(FooterStatus::is_error));
        let composed = error.compose();
        assert!(
            composed.iter().all(|h| h.priority == HINT_NORMAL),
            "an error demotes every hint: {composed:?}"
        );
    }

    #[test]
    fn the_commit_key_stays_pinned_while_dirty_only_when_asked() {
        let hints = || {
            vec![
                Hint::new("Enter", "Edit"),
                Hint::new("d", "Delete"),
                Hint::new("Ctrl+S", "Save"),
                Hint::new("Esc", "Back"),
            ]
        };
        let plain = Footer::new().layer(Some(hints())).dirty(true);
        let composed = plain.compose();
        assert!(
            composed
                .iter()
                .find(|h| h.key == "Ctrl+S")
                .is_some_and(|h| h.priority == HINT_AUTO),
            "no pin without a request: {composed:?}"
        );
        let pinned = Footer::new()
            .layer(Some(hints()))
            .dirty(true)
            .pin_dirty(Some("Ctrl+S"));
        let composed = pinned.compose();
        assert!(
            composed
                .iter()
                .find(|h| h.key == "Ctrl+S")
                .is_some_and(|h| h.priority == HINT_PINNED),
            "the pin holds the commit key: {composed:?}"
        );
    }

    #[test]
    fn a_status_message_reads_back_with_its_tone() {
        let f = Footer::new().status(Some(("Saved", Role::Success)));
        let s = f.status.expect("the status is set");
        assert_eq!((s.text.as_str(), s.tone), ("Saved", TONE_SUCCESS));
        assert!(!s.is_error());
        let parts = vec![("db.example".to_owned(), Role::Danger)];
        let f = Footer::new().status_parts(Some(("refused", Role::Danger)), parts);
        let s = f.status.expect("the status is set");
        assert!(s.is_error());
        assert_eq!(s.parts.len(), 1);
    }
}
