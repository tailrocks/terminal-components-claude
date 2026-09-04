//! 1Password reference metadata. A reference names an account, vault, item
//! and field; it never carries the resolved value. Resolution happens only
//! inside the simulated credential service (`sim::onepassword`).

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpReference {
    /// 1Password account short id, e.g. `chainargos.1password.com`.
    pub account: String,
    pub vault_id: String,
    pub vault_name: String,
    pub item_id: String,
    pub item_title: String,
    pub section: Option<String>,
    pub field_id: String,
    pub field_label: String,
}

impl OpReference {
    /// `op://<vault>/<item>/[<section>/]<field>` with ids.
    pub fn canonical(&self) -> String {
        match &self.section {
            Some(s) => format!(
                "op://{}/{}/{}/{}",
                self.vault_id, self.item_id, s, self.field_id
            ),
            None => format!("op://{}/{}/{}", self.vault_id, self.item_id, self.field_id),
        }
    }

    /// `Engineering › OpenAI · Codex Primary › credential` with names.
    pub fn display_path(&self) -> String {
        format!(
            "{} › {} › {}",
            self.vault_name, self.item_title, self.field_label
        )
    }
}
