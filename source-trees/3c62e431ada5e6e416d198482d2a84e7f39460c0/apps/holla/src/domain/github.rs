//! gh: the signed-in account, its orgs and repos a clone could target.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GhRepo {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) default_branch: String,
    pub(crate) private: bool,
}

impl GhRepo {
    pub(crate) fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GhState {
    pub(crate) login: String,
    pub(crate) orgs: Vec<String>,
    pub(crate) repos: Vec<GhRepo>,
}
