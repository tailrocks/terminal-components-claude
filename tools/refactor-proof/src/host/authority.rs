//! Operator authority, campaign, and receipt parsing.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::json_util::{parse_json_bytes_strict, require_str, sha256_bytes};

const AUTHORITY_SCHEMA: &str = "tc-proof-host-authority/v1";
const CAMPAIGN_SCHEMA: &str = "tc-proof-host-campaign/v1";
const RECEIPT_SCHEMA: &str = "tc-proof-host-receipt/v1";

pub const AUTHORITY_ENV: &str = "TC_PROOF_AUTHORITY_FILE";

#[derive(Debug, Clone)]
pub struct Authority {
    pub campaign_path: PathBuf,
    pub campaign_sha256: String,
    pub accepted_receipts: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Campaign {
    pub path: PathBuf,
    pub repository: PathBuf,
    pub catalog_root: PathBuf,
    pub catalog_sha256: String,
    pub harness_receipt_sha256: String,
    pub run_id: String,
    pub trusted_overlay: TrustedOverlay,
    pub taskfmt: TaskfmtPin,
    pub tasks: HashMap<String, TaskSpec>,
}

#[derive(Debug, Clone)]
pub struct TrustedOverlay {
    pub scope_base: String,
    pub parent: String,
    pub paths: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CheckContextTemplate {
    pub check_id: String,
    pub schema: String,
    pub operation: String,
    pub lane: Option<String>,
    pub namespace: Option<String>,
    pub required_ids: Vec<String>,
    pub output_id: String,
}

#[derive(Debug, Clone)]
pub struct TaskfmtPin {
    pub executable: PathBuf,
    pub sha256: String,
    pub revision: String,
    pub fingerprint: String,
    pub config: PathBuf,
    pub config_sha256: String,
}

#[derive(Debug, Clone)]
pub struct TaskSpec {
    pub package: String,
    pub dependencies: Vec<DependencySpec>,
    pub check_context_templates: Vec<CheckContextTemplate>,
}

#[derive(Debug, Clone)]
pub struct DependencySpec {
    pub producer: String,
    pub product: String,
    pub receipt_sha256: String,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub path: PathBuf,
    pub digest: String,
    pub producer: String,
    pub product: String,
    pub source_commit: String,
    pub executable: Option<PathBuf>,
    pub executable_sha256: Option<String>,
}

pub fn load_authority() -> Result<Authority, String> {
    let path = std::env::var(AUTHORITY_ENV)
        .map_err(|_| format!("missing environment variable: {AUTHORITY_ENV}"))?;
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let value = parse_json_bytes_strict(&bytes)?;
    require_schema(&value, AUTHORITY_SCHEMA)?;
    let campaign_path = PathBuf::from(require_str(&value, "campaign")?);
    let campaign_sha256 = require_str(&value, "campaign_sha256")?;
    let accepted = value
        .get("accepted_receipts")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing or invalid accepted_receipts".to_string())?;
    let mut accepted_receipts = HashMap::new();
    for (digest, path_value) in accepted {
        let receipt_path = path_value
            .as_str()
            .ok_or_else(|| "accepted_receipts values must be strings".to_string())?;
        if accepted_receipts
            .insert(digest.clone(), PathBuf::from(receipt_path))
            .is_some()
        {
            return Err("duplicate accepted receipt digest".to_string());
        }
    }
    Ok(Authority {
        campaign_path,
        campaign_sha256,
        accepted_receipts,
    })
}

pub fn load_campaign(campaign_dir: &Path, authority: &Authority) -> Result<Campaign, String> {
    let campaign_path = campaign_dir.join("campaign.json");
    let bytes = fs::read(&campaign_path).map_err(|error| error.to_string())?;
    let digest = sha256_bytes(&bytes);
    if digest != authority.campaign_sha256 {
        return Err("campaign digest does not match authority".to_string());
    }
    if campaign_path != authority.campaign_path {
        return Err("campaign path does not match authority".to_string());
    }
    let value = parse_json_bytes_strict(&bytes)?;
    require_schema(&value, CAMPAIGN_SCHEMA)?;
    let repository = PathBuf::from(require_str(&value, "repository")?);
    let catalog_root = PathBuf::from(require_str(&value, "catalog_root")?);
    let catalog_sha256 = require_str(&value, "catalog_sha256")?;
    let harness_receipt_sha256 = require_str(&value, "harness_receipt_sha256")?;
    let run_id = require_str(&value, "run_id")?;
    let trusted_overlay = parse_trusted_overlay(
        value
            .get("trusted_overlay")
            .ok_or_else(|| "missing trusted_overlay".to_string())?,
    )?;
    let taskfmt_value = value
        .get("taskfmt")
        .ok_or_else(|| "missing taskfmt pin".to_string())?;
    let taskfmt = TaskfmtPin {
        executable: PathBuf::from(require_str(taskfmt_value, "executable")?),
        sha256: require_str(taskfmt_value, "sha256")?,
        revision: require_str(taskfmt_value, "revision")?,
        fingerprint: require_str(taskfmt_value, "fingerprint")?,
        config: PathBuf::from(require_str(taskfmt_value, "config")?),
        config_sha256: require_str(taskfmt_value, "config_sha256")?,
    };
    let tasks_object = value
        .get("tasks")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing or invalid tasks".to_string())?;
    let mut tasks = HashMap::new();
    for (task_id, task_value) in tasks_object {
        let package = require_str(task_value, "package")?;
        let dependencies = task_value
            .get("dependencies")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("missing dependencies for task {task_id}"))?
            .iter()
            .map(parse_dependency)
            .collect::<Result<Vec<_>, _>>()?;
        let templates = task_value
            .get("check_context_templates")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("missing check_context_templates for task {task_id}"))?
            .iter()
            .map(parse_check_context_template)
            .collect::<Result<Vec<_>, _>>()?;
        tasks.insert(
            task_id.clone(),
            TaskSpec {
                package,
                dependencies,
                check_context_templates: templates,
            },
        );
    }
    Ok(Campaign {
        path: campaign_path,
        repository,
        catalog_root,
        catalog_sha256,
        harness_receipt_sha256,
        run_id,
        trusted_overlay,
        taskfmt,
        tasks,
    })
}

