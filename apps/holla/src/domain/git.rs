//! git: one repository's truth. Dirty/ahead/behind/diverged/detached are all
//! first-class; the primary branch is resolved per repo, never hard-coded.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Worktree {
    pub(crate) path: String,
    pub(crate) branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitRepo {
    pub(crate) root: String,
    /// `None` when detached; `detached_sha` then carries the commit.
    pub(crate) branch: Option<String>,
    pub(crate) detached_sha: Option<String>,
    pub(crate) upstream: Option<String>,
    pub(crate) ahead: u32,
    pub(crate) behind: u32,
    pub(crate) modified: u32,
    pub(crate) staged: u32,
    pub(crate) untracked: u32,
    /// Resolved primary branch for THIS repo (`trunk`, `main`, …).
    pub(crate) primary_branch: String,
    pub(crate) worktrees: Vec<Worktree>,
    /// Submodules: children tracked by this repo.
    pub(crate) submodules: Vec<String>,
    /// Nested independent repositories that are NOT submodules — each a
    /// full repo so bulk plans resolve per-child primary branches.
    pub(crate) children: Vec<GitRepo>,
}

impl GitRepo {
    pub(crate) fn dirty(&self) -> bool {
        self.modified + self.staged + self.untracked > 0
    }

    pub(crate) fn detached(&self) -> bool {
        self.branch.is_none()
    }

    pub(crate) fn diverged(&self) -> bool {
        self.ahead > 0 && self.behind > 0
    }

    /// Short truth for reasons: `3 commits behind`, `4 modified files`.
    pub(crate) fn summary(&self) -> Vec<String> {
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

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "Tests assert fixed fixture structure and bounded values; violations must fail the test"
)]
mod tests {
    use super::*;

    fn repo() -> GitRepo {
        GitRepo {
            root: "~/x".into(),
            branch: Some("b".into()),
            detached_sha: None,
            upstream: None,
            ahead: 0,
            behind: 0,
            modified: 0,
            staged: 0,
            untracked: 0,
            primary_branch: "trunk".into(),
            worktrees: vec![],
            submodules: vec![],
            children: vec![],
        }
    }

    #[test]
    fn summary_names_diverged_and_detached() {
        let mut r = repo();
        r.ahead = 2;
        r.behind = 3;
        assert!(r.diverged());
        assert_eq!(r.summary()[0], "diverged · 2 ahead, 3 behind");

        let mut r = repo();
        r.branch = None;
        r.detached_sha = Some("9f3a21c".into());
        assert!(r.detached());
        assert_eq!(r.summary()[0], "detached at 9f3a21c");

        assert_eq!(repo().summary()[0], "working tree clean");
    }
}
