#![doc = r" Provider accounts and API-key profiles: identity, credential source"]
#![doc = r" metadata (never secret material), lifecycle, validation, usage."]

use super::agent::{Agent, Provider, UsageSurface};
use super::onepassword::OpReference;
use super::usage::{AccountUsage, Freshness};

#[doc = r" Stable identifier for an account in a registry or workspace policy."]
pub type AccountId = String;

#[doc = r" Indicates whether an account was configured or discovered on the host."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountOrigin {
    #[doc = r" Created in the Center; editable, removable."]
    Registered,
    #[doc = r" Found by simulated host discovery; read-only."]
    Discovered,
}

#[doc = r" Describes where an account obtains its credentials."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    #[doc = r" Reference to credential metadata stored in 1Password."]
    OnePassword(OpReference),
    #[doc = r" Credential or profile material found in a local folder."]
    LocalFolder {
        #[doc = r" Folder path containing the detected material."]
        path: String,
        #[doc = r" Kind of material detected in the folder."]
        detected: DetectedKind,
    },
    #[doc = r" Synthetic metadata for a plain API key; secret bytes are not stored."]
    PlainApiKey {
        #[doc = r" 8 hex chars, deterministic from the fixture handle; never derived"]
        #[doc = r" from the secret bytes."]
        fingerprint: String,
        #[doc = r" Exactly four synthetic characters shown as `…k7Qz`."]
        tail: String,
    },
    #[doc = r" Credential exposed through a host environment variable."]
    HostEnv {
        #[doc = r" Name of the environment variable."]
        var: String,
        #[doc = r" Kind of material detected from the variable."]
        detected: DetectedKind,
    },
}

impl CredentialSource {
    #[doc = r" Return the operator-facing label for this credential source."]
    pub fn origin_label(&self) -> &'static str {
        match self {
            CredentialSource::OnePassword(_) => "1Password",
            CredentialSource::LocalFolder { .. } => "Local folder",
            CredentialSource::PlainApiKey { .. } => "API key",
            CredentialSource::HostEnv { .. } => "Host env",
        }
    }

    #[doc = r" Non-secret detail: reference path, folder path, or key fingerprint."]
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

#[doc = r" `••••••••…k7Qz`"]
pub fn masked(tail: &str) -> String {
    format!("••••••••…{tail}")
}

#[doc = r" Deterministic 8-hex fingerprint of a fixture key (not of secret bytes"]
#[doc = r" in the real product; here of the synthetic value so duplicates match)."]
pub fn fingerprint(value: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in value.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{:08x}", (h >> 32) as u32 ^ h as u32)
}

#[doc = r" Last four characters of a typed key become the synthetic tail."]
pub fn tail_of(value: &str) -> String {
    let n = value.chars().count();
    value.chars().skip(n.saturating_sub(4)).collect()
}

#[doc = r" Kind of credential or profile material detected by discovery."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedKind {
    #[doc = r" Claude OAuth profile data."]
    ClaudeOAuthProfile,
    #[doc = r" Claude API key environment variable."]
    ClaudeApiKeyEnv,
    #[doc = r" Codex `auth.json` data."]
    CodexAuthJson,
    #[doc = r" Grok `auth.json` data."]
    GrokAuthJson,
    #[doc = r" OpenCode Go `auth.json` data."]
    OpenCodeGoAuthJson,
    #[doc = r" Amp `secrets.json` data."]
    AmpSecrets,
    #[doc = r" Z.AI API key environment variable."]
    ZaiApiKeyEnv,
    #[doc = r" Kimi API key environment variable."]
    KimiApiKeyEnv,
    #[doc = r" MiniMax token environment variable."]
    MinimaxTokenEnv,
    #[doc = r" Material whose provider-specific kind is not recognised."]
    Unknown,
}

impl DetectedKind {
    #[doc = r" Return the operator-facing label for this detected kind."]
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

    #[doc = r" Map the detected kind to its provider, when known."]
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

#[doc = r" Identity information associated with an account."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentitySubject {
    #[doc = r" A provider- or service-specific display handle."]
    Handle(String),
}

impl IdentitySubject {
    #[doc = r" Return the display text for this identity subject."]
    pub fn label(&self) -> &str {
        match self {
            IdentitySubject::Handle(s) => s,
        }
    }
}

#[doc = r" Optional identity and plan metadata for an account."]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountIdentity {
    #[doc = r" Provider-reported or fixture-provided identity subject."]
    pub subject: Option<IdentitySubject>,
    #[doc = r" Human-readable plan or subscription label."]
    pub plan: Option<String>,
}

