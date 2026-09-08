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
    pub fn reclaimed_bytes(&self) -> u64 {
        match self {
            Self::RemoveContainers(items) => items.iter().map(|item| item.size_bytes).sum(),
            Self::RemoveImages(items)
            | Self::RemoveVolumes(items)
            | Self::RemoveNetworks(items) => items.iter().map(|item| item.size_bytes).sum(),
            Self::PruneCache(bytes) => *bytes,
            Self::RemoveDisk(item) => item.size_bytes,
            Self::RemoveOrphanPackages(items) => items.iter().map(|item| item.size_bytes).sum(),
            _ => 0,
        }
    }

    pub fn lines(&self, partial: bool) -> Vec<String> {
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
                        format!(
                            "Total reclaimed: {}",
                            human_bytes(containers.iter().map(|c| c.size_bytes).sum())
                        ),
                    ]
                }
            }
            Self::RemoveImages(resources) => vec![
                format!("Deleted {} images", resources.len()),
                format!(
                    "Total reclaimed: {}",
                    human_bytes(resources.iter().map(|r| r.size_bytes).sum())
                ),
            ],
            Self::RemoveVolumes(resources) => vec![
                format!("Deleted {} volumes", resources.len()),
                format!(
                    "Total reclaimed: {}",
                    human_bytes(resources.iter().map(|r| r.size_bytes).sum())
                ),
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
                format!(
                    "Freed {}",
                    human_bytes(packages.iter().map(|p| p.size_bytes).sum())
                ),
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
