//! Deterministic provider operations: validate a credential source and
//! refresh an account's usage. Secret material never leaves the call.

use crate::domain::account::{
    Account, AccountIdentity, Confidence, CredentialSource, DetectedKind, IdentitySubject,
    IssueCode, Lifecycle, Recoverability, RecoverableIssue, ValidationLevel,
};
use crate::domain::agent::Provider;
use crate::domain::usage::{
    AccountUsage, FreshnessInfo, QuotaStatus, QuotaWindow, WindowCategory, WindowUnit,
};
use crate::sim::onepassword::{KeyOutcome, OpError, SecretClass, SimOnePassword, classify_plain};

/// Three-level validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationOutcome {
    pub level: Option<ValidationLevel>,
    pub identity: AccountIdentity,
    pub confidence: Confidence,
    pub lifecycle: Lifecycle,
    pub issue: Option<RecoverableIssue>,
    pub usage: Option<AccountUsage>,
    /// Three rows for the validation card.
    pub material: CheckRow,
    pub identity_row: CheckRow,
    pub quota_row: CheckRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckRow {
    Ok(String),
    Failed(String),
    Skipped(String),
}

/// Simulated local folder inventory for folder-backed sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderProbe {
    Missing,
    Unreadable,
    /// Folder exists, holds no credential file.
    NoCredential,
    /// Credential file exists but does not parse.
    Malformed,
    Found(DetectedKind),
}

pub fn probe_folder(path: &str) -> FolderProbe {
    let p = path.trim_end_matches('/');
    match p {
        "~/.claude" | "/Users/alexey/.claude" => {
            FolderProbe::Found(DetectedKind::ClaudeOAuthProfile)
        }
        "~/.claude-work" => FolderProbe::Found(DetectedKind::ClaudeApiKeyEnv),
        "~/.codex" | "/Users/alexey/.codex" => FolderProbe::Found(DetectedKind::CodexAuthJson),
        "~/.grok" => FolderProbe::NoCredential,
        "~/.grok-team" => FolderProbe::Found(DetectedKind::GrokAuthJson),
        "~/.local/share/opencode" | "~/.local/share/opencode/auth.json" => {
            FolderProbe::Found(DetectedKind::OpenCodeGoAuthJson)
        }
        "~/.opencode-broken" => FolderProbe::Malformed,
        "~/.codex-locked" => FolderProbe::Unreadable,
        "~/.kimi" => FolderProbe::Found(DetectedKind::KimiApiKeyEnv),
        "~/.config/amp" => FolderProbe::Found(DetectedKind::AmpSecrets),
        "~/Library/Application Support/jackin/profiles/claude-contractor-2025-archive" => {
            FolderProbe::Found(DetectedKind::ClaudeApiKeyEnv)
        }
        _ => FolderProbe::Missing,
    }
}

fn issue(code: IssueCode, msg: impl Into<String>, rec: Recoverability) -> RecoverableIssue {
    RecoverableIssue::new(code, msg, rec)
}

fn identity_for(provider: Provider, tag: &str) -> (AccountIdentity, Confidence) {
    let (subject, plan) = match (provider, tag) {
        (Provider::Anthropic, t) if t.contains("ant02") => ("alexey@donbeave.dev", "Max 5x"),
        (Provider::Anthropic, _) => ("alexey@chainargos.com", "Team"),
        (Provider::OpenAi, t) if t.contains("cdx02") => ("ChatGPT account org_7Hq2", "Plus"),
        (Provider::OpenAi, _) => ("ChatGPT account org_7Hq2", "Pro 20x"),
        (Provider::XAi, _) => ("team_chainargos", "Team · prepaid"),
        (Provider::OpenCode, _) => ("donbeave", "OpenCode Go"),
        _ => ("unknown", "unknown"),
    };
    (
        AccountIdentity {
            subject: Some(IdentitySubject::Handle(subject.into())),
            plan: Some(plan.into()),
        },
        Confidence::Authoritative,
    )
}

