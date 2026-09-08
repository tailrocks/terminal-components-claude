//! Debian/apt metadata for the upgrade-everything plan (P3): pending
//! upgrades, how many are security, held-back packages, reboot flag.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebianState {
    pub(crate) orphaned_packages: Vec<OrphanPackage>,
    pub(crate) pending: u32,
    pub(crate) security: u32,
    pub(crate) held: u32,
    pub(crate) reboot_required: bool,
}

impl DebianState {
    pub(crate) fn validate(&self) -> Result<(), super::accounting::InventoryError> {
        use super::accounting::{InventoryError, bytes, identities};
        identities(
            self.orphaned_packages
                .iter()
                .map(|package| package.id.as_str()),
        )?;
        bytes(
            self.orphaned_packages
                .iter()
                .map(|package| package.size_bytes),
        )?;
        let available = self
            .pending
            .checked_sub(self.held)
            .ok_or(InventoryError::InvalidCapacity)?;
        if self.security > available {
            return Err(InventoryError::InvalidCapacity);
        }
        Ok(())
    }

    pub(crate) fn summary(&self) -> String {
        let mut s = format!(
            "{} upgrades pending · {} security",
            self.pending, self.security
        );
        if self.held > 0 {
            s.push_str(&format!(" · {} held back", self.held));
        }
        if self.reboot_required {
            s.push_str(" · reboot required");
        }
        s
    }
}

/// Explicit simulated orphan package accounting; no package manager is invoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrphanPackage {
    pub(crate) id: String,
    pub(crate) size_bytes: u64,
}
