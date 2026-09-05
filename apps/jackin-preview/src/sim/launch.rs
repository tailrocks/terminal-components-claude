//! Simulated launch pipeline: the exact ordered 11-stage model advancing
//! on a deterministic tick timeline, a bounded build log, typed failure,
//! and the credential stage's account resolution.

use junie_tui::StepState;

use crate::domain::agent::Agent;
use crate::domain::instance::RunId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Identity,
    Role,
    Credentials,
    Construct,
    AgentBinaries,
    DerivedImage,
    Workspace,
    Network,
    Sidecar,
    Capsule,
    Hardline,
}

impl Stage {
    pub const ALL: [Stage; 11] = [
        Stage::Identity,
        Stage::Role,
        Stage::Credentials,
        Stage::Construct,
        Stage::AgentBinaries,
        Stage::DerivedImage,
        Stage::Workspace,
        Stage::Network,
        Stage::Sidecar,
        Stage::Capsule,
        Stage::Hardline,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stage::Identity => "Identity",
            Stage::Role => "Role",
            Stage::Credentials => "Credentials",
            Stage::Construct => "Construct",
            Stage::AgentBinaries => "Agent Binaries",
            Stage::DerivedImage => "Derived Image",
            Stage::Workspace => "Workspace",
            Stage::Network => "Network",
            Stage::Sidecar => "Sidecar",
            Stage::Capsule => "Capsule",
            Stage::Hardline => "Hardline",
        }
    }

    pub fn index(self) -> usize {
        Stage::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    /// Activity text while running (first word upper-cased, trailing `…`).
    pub fn activity(self, agent: Agent) -> String {
        match self {
            Stage::Identity => "Resolving launch identity…".into(),
            Stage::Role => "Loading role manifest…".into(),
            Stage::Credentials => "Resolving credentials…".into(),
            Stage::Construct => "Preparing the Construct image…".into(),
            Stage::AgentBinaries => format!("Installing {} binaries…", agent.label()),
            Stage::DerivedImage => "Building derived image…".into(),
            Stage::Workspace => "Mounting the Workspace…".into(),
            Stage::Network => "Attaching the network…".into(),
            Stage::Sidecar => "Starting the sidecar…".into(),
            Stage::Capsule => "Waiting for the Capsule daemon…".into(),
            Stage::Hardline => "Opening the hardline…".into(),
        }
    }
}

