use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Outcome {
    #[default]
    Ignored,

    Consumed,

    Changed,
}

impl Outcome {
    pub fn consumed(self) -> bool {
        !matches!(self, Outcome::Ignored)
    }

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

    pub fn cmd(&self) -> bool {
        self.mods.contains(KeyModifiers::SUPER)
    }

    pub fn chord(&self) -> bool {
        self.ctrl() || self.alt() || self.cmd()
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

    Secondary,

    SecondaryDrag,

    SecondaryUp,

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

    pub mods: KeyModifiers,
}

impl Mouse {
    pub const fn new(kind: MouseKind, pos: Position) -> Self {
        Self {
            kind,
            pos,
            mods: KeyModifiers::NONE,
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputConfig {
    pub shift_wheel_horizontal: bool,

    pub ctrl_click_secondary: bool,

    pub legacy_ctrl_punctuation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientPlatform {
    MacOs,

    Other,
}

impl ClientPlatform {
    pub const OVERRIDE_ENV: &'static str = "JUNIE_TUI_CLIENT_PLATFORM";

    pub const MACOS_TERM_PROGRAMS: [&'static str; 2] = ["Apple_Terminal", "iTerm.app"];

    pub const fn build_target() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Other
        }
    }

    pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Self {
        if let Some(v) = get(Self::OVERRIDE_ENV) {
            match v.trim().to_ascii_lowercase().as_str() {
                "macos" | "mac" | "darwin" => return Self::MacOs,
                "other" | "linux" | "windows" => return Self::Other,
                _ => {}
            }
        }
        if get("LC_TERMINAL").is_some_and(|v| v == "iTerm2") {
            return Self::MacOs;
        }
        if get("TERM_PROGRAM").is_some_and(|v| Self::MACOS_TERM_PROGRAMS.contains(&v.as_str())) {
            return Self::MacOs;
        }
        Self::build_target()
    }

    pub fn current() -> Self {
        static CURRENT: std::sync::OnceLock<ClientPlatform> = std::sync::OnceLock::new();
        *CURRENT.get_or_init(|| Self::from_env(|k| std::env::var(k).ok()))
    }

    pub const fn is_macos(self) -> bool {
        matches!(self, Self::MacOs)
    }
}

impl InputConfig {
    pub const fn for_platform(platform: ClientPlatform) -> Self {
        Self {
            shift_wheel_horizontal: true,
            ctrl_click_secondary: platform.is_macos(),
            legacy_ctrl_punctuation: true,
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self::for_platform(ClientPlatform::current())
    }
}

impl Input {
    pub fn from_crossterm(ev: Event) -> Option<Self> {
        Self::from_crossterm_with(ev, InputConfig::default())
    }

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

                    MouseEventKind::Drag(MouseButton::Left)
                        if config.ctrl_click_secondary
                            && modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        MouseKind::SecondaryDrag
                    }
                    MouseEventKind::Up(MouseButton::Left)
                        if config.ctrl_click_secondary
                            && modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        MouseKind::SecondaryUp
                    }
                    MouseEventKind::Moved => MouseKind::Move,
                    MouseEventKind::Down(MouseButton::Left) => MouseKind::Down,
                    MouseEventKind::Up(MouseButton::Left) => MouseKind::Up,
                    MouseEventKind::Drag(MouseButton::Left) => MouseKind::Drag,
                    MouseEventKind::Down(MouseButton::Right) => MouseKind::Secondary,
                    MouseEventKind::Drag(MouseButton::Right) => MouseKind::SecondaryDrag,
                    MouseEventKind::Up(MouseButton::Right) => MouseKind::SecondaryUp,
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
    fn platform_defaults_make_ctrl_click_secondary_on_macos_only() {
        let mac = InputConfig::for_platform(ClientPlatform::MacOs);
        let other = InputConfig::for_platform(ClientPlatform::Other);
        assert!(mac.ctrl_click_secondary);
        assert!(!other.ctrl_click_secondary);
        for cfg in [mac, other] {
            assert!(cfg.shift_wheel_horizontal);
            assert!(cfg.legacy_ctrl_punctuation);
        }
        assert_eq!(
            InputConfig::default(),
            InputConfig::for_platform(ClientPlatform::current())
        );
    }

    #[test]
    fn client_platform_follows_the_terminal_not_the_build_target() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| (*v).to_owned())
            }
        };
        let mac = ClientPlatform::MacOs;
        let other = ClientPlatform::Other;

