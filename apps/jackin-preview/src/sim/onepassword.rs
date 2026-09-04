//! Simulated 1Password: accounts, vaults, items and fields with the
//! session states the real picker can meet. Secret material exists only
//! inside [`SimOnePassword::resolve_into`]'s closure; nothing else in the
//! preview can observe it.

use std::fmt;

use crate::domain::agent::Provider;
use crate::domain::onepassword::OpReference;

/// Opaque secret handle. Not `Clone`, not `Debug`; constructible only here.
pub struct Secret {
    bytes: Vec<u8>,
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Secret")
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Secret {
    /// Which provider family the material belongs to, and whether the
    /// provider would accept it. This is the only question a provider
    /// operation may ask; it never sees the bytes as text.
    pub fn classify(&self) -> SecretClass {
        let s = String::from_utf8_lossy(&self.bytes);
        if s.is_empty() {
            return SecretClass::Empty;
        }
        let (provider, tag) = match s.split_once(':') {
            Some((p, tag)) => (p, tag),
            None => return SecretClass::Unrecognised,
        };
        let provider = match provider {
            "anthropic" => Provider::Anthropic,
            "openai" => Provider::OpenAi,
            "xai" => Provider::XAi,
            "opencode" => Provider::OpenCode,
            _ => return SecretClass::Unrecognised,
        };
        let outcome = match tag.split('-').next().unwrap_or("") {
            "valid" => KeyOutcome::Valid,
            "rotated" => KeyOutcome::Rejected,
            "throttled" => KeyOutcome::RateLimited,
            "outage" => KeyOutcome::Unavailable,
            _ => KeyOutcome::Rejected,
        };
        SecretClass::Key { provider, outcome }
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        for b in self.bytes.iter_mut() {
            *b = 0;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOutcome {
    Valid,
    Rejected,
    RateLimited,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretClass {
    Empty,
    Unrecognised,
    Key {
        provider: Provider,
        outcome: KeyOutcome,
    },
}

/// Marker for closure results that cannot carry the secret out.
pub trait SecretFree {}
impl SecretFree for SecretClass {}
impl SecretFree for () {}
impl SecretFree for bool {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpSession {
    SignedIn,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpAccountState {
    Available,
    Locked,
    AuthorizationRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultAccess {
    ReadWrite,
    ReadOnly,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Concealed,
    Text,
    Url,
}

impl FieldKind {
    pub fn label(self) -> &'static str {
        match self {
            FieldKind::Concealed => "concealed",
            FieldKind::Text => "text",
            FieldKind::Url => "url",
        }
    }
}

#[derive(Clone)]
pub struct OpField {
    pub id: String,
    pub label: String,
    pub kind: FieldKind,
    /// Synthetic material tag (`openai:valid-cdx01`); never shown.
    material: Option<String>,
    /// Synthetic four-character tail for masked previews.
    pub tail: String,
}

impl fmt::Debug for OpField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpField")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("kind", &self.kind)
            .field("material", &"[redacted]")
            .field("tail", &self.tail)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct OpItem {
    pub id: String,
    pub title: String,
    pub category: &'static str,
    pub fields: Vec<OpField>,
}

#[derive(Debug, Clone)]
pub struct OpVault {
    pub id: String,
    pub name: String,
    pub access: VaultAccess,
    pub items: Vec<OpItem>,
}

#[derive(Debug, Clone)]
pub struct OpAccount {
    pub id: String,
    pub email: String,
    pub state: OpAccountState,
    pub vaults: Vec<OpVault>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    Locked,
    AuthorizationRequired { account: String },
    PermissionDenied { vault: String },
    MissingAccount { account: String },
    MissingVault { vault: String },
    MissingItem { item: String, vault: String },
    MissingField { field: String, item: String },
    EmptyMaterial { field: String },
    WrongFieldShape { field: String },
}

impl OpError {
    /// Operator-facing message (sentence case, colon introduces the reason).
    pub fn message(&self) -> String {
        match self {
            OpError::Locked => "1Password locked: unlock the app and retry".into(),
            OpError::AuthorizationRequired { account } => {
                format!("1Password authorization required: sign in to {account}")
            }
            OpError::PermissionDenied { vault } => {
                format!("1Password access denied: no permission for vault {vault}")
            }
            OpError::MissingAccount { account } => {
                format!("1Password account not found: {account}")
            }
            OpError::MissingVault { vault } => format!("1Password vault not found: {vault}"),
            OpError::MissingItem { item, vault } => {
                format!("1Password item not found: {item} in {vault}")
            }
            OpError::MissingField { field, item } => {
                format!("1Password field not found: {field} in {item}")
            }
            OpError::EmptyMaterial { field } => format!("API key required: field {field} is empty"),
            OpError::WrongFieldShape { field } => {
                format!("Field {field} is not a credential: choose a concealed field")
            }
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(self, OpError::Locked)
    }
}

#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    pub masked: String,
}

#[derive(Debug, Clone)]
pub struct SimOnePassword {
    pub session: OpSession,
    pub accounts: Vec<OpAccount>,
    /// Simulated latency for listing/validation, in virtual ms.
    pub latency_ms: i64,
}

fn field(id: &str, label: &str, kind: FieldKind, material: Option<&str>, tail: &str) -> OpField {
    OpField {
        id: id.into(),
        label: label.into(),
        kind,
        material: material.map(str::to_owned),
        tail: tail.into(),
    }
}

fn item(
    id: &str,
    title: &str,
    category: &'static str,
    fields: Vec<OpField>,
    _updated: i64,
) -> OpItem {
    OpItem {
        id: id.into(),
        title: title.into(),
        category,
        fields,
    }
}

impl SimOnePassword {
    /// The fixture directory shared by every scenario.
    pub fn fixture(epoch: i64) -> Self {
        let d = 86_400;
        let engineering = OpVault {
            id: "v_eng01".into(),
            name: "Engineering".into(),
            access: VaultAccess::ReadWrite,
            items: vec![
                item(
                    "it_cdx01",
                    "OpenAI · Codex Primary",
                    "API credential",
                    vec![
                        field(
                            "credential",
                            "credential",
                            FieldKind::Concealed,
                            Some("openai:valid-cdx01"),
                            "k7Qz",
                        ),
                        field("notes", "notes", FieldKind::Text, None, "----"),
                    ],
                    epoch - 12 * d,
                ),
                item(
                    "it_cdx02",
                    "OpenAI · Codex Experiments",
                    "API credential",
                    vec![field(
                        "credential",
                        "credential",
                        FieldKind::Concealed,
                        Some("openai:valid-cdx02"),
                        "m2Xa",
                    )],
                    epoch - 3 * d,
                ),
                item(
                    "it_grk01",
                    "xAI · Grok Team",
                    "API credential",
                    vec![
                        field(
                            "credential",
                            "credential",
                            FieldKind::Concealed,
                            Some("xai:valid-grk01"),
                            "Rt4v",
                        ),
                        field("endpoint", "endpoint", FieldKind::Url, None, "----"),
                    ],
                    epoch - 40 * d,
                ),
                item(
                    "it_ant01",
                    "Anthropic · Work",
                    "API credential",
                    vec![field(
                        "credential",
                        "credential",
                        FieldKind::Concealed,
                        Some("anthropic:valid-ant01"),
                        "3c9e",
                    )],
                    epoch - 7 * d,
                ),
                item(
                    "it_leg01",
                    "Legacy · Rotated key",
                    "API credential",
                    vec![field(
                        "credential",
                        "credential",
                        FieldKind::Concealed,
                        Some("xai:rotated-leg01"),
                        "0x1f",
                    )],
                    epoch - 300 * d,
                ),
                item(
                    "it_brk01",
                    "Broken · Empty credential",
                    "Password",
                    vec![field(
                        "password",
                        "password",
                        FieldKind::Concealed,
                        Some(""),
                        "----",
                    )],
                    epoch - 2 * d,
                ),
                item(
                    "it_brk02",
                    "Broken · Wrong shape",
                    "Secure note",
                    vec![field("notes", "notes", FieldKind::Text, None, "----")],
                    epoch - d,
                ),
                item(
                    "it_thr01",
                    "OpenAI · Throttled sandbox",
                    "API credential",
                    vec![field(
                        "credential",
                        "credential",
                        FieldKind::Concealed,
                        Some("openai:throttled-thr01"),
                        "q9Lp",
                    )],
                    epoch - 5 * d,
                ),
            ],
        };
        let shared = OpVault {
            id: "v_inf01".into(),
            name: "Shared Infra".into(),
            access: VaultAccess::Denied,
            items: vec![item(
                "it_dep01",
                "Prod · Deploy token",
                "API credential",
                vec![field(
                    "credential",
                    "credential",
                    FieldKind::Concealed,
                    Some("opencode:valid-dep01"),
                    "zz00",
                )],
                epoch - 20 * d,
            )],
        };
        let archive = OpVault {
            id: "v_arc01".into(),
            name: "Archive".into(),
            access: VaultAccess::ReadOnly,
            items: vec![],
        };
        let personal = OpVault {
            id: "v_per01".into(),
            name: "Personal".into(),
            access: VaultAccess::ReadWrite,
            items: vec![item(
                "it_ant02",
                "Anthropic · Personal API key",
                "API credential",
                vec![field(
                    "credential",
                    "credential",
                    FieldKind::Concealed,
                    Some("anthropic:valid-ant02"),
                    "8Hj2",
                )],
                epoch - 15 * d,
            )],
        };
        let team = OpVault {
            id: "v_team01".into(),
            name: "Team".into(),
            access: VaultAccess::ReadWrite,
            items: vec![item(
                "it_ocg01",
                "OpenCode · Go",
                "API credential",
                vec![field(
                    "credential",
                    "credential",
                    FieldKind::Concealed,
                    Some("opencode:valid-ocg01"),
                    "Wq7n",
                )],
                epoch - 9 * d,
            )],
        };
        Self {
            session: OpSession::SignedIn,
            accounts: vec![
                OpAccount {
                    id: "chainargos.1password.com".into(),
                    email: "alexey@chainargos.com".into(),
                    state: OpAccountState::Available,
                    vaults: vec![engineering, shared, archive],
                },
                OpAccount {
                    id: "my.1password.com".into(),
                    email: "alexey@donbeave.dev".into(),
                    state: OpAccountState::Locked,
                    vaults: vec![personal],
                },
                OpAccount {
                    id: "acme.1password.com".into(),
                    email: "alexey@acme-fixture.example".into(),
                    state: OpAccountState::AuthorizationRequired,
                    vaults: vec![team],
                },
            ],
            latency_ms: 260,
        }
    }

    fn gate(&self) -> Result<(), OpError> {
        match self.session {
            OpSession::Locked => Err(OpError::Locked),
            OpSession::SignedIn => Ok(()),
        }
    }

    fn account(&self, id: &str) -> Result<&OpAccount, OpError> {
        self.gate()?;
        let a =
            self.accounts
                .iter()
                .find(|a| a.id == id)
                .ok_or_else(|| OpError::MissingAccount {
                    account: id.to_owned(),
                })?;
        match a.state {
            OpAccountState::Available => Ok(a),
            OpAccountState::Locked => Err(OpError::Locked),
            OpAccountState::AuthorizationRequired => Err(OpError::AuthorizationRequired {
                account: a.id.clone(),
            }),
        }
    }

    pub fn list_accounts(&self) -> Result<Vec<&OpAccount>, OpError> {
        self.gate()?;
        Ok(self.accounts.iter().collect())
    }

    pub fn list_vaults(&self, account: &str) -> Result<Vec<&OpVault>, OpError> {
        Ok(self.account(account)?.vaults.iter().collect())
    }

    fn vault(&self, account: &str, vault: &str) -> Result<&OpVault, OpError> {
        let a = self.account(account)?;
        let v = a
            .vaults
            .iter()
            .find(|v| v.id == vault || v.name == vault)
            .ok_or_else(|| OpError::MissingVault {
                vault: vault.to_owned(),
            })?;
        if v.access == VaultAccess::Denied {
            return Err(OpError::PermissionDenied {
                vault: v.name.clone(),
            });
        }
        Ok(v)
    }

    pub fn list_items(&self, account: &str, vault: &str) -> Result<Vec<&OpItem>, OpError> {
        Ok(self.vault(account, vault)?.items.iter().collect())
    }

    fn item(&self, account: &str, vault: &str, item: &str) -> Result<(&OpVault, &OpItem), OpError> {
        let v = self.vault(account, vault)?;
        let it = v
            .items
            .iter()
            .find(|i| i.id == item || i.title == item)
            .ok_or_else(|| OpError::MissingItem {
                item: item.to_owned(),
                vault: v.name.clone(),
            })?;
        Ok((v, it))
    }

    pub fn list_fields(
        &self,
        account: &str,
        vault: &str,
        item: &str,
    ) -> Result<Vec<&OpField>, OpError> {
        Ok(self.item(account, vault, item)?.1.fields.iter().collect())
    }

    /// Full reference (with names) for chosen ids.
    pub fn reference(
        &self,
        account: &str,
        vault: &str,
        item: &str,
        field: &str,
    ) -> Result<OpReference, OpError> {
        let (v, it) = self.item(account, vault, item)?;
        let f = it
            .fields
            .iter()
            .find(|f| f.id == field || f.label == field)
            .ok_or_else(|| OpError::MissingField {
                field: field.to_owned(),
                item: it.title.clone(),
            })?;
        Ok(OpReference {
            account: account.to_owned(),
            vault_id: v.id.clone(),
            vault_name: v.name.clone(),
            item_id: it.id.clone(),
            item_title: it.title.clone(),
            section: None,
            field_id: f.id.clone(),
            field_label: f.label.clone(),
        })
    }

    /// Non-secret metadata plus a masked preview.
    pub fn describe(&self, r: &OpReference) -> Result<FieldDescriptor, OpError> {
        let (_, it) = self.item(&r.account, &r.vault_id, &r.item_id)?;
        let f = it
            .fields
            .iter()
            .find(|f| f.id == r.field_id)
            .ok_or_else(|| OpError::MissingField {
                field: r.field_id.clone(),
                item: it.title.clone(),
            })?;
        if f.kind != FieldKind::Concealed {
            return Err(OpError::WrongFieldShape {
                field: f.label.clone(),
            });
        }
        if f.material.as_deref().is_some_and(str::is_empty) {
            return Err(OpError::EmptyMaterial {
                field: f.label.clone(),
            });
        }
        Ok(FieldDescriptor {
            masked: crate::domain::account::masked(&f.tail),
        })
    }

    /// The only path to secret bytes: the closure is the transient provider
    /// operation and its result cannot carry the secret.
    pub fn resolve_into<R: SecretFree>(
        &self,
        r: &OpReference,
        op: impl FnOnce(&Secret) -> R,
    ) -> Result<R, OpError> {
        let desc = self.describe(r)?;
        let (_, it) = self.item(&r.account, &r.vault_id, &r.item_id)?;
        let Some(f) = it.fields.iter().find(|f| f.id == r.field_id) else {
            return Err(OpError::MissingField {
                field: r.field_id.clone(),
                item: it.title.clone(),
            });
        };
        let secret = Secret {
            bytes: f.material.clone().unwrap_or_default().into_bytes(),
        };
        let _ = desc;
        Ok(op(&secret))
    }

    /// Endpoint URL stored beside a credential (Grok fixture only).
    pub fn endpoint_of(&self, r: &OpReference) -> Option<String> {
        let (_, it) = self.item(&r.account, &r.vault_id, &r.item_id).ok()?;
        it.fields
            .iter()
            .find(|f| f.kind == FieldKind::Url)
            .map(|_| "https://api.x.ai/v1".to_owned())
    }
}

/// Classify plain-text key material typed by the operator. Fixture rule:
/// keys that contain `valid` are accepted; `rotated` is rejected;
/// `throttled` is rate limited; anything else is rejected.
pub fn classify_plain(provider: Provider, value: &str) -> SecretClass {
    if value.trim().is_empty() {
        return SecretClass::Empty;
    }
    let outcome = if value.contains("valid") {
        KeyOutcome::Valid
    } else if value.contains("throttled") {
        KeyOutcome::RateLimited
    } else if value.contains("outage") {
        KeyOutcome::Unavailable
    } else {
        KeyOutcome::Rejected
    };
    // provider prefix classes let a mismatch be detected
    let detected = if value.starts_with("sk-ant-") {
        Provider::Anthropic
    } else if value.starts_with("sk-") {
        Provider::OpenAi
    } else if value.starts_with("xai-") {
        Provider::XAi
    } else if value.starts_with("oc_") {
        Provider::OpenCode
    } else {
        provider
    };
    SecretClass::Key {
        provider: detected,
        outcome,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_only_inside_the_closure() {
        let op = SimOnePassword::fixture(0);
        let r = op
            .reference(
                "chainargos.1password.com",
                "v_eng01",
                "it_cdx01",
                "credential",
            )
            .unwrap();
        assert_eq!(r.canonical(), "op://v_eng01/it_cdx01/credential");
        assert_eq!(
            r.display_path(),
            "Engineering › OpenAI · Codex Primary › credential"
        );
        let class = op.resolve_into(&r, |s| s.classify()).unwrap();
        assert_eq!(
            class,
            SecretClass::Key {
                provider: Provider::OpenAi,
                outcome: KeyOutcome::Valid
            }
        );
        assert_eq!(op.describe(&r).unwrap().masked, "••••••••…k7Qz");
        assert_eq!(
            op.list_vaults("my.1password.com").unwrap_err(),
            OpError::Locked
        );
        assert!(matches!(
            op.list_items("chainargos.1password.com", "v_inf01")
                .unwrap_err(),
            OpError::PermissionDenied { .. }
        ));
        let bad = op
            .reference("chainargos.1password.com", "v_eng01", "it_brk02", "notes")
            .unwrap();
        assert!(matches!(
            op.describe(&bad).unwrap_err(),
            OpError::WrongFieldShape { .. }
        ));
        let empty = op
            .reference(
                "chainargos.1password.com",
                "v_eng01",
                "it_brk01",
                "password",
            )
            .unwrap();
        assert!(matches!(
            op.describe(&empty).unwrap_err(),
            OpError::EmptyMaterial { .. }
        ));
    }
}
