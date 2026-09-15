//! Six host-only operations from the proof contract.

use std::str::FromStr;

/// Host-only CLI operations; no qualification-only extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostOperation {
    Install,
    Prepare,
    Freeze,
    Verify,
    Seal,
    Integrate,
}

impl HostOperation {
    pub const ALL: [Self; 6] = [
        Self::Install,
        Self::Prepare,
        Self::Freeze,
        Self::Verify,
        Self::Seal,
        Self::Integrate,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Prepare => "prepare",
            Self::Freeze => "freeze",
            Self::Verify => "verify",
            Self::Seal => "seal",
            Self::Integrate => "integrate",
        }
    }
}

impl FromStr for HostOperation {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "install" => Ok(Self::Install),
            "prepare" => Ok(Self::Prepare),
            "freeze" => Ok(Self::Freeze),
            "verify" => Ok(Self::Verify),
            "seal" => Ok(Self::Seal),
            "integrate" => Ok(Self::Integrate),
            _ => Err(()),
        }
    }
}
