//! Typed in-memory mutations. Commands and output text never select effects.
use super::{
    disk::Candidate,
    docker::{Container, Resource},
    human_bytes,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Mutation {
    RemoveContainers(Vec<Container>),
    RemoveImages(Vec<Resource>),
    RemoveVolumes(Vec<Resource>),
    RemoveNetworks(Vec<Resource>),
    PruneCache(u64),
    RemoveDisk(Candidate),
    CheckoutGit {
        root: String,
        branch: String,
        upstream: String,
    },
    FastForwardGit {
        root: String,
        branch: String,
        commits: u32,
    },
    RemoveOrphanPackages(Vec<super::debian::OrphanPackage>),
    UpgradeDebian {
        upgraded: u32,
        security: u32,
        held: u32,
    },
    UpgradeTool {
        name: String,
        before: String,
        after: String,
    },
    RestartContainer(Container),
}

impl Mutation {
    pub(crate) fn reclaimed_bytes(&self) -> Result<u64, super::accounting::InventoryError> {
        use super::accounting::bytes;
        match self {
            Self::RemoveContainers(items) => bytes(items.iter().map(|item| item.size_bytes)),
            Self::RemoveImages(items)
            | Self::RemoveVolumes(items)
            | Self::RemoveNetworks(items) => bytes(items.iter().map(|item| item.size_bytes)),
            Self::PruneCache(bytes) => Ok(*bytes),
            Self::RemoveDisk(item) => Ok(item.size_bytes),
            Self::RemoveOrphanPackages(items) => bytes(items.iter().map(|item| item.size_bytes)),
            _ => Ok(0),
        }
    }

    pub(crate) fn lines(&self, partial: bool) -> Vec<String> {
        let amount = match self.reclaimed_bytes() {
            Ok(bytes) => human_bytes(bytes),
            Err(error) => return vec![error.to_string()],
        };
        match self {
            Self::RemoveContainers(containers) => {
                if partial {
                    containers
                        .iter()
                        .map(|c| format!("Removed {}", c.name))
                        .collect()
                } else {
                    vec![
                        format!(
                            "Removed {}",
                            containers
                                .iter()
                                .map(|c| c.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        format!("Total reclaimed: {}", amount.clone()),
                    ]
                }
            }
            Self::RemoveImages(resources) => vec![
                format!("Deleted {} images", resources.len()),
                format!("Total reclaimed: {}", amount.clone()),
            ],
            Self::RemoveVolumes(resources) => vec![
                format!("Deleted {} volumes", resources.len()),
                format!("Total reclaimed: {}", amount.clone()),
            ],
            Self::RemoveNetworks(resources) => {
                vec![format!("Removed {} unused networks", resources.len())]
            }
            Self::PruneCache(bytes) => vec![
                "Deleted build cache entries".into(),
                format!("Total reclaimed: {}", human_bytes(*bytes)),
            ],
            Self::RemoveDisk(candidate) => vec![
                format!("Removed {}", candidate.path),
                format!("Reclaimed {}", human_bytes(candidate.size_bytes)),
            ],
            Self::CheckoutGit { branch, .. } => vec![format!("Switched to branch '{branch}'")],
            Self::FastForwardGit {
                branch, commits, ..
            } => {
                vec![
                    format!("Updating origin/{branch}.."),
                    format!("Fast-forward · {commits} commits"),
                ]
            }
            Self::RemoveOrphanPackages(packages) => vec![
                format!("Removing {} orphaned packages", packages.len()),
                format!("Freed {}", amount.clone()),
            ],
            Self::UpgradeDebian {
                upgraded,
                security,
                held,
            } => vec![format!(
                "{upgraded} upgraded, {security} security, {held} held back"
            )],
            Self::UpgradeTool {
                name,
                before,
                after,
            } => vec![format!("{name} {before} → {after}")],
            Self::RestartContainer(container) => vec![container.name.clone()],
        }
    }
}