fn parse_trusted_overlay(value: &Value) -> Result<TrustedOverlay, String> {
    let scope_base = require_str(value, "scope_base")?;
    let parent = require_str(value, "parent")?;
    let paths_object = value
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing trusted_overlay.paths".to_string())?;
    let mut paths = HashMap::new();
    for (path, digest_value) in paths_object {
        let digest = digest_value
            .as_str()
            .ok_or_else(|| format!("trusted overlay digest for {path} must be a string"))?;
        if paths.insert(path.clone(), digest.to_owned()).is_some() {
            return Err(format!("duplicate trusted overlay path: {path}"));
        }
    }
    Ok(TrustedOverlay {
        scope_base,
        parent,
        paths,
    })
}

fn parse_check_context_template(value: &Value) -> Result<CheckContextTemplate, String> {
    Ok(CheckContextTemplate {
        check_id: require_str(value, "check_id")?,
        schema: require_str(value, "schema")?,
        operation: require_str(value, "operation")?,
        lane: value.get("lane").and_then(Value::as_str).map(str::to_owned),
        namespace: value
            .get("namespace")
            .and_then(Value::as_str)
            .map(str::to_owned),
        required_ids: value
            .get("required_ids")
            .and_then(Value::as_array)
            .ok_or_else(|| "missing required_ids".to_string())?
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "required_ids entries must be strings".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        output_id: require_str(value, "output_id")?,
    })
}

fn parse_dependency(value: &Value) -> Result<DependencySpec, String> {
    Ok(DependencySpec {
        producer: require_str(value, "producer")?,
        product: require_str(value, "product")?,
        receipt_sha256: require_str(value, "receipt_sha256")?,
    })
}

pub fn load_receipt(path: &Path, authority: &Authority) -> Result<Receipt, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let digest = sha256_bytes(&bytes);
    let mapped = authority
        .accepted_receipts
        .get(&digest)
        .ok_or_else(|| "receipt digest is not accepted".to_string())?;
    if mapped != path {
        return Err("receipt path does not match authority binding".to_string());
    }
    let value = parse_json_bytes_strict(&bytes)?;
    require_schema(&value, RECEIPT_SCHEMA)?;
    Ok(Receipt {
        path: path.to_path_buf(),
        digest,
        producer: require_str(&value, "producer")?,
        product: require_str(&value, "product")?,
        source_commit: require_str(&value, "source_commit")?,
        executable: value
            .get("executable")
            .and_then(Value::as_str)
            .map(PathBuf::from),
        executable_sha256: value
            .get("executable_sha256")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn require_schema(value: &Value, expected: &str) -> Result<(), String> {
    match value.get("schema").and_then(Value::as_str) {
        Some(schema) if schema == expected => Ok(()),
        Some(other) => Err(format!("unsupported schema: {other}")),
        None => Err("missing schema".to_string()),
    }
}

pub fn load_preparation(run_dir: &Path) -> Result<(String, String), String> {
    let bytes = fs::read(run_dir.join("preparation.json")).map_err(|error| error.to_string())?;
    let value = parse_json_bytes_strict(&bytes)?;
    Ok((
        require_str(&value, "task")?,
        require_str(&value, "parent")?,
    ))
}

pub fn write_preparation(run_dir: &Path, task: &str, parent: &str) -> Result<(), String> {
    fs::create_dir_all(run_dir).map_err(|error| error.to_string())?;
    let document = serde_json::json!({ "task": task, "parent": parent });
    let text = serde_json::to_string(&document).map_err(|error| error.to_string())?;
    fs::write(run_dir.join("preparation.json"), format!("{text}\n")).map_err(|error| error.to_string())
}

