//! gh: the signed-in account, its orgs and repos a clone could target.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhRepo {
    pub owner: String,
    pub name: String,
    pub default_branch: String,
    pub private: bool,
}

impl GhRepo {
    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhState {
    pub login: String,
    pub orgs: Vec<String>,
    pub repos: Vec<GhRepo>,
}
