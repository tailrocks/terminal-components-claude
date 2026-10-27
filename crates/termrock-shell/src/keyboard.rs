//! Ported from PR #26 `src/core/keyboard.rs` (commit `f4867e9b5`), delivery

use std::sync::atomic::{AtomicU8, Ordering};

use termrock_core::event::Input;
use termrock_core::keys::KeyModifiers;

static SUPER: AtomicU8 = AtomicU8::new(0);

static ALT: AtomicU8 = AtomicU8::new(0);

pub fn super_delivered() -> bool {
    matches!(SUPER.load(Ordering::SeqCst), 1 | 3)
}

pub fn set_super_delivered(on: bool) {
    SUPER.store(if on { 1 } else { 2 }, Ordering::SeqCst);
}

pub fn alt_delivered() -> bool {
    matches!(ALT.load(Ordering::SeqCst), 1 | 3)
}

pub fn set_alt_delivered(on: bool) {
    ALT.store(if on { 1 } else { 2 }, Ordering::SeqCst);
}

pub fn observe(input: &Input) {
    let Input::Key(k) = input else { return };
    if k.mods.contains(KeyModifiers::SUPER) && SUPER.load(Ordering::SeqCst) != 2 {
        SUPER.store(3, Ordering::SeqCst);
    }
    if k.mods.contains(KeyModifiers::ALT) && ALT.load(Ordering::SeqCst) != 2 {
        ALT.store(3, Ordering::SeqCst);
    }
}

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
