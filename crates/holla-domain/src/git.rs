#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    pub path: String,
    pub branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepo {
    pub root: String,
    pub branch: Option<String>,
    pub detached_sha: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub modified: u32,
    pub staged: u32,
    pub untracked: u32,
    pub primary_branch: String,
    pub worktrees: Vec<Worktree>,
    pub submodules: Vec<String>,
    pub children: Vec<GitRepo>,
}

impl GitRepo {
    pub fn dirty(&self) -> bool {
        self.modified != 0 || self.staged != 0 || self.untracked != 0
    }

    pub fn detached(&self) -> bool {
        self.branch.is_none()
    }

    pub fn diverged(&self) -> bool {
        self.ahead > 0 && self.behind > 0
    }

    pub fn summary(&self) -> Vec<String> {
        let mut out = vec![];
        if self.detached() {
            out.push(format!(
                "detached at {}",
                self.detached_sha.as_deref().unwrap_or("?")
            ));
        }
        if self.diverged() {
            out.push(format!(
                "diverged · {} ahead, {} behind",
                self.ahead, self.behind
            ));
        } else if self.behind > 0 {
            out.push(format!(
                "branch is {} {} behind",
                self.behind,
                if self.behind == 1 {
                    "commit"
                } else {
                    "commits"
                }
            ));
        } else if self.ahead > 0 {
            out.push(format!("{} ahead", self.ahead));
        }
        if self.modified > 0 {
            out.push(format!(
                "{} modified {}",
                self.modified,
                if self.modified == 1 { "file" } else { "files" }
            ));
        }
        if self.staged > 0 {
            out.push(format!("{} staged", self.staged));
        }
        if self.untracked > 0 {
            out.push(format!("{} untracked", self.untracked));
        }
        if out.is_empty() {
            out.push("working tree clean".to_owned());
        }
        out
    }
}
