#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpReference {
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
    pub fn canonical(&self) -> String {
        match &self.section {
            Some(s) => format!(
                "op://{}/{}/{}/{}",
                self.vault_id, self.item_id, s, self.field_id
            ),
            None => format!("op://{}/{}/{}", self.vault_id, self.item_id, self.field_id),
        }
    }

    pub fn display_path(&self) -> String {
        format!(
            "{} › {} › {}",
            self.vault_name, self.item_title, self.field_label
        )
    }
}
