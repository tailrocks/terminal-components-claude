//! Client platform detection (ported from PR #26 `src/core/event.rs`,
//! `ClientPlatform` at commit f4867e9b5).
//!
//! Terminal clients differ in which modifier the OS delivers for
//! option-style clicks and shortcuts. macOS conventions (Option as Alt)
//! differ from other platforms, so selection gestures need to know which
//! client they run under. Detection is overridable through the environment
//! so tests and users can pin a platform.

/// The platform family of the terminal client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientPlatform {
    /// macOS: Option is Alt, Cmd is Super.
    MacOs,
    /// Everything else: Ctrl is the primary modifier.
    Other,
}

impl ClientPlatform {
    /// Environment variable that overrides detection
    /// (`macos` / `mac` / `darwin` or `other` / `linux` / `windows`).
    pub const OVERRIDE_ENV: &'static str = "TERMROCK_CLIENT_PLATFORM";

    /// Terminal programs that report a macOS host.
    pub const MACOS_TERM_PROGRAMS: [&'static str; 2] = ["Apple_Terminal", "iTerm.app"];

    /// The platform this binary was built for.
    pub const fn build_target() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Other
        }
    }

    /// Detect from the provided environment lookup.
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

    /// Detect once per process from the real environment.
    pub fn current() -> Self {
        static CURRENT: std::sync::OnceLock<ClientPlatform> = std::sync::OnceLock::new();
        *CURRENT.get_or_init(|| Self::from_env(|k| std::env::var(k).ok()))
    }

    pub const fn is_macos(self) -> bool {
        matches!(self, Self::MacOs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            vars.iter()
                .find(|(name, _)| *name == k)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn env_override_wins() {
        assert_eq!(
            ClientPlatform::from_env(env(&[(ClientPlatform::OVERRIDE_ENV, "macos")])),
            ClientPlatform::MacOs
        );
        assert_eq!(
            ClientPlatform::from_env(env(&[(ClientPlatform::OVERRIDE_ENV, "linux")])),
            ClientPlatform::Other
        );
    }

    #[test]
    fn term_program_detects_macos_hosts() {
        assert_eq!(
            ClientPlatform::from_env(env(&[("TERM_PROGRAM", "iTerm.app")])),
            ClientPlatform::MacOs
        );
        assert_eq!(
            ClientPlatform::from_env(env(&[("LC_TERMINAL", "iTerm2")])),
            ClientPlatform::MacOs
        );
        assert_eq!(
            ClientPlatform::from_env(env(&[("TERM_PROGRAM", "tmux")])),
            ClientPlatform::build_target()
        );
    }
}
