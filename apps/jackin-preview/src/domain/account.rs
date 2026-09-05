//! Provider accounts and API-key profiles: identity, credential source
//! metadata (never secret material), lifecycle, validation, usage.

use super::agent::{Agent, Provider, UsageSurface};
use super::onepassword::OpReference;
use super::usage::{AccountUsage, Freshness};

/// Stable identifier for an account in a registry or workspace policy.
pub type AccountId = String;

/// Indicates whether an account was configured or discovered on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountOrigin {
    /// Created in the Center; editable, removable.
    Registered,
    /// Found by simulated host discovery; read-only.
    Discovered,
}

/// Describes where an account obtains its credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    /// Reference to credential metadata stored in 1Password.
    OnePassword(OpReference),
    /// Credential or profile material found in a local folder.
    LocalFolder {
        /// Folder path containing the detected material.
        path: String,
        /// Kind of material detected in the folder.
        detected: DetectedKind,
    },
    /// Synthetic metadata for a plain API key; secret bytes are not stored.
    PlainApiKey {
        /// 8 hex chars, deterministic from the fixture handle; never derived
        /// from the secret bytes.
        fingerprint: String,
        /// Exactly four synthetic characters shown as `…k7Qz`.
        tail: String,
    },
    /// Credential exposed through a host environment variable.
    HostEnv {
        /// Name of the environment variable.
        var: String,
        /// Kind of material detected from the variable.
        detected: DetectedKind,
    },
}

impl CredentialSource {
    /// Return the operator-facing label for this credential source.
    pub fn origin_label(&self) -> &'static str {
        match self {
            CredentialSource::OnePassword(_) => "1Password",
            CredentialSource::LocalFolder { .. } => "Local folder",
            CredentialSource::PlainApiKey { .. } => "API key",
            CredentialSource::HostEnv { .. } => "Host env",
        }
    }

    /// Non-secret detail: reference path, folder path, or key fingerprint.
    pub fn safe_detail(&self) -> String {
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
pub fn masked(tail: &str) -> String {
    format!("••••••••…{tail}")
}

/// Deterministic 8-hex fingerprint of a fixture key (not of secret bytes
/// in the real product; here of the synthetic value so duplicates match).
pub fn fingerprint(value: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in value.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{:08x}", (h >> 32) as u32 ^ h as u32)
}

/// Last four characters of a typed key become the synthetic tail.
pub fn tail_of(value: &str) -> String {
    let n = value.chars().count();
    value.chars().skip(n.saturating_sub(4)).collect()
}

/// Kind of credential or profile material detected by discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedKind {
    /// Claude OAuth profile data.
    ClaudeOAuthProfile,
    /// Claude API key environment variable.
    ClaudeApiKeyEnv,
    /// Codex `auth.json` data.
    CodexAuthJson,
    /// Grok `auth.json` data.
    GrokAuthJson,
    /// OpenCode Go `auth.json` data.
    OpenCodeGoAuthJson,
    /// Amp `secrets.json` data.
    AmpSecrets,
    /// Z.AI API key environment variable.
    ZaiApiKeyEnv,
    /// Kimi API key environment variable.
    KimiApiKeyEnv,
    /// MiniMax token environment variable.
    MinimaxTokenEnv,
    /// Material whose provider-specific kind is not recognised.
    Unknown,
}

impl DetectedKind {
    /// Return the operator-facing label for this detected kind.
    pub fn label(self) -> &'static str {
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

    /// Map the detected kind to its provider, when known.
    pub fn provider(self) -> Option<Provider> {
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

/// Identity information associated with an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentitySubject {
    /// A provider- or service-specific display handle.
    Handle(String),
}

impl IdentitySubject {
    /// Return the display text for this identity subject.
    pub fn label(&self) -> &str {
        match self {
            IdentitySubject::Handle(s) => s,
        }
    }
}

/// Optional identity and plan metadata for an account.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountIdentity {
    /// Provider-reported or fixture-provided identity subject.
    pub subject: Option<IdentitySubject>,
    /// Human-readable plan or subscription label.
    pub plan: Option<String>,
}

impl AccountIdentity {
    /// Return the subject label, or a stable unresolved label when absent.
    pub fn label(&self) -> String {
        match &self.subject {
            Some(s) => s.label().to_owned(),
            None => "unresolved identity".into(),
        }
    }
}

