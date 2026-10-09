use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::EvidenceReference;

pub(crate) const QUALIFICATION_REVIEW_SCHEMA: &str =
    "termrock-spec/visual-oracle-qualification-review-v1";
pub(crate) const GENERATION_ADMISSION_SCHEMA: &str = "termrock-spec/visual-generation-admission-v1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenerationBinding {
    pub(crate) id: String,
    pub(crate) tree_sha256: String,
    pub(crate) manifest_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OracleBinding {
    pub(crate) tag_ref: String,
    pub(crate) tag_object: String,
    pub(crate) tag_commit: String,
    pub(crate) source_snapshot_tree: String,
    pub(crate) source_inputs_sha256: String,
    pub(crate) build_environment_sha256: String,
    pub(crate) builder_receipt_sha256: String,
    pub(crate) executable_sha256: String,
    pub(crate) validator_toolchain: EvidenceReference,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SuiteBinding {
    pub(crate) revision: String,
    pub(crate) digest: String,
    pub(crate) case_set_digest: String,
    pub(crate) profile_digest: String,
    pub(crate) dependency_lock_sha256: String,
    pub(crate) tuiscotti_revision: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaseBinding {
    pub(crate) id: String,
    pub(crate) input_program_sha256: String,
    pub(crate) checkpoint_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdmissionBinding {
    pub(crate) generation: GenerationBinding,
    pub(crate) oracle: OracleBinding,
    pub(crate) capture: EvidenceReference,
    pub(crate) suite: SuiteBinding,
    pub(crate) case: CaseBinding,
    pub(crate) renderer_sha256: String,
}

/// This document is authored by the independent qualification reviewer. The
/// admission writer accepts only an explicit `approve` decision, a qualified
/// execution anchor, and exact bindings for the tag capture and generation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct QualificationReview {
    pub(crate) schema: String,
    pub(crate) review_id: String,
    pub(crate) reviewer: String,
    pub(crate) decision: String,
    pub(crate) execution_anchor_status: String,
    pub(crate) execution_anchor_evidence: Vec<EvidenceReference>,
    pub(crate) binding: AdmissionBinding,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenerationAdmissionReceipt {
    pub(crate) schema: String,
    pub(crate) decision: String,
    pub(crate) binding: AdmissionBinding,
    pub(crate) qualification_review: EvidenceReference,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedQualificationReview {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) review: QualificationReview,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedAdmissionReceipt {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) receipt: GenerationAdmissionReceipt,
}

pub(crate) fn load_qualification_review(
    path: Option<&Path>,
    expected_sha256: Option<&str>,
    expected_binding: &AdmissionBinding,
) -> Result<LoadedQualificationReview, String> {
    let path = path.ok_or_else(|| "qualification review path is required".to_string())?;
    let expected_sha256 =
        expected_sha256.ok_or_else(|| "qualification review SHA-256 is required".to_string())?;
    require_sha256("qualification review SHA-256", expected_sha256)?;
    if !path.is_absolute() {
        return Err("qualification review path must be absolute".to_string());
    }
    let path =
        crate::canonical_existing_regular_file_without_symlinks(path, "qualification review")?;
    require_external_to_suite(&path, "qualification review")?;
    let bytes = fs::read(&path)
        .map_err(|error| format!("read qualification review {}: {error}", path.display()))?;
    let sha256 = crate::sha256_bytes(&bytes);
    if sha256 != expected_sha256 {
        return Err(format!(
            "qualification review SHA-256 mismatch: expected {expected_sha256}, got {sha256}"
        ));
    }
    crate::reject_duplicate_json_keys(&bytes, "qualification review")?;
    let review: QualificationReview = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse qualification review: {error}"))?;
    validate_qualification_review(&review, expected_binding)?;
    for evidence in &review.execution_anchor_evidence {
        verify_evidence_reference(evidence, "qualification execution-anchor evidence")?;
    }
    let canonical = serde_json::to_vec(&review)
        .map_err(|error| format!("serialize qualification review: {error}"))?;
    if canonical != bytes {
        return Err("qualification review must use canonical compact JSON".to_string());
    }
    Ok(LoadedQualificationReview {
        path,
        sha256,
        review,
    })
}

pub(crate) fn validate_qualification_review(
    review: &QualificationReview,
    expected_binding: &AdmissionBinding,
) -> Result<(), String> {
    if review.schema != QUALIFICATION_REVIEW_SCHEMA {
        return Err("qualification review schema is unsupported".to_string());
    }
    if review.review_id.trim().is_empty() || review.reviewer.trim().is_empty() {
        return Err("qualification review identity must be nonempty".to_string());
    }
    if review.decision != "approve" {
        return Err(format!(
            "qualification review decision must be approve, got {:?}",
            review.decision
        ));
    }
    if review.execution_anchor_status != "qualified" || review.execution_anchor_evidence.is_empty()
    {
        return Err(
            "qualification review must bind a qualified execution anchor and its evidence"
                .to_string(),
        );
    }
    if review.reason.trim().is_empty() {
        return Err("qualification review must explain its decision".to_string());
    }
    if review.binding != *expected_binding {
        return Err(
            "qualification review does not match the exact capture/generation binding".to_string(),
        );
    }
    validate_binding(&review.binding)
}

pub(crate) fn new_admission_receipt(
    binding: AdmissionBinding,
    review: &LoadedQualificationReview,
) -> Result<GenerationAdmissionReceipt, String> {
    validate_qualification_review(&review.review, &binding)?;
    Ok(GenerationAdmissionReceipt {
        schema: GENERATION_ADMISSION_SCHEMA.to_string(),
        decision: "admitted".to_string(),
        binding,
        qualification_review: EvidenceReference {
            path: review.path.display().to_string(),
            sha256: review.sha256.clone(),
        },
    })
}

pub(crate) fn load_pinned_admission_receipt(
    path: &Path,
    expected_sha256: &str,
    expected_binding: &AdmissionBinding,
) -> Result<LoadedAdmissionReceipt, String> {
    let loaded = load_admission_envelope(path, expected_sha256)?;
    validate_admission_receipt(&loaded.receipt, expected_binding)?;
    let review = load_qualification_review(
        Some(Path::new(&loaded.receipt.qualification_review.path)),
        Some(&loaded.receipt.qualification_review.sha256),
        expected_binding,
    )?;
    if review.path.display().to_string() != loaded.receipt.qualification_review.path {
        return Err("qualification-review path is not in canonical form".to_string());
    }
    Ok(loaded)
}

pub(crate) fn load_admission_envelope(
    path: &Path,
    expected_sha256: &str,
) -> Result<LoadedAdmissionReceipt, String> {
    require_sha256("generation admission receipt SHA-256", expected_sha256)?;
    if !path.is_absolute() {
        return Err("generation admission receipt path must be absolute".to_string());
    }
    let path = crate::canonical_existing_regular_file_without_symlinks(
        path,
        "generation admission receipt",
    )?;
    require_external_to_suite(&path, "generation admission receipt")?;
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "read generation admission receipt {}: {error}",
            path.display()
        )
    })?;
    let sha256 = crate::sha256_bytes(&bytes);
    if sha256 != expected_sha256 {
        return Err(format!(
            "generation admission receipt SHA-256 mismatch: expected {expected_sha256}, got {sha256}"
        ));
    }
    crate::reject_duplicate_json_keys(&bytes, "generation admission receipt")?;
    let receipt: GenerationAdmissionReceipt = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse generation admission receipt: {error}"))?;
    if receipt.schema != GENERATION_ADMISSION_SCHEMA || receipt.decision != "admitted" {
        return Err("generation admission receipt is not an admitted v1 record".to_string());
    }
    let canonical = serde_json::to_vec(&receipt)
        .map_err(|error| format!("serialize generation admission receipt: {error}"))?;
    if canonical != bytes {
        return Err("generation admission receipt must use canonical compact JSON".to_string());
    }
    Ok(LoadedAdmissionReceipt {
        path,
        sha256,
        receipt,
    })
}

