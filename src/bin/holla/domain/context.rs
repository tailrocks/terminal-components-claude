//! Context: the host, the working directory, the surrounding project and
//! workspace, and the scope vocabulary every result carries.

/// Where the terminal is and what kind of machine that is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    pub name: String,
    pub role: HostRole,
    pub os: Os,
    pub user: String,
    /// True when the session arrived over SSH.
    pub remote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostRole {
    Local,
    Development,
    Production,
}

impl HostRole {
    pub fn label(self) -> &'static str {
        match self {
            HostRole::Local => "local",
            HostRole::Development => "development",
            HostRole::Production => "production",
        }
    }
    /// Production and staging hosts take the stronger confirmation path.
    pub fn sensitive(self) -> bool {
        matches!(self, HostRole::Production)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Debian,
}

impl Os {
    pub fn label(self) -> &'static str {
        match self {
            Os::MacOs => "macOS",
            Os::Debian => "Debian 13",
        }
    }
}

/// The four navigable scope directions (CONCEPT §5.3). `Here` is the
/// default bias; the others narrow the root to one direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Scope {
    #[default]
    Here,
    Parent,
    Children,
    System,
}

impl Scope {
    /// One step outward on the axis (`Children → Here → Parent → System`).
    pub fn outward(self) -> Scope {
        match self {
            Scope::Children => Scope::Here,
            Scope::Here => Scope::Parent,
            Scope::Parent | Scope::System => Scope::System,
        }
    }

    /// One step inward on the axis.
    pub fn inward(self) -> Scope {
        match self {
            Scope::System => Scope::Parent,
            Scope::Parent => Scope::Here,
            Scope::Here | Scope::Children => Scope::Children,
        }
    }
}

/// Where a result was discovered and where it will execute. Every result
/// carries one so scope is never implicit (CONCEPT §5.4, §5.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeTag {
    /// The direction this result belongs to relative to the working directory.
    pub direction: Scope,
    /// The path that defines the action (a manifest, a config, the host).
    pub defined_at: String,
    /// The effective working directory the action runs in.
    pub runs_in: String,
    /// Short human word for the row: `here`, `project`, `parent`, `child`,
    /// `host`.
    pub word: String,
}

impl ScopeTag {
    pub fn here(cwd: &str) -> Self {
        Self {
            direction: Scope::Here,
            defined_at: cwd.to_owned(),
            runs_in: cwd.to_owned(),
            word: "here".into(),
        }
    }
    pub fn project(root: &str, runs_in: &str) -> Self {
        Self {
            direction: Scope::Here,
            defined_at: root.to_owned(),
            runs_in: runs_in.to_owned(),
            word: "project".into(),
        }
    }
    pub fn parent(root: &str) -> Self {
        Self {
            direction: Scope::Parent,
            defined_at: root.to_owned(),
            runs_in: root.to_owned(),
            word: "parent".into(),
        }
    }
    pub fn child(path: &str) -> Self {
        Self {
            direction: Scope::Children,
            defined_at: path.to_owned(),
            runs_in: path.to_owned(),
            word: "child".into(),
        }
    }
    pub fn host(host: &str) -> Self {
        Self {
            direction: Scope::System,
            defined_at: host.to_owned(),
            runs_in: host.to_owned(),
            word: "host".into(),
        }
    }
    /// A remote host's scope word carries the host name so a production
    /// target is never a bare `host`.
    pub fn remote(host: &str) -> Self {
        Self {
            direction: Scope::System,
            defined_at: host.to_owned(),
            runs_in: host.to_owned(),
            word: format!("on {host}"),
        }
    }
}

/// A recognised project or workspace member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub root: String,
    pub kind: ProjectKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    /// A Cargo package or workspace; `member` is the package the working
    /// directory sits in when the root is a workspace.
    Rust {
        workspace: bool,
        member: Option<String>,
    },
    /// A Node project with its detected package manager.
    Node { manager: &'static str },
    /// A mise monorepo root with configured child roots.
    MiseMonorepo,
    /// A Gradle multi-project build.
    Gradle,
    /// A Python project.
    Python,
    /// A folder of independent Git projects (a project collection).
    Collection,
    /// A remote service checkout with no build system of its own.
    Service,
}

impl ProjectKind {
    pub fn label(&self) -> String {
        match self {
            ProjectKind::Rust { workspace, member } => match (workspace, member) {
                (true, Some(m)) => format!("Rust workspace · member {m}"),
                (true, None) => "Rust workspace".into(),
                _ => "Rust project".into(),
            },
            ProjectKind::Node { manager } => format!("Node project · {manager}"),
            ProjectKind::MiseMonorepo => "mise monorepo".into(),
            ProjectKind::Gradle => "Gradle build".into(),
            ProjectKind::Python => "Python project".into(),
            ProjectKind::Collection => "project collection".into(),
            ProjectKind::Service => "service checkout".into(),
        }
    }
}

/// The location Holla was launched from, resolved outward through its
/// rings: the exact folder, the nearest project, the workspace above it, and
/// the bounded children below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub cwd: String,
    pub home: String,
    /// Nearest project root containing `cwd` (may equal `cwd`).
    pub project: Option<Project>,
    /// A workspace or monorepo above the project, when one is recognised.
    pub workspace: Option<Project>,
    /// Bounded child projects below the active scope.
    pub children: Vec<Project>,
}

impl Location {
    /// `~/work/acme/apps/frontend` for display.
    pub fn short(&self, path: &str) -> String {
        match path.strip_prefix(&self.home) {
            Some(rest) => format!("~{rest}"),
            None => path.to_owned(),
        }
    }

    pub fn cwd_short(&self) -> String {
        self.short(&self.cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_axis_moves_outward_and_inward_and_saturates() {
        assert_eq!(Scope::Here.outward(), Scope::Parent);
        assert_eq!(Scope::Parent.outward(), Scope::System);
        assert_eq!(Scope::System.outward(), Scope::System);
        assert_eq!(Scope::Here.inward(), Scope::Children);
        assert_eq!(Scope::Children.inward(), Scope::Children);
        assert_eq!(Scope::System.inward(), Scope::Parent);
    }

    #[test]
    fn short_paths_use_the_home_tilde() {
        let l = Location {
            cwd: "/Users/alex/work/acme".into(),
            home: "/Users/alex".into(),
            project: None,
            workspace: None,
            children: vec![],
        };
        assert_eq!(l.cwd_short(), "~/work/acme");
        assert_eq!(l.short("/srv/payments"), "/srv/payments");
    }
}
