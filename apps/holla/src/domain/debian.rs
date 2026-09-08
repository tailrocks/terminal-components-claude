//! Debian/apt metadata for the upgrade-everything plan (P3): pending
//! upgrades, how many are security, held-back packages, reboot flag.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebianState {
    pub orphaned_packages: Vec<OrphanPackage>,
    pub pending: u32,
    pub security: u32,
    pub held: u32,
    pub reboot_required: bool,
}

impl DebianState {
    pub fn summary(&self) -> String {
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
pub struct OrphanPackage {
    pub id: String,
    pub size_bytes: u64,
}