impl AccountIdentity {
    #[doc = r" Return the subject label, or a stable unresolved label when absent."]
    pub fn label(&self) -> String {
        match &self.subject {
            Some(s) => s.label().to_owned(),
            None => "unresolved identity".into(),
        }
    }
}

#[doc = r" Source provenance attached to an account observation."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provenance {
    #[doc = r" Data came from a configured credential source."]
    ConfiguredSource,
    #[doc = r" Data came from live host discovery."]
    LiveHost,
}

impl Provenance {
    #[doc = r" Return the operator-facing label for this provenance."]
    pub fn label(self) -> &'static str {
        match self {
            Provenance::ConfiguredSource => "configured source",
            Provenance::LiveHost => "live host",
        }
    }
}

#[doc = r" Confidence level for the account data currently available."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    #[doc = r" The provider verified the account and its data."]
    Authoritative,
    #[doc = r" The value was inferred or estimated."]
    Estimated,
    #[doc = r" Presence was detected without identity or quota verification."]
    PresenceOnly,
    #[doc = r" No confidence has been established."]
    None,
}

impl Confidence {
    #[doc = r" Return the operator-facing label for this confidence level."]
    pub fn label(self) -> &'static str {
        match self {
            Confidence::Authoritative => "authoritative",
            Confidence::Estimated => "estimated",
            Confidence::PresenceOnly => "presence only",
            Confidence::None => "none",
        }
    }
}

#[doc = r" Current account or provider lifecycle state."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    #[doc = r" Credentials and provider data are usable."]
    Available,
    #[doc = r" The agent has not yet been initialized."]
    AgentUninitialized,
    #[doc = r" Interactive login is required."]
    NeedsLogin,
    #[doc = r" A secret is required before use."]
    NeedsSecret,
    #[doc = r" The provider or source is not supported."]
    Unsupported,
    #[doc = r" The provider or source is currently unavailable."]
    Unavailable,
    #[doc = r" A validation or provider error occurred."]
    Error,
}

impl Lifecycle {
    #[doc = r" Return the operator-facing label for this lifecycle state."]
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

#[doc = r" Highest validation milestone reached by an account."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationLevel {
    #[doc = r" Credential material was found."]
    MaterialDiscovered,
    #[doc = r" Provider identity was authenticated."]
    IdentityAuthenticated,
    #[doc = r" Provider quota data was read successfully."]
    QuotaReadable,
}

impl ValidationLevel {
    #[doc = r" Return the operator-facing label for this validation level."]
    pub fn label(self) -> &'static str {
        match self {
            ValidationLevel::MaterialDiscovered => "material discovered",
            ValidationLevel::IdentityAuthenticated => "identity authenticated",
            ValidationLevel::QuotaReadable => "quota readable",
        }
    }
}

#[doc = r" State of the most recent account validation attempt."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationState {
    #[doc = r" No validation has been attempted."]
    NeverValidated,
    #[doc = r" Validation is in progress, identified by its start tick."]
    Validating {
        #[doc = r" Tick at which validation started."]
        started_tick: u64,
    },
    #[doc = r" Validation succeeded at the given level."]
    Valid(ValidationLevel),
    #[doc = r" Validation failed with a recoverable issue."]
    Invalid(RecoverableIssue),
}

impl ValidationState {
    #[doc = r" Return a compact operator-facing description of this state."]
    pub fn label(&self) -> String {
        match self {
            ValidationState::NeverValidated => "never validated".into(),
            ValidationState::Validating { .. } => "validating…".into(),
            ValidationState::Valid(l) => l.label().to_owned(),
            ValidationState::Invalid(i) => i.message.clone(),
        }
    }

    #[doc = r" Return the successful validation level, if validation succeeded."]
    pub fn level(&self) -> Option<ValidationLevel> {
        match self {
            ValidationState::Valid(l) => Some(*l),
            _ => None,
        }
    }
}

#[doc = r" Stable code identifying a recoverable account issue."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueCode {
    #[doc = r" The provider does not expose quota data for this source."]
    QuotaUnsupported,
    #[doc = r" A configured credential folder is missing."]
    FolderMissing,
    #[doc = r" A configured credential folder cannot be read."]
    FolderUnreadable,
    #[doc = r" A detected folder belongs to another provider."]
    FolderWrongProvider,
    #[doc = r" A required credential file is missing."]
    CredentialFileMissing,
    #[doc = r" Credential file contents are malformed."]
    CredentialMalformed,
    #[doc = r" The supplied API key is empty."]
    ApiKeyEmpty,
    #[doc = r" The supplied API key is invalid."]
    ApiKeyInvalid,
    #[doc = r" 1Password is locked."]
    OpLocked,
    #[doc = r" 1Password authorization is required."]
    OpAuthorizationRequired,
    #[doc = r" 1Password denied access."]
    OpPermissionDenied,
    #[doc = r" The referenced 1Password item is missing."]
    OpItemMissing,
    #[doc = r" The referenced 1Password field is missing."]
    OpFieldMissing,
    #[doc = r" The 1Password item belongs to another provider."]
    OpProviderMismatch,
    #[doc = r" The provider rejected the credential."]
    Unauthorized,
    #[doc = r" The provider asked the caller to retry later."]
    RateLimited,
    #[doc = r" The provider is temporarily unavailable."]
    ProviderUnavailable,
    #[doc = r" Previously valid usage data is stale."]
    Stale,
    #[doc = r" The account identity could not be resolved."]
    IdentityUnresolved,
}