/// Windows a freshly validated key-backed account reports.
pub fn windows_for(provider: Provider, now: i64, key_backed: bool) -> Vec<QuotaWindow> {
    let h = 3600;
    match provider {
        Provider::Anthropic => vec![
            QuotaWindow::pct("session", "Session · 5-hour", WindowCategory::Session, 38)
                .reset(now + 3 * h + 12 * 60),
            QuotaWindow::pct(
                "weekly",
                "Weekly · all models",
                WindowCategory::LongRange,
                33,
            )
            .reset(now + 3 * 86_400 + 10 * h),
            QuotaWindow::pct(
                "weekly_sonnet",
                "Weekly · Sonnet",
                WindowCategory::Model,
                21,
            )
            .reset(now + 3 * 86_400 + 10 * h),
            QuotaWindow::pct("weekly_opus", "Weekly · Opus", WindowCategory::Model, 54)
                .reset(now + 3 * 86_400 + 10 * h),
        ],
        Provider::OpenAi if key_backed => vec![QuotaWindow::sentinel(
            "quota",
            "Quota",
            QuotaStatus::Unsupported,
            "Quota not visible for API keys",
        )],
        Provider::OpenAi => vec![
            QuotaWindow::pct("session", "Session · 5-hour", WindowCategory::Session, 12)
                .reset(now + 4 * h + 40 * 60),
            QuotaWindow::pct("weekly", "Weekly · 7-day", WindowCategory::LongRange, 59)
                .reset(now + 2 * 86_400 + 19 * h),
            QuotaWindow::not_started("spark", "Codex Spark · 5-hour", WindowCategory::Model),
            QuotaWindow::counted(
                "credits",
                "Credits",
                WindowCategory::Other,
                WindowUnit::Credits,
                1_240,
                5_000,
            ),
        ],
        Provider::XAi => vec![
            QuotaWindow::pct("monthly", "Monthly", WindowCategory::LongRange, 31)
                .reset(now + 27 * 86_400 + 10 * h),
            QuotaWindow::pct("weekly", "Weekly", WindowCategory::LongRange, 68)
                .reset(now + 3 * 86_400 + 10 * h),
            QuotaWindow::counted(
                "credits",
                "Credits",
                WindowCategory::Other,
                WindowUnit::Usd,
                3_140,
                10_000,
            )
            .spend("$68.60 remaining of $100.00"),
            QuotaWindow::counted(
                "ondemand",
                "On-demand usage",
                WindowCategory::Other,
                WindowUnit::Usd,
                315,
                315,
            )
            .status(QuotaStatus::Available)
            .spend("$3.15 this month"),
        ],
        Provider::OpenCode => vec![
            QuotaWindow::pct("rolling", "Rolling", WindowCategory::Session, 57)
                .reset(now + h + 50 * 60),
            QuotaWindow::pct("weekly", "Weekly", WindowCategory::LongRange, 45)
                .reset(now + 3 * 86_400),
            QuotaWindow::pct("monthly", "Monthly", WindowCategory::LongRange, 22)
                .reset(now + 27 * 86_400),
        ],
        _ => vec![],
    }
}

