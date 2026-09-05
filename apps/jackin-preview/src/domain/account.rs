//! Provider accounts and API-key profiles: identity, credential source
//! metadata (never secret material), lifecycle, validation, usage.

use super::agent::{Agent, Provider, UsageSurface};
use super::onepassword::OpReference;
use super::usage::{AccountUsage, Freshness};

pub(crate) type AccountId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccountOrigin {
    /// Created in the Center; editable, removable.
    Registered,
    /// Found by simulated host discovery; read-only.
    Discovered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CredentialSource {
    OnePassword(OpReference),
    LocalFolder {
        path: String,
        detected: DetectedKind,
    },
    PlainApiKey {
        /// 8 hex chars, deterministic from the fixture handle; never derived
        /// from the secret bytes.
        fingerprint: String,
        /// Exactly four synthetic characters shown as `…k7Qz`.
        tail: String,
    },
    HostEnv {
        var: String,
        detected: DetectedKind,
    },
}

impl CredentialSource {
    pub(crate) fn origin_label(&self) -> &'static str {
        match self {
            CredentialSource::OnePassword(_) => "1Password",
            CredentialSource::LocalFolder { .. } => "Local folder",
            CredentialSource::PlainApiKey { .. } => "API key",
            CredentialSource::HostEnv { .. } => "Host env",
        }
    }

    /// Non-secret detail: reference path, folder path, or key fingerprint.
    pub(crate) fn safe_detail(&self) -> String {
        match self {
            CredentialSource::OnePassword(r) => r.display_path(),
            CredentialSource::LocalFolder { path, .. } => path.clone(),
            CredentialSource::PlainApiKey { fingerprint, tail } => {
                format!("…{tail} · fingerprint {fingerprint}")
            }
            CredentialSource::HostEnv { var, .. } => format!("${var}"),
        }
    }
}

/// `••••••••…k7Qz`
pub(crate) fn masked(tail: &str) -> String {
    format!("••••••••…{tail}")
}

/// Deterministic 8-hex fingerprint of a fixture key (not of secret bytes
/// in the real product; here of the synthetic value so duplicates match).
pub(crate) fn fingerprint(value: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in value.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{:08x}", (h >> 32) as u32 ^ h as u32)
}

