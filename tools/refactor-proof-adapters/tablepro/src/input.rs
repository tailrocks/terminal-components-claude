//! Direct-lane input through production `TableProApp` handlers.

use junie_tui::{Axis, KeyCode, KeyModifiers, MouseKind};
use junie_tui_testing::Harness;
use tablepro_app::TableProApp;

/// Repeat `key` `times` times.
pub fn repeat_key(harness: &mut Harness<TableProApp>, code: KeyCode, times: usize) {
    for _ in 0..times {
        let _ = harness.key(code);
    }
}

/// `↓` `times` times.
pub fn down(harness: &mut Harness<TableProApp>, times: usize) {
    repeat_key(harness, KeyCode::Down, times);
}

/// `↑` `times` times.
pub fn up(harness: &mut Harness<TableProApp>, times: usize) {
    repeat_key(harness, KeyCode::Up, times);
}

/// `→` `times` times.
pub fn right(harness: &mut Harness<TableProApp>, times: usize) {
    repeat_key(harness, KeyCode::Right, times);
}

/// `←` `times` times.
pub fn left(harness: &mut Harness<TableProApp>, times: usize) {
    repeat_key(harness, KeyCode::Left, times);
}

/// `Tab`.
pub fn tab(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::Tab);
}

/// `Shift+Tab`.
pub fn backtab(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::BackTab);
}

/// `Home`.
pub fn home(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::Home);
}

/// `End`.
pub fn end(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::End);
}

/// `Enter`.
pub fn enter(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::Enter);
}

/// `Escape`.
pub fn esc(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::Esc);
}

/// `Space`.
pub fn space(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::Char(' '));
}

/// Type `text`. Newlines become `Enter` so multiline SQL stays source-shaped.
pub fn type_text(harness: &mut Harness<TableProApp>, text: &str) {
    for ch in text.chars() {
        if ch == '\n' {
            enter(harness);
        } else {
            let _ = harness.key(KeyCode::Char(ch));
        }
    }
}

/// Ctrl+`c`.
pub fn ctrl(harness: &mut Harness<TableProApp>, c: char) {
    let _ = harness.ctrl(c);
}

/// Alt+`c`.
pub fn alt(harness: &mut Harness<TableProApp>, c: char) {
    let _ = harness.alt(c);
}

/// Function key `Fn`.
pub fn function(harness: &mut Harness<TableProApp>, n: u8) {
    let _ = harness.key(KeyCode::F(n));
}

/// `PageDown` `times` times.
pub fn page_down(harness: &mut Harness<TableProApp>, times: usize) {
    repeat_key(harness, KeyCode::PageDown, times);
}

/// `PageUp` `times` times.
pub fn page_up(harness: &mut Harness<TableProApp>, times: usize) {
    repeat_key(harness, KeyCode::PageUp, times);
}

/// `Delete`.
pub fn delete(harness: &mut Harness<TableProApp>) {
    let _ = harness.key(KeyCode::Delete);
}

/// Move the pointer over the first cell of `needle`.
pub fn hover_text(harness: &mut Harness<TableProApp>, needle: &str) {
    if let Some((x, y)) = harness.find(needle) {
        let _ = harness.mouse(MouseKind::Move, x, y);
    }
}

/// Primary-button down on the first cell of `needle`.
pub fn pointer_down_text(harness: &mut Harness<TableProApp>, needle: &str) {
    if let Some((x, y)) = harness.find(needle) {
        let _ = harness.mouse(MouseKind::Down, x, y);
    }
}

/// Primary-button up on the first cell of `needle`.
pub fn pointer_up_text(harness: &mut Harness<TableProApp>, needle: &str) {
    if let Some((x, y)) = harness.find(needle) {
        let _ = harness.mouse(MouseKind::Up, x, y);
    }
}

/// Click the first cell of `needle`.
pub fn click_text(harness: &mut Harness<TableProApp>, needle: &str) {
    if let Some((x, y)) = harness.find(needle) {
        let _ = harness.click(x, y);
    }
}

/// Vertical wheel at the first cell of `needle`.
pub fn wheel_down_text(harness: &mut Harness<TableProApp>, needle: &str, times: usize) {
    let Some((x, y)) = harness.find(needle) else {
        return;
    };
    for _ in 0..times {
        let _ = harness.wheel(Axis::V, 3, x, y);
    }
}

/// Ctrl+L then bracketed paste, matching the source `replace` paste half.
pub fn replace_paste(harness: &mut Harness<TableProApp>, value: &str) {
    ctrl(harness, 'l');
    let _ = harness.paste(value);
}

/// Explicit `Tick` events.
pub fn ticks(harness: &mut Harness<TableProApp>, n: usize) {
    harness.ticks(n);
}

/// Shift+`code`.
pub fn shift(harness: &mut Harness<TableProApp>, code: KeyCode) {
    let _ = harness.key_mod(code, KeyModifiers::SHIFT);
}
