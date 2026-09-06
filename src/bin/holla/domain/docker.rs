//! docker: containers plus the `docker system df` accounting classes —
//! images, containers, volumes, builder cache. Cleanup reads these numbers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerState {
    Running,
    Exited,
    Restarting,
    Paused,
}

impl ContainerState {
    pub fn label(self) -> &'static str {
        match self {
            ContainerState::Running => "running",
            ContainerState::Exited => "exited",
            ContainerState::Restarting => "restarting",
            ContainerState::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Healthy,
    Unhealthy,
    Starting,
}

impl Health {
    pub fn label(self) -> &'static str {
        match self {
            Health::Healthy => "healthy",
            Health::Unhealthy => "unhealthy",
            Health::Starting => "starting",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    pub health: Option<Health>,
}

/// Disk accounting per class, mirroring `docker system df -v`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DockerState {
    pub containers: Vec<Container>,
    pub images: u32,
    pub networks: u32,
    pub volumes: u32,
    pub image_bytes: u64,
    pub container_bytes: u64,
    pub volume_bytes: u64,
    pub build_cache_bytes: u64,
}

impl DockerState {
    pub fn running(&self) -> usize {
        self.containers
            .iter()
            .filter(|c| c.state == ContainerState::Running)
            .count()
    }

    pub fn unhealthy(&self) -> Vec<&Container> {
        self.containers
            .iter()
            .filter(|c| c.health == Some(Health::Unhealthy))
            .collect()
    }

    pub fn total_bytes(&self) -> u64 {
        self.image_bytes + self.container_bytes + self.volume_bytes + self.build_cache_bytes
    }

    /// Reclaimable by a full cleanup: builder cache, exited containers and
    /// the dangling share of images (fixture assumes half).
    pub fn reclaimable_bytes(&self) -> u64 {
        let exited: u64 = self
            .containers
            .iter()
            .filter(|c| c.state == ContainerState::Exited)
            .count() as u64;
        self.build_cache_bytes + self.image_bytes / 2 + exited * 40 * 1_024 * 1_024
    }
}