pub(crate) fn validate_admission_receipt(
    receipt: &GenerationAdmissionReceipt,
    expected_binding: &AdmissionBinding,
) -> Result<(), String> {
    if receipt.schema != GENERATION_ADMISSION_SCHEMA || receipt.decision != "admitted" {
        return Err("generation admission receipt is not an admitted v1 record".to_string());
    }
    if receipt.binding != *expected_binding {
        return Err("generation admission receipt does not match current suite inputs".to_string());
    }
    validate_binding(&receipt.binding)?;
    if !receipt.qualification_review.path.starts_with('/') {
        return Err("admission qualification-review path must be absolute".to_string());
    }
    require_sha256(
        "admission qualification-review SHA-256",
        &receipt.qualification_review.sha256,
    )
}

pub(crate) fn validate_binding(binding: &AdmissionBinding) -> Result<(), String> {
    if binding.generation.id.trim().is_empty() || binding.case.id != "HELP-HOLLA-004" {
        return Err(
            "admission binding must identify the Holla pilot generation and case".to_string(),
        );
    }
    for (name, value) in [
        (
            "generation tree SHA-256",
            binding.generation.tree_sha256.as_str(),
        ),
        (
            "generation manifest SHA-256",
            binding.generation.manifest_sha256.as_str(),
        ),
        (
            "tag source snapshot tree",
            binding.oracle.source_snapshot_tree.as_str(),
        ),
        (
            "tag source-input SHA-256",
            binding.oracle.source_inputs_sha256.as_str(),
        ),
        (
            "tag build-environment SHA-256",
            binding.oracle.build_environment_sha256.as_str(),
        ),
        (
            "tag builder-receipt SHA-256",
            binding.oracle.builder_receipt_sha256.as_str(),
        ),
        (
            "tag executable SHA-256",
            binding.oracle.executable_sha256.as_str(),
        ),
        (
            "validator toolchain SHA-256",
            binding.oracle.validator_toolchain.sha256.as_str(),
        ),
        ("capture receipt SHA-256", binding.capture.sha256.as_str()),
        ("suite digest", binding.suite.digest.as_str()),
        (
            "suite case-set digest",
            binding.suite.case_set_digest.as_str(),
        ),
        (
            "suite profile digest",
            binding.suite.profile_digest.as_str(),
        ),
        (
            "suite lock SHA-256",
            binding.suite.dependency_lock_sha256.as_str(),
        ),
        (
            "case input-program SHA-256",
            binding.case.input_program_sha256.as_str(),
        ),
        ("renderer SHA-256", binding.renderer_sha256.as_str()),
    ] {
        require_sha256(name, value)?;
    }
    if binding.oracle.source_snapshot_tree.len() != 40
        || !binding
            .oracle
            .source_snapshot_tree
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "tag source snapshot tree must be a lowercase 40-character Git object ID".to_string(),
        );
    }
    if binding.oracle.tag_ref != "refs/tags/visual-baseline"
        || binding.oracle.tag_object != "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
        || binding.oracle.tag_commit != "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
    {
        return Err(
            "admission binding does not identify the immutable visual-baseline tag".to_string(),
        );
    }
    if !Path::new(&binding.capture.path).is_absolute() {
        return Err("admission capture path must be absolute".to_string());
    }
    if !Path::new(&binding.oracle.validator_toolchain.path).is_absolute() {
        return Err("validator toolchain path must be absolute".to_string());
    }
    if binding.case.checkpoint_ids
        != vec![
            "00-boot".to_string(),
            "01-help".to_string(),
            "02-finder".to_string(),
            "03-help-again".to_string(),
        ]
    {
        return Err("admission binding has a noncanonical Holla checkpoint sequence".to_string());
    }
    if binding.suite.revision.trim().is_empty()
        || binding.suite.tuiscotti_revision.len() != 40
        || !binding
            .suite
            .tuiscotti_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("admission suite revisions must be nonempty".to_string());
    }
    Ok(())
}