/// How the fixture wants the pipeline to behave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchPlan {
    /// Every stage succeeds; Agent Binaries is skipped (cached).
    Clean,
    /// Fails at Network after the build produced output.
    FailNetwork,
    /// Credentials stage hits a locked 1Password reference (recoverable).
    CredentialsLocked,
    /// Sidecar is Blocked: a modeled-only fixture with no runtime producer.
    BlockedSidecar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchFailure {
    pub stage: Stage,
    pub summary: String,
    pub next_step: String,
    pub detail: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchEvent {
    StageChanged(Stage, StepState),
    Activity(String),
    BuildLine(String),
    ContainerReady(String),
    CredentialsResolved {
        origin: String,
        validation: String,
    },
    /// Recoverable: the operator may retry or cancel.
    CredentialError {
        message: String,
    },
    Failed(LaunchFailure),
    Ready,
}

#[derive(Debug, Clone)]
pub struct LaunchRun {
    pub plan: LaunchPlan,
    pub run_id: RunId,
    pub container: String,
    pub agent: Agent,
    pub states: [StepState; 11],
    pub durations: [u64; 11],
    pub tick: u64,
    pub stage_start: u64,
    pub current: Option<usize>,
    pub done: bool,
    pub failure: Option<LaunchFailure>,
    pub blocked_at: Option<Stage>,
    /// Credentials stage paused on an error awaiting a decision.
    pub credential_hold: bool,
    pub credential_retried: bool,
    pub build_lines_emitted: usize,
    pub cancelled: bool,
}

/// Fixture Docker build output (ANSI-like markup handled by the viewer).
pub const BUILD_LOG: [&str; 44] = [
    "#1 [internal] load build definition from Dockerfile.derived",
    "#1 transferring dockerfile: 2.31kB done",
    "#1 DONE 0.0s",
    "#2 [internal] load metadata for ghcr.io/jackin/construct:0.6.4",
    "#2 DONE 0.4s",
    "#3 [internal] load .dockerignore",
    "#3 transferring context: 2B done",
    "#3 DONE 0.0s",
    "#4 [ 1/9] FROM ghcr.io/jackin/construct:0.6.4@sha256:9c41e2f0a7b3c5d6e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1",
    "#4 resolve ghcr.io/jackin/construct:0.6.4 0.1s done",
    "#4 CACHED",
    "#5 [internal] load build context",
    "#5 transferring context: 41.02kB 0.0s done",
    "#5 DONE 0.0s",
    "#6 [ 2/9] RUN --mount=type=cache,target=/var/cache/apt apt-get update && apt-get install -y --no-install-recommends git-lfs ripgrep fd-find jq",
    "#6 0.812 Get:1 http://deb.debian.org/debian bookworm InRelease [151 kB]",
    "#6 1.204 Get:2 http://deb.debian.org/debian bookworm-updates InRelease [55.4 kB]",
    "#6 2.331 Reading package lists...",
    "#6 3.118 Building dependency tree...",
    "#6 3.902 The following NEW packages will be installed: fd-find git-lfs jq ripgrep",
    "#6 5.410 Setting up ripgrep (13.0.0-4+b2) ...",
    "#6 5.612 Setting up git-lfs (3.3.0-1+b5) ...",
    "#6 DONE 6.1s",
    "#7 [ 3/9] RUN corepack enable && corepack prepare pnpm@9.12.0 --activate",
    "#7 1.930 Preparing pnpm@9.12.0 for immediate activation...",
    "#7 DONE 2.4s",
    "#8 [ 4/9] COPY roles/the-architect/manifest.toml /jackin/role/manifest.toml",
    "#8 DONE 0.0s",
    "#9 [ 5/9] RUN /jackin/bin/role-install --manifest /jackin/role/manifest.toml",
    "#9 0.211 role: the-architect@1.4.2 (github.com/chainargos/roles)",
    "#9 0.480 install: cargo-nextest 0.9.72",
    "#9 4.115 install: sqlfluff 3.1.0",
    "#9 7.309 warning: hook post-install.sh exited 0 with output on stderr",
    "#9 DONE 7.6s",
    "#10 [ 6/9] RUN useradd --uid 501 --create-home operator && chown -R operator /workspace",
    "#10 DONE 0.5s",
    "#11 [ 7/9] COPY --chown=operator agent-home/ /home/operator/.claude/",
    "#11 DONE 0.1s",
    "#12 [ 8/9] RUN /jackin/bin/agent-install claude@2.1.14",
    "#12 0.104 claude 2.1.14 already present in layer cache",
    "#12 DONE 0.2s",
    "#13 [ 9/9] LABEL org.jackin.run=9c41e2f0 org.jackin.workspace=payments-platform",
    "#13 DONE 0.0s",
    "#14 exporting to image",
];

impl LaunchRun {
    pub fn new(plan: LaunchPlan, agent: Agent, container: &str, run_id: RunId) -> Self {
        // durations in ticks (33 ms)
        let durations = [14, 18, 26, 30, 8, 92, 22, 20, 18, 34, 16];
        Self {
            plan,
            run_id,
            container: container.to_owned(),
            agent,
            states: [StepState::Queued; 11],
            durations,
            tick: 0,
            stage_start: 0,
            current: None,
            done: false,
            failure: None,
            blocked_at: None,
            credential_hold: false,
            credential_retried: false,
            build_lines_emitted: 0,
            cancelled: false,
        }
    }

    pub fn counts(&self) -> (usize, usize) {
        (
            self.states
                .iter()
                .filter(|s| **s == StepState::Done)
                .count(),
            self.states
                .iter()
                .filter(|s| **s == StepState::Skipped)
                .count(),
        )
    }

    pub fn is_terminal(&self) -> bool {
        self.done || self.failure.is_some() || self.blocked_at.is_some() || self.cancelled
    }

    /// The operator retries the credential stage after fixing the source.
    pub fn retry_credentials(&mut self) {
        self.credential_hold = false;
        self.credential_retried = true;
        self.stage_start = self.tick;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
        if let Some(i) = self.current
            && let Some(state) = self.states.get_mut(i)
        {
            *state = StepState::Failed;
        }
    }

    /// Advance one tick; returns the events produced.
    pub fn advance(&mut self) -> Vec<LaunchEvent> {
        let mut ev = vec![];
        if self.is_terminal() || self.credential_hold {
            return ev;
        }
        self.tick += 1;
        let Some(i) = self.current else {
            self.current = Some(0);
            self.stage_start = self.tick;
            self.states[0] = StepState::Running;
            ev.push(LaunchEvent::StageChanged(
                Stage::Identity,
                StepState::Running,
            ));
            ev.push(LaunchEvent::Activity(Stage::Identity.activity(self.agent)));
            return ev;
        };
        let Some(stage) = Stage::ALL.get(i).copied() else {
            self.current = None;
            return ev;
        };
        let Some(&duration) = self.durations.get(i) else {
            self.current = None;
            return ev;
        };
        let elapsed = self.tick - self.stage_start;
        // stage-specific mid-stage events
        match stage {
            Stage::DerivedImage => {
                let want =
                    ((elapsed as usize) * BUILD_LOG.len() / duration as usize).min(BUILD_LOG.len());
                while self.build_lines_emitted < want {
                    let Some(line) = BUILD_LOG.get(self.build_lines_emitted) else {
                        break;
                    };
                    ev.push(LaunchEvent::BuildLine((*line).to_owned()));
                    self.build_lines_emitted += 1;
                }
            }
            Stage::Construct if elapsed == 6 => {
                ev.push(LaunchEvent::ContainerReady(self.container.clone()));
            }
            Stage::Credentials
                if elapsed == 10
                    && self.plan == LaunchPlan::CredentialsLocked
                    && !self.credential_retried =>
            {
                self.credential_hold = true;
                ev.push(LaunchEvent::CredentialError {
                    message: "1Password locked: unlock the app and retry".into(),
                });
                return ev;
            }
            _ => {}
        }
        if elapsed < duration {
            return ev;
        }
        // finish this stage
        let final_state = match (self.plan, stage) {
            (_, Stage::AgentBinaries) => StepState::Skipped,
            (LaunchPlan::FailNetwork, Stage::Network) => StepState::Failed,
            (LaunchPlan::BlockedSidecar, Stage::Sidecar) => StepState::Blocked,
            _ => StepState::Done,
        };
        if let Some(state) = self.states.get_mut(i) {
            *state = final_state;
        }
        ev.push(LaunchEvent::StageChanged(stage, final_state));
        if stage == Stage::Credentials {
            ev.push(LaunchEvent::CredentialsResolved {
                origin: String::new(),
                validation: String::new(),
            });
        }
        match final_state {
            StepState::Failed => {
                let f = LaunchFailure {
                    stage,
                    summary: "The Construct network could not be attached".into(),
                    next_step: "Check that Docker's jackin-net bridge exists (docker network ls), then launch again.".into(),
                    detail: vec![
                        "network: attach jackin-net to jackin-payments-platform-7f3a".into(),
                        "docker: Error response from daemon: network jackin-net not found".into(),
                        "retry 1/3 after 400 ms: network jackin-net not found".into(),
                        "retry 2/3 after 800 ms: network jackin-net not found".into(),
                        "retry 3/3 after 1600 ms: network jackin-net not found".into(),
                        "sidecar: not started (network stage failed)".into(),
                        "capsule: not started".into(),
                        "cleanup: container jackin-payments-platform-7f3a kept for inspection".into(),
                        "credentials: material was resolved in memory and discarded".into(),
                        "run: 9c41e2f0 · backend docker 27.1 · host darwin 25.5".into(),
                    ],
                };
                self.failure = Some(f.clone());
                ev.push(LaunchEvent::Failed(f));
                return ev;
            }
            StepState::Blocked => {
                self.blocked_at = Some(stage);
                ev.push(LaunchEvent::Activity(
                    "Blocked · modeled state with no runtime producer".into(),
                ));
                return ev;
            }
            _ => {}
        }
        let next_index = i.saturating_add(1);
        if let Some(next) = Stage::ALL.get(next_index).copied() {
            self.current = Some(next_index);
            self.stage_start = self.tick;
            if let Some(state) = self.states.get_mut(next_index) {
                *state = StepState::Running;
            }
            ev.push(LaunchEvent::StageChanged(next, StepState::Running));
            ev.push(LaunchEvent::Activity(next.activity(self.agent)));
        } else {
            self.done = true;
            ev.push(LaunchEvent::Ready);
        }
        ev
    }

    /// Seek to a deterministic frame without consulting wall time.
    pub fn seek(&mut self, frame: u64) {
        for _ in 0..frame {
            if self.is_terminal() {
                break;
            }
            let _ = self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_to_end(mut r: LaunchRun) -> (LaunchRun, Vec<LaunchEvent>) {
        let mut all = vec![];
        for _ in 0..2_000 {
            let ev = r.advance();
            all.extend(ev);
            if r.is_terminal() {
                break;
            }
        }
        (r, all)
    }

    #[test]
    fn clean_plan_walks_all_eleven_stages_in_order() {
        let (r, ev) = run_to_end(LaunchRun::new(
            LaunchPlan::Clean,
            Agent::ClaudeCode,
            "c",
            RunId::new(1),
        ));
        assert!(r.done);
        let running: Vec<Stage> = ev
            .iter()
            .filter_map(|e| match e {
                LaunchEvent::StageChanged(s, StepState::Running) => Some(*s),
                _ => None,
            })
            .collect();
        assert_eq!(running, Stage::ALL.to_vec());
        assert_eq!(r.states[Stage::AgentBinaries.index()], StepState::Skipped);
        assert_eq!(r.counts(), (10, 1));
        assert_eq!(r.build_lines_emitted, BUILD_LOG.len());
        assert!(matches!(ev.last(), Some(LaunchEvent::Ready)));
    }

    #[test]
    fn failure_and_blocked_plans_stop_the_frontier() {
        let (r, ev) = run_to_end(LaunchRun::new(
            LaunchPlan::FailNetwork,
            Agent::Codex,
            "c",
            RunId::new(2),
        ));
        assert_eq!(r.failure.as_ref().map(|f| f.stage), Some(Stage::Network));
        assert!(ev.iter().any(|e| matches!(e, LaunchEvent::Failed(_))));
        assert_eq!(r.states[Stage::Sidecar.index()], StepState::Queued);
        let (b, _) = run_to_end(LaunchRun::new(
            LaunchPlan::BlockedSidecar,
            Agent::Codex,
            "c",
            RunId::new(3),
        ));
        assert_eq!(b.blocked_at, Some(Stage::Sidecar));
    }

    #[test]
    fn credential_error_holds_until_retry() {
        let mut r = LaunchRun::new(
            LaunchPlan::CredentialsLocked,
            Agent::Codex,
            "c",
            RunId::new(4),
        );
        let mut held = false;
        for _ in 0..200 {
            let ev = r.advance();
            if ev
                .iter()
                .any(|e| matches!(e, LaunchEvent::CredentialError { .. }))
            {
                held = true;
                break;
            }
        }
        assert!(held && r.credential_hold);
        assert!(r.advance().is_empty());
        r.retry_credentials();
        let (r, _) = run_to_end(r);
        assert!(r.done);
    }
}
