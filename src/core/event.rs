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
    /// Secondary (right) button pressed: a context action.
    Secondary,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouse {
    pub kind: MouseKind,
    pub pos: Position,
}

/// How raw terminal events are normalised into [`Input`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputConfig {
    /// Shift+wheel scrolls sideways (most terminals report it as a
    /// vertical wheel with Shift held), as on macOS and every desktop.
    pub shift_wheel_horizontal: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            shift_wheel_horizontal: true,
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
            }) => Some(Input::Key(Key {
                code,
                mods: modifiers,
            })),
            Event::Mouse(MouseEvent {
                kind, column, row, ..
            }) => {
                let kind = match kind {
                    MouseEventKind::Moved => MouseKind::Move,
                    MouseEventKind::Down(MouseButton::Left) => MouseKind::Down,
                    MouseEventKind::Up(MouseButton::Left) => MouseKind::Up,
                    MouseEventKind::Drag(MouseButton::Left) => MouseKind::Drag,
                    MouseEventKind::Down(MouseButton::Right) => MouseKind::Secondary,
                    MouseEventKind::ScrollUp => MouseKind::WheelUp,
                    MouseEventKind::ScrollDown => MouseKind::WheelDown,
                    MouseEventKind::ScrollLeft => MouseKind::WheelLeft,
                    MouseEventKind::ScrollRight => MouseKind::WheelRight,
                    _ => return None,
                };
                Some(Input::Mouse(Mouse {
                    kind,
                    pos: Position::new(column, row),
                }))
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
