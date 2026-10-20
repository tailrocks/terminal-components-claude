//! Keyboard deliverability: which keys a terminal can actually deliver, so
//! hints never advertise a chord the app will never see.
//!
//! Ported from PR #26 `src/core/keyboard.rs` (commit `f4867e9b5`), delivery
//! observation and the label filter only. The Kitty-protocol push/pop session
//! sequences stay with the session crate on parity, which owns terminal modes.

use std::sync::atomic::{AtomicU8, Ordering};

use termrock_core::event::Input;
use termrock_core::keys::KeyModifiers;

/// 0 = undecided, 1 = delivered, 2 = refused, 3 = delivered (observed).
static SUPER: AtomicU8 = AtomicU8::new(0);
/// Same vocabulary for the Alt (Option) modifier.
static ALT: AtomicU8 = AtomicU8::new(0);

/// Does the terminal deliver the Super (Cmd) modifier?
pub fn super_delivered() -> bool {
    matches!(SUPER.load(Ordering::SeqCst), 1 | 3)
}

/// Record the terminal's answer for Super from a probe or capability reply.
pub fn set_super_delivered(on: bool) {
    SUPER.store(if on { 1 } else { 2 }, Ordering::SeqCst);
}

/// Does the terminal deliver Alt (Option) chords?
pub fn alt_delivered() -> bool {
    matches!(ALT.load(Ordering::SeqCst), 1 | 3)
}

/// Record the terminal's answer for Alt.
pub fn set_alt_delivered(on: bool) {
    ALT.store(if on { 1 } else { 2 }, Ordering::SeqCst);
}

/// Observe real input: seeing a Super chord is proof the terminal delivers it.
pub fn observe(input: &Input) {
    let Input::Key(k) = input else { return };
    if k.mods.contains(KeyModifiers::SUPER) && SUPER.load(Ordering::SeqCst) != 2 {
        SUPER.store(3, Ordering::SeqCst);
    }
}

/// May a hint with this label be advertised? A Shift+click or Shift+drag is
/// never keyboard-deliverable; a Cmd or Alt chord only when the terminal
/// delivers that modifier.
pub fn label_deliverable(key: &str) -> bool {
    let cmd = key.starts_with('⌘') || key.starts_with("Cmd+") || key.starts_with("Super+");
    let pointer = |rest: &str| rest.starts_with("click") || rest.starts_with("drag");
    let alt = ["Alt+", "Ctrl+Alt+", "Option+", "⌥"]
        .iter()
        .find_map(|p| key.strip_prefix(p))
        .is_some_and(|rest| !pointer(rest));
    let shift_pointer = key.starts_with("Shift+click")
        || key.starts_with("Shift+drag")
        || key.starts_with("Shift-click")
        || key.starts_with("⇧click");
    !shift_pointer && (!cmd || super_delivered()) && (!alt || alt_delivered())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use termrock_core::event::Key;
    use termrock_core::keys::{KeyCode, KeyModifiers};

    /// The delivery answers are process-global; the tests serialize on this.
    fn lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Input {
        Input::Key(Key { code, mods })
    }

    #[test]
    fn pointer_shift_labels_are_never_deliverable() {
        let _env = lock();
        assert!(!label_deliverable("Shift+click"));
        assert!(!label_deliverable("Shift+drag"));
        assert!(!label_deliverable("Shift-click"));
        assert!(!label_deliverable("⇧click"));
        assert!(label_deliverable("Shift+Delete"));
    }

    #[test]
    fn cmd_and_alt_labels_follow_their_delivery_answers() {
        let _env = lock();
        set_super_delivered(false);
        set_alt_delivered(false);
        assert!(!label_deliverable("⌘A"));
        assert!(!label_deliverable("Cmd+A"));
        assert!(!label_deliverable("Super+A"));
        assert!(!label_deliverable("Alt+Left"));
        assert!(!label_deliverable("Ctrl+Alt+Left"));
        assert!(!label_deliverable("Option+Left"));
        assert!(!label_deliverable("⌥Left"));
        assert!(
            label_deliverable("Ctrl+A"),
            "plain ctrl is always deliverable"
        );

        set_super_delivered(true);
        set_alt_delivered(true);
        assert!(label_deliverable("⌘A"));
        assert!(label_deliverable("Alt+Left"));
        assert!(label_deliverable("Ctrl+Alt+Left"));
        assert!(label_deliverable("Option+Left"));
    }

    #[test]
    fn an_alt_pointer_label_follows_the_mouse_not_the_keyboard() {
        let _env = lock();
        set_alt_delivered(true);
        assert!(
            label_deliverable("Alt+click"),
            "a pointer gesture is delivered by the mouse, so it may be advertised"
        );
        assert!(label_deliverable("Alt+Enter"));
        set_alt_delivered(false);
        assert!(
            label_deliverable("Alt+click"),
            "still mouse-delivered with Alt refused"
        );
        assert!(!label_deliverable("Alt+Enter"));
    }

    #[test]
    fn observing_a_super_chord_proves_delivery_until_refused() {
        let _env = lock();
        SUPER.store(0, Ordering::SeqCst);
        assert!(!super_delivered());
        observe(&key(KeyCode::Char('a'), KeyModifiers::SUPER));
        assert!(super_delivered(), "an observed chord is proof");
        set_super_delivered(false);
        observe(&key(KeyCode::Char('a'), KeyModifiers::SUPER));
        assert!(
            !super_delivered(),
            "a refused answer stands even when a chord is observed"
        );
    }
}