/// Validate a credential source for `provider`. Runs the provider op
/// inside the 1Password closure when the source is a reference.
pub fn validate(
    provider: Provider,
    source: &CredentialSource,
    plain_value: Option<&str>,
    op: &SimOnePassword,
    now: i64,
) -> ValidationOutcome {
    let key_class: Result<(SecretClass, &'static str), OpError> = match source {
        CredentialSource::OnePassword(r) => match op.resolve_into(r, |s| s.classify()) {
            Ok(c) => Ok((c, "1Password reference resolved")),
            Err(e) => Err(e),
        },
        CredentialSource::PlainApiKey { .. } => Ok((
            classify_plain(provider, plain_value.unwrap_or("")),
            "Key material present",
        )),
        CredentialSource::LocalFolder { path, .. }
        | CredentialSource::HostEnv { var: path, .. } => {
            return validate_folder(provider, path, now);
        }
    };
    let (class, material_msg) = match key_class {
        Ok(v) => v,
        Err(e) => {
            let code = match &e {
                OpError::Locked => IssueCode::OpLocked,
                OpError::AuthorizationRequired { .. } => IssueCode::OpAuthorizationRequired,
                OpError::PermissionDenied { .. } => IssueCode::OpPermissionDenied,
                OpError::MissingAccount { .. }
                | OpError::MissingVault { .. }
                | OpError::MissingItem { .. } => IssueCode::OpItemMissing,
                OpError::MissingField { .. } | OpError::WrongFieldShape { .. } => {
                    IssueCode::OpFieldMissing
                }
                OpError::EmptyMaterial { .. } => IssueCode::ApiKeyEmpty,
            };
            let rec = if e.retryable() {
                Recoverability::Retryable
            } else {
                Recoverability::ActionRequired
            };
            return ValidationOutcome {
                level: None,
                identity: AccountIdentity::default(),
                confidence: Confidence::None,
                lifecycle: Lifecycle::NeedsSecret,
                issue: Some(issue(code, e.message(), rec)),
                usage: None,
                material: CheckRow::Failed(e.message()),
                identity_row: CheckRow::Skipped("not attempted".into()),
                quota_row: CheckRow::Skipped("not attempted".into()),
            };
        }
    };
    match class {
        SecretClass::Empty => ValidationOutcome {
            level: None,
            identity: AccountIdentity::default(),
            confidence: Confidence::None,
            lifecycle: Lifecycle::NeedsSecret,
            issue: Some(issue(
                IssueCode::ApiKeyEmpty,
                "API key required: the field is empty",
                Recoverability::ActionRequired,
            )),
            usage: None,
            material: CheckRow::Failed("Field is empty".into()),
            identity_row: CheckRow::Skipped("not attempted".into()),
            quota_row: CheckRow::Skipped("not attempted".into()),
        },
        SecretClass::Unrecognised => ValidationOutcome {
            level: Some(ValidationLevel::MaterialDiscovered),
            identity: AccountIdentity::default(),
            confidence: Confidence::None,
            lifecycle: Lifecycle::Error,
            issue: Some(issue(
                IssueCode::ApiKeyInvalid,
                "Key rejected: the provider returned 401",
                Recoverability::ActionRequired,
            )),
            usage: None,
            material: CheckRow::Ok(material_msg.into()),
            identity_row: CheckRow::Failed("Not authenticated (401)".into()),
            quota_row: CheckRow::Skipped("not attempted".into()),
        },
        SecretClass::Key {
            provider: found,
            outcome,
        } => {
            if found != provider {
                return ValidationOutcome {
                    level: Some(ValidationLevel::MaterialDiscovered),
                    identity: AccountIdentity::default(),
                    confidence: Confidence::None,
                    lifecycle: Lifecycle::Error,
                    issue: Some(issue(
                        IssueCode::OpProviderMismatch,
                        format!(
                            "Provider mismatch: the key belongs to {}, expected {}",
                            found.short(),
                            provider.short()
                        ),
                        Recoverability::ActionRequired,
                    )),
                    usage: None,
                    material: CheckRow::Ok(material_msg.into()),
                    identity_row: CheckRow::Failed(format!("Key class is {}", found.short())),
                    quota_row: CheckRow::Skipped("not attempted".into()),
                };
            }
            let key_backed = true;
            let tag = match source {
                CredentialSource::OnePassword(r) => r.item_id.clone(),
                _ => plain_value.unwrap_or("").to_owned(),
            };
            let (identity, confidence) = identity_for(provider, &tag);
            match outcome {
                KeyOutcome::Valid => {
                    let windows = windows_for(provider, now, key_backed);
                    let unsupported = windows.iter().all(|w| w.status == QuotaStatus::Unsupported);
                    ValidationOutcome {
                        level: Some(if unsupported {
                            ValidationLevel::IdentityAuthenticated
                        } else {
                            ValidationLevel::QuotaReadable
                        }),
                        identity: identity.clone(),
                        confidence,
                        lifecycle: Lifecycle::Available,
                        issue: unsupported.then(|| {
                            issue(
                                IssueCode::QuotaUnsupported,
                                format!(
                                    "Quota not visible: {} does not expose usage for API keys",
                                    provider.short()
                                ),
                                Recoverability::Unsupported,
                            )
                        }),
                        usage: Some(AccountUsage {
                            freshness: FreshnessInfo::current(now),
                            windows: windows.clone(),
                        }),
                        material: CheckRow::Ok(material_msg.into()),
                        identity_row: CheckRow::Ok(format!(
                            "Authenticated as {} · {}",
                            identity.label(),
                            identity.plan.clone().unwrap_or_default()
                        )),
                        quota_row: if unsupported {
                            CheckRow::Skipped("Quota not visible for API keys".into())
                        } else {
                            CheckRow::Ok(format!("{} windows readable", windows.len()))
                        },
                    }
                }
                KeyOutcome::Rejected => ValidationOutcome {
                    level: Some(ValidationLevel::MaterialDiscovered),
                    identity: AccountIdentity::default(),
                    confidence: Confidence::None,
                    lifecycle: Lifecycle::NeedsLogin,
                    issue: Some(issue(
                        IssueCode::ApiKeyInvalid,
                        "Key rejected: the provider returned 401",
                        Recoverability::ActionRequired,
                    )),
                    usage: None,
                    material: CheckRow::Ok(material_msg.into()),
                    identity_row: CheckRow::Failed(
                        "Not authenticated (401) · key rotated or revoked".into(),
                    ),
                    quota_row: CheckRow::Skipped("not attempted".into()),
                },
                KeyOutcome::RateLimited => ValidationOutcome {
                    level: Some(ValidationLevel::IdentityAuthenticated),
                    identity: identity.clone(),
                    confidence,
                    lifecycle: Lifecycle::Available,
                    issue: Some(
                        issue(
                            IssueCode::RateLimited,
                            "Rate limited: retry after 25 min",
                            Recoverability::Retryable,
                        )
                        .retry(now + 25 * 60),
                    ),
                    usage: Some(AccountUsage {
                        freshness: FreshnessInfo::failed(None, Some(now + 25 * 60)),
                        windows: vec![],
                    }),
                    material: CheckRow::Ok(material_msg.into()),
                    identity_row: CheckRow::Ok(format!("Authenticated as {}", identity.label())),
                    quota_row: CheckRow::Failed("Rate limited (429) · retry after 25 min".into()),
                },
                KeyOutcome::Unavailable => ValidationOutcome {
                    level: Some(ValidationLevel::MaterialDiscovered),
                    identity: AccountIdentity::default(),
                    confidence: Confidence::None,
                    lifecycle: Lifecycle::Unavailable,
                    issue: Some(issue(
                        IssueCode::ProviderUnavailable,
                        format!("Provider unavailable: {} did not respond", provider.short()),
                        Recoverability::Retryable,
                    )),
                    usage: None,
                    material: CheckRow::Ok(material_msg.into()),
                    identity_row: CheckRow::Failed("Timed out after 8 s".into()),
                    quota_row: CheckRow::Skipped("not attempted".into()),
                },
            }
        }
    }
}

