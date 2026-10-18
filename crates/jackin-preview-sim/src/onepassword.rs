#![doc = r" Simulated 1Password: accounts, vaults, items and fields with the"]
#![doc = r" session states the real picker can meet. Secret material exists only"]
#![doc = r" inside the `SimOnePassword::resolve_into` closure; nothing else in the"]
#![doc = r" preview can observe it."]

use std::fmt;

use jackin_preview_domain::agent::Provider;
use jackin_preview_domain::onepassword::OpReference;

#[doc = r" Opaque secret handle whose debug output redacts its material."]
pub struct Secret {
    bytes: Vec<u8>,
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Secret")
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Secret {
    #[doc = r" Which provider family the material belongs to, and whether the"]
    #[doc = r" provider would accept it. This is the only question a provider"]
    #[doc = r" operation may ask; it never sees the bytes as text."]
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

#[doc = r" Result category for a synthetic provider key."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOutcome {
    #[doc = r" The key is accepted by the provider."]
    Valid,
    #[doc = r" The key is rejected or rotated."]
    Rejected,
    #[doc = r" The provider throttled the key."]
    RateLimited,
    #[doc = r" The provider is unavailable."]
    Unavailable,
}

#[doc = r" Classification of material held by a [`Secret`]."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretClass {
    #[doc = r" No material was present."]
    Empty,
    #[doc = r" The material did not match a recognized fixture shape."]
    Unrecognised,
    #[doc = r" Material tagged for a provider and outcome."]
    Key {
        #[doc = r" Provider family encoded by the fixture."]
        provider: Provider,
        #[doc = r" Synthetic validation outcome."]
        outcome: KeyOutcome,
    },
}

#[doc = r" Marker for closure results that cannot carry the secret out."]
pub trait SecretFree {}
impl SecretFree for SecretClass {}
impl SecretFree for () {}
impl SecretFree for bool {}

#[doc = r" Lock state of the simulated 1Password session."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpSession {
    #[doc = r" The session can read available accounts."]
    SignedIn,
    #[doc = r" Every operation is rejected as locked."]
    Locked,
}

#[doc = r" Availability state of one simulated 1Password account."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpAccountState {
    #[doc = r" The account can be queried."]
    Available,
    #[doc = r" The account requires an unlock."]
    Locked,
    #[doc = r" The account requires authorization."]
    AuthorizationRequired,
}

#[doc = r" Access level of a simulated vault."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultAccess {
    #[doc = r" Items can be read and changed by the fixture."]
    ReadWrite,
    #[doc = r" Items can be read but not changed."]
    ReadOnly,
    #[doc = r" The vault is visible but cannot be read."]
    Denied,
}

#[doc = r" Shape of one simulated 1Password field."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    #[doc = r" Credential-like concealed value."]
    Concealed,
    #[doc = r" Ordinary text value."]
    Text,
    #[doc = r" URL value stored beside a credential."]
    Url,
}

impl FieldKind {
    #[doc = r" Stable lower-case label for a picker row."]
    pub fn label(self) -> &'static str {
        match self {
            FieldKind::Concealed => "concealed",
            FieldKind::Text => "text",
            FieldKind::Url => "url",
        }
    }
}

#[doc = r" Item field metadata with an optional private fixture material tag."]
#[derive(Clone)]
pub struct OpField {
    #[doc = r" Stable field identifier."]
    pub id: String,
    #[doc = r" Operator-facing field label."]
    pub label: String,
    #[doc = r" Field value shape."]
    pub kind: FieldKind,
    #[doc = r" Synthetic material tag (`openai:valid-cdx01`); never shown."]
    material: Option<String>,
    #[doc = r" Synthetic four-character tail for masked previews."]
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

#[doc = r" Simulated 1Password item metadata."]
#[derive(Debug, Clone)]
pub struct OpItem {
    #[doc = r" Stable item identifier."]
    pub id: String,
    #[doc = r" Operator-facing item title."]
    pub title: String,
    #[doc = r" Fixture category label."]
    pub category: &'static str,
    #[doc = r" Fields contained by the item."]
    pub fields: Vec<OpField>,
}

#[doc = r" Simulated 1Password vault metadata and items."]
#[derive(Debug, Clone)]
pub struct OpVault {
    #[doc = r" Stable vault identifier."]
    pub id: String,
    #[doc = r" Operator-facing vault name."]
    pub name: String,
    #[doc = r" Access state for this vault."]
    pub access: VaultAccess,
    #[doc = r" Items contained by the vault."]
    pub items: Vec<OpItem>,
}

#[doc = r" Simulated 1Password account and its vaults."]
#[derive(Debug, Clone)]
pub struct OpAccount {
    #[doc = r" Stable account identifier."]
    pub id: String,
    #[doc = r" Account email shown in the picker."]
    pub email: String,
    #[doc = r" Account availability state."]
    pub state: OpAccountState,
    #[doc = r" Vaults available under the account."]
    pub vaults: Vec<OpVault>,
}

#[doc = r" Errors exposed by simulated 1Password lookups."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    #[doc = r" The overall session is locked."]
    Locked,
    #[doc = r" The account needs an authorization flow."]
    AuthorizationRequired {
        #[doc = r" Account requiring authorization."]
        account: String,
    },
    #[doc = r" The selected vault cannot be read."]
    PermissionDenied {
        #[doc = r" Vault whose contents cannot be read."]
        vault: String,
    },
    #[doc = r" No account matched the requested identifier."]
    MissingAccount {
        #[doc = r" Requested account identifier."]
        account: String,
    },
    #[doc = r" No vault matched the requested identifier."]
    MissingVault {
        #[doc = r" Requested vault identifier."]
        vault: String,
    },
    #[doc = r" No item matched the requested identifier."]
    MissingItem {
        #[doc = r" Requested item identifier."]
        item: String,
        #[doc = r" Vault searched for the item."]
        vault: String,
    },
    #[doc = r" No field matched the requested identifier."]
    MissingField {
        #[doc = r" Requested field identifier."]
        field: String,
        #[doc = r" Item searched for the field."]
        item: String,
    },
    #[doc = r" The selected concealed field has no material."]
    EmptyMaterial {
        #[doc = r" Concealed field without material."]
        field: String,
    },
    #[doc = r" The selected field is not a concealed credential field."]
    WrongFieldShape {
        #[doc = r" Field selected as a credential."]
        field: String,
    },
}