/// Source provenance attached to an account observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provenance {
    /// Data came from a configured credential source.
    ConfiguredSource,
    /// Data came from live host discovery.
    LiveHost,
}

impl Provenance {
    /// Return the operator-facing label for this provenance.
    pub fn label(self) -> &'static str {
        match self {
            Provenance::ConfiguredSource => "configured source",
            Provenance::LiveHost => "live host",
        }
    }
}

/// Confidence level for the account data currently available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// The provider verified the account and its data.
    Authoritative,
    /// The value was inferred or estimated.
    Estimated,
    /// Presence was detected without identity or quota verification.
    PresenceOnly,
    /// No confidence has been established.
    None,
}

impl Confidence {
    /// Return the operator-facing label for this confidence level.
    pub fn label(self) -> &'static str {
        match self {
            Confidence::Authoritative => "authoritative",
            Confidence::Estimated => "estimated",
            Confidence::PresenceOnly => "presence only",
            Confidence::None => "none",
        }
    }
}

/// Current account or provider lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// Credentials and provider data are usable.
    Available,
    /// The agent has not yet been initialized.
    AgentUninitialized,
    /// Interactive login is required.
    NeedsLogin,
    /// A secret is required before use.
    NeedsSecret,
    /// The provider or source is not supported.
    Unsupported,
    /// The provider or source is currently unavailable.
    Unavailable,
    /// A validation or provider error occurred.
    Error,
}

impl Lifecycle {
    /// Return the operator-facing label for this lifecycle state.
    pub fn label(self) -> &'static str {
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

/// Highest validation milestone reached by an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationLevel {
    /// Credential material was found.
    MaterialDiscovered,
    /// Provider identity was authenticated.
    IdentityAuthenticated,
    /// Provider quota data was read successfully.
    QuotaReadable,
}

impl ValidationLevel {
    /// Return the operator-facing label for this validation level.
    pub fn label(self) -> &'static str {
        match self {
            ValidationLevel::MaterialDiscovered => "material discovered",
            ValidationLevel::IdentityAuthenticated => "identity authenticated",
            ValidationLevel::QuotaReadable => "quota readable",
        }
    }
}

/// State of the most recent account validation attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationState {
    /// No validation has been attempted.
    NeverValidated,
    /// Validation is in progress, identified by its start tick.
    Validating {
        /// Tick at which validation started.
        started_tick: u64,
    },
    /// Validation succeeded at the given level.
    Valid(ValidationLevel),
    /// Validation failed with a recoverable issue.
    Invalid(RecoverableIssue),
}

impl ValidationState {
    /// Return a compact operator-facing description of this state.
    pub fn label(&self) -> String {
        match self {
            ValidationState::NeverValidated => "never validated".into(),
            ValidationState::Validating { .. } => "validating…".into(),
            ValidationState::Valid(l) => l.label().to_owned(),
            ValidationState::Invalid(i) => i.message.clone(),
        }
    }

    /// Return the successful validation level, if validation succeeded.
    pub fn level(&self) -> Option<ValidationLevel> {
        match self {
            ValidationState::Valid(l) => Some(*l),
            _ => None,
        }
    }
}

/// Stable code identifying a recoverable account issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueCode {
    /// The provider does not expose quota data for this source.
    QuotaUnsupported,
    /// A configured credential folder is missing.
    FolderMissing,
    /// A configured credential folder cannot be read.
    FolderUnreadable,
    /// A detected folder belongs to another provider.
    FolderWrongProvider,
    /// A required credential file is missing.
    CredentialFileMissing,
    /// Credential file contents are malformed.
    CredentialMalformed,
    /// The supplied API key is empty.
    ApiKeyEmpty,
    /// The supplied API key is invalid.
    ApiKeyInvalid,
    /// 1Password is locked.
    OpLocked,
    /// 1Password authorization is required.
    OpAuthorizationRequired,
    /// 1Password denied access.
    OpPermissionDenied,
    /// The referenced 1Password item is missing.
    OpItemMissing,
    /// The referenced 1Password field is missing.
    OpFieldMissing,
    /// The 1Password item belongs to another provider.
    OpProviderMismatch,
    /// The provider rejected the credential.
    Unauthorized,
    /// The provider asked the caller to retry later.
    RateLimited,
    /// The provider is temporarily unavailable.
    ProviderUnavailable,
    /// Previously valid usage data is stale.
    Stale,
    /// The account identity could not be resolved.
    IdentityUnresolved,
}