fn validate_folder(provider: Provider, path: &str, now: i64) -> ValidationOutcome {
    let fail = |code, msg: String, lifecycle, material: CheckRow| ValidationOutcome {
        level: if matches!(material, CheckRow::Ok(_)) {
            Some(ValidationLevel::MaterialDiscovered)
        } else {
            None
        },
        identity: AccountIdentity::default(),
        confidence: Confidence::None,
        lifecycle,
        issue: Some(issue(code, msg, Recoverability::ActionRequired)),
        usage: None,
        material,
        identity_row: CheckRow::Skipped("not attempted".into()),
        quota_row: CheckRow::Skipped("not attempted".into()),
    };
    match probe_folder(path) {
        FolderProbe::Missing => fail(
            IssueCode::FolderMissing,
            format!("Folder not found: {path}"),
            Lifecycle::Error,
            CheckRow::Failed("Nothing exists at the path".into()),
        ),
        FolderProbe::Unreadable => fail(
            IssueCode::FolderUnreadable,
            format!("Folder not readable: permission denied for {path}"),
            Lifecycle::Error,
            CheckRow::Failed("mode 000 · owner root".into()),
        ),
        FolderProbe::NoCredential => fail(
            IssueCode::CredentialFileMissing,
            format!("No credential found: {path} has no auth.json"),
            Lifecycle::NeedsLogin,
            CheckRow::Failed("Run the agent login first".into()),
        ),
        FolderProbe::Malformed => fail(
            IssueCode::CredentialMalformed,
            "Credential unreadable: auth.json is not valid JSON".into(),
            Lifecycle::Error,
            CheckRow::Ok("auth.json found · parse failed at line 1".into()),
        ),
        FolderProbe::Found(kind) => {
            if kind.provider() != Some(provider) {
                let found = kind.provider().map(|p| p.short()).unwrap_or("unknown");
                return fail(
                    IssueCode::FolderWrongProvider,
                    format!(
                        "Folder belongs to another provider: found a {found} profile, expected {}",
                        provider.short()
                    ),
                    Lifecycle::Error,
                    CheckRow::Ok(format!("Detected {}", kind.label())),
                );
            }
            let presence_only = matches!(
                kind,
                DetectedKind::ClaudeApiKeyEnv | DetectedKind::KimiApiKeyEnv
            );
            if presence_only {
                return ValidationOutcome {
                    level: Some(ValidationLevel::MaterialDiscovered),
                    identity: AccountIdentity::default(),
                    confidence: Confidence::PresenceOnly,
                    lifecycle: Lifecycle::NeedsLogin,
                    issue: Some(issue(
                        IssueCode::IdentityUnresolved,
                        "Identity unresolved · showing usage without a public handle",
                        Recoverability::Unsupported,
                    )),
                    usage: None,
                    material: CheckRow::Ok(format!("Detected {}", kind.label())),
                    identity_row: CheckRow::Skipped("No stable public identity".into()),
                    quota_row: CheckRow::Skipped("Confidence: presence only".into()),
                };
            }
            let (identity, _) = identity_for(provider, path);
            let identity = match provider {
                Provider::Anthropic => AccountIdentity {
                    subject: Some(IdentitySubject::Handle("alexey@donbeave.dev".into())),
                    plan: Some("Max 5x".into()),
                },
                _ => identity,
            };
            let windows = windows_for(provider, now, false);
            ValidationOutcome {
                level: Some(ValidationLevel::QuotaReadable),
                identity: identity.clone(),
                confidence: Confidence::Authoritative,
                lifecycle: Lifecycle::Available,
                issue: None,
                usage: Some(AccountUsage {
                    freshness: FreshnessInfo::current(now),
                    windows: windows.clone(),
                }),
                material: CheckRow::Ok(format!("Detected {}", kind.label())),
                identity_row: CheckRow::Ok(format!(
                    "Authenticated as {} · {}",
                    identity.label(),
                    identity.plan.clone().unwrap_or_default()
                )),
                quota_row: CheckRow::Ok(format!("{} windows readable", windows.len())),
            }
        }
    }
}