/// Last four characters of a typed key become the synthetic tail.
pub(crate) fn tail_of(value: &str) -> String {
    let n = value.chars().count();
    value.chars().skip(n.saturating_sub(4)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetectedKind {
    ClaudeOAuthProfile,
    ClaudeApiKeyEnv,
    CodexAuthJson,
    GrokAuthJson,
    OpenCodeGoAuthJson,
    AmpSecrets,
    ZaiApiKeyEnv,
    KimiApiKeyEnv,
    MinimaxTokenEnv,
    Unknown,
}

impl DetectedKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            DetectedKind::ClaudeOAuthProfile => "Claude OAuth profile",
            DetectedKind::ClaudeApiKeyEnv => "Claude API key env",
            DetectedKind::CodexAuthJson => "Codex auth.json",
            DetectedKind::GrokAuthJson => "Grok auth.json",
            DetectedKind::OpenCodeGoAuthJson => "OpenCode Go auth.json",
            DetectedKind::AmpSecrets => "Amp secrets.json",
            DetectedKind::ZaiApiKeyEnv => "Z.AI API key env",
            DetectedKind::KimiApiKeyEnv => "Kimi API key env",
            DetectedKind::MinimaxTokenEnv => "MiniMax token env",
            DetectedKind::Unknown => "unrecognised",
        }
    }

    pub(crate) fn provider(self) -> Option<Provider> {
        match self {
            DetectedKind::ClaudeOAuthProfile | DetectedKind::ClaudeApiKeyEnv => {
                Some(Provider::Anthropic)
            }
            DetectedKind::CodexAuthJson => Some(Provider::OpenAi),
            DetectedKind::GrokAuthJson => Some(Provider::XAi),
            DetectedKind::OpenCodeGoAuthJson => Some(Provider::OpenCode),
            DetectedKind::AmpSecrets => Some(Provider::Amp),
            DetectedKind::ZaiApiKeyEnv => Some(Provider::Zai),
            DetectedKind::KimiApiKeyEnv => Some(Provider::Moonshot),
            DetectedKind::MinimaxTokenEnv => Some(Provider::MiniMax),
            DetectedKind::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IdentitySubject {
    Handle(String),
}

impl IdentitySubject {
    pub(crate) fn label(&self) -> &str {
        match self {
            IdentitySubject::Handle(s) => s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct AccountIdentity {
    pub subject: Option<IdentitySubject>,
    pub plan: Option<String>,
}

impl AccountIdentity {
    pub(crate) fn label(&self) -> String {
        match &self.subject {
            Some(s) => s.label().to_owned(),
            None => "unresolved identity".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Provenance {
    ConfiguredSource,
    LiveHost,
}

impl Provenance {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Provenance::ConfiguredSource => "configured source",
            Provenance::LiveHost => "live host",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Confidence {
    Authoritative,
    Estimated,
    PresenceOnly,
    None,
}

impl Confidence {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Confidence::Authoritative => "authoritative",
            Confidence::Estimated => "estimated",
            Confidence::PresenceOnly => "presence only",
            Confidence::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lifecycle {
    Available,
    AgentUninitialized,
    NeedsLogin,
    NeedsSecret,
    Unsupported,
    Unavailable,
    Error,
}

impl Lifecycle {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Lifecycle::Available => "available",
            Lifecycle::AgentUninitialized => "agent uninitialized",
            Lifecycle::NeedsLogin => "needs login",
            Lifecycle::NeedsSecret => "needs secret",
            Lifecycle::Unsupported => "unsupported",
            Lifecycle::Unavailable => "unavailable",
            Lifecycle::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ValidationLevel {
    MaterialDiscovered,
    IdentityAuthenticated,
    QuotaReadable,
}

impl ValidationLevel {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ValidationLevel::MaterialDiscovered => "material discovered",
            ValidationLevel::IdentityAuthenticated => "identity authenticated",
            ValidationLevel::QuotaReadable => "quota readable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValidationState {
    NeverValidated,
    Validating { started_tick: u64 },
    Valid(ValidationLevel),
    Invalid(RecoverableIssue),
}

impl ValidationState {
    pub(crate) fn label(&self) -> String {
        match self {
            ValidationState::NeverValidated => "never validated".into(),
            ValidationState::Validating { .. } => "validating…".into(),
            ValidationState::Valid(l) => l.label().to_owned(),
            ValidationState::Invalid(i) => i.message.clone(),
        }
    }

    pub(crate) fn level(&self) -> Option<ValidationLevel> {
        match self {
            ValidationState::Valid(l) => Some(*l),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum IssueCode {
    QuotaUnsupported,
    FolderMissing,
    FolderUnreadable,
    FolderWrongProvider,
    CredentialFileMissing,
    CredentialMalformed,
    ApiKeyEmpty,
    ApiKeyInvalid,
    OpLocked,
    OpAuthorizationRequired,
    OpPermissionDenied,
    OpItemMissing,
    OpFieldMissing,
    OpProviderMismatch,
    Unauthorized,
    RateLimited,
    ProviderUnavailable,
    Stale,
    IdentityUnresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Recoverability {
    Retryable,
    ActionRequired,
    Unsupported,
}

impl Recoverability {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Recoverability::Retryable => "retryable",
            Recoverability::ActionRequired => "action required",
            Recoverability::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoverableIssue {
    pub code: IssueCode,
    pub message: String,
    pub detail: Option<String>,
    pub recoverability: Recoverability,
    pub retry_secs: Option<i64>,
}

impl RecoverableIssue {
    pub(crate) fn new(code: IssueCode, message: impl Into<String>, rec: Recoverability) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            recoverability: rec,
            retry_secs: None,
        }
    }
    pub(crate) fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
    pub(crate) fn retry(mut self, secs: i64) -> Self {
        self.retry_secs = Some(secs);
        self
    }

    /// Issues that are status, not errors.
    pub(crate) fn is_informational(&self) -> bool {
        matches!(self.code, IssueCode::Stale | IssueCode::IdentityUnresolved)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    pub label: String,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Account {
    pub id: AccountId,
    pub origin: AccountOrigin,
    pub display_name: String,
    pub agent: Option<Agent>,
    pub provider: Provider,
    pub surface: UsageSurface,
    pub source: CredentialSource,
    pub identity: AccountIdentity,
    pub provenance: Vec<Provenance>,
    pub confidence: Confidence,
    pub lifecycle: Lifecycle,
    pub purpose: Option<String>,
    pub enabled: bool,
    pub default_for_provider: bool,
    pub validation: ValidationState,
    pub last_refresh_secs: Option<i64>,
    pub issue: Option<RecoverableIssue>,
    /// Only Grok fixtures carry one; the constructor refuses others.
    pub endpoint: Option<Endpoint>,
    pub usage: AccountUsage,
}

impl Account {
    pub(crate) fn registered(
        id: &str,
        name: &str,
        provider: Provider,
        source: CredentialSource,
    ) -> Self {
        Self {
            id: id.to_owned(),
            origin: AccountOrigin::Registered,
            display_name: name.to_owned(),
            agent: provider.agent(),
            provider,
            surface: provider.usage_surface(),
            source,
            identity: AccountIdentity::default(),
            provenance: vec![Provenance::ConfiguredSource],
            confidence: Confidence::None,
            lifecycle: Lifecycle::AgentUninitialized,
            purpose: None,
            enabled: true,
            default_for_provider: false,
            validation: ValidationState::NeverValidated,
            last_refresh_secs: None,
            issue: None,
            endpoint: None,
            usage: AccountUsage::none(),
        }
    }

    pub(crate) fn discovered(
        id: &str,
        name: &str,
        provider: Provider,
        source: CredentialSource,
    ) -> Self {
        let mut a = Self::registered(id, name, provider, source);
        a.origin = AccountOrigin::Discovered;
        a.provenance = vec![Provenance::LiveHost];
        a
    }

    pub(crate) fn with_endpoint(mut self, label: &str, host: &str) -> Self {
        if self.provider.supports_endpoint() {
            self.endpoint = Some(Endpoint {
                label: label.to_owned(),
                host: host.to_owned(),
            });
        }
        self
    }

    pub(crate) fn mutations_allowed(&self) -> bool {
        self.origin == AccountOrigin::Registered
    }

    /// `Codex · Primary`
    pub(crate) fn title(&self) -> String {
        format!("{} · {}", self.surface.surface_name(), self.display_name)
    }

    /// Row status word: exhausted > error > needs … > warning > stale > refreshing > ok.
    pub(crate) fn status_word(&self) -> &'static str {
        if !self.enabled {
            return "disabled";
        }
        match self.usage.freshness.phase {
            Freshness::Refreshing => return "refreshing",
            Freshness::Failed if self.issue.is_some() => {}
            _ => {}
        }
        match self.lifecycle {
            Lifecycle::NeedsLogin => return "needs login",
            Lifecycle::NeedsSecret => return "needs secret",
            Lifecycle::Unsupported => return "unsupported",
            Lifecycle::Unavailable => return "unavailable",
            Lifecycle::Error => return "error",
            Lifecycle::AgentUninitialized => return "uninitialized",
            Lifecycle::Available => {}
        }
        match self.usage.worst_status() {
            Some(super::usage::QuotaStatus::Exhausted) => "exhausted",
            Some(super::usage::QuotaStatus::Warning) => "warning",
            _ => match self.usage.freshness.phase {
                Freshness::Stale => "stale",
                Freshness::Failed => "failed",
                _ => "ok",
            },
        }
    }

    pub(crate) fn is_error_state(&self) -> bool {
        matches!(
            self.lifecycle,
            Lifecycle::Error
                | Lifecycle::NeedsLogin
                | Lifecycle::NeedsSecret
                | Lifecycle::Unavailable
        ) || self.usage.freshness.phase == Freshness::Failed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DuplicateProbe {
    Folder {
        provider: Provider,
        path: String,
    },
    OpReference {
        canonical: String,
        account: String,
    },
    KeyFingerprint {
        provider: Provider,
        fingerprint: String,
    },
    Identity {
        surface: UsageSurface,
        subject: IdentitySubject,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AccountRegistry {
    pub accounts: Vec<Account>,
    pub revision: u64,
}

impl AccountRegistry {
    pub(crate) fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub(crate) fn get_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    pub(crate) fn by_provider(&self, p: Provider) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(move |a| a.provider == p)
    }

    pub(crate) fn default_for(&self, p: Provider) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|a| a.provider == p && a.default_for_provider && a.enabled)
    }

    pub(crate) fn discovered_current(&self, p: Provider) -> Option<&Account> {
        self.accounts.iter().find(|a| {
            a.provider == p
                && a.origin == AccountOrigin::Discovered
                && a.enabled
                && a.lifecycle == Lifecycle::Available
        })
    }

    pub(crate) fn find_duplicate(&self, probe: &DuplicateProbe) -> Option<&Account> {
        self.accounts.iter().find(|a| match probe {
            DuplicateProbe::Folder { provider, path } => {
                a.provider == *provider
                    && matches!(&a.source, CredentialSource::LocalFolder { path: p, .. } if p == path)
            }
            DuplicateProbe::OpReference { canonical, account } => {
                matches!(&a.source, CredentialSource::OnePassword(r) if r.canonical() == *canonical && r.account == *account)
            }
            DuplicateProbe::KeyFingerprint {
                provider,
                fingerprint,
            } => {
                a.provider == *provider
                    && matches!(&a.source, CredentialSource::PlainApiKey { fingerprint: f, .. } if f == fingerprint)
            }
            DuplicateProbe::Identity { surface, subject } => {
                a.surface == *surface && a.identity.subject.as_ref() == Some(subject)
            }
        })
    }

    /// Soft warning: same display name for the same provider.
    pub(crate) fn name_taken(&self, provider: Provider, name: &str, except: Option<&str>) -> bool {
        self.accounts.iter().any(|a| {
            a.provider == provider
                && a.display_name.eq_ignore_ascii_case(name)
                && Some(a.id.as_str()) != except
        })
    }

    /// One default per provider: clears the sibling.
    pub(crate) fn set_default(&mut self, id: &str) -> Result<(), String> {
        let Some(target) = self.get(id) else {
            return Err("account not found".into());
        };
        if !target.enabled {
            return Err("a disabled account cannot be the provider default".into());
        }
        let p = target.provider;
        for a in &mut self.accounts {
            if a.provider == p {
                a.default_for_provider = a.id == id;
            }
        }
        self.revision += 1;
        Ok(())
    }

    pub(crate) fn remove(&mut self, id: &str) -> Option<Account> {
        let i = self.accounts.iter().position(|a| a.id == id)?;
        self.revision += 1;
        Some(self.accounts.remove(i))
    }

    pub(crate) fn insert(&mut self, account: Account) {
        self.accounts.push(account);
        self.revision += 1;
    }

    /// Registry order: surface registry order, defaults first, then name.
    pub(crate) fn sorted(&self) -> Vec<&Account> {
        let mut v: Vec<&Account> = self.accounts.iter().collect();
        v.sort_by(|a, b| {
            a.surface
                .cmp(&b.surface)
                .then(b.default_for_provider.cmp(&a.default_for_provider))
                .then(a.display_name.cmp(&b.display_name))
        });
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking_helpers() {
        assert_eq!(masked("k7Qz"), "••••••••…k7Qz");
        assert_eq!(fingerprint("x").len(), 8);
        assert_eq!(tail_of("sk-ant-k7Qz"), "k7Qz");
    }

    #[test]
    fn one_default_per_provider() {
        let mut r = AccountRegistry::default();
        r.insert(Account::registered(
            "a",
            "Personal",
            Provider::Anthropic,
            CredentialSource::PlainApiKey {
                fingerprint: "1".into(),
                tail: "aaaa".into(),
            },
        ));
        r.insert(Account::registered(
            "b",
            "Work",
            Provider::Anthropic,
            CredentialSource::PlainApiKey {
                fingerprint: "2".into(),
                tail: "bbbb".into(),
            },
        ));
        r.set_default("a").unwrap();
        r.set_default("b").unwrap();
        assert!(!r.get("a").unwrap().default_for_provider);
        assert!(r.get("b").unwrap().default_for_provider);
        assert!(r.name_taken(Provider::Anthropic, "work", None));
        assert!(
            r.find_duplicate(&DuplicateProbe::KeyFingerprint {
                provider: Provider::Anthropic,
                fingerprint: "2".into()
            })
            .is_some()
        );
        let mut c = Account::registered(
            "c",
            "x",
            Provider::OpenCode,
            CredentialSource::PlainApiKey {
                fingerprint: "3".into(),
                tail: "cccc".into(),
            },
        )
        .with_endpoint("custom", "example");
        assert!(c.endpoint.is_none(), "endpoint only for Grok");
        c.enabled = false;
        r.insert(c);
        assert!(r.set_default("c").is_err());
    }
}
