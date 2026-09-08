//! docker: containers plus the `docker system df` accounting classes —
//! images, containers, volumes, builder cache. Cleanup reads these numbers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContainerState {
    Running,
    Exited,
    Restarting,
    Paused,
}

impl ContainerState {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ContainerState::Running => "running",
            ContainerState::Exited => "exited",
            ContainerState::Restarting => "restarting",
            ContainerState::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Health {
    Healthy,
    Unhealthy,
    Starting,
}

impl Health {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Health::Healthy => "healthy",
            Health::Unhealthy => "unhealthy",
            Health::Starting => "starting",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Container {
    pub(crate) size_bytes: u64,
    pub(crate) name: String,
    pub(crate) image: String,
    pub(crate) state: ContainerState,
    pub(crate) health: Option<Health>,
}

/// One deterministic Docker resource. Aggregate counts and bytes are derived
/// from the same inventory that cleanup mutates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Resource {
    pub(crate) id: String,
    pub(crate) size_bytes: u64,
    pub(crate) unused: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DockerState {
    pub(crate) containers: Vec<Container>,
    pub(crate) image_inventory: Vec<Resource>,
    pub(crate) network_inventory: Vec<Resource>,
    pub(crate) volume_inventory: Vec<Resource>,
    pub(crate) cache_bytes: u64,
}

/// Accepted aggregate fixture facts, expanded once into explicit resources.
/// This constructor is fixture data preparation, never live discovery.
pub(crate) struct DockerFixture {
    pub(crate) containers: Vec<Container>,
    pub(crate) images: u32,
    pub(crate) networks: u32,
    pub(crate) volumes: u32,
    pub(crate) image_bytes: u64,
    pub(crate) container_bytes: u64,
    pub(crate) volume_bytes: u64,
    pub(crate) build_cache_bytes: u64,
}

impl DockerFixture {
    pub(crate) fn into_state(mut self) -> DockerState {
        let exited = self
            .containers
            .iter()
            .filter(|c| c.state == ContainerState::Exited)
            .count();
        // The reference explicitly models 40 MiB per stopped container. Retain
        // it, distributing the remaining accepted total across other containers.
        let stopped_size = if exited == 0 {
            0
        } else {
            (40 * 1_024 * 1_024).min(self.container_bytes / exited as u64)
        };
        let remaining = self.container_bytes - stopped_size * exited as u64;
        let active = self.containers.len() - exited;
        let mut active_index = 0;
        for container in &mut self.containers {
            container.size_bytes = if container.state == ContainerState::Exited {
                stopped_size
            } else {
                let bytes = share(remaining, active, active_index);
                active_index += 1;
                bytes
            };
        }
        let unused_images = (self.images / 2) as usize;
        let used_images = self.images as usize - unused_images;
        let unused_bytes = if unused_images == 0 {
            0
        } else {
            self.image_bytes / 2
        };
        let mut image_inventory = resources("unused-image", unused_images, unused_bytes, true);
        image_inventory.extend(resources(
            "used-image",
            used_images,
            self.image_bytes - unused_bytes,
            false,
        ));
        let unused_networks = self.networks.min(2) as usize;
        let mut network_inventory = resources("unused-network", unused_networks, 0, true);
        network_inventory.extend(resources(
            "used-network",
            self.networks as usize - unused_networks,
            0,
            false,
        ));
        DockerState {
            containers: self.containers,
            image_inventory,
            network_inventory,
            volume_inventory: resources(
                "unused-volume",
                self.volumes as usize,
                self.volume_bytes,
                true,
            ),
            cache_bytes: self.build_cache_bytes,
        }
    }
}

fn share(bytes: u64, count: usize, index: usize) -> u64 {
    if count == 0 {
        return 0;
    }
    bytes / count as u64 + u64::from((index as u64) < bytes % count as u64)
}

fn resources(prefix: &str, count: usize, bytes: u64, unused: bool) -> Vec<Resource> {
    (0..count)
        .map(|index| Resource {
            id: format!("fixture:{prefix}:{index}"),
            size_bytes: share(bytes, count, index),
            unused,
        })
        .collect()
}

impl DockerState {
    pub(crate) fn running(&self) -> usize {
        self.containers
            .iter()
            .filter(|c| c.state == ContainerState::Running)
            .count()
    }
    pub(crate) fn unhealthy(&self) -> Vec<&Container> {
        self.containers
            .iter()
            .filter(|c| c.health == Some(Health::Unhealthy))
            .collect()
    }
    pub(crate) fn images(&self) -> usize {
        self.image_inventory.len()
    }
    pub(crate) fn volumes(&self) -> usize {
        self.volume_inventory.len()
    }
    pub(crate) fn networks(&self) -> usize {
        self.network_inventory.len()
    }
    pub(crate) fn image_bytes(&self) -> u64 {
        self.image_inventory.iter().map(|r| r.size_bytes).sum()
    }
    pub(crate) fn container_bytes(&self) -> u64 {
        self.containers.iter().map(|r| r.size_bytes).sum()
    }
    pub(crate) fn volume_bytes(&self) -> u64 {
        self.volume_inventory.iter().map(|r| r.size_bytes).sum()
    }
    pub(crate) fn build_cache_bytes(&self) -> u64 {
        self.cache_bytes
    }
    pub(crate) fn total_bytes(&self) -> u64 {
        self.image_bytes() + self.container_bytes() + self.volume_bytes() + self.cache_bytes
    }
    pub(crate) fn reclaimable_bytes(&self) -> u64 {
        self.cache_bytes
            + self
                .containers
                .iter()
                .filter(|c| c.state == ContainerState::Exited)
                .map(|c| c.size_bytes)
                .sum::<u64>()
            + self
                .image_inventory
                .iter()
                .chain(&self.volume_inventory)
                .filter(|r| r.unused)
                .map(|r| r.size_bytes)
                .sum::<u64>()
    }
}