/// Apply a validation outcome to an account (never touches the source).
pub fn apply_validation(a: &mut Account, v: &ValidationOutcome, now: i64) {
    a.validation = match (&v.level, &v.issue) {
        (Some(l), None) => crate::domain::account::ValidationState::Valid(*l),
        (Some(l), Some(i))
            if i.is_informational()
                || i.code == IssueCode::QuotaUnsupported
                || i.code == IssueCode::RateLimited =>
        {
            crate::domain::account::ValidationState::Valid(*l)
        }
        (_, Some(i)) => crate::domain::account::ValidationState::Invalid(i.clone()),
        (None, None) => crate::domain::account::ValidationState::NeverValidated,
    };
    a.identity = v.identity.clone();
    a.confidence = v.confidence;
    a.lifecycle = v.lifecycle;
    a.issue = v.issue.clone();
    a.last_refresh_secs = Some(now);
    if let Some(u) = &v.usage {
        if u.windows.is_empty() {
            a.usage.freshness = u.freshness.clone();
        } else {
            a.usage = u.clone();
        }
    }
}

/// Refresh duration in virtual ms for an account (deterministic table).
pub fn refresh_duration_ms(a: &Account) -> i64 {
    let base = match a.provider {
        Provider::Anthropic => 800,
        Provider::OpenAi => 1_000,
        Provider::XAi => 900,
        Provider::OpenCode => 600,
        Provider::Amp => 650,
        Provider::Zai => 1_050,
        Provider::Moonshot => 350,
        Provider::MiniMax => 1_600,
    };
    base + (a.id.len() as i64 % 5) * 40
}
