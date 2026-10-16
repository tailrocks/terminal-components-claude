//! Input normalisation and event outcomes.
//!
//! Widgets receive already-normalised [`Input`] values and reply with an
//! [`Outcome`], so the routing layer can decide whether to keep propagating
//! an event and whether a redraw is needed.

use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Position;

/// Result of offering an event to a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Outcome {
    /// Not interested; keep propagating.
    #[default]
    Ignored,
    /// Consumed, nothing visible changed.
    Consumed,
    /// Consumed and the UI must be redrawn.
    Changed,
}

impl Outcome {
    pub fn consumed(self) -> bool {
        !matches!(self, Outcome::Ignored)
    }

    /// Combine with a later outcome (`Changed` dominates).
    pub fn or(self, other: Outcome) -> Outcome {
        match (self, other) {
            (Outcome::Changed, _) | (_, Outcome::Changed) => Outcome::Changed,
            (Outcome::Consumed, _) | (_, Outcome::Consumed) => Outcome::Consumed,
            _ => Outcome::Ignored,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Key(Key),
    Mouse(Mouse),
    Resize(u16, u16),
    Paste(String),
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl Key {
    pub fn ctrl(&self) -> bool {
        self.mods.contains(KeyModifiers::CONTROL)
    }
    pub fn shift(&self) -> bool {
        self.mods.contains(KeyModifiers::SHIFT)
    }
    pub fn alt(&self) -> bool {
        self.mods.contains(KeyModifiers::ALT)
    }
    pub fn plain(&self) -> bool {
        self.mods.difference(KeyModifiers::SHIFT).is_empty()
    }
    pub fn is(&self, code: KeyCode) -> bool {
        self.code == code && self.plain()
    }
    pub fn is_char(&self, c: char) -> bool {
        self.code == KeyCode::Char(c) && self.plain()
    }
    pub fn ctrl_char(&self, c: char) -> bool {
        self.code == KeyCode::Char(c) && self.ctrl()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    Move,
    Down,
    Up,
    Drag,
    /// Secondary (right) button pressed: a context action. With
    /// [`InputConfig::ctrl_click_secondary`], Ctrl + primary click too.
    Secondary,
    /// Middle button pressed (closes a tab, as in browsers and editors).
    Middle,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouse {
    pub kind: MouseKind,
    pub pos: Position,
    /// Modifier keys held with the event (Shift-click extends, Ctrl/⌘-click
    /// toggles, Alt-click picks the alternative).
    pub mods: KeyModifiers,
}

impl Mouse {
    /// An event with no modifier held.
    pub const fn new(kind: MouseKind, pos: Position) -> Self {
        Self {
            kind,
            pos,
            mods: KeyModifiers::NONE,
        }
    }

    /// This event with `mods` held.
    pub const fn with_mods(mut self, mods: KeyModifiers) -> Self {
        self.mods = mods;
        self
    }

    pub fn shift(&self) -> bool {
        self.mods.contains(KeyModifiers::SHIFT)
    }
    pub fn ctrl(&self) -> bool {
        self.mods.contains(KeyModifiers::CONTROL)
    }
    pub fn alt(&self) -> bool {
        self.mods.contains(KeyModifiers::ALT)
    }

    /// The wheel step on each axis, in notches: `(dx, dy)` with negative
    /// meaning left / up. `None` for anything but a wheel event. Widgets
    /// without a horizontal axis ignore `dx` instead of folding it into
    /// vertical scrolling.
    pub fn wheel(&self) -> Option<(i32, i32)> {
        match self.kind {
            MouseKind::WheelUp => Some((0, -1)),
            MouseKind::WheelDown => Some((0, 1)),
            MouseKind::WheelLeft => Some((-1, 0)),
            MouseKind::WheelRight => Some((1, 0)),
            _ => None,
        }
    }
}

/// How raw terminal events are normalised into [`Input`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputConfig {
    /// Shift+wheel scrolls sideways (most terminals report it as a
    /// vertical wheel with Shift held), as on macOS and every desktop.
    pub shift_wheel_horizontal: bool,
    /// Ctrl + primary click is a secondary (context) click, as on macOS.
    /// On by default on macOS builds only, where Ctrl-click is the
    /// one-button context click; elsewhere Ctrl-click stays a primary
    /// click with Ctrl held (toggle selection).
    pub ctrl_click_secondary: bool,
    /// Terminals using the legacy key encoding report Ctrl+\\, Ctrl+],
    /// Ctrl+^ and Ctrl+_ as Ctrl+4..7 (the same control bytes). Map them
    /// back so a binding names the key the user pressed. On by default.
    pub legacy_ctrl_punctuation: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            shift_wheel_horizontal: true,
            ctrl_click_secondary: cfg!(target_os = "macos"),
            legacy_ctrl_punctuation: true,
        }
    }
}

impl Input {
    pub fn from_crossterm(ev: Event) -> Option<Self> {
        Self::from_crossterm_with(ev, InputConfig::default())
    }

    /// [`from_crossterm`](Self::from_crossterm) under `config`.
    pub fn from_crossterm_with(ev: Event, config: InputConfig) -> Option<Self> {
        let ev = match ev {
            Event::Mouse(mut m)
                if config.shift_wheel_horizontal && m.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                m.kind = match m.kind {
                    MouseEventKind::ScrollUp => MouseEventKind::ScrollLeft,
                    MouseEventKind::ScrollDown => MouseEventKind::ScrollRight,
                    k => k,
                };
                Event::Mouse(m)
            }
            ev => ev,
        };
        match ev {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            }) => {
                let code = match code {
                    KeyCode::Char(c @ '4'..='7')
                        if config.legacy_ctrl_punctuation
                            && modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        KeyCode::Char(match c {
                            '4' => '\\',
                            '5' => ']',
                            '6' => '^',
                            _ => '_',
                        })
                    }
                    code => code,
                };
                Some(Input::Key(Key {
                    code,
                    mods: modifiers,
                }))
            }
            Event::Mouse(MouseEvent {
                kind,
                column,
                row,
                modifiers,
            }) => {
                let kind = match kind {
                    MouseEventKind::Down(MouseButton::Left)
                        if config.ctrl_click_secondary
                            && modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        MouseKind::Secondary
                    }
                    MouseEventKind::Moved => MouseKind::Move,
                    MouseEventKind::Down(MouseButton::Left) => MouseKind::Down,
                    MouseEventKind::Up(MouseButton::Left) => MouseKind::Up,
                    MouseEventKind::Drag(MouseButton::Left) => MouseKind::Drag,
                    MouseEventKind::Down(MouseButton::Right) => MouseKind::Secondary,
                    MouseEventKind::Down(MouseButton::Middle) => MouseKind::Middle,
                    MouseEventKind::ScrollUp => MouseKind::WheelUp,
                    MouseEventKind::ScrollDown => MouseKind::WheelDown,
                    MouseEventKind::ScrollLeft => MouseKind::WheelLeft,
                    MouseEventKind::ScrollRight => MouseKind::WheelRight,
                    _ => return None,
                };
                Some(Input::Mouse(
                    Mouse::new(kind, Position::new(column, row)).with_mods(modifiers),
                ))
            }
            Event::Resize(w, h) => Some(Input::Resize(w, h)),
            Event::Paste(s) => Some(Input::Paste(s)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(kind: MouseEventKind, modifiers: KeyModifiers) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 1,
            row: 2,
            modifiers,
        })
    }

    fn mouse(i: Option<Input>) -> Mouse {
        match i {
            Some(Input::Mouse(m)) => m,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn mouse_events_carry_modifiers_middle_button_and_wheel_axes() {
        let m = mouse(Input::from_crossterm(raw(
            MouseEventKind::Down(MouseButton::Left),
            KeyModifiers::SHIFT,
        )));
        assert_eq!(m.kind, MouseKind::Down);
        assert!(m.shift());
        let m = mouse(Input::from_crossterm(raw(
            MouseEventKind::Down(MouseButton::Middle),
            KeyModifiers::NONE,
        )));
        assert_eq!(m.kind, MouseKind::Middle);
        let m = mouse(Input::from_crossterm(raw(
            MouseEventKind::ScrollRight,
            KeyModifiers::NONE,
        )));
        assert_eq!(m.wheel(), Some((1, 0)));
    }

    #[test]
    fn legacy_ctrl_digits_decode_to_the_punctuation_pressed() {
        let key = |c, cfg| match Input::from_crossterm_with(
            Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)),
            cfg,
        ) {
            Some(Input::Key(k)) => k.code,
            other => panic!("{other:?}"),
        };
        let on = InputConfig::default();
        assert_eq!(key('4', on), KeyCode::Char('\\'));
        assert_eq!(key('5', on), KeyCode::Char(']'));
        assert_eq!(key('6', on), KeyCode::Char('^'));
        assert_eq!(key('7', on), KeyCode::Char('_'));
        assert_eq!(key('3', on), KeyCode::Char('3'));
        let off = InputConfig {
            legacy_ctrl_punctuation: false,
            ..on
        };
        assert_eq!(key('4', off), KeyCode::Char('4'));
    }

    #[test]
    fn ctrl_click_is_secondary_only_when_configured() {
        let ctrl_click = raw(
            MouseEventKind::Down(MouseButton::Left),
            KeyModifiers::CONTROL,
        );
        let on = InputConfig {
            ctrl_click_secondary: true,
            ..InputConfig::default()
        };
        let off = InputConfig {
            ctrl_click_secondary: false,
            ..InputConfig::default()
        };
        assert_eq!(
            mouse(Input::from_crossterm_with(ctrl_click.clone(), on)).kind,
            MouseKind::Secondary
        );
        let m = mouse(Input::from_crossterm_with(ctrl_click, off));
        assert_eq!(m.kind, MouseKind::Down);
        assert!(m.ctrl());
    }

    #[test]
    fn shift_wheel_scrolls_sideways_unless_configured_off() {
        let wheel = |kind, modifiers| {
            Event::Mouse(MouseEvent {
                kind,
                column: 3,
                row: 4,
                modifiers,
            })
        };
        let kind = |i: Option<Input>| match i {
            Some(Input::Mouse(m)) => m.kind,
            other => panic!("{other:?}"),
        };
        let shift = KeyModifiers::SHIFT;
        assert_eq!(
            kind(Input::from_crossterm(wheel(
                MouseEventKind::ScrollDown,
                shift
            ))),
            MouseKind::WheelRight
        );
        assert_eq!(
            kind(Input::from_crossterm(wheel(
                MouseEventKind::ScrollUp,
                shift
            ))),
            MouseKind::WheelLeft
        );
        assert_eq!(
            kind(Input::from_crossterm(wheel(
                MouseEventKind::ScrollDown,
                KeyModifiers::NONE
            ))),
            MouseKind::WheelDown
        );
        let off = InputConfig {
            shift_wheel_horizontal: false,
            ..InputConfig::default()
        };
        assert_eq!(
            kind(Input::from_crossterm_with(
                wheel(MouseEventKind::ScrollDown, shift),
                off
            )),
            MouseKind::WheelDown
        );
    }

    #[test]
    fn outcome_combines_with_changed_dominating() {
        assert_eq!(Outcome::Ignored.or(Outcome::Consumed), Outcome::Consumed);
        assert_eq!(Outcome::Consumed.or(Outcome::Changed), Outcome::Changed);
        assert_eq!(Outcome::Ignored.or(Outcome::Ignored), Outcome::Ignored);
        assert!(!Outcome::Ignored.consumed());
    }

    #[test]
    fn key_helpers() {
        let k = Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::CONTROL,
        };
        assert!(k.ctrl_char('a'));
        assert!(!k.is_char('a'));
        let s = Key {
            code: KeyCode::Char('A'),
            mods: KeyModifiers::SHIFT,
        };
        assert!(s.plain());
        assert!(s.is_char('A'));
    }
}
