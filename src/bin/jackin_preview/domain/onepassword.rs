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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpReferenceError {
    MissingScheme,
    MissingVault,
    MissingItem,
    MissingField,
    EmptyComponent,
}

impl OpReferenceError {
    pub fn message(&self) -> &'static str {
        match self {
            OpReferenceError::MissingScheme => {
                "Reference malformed: expected op://vault/item/field"
            }
            OpReferenceError::MissingVault => "Reference malformed: vault is missing",
            OpReferenceError::MissingItem => "Reference malformed: item is missing",
            OpReferenceError::MissingField => "Reference malformed: field is missing",
            OpReferenceError::EmptyComponent => "Reference malformed: an empty path component",
        }
    }
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

    /// Short display for rows: `Engineering › … › credential`.
    pub fn display(&self) -> String {
        self.display_path()
    }

    /// Parse a hand-typed canonical reference into an id-only reference
    /// (names are filled in when the service describes it).
    pub fn parse(account: &str, s: &str) -> Result<Self, OpReferenceError> {
        let rest = s
            .strip_prefix("op://")
            .ok_or(OpReferenceError::MissingScheme)?;
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(OpReferenceError::EmptyComponent);
        }
        match parts.len() {
            0 => Err(OpReferenceError::MissingVault),
            1 => Err(OpReferenceError::MissingItem),
            2 => Err(OpReferenceError::MissingField),
            3 | 4 => Ok(Self {
                account: account.to_owned(),
                vault_id: parts[0].to_owned(),
                vault_name: parts[0].to_owned(),
                item_id: parts[1].to_owned(),
                item_title: parts[1].to_owned(),
                section: if parts.len() == 4 {
                    Some(parts[2].to_owned())
                } else {
                    None
                },
                field_id: parts[parts.len() - 1].to_owned(),
                field_label: parts[parts.len() - 1].to_owned(),
            }),
            _ => Err(OpReferenceError::MissingScheme),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rejects_references() {
        let r = OpReference::parse("acc", "op://v_eng01/it_cdx01/credential").unwrap();
        assert_eq!(r.canonical(), "op://v_eng01/it_cdx01/credential");
        assert_eq!(
            OpReference::parse("acc", "op://v/").unwrap_err(),
            OpReferenceError::EmptyComponent
        );
        assert_eq!(
            OpReference::parse("acc", "op://v/i").unwrap_err(),
            OpReferenceError::MissingField
        );
        assert_eq!(
            OpReference::parse("acc", "vault/item/field").unwrap_err(),
            OpReferenceError::MissingScheme
        );
    }
}