#[doc = r" Indicates how an account issue can be addressed."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recoverability {
    #[doc = r" Retrying may resolve the issue without user action."]
    Retryable,
    #[doc = r" The operator must change credentials or configuration."]
    ActionRequired,
    #[doc = r" The requested capability is not supported."]
    Unsupported,
}

impl Recoverability {
    #[doc = r" Return the operator-facing label for this recoverability category."]
    pub fn label(self) -> &'static str {
        match self {
            Recoverability::Retryable => "retryable",
            Recoverability::ActionRequired => "action required",
            Recoverability::Unsupported => "unsupported",
        }
    }
}

#[doc = r" Structured issue information retained with an account."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverableIssue {
    #[doc = r" Stable issue classification."]
    pub code: IssueCode,
    #[doc = r" Human-readable issue summary."]
    pub message: String,
    #[doc = r" Optional additional operator detail."]
    pub detail: Option<String>,
    #[doc = r" Expected remediation category."]
    pub recoverability: Recoverability,
    #[doc = r" Optional absolute retry time in fixture seconds."]
    pub retry_secs: Option<i64>,
}

impl RecoverableIssue {
    #[doc = r" Create an issue with no detail or retry time."]
    pub fn new(code: IssueCode, message: impl Into<String>, rec: Recoverability) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            recoverability: rec,
            retry_secs: None,
        }
    }
    #[doc = r" Add operator-facing detail to this issue."]
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
    #[doc = r" Add the absolute fixture time at which retrying is appropriate."]
    pub fn retry(mut self, secs: i64) -> Self {
        self.retry_secs = Some(secs);
        self
    }

    #[doc = r" Issues that are status, not errors."]
    pub fn is_informational(&self) -> bool {
        matches!(self.code, IssueCode::Stale | IssueCode::IdentityUnresolved)
    }
}

#[doc = r" Provider endpoint metadata associated with an account."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    #[doc = r" Display label for the endpoint or deployment."]
    pub label: String,
    #[doc = r" Endpoint host or URL."]
    pub host: String,
}

#[doc = r" Account record combining identity, credentials, lifecycle, and usage."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    #[doc = r" Stable registry identifier."]
    pub id: AccountId,
    #[doc = r" Origin of this account record."]
    pub origin: AccountOrigin,
    #[doc = r" Operator-supplied display name."]
    pub display_name: String,
    #[doc = r" Agent runtime associated with the provider, when one exists."]
    pub agent: Option<Agent>,
    #[doc = r" Provider adapter represented by the account."]
    pub provider: Provider,
    #[doc = r" Usage registry surface associated with the provider."]
    pub surface: UsageSurface,
    #[doc = r" Credential source metadata."]
    pub source: CredentialSource,
    #[doc = r" Optional provider identity and plan metadata."]
    pub identity: AccountIdentity,
    #[doc = r" Sources that contributed observations to this record."]
    pub provenance: Vec<Provenance>,
    #[doc = r" Confidence in the account data."]
    pub confidence: Confidence,
    #[doc = r" Current account lifecycle state."]
    pub lifecycle: Lifecycle,
    #[doc = r" Optional operator purpose or note."]
    pub purpose: Option<String>,
    #[doc = r" Whether the account may be selected."]
    pub enabled: bool,
    #[doc = r" Whether this is the provider-wide default account."]
    pub default_for_provider: bool,
    #[doc = r" Most recent validation state."]
    pub validation: ValidationState,
    #[doc = r" Last successful refresh time in fixture seconds."]
    pub last_refresh_secs: Option<i64>,
    #[doc = r" Current recoverable issue, if any."]
    pub issue: Option<RecoverableIssue>,
    #[doc = r" Endpoint metadata, currently supported only for xAI fixtures."]
    pub endpoint: Option<Endpoint>,
    #[doc = r" Usage and freshness data for the account."]
    pub usage: AccountUsage,
}

impl Account {
    #[doc = r" Construct an enabled account configured by the operator."]
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