        assert_eq!(
            ClientPlatform::from_env(env(&[("TERM_PROGRAM", "Apple_Terminal")])),
            mac
        );
        assert_eq!(
            ClientPlatform::from_env(env(&[("TERM_PROGRAM", "iTerm.app")])),
            mac
        );

        assert_eq!(
            ClientPlatform::from_env(env(&[("LC_TERMINAL", "iTerm2")])),
            mac
        );

        let forced_other = env(&[
            ("TERM_PROGRAM", "iTerm.app"),
            (ClientPlatform::OVERRIDE_ENV, "Linux"),
        ]);
        assert_eq!(ClientPlatform::from_env(forced_other), other);
        let forced_mac = env(&[(ClientPlatform::OVERRIDE_ENV, "macos")]);
        assert_eq!(ClientPlatform::from_env(forced_mac), mac);

        let fallback = ClientPlatform::build_target();
        for pairs in [
            &[("TERM_PROGRAM", "WezTerm")][..],
            &[("TERM_PROGRAM", "ghostty")][..],
            &[(ClientPlatform::OVERRIDE_ENV, "amiga")][..],
            &[][..],
        ] {
            let pairs: &'static [(&'static str, &'static str)] = pairs;
            assert_eq!(ClientPlatform::from_env(env(pairs)), fallback, "{pairs:?}");
        }
        assert_eq!(
            ClientPlatform::build_target() == mac,
            cfg!(target_os = "macos")
        );
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
    fn secondary_drag_and_release_are_reported() {
        let none = KeyModifiers::NONE;
        let off = InputConfig {
            ctrl_click_secondary: false,
            ..InputConfig::default()
        };
        let on = InputConfig {
            ctrl_click_secondary: true,
            ..InputConfig::default()
        };
        let kind = |k, mods, cfg| mouse(Input::from_crossterm_with(raw(k, mods), cfg)).kind;
        for cfg in [on, off] {
            assert_eq!(
                kind(MouseEventKind::Drag(MouseButton::Right), none, cfg),
                MouseKind::SecondaryDrag
            );
            assert_eq!(
                kind(MouseEventKind::Up(MouseButton::Right), none, cfg),
                MouseKind::SecondaryUp
            );
        }

        let ctrl = KeyModifiers::CONTROL;
        assert_eq!(
            kind(MouseEventKind::Drag(MouseButton::Left), ctrl, on),
            MouseKind::SecondaryDrag
        );
        assert_eq!(
            kind(MouseEventKind::Up(MouseButton::Left), ctrl, on),
            MouseKind::SecondaryUp
        );
        assert_eq!(
            kind(MouseEventKind::Up(MouseButton::Left), ctrl, off),
            MouseKind::Up
        );
    }

    #[test]
    fn native_horizontal_wheel_stays_horizontal() {
        let off = InputConfig {
            shift_wheel_horizontal: false,
            ..InputConfig::default()
        };
        for mods in [KeyModifiers::NONE, KeyModifiers::SHIFT] {
            for cfg in [InputConfig::default(), off] {
                let left = mouse(Input::from_crossterm_with(
                    raw(MouseEventKind::ScrollLeft, mods),
                    cfg,
                ));
                assert_eq!(left.kind, MouseKind::WheelLeft);
                assert_eq!(left.wheel(), Some((-1, 0)));
                let right = mouse(Input::from_crossterm_with(
                    raw(MouseEventKind::ScrollRight, mods),
                    cfg,
                ));
                assert_eq!(right.kind, MouseKind::WheelRight);
                assert_eq!(right.wheel(), Some((1, 0)));
            }
        }
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
