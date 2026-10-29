use crate::clock::Clock;
use crate::domain::activity::Activity;
use crate::domain::debian::DebianState;
use crate::domain::disk::DiskState;
use crate::domain::docker::DockerState;
use crate::domain::git::GitRepo;
use crate::domain::github::GhState;
use crate::domain::host::Host;
use crate::domain::mise::MiseState;
use crate::domain::pg::PgSession;
use crate::domain::ranking::Memory;
use crate::domain::ssh::SshHost;
use crate::scenario::Scenario;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    Mise,
    Git,
    Ssh,
    Github,
    Docker,
    Pg,
    Disk,
}

impl Domain {
    pub const ALL: [Domain; 7] = [
        Domain::Mise,
        Domain::Git,
        Domain::Ssh,
        Domain::Github,
        Domain::Docker,
        Domain::Pg,
        Domain::Disk,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Domain::Mise => "mise",
            Domain::Git => "git",
            Domain::Ssh => "ssh",
            Domain::Github => "github",
            Domain::Docker => "docker",
            Domain::Pg => "pg",
            Domain::Disk => "disk",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Discovery {
    pub domain: Domain,
    pub at_ms: i64,
    pub fails: bool,
    pub done: bool,
    pub failed: bool,
}

impl Discovery {
    pub fn at(domain: Domain, at_ms: i64) -> Self {
        Self {
            domain,
            at_ms,
            fails: false,
            done: false,
            failed: false,
        }
    }

    pub fn failing(domain: Domain, at_ms: i64) -> Self {
        Self {
            fails: true,
            ..Self::at(domain, at_ms)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    Discovered(Domain),
    DiscoveryFailed(Domain),
}

pub struct World {
    pub effect_revision: u64,
    pub scenario: Scenario,
    pub clock: Clock,
    pub host: Host,
    pub cwd: String,
    pub discovery: Vec<Discovery>,

    pub mise: Option<MiseState>,
    pub git: Option<GitRepo>,
    pub ssh: Vec<SshHost>,
    pub github: Option<GhState>,
    pub docker: Option<DockerState>,
    pub pg: Option<Vec<PgSession>>,
    pub disk: Option<DiskState>,
    pub debian: Option<DebianState>,
    pub memory: Memory,
    pub activities: Vec<Activity>,
}

impl World {
    pub fn new(scenario: Scenario, host: Host, cwd: &str) -> Self {
        Self {
            effect_revision: 0,
            scenario,
            clock: Clock::new(),
            host,
            cwd: cwd.to_owned(),
            discovery: vec![],
            mise: None,
            git: None,
            ssh: vec![],
            github: None,
            docker: None,
            pg: None,
            disk: None,
            debian: None,
            memory: Memory::default(),
            activities: vec![],
        }
    }

    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

    pub fn default_schedule() -> Vec<Discovery> {
        vec![
            Discovery::at(Domain::Mise, 600),
            Discovery::at(Domain::Git, 900),
            Discovery::at(Domain::Ssh, 1_200),
            Discovery::at(Domain::Github, 1_600),
            Discovery::at(Domain::Docker, 2_400),
            Discovery::at(Domain::Pg, 2_800),
            Discovery::at(Domain::Disk, 3_200),
        ]
    }

    pub fn discovering(&self) -> bool {
        self.discovery.iter().any(|d| !d.done && !d.failed)
    }

    pub fn discovered(&self, domain: Domain) -> bool {
        self.discovery.iter().any(|d| d.domain == domain && d.done)
    }

    pub fn discovery_failed(&self, domain: Domain) -> bool {
        self.discovery
            .iter()
            .any(|d| d.domain == domain && d.failed)
    }

    pub fn tick(&mut self, interval_ms: i64) -> Vec<Msg> {
        if !self.clock.running {
            return vec![];
        }
        self.clock.advance(interval_ms);
        self.step()
    }

    pub fn seek(&mut self, frame_ms: u64) {
        let target = i64::try_from(frame_ms).unwrap_or(i64::MAX);
        if target <= self.clock.now_ms {
            return;
        }
        let distance = target.saturating_sub(self.clock.now_ms);
        let ticks = (distance / 80).saturating_add(i64::from(distance % 80 != 0));
        self.clock.now_ms = self.clock.now_ms.saturating_add(ticks.saturating_mul(80));
        self.step();
    }

    pub fn trust_mise_file(&mut self, path: &str) -> bool {
        let Some(mise) = &mut self.mise else {
            return false;
        };
        let mut hit = false;
        for t in &mut mise.tasks {
            if t.defined_in == path && t.trust == crate::domain::mise::Trust::Untrusted {
                t.trust = crate::domain::mise::Trust::Trusted;
                hit = true;
            }
        }
        hit
    }

    fn step(&mut self) -> Vec<Msg> {
        let now = self.clock.now_ms;
        let mut msgs = vec![];
        for d in &mut self.discovery {
            if d.done || d.failed || now < d.at_ms {
                continue;
            }
            if d.fails {
                d.failed = true;
                msgs.push(Msg::DiscoveryFailed(d.domain));
            } else {
                d.done = true;
                msgs.push(Msg::Discovered(d.domain));
            }
        }
        msgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host::Environment;

    #[test]
    fn discovery_is_progressive_and_failures_settle() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = vec![
            Discovery::at(Domain::Git, 100),
            Discovery::failing(Domain::Docker, 200),
        ];
        assert!(w.discovering());
        w.clock.running = true;
        let msgs = w.tick(150);
        assert_eq!(msgs, vec![Msg::Discovered(Domain::Git)]);
        assert!(w.discovering());
        let msgs = w.tick(100);
        assert_eq!(msgs, vec![Msg::DiscoveryFailed(Domain::Docker)]);
        assert!(!w.discovering());
        assert!(w.discovery_failed(Domain::Docker));
        assert!(w.discovered(Domain::Git));

        assert!(w.tick(100).is_empty());
        let _ = Environment::Local;
    }
    #[test]
    fn seek_preserves_reference_quantization_and_pause() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.clock.running = false;
        w.seek(100);
        assert_eq!(w.now_ms(), 160);
        assert!(!w.clock.running);
        w.seek(80);
        assert_eq!(w.now_ms(), 160, "seek never moves backwards");
        w.clock.now_ms = 33;
        w.seek(100);
        assert_eq!(
            w.now_ms(),
            113,
            "quantization starts at the current instant"
        );
    }

    #[test]
    fn seek_maximum_frame_is_bounded_and_settles_discovery() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = World::default_schedule();
        w.seek(u64::MAX);
        assert_eq!(w.now_ms(), i64::MAX);
        assert!(!w.discovering());
        assert!(w.tick(80).is_empty());
        assert_eq!(w.now_ms(), i64::MAX);
    }
}