/// Indicates how an account issue can be addressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recoverability {
    /// Retrying may resolve the issue without user action.
    Retryable,
    /// The operator must change credentials or configuration.
    ActionRequired,
    /// The requested capability is not supported.
    Unsupported,
}

impl Recoverability {
    /// Return the operator-facing label for this recoverability category.
    pub fn label(self) -> &'static str {
        match self {
            Recoverability::Retryable => "retryable",
            Recoverability::ActionRequired => "action required",
            Recoverability::Unsupported => "unsupported",
        }
    }
}

/// Structured issue information retained with an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverableIssue {
    /// Stable issue classification.
    pub code: IssueCode,
    /// Human-readable issue summary.
    pub message: String,
    /// Optional additional operator detail.
    pub detail: Option<String>,
    /// Expected remediation category.
    pub recoverability: Recoverability,
    /// Optional absolute retry time in fixture seconds.
    pub retry_secs: Option<i64>,
}

impl RecoverableIssue {
    /// Create an issue with no detail or retry time.
    pub fn new(code: IssueCode, message: impl Into<String>, rec: Recoverability) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            recoverability: rec,
            retry_secs: None,
        }
    }
    /// Add operator-facing detail to this issue.
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
    /// Add the absolute fixture time at which retrying is appropriate.
    pub fn retry(mut self, secs: i64) -> Self {
        self.retry_secs = Some(secs);
        self
    }

    /// Issues that are status, not errors.
    pub fn is_informational(&self) -> bool {
        matches!(self.code, IssueCode::Stale | IssueCode::IdentityUnresolved)
    }
}

/// Provider endpoint metadata associated with an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// Display label for the endpoint or deployment.
    pub label: String,
    /// Endpoint host or URL.
    pub host: String,
}

/// Account record combining identity, credentials, lifecycle, and usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Stable registry identifier.
    pub id: AccountId,
    /// Origin of this account record.
    pub origin: AccountOrigin,
    /// Operator-supplied display name.
    pub display_name: String,
    /// Agent runtime associated with the provider, when one exists.
    pub agent: Option<Agent>,
    /// Provider adapter represented by the account.
    pub provider: Provider,
    /// Usage registry surface associated with the provider.
    pub surface: UsageSurface,
    /// Credential source metadata.
    pub source: CredentialSource,
    /// Optional provider identity and plan metadata.
    pub identity: AccountIdentity,
    /// Sources that contributed observations to this record.
    pub provenance: Vec<Provenance>,
    /// Confidence in the account data.
    pub confidence: Confidence,
    /// Current account lifecycle state.
    pub lifecycle: Lifecycle,
    /// Optional operator purpose or note.
    pub purpose: Option<String>,
    /// Whether the account may be selected.
    pub enabled: bool,
    /// Whether this is the provider-wide default account.
    pub default_for_provider: bool,
    /// Most recent validation state.
    pub validation: ValidationState,
    /// Last successful refresh time in fixture seconds.
    pub last_refresh_secs: Option<i64>,
    /// Current recoverable issue, if any.
    pub issue: Option<RecoverableIssue>,
    /// Endpoint metadata, currently supported only for xAI fixtures.
    pub endpoint: Option<Endpoint>,
    /// Usage and freshness data for the account.
    pub usage: AccountUsage,
}