impl OpError {
    #[doc = r" Operator-facing message (sentence case, colon introduces the reason)."]
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

    #[doc = r" Whether retrying the same operation may succeed after an unlock."]
    pub fn retryable(&self) -> bool {
        matches!(self, OpError::Locked)
    }
}

#[doc = r" Non-secret field metadata returned by [`SimOnePassword::describe`]."]
#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    #[doc = r" Masked tail suitable for display."]
    pub masked: String,
}

#[doc = r" Deterministic in-memory 1Password service."]
#[derive(Debug, Clone)]
pub struct SimOnePassword {
    #[doc = r" Current session state."]
    pub session: OpSession,
    #[doc = r" Simulated accounts and their vaults."]
    pub accounts: Vec<OpAccount>,
    #[doc = r" Simulated latency for listing/validation, in virtual ms."]
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
    #[doc = r" The fixture directory shared by every scenario."]
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

    #[doc = r" List accounts visible in the current session."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns [`OpError::Locked`] when the session is locked."]
    pub fn list_accounts(&self) -> Result<Vec<&OpAccount>, OpError> {
        self.gate()?;
        Ok(self.accounts.iter().collect())
    }

    #[doc = r" List vaults for an account."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns an account or session error when the account cannot be read."]
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

    #[doc = r" List items in a readable vault."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns an account, vault, session, or permission error when the vault"]
    #[doc = r" cannot be read."]
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

    #[doc = r" List fields in a simulated item."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns an account, vault, item, session, or permission error when the"]
    #[doc = r" item cannot be read."]
    pub fn list_fields(
        &self,
        account: &str,
        vault: &str,
        item: &str,
    ) -> Result<Vec<&OpField>, OpError> {
        Ok(self.item(account, vault, item)?.1.fields.iter().collect())
    }

    #[doc = r" Full reference (with names) for chosen ids."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns an account, vault, item, field, session, or permission error"]
    #[doc = r" when a referenced object is unavailable."]
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

    #[doc = r" Non-secret metadata plus a masked preview."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns an account, vault, item, field, shape, empty-material, session,"]
    #[doc = r" or permission error when the reference cannot be described."]
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
            masked: jackin_preview_domain::account::masked(&f.tail),
        })
    }

    #[doc = r" The only path to secret bytes: the closure is the transient provider"]
    #[doc = r" operation and its result cannot carry the secret."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns the same lookup, shape, and material errors as"]
    #[doc = r" [`Self::describe`]."]
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

    #[doc = r" Endpoint URL stored beside a credential (Grok fixture only)."]
    pub fn endpoint_of(&self, r: &OpReference) -> Option<String> {
        let (_, it) = self.item(&r.account, &r.vault_id, &r.item_id).ok()?;
        it.fields
            .iter()
            .find(|f| f.kind == FieldKind::Url)
            .map(|_| "https://api.x.ai/v1".to_owned())
    }
}

#[doc = r" Classify plain-text key material typed by the operator. Fixture rule:"]
#[doc = r" keys that contain `valid` are accepted; `rotated` is rejected;"]
#[doc = r" `throttled` is rate limited; anything else is rejected."]
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
    fn secret_debug_redacts_material() {
        let secret = Secret {
            bytes: vec![1, 2, 3, 4],
        };
        assert_eq!(format!("{secret:?}"), r#"Secret { bytes: "[redacted]" }"#);
    }

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
