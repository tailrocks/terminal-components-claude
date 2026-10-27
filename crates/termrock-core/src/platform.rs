//! Client platform detection (ported from PR #26 `src/core/event.rs`,
//! `ClientPlatform` at commit f4867e9b5).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientPlatform {
    MacOs,

    Other,
}

impl ClientPlatform {
    pub const OVERRIDE_ENV: &'static str = "TERMROCK_CLIENT_PLATFORM";

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