impl Account {
    /// Construct an enabled account configured by the operator.
    pub fn registered(id: &str, name: &str, provider: Provider, source: CredentialSource) -> Self {
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

    /// Construct an account discovered from the live host.
    pub fn discovered(id: &str, name: &str, provider: Provider, source: CredentialSource) -> Self {
        let mut a = Self::registered(id, name, provider, source);
        a.origin = AccountOrigin::Discovered;
        a.provenance = vec![Provenance::LiveHost];
        a
    }

    /// Attach endpoint metadata when the provider supports endpoints.
    pub fn with_endpoint(mut self, label: &str, host: &str) -> Self {
        if self.provider.supports_endpoint() {
            self.endpoint = Some(Endpoint {
                label: label.to_owned(),
                host: host.to_owned(),
            });
        }
        self
    }

    /// Return whether this account may be edited or removed.
    pub fn mutations_allowed(&self) -> bool {
        self.origin == AccountOrigin::Registered
    }

    /// Return the compact title used for account rows.
    /// `Codex · Primary`
    pub fn title(&self) -> String {
        format!("{} · {}", self.surface.surface_name(), self.display_name)
    }

    /// Return the highest-priority status word for the account row.
    /// Row status word: exhausted > error > needs … > warning > stale > refreshing > ok.
    pub fn status_word(&self) -> &'static str {
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

    /// Return whether the account or its usage is in an error-like state.
    pub fn is_error_state(&self) -> bool {
        matches!(
            self.lifecycle,
            Lifecycle::Error
                | Lifecycle::NeedsLogin
                | Lifecycle::NeedsSecret
                | Lifecycle::Unavailable
        ) || self.usage.freshness.phase == Freshness::Failed
    }
}

/// Criterion used to detect an account duplicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateProbe {
    /// Match a local-folder source by provider and path.
    Folder {
        /// Provider expected for the folder.
        provider: Provider,
        /// Folder path to match.
        path: String,
    },
    /// Match a 1Password source by canonical reference and account.
    OpReference {
        /// Canonical 1Password reference.
        canonical: String,
        /// 1Password account domain or identifier.
        account: String,
    },
    /// Match a plain API key by provider and fingerprint.
    KeyFingerprint {
        /// Provider expected for the key.
        provider: Provider,
        /// Non-secret key fingerprint.
        fingerprint: String,
    },
    /// Match an account by usage surface and identity subject.
    Identity {
        /// Usage surface to match.
        surface: UsageSurface,
        /// Identity subject to match.
        subject: IdentitySubject,
    },
}

/// Mutable collection of account records with a change revision.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountRegistry {
    /// Accounts in insertion order.
    pub accounts: Vec<Account>,
    /// Incremented whenever the registry is mutated.
    pub revision: u64,
}

impl AccountRegistry {
    /// Find an account by stable identifier.
    pub fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Find an account by stable identifier for mutation.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// Iterate over accounts belonging to a provider.
    pub fn by_provider(&self, p: Provider) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(move |a| a.provider == p)
    }

    /// Return the enabled provider default, if one is set.
    pub fn default_for(&self, p: Provider) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|a| a.provider == p && a.default_for_provider && a.enabled)
    }

    /// Return the first enabled, available account discovered on the host.
    pub fn discovered_current(&self, p: Provider) -> Option<&Account> {
        self.accounts.iter().find(|a| {
            a.provider == p
                && a.origin == AccountOrigin::Discovered
                && a.enabled
                && a.lifecycle == Lifecycle::Available
        })
    }

    /// Find the first account matching a duplicate probe.
    pub fn find_duplicate(&self, probe: &DuplicateProbe) -> Option<&Account> {
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

    /// Check whether a provider already uses a display name, excluding one id.
    /// Soft warning: same display name for the same provider.
    pub fn name_taken(&self, provider: Provider, name: &str, except: Option<&str>) -> bool {
        self.accounts.iter().any(|a| {
            a.provider == provider
                && a.display_name.eq_ignore_ascii_case(name)
                && Some(a.id.as_str()) != except
        })
    }

    /// Make an enabled account the only default for its provider.
    ///
    /// # Errors
    ///
    /// Returns an error when the account does not exist or is disabled.
    /// One default per provider: clears the sibling.
    pub fn set_default(&mut self, id: &str) -> Result<(), String> {
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

    /// Remove an account by id and return it when present.
    pub fn remove(&mut self, id: &str) -> Option<Account> {
        let i = self.accounts.iter().position(|a| a.id == id)?;
        self.revision += 1;
        Some(self.accounts.remove(i))
    }

    /// Insert an account and advance the registry revision.
    pub fn insert(&mut self, account: Account) {
        self.accounts.push(account);
        self.revision += 1;
    }

    /// Return accounts in surface order, with defaults before names.
    /// Registry order: surface registry order, defaults first, then name.
    pub fn sorted(&self) -> Vec<&Account> {
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
