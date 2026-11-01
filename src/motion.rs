#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Motion {
    #[default]
    Full,

    Reduced,

    Paused,
}

impl Motion {
    pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion {
        match cli {
            Some(m) => m,
            None if no_motion_env => Motion::Reduced,
            None => Motion::Full,
        }
    }

    pub fn env_requests_reduced(var: &str) -> bool {
        std::env::var_os(var).is_some_and(|v| !v.is_empty() && v != "0")
    }

    pub fn from_cli_or_env(cli: Option<Motion>, var: &str) -> Motion {
        Motion::resolve(cli, Motion::env_requests_reduced(var))
    }

    pub fn ticks(self) -> bool {
        self != Motion::Paused
    }

    pub fn animates(self) -> bool {
        self == Motion::Full
    }

    pub fn is_paused(self) -> bool {
        self == Motion::Paused
    }
}