    #[doc = r" Construct an account discovered from the live host."]
    pub fn discovered(id: &str, name: &str, provider: Provider, source: CredentialSource) -> Self {
        let mut a = Self::registered(id, name, provider, source);
        a.origin = AccountOrigin::Discovered;
        a.provenance = vec![Provenance::LiveHost];
        a
    }

    #[doc = r" Attach endpoint metadata when the provider supports endpoints."]
    pub fn with_endpoint(mut self, label: &str, host: &str) -> Self {
        if self.provider.supports_endpoint() {
            self.endpoint = Some(Endpoint {
                label: label.to_owned(),
                host: host.to_owned(),
            });
        }
        self
    }

    #[doc = r" Return whether this account may be edited or removed."]
    pub fn mutations_allowed(&self) -> bool {
        self.origin == AccountOrigin::Registered
    }

    #[doc = r" Return the compact title used for account rows."]
    #[doc = r" `Codex · Primary`"]
    pub fn title(&self) -> String {
        format!("{} · {}", self.surface.surface_name(), self.display_name)
    }

    #[doc = r" Return the highest-priority status word for the account row."]
    #[doc = r" Row status word: exhausted > error > needs … > warning > stale > refreshing > ok."]
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

    #[doc = r" Return whether the account or its usage is in an error-like state."]
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

#[doc = r" Criterion used to detect an account duplicate."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateProbe {
    #[doc = r" Match a local-folder source by provider and path."]
    Folder {
        #[doc = r" Provider expected for the folder."]
        provider: Provider,
        #[doc = r" Folder path to match."]
        path: String,
    },
    #[doc = r" Match a 1Password source by canonical reference and account."]
    OpReference {
        #[doc = r" Canonical 1Password reference."]
        canonical: String,
        #[doc = r" 1Password account domain or identifier."]
        account: String,
    },
    #[doc = r" Match a plain API key by provider and fingerprint."]
    KeyFingerprint {
        #[doc = r" Provider expected for the key."]
        provider: Provider,
        #[doc = r" Non-secret key fingerprint."]
        fingerprint: String,
    },
    #[doc = r" Match an account by usage surface and identity subject."]
    Identity {
        #[doc = r" Usage surface to match."]
        surface: UsageSurface,
        #[doc = r" Identity subject to match."]
        subject: IdentitySubject,
    },
}

#[doc = r" Mutable collection of account records with a change revision."]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountRegistry {
    #[doc = r" Accounts in insertion order."]
    pub accounts: Vec<Account>,
    #[doc = r" Incremented whenever the registry is mutated."]
    pub revision: u64,
}

impl AccountRegistry {
    #[doc = r" Find an account by stable identifier."]
    pub fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    #[doc = r" Find an account by stable identifier for mutation."]
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    #[doc = r" Iterate over accounts belonging to a provider."]
    pub fn by_provider(&self, p: Provider) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(move |a| a.provider == p)
    }

    #[doc = r" Return the enabled provider default, if one is set."]
    pub fn default_for(&self, p: Provider) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|a| a.provider == p && a.default_for_provider && a.enabled)
    }

    #[doc = r" Return the first enabled, available account discovered on the host."]
    pub fn discovered_current(&self, p: Provider) -> Option<&Account> {
        self.accounts.iter().find(|a| {
            a.provider == p
                && a.origin == AccountOrigin::Discovered
                && a.enabled
                && a.lifecycle == Lifecycle::Available
        })
    }

    #[doc = r" Find the first account matching a duplicate probe."]
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

    #[doc = r" Check whether a provider already uses a display name, excluding one id."]
    #[doc = r" Soft warning: same display name for the same provider."]
    pub fn name_taken(&self, provider: Provider, name: &str, except: Option<&str>) -> bool {
        self.accounts.iter().any(|a| {
            a.provider == provider
                && a.display_name.eq_ignore_ascii_case(name)
                && Some(a.id.as_str()) != except
        })
    }

    #[doc = r" Make an enabled account the only default for its provider."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns an error when the account does not exist or is disabled."]
    #[doc = r" One default per provider: clears the sibling."]
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

    #[doc = r" Remove an account by id and return it when present."]
    pub fn remove(&mut self, id: &str) -> Option<Account> {
        let i = self.accounts.iter().position(|a| a.id == id)?;
        self.revision += 1;
        Some(self.accounts.remove(i))
    }

    #[doc = r" Insert an account and advance the registry revision."]
    pub fn insert(&mut self, account: Account) {
        self.accounts.push(account);
        self.revision += 1;
    }

    #[doc = r" Return accounts in surface order, with defaults before names."]
    #[doc = r" Registry order: surface registry order, defaults first, then name."]
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