fn require_sha256(name: &str, value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{name} must be lowercase 64-character SHA-256"));
    }
    Ok(())
}

fn verify_evidence_reference(reference: &EvidenceReference, label: &str) -> Result<(), String> {
    require_sha256(&format!("{label} SHA-256"), &reference.sha256)?;
    let supplied = Path::new(&reference.path);
    if !supplied.is_absolute() {
        return Err(format!("{label} path must be absolute"));
    }
    let path = crate::canonical_existing_regular_file_without_symlinks(supplied, label)?;
    require_external_to_suite(&path, label)?;
    if path.display().to_string() != reference.path {
        return Err(format!("{label} path is not canonical"));
    }
    if crate::sha256_file(&path)? != reference.sha256 {
        return Err(format!("{label} bytes do not match their pinned SHA-256"));
    }
    Ok(())
}

fn require_external_to_suite(path: &Path, label: &str) -> Result<(), String> {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checkout_root = crate::suite_checkout_root(package_root)?;
    if path.starts_with(&checkout_root) {
        return Err(format!("{label} must be stored outside the suite checkout"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> AdmissionBinding {
        let digest = "a".repeat(64);
        AdmissionBinding {
            generation: GenerationBinding {
                id: "holla-help-pilot".to_string(),
                tree_sha256: digest.clone(),
                manifest_sha256: digest.clone(),
            },
            oracle: OracleBinding {
                tag_ref: "refs/tags/visual-baseline".to_string(),
                tag_object: "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5".to_string(),
                tag_commit: "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b".to_string(),
                source_snapshot_tree: "b".repeat(40),
                source_inputs_sha256: digest.clone(),
                build_environment_sha256: digest.clone(),
                builder_receipt_sha256: digest.clone(),
                executable_sha256: digest.clone(),
                validator_toolchain: EvidenceReference {
                    path: "/private/tmp/validator-toolchain.json".to_string(),
                    sha256: digest.clone(),
                },
            },
            capture: EvidenceReference {
                path: "/private/tmp/capture.json".to_string(),
                sha256: digest.clone(),
            },
            suite: SuiteBinding {
                revision: "pilot-r1".to_string(),
                digest: digest.clone(),
                case_set_digest: digest.clone(),
                profile_digest: digest.clone(),
                dependency_lock_sha256: digest.clone(),
                tuiscotti_revision: "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274".to_string(),
            },
            case: CaseBinding {
                id: "HELP-HOLLA-004".to_string(),
                input_program_sha256: digest.clone(),
                checkpoint_ids: vec![
                    "00-boot".to_string(),
                    "01-help".to_string(),
                    "02-finder".to_string(),
                    "03-help-again".to_string(),
                ],
            },
            renderer_sha256: digest,
        }
    }

    fn approved_review(binding: AdmissionBinding) -> QualificationReview {
        QualificationReview {
            schema: QUALIFICATION_REVIEW_SCHEMA.to_string(),
            review_id: "review-1".to_string(),
            reviewer: "independent-reviewer".to_string(),
            decision: "approve".to_string(),
            execution_anchor_status: "qualified".to_string(),
            execution_anchor_evidence: vec![EvidenceReference {
                path: "/private/tmp/anchor.json".to_string(),
                sha256: "c".repeat(64),
            }],
            binding,
            reason: "all identity bindings reviewed".to_string(),
        }
    }

    #[test]
    fn qualification_review_requires_explicit_path_and_digest() {
        assert!(load_qualification_review(None, None, &binding()).is_err());
    }

    #[test]
    fn qualification_review_rejects_a_stale_raw_digest() {
        let path =
            std::env::temp_dir().join(format!("termrock-admission-review-{}", std::process::id()));
        let bytes = serde_json::to_vec(&approved_review(binding())).unwrap();
        fs::write(&path, &bytes).unwrap();
        let result = load_qualification_review(Some(&path), Some(&"d".repeat(64)), &binding());
        let _ = fs::remove_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn qualification_review_rejects_a_different_immutable_tag() {
        let mut review = approved_review(binding());
        review.binding.oracle.tag_commit = "9".repeat(40);
        assert!(validate_qualification_review(&review, &binding()).is_err());
    }

    #[test]
    fn qualification_review_rejects_a_different_capture_hash() {
        let mut review = approved_review(binding());
        review.binding.capture.sha256 = "e".repeat(64);
        assert!(validate_qualification_review(&review, &binding()).is_err());
    }

    #[test]
    fn qualification_review_rejects_a_nonapproval_decision() {
        let mut review = approved_review(binding());
        review.decision = "reject".to_string();
        assert!(validate_qualification_review(&review, &binding()).is_err());
    }
}
