//! The shell's Esc vocabulary (ported from PR #26 `tests/esc_vocabulary.rs`,
//! commit `f4867e9b5`, scoped to the shell crate): Esc never quits, and the
//! notes the shell writes come from one standard phrase set.
//!
//! The source's test scanned every app's rendered hints; the shell crate owns
//! only its own vocabulary, so the floor here covers the shell's defaults and
//! the quit keys that answer Esc.

use std::collections::BTreeSet;

use termrock_core::event::{Key, KeyCode};
use termrock_core::keys::KeyModifiers;
use termrock_shell::quit::QuitKeys;
use termrock_shell::{KeyContext, ShellKeys};

fn esc() -> Key {
    Key {
        code: KeyCode::Esc,
        mods: KeyModifiers::NONE,
    }
}

#[test]
fn esc_never_quits_by_default() {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(
            !QuitKeys::DEFAULT.esc_quits,
            "Esc is navigation, never a quit key"
        );
    }
    assert_eq!(
        QuitKeys::DEFAULT.classify(&esc(), KeyContext::default()),
        None,
        "a clean Esc names no way out"
    );
}

#[test]
fn the_shell_notes_come_from_one_vocabulary() {
    // Every phrase the shell itself can write, drawn from PR #26's standard
    // set. A new note joins this set or reuses one of these phrases.
    let vocabulary: BTreeSet<&str> = [
        "Esc never quits · Ctrl+Q quits",
        "Unsaved changes · Ctrl+Q asks before quitting",
    ]
    .into_iter()
    .collect();
    let keys = ShellKeys::<()>::new();
    for note in [keys.esc_note.as_deref(), keys.dirty_letter_note.as_deref()]
        .into_iter()
        .flatten()
    {
        assert!(
            vocabulary.contains(note),
            "note {note:?} is outside the shell's Esc vocabulary"
        );
    }
}

#[test]
fn a_silent_esc_note_names_nothing() {
    let keys = ShellKeys::<()>::new().esc_note(None);
    assert_eq!(keys.esc_note, None, "no note, no promise");
}
