//! Effects: what a finished run changes in the fixture world. Nothing is
//! pretended — a restarted container heals, a cancelled backend frees its
//! waiters, a cleanup removes what it claimed — so the root experience is
//! truthful after an action, not only before it. Every effect is declared
//! by the action that owns it; nothing is inferred from an id prefix.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    DockerRestart(String),
    DockerStop(String),
    /// Stop every running container, then remove the captured set.
    DockerStopAll,
    DockerRemoveAll,
    DockerImagesPrune,
    DockerNetworkPrune,
    DockerVolumePrune,
    DockerBuilderPrune,
    ComposeUp,
    ComposeDown,
    ServiceRestart(String),
    PgCancel(u32),
    PgTerminate(u32),
    /// Fast-forward the repository at this path.
    GitPull(String),
    /// A merge or rebase pull that lands the remote commits.
    GitPullMerge(String),
    GitPush(String),
    /// Push to a named remote of the repository.
    GitPushRemote(String, String),
    GitFetch(String),
    GitSwitch(String, String),
    GitGc(String),
    GitDeleteBranches(String, Vec<String>),
    /// Remove the Cargo target directory at this exact path.
    CargoClean(String),
    MiseInstall,
    MiseUpgrade,
    BrewService(String, String),
    /// Homebrew formulae (and casks when `true`) upgraded.
    BrewUpgrade(bool),
    AmpUpdate,
    OmzUpgrade,
    GradleStop,
    /// `gradle clean` in this project.
    GradleClean(String),
    KillProcess(u32),
    /// Remove every entry below this folder through the tool-native command.
    DeleteAll(String),
}
