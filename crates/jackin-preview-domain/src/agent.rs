#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Agent {
    ClaudeCode,
    Codex,
    Amp,
    KimiCode,
    OpenCode,
    GrokBuild,
}

impl Agent {
    pub const ALL: [Agent; 6] = [
        Agent::ClaudeCode,
        Agent::Codex,
        Agent::Amp,
        Agent::KimiCode,
        Agent::OpenCode,
        Agent::GrokBuild,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Agent::ClaudeCode => "Claude Code",
            Agent::Codex => "Codex",
            Agent::Amp => "Amp",
            Agent::KimiCode => "Kimi Code",
            Agent::OpenCode => "OpenCode",
            Agent::GrokBuild => "Grok Build",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Agent::ClaudeCode => "claude",
            Agent::Codex => "codex",
            Agent::Amp => "amp",
            Agent::KimiCode => "kimi",
            Agent::OpenCode => "opencode",
            Agent::GrokBuild => "grok",
        }
    }

    pub fn provider(self) -> Provider {
        match self {
            Agent::ClaudeCode => Provider::Anthropic,
            Agent::Codex => Provider::OpenAi,
            Agent::Amp => Provider::Amp,
            Agent::KimiCode => Provider::Moonshot,
            Agent::OpenCode => Provider::OpenCode,
            Agent::GrokBuild => Provider::XAi,
        }
    }

    pub fn auth_modes(self) -> &'static [AuthMode] {
        match self {
            Agent::ClaudeCode => &[
                AuthMode::Sync,
                AuthMode::ApiKey,
                AuthMode::OAuthToken,
                AuthMode::Ignore,
            ],
            _ => &[AuthMode::Sync, AuthMode::ApiKey, AuthMode::Ignore],
        }
    }

    pub fn registerable(self) -> bool {
        matches!(
            self,
            Agent::ClaudeCode | Agent::Codex | Agent::GrokBuild | Agent::OpenCode
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Provider {
    Anthropic,
    OpenAi,
    Amp,
    XAi,
    OpenCode,
    Moonshot,
    Zai,
    MiniMax,
}

impl Provider {
    pub fn label(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic / Claude",
            Provider::OpenAi => "OpenAI",
            Provider::Amp => "Amp",
            Provider::XAi => "xAI / Grok",
            Provider::OpenCode => "OpenCode",
            Provider::Moonshot => "Moonshot / Kimi",
            Provider::Zai => "Z.AI",
            Provider::MiniMax => "MiniMax",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic",
            Provider::OpenAi => "OpenAI",
            Provider::Amp => "Amp",
            Provider::XAi => "xAI",
            Provider::OpenCode => "OpenCode",
            Provider::Moonshot => "Moonshot",
            Provider::Zai => "Z.AI",
            Provider::MiniMax => "MiniMax",
        }
    }

    pub fn usage_surface(self) -> UsageSurface {
        match self {
            Provider::Anthropic => UsageSurface::Claude,
            Provider::OpenAi => UsageSurface::Codex,
            Provider::Amp => UsageSurface::Amp,
            Provider::XAi => UsageSurface::Grok,
            Provider::OpenCode => UsageSurface::OpenCode,
            Provider::Moonshot => UsageSurface::Kimi,
            Provider::Zai => UsageSurface::Zai,
            Provider::MiniMax => UsageSurface::Minimax,
        }
    }

    pub fn agent(self) -> Option<Agent> {
        match self {
            Provider::Anthropic => Some(Agent::ClaudeCode),
            Provider::OpenAi => Some(Agent::Codex),
            Provider::Amp => Some(Agent::Amp),
            Provider::XAi => Some(Agent::GrokBuild),
            Provider::OpenCode => Some(Agent::OpenCode),
            Provider::Moonshot => Some(Agent::KimiCode),
            Provider::Zai | Provider::MiniMax => None,
        }
    }

    pub fn supports_endpoint(self) -> bool {
        matches!(self, Provider::XAi)
    }

    pub fn plain_key_label(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic API key",
            Provider::OpenAi => "OpenAI API key",
            Provider::XAi => "xAI / deployment API key",
            Provider::OpenCode => "OpenCode API key",
            Provider::Amp => "Amp API key",
            Provider::Moonshot => "Kimi Code API key",
            Provider::Zai => "Z.AI API key",
            Provider::MiniMax => "MiniMax API key",
        }
    }

    pub fn folder_label(self) -> &'static str {
        match self {
            Provider::Anthropic => "Claude profile / home folder",
            Provider::OpenAi => "CODEX_HOME folder",
            Provider::XAi => "Grok profile folder",
            Provider::OpenCode => "OpenCode profile folder",
            Provider::Amp => "Amp profile folder",
            Provider::Moonshot => "Kimi profile folder",
            Provider::Zai => "Z.AI profile folder",
            Provider::MiniMax => "MiniMax profile folder",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UsageSurface {
    Claude,
    Codex,
    Amp,
    Grok,
    Zai,
    Kimi,
    Minimax,
    OpenCode,
    Unsupported,
}

impl UsageSurface {
    pub const ALL: [UsageSurface; 9] = [
        UsageSurface::Claude,
        UsageSurface::Codex,
        UsageSurface::Amp,
        UsageSurface::Grok,
        UsageSurface::Zai,
        UsageSurface::Kimi,
        UsageSurface::Minimax,
        UsageSurface::OpenCode,
        UsageSurface::Unsupported,
    ];

    pub fn label(self) -> &'static str {
        match self {
            UsageSurface::Claude => "Anthropic",
            UsageSurface::Codex => "OpenAI",
            UsageSurface::Amp => "Amp",
            UsageSurface::Grok => "xAI",
            UsageSurface::Zai => "Z.AI",
            UsageSurface::Kimi => "Kimi",
            UsageSurface::Minimax => "MiniMax",
            UsageSurface::OpenCode => "OpenCode",
            UsageSurface::Unsupported => "Usage",
        }
    }

    pub fn surface_name(self) -> &'static str {
        match self {
            UsageSurface::Claude => "Claude",
            UsageSurface::Codex => "Codex",
            UsageSurface::Amp => "Amp",
            UsageSurface::Grok => "Grok",
            UsageSurface::Zai => "Z.AI",
            UsageSurface::Kimi => "Kimi",
            UsageSurface::Minimax => "MiniMax",
            UsageSurface::OpenCode => "OpenCode",
            UsageSurface::Unsupported => "Unsupported",
        }
    }

    pub fn provider(self) -> Option<Provider> {
        match self {
            UsageSurface::Claude => Some(Provider::Anthropic),
            UsageSurface::Codex => Some(Provider::OpenAi),
            UsageSurface::Amp => Some(Provider::Amp),
            UsageSurface::Grok => Some(Provider::XAi),
            UsageSurface::Zai => Some(Provider::Zai),
            UsageSurface::Kimi => Some(Provider::Moonshot),
            UsageSurface::Minimax => Some(Provider::MiniMax),
            UsageSurface::OpenCode => Some(Provider::OpenCode),
            UsageSurface::Unsupported => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthMode {
    Sync,
    ApiKey,
    OAuthToken,
    Ignore,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_are_linked_but_distinct() {
        assert_eq!(Agent::ClaudeCode.provider(), Provider::Anthropic);
        assert_eq!(Provider::Anthropic.usage_surface(), UsageSurface::Claude);
        assert_eq!(Provider::Zai.agent(), None);
        assert_eq!(UsageSurface::Unsupported.provider(), None);
        assert!(Agent::GrokBuild.registerable());
        assert!(!Agent::Amp.registerable());
        assert_eq!(Agent::ClaudeCode.auth_modes().len(), 4);
        assert_eq!(Agent::Codex.auth_modes().len(), 3);
        assert!(Provider::XAi.supports_endpoint());
        assert!(!Provider::OpenCode.supports_endpoint());
    }
}
